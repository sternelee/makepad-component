//! `MpSearch` — the match behind a command palette.
//!
//! ## Everything interesting here is one pure function
//!
//! A palette's row list is whatever [`rank`] returns, so the panel, the input and
//! the list are the part that already exists and the *ranking* is the part worth
//! writing down. It is tested exhaustively for the reason `mp/pagination.rs` is:
//! a match is a set property, so it is checkable with no window at all, and a
//! fuzzy matcher is exactly the kind of thing that is subtly wrong in a way only a
//! reader notices.
//!
//! ## The rules
//!
//! A match is a **case-insensitive subsequence** — `gtf` finds `Go to file` — which
//! is what makes a palette usable from three keystrokes. Among matches the ranking
//! prefers, in order:
//!
//! 1. **A match at a word start**, because `nt` finding `New Terminal` is what the
//!    reader meant, where `nt` finding `environment` is an accident.
//! 2. **Contiguous runs**, because `term` finding `Terminal` should beat `term`
//!    finding `The remote endpoint`.
//! 3. **An earlier first hit**, because a candidate that matches sooner is closer
//!    to what was typed.
//!
//! The weights are a *rank* rather than a number anyone measures: no caller has a
//! budget for a score, only for an order. What a caller can see is the top row, and
//! the tests state which candidate that must be.

use makepad_widgets::*;

/// What a match scored, kept private because a score is only meaningful against
/// another score from the same function.
/// What each preference is worth.
///
/// **Chosen, not measured** — no caller has a budget for a score, only for an
/// order, and all a reader can see is the top row. The tests below state which
/// candidate that must be for the cases that matter.
///
/// ## Why these are weights and not a comparison order
///
/// Two of the preferences point in *opposite* directions, so no single
/// lexicographic order gets both right:
///
/// - `nt` must find `New Terminal` rather than `environment`, so **word starts
///   have to be able to dominate**.
/// - `term` must find `Terminal` rather than `The remote endpoint` — which also
///   matches two word starts (`T` of `The`, `r` of `remote`) but scatters — so a
///   **contiguous run has to be able to dominate word starts**.
///
/// The first version of this file tried a derived `Ord` on three keys and was
/// wrong in one direction or the other depending on which key came first: fields
/// declared `first_hit, run, boundaries` passed the `term` case and failed the
/// `nt` case's *intent*, and the documented order `boundaries, run, first_hit`
/// passed `nt` and failed `term`. Both orders passed a majority of the tests,
/// which is why the disagreement survived a first reading.
///
/// A third case fixes the ratio: `or` should find `Off Road` (two word starts, no
/// contiguity, 16) rather than `Word` (a contiguous `or`, no word starts, 3 — or
/// 4 under a run-first order, which would rank `Word` first and be wrong).
const WORD_START: i32 = 8;
/// A matched character that continues a run. Three contiguous characters
/// therefore beat one word start and lose to two.
const CONTIGUOUS: i32 = 4;

/// A match's score, and where its first character landed.
///
/// The position is carried separately rather than folded in, so that the
/// tie-break is visible where it is applied instead of hidden in the weighting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Score {
    value: i32,
    first_hit: usize,
}

/// Whether `query` is a case-insensitive subsequence of `candidate`.
///
/// The cheap question, for a caller that only wants to filter.
pub fn matches(candidate: &str, query: &str) -> bool {
    score(candidate, query).is_some()
}

/// The indices of `candidates` that match `query`, best first.
///
/// An empty query matches everything in the order given, which is what a palette
/// shows before anything is typed. Ties keep their original order — `sort_by` is
/// stable — so a list of equal matches does not shuffle between keystrokes.
pub fn rank(candidates: &[String], query: &str) -> Vec<usize> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return (0..candidates.len()).collect();
    }
    let mut scored: Vec<(usize, Score)> = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| score(candidate, trimmed).map(|s| (index, s)))
        .collect();
    // Best score first; an earlier first hit breaks a tie; and `sort_by` is
    // stable, so a tie on both keeps the list's own order and a row does not move
    // under the reader's finger between keystrokes.
    scored.sort_by(|a, b| {
        b.1.value
            .cmp(&a.1.value)
            .then(a.1.first_hit.cmp(&b.1.first_hit))
    });
    scored.into_iter().map(|(index, _)| index).collect()
}

