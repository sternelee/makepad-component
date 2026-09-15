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

use agent_core::{AgentEvent, Sequenced};
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

// Agent requests. Same connection, same framing; an agent is just another kind
// of session the daemon owns. Tags 0x08..=0x0f are reserved for them so a new
// terminal request can never collide with an agent one.
const TAG_AGENT_CREATE_REQ: u8 = 0x08;
const TAG_AGENT_PROMPT_REQ: u8 = 0x09;
const TAG_AGENT_STEER_REQ: u8 = 0x0a;
const TAG_AGENT_CANCEL_REQ: u8 = 0x0b;
const TAG_AGENT_KILL_REQ: u8 = 0x0c;
const TAG_AGENT_LIST_REQ: u8 = 0x0d;
const TAG_AGENT_ATTACH_REQ: u8 = 0x0e;
const TAG_AGENT_PERMISSION_REPLY_REQ: u8 = 0x0f;

const TAG_CREATE_RES: u8 = 0x81;
const TAG_ATTACH_RES: u8 = 0x82;
const TAG_LIST_RES: u8 = 0x86;
const TAG_AGENT_CREATED_RES: u8 = 0x88;
const TAG_AGENT_ATTACHED_RES: u8 = 0x89;
const TAG_AGENT_LIST_RES: u8 = 0x8a;
const TAG_OUTPUT: u8 = 0x10;
const TAG_AGENT_EVENT: u8 = 0x30;
const TAG_AGENT_PERMISSION_REQUEST: u8 = 0x31;
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

// ── response payloads ──────────────────────────────────────────────────────

/// `Create` response — the newly assigned session id.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateResponse {
    pub session_id: u64,
}

/// `List` response — one entry per live or recently-live session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionInfo {
    pub name: String,
    pub session_id: u64,
    pub alive: bool,
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

// ── agent payloads ─────────────────────────────────────────────────────────

/// How the daemon should build the model behind a new agent.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AgentProviderConfig {
    /// An OpenAI-compatible `/chat/completions` endpoint.
    ///
    /// The API key travels over this connection by design: the endpoint is a
    /// per-user local socket with the same trust level as the GUI process, and
    /// the daemon needs the key to talk to the provider. It is never logged and
    /// never echoed back in a response.
    OpenAi {
        api_url: String,
        api_key: String,
        model: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        temperature: Option<f32>,
    },
    /// Replays a canned conversation: no network, no key. Used by tests and by
    /// canvas/UI work that needs a live transcript without spending tokens.
    Scripted { turns: Vec<ScriptedTurnConfig> },
}

/// One canned model call for [`AgentProviderConfig::Scripted`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScriptedTurnConfig {
    #[serde(default)]
    pub text: String,
    /// `(tool name, arguments)` the scripted turn should ask for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<(String, serde_json::Value)>,
    /// Stream the text in pieces with this delay between them, so a UI under
    /// test has something to render.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_delay_ms: Option<u64>,
}

/// Which tool calls a new agent may run.
///
/// Defaults to [`Self::ReadOnly`]: an unattended agent should not be able to
/// change the workspace until the caller says so explicitly. An interactive
/// approval gate is a future variant of this, not a change to the others.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AgentPermissionConfig {
    /// Only tools that observe the workspace.
    #[default]
    ReadOnly,
    AllowAll,
    DenyAll,
    AllowList {
        tools: Vec<String>,
    },
    /// Ask the attached clients before every call, and run it only if one
    /// answers yes.
    ///
    /// The gate waits on the session's worker thread, so the agent's turn is
    /// paused rather than the daemon. A call nobody answers within
    /// `timeout_ms`, or a call asked while no client is attached, is denied —
    /// the fail-safe direction.
    Interactive {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
    },
}

/// `AgentCreate` — start a new agent session in the daemon.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentCreateRequest {
    /// Card name, unique among live agents.
    pub name: String,
    /// Workspace root every tool path is resolved against.
    pub cwd: String,
    pub provider: AgentProviderConfig,
    #[serde(default)]
    pub permission: AgentPermissionConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// When set, only these tools exist for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled_tools: Option<Vec<String>>,
}

