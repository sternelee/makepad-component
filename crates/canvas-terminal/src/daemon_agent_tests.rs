//! End-to-end coverage for the daemon's agent path.
//!
//! These drive a real daemon over a real socket, in-process, on an isolated
//! endpoint label — so they exercise the framing, the table, the event pump and
//! the cancellation plumbing exactly as the GUI will, without touching whatever
//! daemon the user happens to have running.
//!
//! The model is a scripted provider, so there is no network and no API key. The
//! *tools* are the real ones: the assertions about a file being read or not
//! written are made against a real temporary workspace.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Distinguishes names created in the same process.
///
/// `SystemTime` is only microsecond-resolution on macOS, so two tests starting
/// in the same instant get the *same* nanosecond value — and two daemons on one
/// label fight over a single socket. The counter is what makes uniqueness
/// guaranteed rather than likely.
static UNIQUE: AtomicU64 = AtomicU64::new(1);

fn unique_suffix() -> String {
    let count = UNIQUE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    format!("{nanos}-{count}")
}

use tokio::io::AsyncRead;

use super::*;
use crate::ipc::{
    AgentAttachRequest, AgentCreateRequest, AgentIdRequest, AgentInputRequest,
    AgentPermissionConfig, AgentProviderConfig::Scripted, Request, ScriptedTurnConfig,
};

/// A workspace that removes itself.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ct-agent-test-{label}-{}", unique_suffix()));
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

fn unique_label() -> String {
    format!("ct-agent-test-{}", unique_suffix())
}

