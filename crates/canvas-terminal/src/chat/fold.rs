//! The card-side model of one chat session: a fold of the event stream
//! into rows.
//!
//! The daemon's per-session event ring is the authority and the socket
//! stream is only a live tail of it, so this model tolerates exactly the two
//! things a tail can do that a ring cannot: deliver an event twice, and skip
//! one. Both are detected by sequence number rather than trusted away.

use super::event::{ChatEvent, Sequenced};

/// How many applied events the card keeps, for rebuilding after a resync.
pub const LOG_CAPACITY: usize = 4_000;

/// One rendered block of the transcript.
#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    /// Something the user asked for.
    User { text: String },
    /// Assistant prose. `streaming` marks the row the deltas land in.
    Assistant { text: String, streaming: bool },
    /// Model reasoning, rendered dimmed.
    Reasoning { text: String },
    /// A tool call. `ok == None` means it has not finished yet.
    Tool {
        name: String,
        ok: Option<bool>,
        summary: String,
    },
    /// Everything that is not conversation: notices, gap warnings.
    Notice { text: String },
}

impl Row {
    /// Single-line form for tests.
    #[cfg(test)]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::User { .. } => "user",
            Self::Assistant { .. } => "assistant",
            Self::Reasoning { .. } => "reasoning",
            Self::Tool { .. } => "tool",
            Self::Notice { .. } => "notice",
        }
    }
}

/// Everything a chat card needs to draw, derived purely from the event
/// stream.
///
/// `busy` and `usage` are recomputed by folding, so a rebuild from the log
/// restores them. The session itself cannot die from events — the hosting
/// terminal can — so `dead` is set by the terminal layer, not here.
#[derive(Default)]
pub struct ChatCardState {
    /// Events applied so far, in sequence order.
    log: Vec<Sequenced>,
    /// Folded view of the log, what the card actually renders.
    rows: Vec<Row>,
    /// Highest sequence number applied.
    seq: u64,
    /// Set when a sequence jump was observed: `(last before, first after)`.
    gap: Option<(u64, u64)>,
    pub busy: bool,
    pub usage: Option<String>,
    /// The CLI's own session id, when it reported one (used to resume).
    pub session_id: Option<String>,
    /// Which CLI is hosting (for the switch button and input formatting).
    pub cli: Option<String>,
}