/// `AgentCreate` response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentCreatedResponse {
    pub agent_id: u64,
    /// The runtime's own session id, for cursors and logs.
    pub session_id: u64,
    pub epoch: u64,
    /// The last sequence number already emitted, so a client that reconnects
    /// can ask for only what it is missing.
    pub seq: u64,
}

/// `AgentAttach` — subscribe to an existing agent's event stream.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentAttachRequest {
    pub agent_id: u64,
    /// Events at or below this sequence are skipped, so a client that already
    /// rendered part of the transcript does not render it twice.
    #[serde(default)]
    pub after_seq: u64,
}

/// `AgentAttach` response — what the client missed, then live events.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentAttachedResponse {
    pub agent_id: u64,
    pub session_id: u64,
    pub epoch: u64,
    pub name: String,
    pub cwd: String,
    pub busy: bool,
    /// Replayed events, oldest first. Any of these may also arrive again on the
    /// live stream, so the client applies them by sequence number.
    pub replay: Vec<Sequenced<AgentEvent>>,
}

/// `AgentPrompt` / `AgentSteer`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentInputRequest {
    pub agent_id: u64,
    pub text: String,
}

/// `AgentCancel` / `AgentKill`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentIdRequest {
    pub agent_id: u64,
}

/// `AgentPermissionReply` — the client's answer to an
/// [`Frame::AgentPermissionRequest`].
///
/// A reply for an id the daemon no longer tracks (the gate already timed out,
/// or the turn was cancelled) is ignored rather than reported: clicking a
/// prompt a moment after it expired is normal, not an error.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentPermissionReplyRequest {
    pub approval_id: u64,
    pub allow: bool,
    /// Shown to the model in place of "denied". Defaults to a plain refusal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One entry of the `AgentList` response.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentInfo {
    pub agent_id: u64,
    pub session_id: u64,
    pub epoch: u64,
    pub name: String,
    pub cwd: String,
    pub busy: bool,
    pub alive: bool,
    /// The last sequence number emitted so far.
    pub seq: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentListResponse {
    pub agents: Vec<AgentInfo>,
}

// ── typed envelopes ────────────────────────────────────────────────────────

