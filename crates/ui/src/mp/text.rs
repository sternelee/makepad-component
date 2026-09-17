//! Measuring and clipping one line of text, for the widgets that paint their own
//! rows.
//!
//! `MpTable`, `MpTree` and `MpList` place their content at absolute positions
//! rather than laying out child widgets, so each of them needs the same two
//! answers: how wide is this string, and what is the longest prefix of it that
//! fits? The first two each grew their own copy — the table with a narrow/wide
//! correction and the tree with a flat estimate — which means two widgets could
//! clip the same string at different points.
//!
//! ## Why an estimate rather than a measurement
//!
//! `DrawText` has no measure entry point that does not also draw: the only way to
//! lay out a string is to lay it out. So the width here is the font's own advance
//! at the size in force, plus a correction for the characters that are nowhere
//! near the average — which is what a row of chrome needs and what a *paragraph*
//! must not use. `mp/input.rs` documents the same distinction for the field's
//! caret; a component that has to break a line needs a layout pass, and none of
//! these do.
//!
//! The correction is deliberately two-sided. Narrow characters (`i`, `.`) pull
//! the estimate down and wide ones (`W`, `@`) push it up, so the number is an
//! estimate of *this string* rather than a fudge factor applied to every string
//! alike. A one-sided correction would clip every row of `illll` short and let
//! every row of `WWWW` overflow.
//!
//! ## Placement needs an upper bound, and symbols were the case that broke it
//!
//! Two-sided is right for **clipping**, where the string is cut to fit and being a
//! point either way is invisible. It is wrong for **placement**, where an estimate
//! that comes out *under* the painted width puts the text past the edge it was
//! aligned to.
//!
//! That is not hypothetical, and it is the second time this crate has paid for it.
//! `mp/segmented.rs` recorded the first: an underestimated label made a control's box
//! too narrow and the Bars toolbar drew `Preview` as **`Previ`**. The second was the
//! Shortcuts page, where a matching list's trailing chord is right-aligned and
//! `⇧⌘S` was drawn with its `S` **past the panel**, on top of the page's scroll bar.
//!
//! Both had **two** causes, and the second was found later and is the larger one:
//!
//! 1. **A glyph is not an average character.** `⌘`, `⇧` and `⌥` were each counted at the average
//!    Latin advance when their real advance is close to double it, so non-ASCII characters now get
//!    [`SYMBOL`].
//! 2. **The declared font size is not the painted one.** Makepad lays text out at 96 dpi, so a
//!    `font_size` is multiplied by `96 / 72` before any glyph is placed — and this module did not
//!    know that, which made **every estimate about 25% under the paint**. That is the real reason a
//!    segmented control's last label was clipped and a list's chord ran past its panel; the symbol
//!    correction above addressed a smaller error with the same symptom, and both faults were "fixed"
//!    with padding before the constant was found.
//!
//! [`DPI`] carries the derivation and the measurement that pins it, and with it in place the
//! estimate for a monospace run agrees with the paint to **0.0016pt per character** — checked in
//! `MpCodeBlock`, which prints both.

use makepad_widgets::*;

/// Which face a string is measured in.
///
/// **The same estimator cannot serve both, and this crate now has a widget of each.** A proportional
/// face gives `⌘` and `A` different advances, so an average-character estimate needs per-character
/// corrections. A monospace face gives every character one advance, so those corrections are exactly
/// wrong for it.
///
/// This was removed once, when nothing measured a monospace string — `MpKbd` draws its text with
/// `Walk::fit()` and lets Makepad lay it out, so the estimator was never involved. `mp/code.rs` is
/// the consumer that brought it back: a code block advances its pen by hand, span by span, and it
/// has nothing to lay out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    /// A proportional face: an average Latin advance, corrected per character.
    Proportional,
    /// A monospace face: one advance for every character, glyphs included.
    Mono,
}

/// The factor between a declared font size and the size Makepad **lays text out at**.
///
/// Makepad converts a `font_size` from points to pixels at 96 dpi: `96 / 72`. Every measurement in
/// this module used to omit it, so **every estimate was about 25% under the paint** — which is why a
/// segmented control's last label was clipped and a list's trailing chord ran past its panel, and why
/// both were "fixed" with a padding fudge rather than with the number that was wrong.
///
/// Measured rather than taken from the constant: `MpCodeBlock` drew a 16-character run at 12pt and
/// the rect came back 153.62 wide, and the same run at 24pt came back 307.25 — `9.6016` and `19.2031`
/// per character, the same ratio to five decimal places. Against the mono face's own `hmtx` advance
/// of 0.6 em (LiberationMono is 1229/2048, JetBrains Mono is 600/1000), that ratio is
/// `0.8001 / 0.6 = 1.3336`, and `96 / 72 = 1.3333`.
pub const DPI: f64 = 96.0 / 72.0;