/// Start an isolated daemon and connect to it.
async fn start_daemon(label: &str) -> tokio::net::UnixStream {
    let owned = label.to_owned();
    std::thread::Builder::new()
        .name("ct-test-daemon".into())
        .spawn(move || {
            let _ = run_with_label(&owned);
        })
        .expect("spawn test daemon");

    let endpoint = rmux_ipc::endpoint_for_label(label).expect("endpoint");
    let path = endpoint.into_path();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match tokio::net::UnixStream::connect(&path).await {
            Ok(stream) => return stream,
            Err(error) => {
                assert!(
                    Instant::now() < deadline,
                    "daemon never accepted a connection on {}: {error}",
                    path.display()
                );
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

/// Start an isolated daemon without a tokio context, for the synchronous
/// client tests.
fn start_daemon_sync(label: &str) {
    let owned = label.to_owned();
    std::thread::Builder::new()
        .name("ct-test-daemon".into())
        .spawn(move || {
            let _ = run_with_label(&owned);
        })
        .expect("spawn test daemon");

    let endpoint = rmux_ipc::endpoint_for_label(label).expect("endpoint");
    let path = endpoint.into_path();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match std::os::unix::net::UnixStream::connect(&path) {
            Ok(_) => return,
            Err(error) => {
                assert!(
                    Instant::now() < deadline,
                    "daemon never accepted a connection on {}: {error}",
                    path.display()
                );
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

/// Read frames until `want` matches, returning everything read.
async fn read_until<R, F>(r: &mut R, mut want: F) -> Vec<ipc::Frame>
where
    R: AsyncRead + Unpin,
    F: FnMut(&ipc::Frame) -> bool,
{
    let mut seen = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "timed out; saw {seen:#?}");
        let frame = tokio::time::timeout(remaining, ipc::read_frame(r))
            .await
            .expect("daemon went quiet")
            .expect("read frame")
            .expect("daemon closed the connection");
        let done = want(&frame);
        seen.push(frame);
        if done {
            return seen;
        }
    }
}

fn agent_events(frames: &[ipc::Frame]) -> Vec<&AgentEvent> {
    frames
        .iter()
        .filter_map(|frame| match frame {
            ipc::Frame::AgentEvent { item, .. } => Some(&item.value),
            _ => None,
        })
        .collect()
}

fn temp_workspace(label: &str, files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new(label);
    for (name, content) in files {
        std::fs::write(dir.path().join(name), content).expect("seed file");
    }
    dir
}

fn scripted_create(name: &str, cwd: &std::path::Path) -> Request {
    Request::AgentCreate(Box::new(AgentCreateRequest {
        name: name.into(),
        cwd: cwd.to_string_lossy().into_owned(),
        provider: Scripted {
            turns: vec![
                ScriptedTurnConfig {
                    text: "let me check the file".into(),
                    tool: Some(("read_file".into(), serde_json::json!({ "path": "a.txt" }))),
                    chunk_delay_ms: None,
                },
                ScriptedTurnConfig {
                    text: "it says hello".into(),
                    tool: None,
                    chunk_delay_ms: None,
                },
            ],
        },
        permission: AgentPermissionConfig::ReadOnly,
        model: None,
        system_prompt: None,
        enabled_tools: None,
    }))
}

/// The full path: create an agent, prompt it, watch a tool run, see the turn
/// finish. This is the test that says the daemon really hosts agents.
#[tokio::test]
async fn an_agent_runs_to_completion_over_the_daemon_socket() {
    let label = unique_label();
    let workspace = temp_workspace("run", &[("a.txt", "hello\n")]);
    let mut stream = start_daemon(&label).await;

    ipc::write_request(&mut stream, &scripted_create("worker", workspace.path()))
        .await
        .expect("send create");

    let frames = read_until(&mut stream, |frame| {
        matches!(frame, ipc::Frame::AgentCreated(_) | ipc::Frame::Error(_))
    })
    .await;
    let agent_id = match frames.last() {
        Some(ipc::Frame::AgentCreated(created)) => created.agent_id,
        other => panic!("expected AgentCreated, got {other:?}"),
    };

    ipc::write_request(
        &mut stream,
        &Request::AgentPrompt(AgentInputRequest {
            agent_id,
            text: "what is in a.txt?".into(),
        }),
    )
    .await
    .expect("send prompt");

    let frames = read_until(&mut stream, |frame| {
        matches!(
            frame,
            ipc::Frame::AgentEvent { item, .. } if item.value.ends_turn()
        )
    })
    .await;
    let events = agent_events(&frames);

    // The turn ran, the tool really executed, and the model saw its output.
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AgentEvent::TurnStarted)),
        "no TurnStarted in {events:#?}"
    );
    let streamed: String = events
        .iter()
        .filter_map(|event| match event {
            AgentEvent::TextDelta { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        streamed.contains("let me check"),
        "streamed text was {streamed:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentEvent::ToolCallRequested { name, .. } if name == "read_file"
        )),
        "the scripted tool call never surfaced: {events:#?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentEvent::ToolCallFinished { ok: true, summary, .. } if summary.contains("hello")
        )),
        "read_file did not return the file's contents: {events:#?}"
    );

    // Events arrive in strictly increasing sequence order, which is what a
    // client's resume logic depends on.
    let seqs: Vec<u64> = frames
        .iter()
        .filter_map(|frame| match frame {
            ipc::Frame::AgentEvent { item, .. } => Some(item.seq),
            _ => None,
        })
        .collect();
    assert!(
        seqs.windows(2).all(|pair| pair[1] > pair[0]),
        "sequences must increase: {seqs:?}"
    );

    // The daemon reports it as live, then accepts a terminal kill.
    ipc::write_request(&mut stream, &Request::AgentList)
        .await
        .expect("send list");
    let frames = read_until(&mut stream, |frame| {
        matches!(frame, ipc::Frame::AgentList(_))
    })
    .await;
    match frames.last() {
        Some(ipc::Frame::AgentList(list)) => {
            assert_eq!(list.agents.len(), 1);
            assert_eq!(list.agents[0].name, "worker");
            assert!(list.agents[0].alive);
            assert!(!list.agents[0].busy);
            assert!(list.agents[0].seq > 0);
        }
        other => panic!("expected AgentList, got {other:?}"),
    }

    ipc::write_request(
        &mut stream,
        &Request::AgentKill(AgentIdRequest { agent_id }),
    )
    .await
    .expect("send kill");
}

