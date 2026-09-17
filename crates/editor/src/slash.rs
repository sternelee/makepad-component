//! The slash menu: type `/` at the start of a block, choose a kind, the block becomes it.
//!
//! ## The menu's answer is [`SetKind`], not a kind of its own
//!
//! A menu could carry its own list of kinds, and that list would be a **second list to keep in step** with
//! [`BlockKind`] — the exact thing `SetKind`'s doc calls out. So the items name `SetKind`, and the edit is
//! `markdown::edit::apply` with a `SetKind` shortcut: **this module decides what the menu says and which row is
//! chosen, and the edit path is the one that already exists.**
//!
//! ## Which order the matches are in, and why it is not a ranking
//!
//! [`mp/search`](../../../ui/src/mp/search.rs) **ranks** a palette, and this deliberately does not: filtering keeps the
//! menu's **declaration order**. The two are different problems. A palette searches a large set once and wants the best
//! row first. A menu is typed into **while watching it**, and a row that moves because a later letter happens to score
//! better is a row you cannot reach by muscle memory — type `h`, press Enter, and the heading is where you learned it
//! was. So `refilter` narrows the set and never reorders it.
//!
//! ## There is always a chosen row, unlike the combobox
//!
//! [`mp::Combobox`](../../../ui/src/mp/combobox.rs) opens with **no row chosen**, because a combobox's text is a value
//! that survives — you can type a name that is not in the list and keep it. A slash menu is the opposite: a chosen row
//! is what Enter means, and there is nowhere for the text to survive to, because the `/query` is not the block's text.
//! So the first match is chosen, and the one case with no choice is the one with no matches — where Enter should insert
//! nothing and Escape should close the menu.
//!
//! ## The trigger is a question, and whether to ask it is the caller's
//!
//! [`query`] answers *what is being typed* — the word after a `/` that begins a line, at the caret. Whether a menu
//! should **open** is a different question: not inside a `Code` block, not mid-word, and not while composing. Keeping
//! those apart is what makes this testable without a document, a selection, or a window.

use makepad_markdown::SetKind;

/// One row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// The row's text.
    pub label: &'static str,
    /// What choosing it makes the block.
    pub kind: SetKind,
    /// The characters shown at the end of the row, or `""`.
    pub hint: &'static str,
}

/// The menu, in the order it is shown.
///
/// A fixed array rather than a built `Vec`, because the order is the muscle memory: a row that moved for any reason
/// would be a row a person could not learn. Every [`SetKind`] appears **exactly once** — a test asserts that from both
/// directions, so a kind added to the model cannot quietly miss a row and a row cannot name a kind twice.
pub const ITEMS: &[Item] = &[
    Item { label: "Paragraph", kind: SetKind::Paragraph, hint: "" },
    Item { label: "Heading 1", kind: SetKind::Heading1, hint: "#" },
    Item { label: "Heading 2", kind: SetKind::Heading2, hint: "##" },
    Item { label: "Heading 3", kind: SetKind::Heading3, hint: "###" },
    Item { label: "Bulleted list", kind: SetKind::Bullet, hint: "-" },
    Item { label: "Numbered list", kind: SetKind::Ordered, hint: "1." },
    Item { label: "Task", kind: SetKind::Task, hint: "[]" },
    Item { label: "Quote", kind: SetKind::Quote, hint: ">" },
    Item { label: "Code", kind: SetKind::Code, hint: "```" },
    Item { label: "Divider", kind: SetKind::Divider, hint: "---" },
];

/// Every row, as a slice.
///
/// Returns the const rather than a copy, because the menu is data a caller reads — and a caller that mutated its own
/// copy would be a caller whose order differed from the order every other caller learned.
pub fn items() -> &'static [Item] {
    ITEMS
}

/// The row for a kind, or `None` if the menu does not offer it.
pub fn item(kind: SetKind) -> Option<&'static Item> {
    ITEMS.iter().find(|item| item.kind == kind)
}

/// The label for a kind, or `None`.
pub fn label(kind: SetKind) -> Option<&'static str> {
    item(kind).map(|item| item.label)
}

