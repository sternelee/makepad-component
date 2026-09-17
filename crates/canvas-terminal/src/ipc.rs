//! Wire protocol shared between the GUI and the bundled PTY daemon
//! (`canvas-terminal --daemon`).
//!
//! Transport: a single local byte stream (Unix domain socket on Unix, named
//! pipe on Windows) provided by `rmux-ipc`. Every message is framed as:
//!
//! ```text
//!   [u32 big-endian length][u8 tag][payload: length-1 bytes]
//! ```
//!
//! `length` counts the tag byte plus the payload, so the total body read after
//! the 4-byte length prefix is `length` bytes. Control payloads are JSON; the
//! byte-carrying messages (`Write`, `Output`, and the `Attach` replay/command)
//! lay bytes out inline after a fixed-width header so terminal throughput
//! stays cheap.
//!
//! The protocol is intentionally minimal — one connection corresponds to one
//! terminal session. After `Create`/`Attach` the daemon streams `Output`
//! frames for that session back over the same connection until `Ended`.
//
// This module is compiled into two binaries (GUI + daemon); each side uses a
// different subset of the codec, so per-side dead-code is expected.
#![allow(dead_code)]

use std::io;

use crate::chat::event::Sequenced;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Socket/pipe label used to derive the per-user `rmux-ipc` endpoint.
pub const LABEL: &str = "canvas-terminal";

/// Env override for the endpoint label.
///
/// Exists so a test (or a second, isolated instance) can run its own daemon
/// without colliding with the user's. Normal runs leave it unset.
pub const LABEL_ENV: &str = "CANVAS_TERMINAL_DAEMON_LABEL";

/// The endpoint label this process should use.
pub fn label() -> String {
    std::env::var(LABEL_ENV)
        .ok()
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| LABEL.to_owned())
}

/// Set to `"1"` in every PTY the daemon spawns. Mirrors herdr's
/// `HERDR_ENV=1`: a process running inside a canvas-terminal card checks this
/// before shelling out to `canvas-terminal ipc ...` to talk to a sibling
/// card, so a process running *outside* canvas-terminal never mistakes a
/// stray socket for one it owns.
pub const ENV_MARKER_VAR: &str = "CANVAS_TERMINAL_ENV";

/// Per-frame byte ceiling (16 MiB). Terminal reads are chunked well below
/// this; the guard exists only to reject a corrupted length prefix.
const MAX_FRAME: usize = 16 * 1024 * 1024;

// ── message type tags ──────────────────────────────────────────────────────
const TAG_CREATE_REQ: u8 = 0x01;
const TAG_ATTACH_REQ: u8 = 0x02;
const TAG_WRITE_REQ: u8 = 0x03;
const TAG_RESIZE_REQ: u8 = 0x04;
const TAG_KILL_REQ: u8 = 0x05;
const TAG_LIST_REQ: u8 = 0x06;
const TAG_RENAME_REQ: u8 = 0x07;
// Chat-view requests. Same connection, same framing: a chat view is just a
// second interpretation of a terminal session's PTY stream, so the requests
// address sessions by name like everything else.
const TAG_CHAT_SEND_REQ: u8 = 0x08;
const TAG_CHAT_SWITCH_REQ: u8 = 0x09;
const TAG_CHAT_SYNC_REQ: u8 = 0x0a;
// `Resume` — relaunch a session the daemon only knows about from its own
// persisted registry (see `crate::daemon_persist`); used after a *daemon*
// restart, when `Attach` would fail because the in-memory session table is
// empty. Appended after the chat tags: existing tag values are never
// renumbered once shipped, so an old client talking to a new daemon (or vice
// versa) degrades to "unknown request/response" instead of misreading a
// frame.
const TAG_RESUME_REQ: u8 = 0x0b;
// Agent automation requests (inter-agent/script control surface): status
// snapshots and an event-driven wait, mirroring herdr's `agent.list` /
// `agent.wait`. Additive, like everything above.
const TAG_AGENT_WAIT_REQ: u8 = 0x0c;
const TAG_AGENT_LIST_REQ: u8 = 0x0d;