/// A second client attaching mid-conversation gets the transcript it missed and
/// then continues live.
#[tokio::test]
async fn attaching_replays_only_what_the_client_is_missing() {
    let label = unique_label();
    let workspace = temp_workspace("attach", &[("a.txt", "hello\n")]);
    let mut stream = start_daemon(&label).await;

    ipc::write_request(&mut stream, &scripted_create("attach-me", workspace.path()))
        .await
        .expect("send create");
    let frames = read_until(&mut stream, |frame| {
        matches!(frame, ipc::Frame::AgentCreated(_) | ipc::Frame::Error(_))
    })
    .await;
    let agent_id = match frames.last() {
        Some(ipc::Frame::AgentCreated(created)) => created.agent_id,
        other => panic!("expected AgentCreated, got {other:?}"),
    };

    ipc::write_request(
        &mut stream,
        &Request::AgentPrompt(AgentInputRequest {
            agent_id,
            text: "go".into(),
        }),
    )
    .await
    .expect("send prompt");
    read_until(&mut stream, |frame| {
        matches!(
            frame,
            ipc::Frame::AgentEvent { item, .. } if item.value.ends_turn()
        )
    })
    .await;

    // A fresh connection with no cursor gets the whole transcript.
    let mut late = start_daemon(&label).await;
    ipc::write_request(
        &mut late,
        &Request::AgentAttach(AgentAttachRequest {
            agent_id,
            after_seq: 0,
        }),
    )
    .await
    .expect("send attach");
    let frames = read_until(&mut late, |frame| {
        matches!(frame, ipc::Frame::AgentAttached(_) | ipc::Frame::Error(_))
    })
    .await;
    let (full_count, head_seq) = match frames.last() {
        Some(ipc::Frame::AgentAttached(attached)) => {
            assert_eq!(attached.name, "attach-me");
            assert!(!attached.busy);
            (
                attached.replay.len(),
                attached.replay.first().map(|item| item.seq),
            )
        }
        other => panic!("expected AgentAttached, got {other:?}"),
    };
    assert!(full_count > 1, "expected a replay, got {full_count} events");
    assert_eq!(
        head_seq,
        Some(1),
        "a first attach starts at the first event"
    );

    // Attaching again from the head replays nothing.
    let mut caught_up = start_daemon(&label).await;
    ipc::write_request(
        &mut caught_up,
        &Request::AgentAttach(AgentAttachRequest {
            agent_id,
            after_seq: u64::MAX,
        }),
    )
    .await
    .expect("send attach");
    let frames = read_until(&mut caught_up, |frame| {
        matches!(frame, ipc::Frame::AgentAttached(_) | ipc::Frame::Error(_))
    })
    .await;
    match frames.last() {
        Some(ipc::Frame::AgentAttached(attached)) => assert!(
            attached.replay.is_empty(),
            "a client at the head should get nothing, got {} events",
            attached.replay.len()
        ),
        other => panic!("expected AgentAttached, got {other:?}"),
    }
}

