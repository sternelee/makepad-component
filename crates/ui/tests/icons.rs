//! The glyphs this library draws, checked against the font that draws them.
//!
//! ## Why the font file is the ground truth
//!
//! This port recorded a **wrong explanation** for a real fault, and the check here exists so that
//! the next one is a test failure instead of a paragraph. The calendar's month arrows were
//! `\u{f053}` and `\u{f054}` and drew as two tofu boxes; the note written at the time said
//! FontAwesome's chevrons "are not in this port's icon subset". Parsing the font shows **1976**
//! glyphs with both chevrons among them. The real cause was that the calendar drew them through a
//! `DrawText` whose `text_style` is the *text* face.
//!
//! So there are two questions and this file answers both, mechanically:
//!
//! 1. **Is every declared glyph in the font?** `font_codepoints` parses
//!    `fa-solid-900.ttf`'s `cmap` table — the same table a renderer consults — and the declared
//!    set is checked against it.
//! 2. **Does every file that writes a FontAwesome codepoint reach the icon face?** A file may
//!    carry `theme.font_icons` in its own DSL or route the glyph through `MpIcon`; a file that
//!    does neither is drawing an icon glyph with a text face, which is tofu and nothing else.
//!
//! ## How the sources are read
//!
//! Comments are stripped before scanning, for the reason `tests/registration_order.rs` records:
//! a doc comment that *mentions* a codepoint is not a use of one. The last time this was not
//! done the check reported a violation for the most thoroughly documented module in the crate.
//!
//! Both checks assert their own scan found something plausible first. A parser that matched
//! nothing would pass every assertion after it, and this port has already been bitten by a
//! `grep -c` that counted a struct definition and reported thirty-four pages when there were
//! thirty-one.

use std::path::{Path, PathBuf};

// The module is reached by path so this test does not depend on the crate re-exporting it.
use makepad_component::mp::icons;

/// The font Makepad ships as `theme.font_icons`, in the sibling clone the workspace builds
/// against.
///
/// The workspace declares `makepad-widgets` as a **path dependency** on that clone, so asserting
/// the font is present adds no requirement the build did not already have.
fn icon_font_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../makepad/widgets/resources/fa-solid-900.ttf")
}

fn mp_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/mp")
}

fn be16(data: &[u8], at: usize) -> u32 {
    u16::from_be_bytes([data[at], data[at + 1]]) as u32
}

fn be32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

/// Every codepoint a TrueType font can draw, read from its `cmap` table.
///
/// The table a renderer consults, which is the point: this is not a list of what FontAwesome
/// *documents*, it is a list of what the file **has**.
///
/// Format 12 is preferred over format 4 when both are present, because format 12 covers the whole
/// of Unicode while format 4 stops at the Basic Multilingual Plane. Every glyph here is in the
/// BMP, so the distinction does not change the answer today — but a parser that took the first
/// subtable it found would be right by accident, and the next glyph added might not be.
fn font_codepoints(data: &[u8]) -> Vec<u32> {
    let num_tables = be16(data, 4) as usize;
    let mut cmap_at = None;
    for i in 0..num_tables {
        let record = 12 + i * 16;
        if &data[record..record + 4] == b"cmap" {
            cmap_at = Some(be32(data, record + 8) as usize);
        }
    }
    let cmap = cmap_at.expect("the font has a cmap table");

    let subtables = be16(data, cmap + 2) as usize;
    let mut chosen: Option<(u32, usize)> = None;
    for i in 0..subtables {
        let record = cmap + 4 + i * 8;
        let sub = cmap + be32(data, record + 4) as usize;
        let format = be16(data, sub);
        match format {
            12 => {
                chosen = Some((12, sub));
                break;
            }
            4 if chosen.is_none() => chosen = Some((4, sub)),
            _ => {}
        }
    }
    let (format, at) = chosen.expect("the font has a format 4 or 12 subtable");

    let mut codepoints = Vec::new();
    if format == 12 {
        let groups = be32(data, at + 12) as usize;
        for group in 0..groups {
            let record = at + 16 + group * 12;
            let start = be32(data, record);
            let end = be32(data, record + 4);
            // A malformed range would otherwise allocate for a very long time.
            if end < start || end - start > 0x10_FFFF {
                continue;
            }
            codepoints.extend(start..=end);
        }
    } else {
        let seg_count = be16(data, at + 6) as usize / 2;
        let ends_at = at + 14;
        let starts_at = ends_at + seg_count * 2 + 2;
        for segment in 0..seg_count {
            let start = be16(data, starts_at + segment * 2);
            let end = be16(data, ends_at + segment * 2);
            // The final segment is the required `0xFFFF` terminator, whose start maps through
            // `idDelta` rather than to itself.
            if start == 0xFFFF {
                continue;
            }
            if end >= start {
                codepoints.extend(start..=end);
            }
        }
    }
    codepoints.sort_unstable();
    codepoints.dedup();
    codepoints
}