const TAG_CREATE_RES: u8 = 0x81;
const TAG_ATTACH_RES: u8 = 0x82;
const TAG_LIST_RES: u8 = 0x86;
const TAG_CHAT_SYNC_RES: u8 = 0x88;
const TAG_AGENT_STATUS_RES: u8 = 0x89;
const TAG_AGENT_LIST_RES: u8 = 0x8a;
const TAG_OUTPUT: u8 = 0x10;
const TAG_CHAT_EVENT: u8 = 0x30;
const TAG_ENDED: u8 = 0x20;
const TAG_ERROR: u8 = 0xFF;

// ── request payloads ───────────────────────────────────────────────────────

/// `Create` — spawn a new PTY session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateRequest {
    pub name: String,
    pub cwd: Option<String>,
    pub argv: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

/// `Attach` — re-attach to an existing named session for replay + live output.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttachRequest {
    pub name: String,
}

/// `Resize` — change a session's PTY grid size.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResizeRequest {
    pub session_id: u64,
    pub cols: u16,
    pub rows: u16,
}

/// `Kill` — terminate a session's child process.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KillRequest {
    pub session_id: u64,
}

/// `Rename` — give a session a new name (the GUI's `/rename` command keeps
/// the daemon name in sync so re-attach after a restart finds the new name).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenameRequest {
    pub session_id: u64,
    pub name: String,
}

/// `Resume` — relaunch `name` from the daemon's persisted registry (argv,
/// cwd, and — when it was hosting a chat CLI — the provider and the CLI's
/// own session id). Response is a plain `Create` on success: the caller
/// treats a resumed session exactly like a freshly created one from there.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResumeRequest {
    pub name: String,
    pub cols: u16,
    pub rows: u16,
}

/// Which lifecycle states `AgentWait` should resolve on. Deliberately a
/// small vocabulary for v1: `Blocked`/`Done` need the CLI adapters to
/// surface an approval/completion signal they do not emit yet (see the
/// project plan's "Out of scope" notes). `Idle` means the session's chat
/// parser is off or reports `busy == false`; `Working` means `busy == true`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentWaitState {
    Idle,
    Working,
}

/// `AgentWait` — resolve when the named session's status matches one of
/// `until`, the session exits, or `timeout_ms` elapses. Event-driven on the
/// daemon side (a waiter list resolved from the PTY reader thread), not
/// polled — mirrors herdr's `agent.wait`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentWaitRequest {
    pub session: String,
    pub until: Vec<AgentWaitState>,
    pub timeout_ms: u64,
}

/// One session's agent status, reported by both `AgentWait`'s resolution
/// and `AgentList`'s snapshot.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentStatusInfo {
    pub name: String,
    pub alive: bool,
    pub busy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_session_id: Option<String>,
    /// True when `AgentWait` resolved because `timeout_ms` elapsed rather
    /// than an observed state change.
    #[serde(default)]
    pub timed_out: bool,
}

/// `AgentList` response — a status snapshot per live session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentListResponse {
    pub agents: Vec<AgentStatusInfo>,
}

// ── response payloads ──────────────────────────────────────────────────────

/// `Create` response — the newly assigned session id.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateResponse {
    pub session_id: u64,
}

/// `List` response — one entry per live, recently-live, or daemon-known
/// (see `crate::daemon_persist`) session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionInfo {
    pub name: String,
    /// 0 when the session has no live/recent in-memory record at all (a
    /// registry-only entry from a prior daemon lifetime) — `Resume` assigns
    /// a fresh id once it actually spawns.
    pub session_id: u64,
    pub alive: bool,
    /// True when `Resume` can relaunch this session by name even though it
    /// is not currently live (daemon restart recovery). Always `false` for
    /// a live session.
    #[serde(default)]
    pub resumable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListResponse {
    pub sessions: Vec<SessionInfo>,
}