/// The advance of an average character, as a fraction of the font size, for the
/// proportional faces this crate bundles.
///
/// The `0.508` is the face's own average advance **per em**; [`DPI`] is what turns the declared size
/// into the size it is painted at.
const ADVANCE: f64 = 0.508 * DPI;

/// The advance of **every** character in the monospace face this crate bundles, as a fraction of the
/// font size.
///
/// Higher than [`ADVANCE`], which is what a monospace advance is: a mono face reserves the width of
/// its widest glyph for all of them.
/// The mono faces are **0.6 em** per character, read from their own `hmtx` tables rather than
/// assumed: `LiberationMono-Regular.ttf` advances 1229/2048 and `jetbrains_mono_variable.ttf` 600/1000,
/// with every glyph in each taking the same advance — which is what makes a code block's pen
/// legitimate. See [`DPI`] for the rest of the ratio.
pub const MONO_ADVANCE: f64 = 0.6 * DPI;

/// How far a narrow or wide character moves the estimate, as a fraction of the
/// average advance.
const NARROW: f64 = 0.45;
const WIDE: f64 = 0.25;

/// What a non-ASCII character adds, as a fraction of the average advance.
///
/// **A glyph is not an average character.** A modifier glyph (`⌘`, `⇧`, `⌥`, `⌃`) is
/// drawn at about **1.25 em** — measured off the rendered sheet, and the same order as
/// the em-width a symbol face reserves. [`ADVANCE`] is 0.508 em, so the correction that
/// reaches 1.25 em is `0.508 × (1 + SYMBOL) = 1.25`, which is **1.45**.
///
/// Getting this wrong in the *under* direction is what put a shortcut's last character
/// past the edge it was aligned to, under the page's scroll bar. Under is the direction
/// that collides; over is a few points of extra air. The first version of this
/// correction used 1.0 — double the average, which still came out under — and the
/// Shortcuts page showed `⇧⌘S` with its `S` on the scroll bar.
const SYMBOL: f64 = 1.45;

/// The ellipsis a clipped string ends with.
const ELLIPSIS: char = '…';

fn is_narrow(ch: char) -> bool {
    matches!(
        ch,
        'i' | 'l'
            | 'j'
            | 't'
            | 'f'
            | 'r'
            | '.'
            | ','
            | ':'
            | ';'
            | '!'
            | '|'
            | '\''
            | '('
            | ')'
            | '['
            | ']'
    )
}

fn is_wide(ch: char) -> bool {
    matches!(ch, 'm' | 'w' | 'M' | 'W' | '@' | '%' | '&')
}

/// Whether a character is a symbol, a glyph or an ideograph rather than a Latin
/// letter — the characters whose advance this module's average does not describe.
///
/// Everything outside ASCII, which covers the modifier glyphs a shortcut is written
/// with (`⌘⇧⌥⌃`), the box-drawing and arrow glyphs an icon face draws, and CJK. It is
/// a category rather than an exact table because the alternative is a table of every
/// codepoint the bundled faces carry.
fn is_symbol(ch: char) -> bool {
    !ch.is_ascii()
}

/// How wide `text` paints at `font_size`, in points.
///
/// `base_advance` is the font's own per-character advance at that size; the two
/// corrections are applied on top of it.
pub fn width(text: &str, font_size: f64) -> f64 {
    width_in(text, font_size, Face::Proportional)
}

/// How wide `text` paints, **measured** by laying it out.
///
/// `DrawText::layout` is public and returns the size the renderer will use, which is a *better* number than
/// anything this module can estimate — so a widget that needs geometry should call this and not `width`. The
/// estimator stays for **clipping**, where an error in either direction is invisible.
///
/// ## The unit, which cost a screenshot to find
///
/// `size_in_lpxs` is in Makepad's **layout pixels**, which are 96-dpi — while a widget's `draw_abs`
/// coordinates, and every number in this module, are in points. So the measurement is multiplied by [`DPI`] like
/// everything else here; without that, a measured label comes out **25% under** and the same clipping returns
/// by a different route. That is the third time this factor has been the answer, which is why it now has one
/// home and a doc comment naming the two ways it is met.
pub fn measured_width(draw: &DrawText, cx: &mut Cx, text: &str) -> f64 {
    let laid = draw.layout(cx, 0.0, 0.0, None, false, Align::default(), text);
    laid.size_in_lpxs.width as f64 * DPI
}

