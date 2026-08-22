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

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Socket/pipe label used to derive the per-user `rmux-ipc` endpoint.
pub const LABEL: &str = "canvas-terminal";

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

const TAG_CREATE_RES: u8 = 0x81;
const TAG_ATTACH_RES: u8 = 0x82;
const TAG_LIST_RES: u8 = 0x86;
const TAG_OUTPUT: u8 = 0x10;
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
