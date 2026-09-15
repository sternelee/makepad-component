//! Cooperative cancellation.
//!
//! Cancellation has to reach *inside* a blocking provider call. Checking a flag
//! only between calls leaves the "stop" button unresponsive for as long as the
//! model keeps streaming — unbounded in practice. A [`CancelToken`] is passed
//! into every provider call so the driver can check it between stream chunks,
//! which bounds the latency by the arrival time of one chunk instead of by the
//! whole turn.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A shared "stop what you are doing" flag.
///
/// Cloning shares the same flag. A session installs a fresh token at the start
/// of every turn, so cancelling one turn cannot bleed into the next.
#[derive(Clone, Debug, Default)]
pub struct CancelToken {
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// A token that is already cancelled, for tests and for callers that must
    /// not start work at all.
    pub fn cancelled() -> Self {
        let token = Self::new();
        token.cancel();
        token
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_one_flag() {
        let token = CancelToken::new();
        let observer = token.clone();
        assert!(!observer.is_cancelled());
        token.cancel();
        assert!(observer.is_cancelled());
    }

    #[test]
    fn a_fresh_token_starts_uncancelled() {
        let cancelled = CancelToken::cancelled();
        assert!(cancelled.is_cancelled());
        let fresh = CancelToken::new();
        assert!(!fresh.is_cancelled());
    }
}