/// The query being typed, if the caret is in a `/word` run that begins a line.
///
/// `None` means **there is no menu here**, and `Some("")` means **the `/` is typed and nothing after it yet** — the
/// two are different states, and a menu that could not tell them apart could not show every row the moment you type
/// the slash, which is the moment a person wants to see what there is.
///
/// The rule: the `/` must **begin its line**, and only word characters, spaces, `-` and `_` may sit between it and the
/// caret. A `/` mid-line is a path or a date — `2026/09/16` is not a menu — and a `/` followed by punctuation is a
/// sentence. This reads **text**, not a document: a caller that must not open inside a code fence decides that for
/// itself, because this does not know what block it is in.
///
/// Never panics: a `caret` past the end of the text, or not on a character boundary, is `None`.
pub fn query(caret: usize, text: &str) -> Option<String> {
    if caret > text.len() || !text.is_char_boundary(caret) {
        return None;
    }
    let before = &text[..caret];
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    let run = &before[line_start..];
    let rest = run.strip_prefix('/')?;
    if rest.is_empty() {
        return Some(String::new());
    }
    if !rest
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-' || c == '_')
    {
        return None;
    }
    Some(rest.to_string())
}

/// The menu, open at a place in the document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlashMenu {
    /// Where the `/` is, so the query text can be spliced out when a row is chosen.
    at: usize,
    /// The characters typed after the `/`.
    query: String,
    /// Indices into [`ITEMS`] that match, **in declaration order**.
    matches: Vec<usize>,
    /// The chosen row, as an index into `matches`.
    active: usize,
}

impl SlashMenu {
    /// Open the menu with the `/` at `at` — byte offset, in the block's own text.
    ///
    /// Opens showing **every** row, because that is what `/` alone means and what tells a person what the menu has.
    pub fn open(at: usize) -> Self {
        Self {
            at,
            query: String::new(),
            matches: (0..ITEMS.len()).collect(),
            active: 0,
        }
    }

    /// Where the `/` is.
    pub fn at(&self) -> usize {
        self.at
    }

    /// The characters typed after the `/`.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The rows that match, in declaration order.
    pub fn matches(&self) -> impl Iterator<Item = &'static Item> + '_ {
        self.matches.iter().map(|index| &ITEMS[*index])
    }

    /// How many rows match.
    pub fn len(&self) -> usize {
        self.matches.len()
    }

    /// Whether nothing matches — the one case with no chosen row.
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    /// Which matching row is chosen, as a position in the **filtered** list.
    pub fn active(&self) -> usize {
        self.active
    }

    /// The chosen row, or `None` when nothing matches.
    ///
    /// `None` is what Enter should act on: with nothing matching, Enter inserts nothing.
    pub fn choice(&self) -> Option<SetKind> {
        self.matches.get(self.active).map(|index| ITEMS[*index].kind)
    }

    /// Narrow the menu to `query`.
    ///
    /// Matches a **case-insensitive substring or subsequence** of the label or of the hint.
    ///
    /// The substring half is the obvious one. The subsequence half is what makes the two-letter forms a person
    /// actually types work: `bl` finds "Bulleted list" and `ni` finds "Numbered list", and neither is a substring of
    /// its label. And because matching compares **compacted** text on both sides, `h1` finds "Heading 1" — the space
    /// does not have to be typed — while `#` still finds the headings through their hints.
    ///
    /// The order is never changed: see the module doc.
    ///
    /// The chosen row goes back to the **first** match, because a narrowing that kept a stale position would apply a
    /// row the person is no longer looking at.
    pub fn refilter(&mut self, query: &str) {
        self.query = query.to_string();
        let needle = compact(query);
        let raw = query.trim();
        self.matches = ITEMS
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                // Nothing typed: every row, so `/` alone shows what the menu has.
                if raw.is_empty() {
                    return true;
                }
                // **The query was punctuation only**, so it compacted to nothing: match it literally instead. This is
                // the case `#` and ``` ``` ``` are, and treating a vanished needle as an empty query would answer `#`
                // with the entire menu — which reads as "# is not understood" when it is the way a person who knows
                // markdown asks for a heading.
                if needle.is_empty() {
                    return item.hint.contains(raw)
                        || item.label.to_ascii_lowercase().contains(&raw.to_ascii_lowercase());
                }
                fits(&compact(item.label), &needle) || fits(&compact(item.hint), &needle)
            })
            .map(|(index, _)| index)
            .collect();
        self.active = 0;
    }

    /// Move the chosen row by `delta`, **clamping** at both ends.
    ///
    /// Clamped rather than wrapped: a menu is a list you walk to a place, and a wrap at the end puts you somewhere you
    /// did not intend while holding a key. Same reasoning as a list that does not wrap.
    pub fn step(&mut self, delta: isize) {
        if self.matches.is_empty() {
            return;
        }
        let last = self.matches.len() - 1;
        let next = self.active as isize + delta;
        self.active = next.clamp(0, last as isize) as usize;
    }

    /// The range the typed query occupies, and how many characters `choice` replaces — the splice a caller performs.
    ///
    /// `(start, end)` in the block's text: `/` through the end of the query. Choosing a row removes it, because the
    /// query is not the block's text; **that is the difference from a combobox**, where the typed text is the value.
    pub fn span(&self, caret: usize) -> (usize, usize) {
        (self.at, caret)
    }
}

