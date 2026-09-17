//! Bundled PTY daemon runtime, selected by `canvas-terminal --daemon`.
//!
//! Holds terminal sessions (PTY + child process) out-of-process so they
//! survive GUI restarts. The GUI connects
//! over a local `rmux-ipc` endpoint (Unix domain socket / Windows named pipe)
//! and speaks the protocol in [`crate::ipc`].
//!
//! Lifecycle: the GUI spawns `canvas-terminal --daemon` detached on first use
//! and reconnects on subsequent launches. The daemon keeps running as long as
//! it has live sessions; `Kill` requests (or the child exiting naturally) end
//! a session. Keeping this in the same binary as the GUI means there is only
//! one artifact to build/distribute, and the app is self-contained — no
//! system-installed rmux, no separate daemon binary to locate.
//!
//! A session has two interpretations of the same PTY stream: the terminal
//! grid (always fed, the raw ring is the authority for replays) and, when the
//! chat parser is on, a sequence of normalized chat events parsed from the
//! hosted CLI's JSONL. Chat events carry sequence numbers, so a client that
//! misses one (a full subscriber channel drops frames) can repair the hole
//! with a `ChatSync` — the daemon's event ring is the authority and the live
//! stream is only a tail of it.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::chat::event::Sequenced;
use crate::chat::{ChatMode, CliAdapter};
use crate::daemon_persist::{SavedRegistry, SavedSession};
use rmux_ipc::{LocalEndpoint, LocalListener};
use rmux_pty::{ChildCommand, PtyChild, PtyIo, PtyMaster, Signal, TerminalSize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

use crate::ipc;

/// Ring buffer capacity per session (256 KiB of scrollback replay).
const RING_CAP: usize = 256 * 1024;
/// Per-subscriber output channel depth.
const SUB_CHAN: usize = 512;
const AGENT_SUB_CHAN: usize = 4096;

/// Chat view state layered on a terminal session.
///
/// When present, the PTY reader also runs every completed output line
/// through the CLI adapter and fans normalized events out as
/// `Frame::ChatEvent`. The raw ring keeps receiving everything, so flipping
/// back to the grid view loses nothing.
struct ChatState {
    adapter: CliAdapter,
    /// Whether the live stream is being parsed. Flipping the view to grid
    /// pauses parsing but keeps the ring, so flipping back restores the
    /// transcript instead of starting blank.
    on: bool,
    /// Partial line across PTY reads.
    line_buf: String,
    seq: u64,
    /// Bounded event ring for `ChatSync` replay.
    events: std::collections::VecDeque<Sequenced>,
    /// The CLI's own session id, when it reported one (used to resume).
    cli_session_id: Option<String>,
    /// Whether the hosted CLI is mid-turn, folded from the same events
    /// `chat/fold.rs::ChatCardState` folds client-side (`UserMessage` ->
    /// `true`, `TurnDone`/`Exited` -> `false`). The daemon keeps its own
    /// copy so `AgentWait`/`AgentList` work without any client attached.
    busy: bool,
}

impl ChatState {
    const RING: usize = 2_000;
    /// A single line longer than this is not JSON worth parsing (the CLIs
    /// emit large but bounded records; anything past this is binary noise).
    const MAX_LINE: usize = 4 * 1024 * 1024;

    fn new(adapter: CliAdapter) -> Self {
        Self {
            adapter,
            on: true,
            line_buf: String::new(),
            seq: 0,
            events: std::collections::VecDeque::new(),
            cli_session_id: None,
            busy: false,
        }
    }

    /// Feed raw PTY bytes; returns the events completed by this chunk, in
    /// order, already sequenced.
    fn feed(&mut self, bytes: &[u8]) -> Vec<Sequenced> {
        let mut out = Vec::new();
        for byte in bytes {
            match byte {
                b'\n' => {
                    let line = std::mem::take(&mut self.line_buf);
                    for event in self.adapter.parse_line(&line) {
                        self.seq += 1;
                        match &event {
                            crate::chat::ChatEvent::SessionInfo {
                                session_id: Some(id),
                                ..
                            } => {
                                self.cli_session_id = Some(id.clone());
                            }
                            crate::chat::ChatEvent::UserMessage { .. } => {
                                self.busy = true;
                            }
                            crate::chat::ChatEvent::TurnDone { .. }
                            | crate::chat::ChatEvent::Exited => {
                                self.busy = false;
                            }
                            _ => {}
                        }
                        let item = Sequenced {
                            seq: self.seq,
                            event,
                        };
                        self.events.push_back(item.clone());
                        if self.events.len() > Self::RING {
                            self.events.pop_front();
                        }
                        out.push(item);
                    }
                }
                b'\r' => {}
                other => {
                    if self.line_buf.len() < Self::MAX_LINE {
                        self.line_buf.push(*other as char);
                    }
                }
            }
        }
        out
    }
}

/// The session table. Sessions are removed when their child exits (see
/// [`end_session`]), so it is bounded by live PTYs.
type Sessions = Arc<Mutex<HashMap<u64, Session>>>;

/// The daemon's own record of session *identity* (argv/cwd/chat provider),
/// persisted to disk so it survives a daemon restart — unlike `Sessions`,
/// which is purely in-memory. See `crate::daemon_persist`.
///
/// Carries its own resolved path (label-scoped; see
/// `daemon_persist::path_for_label`) rather than always writing
/// `daemon_persist::default_path()`, so an isolated daemon (tests, or a
/// second instance on a non-default label) can never race on or clobber the
/// default installation's registry file.
struct KnownRegistry {
    registry: SavedRegistry,
    path: std::path::PathBuf,
}
type Known = Arc<Mutex<KnownRegistry>>;

/// Save `known` to disk, best-effort. Called after every request that
/// changes a session's identity; failures are logged, never fatal (the
/// registry is a resume cache, not a document).
fn persist_known(known: &Known) {
    let (snapshot, path) = match known.lock() {
        Ok(guard) => (guard.registry.clone(), guard.path.clone()),
        Err(_) => return,
    };
    if let Err(e) = snapshot.save(&path) {
        eprintln!("canvas-terminal daemon: failed to persist session registry: {e}");
    }
}

/// Outstanding `AgentWait` calls, keyed by session name (stable across a
/// `Resume` respawn, unlike the session id). Resolved event-driven — from
/// the PTY reader thread when `busy` changes, and from `end_session` when
/// the session exits — never polled.
type Waiters = Arc<Mutex<HashMap<String, Vec<(Vec<ipc::AgentWaitState>, tokio::sync::oneshot::Sender<ipc::AgentStatusInfo>)>>>>;

/// Snapshot one session's agent status for `AgentWait`/`AgentList`.
fn agent_status_info(s: &Session) -> ipc::AgentStatusInfo {
    ipc::AgentStatusInfo {
        name: s.name.clone(),
        alive: s.alive,
        busy: s.chat.as_ref().is_some_and(|c| c.busy),
        cli: s.chat.as_ref().map(|c| c.adapter.name().to_owned()),
        cli_session_id: s.chat.as_ref().and_then(|c| c.cli_session_id.clone()),
        timed_out: false,
    }
}

/// Resolve every outstanding waiter for `name` whose `until` matches
/// `info.busy` (or unconditionally, when `force` is set — used for "the
/// session just exited, it will never change again").
fn resolve_waiters(waiters: &Waiters, name: &str, info: &ipc::AgentStatusInfo, force: bool) {
    let Ok(mut map) = waiters.lock() else { return };
    let Some(list) = map.remove(name) else { return };
    let mut remaining = Vec::with_capacity(list.len());
    for (until, sender) in list {
        let matches = force
            || until.iter().any(|want| match want {
                ipc::AgentWaitState::Idle => !info.busy,
                ipc::AgentWaitState::Working => info.busy,
            });
        if matches {
            let _ = sender.send(info.clone());
        } else {
            remaining.push((until, sender));
        }
    }
    if !remaining.is_empty() {
        map.insert(name.to_owned(), remaining);
    }
}

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
    /// Chat view state; `None` means the grid view is authoritative.
    chat: Option<ChatState>,
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
    run_with_label(&ipc::label())
}

