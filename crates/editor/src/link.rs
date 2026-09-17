//! The paste menu: a URL landed, and what it could be instead.
//!
//! ## The link is already in the block by the time the menu opens
//!
//! That is the shape worth keeping: **backing out is doing nothing, and the richer form is the upgrade.** The URL was
//! pasted, so it is already text the document can write down; the menu offers what to turn it into. Which is also why
//! [`Choice::Dismiss`] is a **row** rather than only a key — what it leaves behind is a bare URL, and a bare URL is a
//! link this model can still write down faithfully. A menu whose only exit used the keyboard would be a menu with no
//! visible exit.
//!
//! ## Which rows are offered depends on where the URL landed
//!
//! - A **chip** is a mark over text, so it fits either way.
//! - A **card** (bookmark, embed) is a *block*, so it is offered only where the URL has a block to itself.
//! - An **image** is offered where a card is, and only for a name that says it is one — see
//!   [`markdown::is_image`]. **A row that paints a broken box is worse than a row that is not there.**
//!
//! So the same paste offers two rows in one place and five in another, and `open` is where that decision lives rather
//! than in the caller.
//!
//! ## Walking the rows clamps, it does not wrap
//!
//! Same rule as the slash menu, and for the same reason: a menu is a list you walk to a place, and a wrap at the end
//! puts you somewhere you did not intend while holding a key.
//!
//! ## What applying a choice needs, and what the model has today
//!
//! The menu's rows, their order, and which are offered from where are **decided and tested**; applying them is the next
//! slice, and it needs model support that does not exist yet. Named precisely, because "wire it up later" is not a
//! plan:
//!
//! | choice | what applying it needs | today |
//! | --- | --- | --- |
//! | [`Choice::Dismiss`] | nothing — the URL is already in the block | **available** |
//! | [`Choice::Chip`] | a link: either a [`Mark`] carrying a URL, or the `[text](url)` syntax | text-only today; the syntax round-trips, so it is usable but is not a mark |
//! | [`Choice::Bookmark`] | a block that holds a URL and renders as a card | **not in the model** |
//! | [`Choice::Embed`] | the same, rendering something else | **not in the model** |
//! | [`Choice::Image`] | a block holding a URL and alt text | **not in the model** |
//!
//! So this module is deliberately **not wired to a key yet**: a menu whose rows do nothing is worse than a module with
//! tests, and there is a rule in this port against exactly that — *"a page whose subject has nothing to act on verifies
//! nothing."* What is missing is one model addition, not a block of UI work.
//!
//! ## A place in the document is a [`Selection`], not a second kind of cursor
//!
//! The reference names this spot with its own `Cursor`. This port already names a place with `Selection` — the slash
//! menu, the history, the layout, and every edit take one — and a second way to name a place would be a second way to
//! be wrong about which place is meant. So `at` is a `Selection`, and it is a caret.

use makepad_markdown::{is_image, Selection};

/// Whether a paste is a URL, and so whether a menu should open at all.
///
/// **A scheme is required, and that is the whole rule.** `http://` or `https://`, nothing before it, and no whitespace
/// anywhere in it. A bare `example.com` is deliberately **not** a URL here: telling a host from a filename needs a list
/// of top-level domains, which is a guess that goes stale, and `README.md` and `thing.py` are the words a document
/// actually contains. Text with a space in it is prose, and a URL never has one.
///
/// The case is checked, because a scheme's case is fixed by the spec even though a host's is not.
pub fn looks_like_url(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() || text.chars().any(char::is_whitespace) {
        return false;
    }
    // Case-insensitively, because `HTTPS://` is the same scheme — and slicing by the prefix's own length rather than
    // by character count, so a scheme spelled in a mixture of cases still leaves the right remainder.
    let rest = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .or_else(|| text.strip_prefix("HTTPS://"))
        .or_else(|| text.strip_prefix("HTTP://"));
    matches!(rest, Some(rest) if !rest.is_empty())
}

/// What a row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Leave the bare URL. **A row, not only a key**: what it leaves behind is a link the model can write down.
    Dismiss,
    /// A mark over the text: what the URL becomes inside a sentence.
    Chip,
    /// A card, alone, with the URL as its title.
    Bookmark,
    /// A card, alone, painting what the URL is.
    Embed,
    /// A picture, alone.
    Image,
}

/// Every choice, in the order the menu offers them.
///
/// A `const` like the slash menu's items, and for the same reason: the order is the muscle memory.
pub const CHOICES: &[Choice] = &[
    Choice::Dismiss,
    Choice::Chip,
    Choice::Bookmark,
    Choice::Embed,
    Choice::Image,
];

