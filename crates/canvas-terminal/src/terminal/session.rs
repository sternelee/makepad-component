use makepad_widgets::*;
use rmux_sdk::{EnsureSession, PaneOutputStart, PaneOutputStream, Rmux, TerminalSizeSpec};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use super::state::TerminalState;

/// Shared rmux daemon connection + tokio runtime.
///
/// One runtime per app: makepad's main loop is synchronous, so all async
/// rmux work happens on a dedicated background thread.
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
/// The rmux daemon owns the PTY; this struct owns a named session + pane
/// handle, a background task that streams output into a shared
/// `TerminalState` (through the vte parser), and an input channel.
pub struct TerminalSession {
    pub command: String,
    /// Shared terminal grid state (written by the output stream task).
    pub state: Arc<Mutex<TerminalState>>,
    pane: rmux_sdk::Pane,
    /// Change notification channel: UI drains this to trigger redraws.
    rx: std::sync::mpsc::Receiver<Vec<u8>>,
    /// Keep the stream task alive.
    _stream: Arc<tokio::task::JoinHandle<()>>,
    /// Last requested grid size (to short-circuit redundant resizes).
    last_cols: std::sync::Mutex<(usize, usize)>,
}

impl TerminalSession {
    /// Spawn `command` in a new rmux session (a detached pane in the daemon).
    pub fn spawn(name: &str, command: &str, cols: usize, rows: usize) -> Result<Self, String> {
        let rmux = rmux()?;

        // Create-or-reuse: if a session with this name already exists in the
        // daemon, we attach to it. Use a unique suffix for fresh terminals so
        // each canvas terminal gets its own PTY.
        let session = runtime()
            .block_on(async {
                rmux.ensure_session(
                    EnsureSession::try_named(format!("{}-{}", name, std::process::id()))?
                        .create_only()
                        .detached(true)
                        .size(TerminalSizeSpec::new(cols as u16, rows as u16))
                        .argv(shell_command(command)),
                )
                .await
            })
            .map_err(|e| format!("ensure_session failed: {e}"))?;

        let pane = session.pane(0, 0);
        let state = Arc::new(Mutex::new(TerminalState::new(cols, rows)));

        // Subscribe to pane output; feed bytes into the vte parser.
        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        let state_reader = state.clone();
        let pane_reader = pane.clone();
        let stream_task = runtime().spawn(async move {
            let stream = match pane_reader
                .output_stream_starting_at(PaneOutputStart::Oldest)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    log!("rmux: output stream failed: {e}");
                    return;
                }
            };
            consume_stream(stream, state_reader.clone(), tx.clone()).await;
        });

        Ok(Self {
            command: command.to_string(),
            state,
            pane,
            rx,
            _stream: Arc::new(stream_task),
            last_cols: std::sync::Mutex::new((cols, rows)),
        })
    }

    /// Drain output notification channel; returns true if new bytes arrived.
    pub fn poll(&self) -> bool {
        let mut any = false;
        while let Ok(_chunk) = self.rx.try_recv() {
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

    /// Write a line of text + carriage return (unified command input).
    pub fn write_line(&self, text: &str) {
        let pane = self.pane.clone();
        let text = text.to_string();
        runtime().spawn(async move {
            let _ = pane.send_text(format!("{text}\r")).await;
        });
    }

    /// Resize the pane grid. Short-circuits when the size is unchanged so
    /// the per-frame draw sync does not spam the rmux daemon with SIGWINCH
    /// resizes (which would reflow the shell and look broken).
    pub fn resize(&self, cols: usize, rows: usize) {
        {
            let mut last = match self.last_cols.lock() {
                Ok(l) => l,
                Err(_) => return,
            };
            if *last == (cols, rows) {
                return;
            }
            *last = (cols, rows);
        }
        log!("rmux: resize to {cols}x{rows}");
        let pane = self.pane.clone();
        runtime().spawn(async move {
            let _ = pane
                .resize(TerminalSizeSpec::new(cols as u16, rows as u16))
                .await;
        });
        if let Ok(mut st) = self.state.lock() {
            st.resize(cols, rows);
        }
    }
}

/// Build the shell wrapper that launches `command`.
fn shell_command(command: &str) -> Vec<String> {
    // Run through the user's shell so PATH and env are normal.
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    vec![shell, "-lc".to_string(), command.to_string()]
}

/// Consume `PaneOutputStream`, feeding every byte chunk into the state parser
/// and forwarding a copy into the UI notification channel.
async fn consume_stream(
    mut stream: PaneOutputStream,
    state: Arc<Mutex<TerminalState>>,
    tx: std::sync::mpsc::Sender<Vec<u8>>,
) {
    loop {
        let next = tokio::time::timeout(Duration::from_secs(30), stream.next()).await;
        match next {
            Ok(Ok(Some(chunk))) => {
                if let rmux_sdk::PaneOutputChunk::Bytes { bytes, .. } = chunk {
                    if let Ok(mut st) = state.lock() {
                        st.feed(&bytes);
                    }
                    let _ = tx.send(bytes);
                } else {
                    log!("rmux: stream chunk (non-bytes)");
                }
            }
            Ok(Err(e)) => {
                log!("rmux: stream error: {e}");
                break;
            }
            Ok(Ok(None)) => break,
            // Timeout with no new output is normal for an idle pane:
            // keep the stream alive so later output still arrives.
            Err(_) => continue,
        }
    }
}