/// `Error` — something went wrong server-side.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub message: String,
}

// ── chat payloads ──────────────────────────────────────────────────────────

/// `ChatSend` — a composer prompt, forwarded to the session's hosted CLI in
/// whatever form its adapter uses (JSONL on stdin, or a relaunch command).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatSendRequest {
    /// The session (terminal card) name.
    pub session: String,
    pub text: String,
}

/// What the daemon should type into the PTY while flipping a session's view.
///
/// The scripts exist because the daemon owns the PTY: switching means the
/// current CLI mode has to exit and the other has to start, in the same
/// terminal, without losing the shell in between.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SwitchScript {
    /// Flip the parser only; the right process is already running. Used by
    /// tests that spawn a JSON emitter directly, and by a client that already
    /// typed the launch line itself.
    None,
    /// The shell is idle: type the chat-mode launch line (a fresh agent).
    FreshLaunch,
    /// A TUI CLI is running: exit it, then launch the chat mode with the
    /// CLI's own session id so the conversation continues.
    FromTui,
    /// The chat mode is running: exit it, then launch the plain TUI.
    FromChat,
}

/// `ChatSwitch` — flip a session between grid and chat views.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatSwitchRequest {
    pub session: String,
    /// `"pi"` / `"claude"` / `"codex"` — which CLI hosts the chat.
    pub cli: String,
    /// `false` → grid view (chat parser off), `true` → chat view.
    pub chat: bool,
    pub script: SwitchScript,
    /// The CLI's own session id when known, so relaunches resume the same
    /// conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume: Option<String>,
}

/// `ChatSync` — replay the event ring after `after_seq`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatSyncRequest {
    pub session: String,
    #[serde(default)]
    pub after_seq: u64,
}

/// `ChatSync` response — the missed events, oldest first. Any of these may
/// also arrive again on the live stream, so the client applies them by
/// sequence number.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatSyncResponse {
    pub events: Vec<Sequenced>,
    /// Whether the session's chat parser is currently on.
    pub chat: bool,
    /// The hosting CLI, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    /// The CLI's own session id, when it reported one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

// ── typed envelopes ────────────────────────────────────────────────────────

/// A client→daemon message.
#[derive(Debug, Clone)]
pub enum Request {
    Create(CreateRequest),
    Attach(AttachRequest),
    Write {
        session_id: u64,
        bytes: Vec<u8>,
    },
    Resize(ResizeRequest),
    Kill(KillRequest),
    Rename(RenameRequest),
    List,
    /// Send a composer prompt to a session's hosted CLI (formatted by the
    /// adapter: JSONL to stdin, or a relaunch command line).
    ChatSend(ChatSendRequest),
    /// Flip a session between the terminal-grid view and the chat view,
    /// optionally writing the launch/exit script into the PTY.
    ChatSwitch(ChatSwitchRequest),
    /// Replay the chat event ring after `after_seq`.
    ChatSync(ChatSyncRequest),
    /// Relaunch a session the daemon's own registry remembers, even though
    /// no live PTY exists for it (daemon restart recovery).
    Resume(ResumeRequest),
    /// Resolve when a session's agent status matches, it exits, or a
    /// timeout elapses. The inter-agent/script control surface.
    AgentWait(AgentWaitRequest),
    /// Snapshot every live session's agent status.
    AgentList,
}

