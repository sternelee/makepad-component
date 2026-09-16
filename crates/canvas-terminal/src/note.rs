//! Note-card body formatting.
//!
//! Borrowed from makepad's `apps/notes` (MIT OR Apache-2.0): the block split is
//! `engine::preview_blocks`, editing a checklist in place is
//! `engine::toggle_checkbox`, and the "edited …" stamp is
//! `engine::format_note_date`. Notes renders markdown prose through the
//! `Markdown` widget and splits checklists out into real tappable rows — the
//! widget's own `TaskListMarker` arm is still a TODO — so checkboxes are handled
//! by hand on both sides.
//!
//! Everything here is pure: parsing, source rewriting and date arithmetic. The
//! measured layout and the drawing live in `canvas.rs`, because widths come from
//! the text stack.

/// Inline emphasis, parsed out of `**bold**`, `*italic*` / `_italic_` and
/// `` `code` `` runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineStyle {
    Plain,
    Bold,
    Italic,
    Code,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: InlineStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// `# `..`#### `; the payload is the level (1..=4).
    Heading(u8),
    Paragraph,
    /// `- ` / `* ` / `+ `.
    Bullet,
    /// `1. ` and friends; the payload is the number to print.
    Numbered(usize),
    /// `- [ ] ` / `- [x] `.
    Check { checked: bool },
    /// `> `.
    Quote,
    /// `---` / `***` / `___`.
    Rule,
    /// A line inside a ``` fence.
    Code,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    /// Inline runs of the line (empty for `Rule`).
    pub spans: Vec<Span>,
    /// Source line this block came from; the handle a checklist click rewrites.
    pub src_line: usize,
    /// Char offset of the block's first char in the source (newlines included),
    /// so the caret index can be mapped back onto a rendered row.
    pub char_start: usize,
}

/// Parse a note body into renderable blocks, one per source line.
///
/// Deliberately a subset: headings, bullets, numbers, checklists, quotes, rules
/// and fenced code — the shapes a sticky note on a canvas actually uses.
pub fn parse_blocks(source: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut char_start = 0usize;
    let mut fence = false;
    for (line_idx, raw) in source.split('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let trimmed = line.trim_start();
        let (kind, spans) = if fence {
            if trimmed.starts_with("```") {
                fence = false;
                (BlockKind::Paragraph, Vec::new())
            } else {
                (
                    BlockKind::Code,
                    vec![Span {
                        text: line.to_string(),
                        style: InlineStyle::Plain,
                    }],
                )
            }
        } else if trimmed.starts_with("```") {
            fence = true;
            (BlockKind::Paragraph, Vec::new())
        } else if let Some((level, rest)) = heading(trimmed) {
            (BlockKind::Heading(level), parse_inline(rest))
        } else if let Some((checked, rest)) = check_item(trimmed) {
            (BlockKind::Check { checked }, parse_inline(rest))
        } else if let Some(rest) = trim_marker(trimmed, &['-', '*', '+']) {
            (BlockKind::Bullet, parse_inline(rest))
        } else if let Some((number, rest)) = numbered_item(trimmed) {
            (BlockKind::Numbered(number), parse_inline(rest))
        } else if let Some(rest) = trim_marker(trimmed, &['>']) {
            (BlockKind::Quote, parse_inline(rest))
        } else if is_rule(trimmed) {
            (BlockKind::Rule, Vec::new())
        } else {
            (BlockKind::Paragraph, parse_inline(line))
        };
        blocks.push(Block {
            kind,
            spans,
            src_line: line_idx,
            char_start,
        });
        // +1 for the '\n' that `split` consumed.
        char_start += line.chars().count() + 1;
    }
    blocks
}

fn heading(line: &str) -> Option<(u8, &str)> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 4 {
        return None;
    }
    let rest = &line[hashes..];
    let rest = rest.strip_prefix(' ')?;
    Some((hashes as u8, rest))
}

fn check_item(line: &str) -> Option<(bool, &str)> {
    let rest = trim_marker(line, &['-', '*', '+'])?;
    for (mark, checked) in [("[ ] ", false), ("[x] ", true), ("[X] ", true)] {
        if let Some(label) = rest.strip_prefix(mark) {
            return Some((checked, label));
        }
    }
    None
}

