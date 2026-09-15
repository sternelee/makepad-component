//! The session: a conversation, a tool loop, and a replayable journal.
//!
//! One worker thread owns the provider, the tool registry and the conversation.
//! The UI thread only sends commands and drains events, which is what lets a
//! canvas card stay responsive while a turn runs — and lets the session outlive
//! the card, since nothing about it is tied to a widget.
//!
//! Three rules shape the concurrency here:
//!
//! * **Nothing the caller does may block on the worker.** `Drop` and
//!   `shutdown_timeout` never call `JoinHandle::join` without a bound, because
//!   the worker can be parked in a provider call.
//! * **The turn is published before its terminal event.** A client that reacts
//!   to `TurnFinished` must never read a stale `is_busy()` or a truncated
//!   `history()`.
//! * **A steer only ever reaches the turn it was written for.** Steering is
//!   tagged with a turn number so the window between "checked busy" and "the
//!   turn ended" cannot leak an instruction into the next turn.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TryRecvError};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use crate::cancel::CancelToken;
use crate::error::{AgentError, Result};
use crate::event::{AgentEvent, Sequenced};
use crate::identity::{process_epoch, Replay, ResumeCursor, SessionId};
use crate::message::Message;
use crate::provider::{CompletionRequest, Provider, ProviderEvent};
use crate::tool::{PermissionDecision, PermissionGate, ToolContext, ToolInvocation, ToolRegistry};

/// Events kept for replay before the oldest are dropped.
pub const JOURNAL_CAPACITY: usize = 20_000;

/// Prompts that may be queued behind the running turn before [`AgentError::Busy`].
///
/// Bounded on purpose: an unbounded queue lets a UI pile up work the user
/// cannot see or cancel yet.
pub const PROMPT_QUEUE_DEPTH: usize = 8;

/// How a session is configured. Provider and tools are supplied separately so
/// tests can substitute either.
#[derive(Clone, Debug)]
pub struct AgentSessionConfig {
    pub model: String,
    /// Prepended to every request.
    pub system_prompt: String,
    pub temperature: Option<f32>,
    /// Model→tool→model rounds allowed inside one prompt before the turn is
    /// cut off. Guards against a model that loops on its own output.
    pub max_tool_iterations: usize,
    /// Workspace root every tool path resolves against.
    pub root: PathBuf,
    /// When set, only these tools exist for this session: the rest are neither
    /// advertised to the model nor executable.
    ///
    /// This is the *existence* axis; [`PermissionGate`] is the *per-call* axis.
    /// Keeping them separate matters because hiding a tool from the model saves
    /// the schema tokens and stops it asking for something it can never have,
    /// which a denial-only policy cannot do.
    pub enabled_tools: Option<Vec<String>>,
}

impl AgentSessionConfig {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            model: String::new(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_owned(),
            temperature: None,
            max_tool_iterations: 24,
            root: root.into(),
            enabled_tools: None,
        }
    }

    /// Restrict the session to `tools`.
    pub fn with_tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.enabled_tools = Some(tools.into_iter().map(Into::into).collect());
        self
    }

    /// Whether this session offers `name` at all. `None` means everything the
    /// registry holds.
    pub fn advertises(&self, name: &str) -> bool {
        match &self.enabled_tools {
            Some(enabled) => enabled.iter().any(|candidate| candidate == name),
            None => true,
        }
    }
}

/// The default persona. Kept short: the tool schemas carry most of the contract.
pub const DEFAULT_SYSTEM_PROMPT: &str = "\
You are a coding agent working inside a workspace. Use the tools to inspect and \
change files, and keep going until the request is actually done. Prefer reading \
a file before editing it, keep edits minimal and targeted, and report what you \
changed rather than what you intend to change.";

/// A command the UI sends into a running session.
///
/// Cancellation and steering deliberately are *not* commands: the worker only
/// reads this channel between turns, so either would arrive after the turn it
/// was meant to affect. They reach the running turn directly instead.
enum SessionCommand {
    Prompt(String),
    Shutdown,
}

/// Steer messages tagged with the turn they were written for.
///
/// The tag is what closes the race where a caller reads `is_busy()` as true,
/// the turn ends, and an untagged message would then be picked up by the *next*
/// turn. A tagged steer either reaches its own turn or is reported as rejected.
#[derive(Default)]
struct SteerInbox {
    /// The turn currently accepting steers.
    turn: u64,
    entries: VecDeque<(u64, String)>,
}