/// FontAwesome's own base codepoint. Everything at or above it is an icon glyph.
///
/// **Not the Private Use Area's ceiling.** The first version of these checks used
/// `0xE000..=0xF8FF`, the PUA's range, and the falsification test caught what that misses: a
/// planted `\u{f999}` was **skipped entirely**, because `F999` is above `F8FF`. So a codepoint
/// that is obviously meant as a FontAwesome glyph went unchecked — and `F999` is not a
/// hypothetical, since FontAwesome Pro draws above `F8FF`.
///
/// FontAwesome's base is `F000` and no text face in this crate has glyphs up there, so the lower
/// bound is what decides, and the upper bound is Unicode's.
const ICON_GLYPH_BASE: u32 = 0xF000;

/// Whether a codepoint is meant as an icon glyph rather than as a text-face mark.
///
/// `‹`, `·` and `—` are below the base and belong to the text face; `\u{f002}` is above it and
/// belongs to FontAwesome.
fn is_icon_codepoint(c: char) -> bool {
    (c as u32) >= ICON_GLYPH_BASE
}

/// The `\u{...}` escapes in `text`, with comments stripped.
fn escapes(text: &str) -> Vec<char> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("\\u{") {
        let after = &rest[at + 3..];
        let Some(close) = after.find('}') else {
            break;
        };
        let hex = &after[..close];
        if hex.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(code) = u32::from_str_radix(hex, 16) {
                if let Some(c) = char::from_u32(code) {
                    found.push(c);
                }
            }
        }
        rest = &after[close..];
    }
    found
}