/// Strip a bullet/quote marker: the char followed by one space.
fn trim_marker<'a>(line: &'a str, markers: &[char]) -> Option<&'a str> {
    let mut chars = line.chars();
    let first = chars.next()?;
    if !markers.contains(&first) {
        return None;
    }
    let rest = &line[first.len_utf8()..];
    rest.strip_prefix(' ')
}

fn numbered_item(line: &str) -> Option<(usize, &str)> {
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let rest = &line[digits..];
    let rest = rest.strip_prefix(". ")?;
    line[..digits].parse::<usize>().ok().map(|n| (n, rest))
}

fn is_rule(line: &str) -> bool {
    let t = line.trim_end();
    (t.len() >= 3) && (t.chars().all(|c| c == '-') || t.chars().all(|c| c == '*') || t.chars().all(|c| c == '_'))
}

/// Split one line into inline runs. An unmatched marker char is literal text, so
/// `2 * 3` stays arithmetic.
pub fn parse_inline(text: &str) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        let (marker, style) = if rest.starts_with("**") || rest.starts_with("__") {
            (2usize, InlineStyle::Bold)
        } else if rest.starts_with('`') {
            (1, InlineStyle::Code)
        } else if rest.starts_with('*') || rest.starts_with('_') {
            (1, InlineStyle::Italic)
        } else {
            (0, InlineStyle::Plain)
        };
        let mut advance = 0usize;
        if marker > 0 {
            let delim = &rest[..marker];
            if let Some(end) = rest[marker..].find(delim) {
                let inner = &rest[marker..marker + end];
                if !inner.is_empty() {
                    if !plain.is_empty() {
                        spans.push(Span {
                            text: std::mem::take(&mut plain),
                            style: InlineStyle::Plain,
                        });
                    }
                    spans.push(Span {
                        text: inner.to_string(),
                        style,
                    });
                }
                advance = marker + end + marker;
            }
        }
        if advance == 0 {
            let ch = rest.chars().next().unwrap_or_default();
            plain.push(ch);
            advance = ch.len_utf8();
        }
        rest = &rest[advance..];
    }
    if !plain.is_empty() || spans.is_empty() {
        spans.push(Span {
            text: plain,
            style: InlineStyle::Plain,
        });
    }
    spans
}

/// Flip a checklist item's box on source line `line_idx`, the way notes'
/// `engine::toggle_checkbox` rewrites its markdown source in place.
///
/// Returns `None` when that line is not a checklist item, so a click on prose
/// stays a click on prose.
pub fn toggle_checkbox(source: &str, line_idx: usize) -> Option<String> {
    let mut out = String::with_capacity(source.len());
    let mut toggled = false;
    for (i, line) in source.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if i != line_idx {
            out.push_str(line);
            continue;
        }
        let indent_len = line.len() - line.trim_start().len();
        let (indent, rest) = line.split_at(indent_len);
        let new = if let Some(label) = check_item(rest) {
            let mark = if label.0 { "- [ ] " } else { "- [x] " };
            toggled = true;
            format!("{indent}{mark}{}", label.1)
        } else {
            line.to_string()
        };
        out.push_str(&new);
    }
    toggled.then_some(out)
}

const DAY_MS: i64 = 86_400_000;
const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// "14:32" / "yesterday" / "12 Mar" / "12 Mar 2025" — notes' `format_note_date`,
/// lower-cased for an inline stamp.
pub fn format_edited(edited_ms: i64, now_ms: i64) -> String {
    if edited_ms <= 0 {
        return String::new();
    }
    let day = edited_ms.div_euclid(DAY_MS);
    let today = now_ms.div_euclid(DAY_MS);
    if day == today {
        let secs = edited_ms.div_euclid(1000);
        let h = (secs / 3600).rem_euclid(24);
        let m = (secs / 60).rem_euclid(60);
        format!("{h:02}:{m:02}")
    } else if day == today - 1 {
        "yesterday".to_string()
    } else {
        let (y, m, d) = ymd_from_days(day);
        let (ny, _, _) = ymd_from_days(today);
        let month = MONTH_ABBR[(m.saturating_sub(1) % 12) as usize];
        if y == ny {
            format!("{d} {month}")
        } else {
            format!("{d} {month} {y}")
        }
    }
}