/// How wide `text` paints in `face` at `font_size`, in points.
///
/// The face matters: see [`Face`].
pub fn width_in(text: &str, font_size: f64, face: Face) -> f64 {
    if matches!(face, Face::Mono) {
        // One advance for every character, which is the definition of the face — and exact rather
        // than estimated, so a code block's pen lands where the glyph does.
        return text.chars().count() as f64 * font_size * MONO_ADVANCE;
    }
    let base = font_size * ADVANCE;
    let mut w = 0.0;
    for ch in text.chars() {
        w += base;
        if is_symbol(ch) {
            // A glyph is wider than a Latin character in **any** face, on top of the DPI factor that
            // `base` already carries.
            // Checked before the Latin corrections, because `—` and `…` are neither
            // narrow nor merely wide and used to be caught by the wide table.
            w += base * SYMBOL;
        } else if is_narrow(ch) {
            w -= base * NARROW;
        } else if is_wide(ch) {
            w += base * WIDE;
        }
    }
    w
}

/// `text` cut to `limit` points at `font_size`, ending in an ellipsis when it had
/// to be cut.
///
/// The result is never wider than `limit`, and an empty string when nothing fits
/// at all — a bare ellipsis in a column of zero width reads as content.
pub fn clip(text: &str, limit: f64, font_size: f64) -> String {
    if limit <= 0.0 {
        return String::new();
    }
    if width(text, font_size) <= limit {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    // One character of the budget goes to the ellipsis, so the answer is never
    // wider than what was asked for. Walking down is linear and these rows have
    // tens of characters, not thousands.
    for take in (1..chars.len()).rev() {
        let mut candidate: String = chars[..take].iter().collect();
        candidate.push(ELLIPSIS);
        if width(&candidate, font_size) <= limit {
            return candidate;
        }
    }
    String::new()
}

/// Where a right-aligned string starts, given the box it is aligned in.
///
/// One function because two widgets right-align and a column of numbers that
/// lands two points further left than the table's is the kind of thing nobody
/// reports and everybody sees.
pub fn right_aligned_x(text: &str, box_x: f64, box_w: f64, pad: f64, font_size: f64) -> f64 {
    let w = width(text, font_size);
    (box_x + box_w - pad - w).max(box_x + pad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_estimate_carries_the_dpi_factor_that_makepad_lays_text_out_at() {
        // **The constant that was missing.** Makepad converts a `font_size` from points to pixels at
        // 96 dpi, so every estimate here was about 25% under the paint until this was applied. The
        // test pins the ratio to its derivation rather than to a remembered number, and pins the
        // measurement that identified it: `MpCodeBlock` drew 16 characters at 12pt and the rect came
        // back 153.62 wide — 9.6016pt per character — against a mono face whose own `hmtx` advance is
        // 0.6 em.
        assert!((DPI - 4.0 / 3.0).abs() < 1e-9, "96/72 is 4/3");
        let measured_per_char = 153.62 / 16.0;
        let estimated_per_char = width_in("m", 12.0, Face::Mono);
        assert!(
            (measured_per_char - estimated_per_char).abs() < 0.01,
            "the measurement is {measured_per_char:.4} per character and the estimate {estimated_per_char:.4},              so the DPI factor is wrong"
        );
        // And the mono advance is the face's own 0.6 em scaled by it, not a number chosen here.
        assert!((MONO_ADVANCE - 0.6 * 4.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_a_glyph_is_wider_than_any_latin_letter() {
        // The correction that fixed the Shortcuts page. A modifier glyph was counted at
        // an average Latin advance, which is roughly half its real width, so a
        // right-aligned chord overshot the panel it was aligned to.
        let font = 11.0;
        let glyph = width("\u{2318}", font); // ⌘
        assert!(glyph > width("m", font), "a glyph should beat a wide letter");
        assert!(glyph > width("W", font));
        assert!(glyph > width("M", font));
        // And a real chord is wider than its three Latin characters would be.
        assert!(
            width("\u{21e7}\u{2318}S", font) > width("XXX", font),
            "a chord is wider than three capitals"
        );
    }

    #[test]
    fn test_all_non_ascii_is_treated_as_wide_rather_than_merely_a_list_of_glyphs() {
        // A category rather than a table: the modifier glyphs, a CJK ideograph, an
        // arrow and an em dash all take the symbol correction.
        for ch in ['\u{2318}', '\u{21e7}', '\u{4e2d}', '\u{2192}', '\u{2014}'] {
            assert!(is_symbol(ch), "{ch:?} should be a symbol");
        }
        for ch in ['a', 'Z', '9', '-', ' ', '@'] {
            assert!(!is_symbol(ch), "{ch:?} is ASCII and should not be a symbol");
        }
    }

    #[test]
    fn test_the_width_grows_with_the_string() {
        let one = width("a", 13.0);
        assert!(width("aa", 13.0) > one);
        assert!(width("aaa", 13.0) > width("aa", 13.0));
        assert!((width("", 13.0) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_the_width_scales_with_the_font_size() {
        // A row that fits at 13pt must not overflow at 26pt.
        let small = width("a name", 13.0);
        let large = width("a name", 26.0);
        assert!((large / small - 2.0).abs() < 1e-9, "{small} {large}");
    }

    #[test]
    fn test_narrow_and_wide_strings_are_estimated_apart() {
        // The correction has to be two-sided, or a row of `illll` clips short
        // while a row of `WWWW` overflows.
        let narrow = width("illll", 13.0);
        let plain = width("aaaaa", 13.0);
        let wide = width("WWWWW", 13.0);
        assert!(narrow < plain, "{narrow} vs {plain}");
        assert!(wide > plain, "{wide} vs {plain}");
    }

    #[test]
    fn test_all_three_estimates_are_the_same_length_string() {
        // Guards the test above from passing because the strings differ in length.
        assert_eq!("illll".chars().count(), "aaaaa".chars().count());
        assert_eq!("WWWWW".chars().count(), "aaaaa".chars().count());
    }

    #[test]
    fn test_a_string_that_fits_is_returned_untouched() {
        assert_eq!(clip("tree.rs", 400.0, 13.0), "tree.rs");
    }

    #[test]
    fn test_a_clipped_string_ends_in_an_ellipsis_and_fits() {
        let long = "a-very-long-file-name-that-cannot-fit.rs";
        let cut = clip(long, 80.0, 13.0);
        assert!(cut.ends_with(ELLIPSIS), "{cut}");
        assert!(width(&cut, 13.0) <= 80.0, "{}", width(&cut, 13.0));
        assert!(cut.chars().count() < long.chars().count());
    }

    #[test]
    fn test_no_room_at_all_is_an_empty_string_not_a_bare_ellipsis() {
        // An ellipsis in a zero-width column reads as content rather than as a
        // truncation.
        assert_eq!(clip("anything", 0.0, 13.0), "");
        assert_eq!(clip("anything", -5.0, 13.0), "");
        assert_eq!(clip("anything", 2.0, 13.0), "");
    }

    #[test]
    fn test_a_clip_is_stable_under_being_clipped_again() {
        // Clipping an already-clipped string must not keep shortening it: a row
        // that is redrawn at the same width would lose a character per frame.
        let once = clip("a-very-long-name-indeed.rs", 90.0, 13.0);
        let twice = clip(&once, 90.0, 13.0);
        assert_eq!(once, twice);
    }

    #[test]
    fn test_right_alignment_puts_the_text_against_the_far_edge() {
        let font = 13.0;
        let text = "1234";
        let x = right_aligned_x(text, 100.0, 200.0, 10.0, font);
        // The text's right edge lands one pad inside the box's right edge.
        let right = x + width(text, font);
        assert!((right - (100.0 + 200.0 - 10.0)).abs() < 1e-9, "{right}");
    }

    #[test]
    fn test_right_alignment_does_not_push_text_out_of_a_narrow_box() {
        // A string wider than the box would put its start before the box's own
        // edge, and the clip that follows would be cutting from outside.
        let x = right_aligned_x("a very wide string", 0.0, 40.0, 10.0, 13.0);
        assert!(x >= 10.0, "{x}");
    }
}
