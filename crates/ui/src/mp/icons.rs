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

    // **The A2UI icon set.** The protocol names its icons with Material Symbols names (`message.rs` says
    // `"settings", "check", "close"`, and the samples use `addToCart`), while this library draws FontAwesome. So a
    // name-to-glyph table is what bridges the two — and every codepoint here joins [`ALL`], which means the cmap test
    // checks each one against the font file. **That is what makes the table trustworthy**: a wrong codepoint is a tofu
    // box, invisible in every log, and the test is the only thing that sees it.
    //
    /// `gear` — settings, preferences.
    pub const GEAR: &str = "\u{f013}";
    /// `xmark` — close, dismiss.
    pub const XMARK: &str = "\u{f00d}";
    /// `plus` — add, create.
    pub const PLUS: &str = "\u{f067}";
    /// `cart-shopping` — a cart, and the "add to cart" affordance.
    pub const CART_SHOPPING: &str = "\u{f07a}";
    /// `trash` — delete.
    pub const TRASH: &str = "\u{f2ed}";
    /// `pen-to-square` — edit.
    pub const PEN_TO_SQUARE: &str = "\u{f044}";
    /// `star` — a rating, a favourite that is on.
    pub const STAR: &str = "\u{f005}";
    /// `heart` — a favourite.
    pub const HEART: &str = "\u{f004}";
    /// `bars` — a menu.
    pub const BARS: &str = "\u{f0c9}";
    /// `house` — home.
    pub const HOUSE: &str = "\u{f015}";
    /// `user` — a person.
    pub const USER: &str = "\u{f007}";
    /// `arrow-left` — back.
    pub const ARROW_LEFT: &str = "\u{f060}";
    /// `arrow-right` — forward.
    pub const ARROW_RIGHT: &str = "\u{f061}";
    /// `ellipsis-vertical` — more.
    pub const ELLIPSIS_VERTICAL: &str = "\u{f142}";
    /// `circle-info` — information.
    pub const CIRCLE_INFO: &str = "\u{f129}";
    /// `triangle-exclamation` — a warning.
    pub const TRIANGLE_EXCLAMATION: &str = "\u{f071}";
    /// `circle-exclamation` — an error.
    pub const CIRCLE_EXCLAMATION: &str = "\u{f06a}";
    /// `arrows-rotate` — refresh.
    pub const ARROWS_ROTATE: &str = "\u{f021}";
    /// `download` — download.
    pub const DOWNLOAD: &str = "\u{f019}";
    /// `upload` — upload.
    pub const UPLOAD: &str = "\u{f093}";
    /// `play` — play.
    pub const PLAY: &str = "\u{f04b}";
    /// `chevron-down` — expand.
    pub const CHEVRON_DOWN: &str = "\u{f078}";
    /// `chevron-up` — collapse.
    pub const CHEVRON_UP: &str = "\u{f077}";
}

/// The declared set: every glyph, as `(FontAwesome name, codepoint)`.
///
/// The list a test walks, and the list a glyph browser would show. Keeping it as data rather
/// than as prose is what lets the check be mechanical.
pub const ALL: [(&str, &str); 32] = [
    ("magnifying-glass", glyph::SEARCH),
    ("check", glyph::CHECK),
    ("chevron-left", glyph::CHEVRON_LEFT),
    ("chevron-right", glyph::CHEVRON_RIGHT),
    ("caret-down", glyph::CARET_DOWN),
    ("caret-right", glyph::CARET_RIGHT),
    ("file-lines", glyph::FILE_LINES),
    ("circle", glyph::CIRCLE),
    ("right-from-bracket", glyph::RIGHT_FROM_BRACKET),
    ("gear", glyph::GEAR),
    ("xmark", glyph::XMARK),
    ("plus", glyph::PLUS),
    ("cart-shopping", glyph::CART_SHOPPING),
    ("trash", glyph::TRASH),
    ("pen-to-square", glyph::PEN_TO_SQUARE),
    ("star", glyph::STAR),
    ("heart", glyph::HEART),
    ("bars", glyph::BARS),
    ("house", glyph::HOUSE),
    ("user", glyph::USER),
    ("arrow-left", glyph::ARROW_LEFT),
    ("arrow-right", glyph::ARROW_RIGHT),
    ("ellipsis-vertical", glyph::ELLIPSIS_VERTICAL),
    ("circle-info", glyph::CIRCLE_INFO),
    ("triangle-exclamation", glyph::TRIANGLE_EXCLAMATION),
    ("circle-exclamation", glyph::CIRCLE_EXCLAMATION),
    ("arrows-rotate", glyph::ARROWS_ROTATE),
    ("download", glyph::DOWNLOAD),
    ("upload", glyph::UPLOAD),
    ("play", glyph::PLAY),
    ("chevron-down", glyph::CHEVRON_DOWN),
    ("chevron-up", glyph::CHEVRON_UP),
];

