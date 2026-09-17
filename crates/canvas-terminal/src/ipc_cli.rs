//! `canvas-terminal ipc <verb>` — a thin one-shot CLI client over the same
//! daemon protocol the GUI uses (`crate::ipc`). This is the control surface
//! behind inter-agent communication: a CLI agent running inside one
//! terminal card (see `ipc::ENV_MARKER_VAR`) shells out to this instead of
//! linking against the GUI, to message, wait on, or read a sibling card.
//! Mirrors herdr's CLI wrappers over its socket API (`herdr agent wait`,
//! `herdr agent prompt`, ...).
//!
//! Every verb opens its own short-lived connection: send one request, read
//! until a matching response frame arrives, print JSON (or plain text for
//! `read`) to stdout, then exit. Exit codes: `0` success, `1` the daemon
//! answered but the outcome was negative (e.g. a wait timed out), `2` bad
//! usage or a request-level error.

use crate::chat::{ChatCardState, Row};
use crate::ipc;
use crate::terminal::session::{connect_stream, ensure_daemon, runtime, spawn_writer_task};

/// Parse and run an `ipc` subcommand. `args` is everything after `ipc` on
/// the command line. Returns the process exit code.
pub fn run(args: &[String]) -> i32 {
    let Some((verb, rest)) = args.split_first() else {
        eprintln!("usage: canvas-terminal ipc <list|send|wait|read> ...");
        return 2;
    };
    match verb.as_str() {
        "list" => cmd_list(),
        "send" => cmd_send(rest),
        "wait" => cmd_wait(rest),
        "read" => cmd_read(rest),
        other => {
            eprintln!("canvas-terminal ipc: unknown verb '{other}' (expected list|send|wait|read)");
            2
        }
    }
}

/// The value following the first `name` flag, if present.
fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// Every value following a `name` flag (repeatable flags, e.g. `--until`).
fn flags<'a>(args: &'a [String], name: &str) -> Vec<&'a str> {
    args.iter()
        .enumerate()
        .filter(|(_, a)| *a == name)
        .filter_map(|(i, _)| args.get(i + 1))
        .map(String::as_str)
        .collect()
}

/// Connect (spawning the daemon detached if it is not already running, like
/// the GUI does), send one request, and read frames until `want` resolves.
fn request_once<T>(
    req: ipc::Request,
    mut want: impl FnMut(ipc::Frame) -> Option<Result<T, String>>,
) -> Result<T, String> {
    ensure_daemon().map_err(|e| format!("daemon unavailable: {e}"))?;
    let rt = runtime();
    let (mut r, w) = rt
        .block_on(connect_stream())
        .map_err(|e| format!("connect: {e}"))?;
    let (writer_tx, _writer_task) = spawn_writer_task(w);
    rt.block_on(async { writer_tx.send(req).await })
        .map_err(|_| "writer closed before request".to_string())?;
    rt.block_on(async {
        loop {
            match ipc::read_frame(&mut r).await {
                Ok(Some(frame)) => {
                    if let Some(out) = want(frame) {
                        return out;
                    }
                }
                Ok(None) => return Err("daemon closed the connection".into()),
                Err(e) => return Err(format!("read response: {e}")),
            }
        }
    })
}

/// Like [`request_once`], but a bounded wait for a definitive answer: used
/// by `send`, which has no success frame of its own — only silence (fine)
/// or an `Error` (worth surfacing) follow it.
fn request_and_wait_briefly(
    req: ipc::Request,
    timeout: std::time::Duration,
    mut want: impl FnMut(ipc::Frame) -> Option<Result<(), String>>,
) -> Result<(), String> {
    ensure_daemon().map_err(|e| format!("daemon unavailable: {e}"))?;
    let rt = runtime();
    let (mut r, w) = rt
        .block_on(connect_stream())
        .map_err(|e| format!("connect: {e}"))?;
    let (writer_tx, _writer_task) = spawn_writer_task(w);
    rt.block_on(async { writer_tx.send(req).await })
        .map_err(|_| "writer closed before request".to_string())?;
    rt.block_on(async {
        let read_loop = async {
            loop {
                match ipc::read_frame(&mut r).await {
                    Ok(Some(frame)) => {
                        if let Some(out) = want(frame) {
                            return out;
                        }
                    }
                    Ok(None) => return Err("daemon closed the connection".into()),
                    Err(e) => return Err(format!("read response: {e}")),
                }
            }
        };
        match tokio::time::timeout(timeout, read_loop).await {
            Ok(outcome) => outcome,
            // No error arrived within the window: treat as accepted.
            Err(_) => Ok(()),
        }
    })
}