impl ChatCardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn gap(&self) -> Option<(u64, u64)> {
        self.gap
    }

    /// Highest sequence number applied; the resume point for `ChatSync`.
    pub fn last_seq(&self) -> u64 {
        self.seq
    }

    /// Apply one event, in whatever order it arrives.
    ///
    /// A duplicate (`seq` already applied) is dropped. A gap is recorded and
    /// surfaced as a notice row, because a transcript that silently lost a
    /// message is worse than one that visibly did.
    pub fn apply(&mut self, item: Sequenced) {
        if item.seq <= self.seq {
            return;
        }
        if item.seq > self.seq + 1 {
            let missed = item.seq - self.seq - 1;
            self.gap = Some((self.seq, item.seq));
            self.rows.push(Row::Notice {
                text: format!("⚠ {missed} event(s) missed — resyncing…"),
            });
        }
        self.seq = item.seq;
        self.log.push(item.clone());
        if self.log.len() > LOG_CAPACITY {
            self.log.remove(0);
        }
        self.fold(&item.event);
    }

    /// Fold one already-sequenced event into the rows.
    fn fold(&mut self, event: &ChatEvent) {
        match event {
            ChatEvent::SessionInfo { cli, session_id } => {
                self.cli = Some(cli.clone());
                if session_id.is_some() {
                    self.session_id = session_id.clone();
                }
            }
            ChatEvent::UserMessage { text } => {
                self.busy = true;
                self.rows.push(Row::User { text: text.clone() });
            }
            ChatEvent::AssistantDelta { text } => match self.rows.last_mut() {
                Some(Row::Assistant {
                    text: acc,
                    streaming: true,
                }) => acc.push_str(text),
                _ => self.rows.push(Row::Assistant {
                    text: text.clone(),
                    streaming: true,
                }),
            },
            ChatEvent::ReasoningDelta { text } => match self.rows.last_mut() {
                Some(Row::Reasoning { text: acc }) => acc.push_str(text),
                _ => self.rows.push(Row::Reasoning { text: text.clone() }),
            },
            ChatEvent::ToolUse {
                name,
                summary,
                done,
                ok,
            } => {
                if !*done {
                    self.rows.push(Row::Tool {
                        name: name.clone(),
                        ok: None,
                        summary: summary.clone(),
                    });
                } else if !self.mark_tool(name, *ok, summary) {
                    // No open row for this call — an attach replay racing the
                    // live stream. Show it anyway.
                    self.rows.push(Row::Tool {
                        name: name.clone(),
                        ok: Some(*ok),
                        summary: summary.clone(),
                    });
                }
            }
            ChatEvent::TurnDone { usage } => {
                self.busy = false;
                if let Some(usage) = usage {
                    self.usage = Some(usage.clone());
                }
                if let Some(Row::Assistant { streaming, .. }) = self.rows.last_mut() {
                    *streaming = false;
                }
            }
            ChatEvent::Exited => {
                self.busy = false;
            }
        }
    }

    /// Attach an outcome to the open tool row for `name`.
    fn mark_tool(&mut self, name: &str, ok: bool, summary: &str) -> bool {
        let Some(row) = self
            .rows
            .iter_mut()
            .rev()
            .find(|row| matches!(row, Row::Tool { name: n, ok: None, .. } if n == name))
        else {
            return false;
        };
        if let Row::Tool {
            ok: slot,
            summary: text,
            ..
        } = row
        {
            *slot = Some(ok);
            if !summary.is_empty() {
                *text = summary.to_owned();
            }
        }
        true
    }

    /// Rebuild from a replay (the daemon's whole ring for this session).
    ///
    /// Used after a gap: rather than guessing where the missing events
    /// belong, the card is rebuilt from the authority. Live events already
    /// applied are deduped by sequence, so nothing renders twice.
    pub fn reload(&mut self, replay: Vec<Sequenced>) {
        let mut merged = std::mem::take(&mut self.log);
        for item in replay {
            if !merged.iter().any(|m| m.seq == item.seq) {
                merged.push(item);
            }
        }
        merged.sort_by_key(|item| item.seq);
        self.rows.clear();
        self.seq = 0;
        self.busy = false;
        self.usage = None;
        self.session_id = None;
        self.cli = None;
        for item in &merged {
            self.seq = item.seq;
            self.fold(&item.event);
        }
        self.log = merged;
        // A fold from scratch cannot know about a gap; the caller re-arms
        // the resync by comparing its own seq against the ring's last.
        self.gap = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(n: u64, event: ChatEvent) -> Sequenced {
        Sequenced { seq: n, event }
    }

    fn row_kinds(state: &ChatCardState) -> Vec<&'static str> {
        state.rows().iter().map(|r| r.kind()).collect()
    }

    #[test]
    fn folds_a_pi_shaped_turn() {
        let mut state = ChatCardState::new();
        state.apply(seq(
            1,
            ChatEvent::SessionInfo {
                cli: "pi".into(),
                session_id: Some("s1".into()),
            },
        ));
        state.apply(seq(
            2,
            ChatEvent::UserMessage {
                text: "run: echo hi".into(),
            },
        ));
        state.apply(seq(
            3,
            ChatEvent::ReasoningDelta {
                text: "Simple ".into(),
            },
        ));
        state.apply(seq(
            4,
            ChatEvent::ReasoningDelta {
                text: "bash request.".into(),
            },
        ));
        state.apply(seq(
            5,
            ChatEvent::ToolUse {
                name: "bash".into(),
                summary: r#"{"command":"echo hi"}"#.into(),
                done: false,
                ok: true,
            },
        ));
        state.apply(seq(
            6,
            ChatEvent::ToolUse {
                name: "bash".into(),
                summary: "hi\n".into(),
                done: true,
                ok: true,
            },
        ));
        state.apply(seq(
            7,
            ChatEvent::AssistantDelta {
                text: "`hi` — ".into(),
            },
        ));
        state.apply(seq(
            8,
            ChatEvent::AssistantDelta {
                text: "ran successfully.".into(),
            },
        ));
        state.apply(seq(
            9,
            ChatEvent::TurnDone {
                usage: Some("34873 tok".into()),
            },
        ));

        assert!(state.busy == false);
        assert_eq!(state.session_id.as_deref(), Some("s1"));
        assert_eq!(state.cli.as_deref(), Some("pi"));
        assert_eq!(state.usage.as_deref(), Some("34873 tok"));
        let kinds = row_kinds(&state);
        assert_eq!(kinds, vec!["user", "reasoning", "tool", "assistant"]);
        match &state.rows()[2] {
            Row::Tool { name, ok, summary } => {
                assert_eq!(name, "bash");
                assert_eq!(*ok, Some(true));
                assert_eq!(summary, "hi\n");
            }
            other => panic!("expected tool row, got {other:?}"),
        }
        match &state.rows()[3] {
            Row::Assistant { text, streaming } => {
                assert_eq!(text, "`hi` — ran successfully.");
                assert!(!*streaming);
            }
            other => panic!("expected assistant row, got {other:?}"),
        }
    }

    #[test]
    fn duplicates_are_dropped_and_gaps_surface() {
        let mut state = ChatCardState::new();
        state.apply(seq(1, ChatEvent::UserMessage { text: "a".into() }));
        state.apply(seq(1, ChatEvent::UserMessage { text: "a".into() }));
        state.apply(seq(3, ChatEvent::TurnDone { usage: None }));
        let kinds = row_kinds(&state);
        assert_eq!(kinds, vec!["user", "notice"]);
        assert_eq!(state.gap(), Some((1, 3)));
    }

    #[test]
    fn reload_dedupes_and_restores_status() {
        let mut state = ChatCardState::new();
        state.apply(seq(
            1,
            ChatEvent::SessionInfo {
                cli: "pi".into(),
                session_id: Some("s1".into()),
            },
        ));
        state.apply(seq(2, ChatEvent::UserMessage { text: "hi".into() }));

        // A replay containing the same events plus more.
        state.reload(vec![
            seq(
                1,
                ChatEvent::SessionInfo {
                    cli: "pi".into(),
                    session_id: Some("s1".into()),
                },
            ),
            seq(2, ChatEvent::UserMessage { text: "hi".into() }),
            seq(
                3,
                ChatEvent::AssistantDelta {
                    text: "hello".into(),
                },
            ),
            seq(4, ChatEvent::TurnDone { usage: None }),
        ]);
        let kinds = row_kinds(&state);
        assert_eq!(kinds, vec!["user", "assistant"]);
        assert!(!state.busy);
    }
}
