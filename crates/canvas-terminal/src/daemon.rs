//! Bundled PTY + agent daemon runtime, selected by `canvas-terminal --daemon`.
//!
//! Holds terminal sessions (PTY + child process) and agent sessions
//! (`agent-core`) out-of-process so both survive GUI restarts. The GUI connects
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
//! Terminals and agents share the two things that make an out-of-process
//! session worth having — a supervisor that outlives the GUI, and a fan-out to
//! whatever connections are attached — but differ in one way worth knowing: a
//! terminal's output is a byte stream, while an agent's output is a *sequence*
//! of events. Terminal bytes lost to a full subscriber channel are simply gone;
//! an agent event lost that way leaves a gap the client can see and repair by
//! re-attaching with `after_seq`, because the runtime's journal is the
//! authority and the stream is only a live tail of it.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use agent_core::{
    AgentEvent, AgentSession, AgentSessionConfig, AllowAll, AllowList, CancelToken, DenyAll,
    PermissionDecision, PermissionGate, ScriptedProvider, ScriptedTurn, Sequenced, ToolInvocation,
    ToolRegistry,
};
use rmux_ipc::{LocalEndpoint, LocalListener};
use rmux_pty::{ChildCommand, PtyChild, PtyIo, PtyMaster, Signal, TerminalSize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

use crate::ipc;

/// Ring buffer capacity per session (256 KiB of scrollback replay).
const RING_CAP: usize = 256 * 1024;
/// Per-subscriber output channel depth.
const SUB_CHAN: usize = 512;
/// Per-subscriber channel depth for agent events.
///
/// Larger than the terminal channel because agent frames are tiny and losing
/// one leaves a hole in a transcript rather than in a byte stream. It is still
/// bounded: a client that falls this far behind sees a gap in `seq` and
/// re-attaches with `after_seq` to backfill from the runtime's journal.
const AGENT_SUB_CHAN: usize = 4096;
/// How long the agent pump waits before re-checking whether its worker died.
const AGENT_POLL: std::time::Duration = std::time::Duration::from_millis(200);

/// Default budget for an unanswered interactive approval.
///
/// Generous, because a human may be away from the card — but finite, because
/// the agent's turn is genuinely paused while it waits.
pub const DEFAULT_APPROVAL_TIMEOUT_MS: u64 = 120_000;

/// How often a waiting gate re-checks for cancellation. Bounds how long a
/// "stop" takes to reach a turn that is sitting on an approval prompt.
const APPROVAL_POLL: std::time::Duration = std::time::Duration::from_millis(100);

/// Interactive approvals, shared by every agent and every connection.
///
/// Lives in the daemon rather than in `agent-core` because it is inherently
/// about *transport*: `agent-core` models the decision, this decides who is
/// asked and how the answer gets back.
struct Approvals {
    next_id: AtomicU64,
    /// approval id → where the answer goes. The waiting gate holds the receiver.
    pending: Mutex<HashMap<u64, std::sync::mpsc::Sender<PermissionDecision>>>,
}

type ApprovalHub = Arc<Approvals>;

/// Clears a pending slot however the wait ends, including on panic.
struct PendingGuard<'a> {
    pending: &'a Mutex<HashMap<u64, std::sync::mpsc::Sender<PermissionDecision>>>,
    id: u64,
}

impl Drop for PendingGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(&self.id);
        }
    }
}

