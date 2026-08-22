use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use super::state::TerminalState;
use crate::ipc;

/// Shared tokio runtime. One per process.
static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

/// A boxed split stream half: read or write side of the daemon connection.
type ReadHalf = Box<dyn tokio::io::AsyncRead + Unpin + Send>;
type WriteHalf = Box<dyn tokio::io::AsyncWrite + Unpin + Send>;

/// Resolve the daemon endpoint and connect a byte stream to it.
async fn connect_stream() -> Result<(ReadHalf, WriteHalf), String> {
    let endpoint =
        rmux_ipc::endpoint_for_label(ipc::LABEL).map_err(|e| format!("resolve endpoint: {e}"))?;

    #[cfg(unix)]
    {
        let path = endpoint.into_path();
        let s = tokio::net::UnixStream::connect(&path)
            .await
            .map_err(|e| format!("connect {}: {e}", path.display()))?;
        let (r, w) = tokio::io::split(s);
        Ok((Box::new(r), Box::new(w)))
    }
    #[cfg(windows)]
    {
        let name = endpoint.as_pipe_name();
        let s = rmux_ipc::connect_windows_pipe(name)
            .await
            .map_err(|e| format!("connect pipe: {e}"))?;
        let (r, w) = tokio::io::split(s);
        Ok((Box::new(r), Box::new(w)))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = endpoint;
        Err("unsupported platform".into())
    }
}

/// Best-effort probe: can we reach a daemon right now?
async fn probe_connect() -> bool {
    connect_stream().await.is_ok()
}

/// Spawn the writer task that drains queued requests over the daemon's write
/// half. Returns the request sender (for callers to enqueue on) plus the task
/// handle so the session keeps it alive for its lifetime.
fn spawn_writer_task(
    mut w: WriteHalf,
) -> (
    tokio::sync::mpsc::Sender<ipc::Request>,
    tokio::task::JoinHandle<()>,
) {
    let (writer_tx, mut writer_rx) = tokio::sync::mpsc::channel::<ipc::Request>(128);
    let writer_task = runtime().spawn(async move {
        while let Some(req) = writer_rx.recv().await {
            if ipc::write_request(&mut w, &req).await.is_err() {
                break;
            }
        }
    });
    (writer_tx, writer_task)
}

/// Spawn the read task that streams `Output` frames into the local grid and
/// notifies the UI, exiting on `Ended`, EOF, or a read error.
fn spawn_read_task(
    mut r: ReadHalf,
    state: Arc<Mutex<TerminalState>>,
    notify: std::sync::mpsc::Sender<bool>,
) -> tokio::task::JoinHandle<()> {
    runtime().spawn(async move {
        loop {
            match ipc::read_frame(&mut r).await {
                Ok(Some(ipc::Frame::Output { bytes, .. })) => {
                    if let Ok(mut st) = state.lock() {
                        st.feed(&bytes);
                    }
                    let _ = notify.send(true);
                }
                Ok(Some(ipc::Frame::Ended { .. })) | Ok(None) | Err(_) => break,
                Ok(Some(_)) => {}
            }
        }
    })
}

/// Spawn the daemon as a detached process so it outlives the GUI.
///
/// The daemon is the same `canvas-terminal` binary re-invoked with
/// `--daemon`, so there is nothing extra to build or locate —
/// [`std::env::current_exe`] always points at it.
fn spawn_daemon_process() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("resolve current_exe: {e}"))?;
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                // New session so terminal signals (Ctrl-C, SIGHUP) delivered
                // to the GUI don't propagate to the daemon.
                let _ = libc::setsid();
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
        cmd.creation_flags(0x0000_0008 | 0x0000_0200);
    }

    cmd.spawn()
        .map_err(|e| format!("spawn daemon ({} --daemon): {e}", exe.display()))?;
    Ok(())
}

/// Ensure a daemon is reachable, launching it detached if necessary.
fn ensure_daemon() -> Result<(), String> {
    let rt = runtime();
    if rt.block_on(probe_connect()) {
        return Ok(());
    }
    spawn_daemon_process()?;
    // Wait up to ~5s for the daemon to bind its endpoint.
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if rt.block_on(probe_connect()) {
            return Ok(());
        }
    }
    Err("daemon did not become reachable".into())
}

/// A terminal session backed by the bundled PTY daemon (`canvas-terminal
/// --daemon`).
///
/// The daemon owns the PTY and child process; this struct holds the local
/// vte-parsed grid (`TerminalState`) and a streaming connection to the
/// daemon. `Output` frames from the daemon feed the local grid; `Write`/
/// `Resize`/`Kill` requests are sent the other way. Sessions persist in the
/// daemon across GUI restarts, so a later `attach` can replay scrollback and
/// continue streaming.
pub struct TerminalSession {
    /// Session name (used as the terminal card title and to re-attach later).
    pub name: String,
    /// The shell command the session runs (shown as the card subtitle).
    pub command: String,
    /// Local terminal grid (written by the read task).
    pub state: Arc<Mutex<TerminalState>>,
    pub(crate) session_id: u64,
    /// Outbound request channel (drained by the writer task).
    writer_tx: tokio::sync::mpsc::Sender<ipc::Request>,
    /// Notify the UI that new output arrived.
    notify: std::sync::mpsc::Sender<bool>,
    rx: std::sync::mpsc::Receiver<bool>,
    /// Keep the read task alive for the session's lifetime.
    _read_task: tokio::task::JoinHandle<()>,
    /// Keep the writer task alive for the session's lifetime.
    _writer_task: tokio::task::JoinHandle<()>,
}

impl TerminalSession {
    /// Spawn `command` in a new PTY session rooted at `cwd`.
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
        ensure_daemon()?;
        let rt = runtime();