/// A daemon→client message.
#[derive(Debug, Clone)]
pub enum Frame {
    Create(CreateResponse),
    Attach {
        session_id: u64,
        /// The shell command the session was spawned with (terminal card subtitle).
        command: String,
        replay: Vec<u8>,
    },
    List(ListResponse),
    Output {
        session_id: u64,
        bytes: Vec<u8>,
    },
    Ended {
        session_id: u64,
    },
    /// The ring replay answering a `ChatSync`.
    ChatSync(ChatSyncResponse),
    /// One chat event, in sequence order. `session_id` stays out of the
    /// payload because it is needed for routing before the body is decoded.
    ChatEvent {
        session_id: u64,
        item: Sequenced,
    },
    /// Resolution of an `AgentWait`, or a one-session status query.
    AgentStatus(AgentStatusInfo),
    /// Snapshot answering `AgentList`.
    AgentList(AgentListResponse),
    Error(ErrorResponse),
}

// ── raw frame codec ────────────────────────────────────────────────────────

/// Reads one framed message. Returns `Ok(None)` on clean EOF.
async fn read_frame_raw<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<(u8, Vec<u8>)>> {
    let mut len_buf = [0u8; 4];
    match r.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    if len == 0 || len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid frame length {len}"),
        ));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    let tag = buf[0];
    Ok(Some((tag, buf[1..].to_vec())))
}

/// Writes one framed message.
async fn write_frame_raw<W: AsyncWrite + Unpin>(
    w: &mut W,
    tag: u8,
    payload: &[u8],
) -> io::Result<()> {
    let len = (1 + payload.len()) as u32;
    w.write_all(&len.to_be_bytes()).await?;
    let mut head = [0u8; 1];
    head[0] = tag;
    w.write_all(&head).await?;
    w.write_all(payload).await?;
    w.flush().await?;
    Ok(())
}

fn io_invalid(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

fn take_u64(p: &[u8]) -> io::Result<(u64, &[u8])> {
    if p.len() < 8 {
        return Err(io_invalid("short u64 payload"));
    }
    let mut a = [0u8; 8];
    a.copy_from_slice(&p[..8]);
    Ok((u64::from_be_bytes(a), &p[8..]))
}

fn take_u32(p: &[u8]) -> io::Result<(u32, &[u8])> {
    if p.len() < 4 {
        return Err(io_invalid("short u32 payload"));
    }
    let mut a = [0u8; 4];
    a.copy_from_slice(&p[..4]);
    Ok((u32::from_be_bytes(a), &p[4..]))
}

// ── request encode/decode ──────────────────────────────────────────────────

/// Writes a request frame.
pub async fn write_request<W: AsyncWrite + Unpin>(w: &mut W, req: &Request) -> io::Result<()> {
    match req {
        Request::Create(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_CREATE_REQ, &p).await
        }
        Request::Attach(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_ATTACH_REQ, &p).await
        }
        Request::Write { session_id, bytes } => {
            let mut p = Vec::with_capacity(8 + bytes.len());
            p.extend_from_slice(&session_id.to_be_bytes());
            p.extend_from_slice(bytes);
            write_frame_raw(w, TAG_WRITE_REQ, &p).await
        }
        Request::Resize(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_RESIZE_REQ, &p).await
        }
        Request::Kill(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_KILL_REQ, &p).await
        }
        Request::Rename(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_RENAME_REQ, &p).await
        }
        Request::List => write_frame_raw(w, TAG_LIST_REQ, &[]).await,
        Request::ChatSend(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_CHAT_SEND_REQ, &p).await
        }
        Request::ChatSwitch(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_CHAT_SWITCH_REQ, &p).await
        }
        Request::ChatSync(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_CHAT_SYNC_REQ, &p).await
        }
        Request::Resume(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_RESUME_REQ, &p).await
        }
        Request::AgentWait(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_WAIT_REQ, &p).await
        }
        Request::AgentList => write_frame_raw(w, TAG_AGENT_LIST_REQ, &[]).await,
    }
}