/// Run the daemon on an explicit endpoint label.
///
/// Exists so a test can start an isolated daemon (see [`ipc::LABEL_ENV`])
/// without racing a real one the user already has running.
pub fn run_with_label(label: &str) -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    runtime.block_on(daemon_main(label))
}

async fn daemon_main(label: &str) -> io::Result<()> {
    let endpoint = rmux_ipc::endpoint_for_label(label)?;
    let listener = bind_listener(&endpoint)?;
    eprintln!(
        "canvas-terminal daemon: listening at {}",
        endpoint_display(&endpoint)
    );

    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));
    let next_id = Arc::new(AtomicU64::new(1));
    // Load whatever the previous daemon process (or this one, on a prior
    // start) recorded about session identity. A missing/corrupt/foreign
    // file just means "nothing known yet" — fresh shells, same as today.
    let sessions_path = crate::daemon_persist::path_for_label(label);
    let known: Known = Arc::new(Mutex::new(KnownRegistry {
        registry: SavedRegistry::load(&sessions_path).unwrap_or_else(SavedRegistry::empty),
        path: sessions_path,
    }));
    let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _peer)) => {
                let sessions = Arc::clone(&sessions);
                let next_id = Arc::clone(&next_id);
                let known = Arc::clone(&known);
                let waiters = Arc::clone(&waiters);
                tokio::spawn(async move {
                    handle_client(stream, sessions, next_id, known, waiters).await;
                });
            }
            Err(e) => eprintln!("canvas-terminal daemon: accept error: {e}"),
        }
    }
}