impl Approvals {
    fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// Ask the attached clients and block until one answers.
    ///
    /// Blocks the calling thread — the session's worker — which is exactly the
    /// intent: that agent's turn waits for the human, and nothing else does.
    /// The event pump, other agents and the tokio runtime all keep running.
    fn request(
        &self,
        agents: &Agents,
        agent_id: u64,
        invocation: &ToolInvocation,
        timeout: std::time::Duration,
        cancel: &CancelToken,
    ) -> PermissionDecision {
        // Subscribers first: with nobody attached there is no one to ask, and
        // waiting out the timeout would just delay a denial.
        let subscribers = {
            let Ok(map) = agents.lock() else {
                return PermissionDecision::Deny("the agent table is poisoned".into());
            };
            match map.get(&agent_id) {
                Some(entry) => entry.subscribers.clone(),
                None => Vec::new(),
            }
        };
        if subscribers.is_empty() {
            return PermissionDecision::Deny(
                "no client is attached to approve this tool call".into(),
            );
        }

        let approval_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = std::sync::mpsc::channel();
        match self.pending.lock() {
            Ok(mut pending) => {
                pending.insert(approval_id, sender);
            }
            Err(_) => return PermissionDecision::Deny("the approval table is poisoned".into()),
        }
        let _guard = PendingGuard {
            pending: &self.pending,
            id: approval_id,
        };

        let frame = ipc::Frame::AgentPermissionRequest {
            agent_id,
            approval_id,
            call_id: invocation.id.clone(),
            name: invocation.name.clone(),
            arguments: invocation.arguments.clone(),
            timeout_ms: timeout.as_millis() as u64,
        };
        let mut delivered = false;
        for subscriber in &subscribers {
            // `blocking_send`, not `try_send`: an approval request dropped for
            // queue pressure would leave the turn waiting for a prompt nobody
            // ever saw, which is the one failure this path must not have. The
            // caller is a plain thread, so blocking is safe.
            if subscriber.agents.blocking_send(frame.clone()).is_ok() {
                delivered = true;
            }
        }
        if !delivered {
            return PermissionDecision::Deny(
                "no client is attached to approve this tool call".into(),
            );
        }

        let deadline = std::time::Instant::now() + timeout;
        loop {
            if cancel.is_cancelled() {
                return PermissionDecision::Deny("the turn was cancelled".into());
            }
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return PermissionDecision::Deny(format!(
                    "no one approved this call within {timeout:?}"
                ));
            }
            match receiver.recv_timeout(remaining.min(APPROVAL_POLL)) {
                Ok(decision) => return decision,
                // Nothing yet: loop, so cancellation is still observed.
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return PermissionDecision::Deny("the approval channel closed".into())
                }
            }
        }
    }

    /// Deliver a client's answer. Returns false when the id is unknown, which
    /// means the gate already gave up.
    fn resolve(&self, approval_id: u64, decision: PermissionDecision) -> bool {
        let Ok(mut pending) = self.pending.lock() else {
            return false;
        };
        match pending.remove(&approval_id) {
            Some(sender) => sender.send(decision).is_ok(),
            None => false,
        }
    }
}

/// A gate that asks the attached clients before running anything.
struct InteractiveGate {
    hub: ApprovalHub,
    agents: Agents,
    agent_id: u64,
    /// Per-agent budget, so a card can be impatient without changing the
    /// default for every other card.
    timeout: std::time::Duration,
}

impl PermissionGate for InteractiveGate {
    fn decide(&self, invocation: &ToolInvocation, cancel: &CancelToken) -> PermissionDecision {
        self.hub.request(
            &self.agents,
            self.agent_id,
            invocation,
            self.timeout,
            cancel,
        )
    }
}

type Sessions = Arc<Mutex<HashMap<u64, Session>>>;
type Agents = Arc<Mutex<HashMap<u64, AgentEntry>>>;

/// The frame queues belonging to one connection.
///
/// Two queues rather than one because the traffic shapes are opposites: a
/// terminal can emit megabytes of output in a burst, while an agent emits a
/// slow trickle of events that must not be lost. Sharing a queue would let a
/// `cargo build` flood evict transcript events, so each gets its own depth and
/// its own writer branch.
#[derive(Clone)]
struct Outbox {
    /// Control replies, terminal output, terminal `Ended`.
    control: mpsc::Sender<ipc::Frame>,
    /// Agent events.
    agents: mpsc::Sender<ipc::Frame>,
}