/// The live end of a session, handed to whichever host forwards its events.
///
/// Taken exactly once, and only by a host that pumps: a session's event stream
/// has one consumer, and pretending otherwise (sharing it behind an `Arc`) only
/// produces two threads racing to drain the same channel.
pub struct SessionStream {
    pub events: Receiver<Sequenced<AgentEvent>>,
    /// True once the worker has ended, including on a panic. A host must check
    /// this when a read times out — a quiet stream and a dead worker look
    /// identical otherwise.
    pub finished: Arc<AtomicBool>,
}

/// A live agent session.
///
/// Dropping this asks the worker to stop and then **detaches** it — it never
/// blocks, so dropping a session from a UI thread is safe even while a model
/// call is in flight. Use [`Self::shutdown_timeout`] when a caller genuinely
/// needs to know the worker is gone.
pub struct AgentSession {
    id: SessionId,
    epoch: u64,
    commands: SyncSender<SessionCommand>,
    /// `None` once [`Self::take_stream`] has handed the live stream to a host.
    /// The journal remains authoritative either way.
    events: Option<Receiver<Sequenced<AgentEvent>>>,
    journal: Arc<Mutex<Vec<Sequenced<AgentEvent>>>>,
    history: Arc<Mutex<Vec<Message>>>,
    /// Cancels the turn currently running. A fresh token is installed at the
    /// start of every turn, so cancelling one turn cannot bleed into the next.
    cancel: Arc<Mutex<CancelToken>>,
    steer: Arc<Mutex<SteerInbox>>,
    /// Asks the worker to exit instead of starting anything new.
    stop: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
    /// Signalled by the worker the moment its thread ends — including on a
    /// panic, so a host that pumps events can always tell why the stream
    /// stopped instead of waiting forever.
    finished_flag: Arc<AtomicBool>,
    /// Signalled by the worker immediately before it returns. `JoinHandle::join`
    /// has no timeout, so a bounded wait needs a signal of its own.
    finished: Receiver<()>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl AgentSession {
    /// Start a session on its own thread, stamped with the process epoch.
    pub fn start(
        provider: Box<dyn Provider>,
        registry: ToolRegistry,
        gate: Arc<dyn PermissionGate>,
        config: AgentSessionConfig,
    ) -> Self {
        Self::start_with_epoch(process_epoch(), provider, registry, gate, config)
    }

    /// Start a session with an explicit journal epoch.
    ///
    /// A host that outlives its sessions (a daemon) should pass a stable epoch
    /// of its own, so cursors handed out before a worker restart are still
    /// recognised as belonging to the same generation.
    pub fn start_with_epoch(
        epoch: u64,
        mut provider: Box<dyn Provider>,
        registry: ToolRegistry,
        gate: Arc<dyn PermissionGate>,
        config: AgentSessionConfig,
    ) -> Self {
        let id = SessionId::next();
        let (command_tx, command_rx) = mpsc::sync_channel(PROMPT_QUEUE_DEPTH);
        let (event_tx, event_rx) = mpsc::channel();
        // The worker keeps the sending half; if it panics the handle drops and
        // a bounded shutdown sees `Disconnected` instead of waiting out its
        // whole timeout.
        let (finished_tx, finished_rx) = mpsc::channel();

        let journal = Arc::new(Mutex::new(Vec::new()));
        let history = Arc::new(Mutex::new(Vec::new()));
        let cancel = Arc::new(Mutex::new(CancelToken::new()));
        let steer = Arc::new(Mutex::new(SteerInbox::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(AtomicBool::new(false));
        let finished_flag = Arc::new(AtomicBool::new(false));

        let worker = {
            let journal = Arc::clone(&journal);
            let history = Arc::clone(&history);
            let cancel = Arc::clone(&cancel);
            let steer = Arc::clone(&steer);
            let stop = Arc::clone(&stop);
            let busy = Arc::clone(&busy);
            let finished_flag = Arc::clone(&finished_flag);
            std::thread::Builder::new()
                .name(format!("agent-session-{id}"))
                .spawn(move || {
                    // Cleared on unwind too, so a panicking worker is still
                    // reported as finished rather than as a hung one.
                    let _exit = ExitSignal(finished_flag);
                    let mut emitter = Emitter {
                        seq: 0,
                        journal,
                        sink: event_tx,
                    };
                    run_session(
                        &mut *provider,
                        &registry,
                        gate.as_ref(),
                        &config,
                        &command_rx,
                        &history,
                        &cancel,
                        &steer,
                        &stop,
                        &busy,
                        &mut emitter,
                    );
                    emitter.emit(AgentEvent::Exited);
                    let _ = finished_tx.send(());
                })
                .expect("agent session thread")
        };

        Self {
            id,
            epoch,
            commands: command_tx,
            events: Some(event_rx),
            journal,
            history,
            cancel,
            steer,
            stop,
            busy,
            finished_flag,
            finished: finished_rx,
            worker: Some(worker),
        }
    }

    /// Hand the live event stream to a host that will pump it.
    ///
    /// Callable once. Afterwards [`Self::try_recv`], [`Self::drain`] and
    /// [`Self::recv_timeout`] return nothing — the journal, not the channel, is
    /// what a late attacher reads from, so nothing is lost by handing the
    /// stream away.
    pub fn take_stream(&mut self) -> Option<SessionStream> {
        Some(SessionStream {
            events: self.events.take()?,
            finished: Arc::clone(&self.finished_flag),
        })
    }

    /// This session's identity.
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// This session's journal generation.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// The position a client should store to resume this session later.
    pub fn cursor(&self) -> ResumeCursor {
        let seq = self
            .journal
            .lock()
            .map(|journal| journal.last().map(|item| item.seq).unwrap_or(0))
            .unwrap_or(0);
        ResumeCursor {
            session_id: self.id,
            epoch: self.epoch,
            seq,
        }
    }

    /// Resume from a stored cursor.
    ///
    /// A cursor from another session, or from an earlier epoch, is
    /// [`Replay::Stale`]: the client's sequence numbers refer to a stream that
    /// no longer exists, so it must discard its transcript rather than apply a
    /// diff against a different session's events.
    pub fn resume(&self, cursor: ResumeCursor) -> Replay {
        if cursor.session_id != self.id || cursor.epoch != self.epoch {
            return Replay::Stale;
        }
        let events: Vec<Sequenced<AgentEvent>> = self
            .journal()
            .into_iter()
            .filter(|item| item.seq > cursor.seq)
            .collect();
        if events.is_empty() {
            Replay::UpToDate
        } else {
            Replay::Events(events)
        }
    }

    /// Submit a prompt. The turn runs on the worker thread.
    ///
    /// Fails with [`AgentError::Busy`] once [`PROMPT_QUEUE_DEPTH`] prompts are
    /// already waiting, and with [`AgentError::Disconnected`] once the worker
    /// is gone.
    pub fn prompt(&self, text: impl Into<String>) -> Result<()> {
        self.commands
            .try_send(SessionCommand::Prompt(text.into()))
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => AgentError::Busy,
                mpsc::TrySendError::Disconnected(_) => AgentError::Disconnected,
            })
    }

    /// Add an instruction to the turn that is already running.
    ///
    /// The message is appended to the conversation before the next model call,
    /// so the running turn sees it without starting a new one. Fails with
    /// [`AgentError::NotRunning`] when no turn is running — an idle session
    /// wants [`Self::prompt`] instead.
    ///
    /// A steer written in the moment a turn ends is reported back as
    /// [`AgentEvent::SteerRejected`] rather than being carried into the next
    /// turn.
    pub fn steer(&self, text: impl Into<String>) -> Result<()> {
        if !self.is_busy() {
            return Err(AgentError::NotRunning);
        }
        let mut inbox = self.steer.lock().map_err(|_| AgentError::Disconnected)?;
        let turn = inbox.turn;
        inbox.entries.push_back((turn, text.into()));
        Ok(())
    }

    /// Ask the running turn to stop.
    ///
    /// Reaches the provider driver's stream loop, so the turn ends within
    /// roughly one streamed chunk — not at the next model call.
    pub fn cancel(&self) {
        if let Ok(token) = self.cancel.lock() {
            token.cancel();
        }
    }

    /// True while a turn is running, so a card can show a spinner and keep the
    /// composer in "steer or cancel" mode.
    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::Acquire)
    }

    /// True once the worker thread has ended, for any reason including a panic.
    ///
    /// A host that reads the event stream must check this when a read times
    /// out: a silent stream and a dead worker look identical otherwise.
    pub fn is_finished(&self) -> bool {
        self.finished_flag.load(Ordering::Acquire)
    }

    /// Ask the worker to exit once it reaches a boundary. Non-blocking.
    ///
    /// The same signal [`Drop`] sends, exposed so a host that shares a session
    /// through an `Arc` can stop it without waiting for the last reference to
    /// go away. Cancels the running turn so the exit does not have to wait for
    /// a model call to return on its own.
    pub fn request_stop(&self) {
        self.stop.store(true, Ordering::Release);
        self.cancel();
        self.request_shutdown();
    }

    /// Stop the worker and wait up to `timeout` for it to actually exit.
    ///
    /// Returns `true` when the worker is gone, `false` when it is still running
    /// — which happens when a provider call has not reached its next
    /// cancellation check. A `false` result leaves the session detached: it
    /// keeps its own resources and will exit on its own, so the caller must not
    /// assume the thread is reaped.
    pub fn shutdown_timeout(&mut self, timeout: Duration) -> bool {
        self.request_stop();
        match self.finished.recv_timeout(timeout) {
            // Signalled, or the worker died without signalling: either way it
            // is gone, so the join below returns immediately.
            Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                if let Some(worker) = self.worker.take() {
                    let _ = worker.join();
                }
                true
            }
            Err(RecvTimeoutError::Timeout) => false,
        }
    }

    /// Deliver `Shutdown` without ever blocking on a full queue.
    fn request_shutdown(&self) {
        match self.commands.try_send(SessionCommand::Shutdown) {
            Ok(()) | Err(mpsc::TrySendError::Disconnected(_)) => {}
            Err(mpsc::TrySendError::Full(_)) => {
                // The queue is full of prompts. The worker checks `stop` before
                // each one, so a later send is enough; do it off-thread so the
                // caller is never delayed by a queue it cannot see.
                let commands = self.commands.clone();
                let _ = std::thread::Builder::new()
                    .name("agent-shutdown".into())
                    .spawn(move || {
                        let _ = commands.send(SessionCommand::Shutdown);
                    });
            }
        }
    }

    /// The next event, if one is already queued. Always `None` once the stream
    /// has been handed to a pump via [`Self::take_stream`].
    pub fn try_recv(&self) -> Option<Sequenced<AgentEvent>> {
        match self.events.as_ref()?.try_recv() {
            Ok(item) => Some(item),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Drain everything currently queued, in order.
    pub fn drain(&self) -> Vec<Sequenced<AgentEvent>> {
        let mut items = Vec::new();
        while let Some(item) = self.try_recv() {
            items.push(item);
        }
        items
    }

    /// Block until one event arrives or `timeout` elapses. Always `None` once
    /// the stream has been handed to a pump via [`Self::take_stream`].
    pub fn recv_timeout(&self, timeout: Duration) -> Option<Sequenced<AgentEvent>> {
        self.events.as_ref()?.recv_timeout(timeout).ok()
    }

    /// Snapshot of every event ever emitted, for a client that reconnects.
    pub fn journal(&self) -> Vec<Sequenced<AgentEvent>> {
        self.journal
            .lock()
            .map(|journal| journal.clone())
            .unwrap_or_default()
    }

    /// Snapshot of the conversation, for persistence.
    pub fn history(&self) -> Vec<Message> {
        self.history
            .lock()
            .map(|history| history.clone())
            .unwrap_or_default()
    }
}

impl Drop for AgentSession {
    fn drop(&mut self) {
        // Deliberately no `join()`. The worker can be parked inside a provider
        // call, and `join()` has no timeout, so joining here would block
        // whichever thread dropped the session — in the daemon that is the UI
        // thread. Signal, then detach: dropping the handle lets the thread run
        // to its own exit.
        self.request_stop();
        self.worker.take();
    }
}

/// Sets the finished flag when the worker's stack unwinds, however it does.
struct ExitSignal(Arc<AtomicBool>);

impl Drop for ExitSignal {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

/// Writes each event to the journal and to the live channel.
struct Emitter {
    seq: u64,
    journal: Arc<Mutex<Vec<Sequenced<AgentEvent>>>>,
    sink: mpsc::Sender<Sequenced<AgentEvent>>,
}

impl Emitter {
    fn emit(&mut self, event: AgentEvent) {
        self.seq += 1;
        let item = Sequenced {
            seq: self.seq,
            value: event,
        };
        if let Ok(mut journal) = self.journal.lock() {
            if journal.len() >= JOURNAL_CAPACITY {
                journal.remove(0);
            }
            journal.push(item.clone());
        }
        // A disconnected sink means the UI is gone; the journal still records
        // the turn so a reconnect can replay it.
        let _ = self.sink.send(item);
    }
}

/// Remove every queued steer, splitting by whether it belongs to `turn`.
fn take_steers(inbox: &Arc<Mutex<SteerInbox>>, turn: u64) -> (Vec<String>, Vec<String>) {
    let Ok(mut inbox) = inbox.lock() else {
        return (Vec::new(), Vec::new());
    };
    let mut current = Vec::new();
    let mut stale = Vec::new();
    while let Some((tag, text)) = inbox.entries.pop_front() {
        if tag == turn {
            current.push(text);
        } else {
            stale.push(text);
        }
    }
    (current, stale)
}

#[allow(clippy::too_many_arguments)]
fn run_session(
    provider: &mut dyn Provider,
    registry: &ToolRegistry,
    gate: &dyn PermissionGate,
    config: &AgentSessionConfig,
    commands: &Receiver<SessionCommand>,
    history: &Arc<Mutex<Vec<Message>>>,
    cancel: &Arc<Mutex<CancelToken>>,
    steer: &Arc<Mutex<SteerInbox>>,
    stop: &Arc<AtomicBool>,
    busy: &Arc<AtomicBool>,
    emitter: &mut Emitter,
) {
    let mut messages: Vec<Message> = Vec::new();
    if !config.system_prompt.trim().is_empty() {
        push_message(
            history,
            &mut messages,
            Message::system(&config.system_prompt),
        );
    }
    let mut turn: u64 = 0;

    // Blocking `recv` rather than polling: `Drop` and `shutdown_timeout` always
    // deliver `Shutdown`, so there is nothing to poll for.
    while let Ok(command) = commands.recv() {
        match command {
            SessionCommand::Prompt(text) => {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                turn += 1;
                if let Ok(mut inbox) = steer.lock() {
                    inbox.turn = turn;
                }
                // Install the turn's token *before* publishing `busy`, so a
                // cancel that observes "busy" always reaches this turn.
                let token = CancelToken::new();
                if let Ok(mut slot) = cancel.lock() {
                    *slot = token.clone();
                }
                busy.store(true, Ordering::Release);
                // Record what was asked before anything else, so a replayed
                // journal shows the question next to the answer.
                emitter.emit(AgentEvent::PromptSubmitted { text: text.clone() });
                push_message(history, &mut messages, Message::user(text));

                // The turn is published *before* the terminal event, so a
                // client that reacts to `TurnFinished` never reads a stale
                // `is_busy()` or a truncated `history()`.
                let stop_reason = run_turn(
                    provider,
                    registry,
                    gate,
                    config,
                    history,
                    &mut messages,
                    turn,
                    steer,
                    &token,
                    emitter,
                );

                // Anything still queued belongs to a turn that is over. Report
                // it instead of carrying it into the next turn.
                let (late, stale) = take_steers(steer, turn);
                for text in late {
                    emitter.emit(AgentEvent::SteerRejected {
                        text,
                        reason: "the turn had already finished".into(),
                    });
                }
                for text in stale {
                    emitter.emit(AgentEvent::SteerRejected {
                        text,
                        reason: "the steer was written for an earlier turn".into(),
                    });
                }

                busy.store(false, Ordering::Release);
                emitter.emit(AgentEvent::TurnFinished { stop_reason });
            }
            SessionCommand::Shutdown => break,
        }
    }
}

/// Append a message to the worker's own conversation and to the shared
/// snapshot the UI reads. The conversation is append-only, so mirroring on
/// every push keeps the snapshot current without re-cloning the whole thing.
fn push_message(history: &Arc<Mutex<Vec<Message>>>, messages: &mut Vec<Message>, message: Message) {
    messages.push(message.clone());
    if let Ok(mut shared) = history.lock() {
        shared.push(message);
    }
}

/// One prompt: model call, tool calls, repeat until the model stops asking.
///
/// Returns the stop reason. The caller emits `TurnFinished` so that every exit
/// path shares one publication order.
#[allow(clippy::too_many_arguments)]
fn run_turn(
    provider: &mut dyn Provider,
    registry: &ToolRegistry,
    gate: &dyn PermissionGate,
    config: &AgentSessionConfig,
    history: &Arc<Mutex<Vec<Message>>>,
    messages: &mut Vec<Message>,
    turn: u64,
    steer: &Arc<Mutex<SteerInbox>>,
    cancel: &CancelToken,
    emitter: &mut Emitter,
) -> Option<String> {
    emitter.emit(AgentEvent::TurnStarted);
    let context = ToolContext::new(config.root.clone());
    // Visibility, not permission: a tool this session does not offer is never
    // described to the model, so it cannot ask for one it can never run.
    let specs: Vec<_> = registry
        .specs()
        .into_iter()
        .filter(|spec| config.advertises(&spec.name))
        .collect();

    for _ in 0..config.max_tool_iterations {
        if cancel.is_cancelled() {
            return Some("cancelled".into());
        }

        // Fold in anything the user steered while the previous model call was
        // running, so the next call already sees it.
        let (steered, stale) = take_steers(steer, turn);
        for text in steered {
            emitter.emit(AgentEvent::SteerAccepted { text: text.clone() });
            push_message(history, messages, Message::user(text));
        }
        for text in stale {
            emitter.emit(AgentEvent::SteerRejected {
                text,
                reason: "the steer was written for an earlier turn".into(),
            });
        }

        let request = CompletionRequest {
            model: config.model.clone(),
            messages: messages.clone(),
            tools: specs.clone(),
            temperature: config.temperature,
        };

        // Forward provider deltas as they arrive; the caller needs live text,
        // not just the finished message.
        let mut forward = |event: ProviderEvent| match event {
            ProviderEvent::TextDelta(text) => emitter.emit(AgentEvent::TextDelta { text }),
            ProviderEvent::ReasoningDelta(text) => {
                emitter.emit(AgentEvent::ReasoningDelta { text })
            }
            ProviderEvent::Usage(usage) => emitter.emit(AgentEvent::Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
            }),
            // Tool-call fragments and finish reasons are surfaced from the
            // assembled response instead, where they are already coherent.
            ProviderEvent::ToolCallDelta { .. } | ProviderEvent::Finished { .. } => {}
        };

        let response = match provider.complete(&request, cancel, &mut forward) {
            Ok(response) => response,
            Err(AgentError::Cancelled) => return Some("cancelled".into()),
            Err(error) => {
                emitter.emit(AgentEvent::Error {
                    message: error.to_string(),
                });
                return Some("error".into());
            }
        };

        if response.text.is_empty() && response.tool_calls.is_empty() {
            return response.stop_reason.or_else(|| Some("empty".into()));
        }

        push_message(
            history,
            messages,
            Message::assistant_tool_calls(response.text.clone(), response.tool_calls.clone()),
        );

        if response.tool_calls.is_empty() {
            return response.stop_reason;
        }

        for call in &response.tool_calls {
            let invocation = ToolInvocation {
                id: call.id.clone(),
                name: call.name.clone(),
                arguments: call.arguments.clone(),
            };
            emitter.emit(AgentEvent::ToolCallRequested {
                id: invocation.id.clone(),
                name: invocation.name.clone(),
                arguments: invocation.arguments.clone(),
            });

            // Two gates, in order: existence, then the caller's per-call policy.
            // The first is enforced here so a tool outside `enabled_tools`
            // cannot be run even if the caller's gate would allow it.
            let decision = if config.advertises(&invocation.name) {
                gate.decide(&invocation, cancel)
            } else {
                PermissionDecision::Deny(format!(
                    "tool {:?} is not enabled for this session",
                    invocation.name
                ))
            };

            let content = match decision {
                PermissionDecision::Allow => {
                    let outcome = registry.invoke(&invocation, &context);
                    emitter.emit(AgentEvent::ToolCallFinished {
                        id: invocation.id.clone(),
                        name: invocation.name.clone(),
                        ok: !outcome.is_error,
                        summary: AgentEvent::summarize(&outcome.content, 160),
                    });
                    outcome.content
                }
                PermissionDecision::Deny(reason) => {
                    emitter.emit(AgentEvent::ToolCallDenied {
                        id: invocation.id.clone(),
                        name: invocation.name.clone(),
                        reason: reason.clone(),
                    });
                    format!("Tool call denied: {reason}")
                }
            };

            push_message(
                history,
                messages,
                Message::tool_result(invocation.id.clone(), content),
            );
        }

        if cancel.is_cancelled() {
            return Some("cancelled".into());
        }
    }

    Some("tool_iteration_limit".into())
}
