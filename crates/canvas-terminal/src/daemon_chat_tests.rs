//! End-to-end chat-view tests over the real daemon, on isolated sockets.
//!
//! The agent under test is not a real CLI: a session running `cat` stands in
//! for one (input comes back as output, so the daemon's script-writing and
//! input-formatting are observable), and a session running a `printf` script
//! plays a pi-shaped JSONL emitter, so the parser path runs against the same
//! bytes the real CLI produces.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::ipc;

static LABEL: AtomicU64 = AtomicU64::new(0);

fn unique_label() -> String {
    let n = LABEL.fetch_add(1, Ordering::SeqCst);
    format!("ct-chat-test-{n}-{}", std::process::id())
}

/// Run an isolated daemon on a background thread and wait for its socket.
fn start_daemon_sync(label: &str) {
    let thread_label = label.to_owned();
    std::thread::Builder::new()
        .name("test-daemon".into())
        .spawn(move || {
            crate::daemon::run_with_label(&thread_label).expect("daemon run");
        })
        .expect("spawn daemon thread");
    let endpoint = rmux_ipc::endpoint_for_label(label).expect("endpoint");
    let path = endpoint.into_path();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(Instant::now() < deadline, "daemon socket never appeared");
        std::thread::sleep(Duration::from_millis(10));
    }
}

async fn connect_on(
    label: &str,
) -> std::result::Result<
    (
        Box<dyn AsyncRead + Unpin + Send>,
        Box<dyn AsyncWrite + Unpin + Send>,
    ),
    String,
> {
    let endpoint = rmux_ipc::endpoint_for_label(label).map_err(|e| e.to_string())?;
    let path = endpoint.into_path();
    let s = tokio::net::UnixStream::connect(&path)
        .await
        .map_err(|e| format!("connect: {e}"))?;
    let (r, w) = tokio::io::split(s);
    Ok((Box::new(r), Box::new(w)))
}

/// Drive one request/response round trip on a dedicated connection.
async fn rpc(label: &str, req: ipc::Request) -> Result<ipc::Frame, String> {
    let (mut r, mut w) = connect_on(label).await?;
    ipc::write_request(&mut w, &req)
        .await
        .map_err(|e| e.to_string())?;
    w.flush().await.map_err(|e| e.to_string())?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let frame = tokio::time::timeout_at(deadline, ipc::read_frame(&mut r))
            .await
            .map_err(|_| "timed out".to_string())?
            .map_err(|e| e.to_string())?;
        let Some(frame) = frame else {
            return Err("daemon closed".into());
        };
        match frame {
            // The session's own output streams first; skip to the reply.
            ipc::Frame::Output { .. } | ipc::Frame::ChatEvent { .. } => continue,
            other => return Ok(other),
        }
    }
}