/// A daemon-owned agent session.
struct AgentEntry {
    id: u64,
    name: String,
    cwd: String,
    /// Owned, not shared. The event pump takes the session's stream instead, so
    /// nothing ever needs to hold this session while blocking — and the table
    /// lock is only ever held across the non-blocking calls below.
    session: AgentSession,
    subscribers: Vec<Outbox>,
    alive: bool,
}

impl AgentEntry {
    fn info(&self) -> ipc::AgentInfo {
        ipc::AgentInfo {
            agent_id: self.id,
            session_id: self.session.id().raw(),
            epoch: self.session.epoch(),
            name: self.name.clone(),
            cwd: self.cwd.clone(),
            busy: self.session.is_busy(),
            alive: self.alive && !self.session.is_finished(),
            seq: self.session.cursor().seq,
        }
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
    let agents: Agents = Arc::new(Mutex::new(HashMap::new()));
    let next_agent_id = Arc::new(AtomicU64::new(1));
    let approvals: ApprovalHub = Arc::new(Approvals::new());

    loop {
        match listener.accept().await {
            Ok((stream, _peer)) => {
                let sessions = Arc::clone(&sessions);
                let next_id = Arc::clone(&next_id);
                let agents = Arc::clone(&agents);
                let next_agent_id = Arc::clone(&next_agent_id);
                let approvals = Arc::clone(&approvals);
                tokio::spawn(async move {
                    handle_client(stream, sessions, next_id, agents, next_agent_id, approvals)
                        .await;
                });
            }
            Err(e) => eprintln!("canvas-terminal daemon: accept error: {e}"),
        }
    }
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
    agents: Agents,
    next_agent_id: Arc<AtomicU64>,
    approvals: ApprovalHub,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut r, mut w) = tokio::io::split(stream);

    // Writer task: multiplexes the connection's two outbound queues into the
    // stream. `select!` keeps agent events from waiting behind a terminal
    // burst while still writing frames one at a time.
    let (control_tx, mut control_rx) = mpsc::channel::<ipc::Frame>(SUB_CHAN);
    let (agent_tx, mut agent_rx) = mpsc::channel::<ipc::Frame>(AGENT_SUB_CHAN);
    let writer = tokio::spawn(async move {
        loop {
            let frame = tokio::select! {
                Some(frame) = control_rx.recv() => frame,
                Some(frame) = agent_rx.recv() => frame,
                else => break,
            };
            if ipc::write_frame(&mut w, &frame).await.is_err() {
                break;
            }
        }
    });
    let outbox = Outbox {
        control: control_tx,
        agents: agent_tx,
    };

    loop {
        match ipc::read_request(&mut r).await {
            Ok(Some(req)) => {
                handle_request(
                    req,
                    &sessions,
                    &next_id,
                    &agents,
                    &next_agent_id,
                    &approvals,
                    &outbox,
                )
                .await
            }
            Ok(None) => break,
            Err(e) => {
                eprintln!("canvas-terminal daemon: read error: {e}");
                break;
            }
        }
    }

    // Dropping the outbox ends the writer and disconnects the subscribers
    // cloned into sessions; the next fanout prunes the stale entries.
    drop(outbox);
    let _ = writer.await;
}

