//! Session identity and the cursor a client resumes from.
//!
//! A canvas card, a restarted daemon and a second window all need to agree on
//! "which session, and how far through its event stream am I". A bare sequence
//! number cannot answer that: sequence numbers restart when a process restarts,
//! so `seq = 37` from a previous run would silently match a brand new session
//! that has already emitted 40 events.
//!
//! The cursor is therefore `(session_id, epoch, seq)`:
//!
//! * `session_id` — which conversation.
//! * `epoch` — which *generation* of the journal. A daemon stamps a fresh epoch
//!   on every start, so a cursor from an earlier run is recognised as stale
//!   instead of being applied to a session that merely reused an id.
//! * `seq` — how far into that generation the client has applied.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::event::{AgentEvent, Sequenced};

/// Identifies one session within an epoch.
///
/// Minted from a process-global counter: sessions live in whichever process
/// hosts the runtime (today, the terminal daemon), so a counter is enough —
/// uniqueness across *runs* is the epoch's job, not the id's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(u64);

impl SessionId {
    /// Allocate the next id in this process.
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    /// Wrap a known id, for clients decoding a cursor.
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "s{}", self.0)
    }
}

impl serde::Serialize for SessionId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> serde::Deserialize<'de> for SessionId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(u64::deserialize(deserializer)?))
    }
}

/// The default epoch for sessions created in this process.
///
/// Derived from the wall clock at first use rather than random, so it needs no
/// dependency and still changes on every process start. A host that wants to
/// control it (a daemon keeping its epoch stable across a restart of its own
/// worker threads) passes one explicitly to
/// [`AgentSession::start_with_epoch`](crate::AgentSession::start_with_epoch).
pub fn process_epoch() -> u64 {
    static EPOCH: OnceLock<u64> = OnceLock::new();
    *EPOCH.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.as_nanos() as u64)
            .unwrap_or_default()
    })
}

/// A resumable position in one session's event stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResumeCursor {
    pub session_id: SessionId,
    pub epoch: u64,
    /// The last sequence number the client has already applied. `0` means
    /// "no events yet", so every event replays.
    pub seq: u64,
}

/// What a client should do with a stored cursor.
#[derive(Clone, Debug, PartialEq)]
pub enum Replay {
    /// The cursor is already at the head of this session's journal.
    UpToDate,
    /// Events the client has not applied yet, in order.
    Events(Vec<Sequenced<AgentEvent>>),
    /// The cursor belongs to a different session, or to a previous epoch. The
    /// client must discard its transcript and start over — the sequence numbers
    /// it holds refer to a stream that no longer exists.
    Stale,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_ids_are_unique_and_readable() {
        let first = SessionId::next();
        let second = SessionId::next();
        assert_ne!(first, second);
        assert!(second.raw() > first.raw());
        assert_eq!(first.to_string(), format!("s{}", first.raw()));
    }

    #[test]
    fn epoch_is_stable_within_a_process() {
        assert_eq!(process_epoch(), process_epoch());
    }

    #[test]
    fn session_id_survives_a_serde_round_trip() {
        let id = SessionId::next();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(serde_json::from_str::<SessionId>(&json).unwrap(), id);
    }
}