/// Detect which agent CLI is running inside a session's shell, by walking
/// the process subtree under the PTY child (`ps` on macOS has no /proc).
///
/// Returns `None` when the subtree holds no known CLI: the caller falls back
/// to its own preference (the GUI's `AGENT_CLI`, or the adapter default).
#[cfg(unix)]
fn detect_hosted_cli(child_pid: u32) -> Option<String> {
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid=,comm="])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    // Build the child->parent map, then walk up from every process to see
    // which ones belong to our subtree.
    let mut parent_of: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut comm_of: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(pid), Some(comm)) = (parts.next(), parts.next()) else {
            continue;
        };
        let (Ok(pid), Ok(ppid)) = (
            pid.parse::<u32>(),
            comm.parse::<String>().map(|_| 0u32), // placeholder, fixed below
        ) else {
            continue;
        };
        let _ = ppid;
        comm_of.insert(pid, comm.to_owned());
    }
    // ps comm column loses the parent; use ppid via a second pass with -o ppid
    let output = std::process::Command::new("ps")
        .args(["-eo", "pid=,ppid=,comm="])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut subtree: Vec<(u32, String)> = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(pid), Some(ppid), Some(comm)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let (Ok(pid), Ok(ppid)) = (pid.parse::<u32>(), ppid.parse::<u32>()) else {
            continue;
        };
        parent_of.insert(pid, ppid);
        comm_of.insert(pid, comm.to_owned());
    }
    for (&pid, comm) in &comm_of {
        let mut cur = pid;
        // Walk up; the subtree is small and shallow, the cap is a guard.
        for _ in 0..16 {
            if cur == child_pid {
                subtree.push((pid, comm.clone()));
                break;
            }
            match parent_of.get(&cur) {
                Some(&p) => cur = p,
                None => break,
            }
        }
    }
    // Deepest matching CLI wins: the shell's direct children (editors, ls)
    // are shallower than the REPL a user typed last.
    let mut best: Option<(u32, String)> = None;
    for (pid, comm) in subtree {
        let adapter = CliAdapter::from_comm(&comm);
        if matches!(adapter, CliAdapter::Unknown) {
            continue;
        }
        let depth = parent_of.get(&pid).map(|_| 0usize).unwrap_or(0);
        let _ = depth;
        let better = best
            .as_ref()
            .map(|(bpid, _)| pid > *bpid) // same-depth tiebreak; deeper pid is a later spawn
            .unwrap_or(true);
        if better {
            best = Some((pid, adapter.name().to_owned()));
        }
    }
    best.map(|(_, name)| name)
}

