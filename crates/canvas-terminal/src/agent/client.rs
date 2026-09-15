//! The GUI-side client for a daemon-hosted agent.
//!
//! Mirrors [`crate::terminal::TerminalSession`]: connect on demand, hold the
//! local card state behind a mutex that a reader task keeps fresh, and let the
//! UI poll for changes. The one structural difference is the resync path — a
//! terminal's byte stream is gone when it is gone, but an agent's journal
//! survives in the daemon, so a gap in the event stream can be repaired by
//! re-reading it.

use std::sync::{Arc, Mutex};

use super::card::{AgentCardState, PendingApproval};
use crate::ipc;
use crate::terminal::session::{ensure_daemon, runtime, spawn_writer_task};

/// A live agent card, backed by a daemon session.
///
/// `list`/`attach` are the GUI-restart path, `has_gap`/`reload` the resync
/// path, and `cancel` the composer's stop affordance — kept together here so
/// the card and a future startup hook share one client.
///
/// The label variants (`*_on`) exist so a host can name the endpoint it talks
/// to explicitly; the tests use them to run an isolated daemon without
/// touching process-global environment state.
pub struct AgentClient {
    #[allow(dead_code)]
    pub name: String,
    /// The daemon endpoint this client talks to.
    label: String,
    pub cwd: String,
    pub agent_id: u64,
    /// The card's view of the event stream; the reader task keeps it fresh.
    pub state: Arc<Mutex<AgentCardState>>,
    writer_tx: tokio::sync::mpsc::Sender<ipc::Request>,
    notify: std::sync::mpsc::Sender<bool>,
    rx: std::sync::mpsc::Receiver<bool>,
    /// Keeps the reader and writer tasks alive for the client's lifetime.
    _read_task: tokio::task::JoinHandle<()>,
    _writer_task: tokio::task::JoinHandle<()>,
}

impl AgentClient {
    /// Create a new agent in the daemon and start streaming its events.
    pub fn spawn(request: ipc::AgentCreateRequest) -> Result<Self, String> {
        ensure_daemon()?;
        Self::spawn_on(&ipc::label(), request)
    }

