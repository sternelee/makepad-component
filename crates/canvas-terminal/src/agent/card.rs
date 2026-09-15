//! The card-side model of one agent: a fold of the event stream into rows.
//!
//! The daemon's journal is the authority and the socket stream is only a live
//! tail of it, so this model is built to tolerate exactly the two things a tail
//! can do that a journal cannot: deliver an event twice, and skip one. Both are
//! detected by sequence number rather than trusted away.

use agent_core::{AgentEvent, Sequenced};

/// How many applied events the card keeps, for rebuilding after a resync.
///
/// Matches the daemon-side journal's purpose: enough to refold a transcript
/// after a gap without keeping every event of a very long session forever.
pub const LOG_CAPACITY: usize = 4_000;

/// One rendered block of the transcript.
#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    /// Something the user asked for, either a prompt or an accepted steer.
    User { text: String },
    /// Assistant prose. `streaming` marks the row the deltas are landing in.
    Assistant { text: String, streaming: bool },
    /// Model reasoning, rendered dimmed.
    Reasoning { text: String },
    /// A tool call. `ok == None` means it has not finished yet.
    Tool {
        id: String,
        name: String,
        ok: Option<bool>,
        summary: Option<String>,
    },
    /// Everything that is not conversation: errors, notices, gap warnings.
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

/// A tool call waiting for a human decision.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingApproval {
    pub approval_id: u64,
    /// The model's tool-call id, so the prompt can be tied to the tool row.
    pub call_id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub timeout_ms: u64,
}

/// Everything a card needs to draw, derived purely from the event stream.
///
/// `busy`, `dead`, `usage` and `approval` are all recomputed by folding, so a
/// rebuild from the log restores them — with one exception: an approval is
/// carried by its own frame, not by the journal, so a resync forgets a pending
/// prompt. The daemon's gate still times out in that case, which denies; the
/// card simply stops showing buttons that can no longer have an effect.
#[derive(Default)]
pub struct AgentCardState {
    /// Events applied so far, in sequence order.
    log: Vec<Sequenced<AgentEvent>>,
    /// Folded view of the log, what the card actually renders.
    rows: Vec<Row>,
    /// Highest sequence number applied.
    seq: u64,
    /// Set when a sequence jump was observed: `(last before the gap, first after)`.
    gap: Option<(u64, u64)>,
    pub busy: bool,
    pub dead: bool,
    pub usage: Option<(u64, u64)>,
    pub approval: Option<PendingApproval>,
}