        let (mut r, w) = rt.block_on(connect_stream())?;

        let (writer_tx, writer_task) = spawn_writer_task(w);

        // Build the shell argv and send Create.
        let argv = shell_command(command);
        let working_directory = cwd.map(expand_tilde);
        let create = ipc::Request::Create(ipc::CreateRequest {
            name: name.to_string(),
            cwd: working_directory,
            argv,
            cols: cols as u16,
            rows: rows as u16,
        });
        rt.block_on(async { writer_tx.send(create).await })
            .map_err(|_| "writer closed before Create".to_string())?;

        // Wait for the Create response (or error) on the read half.
        let state = Arc::new(Mutex::new(TerminalState::new(cols, rows)));
        let (notify, rx) = std::sync::mpsc::channel::<bool>();

        let session_id: u64 = rt.block_on(async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(ipc::Frame::Create(c))) => return Ok(c.session_id),
                    Ok(Some(ipc::Frame::Error(e))) => return Err(e.message),
                    Ok(Some(_)) => continue, // ignore unexpected early frames
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read Create response: {e}")),
                }
            }
        })?;

        // Read task: stream Output/Ended into the local grid.
        let read_task = spawn_read_task(r, Arc::clone(&state), notify.clone());

        Ok(Self {
            name: name.to_string(),
            command: command.to_string(),
            state,
            session_id,
            writer_tx,
            notify,
            rx,
            _read_task: read_task,
            _writer_task: writer_task,
        })
    }

    /// Re-attach to an existing named session in the daemon, replaying its
    /// buffered scrollback and continuing the live output stream.
    pub fn attach(name: &str, cols: usize, rows: usize) -> Result<Self, String> {
        ensure_daemon()?;
        let rt = runtime();
        let (mut r, w) = rt.block_on(connect_stream())?;

        let (writer_tx, writer_task) = spawn_writer_task(w);

        let attach = ipc::Request::Attach(ipc::AttachRequest {
            name: name.to_string(),
        });
        rt.block_on(async { writer_tx.send(attach).await })
            .map_err(|_| "writer closed before Attach".to_string())?;

        let state = Arc::new(Mutex::new(TerminalState::new(cols, rows)));
        let (notify, rx) = std::sync::mpsc::channel::<bool>();

        let (session_id, command, replay): (u64, String, Vec<u8>) = rt.block_on(async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(ipc::Frame::Attach {
                        session_id,
                        command,
                        replay,
                    })) => return Ok((session_id, command, replay)),
                    Ok(Some(ipc::Frame::Error(e))) => return Err(e.message),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read Attach response: {e}")),
                }
            }
        })?;

        // Seed the local grid with the replayed scrollback.
        if !replay.is_empty() {
            if let Ok(mut st) = state.lock() {
                st.feed(&replay);
            }
        }

        let read_task = spawn_read_task(r, Arc::clone(&state), notify.clone());

        Ok(Self {
            name: name.to_string(),
            command,
            state,
            session_id,
            writer_tx,
            notify,
            rx,
            _read_task: read_task,
            _writer_task: writer_task,
        })
    }

    /// List the sessions currently held by the daemon (live or exited).
    /// The GUI calls this at startup to re-attach to surviving sessions.
    pub fn list_sessions() -> Result<Vec<ipc::SessionInfo>, String> {
        ensure_daemon()?;
        let rt = runtime();
        let (mut r, w) = rt.block_on(connect_stream())?;

        let (writer_tx, _writer_task) = spawn_writer_task(w);
        rt.block_on(async { writer_tx.send(ipc::Request::List).await })
            .map_err(|_| "writer closed before List".to_string())?;

        rt.block_on(async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(ipc::Frame::List(l))) => return Ok(l.sessions),
                    Ok(Some(ipc::Frame::Error(e))) => return Err(e.message),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read List response: {e}")),
                }
            }
        })
    }

    /// Drain the output-notification channel; returns true if any output
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
        let _ = self.writer_tx.try_send(ipc::Request::Write {
            session_id: self.session_id,
            bytes: bytes.to_vec(),
        });
    }

    /// Write a line of text + carriage return.
    pub fn write_line(&self, text: &str) {
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(b'\r');
        let _ = self.writer_tx.try_send(ipc::Request::Write {
            session_id: self.session_id,
            bytes,
        });
    }

    /// Resize the pane grid.
    pub fn resize(&self, cols: usize, rows: usize) {
        // Reflow the local grid (wrap/truncate per alacritty semantics) so the
        // UI reacts instantly; the daemon applies the PTY resize in parallel.
        if let Ok(mut st) = self.state.lock() {
            st.resize(cols, rows);
        }
        let _ = self
            .writer_tx
            .try_send(ipc::Request::Resize(ipc::ResizeRequest {
                session_id: self.session_id,
                cols: cols as u16,
                rows: rows as u16,
            }));
        let _ = self.notify.send(true);
    }

    /// Fully terminate the session (kills the underlying PTY process).
    pub fn kill(&self) {
        let _ = self
            .writer_tx
            .try_send(ipc::Request::Kill(ipc::KillRequest {
                session_id: self.session_id,
            }));
    }

    /// Rename the session in the daemon so a later re-attach after a GUI
    /// restart finds it under `new_name`.
    pub fn rename(&self, new_name: &str) {
        let _ = self
            .writer_tx
            .try_send(ipc::Request::Rename(ipc::RenameRequest {
                session_id: self.session_id,
                name: new_name.to_string(),
            }));
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
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("/usr/local/bin"), "/usr/local/bin");
        assert_eq!(expand_tilde("~bob/x"), "~bob/x");
    }
}
