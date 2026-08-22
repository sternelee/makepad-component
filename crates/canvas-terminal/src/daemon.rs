//! Bundled PTY daemon runtime, selected by `canvas-terminal --daemon`.
//!
//! Holds terminal sessions (PTY + child process) out-of-process so they
//! survive GUI restarts. The GUI connects over a local `rmux-ipc` endpoint
//! (Unix domain socket / Windows named pipe) and speaks the protocol in
//! [`crate::ipc`].
//!
//! Lifecycle: the GUI spawns `canvas-terminal --daemon` detached on first use
//! and reconnects on subsequent launches. The daemon keeps running as long as
//! it has live sessions; `Kill` requests (or the child exiting naturally) end
//! a session. Keeping this in the same binary as the GUI means there is only
//! one artifact to build/distribute, and the app is self-contained — no
//! system-installed rmux, no separate daemon binary to locate.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use rmux_ipc::{LocalEndpoint, LocalListener};
use rmux_pty::{ChildCommand, PtyChild, PtyIo, PtyMaster, Signal, TerminalSize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

use crate::ipc;

/// Ring buffer capacity per session (256 KiB of scrollback replay).
const RING_CAP: usize = 256 * 1024;
/// Per-subscriber output channel depth.
const SUB_CHAN: usize = 512;

type Sessions = Arc<Mutex<HashMap<u64, Session>>>;

struct Session {
    id: u64,
    name: String,
    /// The shell command the session was spawned with (the last argv
    /// element, e.g. `zsh` for `[zsh, -lc, zsh]`), used as the terminal card
    /// subtitle after a GUI restart re-attaches.
    command: String,
    master: PtyMaster,
    child: PtyChild,
    cols: u16,
    rows: u16,
    alive: bool,
    /// Bounded scrollback ring for `Attach` replay.
    ring: VecDeque<u8>,
    /// Subscribed client writers (one per attached connection).
    subscribers: Vec<mpsc::Sender<ipc::Frame>>,
}

impl Session {
    fn push_ring(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.ring.push_back(b);
        }
        while self.ring.len() > RING_CAP {
            self.ring.pop_front();
        }
    }

    fn snapshot_ring(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.ring.len());
        out.extend(self.ring.iter().copied());
        out
    }
}

/// Daemon entry point. Builds its own multi-thread tokio runtime and blocks
/// until the process is terminated.
pub fn run() -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    runtime.block_on(daemon_main())
}

async fn daemon_main() -> io::Result<()> {
    let endpoint = rmux_ipc::endpoint_for_label(ipc::LABEL)?;
    let listener = bind_listener(&endpoint)?;
    eprintln!(
        "canvas-terminal daemon: listening at {}",
        endpoint_display(&endpoint)
    );

    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));
    let next_id = Arc::new(AtomicU64::new(1));

    loop {
        match listener.accept().await {
            Ok((stream, _peer)) => {
                let sessions = Arc::clone(&sessions);
                let next_id = Arc::clone(&next_id);
                tokio::spawn(async move {
                    handle_client(stream, sessions, next_id).await;
                });
            }
            Err(e) => eprintln!("canvas-terminal daemon: accept error: {e}"),
        }
    }
}

/// Bind the listener, clearing a stale socket file on Unix when no daemon is
/// actually listening there.
fn bind_listener(endpoint: &LocalEndpoint) -> io::Result<LocalListener> {
    match LocalListener::bind(endpoint) {
        Ok(l) => Ok(l),
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            #[cfg(unix)]
            {
                if !probe_connect(endpoint) {
                    let _ = std::fs::remove_file(endpoint.as_path());
                    return LocalListener::bind(endpoint);
                }
            }
            Err(e)
        }
        Err(e) => Err(e),
    }
}

#[cfg(unix)]
fn probe_connect(endpoint: &LocalEndpoint) -> bool {
    use std::os::unix::net::UnixStream;
    UnixStream::connect(endpoint.as_path()).is_ok()
}

fn endpoint_display(endpoint: &LocalEndpoint) -> String {
    #[cfg(unix)]
    {
        endpoint.as_path().display().to_string()
    }
    #[cfg(windows)]
    {
        endpoint.as_pipe_name().to_string_lossy().into_owned()
    }
    #[cfg(not(any(unix, windows)))]
    {
        String::from("(unknown endpoint)")
    }
}