/// Days since the epoch → (year, month, day). Howard Hinnant's `civil_from_days`.
fn ymd_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as i64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<BlockKind> {
        parse_blocks(source).iter().map(|b| b.kind).collect()
    }

    #[test]
    fn blocks_split_per_line() {
        assert_eq!(
            kinds("# Title\nplain\ntail"),
            vec![
                BlockKind::Heading(1),
                BlockKind::Paragraph,
                BlockKind::Paragraph
            ]
        );
    }

    #[test]
    fn lists_and_quotes() {
        assert_eq!(
            kinds("- one\n* two\n1. three\n> four\n---"),
            vec![
                BlockKind::Bullet,
                BlockKind::Bullet,
                BlockKind::Numbered(1),
                BlockKind::Quote,
                BlockKind::Rule,
            ]
        );
    }

    #[test]
    fn checklists_win_over_bullets() {
        let blocks = parse_blocks("- [ ] todo\n- [x] done");
        assert_eq!(
            blocks[0].kind,
            BlockKind::Check { checked: false }
        );
        assert_eq!(blocks[1].kind, BlockKind::Check { checked: true });
        // The label keeps the text after the box.
        assert_eq!(blocks[0].spans[0].text, "todo");
    }

    #[test]
    fn fenced_code_is_kept_literal() {
        let blocks = parse_blocks("```\n**not bold**\n```");
        assert_eq!(blocks[0].kind, BlockKind::Paragraph);
        assert_eq!(blocks[1].kind, BlockKind::Code);
        assert_eq!(blocks[1].spans[0].text, "**not bold**");
        assert_eq!(blocks[1].spans[0].style, InlineStyle::Plain);
    }

    #[test]
    fn inline_styles() {
        assert_eq!(
            parse_inline("a **b** c"),
            vec![
                Span { text: "a ".into(), style: InlineStyle::Plain },
                Span { text: "b".into(), style: InlineStyle::Bold },
                Span { text: " c".into(), style: InlineStyle::Plain },
            ]
        );
        assert_eq!(
            parse_inline("*i* and `c`"),
            vec![
                Span { text: "i".into(), style: InlineStyle::Italic },
                Span { text: " and ".into(), style: InlineStyle::Plain },
                Span { text: "c".into(), style: InlineStyle::Code },
            ]
        );
    }

    #[test]
    fn unmatched_markers_stay_literal() {
        assert_eq!(
            parse_inline("2 * 3"),
            vec![Span { text: "2 * 3".into(), style: InlineStyle::Plain }]
        );
    }

    #[test]
    fn char_offsets_track_newlines() {
        let blocks = parse_blocks("abc\ndef");
        assert_eq!(blocks[0].char_start, 0);
        assert_eq!(blocks[1].char_start, 4);
    }

    #[test]
    fn toggle_rewrites_only_a_checklist_line() {
        let src = "- [ ] one\nplain\n- [x] two";
        let out = toggle_checkbox(src, 0).expect("line 0 is a checklist");
        assert_eq!(out, "- [x] one\nplain\n- [x] two");
        let back = toggle_checkbox(&out, 0).unwrap();
        assert_eq!(back, src);
        // Toggling the other direction, and indented items.
        assert_eq!(
            toggle_checkbox("- [x] two", 0).unwrap(),
            "- [ ] two"
        );
        assert_eq!(
            toggle_checkbox("  - [ ] indented", 0).unwrap(),
            "  - [x] indented"
        );
    }

    #[test]
    fn toggle_declines_non_checklist_lines() {
        assert!(toggle_checkbox("plain text", 0).is_none());
        assert!(toggle_checkbox("- bullet", 0).is_none());
        assert!(toggle_checkbox("- [ ] one\n- bullet", 1).is_none());
    }

    #[test]
    fn edited_stamp_is_relative_then_absolute() {
        // 2026-03-12 15:32:00 UTC
        let now = 1_773_329_520_000;
        assert_eq!(format_edited(now - 60_000, now), "15:31");
        assert_eq!(format_edited(now - DAY_MS, now), "yesterday");
        assert_eq!(format_edited(now - 5 * DAY_MS, now), "7 Mar");
        assert_eq!(format_edited(now - 400 * DAY_MS, now), "5 Feb 2025");
        // A note that was never stamped shows nothing.
        assert_eq!(format_edited(0, now), "");
    }

    #[test]
    fn civil_dates_round_trip_known_epochs() {
        assert_eq!(ymd_from_days(0), (1970, 1, 1));
        assert_eq!(ymd_from_days(19_723), (2024, 1, 1));
    }
}