impl Choice {
    /// The row's text.
    pub fn label(self) -> &'static str {
        match self {
            Self::Dismiss => "Dismiss",
            Self::Chip => "Create chip",
            Self::Bookmark => "Create bookmark",
            Self::Embed => "Create embed",
            Self::Image => "Create image",
        }
    }

    /// Whether this choice needs the URL to have a block to itself.
    ///
    /// A chip is a mark over text, so it does not; a card is a block, so it does. Stated here rather than only inside
    /// `open`, because a **caller that offers the choices itself** — a keyboard path, a context menu — has to ask the
    /// same question, and asking it a second way is how the two disagree.
    pub fn needs_alone(self) -> bool {
        matches!(self, Self::Bookmark | Self::Embed | Self::Image)
    }
}

/// An open paste menu: the link that landed, and the row that is chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paste {
    /// Where the URL went — what a card replaces, and where the menu is anchored.
    pub at: Selection,
    /// The URL, as pasted.
    pub url: String,
    /// Whether the URL has a block to itself, which is what decides the rows and what a chip becomes: an element with
    /// its own title there, a mark over shaped text anywhere else.
    pub alone: bool,
    /// What this spot can hold, in the menu's order.
    pub rows: Vec<Choice>,
    /// The chosen row, as a position in `rows`.
    pub active: usize,
}

impl Paste {
    /// Open the menu for a URL that just landed at `at`.
    ///
    /// The rows are decided **here**, from `alone` and from the URL's own name, so a caller cannot offer a card where
    /// there is no block for it.
    pub fn open(at: Selection, url: impl Into<String>, alone: bool) -> Self {
        let url = url.into();
        let mut rows = vec![Choice::Dismiss, Choice::Chip];
        if alone {
            rows.push(Choice::Bookmark);
            rows.push(Choice::Embed);
            // **Only for a name that says it is one.** A row that paints a broken box is worse than a row that is not
            // there, and `is_image` is the same predicate a renderer asks, so the two cannot disagree.
            if is_image(&url) {
                rows.push(Choice::Image);
            }
        }
        Self {
            at,
            url,
            alone,
            rows,
            active: 0,
        }
    }