/// Reads a request frame. Returns `Ok(None)` on clean EOF.
pub async fn read_request<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Request>> {
    let Some((tag, p)) = read_frame_raw(r).await? else {
        return Ok(None);
    };
    let req = match tag {
        TAG_CREATE_REQ => Request::Create(serde_json::from_slice(&p)?),
        TAG_ATTACH_REQ => Request::Attach(serde_json::from_slice(&p)?),
        TAG_WRITE_REQ => {
            let (id, rest) = take_u64(&p)?;
            Request::Write {
                session_id: id,
                bytes: rest.to_vec(),
            }
        }
        TAG_RESIZE_REQ => Request::Resize(serde_json::from_slice(&p)?),
        TAG_KILL_REQ => Request::Kill(serde_json::from_slice(&p)?),
        TAG_RENAME_REQ => Request::Rename(serde_json::from_slice(&p)?),
        TAG_LIST_REQ => Request::List,
        TAG_CHAT_SEND_REQ => Request::ChatSend(serde_json::from_slice(&p)?),
        TAG_CHAT_SWITCH_REQ => Request::ChatSwitch(serde_json::from_slice(&p)?),
        TAG_CHAT_SYNC_REQ => Request::ChatSync(serde_json::from_slice(&p)?),
        TAG_RESUME_REQ => Request::Resume(serde_json::from_slice(&p)?),
        TAG_AGENT_WAIT_REQ => Request::AgentWait(serde_json::from_slice(&p)?),
        TAG_AGENT_LIST_REQ => Request::AgentList,
        _ => return Err(io_invalid(&format!("unknown request tag {tag:#x}"))),
    };
    Ok(Some(req))
}

// ── response (Frame) encode/decode ─────────────────────────────────────────

/// Writes a response frame.
pub async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, frame: &Frame) -> io::Result<()> {
    match frame {
        Frame::Create(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_CREATE_RES, &p).await
        }
        Frame::Attach {
            session_id,
            command,
            replay,
        } => {
            // [id u64][replay_len u32][replay][cmd_len u32][cmd bytes] — all
            // inline binary so the bulk of the frame stays cheap.
            let mut p = Vec::with_capacity(12 + replay.len() + command.len());
            p.extend_from_slice(&session_id.to_be_bytes());
            p.extend_from_slice(&(replay.len() as u32).to_be_bytes());
            p.extend_from_slice(replay);
            p.extend_from_slice(&(command.len() as u32).to_be_bytes());
            p.extend_from_slice(command.as_bytes());
            write_frame_raw(w, TAG_ATTACH_RES, &p).await
        }
        Frame::List(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_LIST_RES, &p).await
        }
        Frame::Output { session_id, bytes } => {
            let mut p = Vec::with_capacity(8 + bytes.len());
            p.extend_from_slice(&session_id.to_be_bytes());
            p.extend_from_slice(bytes);
            write_frame_raw(w, TAG_OUTPUT, &p).await
        }
        Frame::Ended { session_id } => {
            write_frame_raw(w, TAG_ENDED, &session_id.to_be_bytes()).await
        }
        Frame::ChatSync(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_CHAT_SYNC_RES, &p).await
        }
        Frame::ChatEvent { session_id, item } => {
            // [session_id u64][event json]: routing stays cheap, and the
            // event body round-trips the exact wire shape of `chat::event`.
            let body = serde_json::to_vec(item)?;
            let mut p = Vec::with_capacity(8 + body.len());
            p.extend_from_slice(&session_id.to_be_bytes());
            p.extend_from_slice(&body);
            write_frame_raw(w, TAG_CHAT_EVENT, &p).await
        }
        Frame::AgentStatus(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_STATUS_RES, &p).await
        }
        Frame::AgentList(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_LIST_RES, &p).await
        }
        Frame::Error(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_ERROR, &p).await
        }
    }
}