/// The default permission policy is what stops an unattended agent from being
/// able to change the workspace.
#[tokio::test]
async fn the_default_policy_blocks_writes_even_when_the_model_asks() {
    let label = unique_label();
    let workspace = temp_workspace("readonly", &[]);
    let mut stream = start_daemon(&label).await;

    let request = Request::AgentCreate(Box::new(AgentCreateRequest {
        name: "readonly".into(),
        cwd: workspace.path().to_string_lossy().into_owned(),
        provider: Scripted {
            turns: vec![ScriptedTurnConfig {
                text: "writing".into(),
                tool: Some((
                    "write_file".into(),
                    serde_json::json!({ "path": "nope.txt", "content": "nope" }),
                )),
                chunk_delay_ms: None,
            }],
        },
        // Explicitly the default, to pin the behaviour rather than inherit it.
        permission: AgentPermissionConfig::ReadOnly,
        model: None,
        system_prompt: None,
        enabled_tools: None,
    }));
    ipc::write_request(&mut stream, &request)
        .await
        .expect("create");
    let frames = read_until(&mut stream, |frame| {
        matches!(frame, ipc::Frame::AgentCreated(_) | ipc::Frame::Error(_))
    })
    .await;
    let agent_id = match frames.last() {
        Some(ipc::Frame::AgentCreated(created)) => created.agent_id,
        other => panic!("expected AgentCreated, got {other:?}"),
    };

    ipc::write_request(
        &mut stream,
        &Request::AgentPrompt(AgentInputRequest {
            agent_id,
            text: "write it".into(),
        }),
    )
    .await
    .expect("prompt");
    let frames = read_until(&mut stream, |frame| {
        matches!(
            frame,
            ipc::Frame::AgentEvent { item, .. } if item.value.ends_turn()
        )
    })
    .await;

    let events = agent_events(&frames);
    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentEvent::ToolCallDenied { name, .. } if name == "write_file"
        )),
        "expected the write to be denied: {events:#?}"
    );
    assert!(
        !workspace.path().join("nope.txt").exists(),
        "a denied tool must not have touched the workspace"
    );
}

// ---------------------------------------------------------------------------
// Interactive approval. The gate blocks the session's worker until a client
// answers, so these cover the three endings that matter — approved, refused,
// and never answered — plus the one that would otherwise wedge a turn:
// cancelling while an approval is outstanding.
// ---------------------------------------------------------------------------

/// A one-turn agent whose scripted call needs approval.
fn interactive_create(
    name: &str,
    cwd: &std::path::Path,
    tool: &str,
    arguments: serde_json::Value,
    timeout_ms: u64,
) -> Request {
    Request::AgentCreate(Box::new(AgentCreateRequest {
        name: name.into(),
        cwd: cwd.to_string_lossy().into_owned(),
        provider: Scripted {
            turns: vec![ScriptedTurnConfig {
                text: "attempting".into(),
                tool: Some((tool.into(), arguments)),
                chunk_delay_ms: None,
            }],
        },
        permission: AgentPermissionConfig::Interactive {
            timeout_ms: Some(timeout_ms),
        },
        model: None,
        system_prompt: None,
        enabled_tools: None,
    }))
}

async fn create_agent(stream: &mut tokio::net::UnixStream, request: Request) -> u64 {
    ipc::write_request(stream, &request).await.expect("create");
    let frames = read_until(stream, |frame| {
        matches!(frame, ipc::Frame::AgentCreated(_) | ipc::Frame::Error(_))
    })
    .await;
    match frames.last() {
        Some(ipc::Frame::AgentCreated(created)) => created.agent_id,
        other => panic!("expected AgentCreated, got {other:?}"),
    }
}

async fn prompt(stream: &mut tokio::net::UnixStream, agent_id: u64, text: &str) {
    ipc::write_request(
        stream,
        &Request::AgentPrompt(AgentInputRequest {
            agent_id,
            text: text.into(),
        }),
    )
    .await
    .expect("prompt");
}

/// Read until an approval prompt arrives, returning its id and the tool asked for.
async fn await_approval(stream: &mut tokio::net::UnixStream) -> (u64, String, u64) {
    let frames = read_until(stream, |frame| {
        matches!(frame, ipc::Frame::AgentPermissionRequest { .. })
    })
    .await;
    match frames.last() {
        Some(ipc::Frame::AgentPermissionRequest {
            approval_id,
            name,
            timeout_ms,
            ..
        }) => (*approval_id, name.clone(), *timeout_ms),
        other => panic!("expected an approval prompt, got {other:?}"),
    }
}

async fn reply(
    stream: &mut tokio::net::UnixStream,
    approval_id: u64,
    allow: bool,
    reason: Option<&str>,
) {
    ipc::write_request(
        stream,
        &Request::AgentPermissionReply(crate::ipc::AgentPermissionReplyRequest {
            approval_id,
            allow,
            reason: reason.map(str::to_owned),
        }),
    )
    .await
    .expect("reply");
}

