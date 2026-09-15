//! End-to-end tests for the session loop: provider → tool → provider → done.
//!
//! A scripted provider stands in for the model so the loop, the permission
//! gate, the real tool implementations and the event journal are all exercised
//! without a network or an API key.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use agent_core::{
    AgentError, AgentEvent, AgentSession, AgentSessionConfig, AllowAll, CancelToken,
    CompletionRequest, CompletionResponse, DenyAll, Provider, ProviderEvent, Replay, ResumeCursor,
    Sequenced, ToolCall, ToolRegistry,
};
use serde_json::json;

/// Temporary workspace, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "agent-core-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Replays a scripted response per call, recording every request it receives.
struct ScriptedProvider {
    script: Vec<CompletionResponse>,
    requests: Arc<Mutex<Vec<CompletionRequest>>>,
    index: usize,
}

impl ScriptedProvider {
    fn new(script: Vec<CompletionResponse>) -> (Self, Arc<Mutex<Vec<CompletionRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                script,
                requests: Arc::clone(&requests),
                index: 0,
            },
            requests,
        )
    }
}

impl Provider for ScriptedProvider {
    fn complete(
        &mut self,
        request: &CompletionRequest,
        _cancel: &CancelToken,
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> agent_core::Result<CompletionResponse> {
        self.requests.lock().unwrap().push(request.clone());
        let response = self
            .script
            .get(self.index)
            .cloned()
            .unwrap_or_else(|| CompletionResponse {
                text: "(script exhausted)".into(),
                stop_reason: Some("stop".into()),
                ..Default::default()
            });
        self.index += 1;

        // Stream the text the way a real driver would, so the loop's delta
        // forwarding is under test too.
        if !response.text.is_empty() {
            on_event(ProviderEvent::TextDelta(response.text.clone()));
        }
        for (index, call) in response.tool_calls.iter().enumerate() {
            on_event(ProviderEvent::ToolCallDelta {
                index,
                id: Some(call.id.clone()),
                name: Some(call.name.clone()),
                arguments_delta: call.arguments.to_string(),
            });
        }
        on_event(ProviderEvent::Finished {
            stop_reason: response.stop_reason.clone(),
        });
        Ok(response)
    }
}

fn text_response(text: &str) -> CompletionResponse {
    CompletionResponse {
        text: text.into(),
        stop_reason: Some("stop".into()),
        ..Default::default()
    }
}

fn tool_response(id: &str, name: &str, arguments: serde_json::Value) -> CompletionResponse {
    CompletionResponse {
        tool_calls: vec![ToolCall {
            id: id.into(),
            name: name.into(),
            arguments,
        }],
        stop_reason: Some("tool_calls".into()),
        ..Default::default()
    }
}

/// Collect events until the turn ends, or fail the test on timeout.
fn collect_turn(session: &AgentSession) -> Vec<AgentEvent> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut events = Vec::new();
    loop {
        match session.recv_timeout(Duration::from_millis(100)) {
            Some(Sequenced { value, .. }) => {
                let ends = value.ends_turn();
                events.push(value);
                if ends {
                    return events;
                }
            }
            None => assert!(
                Instant::now() < deadline,
                "turn did not finish in time; saw {events:#?}"
            ),
        }
    }
}

fn kinds(events: &[AgentEvent]) -> Vec<&'static str> {
    events
        .iter()
        .map(|event| match event {
            AgentEvent::PromptSubmitted { .. } => "prompt",
            AgentEvent::TurnStarted => "started",
            AgentEvent::TextDelta { .. } => "text",
            AgentEvent::ReasoningDelta { .. } => "reasoning",
            AgentEvent::ToolCallRequested { .. } => "tool_requested",
            AgentEvent::ToolCallFinished { .. } => "tool_finished",
            AgentEvent::ToolCallDenied { .. } => "tool_denied",
            AgentEvent::SteerAccepted { .. } => "steer_accepted",
            AgentEvent::SteerRejected { .. } => "steer_rejected",
            AgentEvent::Usage { .. } => "usage",
            AgentEvent::TurnFinished { .. } => "finished",
            AgentEvent::Error { .. } => "error",
            AgentEvent::Exited => "exited",
        })
        .collect()
}