/// Reads a response frame. Returns `Ok(None)` on clean EOF.
pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Option<Frame>> {
    let Some((tag, p)) = read_frame_raw(r).await? else {
        return Ok(None);
    };
    let frame = match tag {
        TAG_CREATE_RES => Frame::Create(serde_json::from_slice(&p)?),
        TAG_ATTACH_RES => {
            let (id, rest) = take_u64(&p)?;
            let (rlen, rest) = take_u32(rest)?;
            let rlen = rlen as usize;
            if rest.len() < rlen {
                return Err(io_invalid("attach replay truncated"));
            }
            let replay = rest[..rlen].to_vec();
            let (clen, rest) = take_u32(&rest[rlen..])?;
            let clen = clen as usize;
            if rest.len() < clen {
                return Err(io_invalid("attach command truncated"));
            }
            let command = std::str::from_utf8(&rest[..clen])
                .map_err(|_| io_invalid("attach command not utf8"))?
                .to_string();
            Frame::Attach {
                session_id: id,
                command,
                replay,
            }
        }
        TAG_LIST_RES => Frame::List(serde_json::from_slice(&p)?),
        TAG_OUTPUT => {
            let (id, rest) = take_u64(&p)?;
            Frame::Output {
                session_id: id,
                bytes: rest.to_vec(),
            }
        }
        TAG_ENDED => {
            let (id, _) = take_u64(&p)?;
            Frame::Ended { session_id: id }
        }
        TAG_CHAT_SYNC_RES => Frame::ChatSync(serde_json::from_slice(&p)?),
        TAG_CHAT_EVENT => {
            let (session_id, rest) = take_u64(&p)?;
            Frame::ChatEvent {
                session_id,
                item: serde_json::from_slice(rest)?,
            }
        }
        TAG_AGENT_STATUS_RES => Frame::AgentStatus(serde_json::from_slice(&p)?),
        TAG_AGENT_LIST_RES => Frame::AgentList(serde_json::from_slice(&p)?),
        TAG_ERROR => Frame::Error(serde_json::from_slice(&p)?),
        _ => return Err(io_invalid(&format!("unknown response tag {tag:#x}"))),
    };
    Ok(Some(frame))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    async fn roundtrip(req: Request) -> Request {
        // 8 KiB is comfortably larger than any control frame so the write
        // completes before the read starts (avoids a fill/drain deadlock).
        let (mut a, mut b) = duplex(8 * 1024);
        write_request(&mut a, &req).await.unwrap();
        read_request(&mut b).await.unwrap().unwrap()
    }

    #[tokio::test]
    async fn roundtrips_create() {
        let r = Request::Create(CreateRequest {
            name: "term".into(),
            cwd: Some("~/x".into()),
            argv: vec!["/bin/zsh".into(), "-lc".into()],
            cols: 80,
            rows: 24,
        });
        match roundtrip(r).await {
            Request::Create(c) => {
                assert_eq!(c.name, "term");
                assert_eq!(c.cwd.as_deref(), Some("~/x"));
                assert_eq!(c.cols, 80);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[tokio::test]
    async fn roundtrips_write_bytes() {
        let r = Request::Write {
            session_id: 42,
            bytes: vec![0x1b, b'x', 0, 0xff],
        };
        match roundtrip(r).await {
            Request::Write { session_id, bytes } => {
                assert_eq!(session_id, 42);
                assert_eq!(bytes, vec![0x1b, b'x', 0, 0xff]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[tokio::test]
    async fn roundtrips_attach_frame() {
        let f = Frame::Attach {
            session_id: 7,
            command: "zsh".into(),
            replay: vec![1, 2, 3, 4],
        };
        let (mut a, mut b) = duplex(64 * 1024);
        write_frame(&mut a, &f).await.unwrap();
        match read_frame(&mut b).await.unwrap().unwrap() {
            Frame::Attach {
                session_id,
                command,
                replay,
            } => {
                assert_eq!(session_id, 7);
                assert_eq!(command, "zsh");
                assert_eq!(replay, vec![1, 2, 3, 4]);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[tokio::test]
    async fn eof_returns_none() {
        let (a, mut b) = duplex(8 * 1024);
        drop(a);
        assert!(read_request(&mut b).await.unwrap().is_none());
    }
}