async fn run_turn_to_end(stream: &mut tokio::net::UnixStream) -> Vec<ipc::Frame> {
    read_until(stream, |frame| {
        matches!(
            frame,
            ipc::Frame::AgentEvent { item, .. } if item.value.ends_turn()
        )
    })
    .await
}

#[tokio::test]
async fn an_approved_tool_call_runs() {
    let label = unique_label();
    let workspace = temp_workspace("approve", &[]);
    let mut stream = start_daemon(&label).await;

    let agent_id = create_agent(
        &mut stream,
        interactive_create(
            "approve",
            workspace.path(),
            "write_file",
            serde_json::json!({ "path": "made.txt", "content": "written" }),
            30_000,
        ),
    )
    .await;
    prompt(&mut stream, agent_id, "write it").await;

    let (approval_id, name, timeout_ms) = await_approval(&mut stream).await;
    assert_eq!(name, "write_file");
    assert_eq!(timeout_ms, 30_000);
    // The prompt must not have run anything yet.
    assert!(!workspace.path().join("made.txt").exists());

    reply(&mut stream, approval_id, true, None).await;
    let frames = run_turn_to_end(&mut stream).await;

    let events = agent_events(&frames);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AgentEvent::ToolCallFinished { ok: true, .. })),
        "an approved call should have run: {events:#?}"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("made.txt")).unwrap(),
        "written"
    );
}

#[tokio::test]
async fn a_refused_tool_call_does_not_run_and_the_model_hears_why() {
    let label = unique_label();
    let workspace = temp_workspace("refuse", &[]);
    let mut stream = start_daemon(&label).await;

    let agent_id = create_agent(
        &mut stream,
        interactive_create(
            "refuse",
            workspace.path(),
            "write_file",
            serde_json::json!({ "path": "made.txt", "content": "written" }),
            30_000,
        ),
    )
    .await;
    prompt(&mut stream, agent_id, "write it").await;

    let (approval_id, _, _) = await_approval(&mut stream).await;
    reply(&mut stream, approval_id, false, Some("not this file")).await;
    let frames = run_turn_to_end(&mut stream).await;

    let events = agent_events(&frames);
    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentEvent::ToolCallDenied { reason, .. } if reason == "not this file"
        )),
        "the client's reason should reach the denial: {events:#?}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AgentEvent::ToolCallFinished { .. })),
        "a refused call must never run"
    );
    assert!(
        !workspace.path().join("made.txt").exists(),
        "a refused call must not touch the workspace"
    );
}

#[tokio::test]
async fn an_unanswered_approval_denies_rather_than_hanging() {
    let label = unique_label();
    let workspace = temp_workspace("timeout", &[]);
    let mut stream = start_daemon(&label).await;

    let agent_id = create_agent(
        &mut stream,
        interactive_create(
            "timeout",
            workspace.path(),
            "write_file",
            serde_json::json!({ "path": "made.txt", "content": "written" }),
            300,
        ),
    )
    .await;
    prompt(&mut stream, agent_id, "write it").await;

    // Deliberately never reply.
    let (_, _, timeout_ms) = await_approval(&mut stream).await;
    assert_eq!(timeout_ms, 300);
    let started = Instant::now();
    let frames = run_turn_to_end(&mut stream).await;
    let elapsed = started.elapsed();

    let events = agent_events(&frames);
    assert!(
        events.iter().any(|event| matches!(
            event,
            AgentEvent::ToolCallDenied { reason, .. } if reason.contains("no one approved")
        )),
        "an unanswered prompt should deny: {events:#?}"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "the gate must give up on time, waited {elapsed:?}"
    );
    assert!(!workspace.path().join("made.txt").exists());
}