    /// The rows, in the menu's order.
    pub fn rows(&self) -> impl Iterator<Item = Choice> + '_ {
        self.rows.iter().copied()
    }

    /// How many rows there are. **Never zero** — `Dismiss` is always one, which is what makes a visible exit
    /// guaranteed.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Never true, and present because a `len` without it is a lint: `Dismiss` is always offered.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The chosen row.
    pub fn choice(&self) -> Choice {
        // The rows are never empty — `open` always pushes `Dismiss` — so this cannot panic; the fallback is the exit
        // row rather than an index, because if it were ever reached, leaving the URL alone is the safe answer.
        self.rows.get(self.active).copied().unwrap_or(Choice::Dismiss)
    }

    /// Which row is chosen, as a position in `rows`.
    pub fn active(&self) -> usize {
        self.active
    }

    /// Walk the rows, **clamping** at both ends.
    pub fn step(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let last = self.rows.len() as isize - 1;
        self.active = (self.active as isize + delta).clamp(0, last) as usize;
    }

    /// Whether the chosen row leaves the document unchanged.
    ///
    /// The caller's test for "did the user accept an upgrade or back out", so it does not have to match on the variant
    /// — and so a second dismissing variant cannot be added without this being the place that notices.
    pub fn dismisses(&self) -> bool {
        self.choice() == Choice::Dismiss
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caret() -> Selection {
        Selection::caret(0, 0)
    }

    #[test]
    fn test_a_url_in_a_sentence_is_offered_the_mark_and_the_exit() {
        // A chip is a mark over text, so it fits inside a sentence; a card is a block, so it does not. So the same
        // paste offers two rows here and five in a block of its own.
        let paste = Paste::open(caret(), "https://example.com/thing", false);
        assert_eq!(paste.rows().collect::<Vec<_>>(), vec![Choice::Dismiss, Choice::Chip]);
        assert_eq!(paste.len(), 2);
    }

    #[test]
    fn test_a_url_alone_is_offered_the_cards_as_well() {
        let paste = Paste::open(caret(), "https://example.com/thing", true);
        assert_eq!(
            paste.rows().collect::<Vec<_>>(),
            vec![Choice::Dismiss, Choice::Chip, Choice::Bookmark, Choice::Embed]
        );
    }

    #[test]
    fn test_a_picture_is_offered_only_where_a_card_is_and_only_for_a_name_that_says_so() {
        // **A row that paints a broken box is worse than a row that is not there.** Both halves of the rule: the URL
        // must have a block to itself, and its name must say it is an image.
        let alone = Paste::open(caret(), "https://host/pic.png", true);
        assert!(alone.rows().any(|choice| choice == Choice::Image));

        let in_a_sentence = Paste::open(caret(), "https://host/pic.png", false);
        assert!(!in_a_sentence.rows().any(|choice| choice == Choice::Image));

        let not_a_picture = Paste::open(caret(), "https://host/page.html", true);
        assert!(!not_a_picture.rows().any(|choice| choice == Choice::Image));
    }

    #[test]
    fn test_the_exit_is_always_a_row_and_is_always_the_first_one() {
        // **What it leaves behind is a bare URL, and that is a link this model can still write down faithfully.** A
        // menu whose only exit used the keyboard would be a menu with no visible exit. Being first also means Enter
        // with nothing touched leaves the document alone, which is the safe default for something that appeared on its
        // own.
        for (url, alone) in [
            ("https://host/thing", false),
            ("https://host/thing", true),
            ("https://host/pic.png", true),
            ("", false),
            ("not a url at all", false),
        ] {
            let paste = Paste::open(caret(), url, alone);
            assert!(paste.len() >= 2, "{url:?} offered {} rows", paste.len());
            assert_eq!(paste.choice(), Choice::Dismiss, "{url:?} did not open on the exit");
            assert!(paste.dismisses());
            assert!(!paste.is_empty());
        }
    }

    #[test]
    fn test_walking_the_rows_clamps_at_both_ends_rather_than_wrapping() {
        // Same rule as the slash menu: a menu is a list you walk to a place, and a wrap puts you somewhere you did not
        // intend while holding a key.
        let mut paste = Paste::open(caret(), "https://host/thing", true);
        assert_eq!(paste.len(), 4);
        paste.step(-1);
        assert_eq!(paste.active(), 0, "stepping up from the first row wrapped");
        for _ in 0..10 {
            paste.step(1);
        }
        assert_eq!(paste.active(), paste.len() - 1, "stepping past the last row wrapped");
        assert_eq!(paste.choice(), Choice::Embed);
        assert!(!paste.dismisses());
    }

    #[test]
    fn test_every_choice_has_a_label_and_the_untouched_menu_agrees_with_needs_alone() {
        // `needs_alone` is public because a caller offering the choices itself has to ask the same question. This
        // checks it against what `open` actually does, so the two cannot drift: a choice that claims to need a block of
        // its own must be **absent** where there is none.
        for choice in CHOICES {
            assert!(!choice.label().is_empty(), "{choice:?} has no label");

            let alone = Paste::open(caret(), "https://host/pic.png", true);
            let in_a_sentence = Paste::open(caret(), "https://host/pic.png", false);
            let offered_alone = alone.rows().any(|row| row == *choice);
            let offered_in_a_sentence = in_a_sentence.rows().any(|row| row == *choice);
            assert!(offered_alone, "{choice:?} is not offered even alone");
            if choice.needs_alone() {
                assert!(
                    !offered_in_a_sentence,
                    "{choice:?} says it needs a block of its own but was offered inside a sentence"
                );
            } else {
                assert!(
                    offered_in_a_sentence,
                    "{choice:?} does not need a block of its own but was not offered inside a sentence"
                );
            }
        }
    }

    #[test]
    fn test_the_choice_list_and_the_menu_are_the_same_set() {
        // `CHOICES` is what a caller iterates to offer the menu itself; `open` decides which of them this spot can
        // hold. A choice in `CHOICES` that `open` can never offer is a row nobody can reach, and a row `open` offers
        // that is not in `CHOICES` is one a caller's own menu would omit.
        let widest = Paste::open(caret(), "https://host/pic.png", true);
        for choice in &widest.rows {
            assert!(CHOICES.contains(choice), "{choice:?} is not in CHOICES");
        }
        for choice in CHOICES {
            assert!(
                widest.rows.contains(choice),
                "{choice:?} is in CHOICES but the widest menu cannot offer it"
            );
        }
    }

    #[test]
    fn test_a_scheme_is_required_and_a_bare_host_is_not_a_url() {
        // **The rule that keeps this from eating prose.** A bare `example.com` needs a list of top-level domains to
        // recognize, which is a guess that goes stale — and `README.md` and `thing.py` are the words a document
        // actually contains.
        for url in [
            "http://localhost:8080",
            "https://example.com",
            "https://example.com/a/b?c=d#e",
            "  https://example.com  ",
            "HTTPS://EXAMPLE.COM",
        ] {
            assert!(looks_like_url(url), "{url:?} should be a URL");
        }
        for not in [
            "",
            "   ",
            "example.com",
            "www.example.com",
            "README.md",
            "thing.py",
            "see https://example.com for more",
            "https://",
            "http:// two words",
            "not a url at all",
        ] {
            assert!(!looks_like_url(not), "{not:?} should not be a URL");
        }
    }

    #[test]
    fn test_the_paste_keeps_the_url_it_was_given() {
        // The menu does not normalize, trim or rewrite it: what it holds is what was pasted, because that is what the
        // document will contain if the row leaves it alone.
        let url = "https://example.com/a b?c=d#e";
        let paste = Paste::open(caret(), url, true);
        assert_eq!(paste.url, url);
    }
}