async fn handle_request(
    req: ipc::Request,
    sessions: &Sessions,
    next_id: &Arc<AtomicU64>,
    agents: &Agents,
    next_agent_id: &Arc<AtomicU64>,
    approvals: &ApprovalHub,
    outbox: &Outbox,
) {
    // Control replies all go out on the control queue; only `fanout_agent_event`
    // uses the agent one.
    let tx = &outbox.control;
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
        ipc::Request::AgentCreate(c) => {
            match spawn_agent(&c, agents, next_agent_id, approvals, outbox.clone()) {
                Ok(resp) => {
                    let _ = tx.send(ipc::Frame::AgentCreated(resp)).await;
                }
                Err(message) => {
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                        .await;
                }
            }
        }
        ipc::Request::AgentAttach(a) => {
            // Snapshot the journal and subscribe under one lock, so an event
            // cannot slip through the gap between "catch up" and "go live".
            //
            // An event *can* still arrive twice: the runtime appends to its
            // journal slightly before the pump forwards it, so a client that
            // attaches in that window sees it in `replay` and again on the
            // live stream. That is why every event carries a sequence number —
            // the client applies by `seq` and ignores anything it already has.
            let outcome = {
                let Ok(map) = agents.lock() else { return };
                match map.get(&a.agent_id) {
                    None => Err(format!("no agent {}", a.agent_id)),
                    Some(entry) => {
                        let session = &entry.session;
                        let replay: Vec<Sequenced<AgentEvent>> = session
                            .journal()
                            .into_iter()
                            .filter(|item| item.seq > a.after_seq)
                            .collect();
                        let mut subscribers = entry.subscribers.clone();
                        subscribers.push(outbox.clone());
                        let response = ipc::AgentAttachedResponse {
                            agent_id: entry.id,
                            session_id: session.id().raw(),
                            epoch: session.epoch(),
                            name: entry.name.clone(),
                            cwd: entry.cwd.clone(),
                            busy: session.is_busy(),
                            replay,
                        };
                        Ok((response, subscribers))
                    }
                }
            };
            match outcome {
                Ok((response, subscribers)) => {
                    if let Ok(mut map) = agents.lock() {
                        if let Some(entry) = map.get_mut(&a.agent_id) {
                            entry.subscribers = subscribers;
                        }
                    }
                    let _ = tx.send(ipc::Frame::AgentAttached(Box::new(response))).await;
                }
                Err(message) => {
                    let _ = tx
                        .send(ipc::Frame::Error(ipc::ErrorResponse { message }))
                        .await;
                }
            }
        }
        ipc::Request::AgentPrompt(r) => {
            let result = with_agent(agents, r.agent_id, |entry| entry.session.prompt(r.text));
            report_agent_result(agents, r.agent_id, result, tx, "prompt").await;
        }
        ipc::Request::AgentSteer(r) => {
            let result = with_agent(agents, r.agent_id, |entry| entry.session.steer(r.text));
            report_agent_result(agents, r.agent_id, result, tx, "steer").await;
        }
        ipc::Request::AgentCancel(r) => {
            let result = with_agent(agents, r.agent_id, |entry| {
                entry.session.cancel();
                Ok(())
            });
            report_agent_result(agents, r.agent_id, result, tx, "cancel").await;
        }
        ipc::Request::AgentKill(r) => {
            // Ask the worker to stop, then forget it. The pump holds the last
            // `Arc`, so the session is dropped only once the pump returns on
            // `Exited` — which is also when the entry stops being `alive`.
            let result = with_agent(agents, r.agent_id, |entry| {
                entry.session.request_stop();
                Ok(())
            });
            report_agent_result(agents, r.agent_id, result, tx, "kill").await;
        }
        ipc::Request::AgentPermissionReply(r) => {
            // An unknown id means the gate already gave up: the prompt expired
            // or the turn was cancelled. Clicking a moment too late is normal,
            // so it is ignored rather than reported.
            let decision = if r.allow {
                PermissionDecision::Allow
            } else {
                PermissionDecision::Deny(
                    r.reason
                        .clone()
                        .unwrap_or_else(|| "the user refused this tool call".into()),
                )
            };
            approvals.resolve(r.approval_id, decision);
        }
        ipc::Request::AgentList => {
            let infos = {
                let Ok(map) = agents.lock() else { return };
                map.values().map(AgentEntry::info).collect::<Vec<_>>()
            };
            let _ = tx
                .send(ipc::Frame::AgentList(ipc::AgentListResponse {
                    agents: infos,
                }))
                .await;
        }
    }
}

/// Run `f` against one agent without holding the table lock across an await.
fn with_agent<R>(
    agents: &Agents,
    id: u64,
    f: impl FnOnce(&AgentEntry) -> agent_core::Result<R>,
) -> agent_core::Result<R> {
    let Ok(map) = agents.lock() else {
        return Err(agent_core::AgentError::Disconnected);
    };
    match map.get(&id) {
        Some(entry) => f(entry),
        None => Err(agent_core::AgentError::Disconnected),
    }
}