#[cfg(not(unix))]
fn detect_hosted_cli(_child_pid: u32) -> Option<String> {
    None
}

/// Create the endpoint's parent directory when it is missing.
///
/// `rmux-ipc` resolves the endpoint to `<tmp>/rmux-<uid>/<label>` and leaves
/// creating that directory to the rmux daemon that normally owns it.
/// `LocalListener::bind` then calls `UnixListener::bind` on the full path with
/// no `create_dir_all`, so on a machine that has never run rmux the very first
/// bind fails with `ENOENT` and the bundled daemon could never start — which
/// is exactly the case the bundled daemon exists to serve.
///
/// Best-effort: if the directory cannot be created, `bind` still runs and
/// reports its own error.
#[cfg(unix)]
fn ensure_socket_dir(endpoint: &LocalEndpoint) {
    if let Some(parent) = endpoint.as_path().parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
}

/// Bind the listener, clearing a stale socket file on Unix when no daemon is
/// actually listening there.
fn bind_listener(endpoint: &LocalEndpoint) -> io::Result<LocalListener> {
    #[cfg(unix)]
    ensure_socket_dir(endpoint);
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
async fn handle_client<S>(
    stream: S,
    sessions: Sessions,
    next_id: Arc<AtomicU64>,
    known: Known,
    waiters: Waiters,
)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut r, mut w) = tokio::io::split(stream);

    // Single writer task: control replies and session fanout share one
    // queue, so frames leave in a deterministic order per client.
    let (control_tx, mut control_rx) = mpsc::channel::<ipc::Frame>(SUB_CHAN);
    let writer = tokio::spawn(async move {
        while let Some(frame) = control_rx.recv().await {
            if ipc::write_frame(&mut w, &frame).await.is_err() {
                break;
            }
        }
    });

    loop {
        match ipc::read_request(&mut r).await {
            Ok(Some(req)) => {
                handle_request(req, &sessions, &next_id, &control_tx, &known, &waiters).await
            }
            Ok(None) => break,
            Err(e) => {
                eprintln!("canvas-terminal daemon: read error: {e}");
                break;
            }
        }
    }

    // Dropping the control sender ends the writer; the next fanout prunes
    // the stale subscriber entry.
    drop(control_tx);
    let _ = writer.await;
}

