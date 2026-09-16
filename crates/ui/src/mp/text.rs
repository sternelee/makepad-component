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

/// The advance of an average character, as a fraction of the font size, for the
/// faces this crate bundles.
const ADVANCE: f64 = 0.508;

/// How far a narrow or wide character moves the estimate, as a fraction of the
/// average advance.
const NARROW: f64 = 0.45;
const WIDE: f64 = 0.25;

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
    matches!(
        ch,
        'm' | 'w' | 'M' | 'W' | '@' | '%' | '&' | '—' | ELLIPSIS
    )
}

/// How wide `text` paints at `font_size`, in points.
///
/// `base_advance` is the font's own per-character advance at that size; the two
/// corrections are applied on top of it.
pub fn width(text: &str, font_size: f64) -> f64 {
    let base = font_size * ADVANCE;
    let mut w = 0.0;
    for ch in text.chars() {
        w += base;
        if is_narrow(ch) {
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