/// Turn an agent request outcome into either nothing (success) or an `Error`
/// frame that names the agent and the verb that failed.
async fn report_agent_result(
    _agents: &Agents,
    agent_id: u64,
    result: agent_core::Result<()>,
    tx: &mpsc::Sender<ipc::Frame>,
    verb: &str,
) {
    if let Err(error) = result {
        let _ = tx
            .send(ipc::Frame::Error(ipc::ErrorResponse {
                message: format!("agent {agent_id}: {verb} failed: {error}"),
            }))
            .await;
    }
}

/// Build the provider, tools and permission gate for a new agent, start it, and
/// register it in the table.
fn spawn_agent(
    c: &ipc::AgentCreateRequest,
    agents: &Agents,
    next_agent_id: &Arc<AtomicU64>,
    approvals: &ApprovalHub,
    subscriber: Outbox,
) -> Result<ipc::AgentCreatedResponse, String> {
    let provider: Box<dyn agent_core::Provider> = match &c.provider {
        ipc::AgentProviderConfig::OpenAi {
            api_url,
            api_key,
            model,
            temperature,
        } => Box::new(agent_core::OpenAiProvider::new(agent_core::OpenAiConfig {
            api_url: api_url.clone(),
            api_key: api_key.clone(),
            model: model.clone(),
            temperature: *temperature,
        })),
        ipc::AgentProviderConfig::Scripted { turns } => Box::new(
            ScriptedProvider::from_turns(turns.iter().map(|turn| ScriptedTurn {
                text: turn.text.clone(),
                tool: turn.tool.clone(),
            }))
            .with_streaming(
                24,
                std::time::Duration::from_millis(
                    turns
                        .iter()
                        .find_map(|turn| turn.chunk_delay_ms)
                        .unwrap_or(0),
                ),
            ),
        ),
    };

    let mut config = AgentSessionConfig::new(&c.cwd);
    if let Some(model) = &c.model {
        config.model = model.clone();
    }
    if let Some(prompt) = &c.system_prompt {
        config.system_prompt = prompt.clone();
    }
    config.enabled_tools = c.enabled_tools.clone();

    // The id is minted before the session because an interactive gate needs it
    // to find the agent's subscribers when it asks for approval.
    let id = next_agent_id.fetch_add(1, Ordering::SeqCst);

    let mut session = AgentSession::start(
        provider,
        ToolRegistry::coding(),
        permission_gate(&c.permission, approvals, agents, id),
        config,
    );
    // The stream has exactly one consumer, and that is the pump below.
    let stream = session.take_stream();
    let cursor = session.cursor();

    {
        let Ok(mut map) = agents.lock() else {
            return Err("agent table poisoned".into());
        };
        // A name identifies a card: dropping dead entries keeps `AgentList`
        // honest and stops a stale agent from shadowing a live one.
        map.retain(|_, entry| entry.alive && !entry.session.is_finished());
        if map.values().any(|entry| entry.name == c.name) {
            return Err(format!("an agent named '{}' is already running", c.name));
        }
        map.insert(
            id,
            AgentEntry {
                id,
                name: c.name.clone(),
                cwd: c.cwd.clone(),
                session,
                subscribers: vec![subscriber],
                alive: true,
            },
        );
    }

    if let Some(stream) = stream {
        spawn_agent_pump(id, stream, Arc::clone(agents));
    }

    Ok(ipc::AgentCreatedResponse {
        agent_id: id,
        session_id: cursor.session_id.raw(),
        epoch: cursor.epoch,
        seq: cursor.seq,
    })
}