/// The score of the best subsequence match, or `None`.
///
/// A greedy left-to-right walk rather than a search over every subsequence: the
/// best match for a palette is found by taking each query character at the
/// earliest position that still allows the rest, and a matcher that explored all
/// arrangements would be slower and no better *for this use*. What it costs is
/// that a candidate can be ranked below one a smarter matcher would prefer; what
/// it guarantees is that a match is always found when one exists, which is the
/// property a reader notices.
fn score(candidate: &str, query: &str) -> Option<Score> {
    let candidate_chars: Vec<char> = candidate.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();
    if query_chars.is_empty() {
        return Some(Score {
            value: 0,
            first_hit: 0,
        });
    }

    let mut qi = 0;
    let mut first_hit = None;
    let mut value = 0i32;
    let mut previous_matched = false;

    for (ci, ch) in candidate_chars.iter().enumerate() {
        if qi >= query_chars.len() {
            break;
        }
        if !ch.eq_ignore_ascii_case(&query_chars[qi]) {
            previous_matched = false;
            continue;
        }
        if first_hit.is_none() {
            first_hit = Some(ci);
        }
        if is_word_start(&candidate_chars, ci) {
            value += WORD_START;
        }
        if previous_matched {
            value += CONTIGUOUS;
        }
        previous_matched = true;
        qi += 1;
    }

    // Every query character must have been consumed; a partial walk is not a
    // match, no matter how well the characters it did consume scored.
    if qi < query_chars.len() {
        return None;
    }
    // The position is deliberately **not** folded into `value`. An earlier version
    // subtracted it ("a mild preference for matching sooner"), which double-counted
    // the tie-break: `New Terminal` scores 20 for `term` — one word start and a
    // three-character run — and `The remote endpoint` scores 16 for two scattered
    // word starts, so the first is strictly better; subtracting position 4 from
    // the first and 0 from the second tied them at 16 and handed the row to the
    // *worse* match. A tie-break is only a tie-break if it runs after the score.
    Some(Score {
        value,
        first_hit: first_hit.unwrap_or(0),
    })
}