#[tokio::test]
async fn cancelling_aborts_an_outstanding_approval() {
    let label = unique_label();
    let workspace = temp_workspace("cancel-approval", &[]);
    let mut stream = start_daemon(&label).await;

    // A long timeout: if cancellation did not reach the waiting gate, the turn
    // would sit here for half a minute.
    let agent_id = create_agent(
        &mut stream,
        interactive_create(
            "cancel-approval",
            workspace.path(),
            "write_file",
            serde_json::json!({ "path": "made.txt", "content": "written" }),
            30_000,
        ),
    )
    .await;
    prompt(&mut stream, agent_id, "write it").await;
    await_approval(&mut stream).await;

    let started = Instant::now();
    ipc::write_request(
        &mut stream,
        &Request::AgentCancel(AgentIdRequest { agent_id }),
    )
    .await
    .expect("cancel");
    let frames = run_turn_to_end(&mut stream).await;
    let elapsed = started.elapsed();

    let events = agent_events(&frames);
    match events.last() {
        Some(AgentEvent::TurnFinished { stop_reason }) => assert_eq!(
            stop_reason.as_deref(),
            Some("cancelled"),
            "the turn should end as cancelled, events were {events:#?}"
        ),
        other => panic!("expected TurnFinished, got {other:?}"),
    }
    assert!(
        elapsed < Duration::from_secs(5),
        "cancel must reach a waiting gate, waited {elapsed:?}"
    );
    assert!(!workspace.path().join("made.txt").exists());
}

#[tokio::test]
async fn a_reply_for_an_unknown_prompt_is_ignored_not_fatal() {
    let label = unique_label();
    let workspace = temp_workspace("stale-reply", &[]);
    let mut stream = start_daemon(&label).await;

    let agent_id = create_agent(
        &mut stream,
        scripted_create("stale-reply", workspace.path()),
    )
    .await;

    // Nothing is waiting on this id — a prompt that expired, or a stale window.
    reply(&mut stream, 999_999, true, None).await;

    // The connection must stay healthy and the agent must still work.
    prompt(&mut stream, agent_id, "go").await;
    let frames = run_turn_to_end(&mut stream).await;
    let events = agent_events(&frames);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AgentEvent::TurnFinished { .. })),
        "a stale reply must not break the stream: {frames:#?}"
    );
    assert!(
        !frames
            .iter()
            .any(|frame| matches!(frame, ipc::Frame::Error(_))),
        "a stale reply is normal, not an error"
    );
}

// ---------------------------------------------------------------------------
// The canvas card's own client. Everything above drives the daemon with raw
// frames; these drive `AgentClient` — the object a canvas card actually holds
// — across the same socket, so the card's create→prompt→fold→approve sequence
// is covered end to end rather than assumed from the protocol tests.
// ---------------------------------------------------------------------------

use crate::agent::AgentClient;
use crate::agent::Row;

/// The scripted create request the command bar's `/new agent` path builds.
fn card_create(name: &str, cwd: &std::path::Path) -> crate::ipc::AgentCreateRequest {
    crate::ipc::AgentCreateRequest {
        name: name.into(),
        cwd: cwd.to_string_lossy().into_owned(),
        provider: Scripted {
            turns: vec![
                ScriptedTurnConfig {
                    text: "looking".into(),
                    tool: Some(("read_file".into(), serde_json::json!({ "path": "a.txt" }))),
                    chunk_delay_ms: None,
                },
                ScriptedTurnConfig {
                    text: "found it".into(),
                    tool: None,
                    chunk_delay_ms: None,
                },
            ],
        },
        permission: AgentPermissionConfig::ReadOnly,
        model: None,
        system_prompt: None,
        enabled_tools: None,
    }
}

fn row_kinds(state: &crate::agent::AgentCardState) -> Vec<&'static str> {
    state.rows().iter().map(Row::kind).collect()
}

