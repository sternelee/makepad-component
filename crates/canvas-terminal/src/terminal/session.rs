use makepad_widgets::*;
use rmux_sdk::{EnsureSession, Rmux, TerminalSizeSpec};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use super::state::TerminalState;

/// Shared rmux daemon connection + tokio runtime.
static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
static RMUX: std::sync::OnceLock<Rmux> = std::sync::OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

fn rmux() -> Result<&'static Rmux, String> {
    RMUX.get_or_init(|| {
        runtime()
            .block_on(async { Rmux::builder().connect_or_start().await })
            .map_err(|e| {
                log!("rmux: failed to connect/start daemon: {e}");
                std::process::exit(1);
            })
            .expect("rmux connect")
    });
    Ok(RMUX.get().unwrap())
}

/// A rmux-backed terminal session.
///
/// Uses `pane.snapshot()` polling instead of raw output-stream parsing. The
/// snapshot gives us the complete, already-rendered screen state (cells with
/// colors, cursor position) — exactly what the rmux pane displays. This
/// eliminates vte parsing, keeps glyphs/colors in sync with the actual PTY,
/// and makes resize instantly correct (snapshot reflects the new width).
pub struct TerminalSession {
    pub command: String,
    /// Shared terminal grid state (written by the snapshot poll task).
    pub state: Arc<Mutex<TerminalState>>,
    pane: rmux_sdk::Pane,
    /// Session handle kept so we can fully terminate the session on close.
    #[allow(dead_code)]
    session: rmux_sdk::Session,
    /// Set when `kill()` was called; the poll task checks this to exit.
    #[allow(dead_code)]
    killed: Arc<std::sync::atomic::AtomicBool>,
    /// Set when the snapshot poll task reported new data.
    rx: std::sync::mpsc::Receiver<bool>,
    /// Notify the UI (fetch_history etc.) to redraw.
    notify: std::sync::mpsc::Sender<bool>,
    /// Keep the poll task alive.
    _poll: Arc<tokio::task::JoinHandle<()>>,
    /// Last requested grid size (to short-circuit redundant resizes).
    last_size: std::sync::Mutex<(usize, usize)>,
    window: rmux_sdk::Window,
}