/// Whether the character at `index` begins a word.
///
/// The first character always does, and any character after a space, a hyphen, an
/// underscore or a path separator does. Those four are the separators the labels
/// in a real palette use — `New Terminal`, `agent-workbench`, `mp/list.rs` — and a
/// boundary rule that only knew spaces would rank `New Terminal` below
/// `environment` for the query `nt`.
fn is_word_start(chars: &[char], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    // `index > 0` because the caller only ever passes a real index.
    matches!(
        chars[index - 1],
        ' ' | '-' | '_' | '/' | '.' | ':' | '·' | '—'
    )
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// A search field: a leading magnifier, the input, and a trailing cap.
    ///
    /// A DSL prototype rather than a widget — the input, the glyph and the
    /// shortcut are all things this crate already has, and the only decision is
    /// that the glyph *leads* (so the eye finds the field by its icon) while the
    /// shortcut *trails* (so it is read after the query, not before it).
    mod.mp.MpSearch = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8
        align: Align{x: 0.0, y: 0.5}

        search_glyph := mod.mp.MpIcon{
            width: 14
            height: 14
            control: mod.mpc.ControlSize.Small
            glyph: "\u{f002}"
        }
        search_input := mod.mp.MpTextInputSearch{
            width: Fill
        }
        search_hint := mod.mp.MpKbd{
            text: "⌘K"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// The top-ranked candidate, which is the only part of a score anyone sees.
    fn top(candidates: &[&str], query: &str) -> Option<String> {
        let list = owned(candidates);
        rank(&list, query).first().map(|i| list[*i].clone())
    }

    #[test]
    fn test_an_empty_query_matches_everything_in_order() {
        // What a palette shows before anything is typed.
        let list = owned(&["one", "two", "three"]);
        assert_eq!(rank(&list, ""), vec![0, 1, 2]);
        assert_eq!(rank(&list, "   "), vec![0, 1, 2]);
    }

    #[test]
    fn test_a_substring_matches() {
        assert!(matches("Go to file", "to file"));
        assert!(matches("Go to file", "Go"));
    }

    #[test]
    fn test_a_subsequence_matches() {
        // Three keystrokes find `Go to file`, which is what makes a palette usable.
        assert!(matches("Go to file", "gtf"));
        assert!(matches("agent-workbench", "awb"));
        assert!(matches("mp/list.rs", "mlr"));
    }

    #[test]
    fn test_a_non_subsequence_does_not_match() {
        assert!(!matches("Go to file", "ftg"));
        assert!(!matches("Terminal", "xyz"));
        // Every character present but out of order is not a match.
        assert!(!matches("abc", "cb"));
    }

    #[test]
    fn test_matching_is_case_insensitive_both_ways() {
        assert!(matches("Go to file", "GTF"));
        assert!(matches("go to file", "GTF"));
        assert!(matches("GOTO", "goto"));
    }

    #[test]
    fn test_a_word_start_match_outranks_a_mid_word_one() {
        // `nt` finding `New Terminal` is what the reader meant; `nt` finding
        // `environment` is an accident.
        assert_eq!(
            top(&["environment", "New Terminal"], "nt").as_deref(),
            Some("New Terminal")
        );
    }

    #[test]
    fn test_hyphens_underscores_and_paths_are_word_starts() {
        // The separators a real palette's labels use. A boundary rule that only
        // knew spaces would rank these below a mid-word accident.
        assert!(is_word_start(&['a', '-', 'b'], 2));
        assert!(is_word_start(&['a', '_', 'b'], 2));
        assert!(is_word_start(&['a', '/', 'b'], 2));
        assert!(is_word_start(&['a', '.', 'b'], 2));
        assert!(!is_word_start(&['a', 'b', 'c'], 2));
    }

    #[test]
    fn test_a_contiguous_run_outranks_a_scattered_one() {
        // `term` should find `Terminal` before `The remote endpoint` — which also
        // matches two word starts (`T` of `The`, `r` of `remote`) but scatters a
        // character over each word. This is the case that rules out a
        // word-starts-first comparison order.
        assert_eq!(
            top(&["The remote endpoint", "Terminal"], "term").as_deref(),
            Some("Terminal")
        );
    }

    #[test]
    fn test_a_long_run_outranks_two_scattered_word_starts() {
        // The case the position penalty used to break: `New Terminal` scores 20 for
        // `term` (one word start plus a three-character run) against 16 for the two
        // scattered word starts in `The remote endpoint`, so it must lead — and it
        // must lead *despite* matching four characters later, which is why the
        // position cannot be folded into the score.
        assert_eq!(
            top(&["The remote endpoint", "New Terminal"], "term").as_deref(),
            Some("New Terminal")
        );
    }

    #[test]
    fn test_two_word_starts_outrank_one_contiguous_run() {
        // The case that rules out a run-first order, and with it the ratio between
        // the two weights: every duplicate of the same weight would let `Word`
        // win on its contiguous `or`, and `Off Road` is the abbreviation the
        // reader typed.
        assert_eq!(top(&["Word", "Off Road"], "or").as_deref(), Some("Off Road"));
    }

    #[test]
    fn test_an_earlier_first_hit_outranks_a_later_one() {
        // Same boundaries, same run, so the tie falls to where the match starts.
        assert_eq!(
            top(&["xx ab", "ab xx"], "ab").as_deref(),
            Some("ab xx")
        );
    }

    #[test]
    fn test_ties_keep_their_original_order() {
        // A list of equal matches must not shuffle between keystrokes, or the row
        // under the reader's finger moves.
        let list = owned(&["ab one", "ab two", "ab three"]);
        assert_eq!(rank(&list, "ab"), vec![0, 1, 2]);
        // ...and again, to make the claim about stability rather than about one run.
        assert_eq!(rank(&list, "ab"), vec![0, 1, 2]);
    }

    #[test]
    fn test_non_matches_are_dropped_rather_than_ranked_last() {
        // A palette's list is the matches; a non-match is not a low row.
        let list = owned(&["Terminal", "Browser", "Note"]);
        assert_eq!(rank(&list, "term"), vec![0]);
        assert_eq!(rank(&list, "zzz"), Vec::<usize>::new());
    }

    #[test]
    fn test_a_shorter_candidate_wins_a_fuller_match() {
        // Both match `term` at a word start with a contiguous run, so the score is
        // equal and the order is the list's. Recorded because it is the case a
        // reader is most likely to find surprising: a palette does not prefer
        // shorter labels, and pretending it does would need a weight nobody has a
        // budget for.
        let list = owned(&["Term", "Terminal"]);
        assert_eq!(rank(&list, "term"), vec![0, 1]);
    }

    #[test]
    fn test_ranking_survives_a_real_palette() {
        // The shape the page shows: a mix of command labels and file paths.
        let list = owned(&[
            "New Terminal",
            "Split Right",
            "Rename…",
            "Move to Space…",
            "Delete",
            "Toggle Terminal",
            "Go to File",
            "Command Palette",
            "agent-workbench/terminal.rs",
        ]);
        // `tt` on this list, and why each row is in it — which is the part worth
        // reading, because a fuzzy matcher's output is not obvious enough for a
        // bare assertion to teach anything:
        //
        //   5 `Toggle Terminal`  — `T` at 0 and `T` at 7, both word starts → 16
        //   8 `agent-workbench/terminal.rs` — `t` in `agent`, `t` after the `/` → 8
        //   7 `Command Palette`  — the two contiguous `t`s at the end → 4
        //   1 `Split Right`      — `t` in `Split`, `t` in `Right`, neither at a
        //                          word start and neither contiguous → 0
        //
        // ...and `New Terminal` is **not** a match, because the second word spells
        // `Terminal` with one `t`. The first version of this test asserted the
        // opposite and then asserted a single result; both were my expectations
        // about spelling rather than the matcher, which is exactly what a fuzzy
        // matcher's tests exist to catch.
        let ranked = rank(&list, "tt");
        assert_eq!(ranked, vec![5, 8, 7, 1]);
        assert!(!matches("New Terminal", "tt"));
        // The word-start key, on the rows where it decides: `Toggle Terminal` has
        // two boundaries and `agent-workbench/terminal.rs` one.
        assert_eq!(*ranked.first().unwrap(), 5);
        // ...and `Command Palette` beats `Split Right` for the same reason
        // `Terminal` beats `The remote endpoint`: a contiguous run is a real
        // match where two scattered characters with no word start are an accident.
        // Position alone does not decide it — `Split Right` matches at 4 and
        // `Command Palette` at 12.
        // `gtf` finds exactly one thing.
        assert_eq!(rank(&list, "gtf"), vec![6]);
        // `de` finds Delete first: a word start and a contiguous run.
        assert_eq!(*rank(&list, "de").first().unwrap(), 4);
    }

    #[test]
    fn test_a_unicode_candidate_is_matched_by_character_not_by_byte() {
        // `…` and `⌘` are multi-byte; a byte-indexed walk would slice a character
        // in half and either panic or miss.
        assert!(matches("Rename…", "e…"));
        assert!(matches("切换终端", "切换"));
        assert_eq!(top(&["Rename…", "Rename"], "e…").as_deref(), Some("Rename…"));
    }
}