#[test]
fn plain_turn_streams_text_and_finishes() {
    let workspace = TempDir::new("plain");
    let (provider, requests) = ScriptedProvider::new(vec![text_response("all done")]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("say hi").expect("prompt accepted");
    let events = collect_turn(&session);

    assert_eq!(
        kinds(&events),
        vec!["prompt", "started", "text", "finished"]
    );
    match &events[2] {
        AgentEvent::TextDelta { text } => assert_eq!(text, "all done"),
        other => panic!("expected text delta, got {other:?}"),
    }

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    // The system prompt is prepended, then the user turn.
    assert_eq!(requests[0].messages[0].role.as_str(), "system");
    assert_eq!(requests[0].messages[1].content, "say hi");
    // Every coding tool was advertised.
    let mut names: Vec<&str> = requests[0]
        .tools
        .iter()
        .map(|tool| tool.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec![
            "edit_file",
            "find_files",
            "read_file",
            "run_command",
            "search_files",
            "write_file",
        ]
    );
}

#[test]
fn tool_call_runs_the_tool_and_feeds_the_result_back() {
    let workspace = TempDir::new("tool");
    std::fs::write(workspace.path().join("hello.txt"), "line one\nline two\n").unwrap();

    let (provider, requests) = ScriptedProvider::new(vec![
        tool_response("call_1", "read_file", json!({ "path": "hello.txt" })),
        text_response("the file has two lines"),
    ]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("what is in hello.txt?").expect("prompt");
    let events = collect_turn(&session);

    assert_eq!(
        kinds(&events),
        vec![
            "prompt",
            "started",
            "tool_requested",
            "tool_finished",
            "text",
            "finished"
        ]
    );
    match &events[3] {
        AgentEvent::ToolCallFinished { ok, summary, .. } => {
            assert!(*ok, "tool should succeed");
            assert!(summary.contains("1\tline one"), "summary was {summary:?}");
        }
        other => panic!("expected tool finished, got {other:?}"),
    }

    // The model was called twice, and the second call saw the real file text.
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    let second = &requests[1].messages;
    let tool_message = second
        .iter()
        .find(|message| message.role.as_str() == "tool")
        .expect("tool result in the follow-up request");
    assert_eq!(tool_message.tool_call_id.as_deref(), Some("call_1"));
    assert!(tool_message.content.contains("line two"));
    // ...and the assistant turn that asked for it is preserved.
    assert!(second
        .iter()
        .any(|message| message.role.as_str() == "assistant" && !message.tool_calls.is_empty()));
}

#[test]
fn denied_tool_never_runs_and_the_model_is_told() {
    let workspace = TempDir::new("denied");
    let (provider, requests) = ScriptedProvider::new(vec![
        tool_response(
            "call_9",
            "write_file",
            json!({ "path": "should-not-exist.txt", "content": "nope" }),
        ),
        text_response("understood"),
    ]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(DenyAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("write a file").expect("prompt");
    let events = collect_turn(&session);

    assert_eq!(
        kinds(&events),
        vec![
            "prompt",
            "started",
            "tool_requested",
            "tool_denied",
            "text",
            "finished"
        ]
    );
    assert!(
        !workspace.path().join("should-not-exist.txt").exists(),
        "a denied tool must not touch the filesystem"
    );

    let requests = requests.lock().unwrap();
    let tool_message = requests[1]
        .messages
        .iter()
        .find(|message| message.role.as_str() == "tool")
        .expect("tool result");
    assert!(tool_message.content.contains("denied"));
}

#[test]
fn unknown_tool_is_reported_without_ending_the_turn() {
    let workspace = TempDir::new("unknown");
    let (provider, requests) = ScriptedProvider::new(vec![
        tool_response("call_x", "delete_everything", json!({})),
        text_response("sorry"),
    ]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("do something bad").expect("prompt");
    let events = collect_turn(&session);

    // The failure is reported as a finished (failed) tool, not as an error.
    assert_eq!(
        kinds(&events),
        vec![
            "prompt",
            "started",
            "tool_requested",
            "tool_finished",
            "text",
            "finished"
        ]
    );
    match &events[3] {
        AgentEvent::ToolCallFinished { ok, summary, .. } => {
            assert!(!ok);
            assert!(summary.contains("no such tool"));
        }
        other => panic!("expected tool finished, got {other:?}"),
    }
    assert!(requests.lock().unwrap().len() == 2);
}

#[test]
fn a_model_that_loops_on_tools_is_cut_off() {
    let workspace = TempDir::new("loop");
    std::fs::write(workspace.path().join("a.txt"), "x").unwrap();
    // Every call asks for the same tool again.
    let script: Vec<CompletionResponse> = (0..50)
        .map(|_| tool_response("call_loop", "read_file", json!({ "path": "a.txt" })))
        .collect();
    let (provider, requests) = ScriptedProvider::new(script);

    let mut config = AgentSessionConfig::new(workspace.path());
    config.max_tool_iterations = 3;
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        config,
    );

    session.prompt("loop forever").expect("prompt");
    let events = collect_turn(&session);

    match events.last() {
        Some(AgentEvent::TurnFinished { stop_reason }) => {
            assert_eq!(stop_reason.as_deref(), Some("tool_iteration_limit"))
        }
        other => panic!("expected iteration-limit finish, got {other:?}"),
    }
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3, "the loop must stop after the limit");
}

#[test]
fn journal_is_sequenced_and_replayable() {
    let workspace = TempDir::new("journal");
    let (provider, _) =
        ScriptedProvider::new(vec![text_response("first"), text_response("second")]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("one").expect("prompt one");
    collect_turn(&session);
    session.prompt("two").expect("prompt two");
    collect_turn(&session);

    let journal = session.journal();
    // Sequences are 1-based and strictly increasing, so a client can resume.
    for (index, item) in journal.iter().enumerate() {
        assert_eq!(item.seq, index as u64 + 1);
    }
    // Resuming from two events back replays exactly those two.
    let cursor = ResumeCursor {
        seq: journal.len() as u64 - 2,
        ..session.cursor()
    };
    match session.resume(cursor) {
        Replay::Events(events) => {
            assert_eq!(events.len(), 2);
            assert!(events.iter().all(|item| item.seq > cursor.seq));
        }
        other => panic!("expected two replayed events, got {other:?}"),
    }
    // A cursor already at the head has nothing to replay.
    assert_eq!(session.resume(session.cursor()), Replay::UpToDate);

    // Two turns are recorded in the conversation history.
    let history = session.history();
    assert_eq!(
        history
            .iter()
            .filter(|message| message.role.as_str() == "user")
            .count(),
        2
    );
}

#[test]
fn provider_failure_ends_the_turn_with_an_error() {
    struct FailingProvider;

    impl Provider for FailingProvider {
        fn complete(
            &mut self,
            _request: &CompletionRequest,
            _cancel: &CancelToken,
            _on_event: &mut dyn FnMut(ProviderEvent),
        ) -> agent_core::Result<CompletionResponse> {
            Err(agent_core::AgentError::Provider("HTTP 401".into()))
        }
    }

    let workspace = TempDir::new("failure");
    let session = AgentSession::start(
        Box::new(FailingProvider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("hello").expect("prompt");
    let events = collect_turn(&session);

    assert_eq!(
        kinds(&events),
        vec!["prompt", "started", "error", "finished"]
    );
    match &events[2] {
        AgentEvent::Error { message } => assert!(message.contains("HTTP 401")),
        other => panic!("expected error, got {other:?}"),
    }
    // The session survives a failed turn and is not busy afterwards.
    assert!(!session.is_busy());
}

#[test]
fn edit_file_replaces_once_and_refuses_ambiguity() {
    let workspace = TempDir::new("edit");
    let path = workspace.path().join("code.rs");
    std::fs::write(&path, "fn a() {}\nfn b() {}\n").unwrap();

    let (provider, _) = ScriptedProvider::new(vec![
        tool_response(
            "c1",
            "edit_file",
            json!({ "path": "code.rs", "old": "fn a() {}", "new": "fn a() { 1 }" }),
        ),
        tool_response(
            "c2",
            "edit_file",
            json!({ "path": "code.rs", "old": "fn ", "new": "fn " }),
        ),
        text_response("done"),
    ]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("edit it").expect("prompt");
    let events = collect_turn(&session);

    let finished: Vec<&AgentEvent> = events
        .iter()
        .filter(|event| matches!(event, AgentEvent::ToolCallFinished { .. }))
        .collect();
    assert_eq!(finished.len(), 2);
    match finished[0] {
        AgentEvent::ToolCallFinished { ok, .. } => assert!(ok),
        other => panic!("expected ok edit, got {other:?}"),
    }
    match finished[1] {
        // "fn " appears twice, so the edit is refused rather than guessed.
        AgentEvent::ToolCallFinished { ok, summary, .. } => {
            assert!(!ok);
            assert!(summary.contains("occurs 2 times"), "summary: {summary}");
        }
        other => panic!("expected refused edit, got {other:?}"),
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "fn a() { 1 }\nfn b() {}\n"
    );
}

#[test]
fn history_and_journal_survive_multiple_sessions_sharing_a_workspace() {
    let workspace = TempDir::new("shared");
    let (provider, _) = ScriptedProvider::new(vec![text_response("ok")]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );
    session.prompt("first").expect("prompt");
    collect_turn(&session);

    // A second session in the same workspace is independent.
    let (provider, _) = ScriptedProvider::new(vec![text_response("ok")]);
    let other = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );
    assert!(other.history().is_empty());
    assert!(other.journal().is_empty());
    assert_eq!(session.history().len(), 3); // system + user + assistant
}

// ---------------------------------------------------------------------------
// Concurrency guarantees. These are the regressions for D1 (drop must not
// block) and D2 (cancel must reach inside a provider call), both measured
// rather than asserted structurally.
// ---------------------------------------------------------------------------

/// Streams many small chunks, checking the token between them the way a real
/// driver checks between SSE lines.
struct StreamingProvider {
    chunks: usize,
    chunk_delay: Duration,
}

impl Provider for StreamingProvider {
    fn complete(
        &mut self,
        _request: &CompletionRequest,
        cancel: &CancelToken,
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> agent_core::Result<CompletionResponse> {
        let mut text = String::new();
        for index in 0..self.chunks {
            if cancel.is_cancelled() {
                return Err(AgentError::Cancelled);
            }
            std::thread::sleep(self.chunk_delay);
            let piece = format!("{index} ");
            text.push_str(&piece);
            on_event(ProviderEvent::TextDelta(piece));
        }
        Ok(CompletionResponse {
            text,
            stop_reason: Some("stop".into()),
            ..Default::default()
        })
    }
}

/// Never notices cancellation. Stands in for a stalled connection.
///
/// Signals `entered` from *inside* the call, so a test can wait until the
/// worker is genuinely parked in the provider. Waiting on `TurnStarted`
/// instead would race: that event is emitted just before the turn's first
/// cancellation check, so a cancel arriving in that window ends the turn
/// before the provider is ever called.
struct StubbornProvider {
    delay: Duration,
    entered: Option<std::sync::mpsc::Sender<()>>,
}

impl Provider for StubbornProvider {
    fn complete(
        &mut self,
        _request: &CompletionRequest,
        _cancel: &CancelToken,
        _on_event: &mut dyn FnMut(ProviderEvent),
    ) -> agent_core::Result<CompletionResponse> {
        if let Some(entered) = self.entered.take() {
            let _ = entered.send(());
        }
        std::thread::sleep(self.delay);
        Ok(CompletionResponse {
            text: "late".into(),
            stop_reason: Some("stop".into()),
            ..Default::default()
        })
    }
}

#[test]
fn cancel_stops_a_streaming_turn_within_a_chunk_not_at_the_end() {
    let workspace = TempDir::new("cancel");
    // 400 chunks x 10ms = 4s if nothing interrupts it.
    let session = AgentSession::start(
        Box::new(StreamingProvider {
            chunks: 400,
            chunk_delay: Duration::from_millis(10),
        }),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("go").expect("prompt");
    // Wait until the stream is genuinely mid-flight.
    let mut saw_delta = false;
    while !saw_delta {
        let item = session
            .recv_timeout(Duration::from_secs(5))
            .expect("stream should be running");
        saw_delta = matches!(item.value, AgentEvent::TextDelta { .. });
    }

    let started = Instant::now();
    session.cancel();
    let mut last = None;
    while let Some(item) = session.recv_timeout(Duration::from_secs(5)) {
        let ends = item.value.ends_turn();
        last = Some(item.value);
        if ends {
            break;
        }
    }
    let elapsed = started.elapsed();

    match last {
        Some(AgentEvent::TurnFinished { stop_reason }) => {
            assert_eq!(stop_reason.as_deref(), Some("cancelled"))
        }
        other => panic!("expected a cancelled finish, got {other:?}"),
    }
    assert!(
        elapsed < Duration::from_secs(1),
        "cancel took {elapsed:?}; it must land between chunks, not after the whole 4s stream"
    );
}

#[test]
fn dropping_a_session_never_blocks_the_caller() {
    let workspace = TempDir::new("drop");
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let session = AgentSession::start(
        Box::new(StubbornProvider {
            delay: Duration::from_secs(2),
            entered: Some(entered_tx),
        }),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("go").expect("prompt");
    entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the provider call should start");

    let started = Instant::now();
    drop(session);
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_millis(250),
        "drop blocked for {elapsed:?} while a 2s provider call was in flight"
    );
}

#[test]
fn shutdown_timeout_reports_whether_the_worker_actually_stopped() {
    let workspace = TempDir::new("shutdown");
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let mut session = AgentSession::start(
        Box::new(StubbornProvider {
            delay: Duration::from_secs(1),
            entered: Some(entered_tx),
        }),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("go").expect("prompt");
    entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the provider call should start");

    // The provider ignores cancellation, so the worker cannot be reaped yet.
    assert!(
        !session.shutdown_timeout(Duration::from_millis(100)),
        "a worker parked in a provider call must not be reported as stopped"
    );

    // Once the call returns the worker drains the queued Shutdown and exits.
    assert!(
        session.shutdown_timeout(Duration::from_secs(5)),
        "once the call returns the worker should stop within the timeout"
    );
}

// ---------------------------------------------------------------------------
// Identity, resume, steering and tool visibility. These cover D4 (a bare
// sequence number cannot identify a stream across restarts), Q1 (what a prompt
// or a steer means while a turn is running) and D3 (visibility vs permission).
// ---------------------------------------------------------------------------

/// Streams slowly on its first call so a test can steer mid-turn, then asks for
/// a tool so the turn loops into a second model call that must see the steer.
struct SteerableProvider {
    requests: Arc<Mutex<Vec<CompletionRequest>>>,
    calls: usize,
    first_call_chunks: usize,
    chunk_delay: Duration,
}

impl SteerableProvider {
    fn new(
        first_call_chunks: usize,
        chunk_delay: Duration,
    ) -> (Self, Arc<Mutex<Vec<CompletionRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                requests: Arc::clone(&requests),
                calls: 0,
                first_call_chunks,
                chunk_delay,
            },
            requests,
        )
    }
}

impl Provider for SteerableProvider {
    fn complete(
        &mut self,
        request: &CompletionRequest,
        cancel: &CancelToken,
        on_event: &mut dyn FnMut(ProviderEvent),
    ) -> agent_core::Result<CompletionResponse> {
        self.requests.lock().unwrap().push(request.clone());
        let call = self.calls;
        self.calls += 1;

        if call == 0 {
            for index in 0..self.first_call_chunks {
                if cancel.is_cancelled() {
                    return Err(AgentError::Cancelled);
                }
                std::thread::sleep(self.chunk_delay);
                on_event(ProviderEvent::TextDelta(format!("{index} ")));
            }
            return Ok(CompletionResponse {
                text: "working".into(),
                tool_calls: vec![ToolCall {
                    id: "call_a".into(),
                    name: "read_file".into(),
                    arguments: json!({ "path": "a.txt" }),
                }],
                stop_reason: Some("tool_calls".into()),
                ..Default::default()
            });
        }

        Ok(CompletionResponse {
            text: "done".into(),
            stop_reason: Some("stop".into()),
            ..Default::default()
        })
    }
}

fn run_until(session: &AgentSession, mut done: impl FnMut(&AgentEvent) -> bool) -> Vec<AgentEvent> {
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut seen = Vec::new();
    while Instant::now() < deadline {
        let Some(item) = session.recv_timeout(Duration::from_millis(50)) else {
            continue;
        };
        let finished = done(&item.value);
        let ends = item.value.ends_turn();
        seen.push(item.value);
        if finished || ends {
            break;
        }
    }
    seen
}

#[test]
fn a_cursor_from_another_session_is_stale() {
    let workspace = TempDir::new("stale-id");
    let (provider, _) = ScriptedProvider::new(vec![text_response("hi")]);
    let first = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );
    let (provider, _) = ScriptedProvider::new(vec![text_response("hi")]);
    let second = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    assert_ne!(first.id(), second.id());
    // Resuming a different session with another session's cursor must not
    // silently apply a diff against the wrong event stream.
    assert_eq!(second.resume(first.cursor()), Replay::Stale);
}

#[test]
fn a_cursor_from_an_earlier_epoch_is_stale() {
    let workspace = TempDir::new("stale-epoch");
    let (provider, _) = ScriptedProvider::new(vec![text_response("hi")]);
    let old = AgentSession::start_with_epoch(
        7,
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );
    let (provider, _) = ScriptedProvider::new(vec![text_response("hi")]);
    let new = AgentSession::start_with_epoch(
        8,
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    assert_eq!(old.epoch(), 7);
    assert_eq!(new.epoch(), 8);
    // The ids differ too, but the epoch alone must already disqualify it: a
    // daemon restart reuses ids.
    let cursor = ResumeCursor {
        session_id: new.id(),
        epoch: 7,
        seq: 0,
    };
    assert_eq!(new.resume(cursor), Replay::Stale);
}

#[test]
fn steer_reaches_the_running_turn_without_starting_a_new_one() {
    let workspace = TempDir::new("steer");
    std::fs::write(workspace.path().join("a.txt"), "content\n").unwrap();
    let (provider, requests) = SteerableProvider::new(20, Duration::from_millis(20)); // ~400ms first call
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("start").expect("prompt");
    // Wait until the turn is genuinely mid-stream.
    run_until(&session, |event| {
        matches!(event, AgentEvent::TextDelta { .. })
    });
    assert!(session.is_busy());

    session
        .steer("also check the tests")
        .expect("a steer during a running turn should be accepted");

    let accepted = run_until(&session, |event| {
        matches!(event, AgentEvent::SteerAccepted { .. })
    });
    // The steer is acknowledged, not silently swallowed, and it arrived inside
    // the turn that was already running rather than starting a second one.
    assert!(
        accepted.iter().any(|event| matches!(
            event,
            AgentEvent::SteerAccepted { text } if text == "also check the tests"
        )),
        "expected SteerAccepted, saw {:?}",
        kinds(&accepted)
    );
    assert_eq!(
        accepted
            .iter()
            .filter(|event| matches!(event, AgentEvent::TurnStarted))
            .count(),
        0,
        "SteerAccepted must arrive inside the turn that was already running"
    );

    // Let the turn finish so the follow-up model call is actually recorded,
    // then copy the requests out before asserting: holding the lock across a
    // panic would poison it and take the worker down with it.
    run_until(&session, |event| event.ends_turn());
    let requests: Vec<CompletionRequest> = requests.lock().unwrap().clone();
    assert_eq!(
        requests.len(),
        2,
        "a tool call should have driven a second call"
    );
    let steered = requests[1].messages.iter().any(|message| {
        message.role.as_str() == "user" && message.content == "also check the tests"
    });
    assert!(
        steered,
        "the follow-up request should carry the steer; messages were {:?}",
        requests[1]
            .messages
            .iter()
            .map(|m| (m.role.as_str(), m.content.as_str()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn steer_without_a_running_turn_is_refused() {
    let workspace = TempDir::new("steer-idle");
    let (provider, _) = ScriptedProvider::new(vec![text_response("hi")]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );
    assert!(!session.is_busy());
    let error = session.steer("too late").expect_err("idle steer must fail");
    assert!(matches!(error, AgentError::NotRunning), "got {error:?}");
}

#[test]
fn a_steer_that_misses_the_turn_is_rejected_not_carried_forward() {
    let workspace = TempDir::new("steer-late");
    std::fs::write(workspace.path().join("a.txt"), "content\n").unwrap();
    // One iteration only: the turn ends as soon as its single tool call runs,
    // so a steer taken during that call can never be drained inside the turn.
    let (provider, _) = SteerableProvider::new(20, Duration::from_millis(20));
    let mut config = AgentSessionConfig::new(workspace.path());
    config.max_tool_iterations = 1;
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        config,
    );

    session.prompt("start").expect("prompt");
    run_until(&session, |event| {
        matches!(event, AgentEvent::TextDelta { .. })
    });
    session.steer("this will miss").expect("steer accepted");

    let events = run_until(&session, |event| event.ends_turn());
    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentEvent::SteerRejected { text, .. } if text == "this will miss"
        )),
        "a steer that missed its turn must be reported, saw {:?}",
        kinds(&events)
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AgentEvent::SteerAccepted { .. })),
        "it must not be reported as accepted"
    );
}

/// Blocks inside the provider until released, so a test can fill the queue.
struct BlockingProvider {
    entered: Option<std::sync::mpsc::Sender<()>>,
    release: std::sync::mpsc::Receiver<()>,
}

impl Provider for BlockingProvider {
    fn complete(
        &mut self,
        _request: &CompletionRequest,
        _cancel: &CancelToken,
        _on_event: &mut dyn FnMut(ProviderEvent),
    ) -> agent_core::Result<CompletionResponse> {
        if let Some(entered) = self.entered.take() {
            let _ = entered.send(());
        }
        let _ = self.release.recv();
        Ok(CompletionResponse {
            text: "released".into(),
            stop_reason: Some("stop".into()),
            ..Default::default()
        })
    }
}

#[test]
fn the_prompt_queue_is_bounded() {
    let workspace = TempDir::new("busy");
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let session = AgentSession::start(
        Box::new(BlockingProvider {
            entered: Some(entered_tx),
            release: release_rx,
        }),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("running").expect("first prompt");
    entered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the worker should reach the provider");

    // The worker took "running" off the channel, so exactly
    // PROMPT_QUEUE_DEPTH more fit before the queue is full.
    for index in 0..agent_core::session::PROMPT_QUEUE_DEPTH {
        session
            .prompt(format!("queued {index}"))
            .unwrap_or_else(|error| panic!("prompt {index} should fit: {error}"));
    }
    let error = session
        .prompt("one too many")
        .expect_err("the queue must be bounded");
    assert!(matches!(error, AgentError::Busy), "got {error:?}");

    let _ = release_tx.send(());
}

#[test]
fn prompting_a_stopped_session_reports_disconnected_not_cancelled() {
    let workspace = TempDir::new("disconnected");
    let mut session = AgentSession::start(
        Box::new(BlockingProvider {
            entered: None,
            release: {
                let (_tx, rx) = std::sync::mpsc::channel();
                rx
            },
        }),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );
    assert!(session.shutdown_timeout(Duration::from_secs(5)));

    let error = session
        .prompt("anyone there?")
        .expect_err("a stopped session cannot take prompts");
    assert!(
        matches!(error, AgentError::Disconnected),
        "a dead session is not a cancelled one, got {error:?}"
    );
}

#[test]
fn tools_outside_enabled_tools_are_neither_advertised_nor_runnable() {
    let workspace = TempDir::new("enabled-tools");
    let (provider, requests) = ScriptedProvider::new(vec![
        tool_response(
            "call_w",
            "write_file",
            json!({ "path": "should-not-exist.txt", "content": "nope" }),
        ),
        text_response("understood"),
    ]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()).with_tools(["read_file"]),
    );

    session.prompt("write a file").expect("prompt");
    let events = collect_turn(&session);

    // The model is told about one tool, so it never learns the others exist.
    let advertised: Vec<String> = {
        let requests = requests.lock().unwrap();
        requests[0]
            .tools
            .iter()
            .map(|tool| tool.name.clone())
            .collect()
    };
    assert_eq!(advertised, vec!["read_file".to_owned()]);

    // And asking for one anyway is refused by existence, before the caller's
    // gate is even consulted.
    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentEvent::ToolCallDenied { reason, .. } if reason.contains("not enabled")
        )),
        "saw {:?}",
        kinds(&events)
    );
    assert!(!workspace.path().join("should-not-exist.txt").exists());
}

#[test]
fn history_and_busy_are_already_settled_when_the_turn_is_reported_finished() {
    // The invariant a daemon relies on: on `TurnFinished` it can persist
    // `history()` and drop its "running" indicator without re-reading either
    // later. Checked at the exact moment the event is observed rather than
    // after the fact, so a violation fails deterministically instead of as a
    // timing race.
    let workspace = TempDir::new("settled");
    std::fs::write(workspace.path().join("a.txt"), "content\n").unwrap();
    let (provider, _) = ScriptedProvider::new(vec![
        tool_response("call_1", "read_file", json!({ "path": "a.txt" })),
        text_response("done"),
    ]);
    let session = AgentSession::start(
        Box::new(provider),
        ToolRegistry::coding(),
        Arc::new(AllowAll),
        AgentSessionConfig::new(workspace.path()),
    );

    session.prompt("go").expect("prompt");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let item = session
            .recv_timeout(Duration::from_millis(100))
            .unwrap_or_else(|| {
                assert!(Instant::now() < deadline, "turn did not finish in time");
                session
                    .recv_timeout(Duration::from_millis(100))
                    .expect("event")
            });
        if matches!(item.value, AgentEvent::TurnFinished { .. }) {
            // system + user + assistant(tool call) + tool result + assistant
            let history = session.history();
            assert_eq!(
                history.len(),
                5,
                "history must be complete on TurnFinished, got {:?}",
                history
                    .iter()
                    .map(|m| (m.role.as_str(), m.content.as_str()))
                    .collect::<Vec<_>>()
            );
            assert!(
                !session.is_busy(),
                "busy must be cleared before TurnFinished is published"
            );
            break;
        }
    }
}