/// Translate the request's permission policy into a gate.
fn permission_gate(
    policy: &ipc::AgentPermissionConfig,
    approvals: &ApprovalHub,
    agents: &Agents,
    agent_id: u64,
) -> Arc<dyn PermissionGate> {
    match policy {
        ipc::AgentPermissionConfig::ReadOnly => Arc::new(AllowList(
            crate::daemon::READ_ONLY_TOOLS
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
        )),
        ipc::AgentPermissionConfig::AllowAll => Arc::new(AllowAll),
        ipc::AgentPermissionConfig::DenyAll => Arc::new(DenyAll),
        ipc::AgentPermissionConfig::AllowList { tools } => Arc::new(AllowList(tools.clone())),
        ipc::AgentPermissionConfig::Interactive { timeout_ms } => Arc::new(InteractiveGate {
            hub: Arc::clone(approvals),
            agents: Arc::clone(agents),
            agent_id,
            timeout: timeout_ms.map(std::time::Duration::from_millis).unwrap_or(
                std::time::Duration::from_millis(DEFAULT_APPROVAL_TIMEOUT_MS),
            ),
        }),
    }
}

/// Tools that only observe the workspace. The default policy for a new agent,
/// so an unattended agent cannot change anything until the caller opts in.
pub const READ_ONLY_TOOLS: &[&str] = &["read_file", "find_files", "search_files"];

/// Drain an agent's event stream into its subscribers.
///
/// A dedicated OS thread rather than a tokio task: `AgentSession` hands out
/// events through a blocking channel receiver, and blocking a runtime worker
/// for up to [`AGENT_POLL`] at a time would starve every other connection.
fn spawn_agent_pump(id: u64, stream: agent_core::SessionStream, agents: Agents) {
    std::thread::Builder::new()
        .name(format!("ct-agent-{id}"))
        .spawn(move || {
            let agent_core::SessionStream { events, finished } = stream;
            loop {
                match events.recv_timeout(AGENT_POLL) {
                    Ok(item) => {
                        let exited = matches!(item.value, AgentEvent::Exited);
                        fanout_agent_event(&agents, id, item);
                        if exited {
                            mark_agent_dead(&agents, id);
                            return;
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // A quiet stream and a dead worker look identical from
                        // the channel alone, so ask the flag the worker sets on
                        // its way out — including when it panics.
                        if finished.load(Ordering::Acquire) {
                            mark_agent_dead(&agents, id);
                            return;
                        }
                    }
                    // Buffered events are always delivered before this, so the
                    // worker is gone and there is nothing left to forward.
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        mark_agent_dead(&agents, id);
                        return;
                    }
                }
            }
        })
        .expect("spawn agent pump thread");
}

/// Send one agent event to every subscriber, pruning dead ones.
///
/// A full channel drops the frame rather than blocking the pump: the pump must
/// stay responsive for every other agent, and the dropped sequence number is
/// exactly what tells the client to re-attach and backfill.
fn fanout_agent_event(agents: &Agents, id: u64, item: Sequenced<AgentEvent>) {
    let subs = {
        let Ok(map) = agents.lock() else { return };
        let Some(entry) = map.get(&id) else { return };
        entry.subscribers.clone()
    };

    let mut keep: Vec<Outbox> = Vec::with_capacity(subs.len());
    for st in subs {
        match st.agents.try_send(ipc::Frame::AgentEvent {
            agent_id: id,
            item: item.clone(),
        }) {
            Ok(()) => keep.push(st),
            Err(mpsc::error::TrySendError::Full(_)) => keep.push(st),
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }
    }

    if let Ok(mut map) = agents.lock() {
        if let Some(entry) = map.get_mut(&id) {
            entry.subscribers = keep;
        }
    }
}

/// Mark an agent's worker as gone and release its subscribers.
fn mark_agent_dead(agents: &Agents, id: u64) {
    let Ok(mut map) = agents.lock() else { return };
    if let Some(entry) = map.get_mut(&id) {
        entry.alive = false;
        entry.subscribers.clear();
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

#[cfg(all(test, unix))]
#[path = "daemon_agent_tests.rs"]
mod agent_tests;