impl TerminalSession {
    /// Spawn `command` in a new rmux session rooted at `cwd`.
    ///
    /// `cwd == None` falls back to the current process directory; `~` is
    /// expanded to the user's home directory.
    pub fn spawn(
        name: &str,
        command: &str,
        cwd: Option<&str>,
        cols: usize,
        rows: usize,
    ) -> Result<Self, String> {
        let rmux = rmux()?;

        // Resolve the working directory: explicit value (with `~` expanded),
        // else the current process directory.
        let working_directory = cwd.map(expand_tilde).unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

        let session = runtime()
            .block_on(async {
                rmux.ensure_session(
                    EnsureSession::try_named(format!("{}-{}", name, std::process::id()))?
                        .create_only()
                        .detached(true)
                        .size(TerminalSizeSpec::new(cols as u16, rows as u16))
                        .working_directory(working_directory.clone())
                        .argv(shell_command(command)),
                )
                .await
            })
            .map_err(|e| format!("ensure_session failed: {e}"))?;

        let window = session.window(0);
        let pane = session.pane(0, 0);
        let state = Arc::new(Mutex::new(TerminalState::new(cols, rows)));

        // Poll pane.snapshot() at ~30fps and push the result into the shared
        // TerminalState. The rmux daemon owns the PTY; we parse its raw
        // output stream locally with vte into the grid (like alacritty), so
        // colors, wide chars, scrollback, cursor are all maintained here.
        let killed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel::<bool>();
        let notify = tx.clone();
        let state_poll = state.clone();
        let pane_poll = pane.clone();
        let killed_poll = killed.clone();
        let poll_task = runtime().spawn(async move {
            // Subscribe to the raw pane output stream (oldest => replay).
            let stream = match pane_poll
                .output_stream_starting_at(rmux_sdk::PaneOutputStart::Oldest)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    log!("rmux: output stream failed: {e}");
                    return;
                }
            };
            let mut stream = stream;
            loop {
                if killed_poll.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let next = tokio::time::timeout(Duration::from_secs(30), stream.next()).await;
                match next {
                    Ok(Ok(Some(chunk))) => {
                        if let rmux_sdk::PaneOutputChunk::Bytes { bytes, .. } = chunk {
                            if let Ok(mut st) = state_poll.lock() {
                                st.feed(&bytes);
                            }
                            let _ = tx.send(true);
                        }
                    }
                    Ok(Err(e)) => {
                        log!("rmux: stream error: {e}");
                        break;
                    }
                    Ok(Ok(None)) => break,
                    // Timeout with no output is normal for an idle pane:
                    // keep the stream alive.
                    Err(_) => continue,
                }
            }
        });

        Ok(Self {
            command: command.to_string(),
            state,
            pane,
            session,
            killed,
            rx,
            notify,
            _poll: Arc::new(poll_task),
            last_size: std::sync::Mutex::new((cols, rows)),
            window,
        })
    }

    /// Drain the snapshot-notification channel; returns true if any snapshot
    /// arrived since the last check.
    pub fn poll(&self) -> bool {
        let mut any = false;
        while self.rx.try_recv().is_ok() {
            any = true;
        }
        any
    }

    /// Write raw bytes into the pane.
    pub fn write_bytes(&self, bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes).into_owned();
        let pane = self.pane.clone();
        runtime().spawn(async move {
            let _ = pane.send_text(text).await;
        });
    }

    /// Write a line of text + carriage return.
    pub fn write_line(&self, text: &str) {
        let pane = self.pane.clone();
        let text = text.to_string();
        runtime().spawn(async move {
            let _ = pane.send_text(format!("{text}\r")).await;
        });
    }

    /// Resize the pane grid.
    pub fn resize(&self, cols: usize, rows: usize) {
        {
            let mut last = match self.last_size.lock() {
                Ok(l) => l,
                Err(_) => return,
            };
            if *last == (cols, rows) {
                return;
            }
            *last = (cols, rows);
        }
        let pane = self.pane.clone();
        let window = self.window.clone();
        runtime().spawn(async move {
            match pane
                .resize(TerminalSizeSpec::new(cols as u16, rows as u16))
                .await
            {
                Ok(_) => log!("rmux: pane resize ok {cols}x{rows}"),
                Err(e) => log!("rmux: pane resize FAILED {cols}x{rows}: {e}"),
            }
            match window.resize(Some(cols as u16), Some(rows as u16)).await {
                Ok(_) => log!("rmux: window resize ok {cols}x{rows}"),
                Err(e) => log!("rmux: window resize FAILED {cols}x{rows}: {e}"),
            }
        });
        // Reflow the local grid (wrap/truncate per alacritty semantics).
        if let Ok(mut st) = self.state.lock() {
            st.resize(cols, rows);
        }
        let _ = self.notify.send(true);
    }

    /// Fully terminate the rmux session (kills the underlying PTY process).
    #[allow(dead_code)]
    pub fn kill(&self) {
        self.killed.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = runtime().block_on(self.session.kill());
    }

    /// Scroll the local scrollback viewport by `delta` lines (positive = up).
    pub fn scroll_display(&self, delta: i32) {
        if let Ok(mut st) = self.state.lock() {
            st.scroll_display(delta);
        }
        let _ = self.notify.send(true);
    }
}

/// Build the shell wrapper that launches `command`.
fn shell_command(command: &str) -> Vec<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    vec![shell, "-lc".to_string(), command.to_string()]
}

/// Expand a leading `~` to the user's home directory (like the shell does).
fn expand_tilde(path: &str) -> String {
    match path.strip_prefix("~") {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => {
            if let Some(home) = std::env::var_os("HOME") {
                let home = std::path::Path::new(&home);
                let rest = rest.trim_start_matches('/');
                // Bare `~` returns the home dir as-is (no trailing slash).
                return if rest.is_empty() {
                    home.display().to_string()
                } else {
                    home.join(rest).display().to_string()
                };
            }
            path.to_string()
        }
        _ => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::expand_tilde;

    #[test]
    fn expands_tilde_to_home() {
        let home = std::env::var("HOME").unwrap();
        let out = expand_tilde("~/projects/foo");
        assert_eq!(out, format!("{home}/projects/foo"));
        // Bare `~` maps to home itself.
        assert_eq!(expand_tilde("~"), home);
        // No tilde -> unchanged.
        assert_eq!(expand_tilde("/usr/local/bin"), "/usr/local/bin");
        // ~user forms are left alone (no passwd lookup).
        assert_eq!(expand_tilde("~bob/x"), "~bob/x");
    }
}