impl AgentCardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    #[cfg(test)]
    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn gap(&self) -> Option<(u64, u64)> {
        self.gap
    }

    /// Apply one event, in whatever order it arrives.
    ///
    /// A duplicate (`seq` already applied) is dropped. A gap is recorded and
    /// surfaced as a notice row, because a transcript that silently lost a
    /// message is worse than one that visibly did.
    pub fn apply(&mut self, item: Sequenced<AgentEvent>) {
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
        self.fold(&item.value);
    }

    /// Fold one already-sequenced event into the rows.
    fn fold(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::PromptSubmitted { text } => {
                self.rows.push(Row::User { text: text.clone() });
            }
            AgentEvent::TurnStarted => self.busy = true,
            AgentEvent::TurnFinished { .. } => {
                self.busy = false;
                // Whatever was streaming is done now.
                if let Some(Row::Assistant { streaming, .. }) = self.rows.last_mut() {
                    *streaming = false;
                }
            }
            AgentEvent::TextDelta { text } => match self.rows.last_mut() {
                Some(Row::Assistant {
                    text: acc,
                    streaming: true,
                }) => acc.push_str(text),
                _ => self.rows.push(Row::Assistant {
                    text: text.clone(),
                    streaming: true,
                }),
            },
            AgentEvent::ReasoningDelta { text } => match self.rows.last_mut() {
                Some(Row::Reasoning { text: acc }) => acc.push_str(text),
                _ => self.rows.push(Row::Reasoning { text: text.clone() }),
            },
            AgentEvent::ToolCallRequested { id, name, .. } => {
                self.rows.push(Row::Tool {
                    id: id.clone(),
                    name: name.clone(),
                    ok: None,
                    summary: None,
                });
            }
            AgentEvent::ToolCallFinished {
                id,
                name,
                ok,
                summary,
                ..
            } => {
                if !self.mark_tool(id, Some(*ok), summary) {
                    // No open row for this id — an attach replay racing the
                    // live stream, or an ordering we did not create. Show it
                    // anyway; a lost tool result is exactly what the log is for.
                    self.rows.push(Row::Tool {
                        id: id.clone(),
                        name: name.clone(),
                        ok: Some(*ok),
                        summary: Some(summary.clone()),
                    });
                }
            }
            AgentEvent::ToolCallDenied { id, name, reason } => {
                if !self.mark_tool(id, Some(false), reason) {
                    self.rows.push(Row::Tool {
                        id: id.clone(),
                        name: name.clone(),
                        ok: Some(false),
                        summary: Some(reason.clone()),
                    });
                }
            }
            AgentEvent::SteerAccepted { text } => {
                // A steer lands mid-turn, so it reads as another user message.
                self.rows.push(Row::User { text: text.clone() });
            }
            AgentEvent::SteerRejected { text, reason } => {
                self.rows.push(Row::Notice {
                    text: format!("steer not delivered ({reason}): {text}"),
                });
            }
            AgentEvent::Usage {
                prompt_tokens,
                completion_tokens,
            } => self.usage = Some((*prompt_tokens, *completion_tokens)),
            AgentEvent::Error { message } => {
                self.rows.push(Row::Notice {
                    text: format!("error: {message}"),
                });
            }
            AgentEvent::Exited => {
                self.dead = true;
                self.busy = false;
                self.approval = None;
            }
        }
    }

    /// Attach an outcome to the open tool row for `id`.
    fn mark_tool(&mut self, id: &str, ok: Option<bool>, summary: &str) -> bool {
        let Some(row) = self
            .rows
            .iter_mut()
            .rev()
            .find(|row| matches!(row, Row::Tool { id: row_id, ok: None, .. } if row_id == id))
        else {
            return false;
        };
        if let Row::Tool {
            ok: slot,
            summary: text,
            ..
        } = row
        {
            *slot = ok;
            *text = Some(summary.to_owned());
        }
        true
    }

    /// Merge a replay (from `after_seq = 0`, i.e. the daemon's whole journal
    /// for this session) into the applied log and refold everything.
    ///
    /// Used after a gap was detected: rather than guessing where the missing
    /// events belong, the card is rebuilt from the authority. Live events
    /// already applied are deduped by sequence, so nothing renders twice.
    pub fn reload(&mut self, replay: Vec<Sequenced<AgentEvent>>) {
        // Merge the two seq-ordered lists, dropping duplicates.
        let mut merged: Vec<Sequenced<AgentEvent>> =
            Vec::with_capacity(self.log.len() + replay.len());
        let (mut mine, mut theirs) = (0usize, 0usize);
        while mine < self.log.len() || theirs < replay.len() {
            match (self.log.get(mine), replay.get(theirs)) {
                (Some(a), Some(b)) => {
                    if a.seq < b.seq {
                        merged.push(a.clone());
                        mine += 1;
                    } else if b.seq < a.seq {
                        merged.push(b.clone());
                        theirs += 1;
                    } else {
                        merged.push(a.clone());
                        mine += 1;
                        theirs += 1;
                    }
                }
                (Some(a), None) => {
                    merged.push(a.clone());
                    mine += 1;
                }
                (None, Some(b)) => {
                    merged.push(b.clone());
                    theirs += 1;
                }
                (None, None) => break,
            }
        }

        self.log = merged;
        self.rows.clear();
        self.seq = 0;
        self.gap = None;
        self.busy = false;
        self.dead = false;
        self.usage = None;
        self.approval = None;
        let log = std::mem::take(&mut self.log);
        for item in log {
            self.apply(item);
        }
    }

    /// An approval prompt arrived (or was replaced by a newer one).
    pub fn set_approval(&mut self, approval: PendingApproval) {
        self.approval = Some(approval);
    }

    /// The prompt was answered or expired; stop showing buttons for it.
    pub fn clear_approval(&mut self, approval_id: u64) {
        if matches!(&self.approval, Some(a) if a.approval_id == approval_id) {
            self.approval = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn at(seq: u64, event: AgentEvent) -> Sequenced<AgentEvent> {
        Sequenced { seq, value: event }
    }

    fn text(seq: u64, chunk: &str) -> Sequenced<AgentEvent> {
        at(seq, AgentEvent::TextDelta { text: chunk.into() })
    }

    #[test]
    fn deltas_accumulate_into_one_streaming_row() {
        let mut state = AgentCardState::new();
        state.apply(at(1, AgentEvent::TurnStarted));
        state.apply(text(2, "Hel"));
        state.apply(text(3, "lo"));
        state.apply(at(
            4,
            AgentEvent::TurnFinished {
                stop_reason: Some("stop".into()),
            },
        ));

        assert!(!state.busy);
        assert_eq!(state.rows().len(), 1);
        match &state.rows()[0] {
            Row::Assistant { text, streaming } => {
                assert_eq!(text, "Hello");
                assert!(!streaming, "TurnFinished closes the streaming row");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_tool_row_opens_then_takes_its_outcome() {
        let mut state = AgentCardState::new();
        state.apply(at(
            1,
            AgentEvent::ToolCallRequested {
                id: "c1".into(),
                name: "read_file".into(),
                arguments: json!({"path":"a"}),
            },
        ));
        state.apply(at(
            2,
            AgentEvent::ToolCallFinished {
                id: "c1".into(),
                name: "read_file".into(),
                ok: true,
                summary: "1\tHello".into(),
            },
        ));
        assert_eq!(state.rows().len(), 1);
        match &state.rows()[0] {
            Row::Tool {
                name, ok, summary, ..
            } => {
                assert_eq!(name, "read_file");
                assert_eq!(*ok, Some(true));
                assert_eq!(summary.as_deref(), Some("1\tHello"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_denied_call_marks_the_row_rather_than_adding_one() {
        let mut state = AgentCardState::new();
        state.apply(at(
            1,
            AgentEvent::ToolCallRequested {
                id: "c1".into(),
                name: "write_file".into(),
                arguments: json!({}),
            },
        ));
        state.apply(at(
            2,
            AgentEvent::ToolCallDenied {
                id: "c1".into(),
                name: "write_file".into(),
                reason: "not this file".into(),
            },
        ));
        assert_eq!(state.rows().len(), 1);
        match &state.rows()[0] {
            Row::Tool { ok, summary, .. } => {
                assert_eq!(*ok, Some(false));
                assert_eq!(summary.as_deref(), Some("not this file"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn an_outcome_without_a_row_is_not_dropped() {
        let mut state = AgentCardState::new();
        state.apply(at(
            1,
            AgentEvent::ToolCallFinished {
                id: "orphan".into(),
                name: "read_file".into(),
                ok: true,
                summary: "done".into(),
            },
        ));
        match &state.rows()[0] {
            Row::Tool { ok, .. } => assert_eq!(*ok, Some(true)),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn steer_is_rendered_as_a_user_message() {
        let mut state = AgentCardState::new();
        state.apply(at(1, AgentEvent::TurnStarted));
        state.apply(at(
            2,
            AgentEvent::SteerAccepted {
                text: "also this".into(),
            },
        ));
        match &state.rows()[0] {
            Row::User { text } => assert_eq!(text, "also this"),
            other => panic!("{other:?}"),
        }
        // A new delta after a steer starts a fresh assistant row.
        state.apply(text(3, "ok"));
        match &state.rows()[1] {
            Row::Assistant { streaming, .. } => assert!(streaming),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn duplicates_are_ignored() {
        let mut state = AgentCardState::new();
        state.apply(text(1, "a"));
        state.apply(text(1, "a")); // replayed by an attach racing the live stream
        state.apply(text(2, "b"));
        assert_eq!(state.rows().len(), 1);
        assert_eq!(state.seq(), 2);
        match &state.rows()[0] {
            Row::Assistant { text, .. } => assert_eq!(text, "ab"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_gap_is_visible_and_recorded() {
        let mut state = AgentCardState::new();
        state.apply(text(1, "a"));
        state.apply(text(5, "e"));
        assert_eq!(state.gap(), Some((1, 5)));
        // The notice sits between the two deltas, so the second one cannot
        // join the first assistant row — it starts its own.
        let kinds: Vec<&str> = state.rows().iter().map(Row::kind).collect();
        assert_eq!(kinds, vec!["assistant", "notice", "assistant"]);
        match &state.rows()[1] {
            Row::Notice { text } => assert!(text.contains("3 event")),
            other => panic!("{other:?}"),
        }
        match &state.rows()[2] {
            Row::Assistant { text, .. } => assert_eq!(text, "e"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn reload_merges_the_journal_back_in_and_refolds() {
        let mut state = AgentCardState::new();
        state.apply(text(1, "a"));
        state.apply(text(5, "e")); // gap: 2..4 lost in transit

        // The daemon's journal has everything; reload supplies the middle.
        state.reload(vec![
            text(1, "a"),
            text(2, "b"),
            text(3, "c"),
            text(4, "d"),
            text(5, "e"),
        ]);
        assert_eq!(state.gap(), None);
        assert_eq!(state.seq(), 5);
        match &state.rows()[0] {
            Row::Assistant { text, .. } => assert_eq!(text, "abcde"),
            other => panic!("{other:?}"),
        }
        // The gap notice is gone too — the transcript is whole again.
        assert_eq!(state.rows().len(), 1);
    }

    #[test]
    fn reload_keeps_events_that_arrived_after_the_replay() {
        let mut state = AgentCardState::new();
        state.apply(text(1, "a"));
        state.apply(text(2, "b"));
        // A replay snapshot that stops at 1 must not discard 2.
        state.reload(vec![text(1, "a")]);
        assert_eq!(state.seq(), 2);
        match &state.rows()[0] {
            Row::Assistant { text, .. } => assert_eq!(text, "ab"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn reload_restores_the_turn_state_from_the_log() {
        let mut state = AgentCardState::new();
        state.apply(at(1, AgentEvent::TurnStarted));
        state.apply(text(2, "half"));
        assert!(state.busy);
        // Reload from a journal that has since seen the turn finish.
        state.reload(vec![
            at(1, AgentEvent::TurnStarted),
            text(2, "half"),
            at(
                3,
                AgentEvent::TurnFinished {
                    stop_reason: Some("stop".into()),
                },
            ),
        ]);
        assert!(!state.busy);
        assert_eq!(state.rows().len(), 1);
        match &state.rows()[0] {
            Row::Assistant { streaming, .. } => assert!(!streaming),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn approvals_are_replaced_and_cleared_by_id() {
        let mut state = AgentCardState::new();
        state.set_approval(PendingApproval {
            approval_id: 7,
            call_id: "c1".into(),
            name: "write_file".into(),
            arguments: json!({}),
            timeout_ms: 1000,
        });
        assert!(state.approval.is_some());
        // A stale clear must not remove a newer prompt.
        state.clear_approval(999);
        assert!(state.approval.is_some());
        state.clear_approval(7);
        assert!(state.approval.is_none());
    }

    #[test]
    fn exit_marks_the_card_dead_and_drops_its_prompt() {
        let mut state = AgentCardState::new();
        state.set_approval(PendingApproval {
            approval_id: 1,
            call_id: "c".into(),
            name: "read_file".into(),
            arguments: json!({}),
            timeout_ms: 1000,
        });
        state.apply(at(1, AgentEvent::Exited));
        assert!(state.dead);
        assert!(state.approval.is_none());
    }

    #[test]
    fn reasoning_folds_into_its_own_dim_row() {
        let mut state = AgentCardState::new();
        state.apply(at(
            1,
            AgentEvent::ReasoningDelta {
                text: "think".into(),
            },
        ));
        state.apply(at(2, AgentEvent::ReasoningDelta { text: "ing".into() }));
        match &state.rows()[0] {
            Row::Reasoning { text } => assert_eq!(text, "thinking"),
            other => panic!("{other:?}"),
        }
        // ...and prose after it starts a new row rather than joining it.
        state.apply(text(3, "answer"));
        match &state.rows()[1] {
            Row::Assistant { text, .. } => assert_eq!(text, "answer"),
            other => panic!("{other:?}"),
        }
    }
}