/// A name the A2UI protocol uses, and the glyph this library draws for it.
///
/// **The two vocabularies are different and neither is wrong**: the protocol is written against Material Symbols
/// (`"settings"`, `"check"`, `"close"`, `addToCart`), and this library draws FontAwesome, because that is the icon face
/// Makepad ships. So the bridge is a table, and the table is the only place the two names for one idea meet.
///
/// The names are matched **case-insensitively and after trimming**, because they arrive in JSON that a model wrote —
/// `"addToCart"` and `"add_to_cart"` and `"ADD_TO_CART"` are one request. Both spellings are listed rather than derived:
/// a rule that turned `addToCart` into `add_to_cart` would also turn `arrowBack` into `arrow_back`, which is right, and
/// `moreVert` into `more_vert`, which is also right — but the rule is a guess about a third-party vocabulary, and the
/// cost of listing a name twice is one line while the cost of guessing wrong is an icon that silently does not draw.
///
/// **An unknown name is `None` and the caller draws nothing.** A fallback glyph would be a lie about what the document
/// asked for, and the rule this port follows is that a row which paints a broken box is worse than a row that is not
/// there — the same rule that keeps an unknown image URL out of the paste menu's rows.
pub const NAMES: [(&str, &str); 34] = [
    ("settings", glyph::GEAR),
    ("gear", glyph::GEAR),
    ("check", glyph::CHECK),
    ("done", glyph::CHECK),
    ("close", glyph::XMARK),
    ("cancel", glyph::XMARK),
    ("add", glyph::PLUS),
    ("addtocart", glyph::CART_SHOPPING),
    ("add_to_cart", glyph::CART_SHOPPING),
    ("shoppingcart", glyph::CART_SHOPPING),
    ("delete", glyph::TRASH),
    ("trash", glyph::TRASH),
    ("edit", glyph::PEN_TO_SQUARE),
    ("star", glyph::STAR),
    ("favorite", glyph::HEART),
    ("favorite_border", glyph::HEART),
    ("menu", glyph::BARS),
    ("home", glyph::HOUSE),
    ("person", glyph::USER),
    ("accountcircle", glyph::USER),
    ("arrowback", glyph::ARROW_LEFT),
    ("arrow_back", glyph::ARROW_LEFT),
    ("arrowforward", glyph::ARROW_RIGHT),
    ("arrow_forward", glyph::ARROW_RIGHT),
    ("morevert", glyph::ELLIPSIS_VERTICAL),
    ("more_vert", glyph::ELLIPSIS_VERTICAL),
    ("info", glyph::CIRCLE_INFO),
    ("warning", glyph::TRIANGLE_EXCLAMATION),
    ("error", glyph::CIRCLE_EXCLAMATION),
    ("refresh", glyph::ARROWS_ROTATE),
    ("download", glyph::DOWNLOAD),
    ("upload", glyph::UPLOAD),
    ("playarrow", glyph::PLAY),
    ("play_arrow", glyph::PLAY),
];

/// The glyph for a protocol icon name, or `None` when this library has no glyph for it.
///
/// Case-insensitive and trimmed. See [`NAMES`] for why unknown is `None` rather than a fallback, and why
/// `chevron_down` is reachable under the FontAwesome spelling `chevron-down` — the names this library draws *for
/// itself* and the names a protocol asks for are separate tables that happen to share codepoints.
pub fn by_name(name: &str) -> Option<&'static str> {
    let needle = name.trim().to_ascii_lowercase().replace('-', "_");
    NAMES
        .iter()
        .find(|(candidate, _)| *candidate == needle)
        .map(|(_, glyph)| *glyph)
        .or_else(|| {
            // The FontAwesome spellings, so a document that names a glyph the way this library names it also works.
            ALL.iter()
                .find(|(candidate, _)| candidate.replace('-', "_") == needle)
                .map(|(_, glyph)| *glyph)
        })
}

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
        //
        // **This used to assert `ALL.len() == 9` — a hardcoded number, which does not check the claim above it.** It
        // caught only "you changed `ALL`", and a constant added *without* an `ALL` entry left the length at 9 and the
        // test green: the one case the comment names was the one case it could not see. Adding twenty-three glyphs is
        // what exposed it — the constants and `ALL` were both updated, and the test failed anyway.
        //
        // So it counts the declarations in `glyph` out of the source, the way `tests/registration_order.rs` reads
        // `src/mp/` for the same kind of rule. **The claim and the check are now the same claim.**
        let source = std::fs::read_to_string("src/mp/icons.rs").expect("the source of this file");
        let glyphs_section = source
            .split_once("pub mod glyph {")
            .and_then(|(_, rest)| rest.split_once("\n}"))
            .map(|(body, _)| body)
            .expect("the glyph module");
        let declared = glyphs_section.matches("pub const ").count();
        assert_eq!(
            ALL.len(),
            declared,
            "there are {declared} constants in `glyph` but {} entries in `ALL` — one of them was added without the \
             other",
            ALL.len()
        );
    }

    #[test]
    fn test_codepoint_refuses_a_string_rather_than_taking_its_first_character() {
        assert_eq!(codepoint("\u{f002}"), Some('\u{f002}'));
        assert_eq!(codepoint(""), None);
        assert_eq!(codepoint("ab"), None);
        assert_eq!(codepoint("\u{f002}\u{f00c}"), None);
    }
}