async fn handle_request(
    req: ipc::Request,
    sessions: &Sessions,
    next_id: &Arc<AtomicU64>,
    tx: &mpsc::Sender<ipc::Frame>,
    known: &Known,
    waiters: &Waiters,
) {
    match req {
        ipc::Request::Create(c) => match spawn_session(&c, sessions, next_id, tx.clone(), known, waiters)
        {
            Ok(id) => {
                // Record identity (argv/cwd) so a daemon restart can
                // `Resume` this session by name even with no chat CLI
                // ever involved.
                if let Ok(mut k) = known.lock() {
                    k.registry.upsert(SavedSession {
                        name: c.name.clone(),
                        argv: c.argv.clone(),
                        cwd: c.cwd.clone(),
                        chat_provider: None,
                        cli_session_id: None,
                    });
                }
                persist_known(known);
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
                        let old_name = s.name.clone();
                        s.name = r.name.clone();
                        Ok(old_name)
                    }
                    _ => Err(format!("no live session {}", r.session_id)),
                }
            };
            match outcome {
                Ok(old_name) => {
                    if let Ok(mut k) = known.lock() {
                        k.registry.rename(&old_name, &r.name);
                    }
                    persist_known(known);
                }
                Err(message) => {
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                        .await;
                }
            }
        }
        ipc::Request::List => {
            let infos = {
                let Ok(map) = sessions.lock() else { return };
                let mut infos: Vec<ipc::SessionInfo> = map
                    .values()
                    .map(|s| ipc::SessionInfo {
                        name: s.name.clone(),
                        session_id: s.id,
                        alive: s.alive,
                        resumable: !s.alive,
                    })
                    .collect();
                // A fresh daemon (post-restart) has no in-memory record at
                // all for a session the registry remembers; surface it as
                // resumable so a caller like `restore_canvas` can `Resume`
                // it by name instead of silently dropping the card.
                if let Ok(k) = known.lock() {
                    for saved in &k.registry.sessions {
                        if !infos.iter().any(|i| i.name == saved.name) {
                            infos.push(ipc::SessionInfo {
                                name: saved.name.clone(),
                                session_id: 0,
                                alive: false,
                                resumable: true,
                            });
                        }
                    }
                }
                infos
            };
            let _ = tx
                .send(ipc::Frame::List(ipc::ListResponse { sessions: infos }))
                .await;
        }
        ipc::Request::ChatSend(r) => {
            // Format the prompt with the session's adapter and write it to
            // the PTY — as JSONL on stdin for persistent CLIs, or as the next
            // relaunch command line for per-turn ones.
            let outcome = {
                let Ok(mut map) = sessions.lock() else { return };
                match map.values_mut().find(|s| s.name == r.session) {
                    None => Err(format!("no session named '{}'", r.session)),
                    Some(s) => {
                        let Some(chat) = s.chat.as_ref() else {
                            return;
                        };
                        let bytes = chat
                            .adapter
                            .format_input(&r.text, chat.cli_session_id.as_deref());
                        match s.master.write_all(&bytes) {
                            Ok(()) => Ok(()),
                            Err(e) => Err(format!("write failed: {e}")),
                        }
                    }
                }
            };
            if let Err(message) = outcome {
                let _ = tx
                    .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                    .await;
            }
        }
        ipc::Request::ChatSwitch(r) => {
            // Flip the parser, typing the launch/exit script into the PTY on
            // the way: the daemon owns the terminal, so view switches happen
            // the way a user would do them by hand. Replies with the new mode
            // (an empty sync) so the caller learns the flip landed.
            let outcome = {
                let Ok(mut map) = sessions.lock() else { return };
                match map.values_mut().find(|s| s.name == r.session) {
                    None => Err(format!("no session named '{}'", r.session)),
                    Some(s) => {
                        // "auto" asks the daemon to detect what the user
                        // launched in this shell, so the switch button does
                        // not need the GUI to guess.
                        let cli_name = if r.cli == "auto" {
                            detect_hosted_cli(s.child.pid().as_u32()).unwrap_or_else(|| {
                                std::env::var("AGENT_CLI").unwrap_or_else(|_| "pi".into())
                            })
                        } else {
                            r.cli.clone()
                        };
                        let adapter = CliAdapter::from_comm(&cli_name);
                        let resume = r
                            .resume
                            .clone()
                            .or_else(|| s.chat.as_ref().and_then(|c| c.cli_session_id.clone()));
                        let script = match (&r.script, r.chat) {
                            (ipc::SwitchScript::None, _) => String::new(),
                            (ipc::SwitchScript::FreshLaunch, true) => {
                                format!("{}\n", adapter.launch(ChatMode::Chat, resume.as_deref()))
                            }
                            (ipc::SwitchScript::FromTui, true) => {
                                let exit = adapter.exit_sequence().unwrap_or_default();
                                format!(
                                    "{exit}{}\n",
                                    adapter.launch(ChatMode::Chat, resume.as_deref())
                                )
                            }
                            (ipc::SwitchScript::FromChat, false) => {
                                let exit = adapter.exit_sequence().unwrap_or_default();
                                format!(
                                    "{exit}{}\n",
                                    adapter.launch(ChatMode::Tui, resume.as_deref())
                                )
                            }
                            // Inconsistent script/direction pairs: flip the
                            // parser anyway, without typing anything.
                            _ => String::new(),
                        };
                        let write_err = if script.is_empty() {
                            None
                        } else {
                            s.master.write_all(script.as_bytes()).err()
                        };
                        match (&mut s.chat, r.chat) {
                            (Some(state), true) => state.on = true,
                            (Some(state), false) => state.on = false,
                            (None, true) => s.chat = Some(ChatState::new(adapter)),
                            (None, false) => {}
                        }
                        match write_err {
                            Some(e) => Err(format!("write failed: {e}")),
                            None => Ok((
                                ipc::ChatSyncResponse {
                                    events: Vec::new(),
                                    chat: r.chat,
                                    cli: Some(r.cli.clone()),
                                    session_id: resume.clone(),
                                },
                                cli_name,
                                resume,
                            )),
                        }
                    }
                }
            };
            match outcome {
                Ok((resp, cli_name, resume)) => {
                    // Only a switch *into* chat means the session is really
                    // hosting that CLI; a switch back to the grid view
                    // leaves the last known provider/session id untouched.
                    if r.chat {
                        if let Ok(mut k) = known.lock() {
                            k.registry.set_chat(&r.session, Some(cli_name), resume);
                        }
                        persist_known(known);
                    }
                    let _ = tx.send(ipc::Frame::ChatSync(resp)).await;
                }
                Err(message) => {
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                        .await;
                }
            }
        }
        ipc::Request::ChatSync(r) => {
            let outcome = {
                let Ok(map) = sessions.lock() else { return };
                match map.values().find(|s| s.name == r.session) {
                    None => Err(format!("no session named '{}'", r.session)),
                    Some(s) => Ok(ipc::ChatSyncResponse {
                        events: s
                            .chat
                            .as_ref()
                            .map(|c| {
                                c.events
                                    .iter()
                                    .filter(|item| item.seq > r.after_seq)
                                    .cloned()
                                    .collect()
                            })
                            .unwrap_or_default(),
                        chat: s.chat.as_ref().is_some_and(|c| c.on),
                        cli: s.chat.as_ref().map(|c| c.adapter.name().to_owned()),
                        session_id: s.chat.as_ref().and_then(|c| c.cli_session_id.clone()),
                    }),
                }
            };
            match outcome {
                Ok(resp) => {
                    let _ = tx.send(ipc::Frame::ChatSync(resp)).await;
                }
                Err(message) => {
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                        .await;
                }
            }
        }
        ipc::Request::Resume(r) => {
            // Daemon-restart recovery: no live PTY exists for `r.name`
            // (otherwise the caller would `Attach`), but the daemon's own
            // registry may still remember how to relaunch it. Reuses
            // `spawn_session` exactly like `Create`, then — when the
            // registry also remembers a hosted CLI — arms the chat parser
            // and types the adapter's resume launch line, the same way
            // `canvas.rs`'s old restore-from-scratch path used to.
            let saved = {
                let Ok(k) = known.lock() else { return };
                k.registry.get(&r.name).cloned()
            };
            let Some(saved) = saved else {
                let _ = tx
                    .send(ipc::Frame::Error(ipc::ErrorResponse {
                        message: format!("no known session named '{}'", r.name),
                    }))
                    .await;
                return;
            };
            let argv = if saved.argv.is_empty() {
                default_shell_argv()
            } else {
                saved.argv.clone()
            };
            let create = ipc::CreateRequest {
                name: r.name.clone(),
                cwd: saved.cwd.clone(),
                argv,
                cols: r.cols,
                rows: r.rows,
            };
            match spawn_session(&create, sessions, next_id, tx.clone(), known, waiters) {
                Ok(id) => {
                    if let Some(provider) = saved.chat_provider.clone() {
                        let adapter = CliAdapter::from_comm(&provider);
                        let resume = saved.cli_session_id.clone();
                        let line = format!("{}\n", adapter.launch(ChatMode::Chat, resume.as_deref()));
                        let write_result = with_session_mut::<_, io::Error>(sessions, id, |s| {
                            s.chat = Some(ChatState::new(adapter));
                            if let Some(chat) = s.chat.as_mut() {
                                chat.cli_session_id = resume.clone();
                            }
                            s.master.write_all(line.as_bytes())
                        });
                        if let Some(Err(e)) = write_result {
                            eprintln!("canvas-terminal daemon: resume relaunch write failed: {e}");
                        }
                    }
                    let _ = tx
                        .send(ipc::Frame::Create(ipc::CreateResponse { session_id: id }))
                        .await;
                }
                Err(msg) => {
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse { message: msg }))
                        .await;
                }
            }
        }
        ipc::Request::AgentList => {
            let agents = {
                let Ok(map) = sessions.lock() else { return };
                map.values().map(agent_status_info).collect::<Vec<_>>()
            };
            let _ = tx
                .send(ipc::Frame::AgentList(ipc::AgentListResponse { agents }))
                .await;
        }
        ipc::Request::AgentWait(r) => {
            // Fast path: already satisfied, resolve without registering a
            // waiter at all.
            let current = {
                let Ok(map) = sessions.lock() else { return };
                map.values().find(|s| s.name == r.session).map(agent_status_info)
            };
            let Some(current) = current else {
                let _ = tx
                    .send(ipc::Frame::Error(ipc::ErrorResponse {
                        message: format!("no session named '{}'", r.session),
                    }))
                    .await;
                return;
            };
            let already = r.until.iter().any(|want| match want {
                ipc::AgentWaitState::Idle => !current.busy,
                ipc::AgentWaitState::Working => current.busy,
            });
            if already || !current.alive {
                let _ = tx.send(ipc::Frame::AgentStatus(current)).await;
                return;
            }
            let (wtx, wrx) = tokio::sync::oneshot::channel();
            {
                let Ok(mut w) = waiters.lock() else { return };
                w.entry(r.session.clone())
                    .or_default()
                    .push((r.until.clone(), wtx));
            }
            let timeout = std::time::Duration::from_millis(r.timeout_ms.max(1));
            match tokio::time::timeout(timeout, wrx).await {
                Ok(Ok(info)) => {
                    let _ = tx.send(ipc::Frame::AgentStatus(info)).await;
                }
                Ok(Err(_)) => {
                    // The sender side was dropped without resolving — should
                    // not happen, but report it rather than hang the caller.
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse {
                            message: "wait cancelled".into(),
                        }))
                        .await;
                }
                Err(_) => {
                    // Timed out: drop our now-stale waiter entry and report
                    // the last-known status with `timed_out` set.
                    if let Ok(mut w) = waiters.lock() {
                        if let Some(list) = w.get_mut(&r.session) {
                            list.retain(|(_, sender)| !sender.is_closed());
                        }
                    }
                    let mut info = {
                        let Ok(map) = sessions.lock() else { return };
                        map.values()
                            .find(|s| s.name == r.session)
                            .map(agent_status_info)
                            .unwrap_or(current)
                    };
                    info.timed_out = true;
                    let _ = tx.send(ipc::Frame::AgentStatus(info)).await;
                }
            }
        }
    }
}