    /// [`Self::spawn`] against an explicitly named daemon endpoint.
    pub fn spawn_on(label: &str, request: ipc::AgentCreateRequest) -> Result<Self, String> {
        ensure_daemon_on(label)?;
        let rt = runtime();
        let (mut r, w) = rt.block_on(connect_stream_on(label))?;
        let (writer_tx, writer_task) = spawn_writer_task(w);

        let create = ipc::Request::AgentCreate(Box::new(request.clone()));
        rt.block_on(async { writer_tx.send(create).await })
            .map_err(|_| "writer closed before AgentCreate".to_string())?;

        let state = Arc::new(Mutex::new(AgentCardState::new()));
        let (notify, rx) = std::sync::mpsc::channel::<bool>();
        let agent_id: u64 = rt.block_on(async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(ipc::Frame::AgentCreated(c))) => return Ok(c.agent_id),
                    Ok(Some(ipc::Frame::Error(e))) => return Err(e.message),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read AgentCreated: {e}")),
                }
            }
        })?;

        let read_task = spawn_reader(r, Arc::clone(&state), notify.clone());
        Ok(Self {
            name: request.name,
            label: label.to_owned(),
            cwd: request.cwd,
            agent_id,
            state,
            writer_tx,
            notify,
            rx,
            _read_task: read_task,
            _writer_task: writer_task,
        })
    }

    /// List the agents the daemon is holding (for re-attach after a GUI restart).
    #[allow(dead_code)]
    pub fn list() -> Result<Vec<ipc::AgentInfo>, String> {
        Self::list_on(&ipc::label())
    }

    /// [`Self::list`] against an explicitly named daemon endpoint.
    #[allow(dead_code)]
    pub fn list_on(label: &str) -> Result<Vec<ipc::AgentInfo>, String> {
        ensure_daemon_on(label)?;
        let rt = runtime();
        let (mut r, w) = rt.block_on(connect_stream_on(label))?;
        let (writer_tx, _writer_task) = spawn_writer_task(w);
        rt.block_on(async { writer_tx.send(ipc::Request::AgentList).await })
            .map_err(|_| "writer closed before AgentList".to_string())?;
        rt.block_on(async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(ipc::Frame::AgentList(l))) => return Ok(l.agents),
                    Ok(Some(ipc::Frame::Error(e))) => return Err(e.message),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read AgentList: {e}")),
                }
            }
        })
    }

    /// Re-attach to an existing agent: replay its journal, then stream live.
    #[allow(dead_code)]
    pub fn attach(info: ipc::AgentInfo) -> Result<Self, String> {
        Self::attach_on(&ipc::label(), info)
    }

    /// [`Self::attach`] against an explicitly named daemon endpoint.
    #[allow(dead_code)]
    pub fn attach_on(label: &str, info: ipc::AgentInfo) -> Result<Self, String> {
        ensure_daemon_on(label)?;
        let rt = runtime();
        let (mut r, w) = rt.block_on(connect_stream_on(label))?;
        let (writer_tx, writer_task) = spawn_writer_task(w);

        let attach = ipc::Request::AgentAttach(ipc::AgentAttachRequest {
            agent_id: info.agent_id,
            // The whole journal: a card re-opened after a restart has nothing.
            after_seq: 0,
        });
        rt.block_on(async { writer_tx.send(attach).await })
            .map_err(|_| "writer closed before AgentAttach".to_string())?;

        let state = Arc::new(Mutex::new(AgentCardState::new()));
        let (notify, rx) = std::sync::mpsc::channel::<bool>();
        let (agent_id, cwd): (u64, String) = rt.block_on(async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(ipc::Frame::AgentAttached(a))) => {
                        let replay = a.replay.clone();
                        if let Ok(mut card) = state.lock() {
                            card.reload(replay);
                        }
                        return Ok((a.agent_id, a.cwd));
                    }
                    Ok(Some(ipc::Frame::Error(e))) => return Err(e.message),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read AgentAttached: {e}")),
                }
            }
        })?;

        let read_task = spawn_reader(r, Arc::clone(&state), notify.clone());
        Ok(Self {
            name: info.name,
            label: label.to_owned(),
            cwd,
            agent_id,
            state,
            writer_tx,
            notify,
            rx,
            _read_task: read_task,
            _writer_task: writer_task,
        })
    }

    /// Drain the change-notification channel; true when anything arrived.
    pub fn poll(&self) -> bool {
        let mut any = false;
        while self.rx.try_recv().is_ok() {
            any = true;
        }
        any
    }

    /// True when the last fold observed a sequence gap.
    ///
    /// The live stream continues regardless; the caller should follow this
    /// with [`Self::reload`] to pull the missing range from the journal.
    #[allow(dead_code)]
    pub fn has_gap(&self) -> bool {
        self.state
            .lock()
            .map(|card| card.gap().is_some())
            .unwrap_or(false)
    }

    /// Repair a gap by re-reading the daemon's journal for this session.
    ///
    /// A second connection, not the card's own: the card's stream keeps
    /// running, and events are deduped by sequence when they fold in, so
    /// nothing renders twice.
    pub fn reload(&self) -> Result<(), String> {
        let rt = runtime();
        let (mut r, w) = rt.block_on(connect_stream_on(&self.label))?;
        let (writer_tx, _writer_task) = spawn_writer_task(w);
        rt.block_on(async {
            writer_tx
                .send(ipc::Request::AgentAttach(ipc::AgentAttachRequest {
                    agent_id: self.agent_id,
                    after_seq: 0,
                }))
                .await
        })
        .map_err(|_| "writer closed before reload".to_string())?;

        let attached = rt.block_on(async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(ipc::Frame::AgentAttached(a))) => return Ok(a),
                    Ok(Some(ipc::Frame::Error(e))) => return Err(e.message),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read AgentAttached: {e}")),
                }
            }
        })?;
        // Dropping the writer task and channel unsubscribes this one-shot
        // connection on the daemon's next fanout.
        drop(_writer_task);
        drop(writer_tx);

        if let Ok(mut card) = self.state.lock() {
            card.reload(attached.replay);
        }
        let _ = self.notify.send(true);
        Ok(())
    }

    /// Submit a prompt: a new turn, or queued when one is already running.
    pub fn prompt(&self, text: &str) {
        let _ = self
            .writer_tx
            .try_send(ipc::Request::AgentPrompt(ipc::AgentInputRequest {
                agent_id: self.agent_id,
                text: text.to_owned(),
            }));
    }

    /// Add an instruction to the turn that is already running.
    pub fn steer(&self, text: &str) {
        let _ = self
            .writer_tx
            .try_send(ipc::Request::AgentSteer(ipc::AgentInputRequest {
                agent_id: self.agent_id,
                text: text.to_owned(),
            }));
    }

    /// Ask the running turn to stop.
    #[allow(dead_code)]
    pub fn cancel(&self) {
        let _ = self
            .writer_tx
            .try_send(ipc::Request::AgentCancel(ipc::AgentIdRequest {
                agent_id: self.agent_id,
            }));
    }

    /// Answer an approval prompt.
    pub fn reply(&self, approval_id: u64, allow: bool, reason: Option<&str>) {
        let _ = self.writer_tx.try_send(ipc::Request::AgentPermissionReply(
            ipc::AgentPermissionReplyRequest {
                approval_id,
                allow,
                reason: reason.map(str::to_owned),
            },
        ));
        if let Ok(mut card) = self.state.lock() {
            card.clear_approval(approval_id);
        }
        let _ = self.notify.send(true);
    }

    /// Terminate the agent.
    pub fn kill(&self) {
        let _ = self
            .writer_tx
            .try_send(ipc::Request::AgentKill(ipc::AgentIdRequest {
                agent_id: self.agent_id,
            }));
    }

    /// True while the agent's turn is running.
    pub fn is_busy(&self) -> bool {
        self.state.lock().map(|card| card.busy).unwrap_or(false)
    }

    /// True once the daemon reported the worker gone.
    #[allow(dead_code)]
    pub fn is_dead(&self) -> bool {
        self.state.lock().map(|card| card.dead).unwrap_or(true)
    }
}

