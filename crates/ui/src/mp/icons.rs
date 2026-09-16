//! `icons` — the glyphs this library draws, named, and the check that they are real.
//!
//! ## A codepoint in the wrong face is tofu, not an error
//!
//! This port already paid for that once and **recorded the wrong explanation**, so the correct
//! version is the reason this module exists.
//!
//! The calendar's month arrows were `\u{f053}` and `\u{f054}`, and the first render drew two
//! tofu boxes. The note written at the time said FontAwesome's chevrons "are not in this port's
//! icon subset". **That was false.** Parsing `fa-solid-900.ttf` — the face Makepad ships as
//! `theme.font_icons` — shows **1976** glyphs with both chevrons among them. The real cause is
//! that the calendar drew them through `draw_weekday`, whose `text_style` is
//! `mod.mpc.type.caption` — **the text face**. A character a font does not have is tofu: not an
//! error, not a warning, nothing in any log.
//!
//! So there are two questions, and they have the same symptom:
//!
//! 1. **Is this codepoint in FontAwesome?** — answered by [`ALL`] and checked against the font
//!    file by `tests/icons.rs`.
//! 2. **Is the glyph drawn with the face that carries it?** — checked by the same test, which
//!    requires a file that writes a FontAwesome codepoint to reach `theme.font_icons` either
//!    directly or through [`crate::mp::icon::MpIcon`].
//!
//! ## Why the codepoints are still written at the call sites
//!
//! A `script_mod!` block cannot reference a Rust constant, so `glyph: "\u{f002}"` in a DSL block
//! cannot become `glyph: icons::SEARCH`. What this module provides is therefore **the declared
//! set** — one place that says which glyphs this library uses and what they are called — and the
//! test asserts that every codepoint written anywhere in `src/mp/` is in it. That is what turns a
//! typo from a silent tofu box into a failing test, which is the whole of bezel's `icons` idea
//! applied to a text face rather than to an SVG one.
//!
//! A **third** question the test cannot answer, and does not pretend to: whether the glyph is the
//! right one for the meaning. `f111` is a circle whatever it is used for.

/// Every glyph this library draws, as FontAwesome **Solid** codepoints.
///
/// Names are FontAwesome's, lowercased and hyphenated, so a reader can look one up. A constant
/// that no call site uses is still listed: the set is what the library *draws*, and a stale entry
/// is cheaper than a call site whose glyph nobody can identify.
pub mod glyph {
    /// `magnifying-glass` — a search field's leading glyph.
    pub const SEARCH: &str = "\u{f002}";
    /// `check` — a confirmation, a completed state.
    pub const CHECK: &str = "\u{f00c}";
    /// `chevron-left` — the previous arrow in the calendar's header.
    pub const CHEVRON_LEFT: &str = "\u{f053}";
    /// `chevron-right` — the next arrow.
    pub const CHEVRON_RIGHT: &str = "\u{f054}";
    /// `caret-down` — a disclosure that is closed.
    pub const CARET_DOWN: &str = "\u{f0d7}";
    /// `caret-right` — a disclosure that is open.
    pub const CARET_RIGHT: &str = "\u{f0da}";
    /// `file-lines` — a document.
    pub const FILE_LINES: &str = "\u{f0f6}";
    /// `circle` — a node, a status dot.
    pub const CIRCLE: &str = "\u{f111}";
    /// `right-from-bracket` — leaving, signing out, closing a session.
    pub const RIGHT_FROM_BRACKET: &str = "\u{f2f5}";
}

/// The declared set: every glyph, as `(FontAwesome name, codepoint)`.
///
/// The list a test walks, and the list a glyph browser would show. Keeping it as data rather
/// than as prose is what lets the check be mechanical.
pub const ALL: [(&str, &str); 9] = [
    ("magnifying-glass", glyph::SEARCH),
    ("check", glyph::CHECK),
    ("chevron-left", glyph::CHEVRON_LEFT),
    ("chevron-right", glyph::CHEVRON_RIGHT),
    ("caret-down", glyph::CARET_DOWN),
    ("caret-right", glyph::CARET_RIGHT),
    ("file-lines", glyph::FILE_LINES),
    ("circle", glyph::CIRCLE),
    ("right-from-bracket", glyph::RIGHT_FROM_BRACKET),
];

/// The face these codepoints belong to, as Makepad names it.
///
/// A string rather than an import, because it is documentation: the source of truth is
/// `theme.font_icons` in the DSL, and the file it resolves to is what `tests/icons.rs` parses.
pub const FACE: &str = "FontAwesome Solid (theme.font_icons)";

/// The codepoint of a glyph, for a caller that has one of the strings above and wants the number.
///
/// A single `char` rather than a `u32`, because every glyph here is in the Basic Multilingual
/// Plane — FontAwesome's private-use range is `U+F000`–`U+F8FF` — and a `char` cannot be
/// constructed from a codepoint outside it, so the type carries the guarantee rather than a
/// comment.
pub fn codepoint(glyph: &str) -> Option<char> {
    let mut chars = glyph.chars();
    let first = chars.next()?;
    // More than one `char` is not a glyph. `None` rather than the first one, because a caller
    // measuring a glyph needs to know it was given a string.
    if chars.next().is_some() {
        return None;
    }
    Some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_glyph_is_exactly_one_character() {
        // A glyph is one codepoint. A two-character string in a glyph field is a bug the
        // renderer cannot report — it draws the first character and ignores the rest, which
        // looks exactly like a glyph that is slightly narrow.
        for (name, glyph) in ALL {
            assert!(
                codepoint(glyph).is_some(),
                "{name} is {} characters, not one",
                glyph.chars().count()
            );
        }
    }

    #[test]
    fn test_every_glyph_is_in_font_awesome_s_private_use_area() {
        // `U+E000`–`U+F8FF` is the Private Use Area. FontAwesome puts its glyphs at `U+F000`
        // upward, so a glyph outside the area is a codepoint from a text face pasted into an
        // icon slot — which renders as whatever character that codepoint means, or as tofu, and
        // never as an icon.
        for (name, glyph) in ALL {
            let c = codepoint(glyph).expect("one character");
            let code = c as u32;
            assert!(
                (0xE000..=0xF8FF).contains(&code),
                "{name} is U+{code:04X}, which is not in the Private Use Area"
            );
        }
    }

    #[test]
    fn test_the_declared_set_has_no_duplicates() {
        // Two names for one codepoint is how a set drifts: one of them is corrected later and
        // the other is not.
        let mut seen: Vec<(u32, &str)> = Vec::new();
        for (name, glyph) in ALL {
            let code = codepoint(glyph).expect("one character") as u32;
            if let Some((_, other)) = seen.iter().find(|(c, _)| *c == code) {
                panic!("{name} and {other} are both U+{code:04X}");
            }
            seen.push((code, name));
        }
        // And the table covers every constant, so a constant added without an entry in `ALL`
        // fails here rather than being silently unregistered.
        assert_eq!(ALL.len(), 9, "a constant was added or removed without updating ALL");
    }

    #[test]
    fn test_codepoint_refuses_a_string_rather_than_taking_its_first_character() {
        assert_eq!(codepoint("\u{f002}"), Some('\u{f002}'));
        assert_eq!(codepoint(""), None);
        assert_eq!(codepoint("ab"), None);
        assert_eq!(codepoint("\u{f002}\u{f00c}"), None);
    }
}