/// A client→daemon message.
#[derive(Debug, Clone)]
pub enum Request {
    Create(CreateRequest),
    Attach(AttachRequest),
    Write { session_id: u64, bytes: Vec<u8> },
    Resize(ResizeRequest),
    Kill(KillRequest),
    Rename(RenameRequest),
    List,
    AgentCreate(Box<AgentCreateRequest>),
    AgentAttach(AgentAttachRequest),
    AgentPrompt(AgentInputRequest),
    AgentSteer(AgentInputRequest),
    AgentCancel(AgentIdRequest),
    AgentKill(AgentIdRequest),
    AgentPermissionReply(AgentPermissionReplyRequest),
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
    AgentCreated(AgentCreatedResponse),
    AgentAttached(Box<AgentAttachedResponse>),
    AgentList(AgentListResponse),
    /// One agent event, in sequence order. `agent_id` stays out of the payload
    /// because it is needed for routing before the body is decoded.
    AgentEvent {
        agent_id: u64,
        item: Sequenced<AgentEvent>,
    },
    /// An agent is blocked waiting for a human to allow or refuse a tool call.
    /// Sent to every attached client; the first reply wins.
    AgentPermissionRequest {
        agent_id: u64,
        /// Identifies this prompt in the reply. Minted by the daemon, so it is
        /// unique across agents and turns — unlike the model's own tool-call id.
        approval_id: u64,
        /// The model's tool-call id, so a client can tie the prompt to the
        /// `ToolCallRequested` event it already rendered.
        call_id: String,
        name: String,
        arguments: serde_json::Value,
        /// How long the gate will wait before denying.
        timeout_ms: u64,
    },
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
        Request::AgentCreate(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_CREATE_REQ, &p).await
        }
        Request::AgentAttach(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_ATTACH_REQ, &p).await
        }
        Request::AgentPrompt(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_PROMPT_REQ, &p).await
        }
        Request::AgentSteer(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_STEER_REQ, &p).await
        }
        Request::AgentCancel(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_CANCEL_REQ, &p).await
        }
        Request::AgentKill(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_KILL_REQ, &p).await
        }
        Request::AgentPermissionReply(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_PERMISSION_REPLY_REQ, &p).await
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
        TAG_AGENT_CREATE_REQ => Request::AgentCreate(Box::new(serde_json::from_slice(&p)?)),
        TAG_AGENT_ATTACH_REQ => Request::AgentAttach(serde_json::from_slice(&p)?),
        TAG_AGENT_PROMPT_REQ => Request::AgentPrompt(serde_json::from_slice(&p)?),
        TAG_AGENT_STEER_REQ => Request::AgentSteer(serde_json::from_slice(&p)?),
        TAG_AGENT_CANCEL_REQ => Request::AgentCancel(serde_json::from_slice(&p)?),
        TAG_AGENT_KILL_REQ => Request::AgentKill(serde_json::from_slice(&p)?),
        TAG_AGENT_PERMISSION_REPLY_REQ => {
            Request::AgentPermissionReply(serde_json::from_slice(&p)?)
        }
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
        Frame::AgentCreated(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_CREATED_RES, &p).await
        }
        Frame::AgentAttached(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_ATTACHED_RES, &p).await
        }
        Frame::AgentList(r) => {
            let p = serde_json::to_vec(r)?;
            write_frame_raw(w, TAG_AGENT_LIST_RES, &p).await
        }
        Frame::AgentEvent { agent_id, item } => {
            // [agent_id u64][event json]: routing stays cheap, and the event
            // body keeps the exact shape `agent-core` already round-trips.
            let body = serde_json::to_vec(item)?;
            let mut p = Vec::with_capacity(8 + body.len());
            p.extend_from_slice(&agent_id.to_be_bytes());
            p.extend_from_slice(&body);
            write_frame_raw(w, TAG_AGENT_EVENT, &p).await
        }
        Frame::AgentPermissionRequest {
            agent_id,
            approval_id,
            call_id,
            name,
            arguments,
            timeout_ms,
        } => {
            let body = serde_json::json!({
                "approvalId": approval_id,
                "callId": call_id,
                "name": name,
                "arguments": arguments,
                "timeoutMs": timeout_ms,
            });
            let body = serde_json::to_vec(&body)?;
            let mut p = Vec::with_capacity(8 + body.len());
            p.extend_from_slice(&agent_id.to_be_bytes());
            p.extend_from_slice(&body);
            write_frame_raw(w, TAG_AGENT_PERMISSION_REQUEST, &p).await
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
        TAG_AGENT_CREATED_RES => Frame::AgentCreated(serde_json::from_slice(&p)?),
        TAG_AGENT_ATTACHED_RES => Frame::AgentAttached(Box::new(serde_json::from_slice(&p)?)),
        TAG_AGENT_LIST_RES => Frame::AgentList(serde_json::from_slice(&p)?),
        TAG_AGENT_EVENT => {
            let (agent_id, rest) = take_u64(&p)?;
            Frame::AgentEvent {
                agent_id,
                item: serde_json::from_slice(rest)?,
            }
        }
        TAG_AGENT_PERMISSION_REQUEST => {
            let (agent_id, rest) = take_u64(&p)?;
            let body: serde_json::Value = serde_json::from_slice(rest)?;
            let field = |key: &str| body.get(key).cloned().unwrap_or(serde_json::Value::Null);
            Frame::AgentPermissionRequest {
                agent_id,
                approval_id: field("approvalId").as_u64().unwrap_or_default(),
                call_id: field("callId").as_str().unwrap_or_default().to_owned(),
                name: field("name").as_str().unwrap_or_default().to_owned(),
                arguments: field("arguments"),
                timeout_ms: field("timeoutMs").as_u64().unwrap_or_default(),
            }
        }
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