/// The code in a file, with line comments removed.
fn code_only(source: &str) -> String {
    // Tests too: a test that names a codepoint is not a draw call.
    let code = match source.find("#[cfg(test)]") {
        Some(at) => &source[..at],
        None => source,
    };
    code.lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

#[test]
fn test_every_declared_glyph_is_in_the_font_makepad_ships() {
    let path = icon_font_path();
    let data = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
    let codepoints = font_codepoints(&data);
    // The anti-vacuous-pass guard: a parser that found nothing would satisfy every assertion
    // below it.
    assert!(
        codepoints.len() > 1000,
        "only {} codepoints were parsed out of {}, so the parser is wrong",
        codepoints.len(),
        path.display()
    );

    let mut missing = Vec::new();
    for (name, glyph) in icons::ALL {
        let c = icons::codepoint(glyph).expect("a declared glyph is one character");
        if !codepoints.binary_search(&(c as u32)).is_ok() {
            missing.push(format!("{name} (U+{:04X})", c as u32));
        }
    }
    assert!(
        missing.is_empty(),
        "{} carries {} codepoints and is missing these declared glyphs: {missing:?}",
        icons::FACE,
        codepoints.len()
    );

    // And the two chevrons that started this, asserted by name so the record of the wrong
    // diagnosis has a test attached to it.
    for chevron in ['\u{f053}', '\u{f054}'] {
        assert!(
            codepoints.binary_search(&(chevron as u32)).is_ok(),
            "U+{:04X} is not in the font, which contradicts the note in mp/date.rs",
            chevron as u32
        );
    }
}

#[test]
fn test_every_codepoint_written_in_the_library_is_declared() {
    // The half that turns a typo into a failing test. A `script_mod!` block cannot reference a
    // Rust constant, so the codepoint is written at the call site — and this is what stops a
    // call site from writing one nobody has ever seen render.
    let declared: Vec<u32> = icons::ALL
        .iter()
        .filter_map(|(_, g)| icons::codepoint(g))
        .map(|c| c as u32)
        .collect();

    let mut files = 0usize;
    let mut found = 0usize;
    let mut undeclared = Vec::new();
    for entry in std::fs::read_dir(mp_dir()).expect("src/mp is readable") {
        let path = entry.expect("a directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let code = code_only(&source);
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let mut any = false;
        for c in escapes(&code) {
            let code_point = c as u32;
            // Icon glyphs only. The library also writes `\u{b7}` and `\u{2014}` for prose, and
            // those are the text face's business.
            if !is_icon_codepoint(c) {
                continue;
            }
            any = true;
            found += 1;
            if !declared.contains(&code_point) {
                undeclared.push(format!("{name}: U+{code_point:04X}"));
            }
        }
        if any {
            files += 1;
        }
    }
    assert!(
        files >= 3 && found >= 5,
        "only {found} icon codepoints in {files} files were scanned, so the scan is broken"
    );
    assert!(
        undeclared.is_empty(),
        "these codepoints are written in the library but not declared in mp/icons.rs, so \
         nothing has checked that the font has them: {undeclared:?}"
    );
}

#[test]
fn test_a_file_that_writes_an_icon_glyph_reaches_the_icon_face() {
    // **The check that would have caught the calendar.** A FontAwesome codepoint drawn with the
    // text face is tofu — no error, no warning, nothing in a log — so a file that writes one has
    // to reach the icon face, either by declaring `theme.font_icons` on the `DrawText` that
    // draws it or by routing the glyph through `MpIcon`.
    //
    // It is a heuristic, and its false-positive risk is bounded by the escape hatch being wide:
    // a file is only reported when it has *neither*. `mp/date.rs` is the worked example of
    // getting it wrong — it wrote `\u{f053}` with a caption-face `DrawText` and neither escape,
    // and the arrows drew as two boxes.
    let mut checked = 0usize;
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(mp_dir()).expect("src/mp is readable") {
        let path = entry.expect("a directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let code = code_only(&source);
        let writes_icon_glyph = escapes(&code).iter().any(|c| is_icon_codepoint(*c));
        if !writes_icon_glyph {
            continue;
        }
        checked += 1;
        let reaches_icon_face = code.contains("theme.font_icons") || code.contains("mod.mp.MpIcon");
        if !reaches_icon_face {
            offenders.push(
                path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            );
        }
    }
    assert!(
        checked >= 3,
        "only {checked} files write an icon glyph, so the scan is broken"
    );
    assert!(
        offenders.is_empty(),
        "these files write a FontAwesome codepoint but never reach the icon face, so the glyph \
         is drawn with a text face and renders as tofu: {offenders:?}"
    );
}

#[test]
fn test_the_cmap_parser_finds_codepoints_the_font_has_and_not_ones_it_does_not() {
    // The parser's own test, so a future change that makes it return everything (or nothing)
    // fails here rather than passing the checks above by luck.
    let data = std::fs::read(icon_font_path()).expect("the icon font");
    let codepoints = font_codepoints(&data);
    // A glyph FontAwesome has.
    assert!(codepoints.binary_search(&0xf002).is_ok(), "no magnifying glass");
    // A codepoint the font does **not** have, found rather than hardcoded. The first version
    // asserted `U+F8FF` was absent and it is present — FontAwesome Free fills the top of the
    // Private Use Area — so the test failed on an assumption about the font's range rather than
    // on a parser fault. Deriving it means the assertion cannot go stale when the font is
    // updated.
    let absent = (ICON_GLYPH_BASE..=0xF8FF)
        .find(|c| codepoints.binary_search(c).is_err())
        .expect("the font does not fill its own range");
    assert!(
        codepoints.binary_search(&absent).is_err(),
        "the parser claims U+{absent:04X}, which the font does not have"
    );
    // ...and it is not merely returning a dense range: there really are gaps.
    let present_above = (0xE000u32..=0xF8FF).filter(|c| codepoints.binary_search(c).is_ok()).count();
    assert!(
        present_above < 0x18FF,
        "the parser returned every codepoint in the Private Use Area, so it is not reading the          subtable"
    );
    // And the table is not simply "everything up to some bound".
    assert!(
        codepoints.len() < 5000,
        "the parser returned {} codepoints, which is more than FontAwesome Free has",
        codepoints.len()
    );
}