/// Fallback argv when a registry entry predates argv being recorded (or was
/// created by `set_chat` alone, which does not know argv). Mirrors the
/// GUI's own default shell choice (`terminal/session.rs::shell_command`).
fn default_shell_argv() -> Vec<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    vec![shell, "-lc".to_string(), "zsh".to_string()]
}

fn spawn_session(
    c: &ipc::CreateRequest,
    sessions: &Sessions,
    next_id: &Arc<AtomicU64>,
    subscriber: mpsc::Sender<ipc::Frame>,
    known: &Known,
    waiters: &Waiters,
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
    // Every card's PTY carries this so a CLI running inside it (including
    // one card's hosted agent) can tell it is safe to shell out to
    // `canvas-terminal ipc ...` and reach sibling cards — the control
    // surface behind inter-agent communication. Mirrors herdr's
    // `HERDR_ENV=1`.
    cmd = cmd.env(ipc::ENV_MARKER_VAR, "1");

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
        chat: None,
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

    spawn_pty_reader(
        id,
        reader_io,
        Arc::clone(sessions),
        Arc::clone(known),
        Arc::clone(waiters),
    );
    Ok(id)
}

/// Dedicated OS thread that reads PTY output, appends it to the session ring,
/// and fans `Output` frames out to subscribers. On EOF/error it marks the
/// session dead, reaps the child, and emits `Ended`.
fn spawn_pty_reader(id: u64, io: PtyIo, sessions: Sessions, known: Known, waiters: Waiters) {
    std::thread::Builder::new()
        .name(format!("ct-pty-{id}"))
        .spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match io.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        end_session(&sessions, id, &waiters);
                        return;
                    }
                    Ok(n) => fanout_output(&sessions, id, buf[..n].to_vec(), &known, &waiters),
                }
            }
        })
        .expect("spawn pty reader thread");
}