fn spawn_emitter_session(
    rt: &tokio::runtime::Runtime,
    label: &str,
    name: &str,
    lines: &[&str],
) -> u64 {
    // The emitter writes its JSONL from a script file: shell quoting must
    // not touch the JSON (an escaped quote inside single quotes stays
    // literal and the line stops parsing as JSON). It also waits briefly
    // before producing, so the test can flip the chat parser on while the
    // process is still live - parsing happens on the live stream.
    let dir = std::env::temp_dir().join(format!(
        "ct-chat-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let script_path = dir.join("emit.sh");
    let body = format!("sleep 0.4\ncat <<'CT_EOF'\n{}\nCT_EOF\n", lines.join("\n"));
    std::fs::write(&script_path, body).expect("write script");
    let req = ipc::Request::Create(ipc::CreateRequest {
        name: name.to_owned(),
        cwd: None,
        argv: vec!["/bin/sh".into(), script_path.display().to_string()],
        cols: 80,
        rows: 24,
    });
    match rt.block_on(rpc(label, req)) {
        Ok(ipc::Frame::Create(c)) => c.session_id,
        other => panic!("expected Create reply, got {other:?}"),
    }
}

#[test]
fn chat_events_parse_from_a_pi_shaped_stream_and_replay_for_late_clients() {
    let label = unique_label();
    start_daemon_sync(&label);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let emitter_lines = [
        r#"{"type":"session","version":3,"id":"sid-1","cwd":"/tmp"}"#,
        r#"{"type":"message_start","message":{"role":"user","content":[{"type":"text","text":"hello"}]}}"#,
        r#"{"type":"message_update","assistantMessageEvent":{"type":"text_delta","contentIndex":0,"delta":"hi there"}}"#,
        r#"{"type":"turn_end","message":{"role":"assistant","usage":{"totalTokens":42}},"toolResults":[]}"#,
    ];
    let _session_id = spawn_emitter_session(&rt, &label, "emitter", &emitter_lines);

    // Flip the parser on with no script: the emitter is already running.
    let switch = ipc::Request::ChatSwitch(ipc::ChatSwitchRequest {
        session: "emitter".into(),
        cli: "pi".into(),
        chat: true,
        script: ipc::SwitchScript::None,
        resume: None,
    });
    // The reply may be nothing (no Error frame) — an RPC connection with no
    // reply still succeeds when the handler produced no frame.
    let _ = rt.block_on(rpc(&label, switch));

    // A client that subscribes sees the parsed events as they stream.
    let (seen, replayed) = rt.block_on(async {
        let (mut r, mut w) = connect_on(&label).await.expect("connect");
        let attach = ipc::Request::Attach(ipc::AttachRequest {
            name: "emitter".into(),
        });
        ipc::write_request(&mut w, &attach)
            .await
            .expect("write attach");
        w.flush().await.expect("flush");

        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        let mut seen: Vec<crate::chat::ChatEvent> = Vec::new();
        loop {
            let Some(frame) = tokio::time::timeout_at(deadline, ipc::read_frame(&mut r))
                .await
                .expect("timeout waiting for chat events")
                .expect("read")
            else {
                break;
            };
            match frame {
                ipc::Frame::ChatEvent { item, .. } => {
                    seen.push(item.event);
                    if matches!(seen.last(), Some(crate::chat::ChatEvent::TurnDone { .. })) {
                        break;
                    }
                }
                _ => continue,
            }
        }

        // A late client asks for everything from zero.
        let sync = ipc::Request::ChatSync(ipc::ChatSyncRequest {
            session: "emitter".into(),
            after_seq: 0,
        });
        let sync_frame = {
            let (mut r2, mut w2) = connect_on(&label).await.expect("connect 2");
            ipc::write_request(&mut w2, &sync)
                .await
                .expect("write sync");
            w2.flush().await.expect("flush");
            let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
            loop {
                let Some(frame) = tokio::time::timeout_at(deadline, ipc::read_frame(&mut r2))
                    .await
                    .expect("timeout waiting for sync")
                    .expect("read")
                else {
                    panic!("closed before sync reply");
                };
                match frame {
                    ipc::Frame::Output { .. } | ipc::Frame::ChatEvent { .. } => continue,
                    ipc::Frame::ChatSync(resp) => break resp,
                    other => panic!("unexpected frame {other:?}"),
                }
            }
        };
        (seen, sync_frame)
    });

    // The live stream parsed the emitted conversation.
    assert!(
        seen.contains(&crate::chat::ChatEvent::SessionInfo {
            cli: "pi".into(),
            session_id: Some("sid-1".into()),
        }),
        "live stream saw {seen:?}"
    );
    assert!(
        seen.contains(&crate::chat::ChatEvent::UserMessage {
            text: "hello".into()
        }),
        "live stream saw {seen:?}"
    );
    assert!(
        seen.contains(&crate::chat::ChatEvent::AssistantDelta {
            text: "hi there".into()
        }),
        "live stream saw {seen:?}"
    );

    // The ring replays the same events for the late client, with metadata.
    assert!(replayed.chat, "parser should be on");
    assert_eq!(replayed.cli.as_deref(), Some("pi"));
    assert_eq!(replayed.session_id.as_deref(), Some("sid-1"));
    let kinds: Vec<_> = replayed.events.iter().map(|item| item.seq).collect();
    assert_eq!(kinds, vec![1, 2, 3, 4], "replay: {:?}", replayed.events);
}

#[test]
fn chat_switch_writes_the_launch_line_and_chat_send_formats_input() {
    let label = unique_label();
    start_daemon_sync(&label);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // A `cat` session stands in for the shell: whatever the daemon writes
    // comes back as output, so scripts and formatted input are observable.
    let create = ipc::Request::Create(ipc::CreateRequest {
        name: "echoer".into(),
        cwd: None,
        argv: vec!["cat".into()],
        cols: 80,
        rows: 24,
    });
    match rt.block_on(rpc(&label, create)) {
        Ok(ipc::Frame::Create(c)) => {
            let _ = c.session_id;
        }
        other => panic!("expected Create reply, got {other:?}"),
    }

    // FreshLaunch types the chat-mode line into the PTY.
    let switch = ipc::Request::ChatSwitch(ipc::ChatSwitchRequest {
        session: "echoer".into(),
        cli: "pi".into(),
        chat: true,
        script: ipc::SwitchScript::FreshLaunch,
        resume: None,
    });
    let _ = rt.block_on(rpc(&label, switch));

    // Once the CLI would report a session id, input is formatted as a
    // relaunch command line with that id.
    let send = ipc::Request::ChatSend(ipc::ChatSendRequest {
        session: "echoer".into(),
        text: "what is 2+2".into(),
    });
    let _ = rt.block_on(rpc(&label, send));

    // Read the session's echoed output and look for both written lines.
    let echoed = rt.block_on(async {
        let (mut r, mut w) = connect_on(&label).await.expect("connect");
        let attach = ipc::Request::Attach(ipc::AttachRequest {
            name: "echoer".into(),
        });
        ipc::write_request(&mut w, &attach).await.expect("attach");
        w.flush().await.expect("flush");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        let mut buf = String::new();
        loop {
            let frame = tokio::time::timeout_at(deadline, ipc::read_frame(&mut r))
                .await
                .ok()
                .and_then(|f| f.ok().flatten());
            match frame {
                Some(ipc::Frame::Output { bytes, .. }) => {
                    buf.push_str(&String::from_utf8_lossy(&bytes));
                    if buf.contains("what is 2+2") {
                        break;
                    }
                }
                Some(ipc::Frame::Attach { replay, .. }) => {
                    buf.push_str(&String::from_utf8_lossy(&replay));
                }
                _ => break,
            }
        }
        buf
    });

    assert!(
        echoed.contains("pi --mode json"),
        "launch line missing from PTY: {echoed:?}"
    );
    assert!(
        echoed.contains("pi --mode json \"what is 2+2\""),
        "formatted input missing from PTY: {echoed:?}"
    );
}

#[test]
fn flipping_back_to_grid_stops_the_chat_parser() {
    let label = unique_label();
    start_daemon_sync(&label);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let emitter_lines = [
        r#"{"type":"session","version":3,"id":"sid-2","cwd":"/tmp"}"#,
        r#"{"type":"message_start","message":{"role":"user","content":[{"type":"text","text":"q"}]}}"#,
    ];
    let _ = spawn_emitter_session(&rt, &label, "flipper", &emitter_lines);

    let on = ipc::Request::ChatSwitch(ipc::ChatSwitchRequest {
        session: "flipper".into(),
        cli: "pi".into(),
        chat: true,
        script: ipc::SwitchScript::None,
        resume: None,
    });
    let _ = rt.block_on(rpc(&label, on));

    // Wait until the ring has the parsed events…
    let wait_sync = rt.block_on(async {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let (mut r, mut w) = connect_on(&label).await.expect("connect");
            let sync = ipc::Request::ChatSync(ipc::ChatSyncRequest {
                session: "flipper".into(),
                after_seq: 0,
            });
            ipc::write_request(&mut w, &sync).await.expect("write");
            w.flush().await.expect("flush");
            let inner = tokio::time::timeout_at(deadline, ipc::read_frame(&mut r))
                .await
                .expect("timeout")
                .expect("read")
                .expect("frame");
            match inner {
                ipc::Frame::ChatSync(resp) if !resp.events.is_empty() => {
                    break resp;
                }
                ipc::Frame::ChatSync(_)
                | ipc::Frame::Output { .. }
                | ipc::Frame::ChatEvent { .. } => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
                _ => panic!("unexpected"),
            }
        }
    });
    assert!(wait_sync.chat);

    // …then flip off and confirm the ring stops growing while the emitter
    // keeps producing (a second emitter burst arrives as Output only).
    let off = ipc::Request::ChatSwitch(ipc::ChatSwitchRequest {
        session: "flipper".into(),
        cli: "pi".into(),
        chat: false,
        script: ipc::SwitchScript::None,
        resume: None,
    });
    let _ = rt.block_on(rpc(&label, off));

    let after = rt.block_on(async {
        let (mut r, mut w) = connect_on(&label).await.expect("connect");
        let sync = ipc::Request::ChatSync(ipc::ChatSyncRequest {
            session: "flipper".into(),
            after_seq: 0,
        });
        ipc::write_request(&mut w, &sync).await.expect("write");
        w.flush().await.expect("flush");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        loop {
            let frame = tokio::time::timeout_at(deadline, ipc::read_frame(&mut r))
                .await
                .expect("timeout")
                .expect("read");
            match frame {
                Some(ipc::Frame::ChatSync(resp)) => break resp,
                _ => continue,
            }
        }
    });
    assert!(!after.chat, "parser should be off");
    assert_eq!(
        after.events.len(),
        wait_sync.events.len(),
        "the ring must stop growing once the parser is off"
    );
}