/// Per-connection handler. One connection drives one session: it issues
/// `Create`/`Attach`, then `Write`/`Resize`/`Kill` while receiving `Output`
/// frames streamed back over the same connection.
async fn handle_client<S>(stream: S, sessions: Sessions, next_id: Arc<AtomicU64>)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut r, mut w) = tokio::io::split(stream);

    // Writer task: drains the client's outbound channel into the stream.
    let (tx, mut rx) = mpsc::channel::<ipc::Frame>(SUB_CHAN);
    let writer = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if ipc::write_frame(&mut w, &frame).await.is_err() {
                break;
            }
        }
    });

    loop {
        match ipc::read_request(&mut r).await {
            Ok(Some(req)) => handle_request(req, &sessions, &next_id, &tx).await,
            Ok(None) => break,
            Err(e) => {
                eprintln!("canvas-terminal daemon: read error: {e}");
                break;
            }
        }
    }

    // Dropping `tx` ends the writer and disconnects the subscriber cloned
    // into the session; the next fanout prunes the stale entry.
    drop(tx);
    let _ = writer.await;
}

async fn handle_request(
    req: ipc::Request,
    sessions: &Sessions,
    next_id: &Arc<AtomicU64>,
    tx: &mpsc::Sender<ipc::Frame>,
) {
    match req {
        ipc::Request::Create(c) => match spawn_session(&c, sessions, next_id, tx.clone()) {
            Ok(id) => {
                let _ = tx
                    .send(ipc::Frame::Create(ipc::CreateResponse { session_id: id }))
                    .await;
            }
            Err(msg) => {
                let _ = tx
                    .send(ipc::Frame::Error(ipc::ErrorResponse { message: msg }))
                    .await;
            }
        },
        ipc::Request::Attach(a) => {
            // Decide under the lock (no await while holding the guard), then
            // send outside it. This avoids racing the reader thread's
            // end-of-session drain and refuses a dead session up front.
            enum AttachOutcome {
                Ok {
                    id: u64,
                    command: String,
                    replay: Vec<u8>,
                },
                Err(String),
            }
            let outcome = {
                let Ok(mut map) = sessions.lock() else { return };
                match map.values_mut().find(|s| s.name == a.name) {
                    None => AttachOutcome::Err(format!("no session named '{}'", a.name)),
                    Some(s) if !s.alive => {
                        AttachOutcome::Err(format!("session '{}' has exited", a.name))
                    }
                    Some(s) => {
                        let replay = s.snapshot_ring();
                        s.subscribers.push(tx.clone());
                        AttachOutcome::Ok {
                            id: s.id,
                            command: s.command.clone(),
                            replay,
                        }
                    }
                }
            };
            match outcome {
                AttachOutcome::Ok {
                    id,
                    command,
                    replay,
                } => {
                    let _ = tx
                        .send(ipc::Frame::Attach {
                            session_id: id,
                            command,
                            replay,
                        })
                        .await;
                }
                AttachOutcome::Err(message) => {
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                        .await;
                }
            }
        }
        ipc::Request::Write { session_id, bytes } => {
            if let Some(Err(e)) = with_session(sessions, session_id, |s| s.master.write_all(&bytes))
            {
                let _ = tx
                    .send(ipc::Frame::Error(ipc::ErrorResponse {
                        message: format!("write failed: {e}"),
                    }))
                    .await;
            }
        }
        ipc::Request::Resize(r) => {
            let res = with_session_mut::<_, rmux_pty::PtyError>(sessions, r.session_id, |s| {
                s.master.resize(TerminalSize::new(r.cols, r.rows))?;
                s.cols = r.cols;
                s.rows = r.rows;
                Ok(())
            });
            if let Some(Err(e)) = res {
                let _ = tx
                    .send(ipc::Frame::Error(ipc::ErrorResponse {
                        message: format!("resize failed: {e}"),
                    }))
                    .await;
            }
        }
        ipc::Request::Kill(k) => {
            with_session_mut::<_, io::Error>(sessions, k.session_id, |s| {
                let _ = s.child.kill(Signal::KILL);
                s.alive = false;
                Ok(())
            });
            // A killed session is gone for good: drop it from the table so
            // `List`/`Attach` can't resurrect it (or collide with a future
            // session of the same name).
            let _ = remove_session(sessions, k.session_id);
        }
        ipc::Request::Rename(r) => {
            let outcome = {
                let Ok(mut map) = sessions.lock() else { return };
                // Free any other live session already using the new name.
                map.retain(|id, other| {
                    !(other.alive && *id != r.session_id && other.name == r.name)
                });
                match map.get_mut(&r.session_id) {
                    // Refuse to rename a dead session: it can't be attached
                    // anyway, and its name is freed on the next `Create`.
                    Some(s) if s.alive => {
                        s.name = r.name.clone();
                        Ok(())
                    }
                    _ => Err(format!("no live session {}", r.session_id)),
                }
            };
            if let Err(message) = outcome {
                let _ = tx
                    .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                    .await;
            }
        }
        ipc::Request::List => {
            let infos = {
                let Ok(map) = sessions.lock() else { return };
                map.values()
                    .map(|s| ipc::SessionInfo {
                        name: s.name.clone(),
                        session_id: s.id,
                        alive: s.alive,
                    })
                    .collect::<Vec<_>>()
            };
            let _ = tx
                .send(ipc::Frame::List(ipc::ListResponse { sessions: infos }))
                .await;
        }
    }
}