fn cmd_list() -> i32 {
    let result = request_once(ipc::Request::AgentList, |frame| match frame {
        ipc::Frame::AgentList(r) => Some(Ok(r)),
        ipc::Frame::Error(e) => Some(Err(e.message)),
        _ => None,
    });
    match result {
        Ok(r) => {
            println!("{}", serde_json::to_string(&r).unwrap_or_default());
            0
        }
        Err(e) => {
            eprintln!("canvas-terminal ipc list: {e}");
            2
        }
    }
}

fn cmd_send(args: &[String]) -> i32 {
    let Some(to) = flag(args, "--to") else {
        eprintln!("usage: canvas-terminal ipc send --to NAME --text TEXT");
        return 2;
    };
    let text = flag(args, "--text").unwrap_or("");
    let result = request_and_wait_briefly(
        ipc::Request::ChatSend(ipc::ChatSendRequest {
            session: to.to_string(),
            text: text.to_string(),
        }),
        std::time::Duration::from_millis(300),
        |frame| match frame {
            ipc::Frame::Error(e) => Some(Err(e.message)),
            _ => None,
        },
    );
    match result {
        Ok(()) => {
            println!("{{\"ok\":true}}");
            0
        }
        Err(e) => {
            eprintln!("canvas-terminal ipc send: {e}");
            2
        }
    }
}

fn cmd_wait(args: &[String]) -> i32 {
    let Some(target) = flag(args, "--for") else {
        eprintln!(
            "usage: canvas-terminal ipc wait --for NAME [--until idle|working] [--timeout-ms N]"
        );
        return 2;
    };
    let mut until = Vec::new();
    for u in flags(args, "--until") {
        match u {
            "idle" => until.push(ipc::AgentWaitState::Idle),
            "working" => until.push(ipc::AgentWaitState::Working),
            other => {
                eprintln!(
                    "canvas-terminal ipc wait: unknown --until '{other}' (expected idle|working)"
                );
                return 2;
            }
        }
    }
    if until.is_empty() {
        // Idle is what "the agent is done and ready for input" means in
        // practice — the useful default for "wait for this agent to finish".
        until.push(ipc::AgentWaitState::Idle);
    }
    let timeout_ms: u64 = flag(args, "--timeout-ms")
        .and_then(|v| v.parse().ok())
        .unwrap_or(30_000);
    let result = request_once(
        ipc::Request::AgentWait(ipc::AgentWaitRequest {
            session: target.to_string(),
            until,
            timeout_ms,
        }),
        |frame| match frame {
            ipc::Frame::AgentStatus(s) => Some(Ok(s)),
            ipc::Frame::Error(e) => Some(Err(e.message)),
            _ => None,
        },
    );
    match result {
        Ok(status) => {
            let timed_out = status.timed_out;
            println!("{}", serde_json::to_string(&status).unwrap_or_default());
            if timed_out {
                1
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("canvas-terminal ipc wait: {e}");
            2
        }
    }
}

fn cmd_read(args: &[String]) -> i32 {
    let Some(from) = flag(args, "--from") else {
        eprintln!("usage: canvas-terminal ipc read --from NAME [--lines N]");
        return 2;
    };
    let lines_limit: Option<usize> = flag(args, "--lines").and_then(|v| v.parse().ok());
    let result = request_once(
        ipc::Request::ChatSync(ipc::ChatSyncRequest {
            session: from.to_string(),
            after_seq: 0,
        }),
        |frame| match frame {
            ipc::Frame::ChatSync(r) => Some(Ok(r)),
            ipc::Frame::Error(e) => Some(Err(e.message)),
            _ => None,
        },
    );
    match result {
        Ok(resp) => {
            // Fold the full replayed ring through the same model the GUI
            // card uses, so the rendered transcript matches what a human
            // would see on the card.
            let mut card = ChatCardState::new();
            for item in resp.events {
                card.apply(item);
            }
            let mut rendered: Vec<String> = card.rows().iter().map(render_row).collect();
            if let Some(n) = lines_limit {
                if rendered.len() > n {
                    rendered = rendered.split_off(rendered.len() - n);
                }
            }
            for line in rendered {
                println!("{line}");
            }
            0
        }
        Err(e) => {
            eprintln!("canvas-terminal ipc read: {e}");
            2
        }
    }
}

fn render_row(row: &Row) -> String {
    match row {
        Row::User { text } => format!("> {text}"),
        Row::Assistant { text, .. } => text.clone(),
        Row::Reasoning { text } => format!("(thinking) {text}"),
        Row::Tool { name, ok, summary } => {
            let status = match ok {
                Some(true) => "ok",
                Some(false) => "failed",
                None => "running",
            };
            format!("[tool {name} {status}] {summary}")
        }
        Row::Notice { text } => format!("({text})"),
    }
}