/// Append bytes to the session ring and fan `Output` (always) and
/// `ChatEvent`s (when the chat parser is on) to subscribers.
fn fanout_output(sessions: &Sessions, id: u64, bytes: Vec<u8>, known: &Known, waiters: &Waiters) {
    let (subs, chat_events, learned, status_after) = {
        let Ok(mut map) = sessions.lock() else { return };
        let Some(s) = map.get_mut(&id) else { return };
        s.push_ring(&bytes);
        let chat_events = s
            .chat
            .as_mut()
            .filter(|chat| chat.on)
            .map(|chat| chat.feed(&bytes))
            .unwrap_or_default();
        // A `SessionInfo` event with a session id is the daemon's only
        // chance to learn the CLI's own conversation id from the live
        // stream (as opposed to the composer-driven `ChatSwitch` path,
        // which may not know it yet on a first launch). Record it so a
        // later daemon restart can `Resume` into the same conversation.
        let learned = chat_events.iter().find_map(|item| match &item.event {
            crate::chat::ChatEvent::SessionInfo {
                cli,
                session_id: Some(session_id),
            } => Some((s.name.clone(), cli.clone(), session_id.clone())),
            _ => None,
        });
        // Only worth checking waiters when this chunk actually produced chat
        // events (busy only changes on `UserMessage`/`TurnDone`/`Exited`).
        let status_after = if chat_events.is_empty() {
            None
        } else {
            Some(agent_status_info(s))
        };
        (s.subscribers.clone(), chat_events, learned, status_after)
    };
    if let Some((name, cli, session_id)) = learned {
        if let Ok(mut k) = known.lock() {
            k.registry.set_chat(&name, Some(cli), Some(session_id));
        }
        persist_known(known);
    }
    if let Some(info) = status_after {
        resolve_waiters(waiters, &info.name, &info, false);
    }

    let mut keep: Vec<mpsc::Sender<ipc::Frame>> = Vec::with_capacity(subs.len());
    for st in subs {
        let mut ok = true;
        match st.try_send(ipc::Frame::Output {
            session_id: id,
            bytes: bytes.clone(),
        }) {
            Ok(()) => {}
            // Channel full: keep the subscriber but drop this frame.
            Err(mpsc::error::TrySendError::Full(_)) => {}
            // Subscriber gone: prune it.
            Err(mpsc::error::TrySendError::Closed(_)) => ok = false,
        }
        for item in &chat_events {
            let _ = st.try_send(ipc::Frame::ChatEvent {
                session_id: id,
                item: item.clone(),
            });
        }
        if ok {
            keep.push(st);
        }
    }

    if let Ok(mut map) = sessions.lock() {
        if let Some(s) = map.get_mut(&id) {
            s.subscribers = keep;
        }
    }
}

/// Mark a session dead, reap its child, and emit `Ended` to subscribers.
/// Also force-resolves any outstanding `AgentWait` for this session: it will
/// never change status again, so a wait on it should not hang until timeout.
fn end_session(sessions: &Sessions, id: u64, waiters: &Waiters) {
    let ended = {
        let Ok(mut map) = sessions.lock() else { return };
        let Some(s) = map.get_mut(&id) else { return };
        s.alive = false;
        let _ = s.child.try_wait(); // reap
        let subs = std::mem::take(&mut s.subscribers);
        (subs, agent_status_info(s))
    };
    let (subs, info) = ended;
    for st in subs {
        let _ = st.try_send(ipc::Frame::Ended { session_id: id });
    }
    resolve_waiters(waiters, &info.name, &info, true);
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

#[cfg(all(test, unix))]
#[path = "daemon_chat_tests.rs"]
mod chat_tests;