/// Spawns a PTY child, stores the session, starts its reader thread, and
/// registers the calling connection as the first subscriber.
fn spawn_session(
    c: &ipc::CreateRequest,
    sessions: &Sessions,
    next_id: &Arc<AtomicU64>,
    subscriber: mpsc::Sender<ipc::Frame>,
) -> Result<u64, String> {
    if c.argv.is_empty() {
        return Err("argv must not be empty".into());
    }
    let mut cmd = ChildCommand::new(&c.argv[0]);
    for a in &c.argv[1..] {
        cmd = cmd.arg(a);
    }
    if let Some(cwd) = &c.cwd {
        cmd = cmd.current_dir(cwd);
    }
    cmd = cmd.size(TerminalSize::new(c.cols, c.rows));

    let spawned = cmd.spawn().map_err(|e| format!("spawn failed: {e}"))?;
    let (master, child) = spawned.into_parts();
    let reader_io = master
        .try_clone_io()
        .map_err(|e| format!("clone io: {e}"))?;

    let id = next_id.fetch_add(1, Ordering::SeqCst);

    let session = Session {
        id,
        name: c.name.clone(),
        command: c.argv.last().cloned().unwrap_or_default(),
        master,
        child,
        cols: c.cols,
        rows: c.rows,
        alive: true,
        ring: VecDeque::with_capacity(RING_CAP),
        subscribers: vec![subscriber],
    };

    {
        let Ok(mut map) = sessions.lock() else {
            return Err("session table poisoned".into());
        };
        // Session names are unique among live sessions; a previous session
        // with this name (already exited) must not shadow the new one on a
        // later `Attach`. Dead sessions also hold a 256 KiB ring, so this
        // keeps the table bounded by the number of live PTYs.
        map.retain(|_, s| s.alive);
        map.insert(id, session);
    }

    spawn_pty_reader(id, reader_io, Arc::clone(sessions));
    Ok(id)
}

/// Dedicated OS thread that reads PTY output, appends it to the session ring,
/// and fans `Output` frames out to subscribers. On EOF/error it marks the
/// session dead, reaps the child, and emits `Ended`.
fn spawn_pty_reader(id: u64, io: PtyIo, sessions: Sessions) {
    std::thread::Builder::new()
        .name(format!("ct-pty-{id}"))
        .spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match io.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        end_session(&sessions, id);
                        return;
                    }
                    Ok(n) => fanout_output(&sessions, id, buf[..n].to_vec()),
                }
            }
        })
        .expect("spawn pty reader thread");
}

/// Append bytes to the session ring and fan `Output` to subscribers.
fn fanout_output(sessions: &Sessions, id: u64, bytes: Vec<u8>) {
    let subs = {
        let Ok(mut map) = sessions.lock() else { return };
        let Some(s) = map.get_mut(&id) else { return };
        s.push_ring(&bytes);
        s.subscribers.clone()
    };

    let mut keep: Vec<mpsc::Sender<ipc::Frame>> = Vec::with_capacity(subs.len());
    for st in subs {
        match st.try_send(ipc::Frame::Output {
            session_id: id,
            bytes: bytes.clone(),
        }) {
            Ok(()) => keep.push(st),
            // Channel full: keep the subscriber but drop this frame.
            Err(mpsc::error::TrySendError::Full(_)) => keep.push(st),
            // Subscriber gone: prune it.
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
    }

    if let Ok(mut map) = sessions.lock() {
        if let Some(s) = map.get_mut(&id) {
            s.subscribers = keep;
        }
    }
}

/// Mark a session dead, reap its child, and emit `Ended` to subscribers.
fn end_session(sessions: &Sessions, id: u64) {
    let subs = {
        let Ok(mut map) = sessions.lock() else { return };
        let Some(s) = map.get_mut(&id) else { return };
        s.alive = false;
        let _ = s.child.try_wait(); // reap
        std::mem::take(&mut s.subscribers)
    };
    for st in subs {
        let _ = st.try_send(ipc::Frame::Ended { session_id: id });
    }
}

/// Remove a session from the table entirely (used by `Kill`).
fn remove_session(sessions: &Sessions, id: u64) -> Option<Session> {
    let Ok(mut map) = sessions.lock() else {
        return None;
    };
    map.remove(&id)
}

fn with_session<R>(sessions: &Sessions, id: u64, f: impl FnOnce(&Session) -> R) -> Option<R> {
    let Ok(map) = sessions.lock() else {
        return None;
    };
    map.get(&id).map(f)
}

fn with_session_mut<R, E>(
    sessions: &Sessions,
    id: u64,
    f: impl FnOnce(&mut Session) -> Result<R, E>,
) -> Option<Result<R, E>> {
    let Ok(mut map) = sessions.lock() else {
        return None;
    };
    map.get_mut(&id).map(f)
}