fn wait_for_card<F>(client: &AgentClient, mut done: F)
where
    F: FnMut(&crate::agent::AgentCardState) -> bool,
{
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        client.poll();
        let satisfied = client.state.lock().map(|card| done(&card)).unwrap_or(false);
        if satisfied {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "card did not reach the expected state"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn the_canvas_card_client_runs_a_full_conversation() {
    let label = unique_label();
    let workspace = temp_workspace("client", &[("a.txt", "hello\n")]);
    start_daemon_sync(&label);

    let client = AgentClient::spawn_on(&label, card_create("card", workspace.path()))
        .expect("the card client should connect");
    assert_eq!(client.name, "card");
    assert!(!client.is_busy());

    // A prompt drives a tool call and then a closing model call.
    client.prompt("what is in a.txt?");
    wait_for_card(&client, |card| card.seq() >= 1 && !card.busy);

    let state = client.state.lock().unwrap();
    let kinds = row_kinds(&state);
    assert!(
        kinds.contains(&"user"),
        "the prompt should be in the transcript: {kinds:?}"
    );
    assert!(
        kinds.contains(&"tool"),
        "the scripted tool call should be folded into a tool row: {kinds:?}"
    );
    let assistant: String = state
        .rows()
        .iter()
        .filter_map(|row| match row {
            Row::Assistant { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        assistant.contains("found it"),
        "assistant text missing from {kinds:?}"
    );
    // No gap in a healthy session.
    assert!(state.gap().is_none());
}

#[test]
fn the_card_client_answers_an_approval_prompt() {
    let label = unique_label();
    let workspace = temp_workspace("client-approve", &[]);
    start_daemon_sync(&label);

    let mut request = card_create("approver", workspace.path());
    // One scripted turn that needs approval to run.
    request.provider = Scripted {
        turns: vec![ScriptedTurnConfig {
            text: "writing".into(),
            tool: Some((
                "write_file".into(),
                serde_json::json!({ "path": "made.txt", "content": "written" }),
            )),
            chunk_delay_ms: None,
        }],
    };
    // Read-only tools plus write_file: the interactive gate decides.
    request.permission = AgentPermissionConfig::Interactive {
        timeout_ms: Some(30_000),
    };
    let client = AgentClient::spawn_on(&label, request).expect("spawn");

    client.prompt("write it");
    wait_for_card(&client, |card| card.approval.is_some());

    // The card renders a prompt for this call…
    {
        let state = client.state.lock().unwrap();
        let approval = state.approval.as_ref().unwrap();
        assert_eq!(approval.name, "write_file");
        assert_eq!(approval.call_id, "scripted_0");
    }
    assert!(!workspace.path().join("made.txt").exists());

    // …and replying Allow runs it.
    let approval_id = client
        .state
        .lock()
        .unwrap()
        .approval
        .as_ref()
        .unwrap()
        .approval_id;
    client.reply(approval_id, true, None);

    wait_for_card(&client, |card| card.approval.is_none() && !card.busy);
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("made.txt")).unwrap(),
        "written"
    );
    // The approval is cleared on the card as well as in the daemon.
    let state = client.state.lock().unwrap();
    assert!(state.approval.is_none());
}

#[test]
fn the_card_client_reloads_its_transcript_from_the_daemon() {
    let label = unique_label();
    let workspace = temp_workspace("client-reload", &[("a.txt", "hello\n")]);
    start_daemon_sync(&label);

    let client =
        AgentClient::spawn_on(&label, card_create("reloader", workspace.path())).expect("spawn");
    client.prompt("go");
    wait_for_card(&client, |card| !card.busy && card.seq() > 0);

    let before = client.state.lock().unwrap().seq();

    // A fresh client attaching to the same agent id replays the journal: this
    // is what happens when the GUI restarts and the card is re-created.
    let agents = AgentClient::list_on(&label).expect("list");
    let mine = agents
        .iter()
        .find(|a| a.name == "reloader")
        .expect("the daemon should still hold the agent")
        .clone();
    let second = AgentClient::attach_on(&label, mine).expect("attach");

    wait_for_card(&second, |card| card.seq() >= before);
    let kinds = row_kinds(&second.state.lock().unwrap());
    assert!(
        kinds.contains(&"assistant"),
        "an attached card should have the transcript, got {kinds:?}"
    );
    assert!(
        kinds.contains(&"tool"),
        "the replayed transcript should include the tool call: {kinds:?}"
    );
    assert!(!second.is_dead());
}