/// Ensure a daemon is reachable on `label`, launching one if necessary.
fn ensure_daemon_on(label: &str) -> Result<(), String> {
    let rt = runtime();
    if rt.block_on(probe_connect_on(label)) {
        return Ok(());
    }
    // Only the default-label daemon is ours to spawn: the detached re-exec is
    // started for the endpoint the app itself uses. An explicitly named
    // endpoint is managed by its caller (the tests start their own).
    if label == ipc::label() {
        return ensure_daemon();
    }
    Err(format!("no daemon is listening on the '{label}' endpoint"))
}

async fn probe_connect_on(label: &str) -> bool {
    match connect_stream_on(label).await {
        Ok((_r, _w)) => true,
        Err(_) => false,
    }
}

async fn connect_stream_on(label: &str) -> Result<(ReadHalf, WriteHalf), String> {
    let endpoint = rmux_ipc::endpoint_for_label(label).map_err(|e| format!("resolve: {e}"))?;
    #[cfg(unix)]
    {
        let path = endpoint.into_path();
        let stream = tokio::net::UnixStream::connect(&path)
            .await
            .map_err(|e| format!("connect {}: {e}", path.display()))?;
        let (r, w) = tokio::io::split(stream);
        Ok((Box::new(r), Box::new(w)))
    }
    #[cfg(windows)]
    {
        let name = endpoint.as_pipe_name();
        let stream = rmux_ipc::connect_windows_pipe(name)
            .await
            .map_err(|e| format!("connect pipe: {e}"))?;
        let (r, w) = tokio::io::split(stream);
        Ok((Box::new(r), Box::new(w)))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = label;
        Err("unsupported platform".into())
    }
}

/// The read/write halves of a daemon connection.
type ReadHalf = Box<dyn tokio::io::AsyncRead + Unpin + Send>;
type WriteHalf = Box<dyn tokio::io::AsyncWrite + Unpin + Send>;

/// The reader task: folds every frame into the card state and pokes the UI.
fn spawn_reader(
    r: Box<dyn tokio::io::AsyncRead + Unpin + Send>,
    state: Arc<Mutex<AgentCardState>>,
    notify: std::sync::mpsc::Sender<bool>,
) -> tokio::task::JoinHandle<()> {
    let mut r = r;
    runtime().spawn(async move {
        loop {
            match ipc::read_frame(&mut r).await {
                Ok(Some(frame)) => {
                    let changed = match frame {
                        ipc::Frame::AgentEvent { item, .. } => {
                            if let Ok(mut card) = state.lock() {
                                card.apply(item);
                            }
                            true
                        }
                        ipc::Frame::AgentPermissionRequest {
                            approval_id,
                            call_id,
                            name,
                            arguments,
                            timeout_ms,
                            ..
                        } => {
                            if let Ok(mut card) = state.lock() {
                                card.set_approval(PendingApproval {
                                    approval_id,
                                    call_id,
                                    name,
                                    arguments,
                                    timeout_ms,
                                });
                            }
                            true
                        }
                        _ => false,
                    };
                    if changed {
                        let _ = notify.send(true);
                    }
                }
                Ok(None) | Err(_) => {
                    // The daemon went away. The card keeps its transcript, but
                    // it must stop looking like the agent is still working.
                    if let Ok(mut card) = state.lock() {
                        card.dead = true;
                        card.busy = false;
                        card.approval = None;
                    }
                    let _ = notify.send(true);
                    return;
                }
            }
        }
    })
}