/// A label or hint reduced to what matching compares: lowercased, with spaces and punctuation dropped.
///
/// So `h1` finds "Heading 1" and `bulleted` finds "Bulleted list", and a person does not have to type the space. Done
/// on both sides of the comparison, because doing it on one side only is how `#` fails to find a `#` hint.
///
/// Note what it drops and what it keeps: `#` and ``` ``` ``` compact to nothing, which is why the **hint** of a row
/// has to be compared as its own compacted string rather than folded into the label.
fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Whether a compacted label answers to a compacted needle: as a substring, or as a subsequence.
///
/// Substring first, because it is the case a person expects, and the subsequence after, because it is the case that
/// makes two-letter queries work. Both would be satisfied by the subsequence alone; the substring is kept first
/// because it states the intent, and because a later change that stopped allowing gaps would then only narrow the
/// subsequence half rather than silently changing what the common case means.
fn fits(haystack: &str, needle: &str) -> bool {
    if haystack.contains(needle) {
        return true;
    }
    let mut candidates = haystack.chars();
    needle
        .chars()
        .all(|wanted| candidates.any(|candidate| candidate == wanted))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A **subsequence** match, over a compacted needle: every character of the needle in order, gaps allowed.
    ///
    /// This is the second rule matching has, beyond the substring, and it is what makes `bl` find "Bulleted list" and
    /// `ni` find "Numbered list" — the two-letter forms a person actually types.
    fn subsequence(haystack: &str, needle: &str) -> bool {
        let mut chars = haystack.chars();
        needle
            .chars()
            .all(|needle| chars.any(|candidate| candidate == needle))
    }

    #[test]
    fn test_every_kind_the_model_has_appears_exactly_once_in_the_menu() {
        // The whole reason the menu names `SetKind`. Checked from **both** directions: a kind added to the model must
        // not quietly miss a row, and a row must not name a kind twice — which would be a row that shadows another.
        let all = [
            SetKind::Paragraph,
            SetKind::Heading1,
            SetKind::Heading2,
            SetKind::Heading3,
            SetKind::Bullet,
            SetKind::Ordered,
            SetKind::Task,
            SetKind::Quote,
            SetKind::Code,
            SetKind::Divider,
        ];
        assert_eq!(
            all.len(),
            ITEMS.len(),
            "the model has a different number of kinds than the menu has rows"
        );
        for kind in all {
            let count = ITEMS.iter().filter(|item| item.kind == kind).count();
            assert_eq!(count, 1, "{kind:?} appears {count} times in the menu");
        }
    }

    #[test]
    fn test_a_slash_at_a_line_start_is_a_query_and_one_mid_line_is_not() {
        // A slash mid-line is a path or a date: `2026/09/16` is not a menu, and a menu that opened on it would make
        // dates unusable.
        assert_eq!(query(1, "/"), Some(String::new()));
        assert_eq!(query(5, "/head"), Some("head".to_string()));
        assert_eq!(query(10, "text\n/head"), Some("head".to_string()));
        assert_eq!(query(11, "2026/09/16"), None);
        assert_eq!(query(6, "and/or more"), None);
    }

    #[test]
    fn test_a_slash_with_nothing_after_it_is_an_empty_query_rather_than_no_menu() {
        // **The two states are different.** `Some("")` is the slash typed with nothing after it — the moment a person
        // wants to see everything the menu has — and `None` is no menu here at all. A menu that could not tell them
        // apart could not show the rows at the moment they are wanted.
        assert_eq!(query(2, "/x").map(|q| q.is_empty()), Some(false));
        assert_eq!(query(1, "/"), Some(String::new()));
        assert_eq!(SlashMenu::open(0).len(), ITEMS.len());

        let mut menu = SlashMenu::open(0);
        menu.refilter("");
        assert_eq!(menu.len(), ITEMS.len(), "an empty query shows every row");
    }

    #[test]
    fn test_punctuation_after_the_slash_is_not_a_query() {
        // A `/` followed by punctuation is a sentence, not a menu.
        assert_eq!(query(3, "/a."), None);
        assert_eq!(query(4, "/a/b"), None);
        assert_eq!(query(3, "/a!"), None);
        // ...but the characters a menu name actually has are allowed.
        assert_eq!(query(8, "/number-"), Some("number-".to_string()));
        assert_eq!(query(7, "/num_be"), Some("num_be".to_string()));
        assert_eq!(query(5, "/task"), Some("task".to_string()));
    }

    #[test]
    fn test_a_caret_outside_the_text_answers_none_rather_than_panicking() {
        // A caller computes the caret, and a caret that is stale by one frame is normal. A panic here would be a crash
        // from a very ordinary race, so the boundary is checked rather than trusted — including a caret that is not on
        // a character boundary, which would make slicing the text panic.
        assert_eq!(query(10, "/abc"), None);
        assert_eq!(query(0, "/abc"), None);
        assert_eq!(query(99, "/abc"), None);
        // `/é` is 3 bytes: a caret at byte 2 is inside the `é`, so it cannot slice.
        assert_eq!(query(2, "/é"), None);
    }

    #[test]
    fn test_filtering_narrows_the_set_and_never_reorders_it() {
        // **The design point.** A palette ranks; a menu keeps its order, because a row that moved while you were
        // watching it is a row you cannot reach by muscle memory.
        let mut menu = SlashMenu::open(0);
        menu.refilter("list");
        let matching: Vec<&str> = menu.matches().map(|item| item.label).collect();
        assert_eq!(matching, vec!["Bulleted list", "Numbered list"]);

        // In declaration order, whatever the query, and a later query is a narrowing of the declaration order.
        for needle in ["l", "li", "list", "i"] {
            let mut menu = SlashMenu::open(0);
            menu.refilter(needle);
            let indices: Vec<usize> = menu
                .matches()
                .map(|item| ITEMS.iter().position(|other| other.kind == item.kind).unwrap())
                .collect();
            let mut sorted = indices.clone();
            sorted.sort();
            assert_eq!(indices, sorted, "{needle:?} reordered the menu");
        }
    }

    #[test]
    fn test_a_prefix_of_a_later_word_finds_a_row() {
        // `h1` finds "Heading 1" and `bl` finds "Bulleted list": the forms a person actually types, which is why
        // matching compacts spaces out of both sides rather than requiring them.
        for (needle, expected) in [("h1", "Heading 1"), ("bl", "Bulleted list"), ("ni", "Numbered list")] {
            let mut menu = SlashMenu::open(0);
            menu.refilter(needle);
            let labels: Vec<&str> = menu.matches().map(|item| item.label).collect();
            assert!(
                labels.contains(&expected),
                "{needle:?} did not find {expected:?}, found {labels:?}"
            );
        }
    }

    #[test]
    fn test_the_slash_alone_shows_every_row_and_the_hint_finds_the_headings() {
        // `#` finding the headings is how a person who knows markdown types it, and it is why the hint is matched too.
        // **And it is the case that punishes a naive implementation**: `#` compacts to nothing, so a matcher that
        // treated "the needle compacted away" as "nothing was typed" would answer `#` with the whole menu.
        let mut menu = SlashMenu::open(0);
        menu.refilter("#");
        let labels: Vec<&str> = menu.matches().map(|item| item.label).collect();
        assert_eq!(labels, vec!["Heading 1", "Heading 2", "Heading 3"]);

        let mut menu = SlashMenu::open(0);
        menu.refilter("```");
        let labels: Vec<&str> = menu.matches().map(|item| item.label).collect();
        assert_eq!(labels, vec!["Code"]);

        // A punctuation query that matches nothing is still nothing, rather than every row.
        let mut menu = SlashMenu::open(0);
        menu.refilter("???");
        assert!(menu.is_empty(), "punctuation matched the whole menu: {:?}", menu.matches().map(|i| i.label).collect::<Vec<_>>());

        // ...while a genuinely empty query is every row, which is the other state.
        let mut menu = SlashMenu::open(0);
        menu.refilter("");
        assert_eq!(menu.len(), ITEMS.len());
    }

    #[test]
    fn test_the_chosen_row_is_the_first_match_so_enter_always_has_a_row() {
        // Unlike the combobox, which opens with **no** row chosen because its text is a value that survives. A slash
        // menu has nowhere for the query to survive to, so there is always a row for Enter to take.
        let mut menu = SlashMenu::open(0);
        assert_eq!(menu.active(), 0);
        assert_eq!(menu.choice(), Some(SetKind::Paragraph));
        menu.refilter("head");
        assert_eq!(menu.active(), 0, "narrowing did not reset the chosen row");
        assert_eq!(menu.choice(), Some(SetKind::Heading1));
    }

    #[test]
    fn test_nothing_matching_is_the_one_case_with_no_row_chosen() {
        // Enter inserts nothing rather than the top row of a list that has no rows.
        let mut menu = SlashMenu::open(0);
        menu.refilter("zzzz");
        assert!(menu.is_empty());
        assert_eq!(menu.choice(), None);
        // ...and walking a menu with no rows does nothing, rather than wrapping to a row that is not there.
        menu.step(1);
        menu.step(-1);
        assert_eq!(menu.active(), 0);
        assert_eq!(menu.choice(), None);
    }

    #[test]
    fn test_walking_the_menu_clamps_at_both_ends_rather_than_wrapping() {
        // A menu is a list you walk to a place. A wrap puts you somewhere you did not intend while holding a key.
        let mut menu = SlashMenu::open(0);
        menu.refilter("head");
        assert_eq!(menu.len(), 3);
        menu.step(-1);
        assert_eq!(menu.active(), 0, "stepping up from the first row wrapped to the last");
        menu.step(1);
        menu.step(1);
        assert_eq!(menu.active(), 2);
        menu.step(1);
        assert_eq!(menu.active(), 2, "stepping down from the last row wrapped to the first");
    }

    #[test]
    fn test_the_answer_is_the_kind_the_row_names() {
        // The menu decides which row; the edit path takes it from there. Checked for a row in the middle of the list
        // as well as the first, because an off-by-one in the index would still pass for row zero.
        let mut menu = SlashMenu::open(0);
        menu.refilter("quote");
        assert_eq!(menu.choice(), Some(SetKind::Quote));

        let mut menu = SlashMenu::open(0);
        menu.refilter("divider");
        assert_eq!(menu.choice(), Some(SetKind::Divider));

        // Label lookup is the inverse, and covers every row.
        for item in items() {
            assert_eq!(label(item.kind), Some(item.label));
        }
        assert!(label(SetKind::Paragraph).is_some());
    }

    #[test]
    fn test_choosing_a_row_replaces_the_typed_query_and_not_the_block() {
        // `span` is the splice: from the `/` to the caret. The query is not the block's text — that is the difference
        // from a combobox, where the typed text **is** the value and survives.
        // A block whose whole text is the query — the normal case, since the `/` is the first character.
        let text = "/head";
        let caret = text.len();
        let query = query(caret, text).expect("the caret is in a query");
        let at = caret - query.len() - 1;
        assert_eq!(at, 0, "the `/` is the first character");

        let mut menu = SlashMenu::open(at);
        menu.refilter(&query);
        assert_eq!(menu.query(), "head");
        assert_eq!(menu.span(caret), (0, 5));

        // Splice the span out and the block's own text is what remains: nothing. The query was never the block's
        // text, which is the difference from a combobox — where the same splice would leave the block's value intact
        // because the typed characters **are** the value.
        let (start, end) = menu.span(caret);
        assert_eq!(format!("{}{}", &text[..start], &text[end..]), "");

        // ...and a query that follows text leaves that text: `/head` typed after "Title" splices to "Title".
        let text = "Title /head";
        let caret = text.len();
        let menu = {
            let mut menu = SlashMenu::open(6);
            menu.refilter("head");
            menu
        };
        let (start, end) = menu.span(caret);
        assert_eq!(format!("{}{}", &text[..start], &text[end..]), "Title ");
    }

    #[test]
    fn test_the_subsequence_rule_the_matcher_promises() {
        // `compact` is what makes the promise; this checks the rule it enables, so the doc's example cannot drift from
        // the behaviour. (`h1` is already a substring of "heading1", so the subsequence rule is what `bl` relies on.)
        assert!(subsequence(&compact("Bulleted list"), &compact("bl")));
        assert!(subsequence(&compact("Numbered list"), &compact("ni")));
        assert!(!subsequence(&compact("Quote"), &compact("qz")));
    }
}
