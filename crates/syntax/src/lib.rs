//! `makepad-syntax` — classifying source into the theme's highlight kinds.
//!
//! ## One language per function, and a router over them
//!
//! [`classify`] takes a source string and a **fence tag** — `json`, `rs`, `toml` — and returns
//! [`Highlight`] spans, or `None` when the tag names no language this build carries. That is the
//! shape bezel's `syntax` crate has, and the reason it is a shape rather than a detail: it puts the
//! *set of languages* behind one call, so a caller renders plain text for an unknown tag instead of
//! choosing what to do per language.
//!
//! The spans come back **normalized** — sorted, clipped to the source, and non-overlapping — because
//! [`makepad_theme::syntax::normalize`] is the contract every classifier here goes through. A
//! caller can therefore paint them in order without checking anything, which is what the contract
//! is for.
//!
//! ## Why the classifiers are hand-written, and why that is a decision rather than a shortcut
//!
//! bezel runs **tree-sitter** with a grammar per language. That is the better answer for a general
//! editor and it is *not* the answer taken here, for a reason worth stating rather than leaving as
//! an absence:
//!
//! - Adding tree-sitter to this workspace adds the runtime **and** a grammar crate per language.
//!   Cargo features union across the graph, so a grammar any dependency turns on is one no consumer
//!   can turn back off — bezel's own note about its `blocks` crate.
//! - This port's plan already lists `makepad-code-editor` for reuse, and reading it showed its
//!   highlighting is **a hand-written tokenizer too**, not tree-sitter. Taking that as the reference
//!   means the port and the thing it is porting alongside agree about the approach.
//! - A tree-sitter backend satisfies the *same* `(byte range, kind)` contract, so it can replace
//!   these functions one language at a time without a caller changing. That is exactly why
//!   [`normalize`] is a contract rather than a convention.
//!
//! ## JSON first, because it is the one language with no ambiguity
//!
//! A JSON tokenizer is **total**: the grammar has no context-sensitivity, no keywords that are also
//! identifiers, and no nesting that changes what a token means. So it is the language where a wrong
//! answer is a *bug in this file* rather than a judgement call, which makes it the right place to
//! establish the contract — and it is what this workspace's A2UI demo exchanges, so it has a reader
//! before any other language does.
//!
//! The traps a JSON tokenizer has, all of which are tested: an escaped quote inside a string, a
//! multi-byte character (offsets are **bytes**), a key that looks like a value and a value that
//! looks like a key, an **unterminated** string (which is what an editor holds most of the time
//! while someone is typing), and a stray byte that is not JSON at all.

use std::ops::Range;

use makepad_theme::syntax::{Highlight, HighlightKind, normalize};

/// The fence tags this build answers to.
pub fn languages() -> &'static [&'static str] {
    &["json"]
}

/// Classify `source` as `language` (a fence tag), or `None` when the tag is not one of
/// [`languages`].
///
/// The spans are normalized: sorted into document order, clipped to the source, and non-overlapping.
pub fn classify(source: &str, language: &str) -> Option<Vec<Highlight>> {
    match language.trim().to_ascii_lowercase().as_str() {
        "json" => Some(classify_json(source)),
        // `None` for an unknown tag, a tag no block claims, and a language turned off at compile
        // time — one answer for all three, which is the point of the shape.
        _ => None,
    }
}

/// JSON, as spans.
///
/// Whitespace is **not** emitted. bezel's doc states the rule for the whole crate: *"everything
/// outside those spans is unhighlighted text"* — so a gap is the absence of a span and not a span of
/// its own, and a caller paints the background through it.
pub fn classify_json(source: &str) -> Vec<Highlight> {
    let bytes = source.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'"' => {
                let (end, terminated) = scan_string(bytes, i);
                // A key is a string followed by `:` across any whitespace.
                let mut peek = end;
                while peek < bytes.len() && bytes[peek].is_ascii_whitespace() {
                    peek += 1;
                }
                let kind = if terminated && peek < bytes.len() && bytes[peek] == b':' {
                    HighlightKind::Attribute
                } else {
                    HighlightKind::String
                };
                spans.push(Highlight {
                    range: i..end,
                    kind,
                });
                i = end;
            }
            b'-' | b'0'..=b'9' => {
                let (end, is_number) = scan_number(bytes, i);
                spans.push(Highlight {
                    range: i..end,
                    // A lone `-` is not a number: reporting it as one colours a typo as valid, which
                    // is the worst thing a highlighter can do. The scan says whether it found one.
                    kind: if is_number {
                        HighlightKind::Number
                    } else {
                        HighlightKind::Invalid
                    },
                });
                i = end;
            }
            b if b.is_ascii_alphabetic() => {
                // Any bare word, not just the three JSON has. `true`, `false` and `null` are the only
                // **valid** ones — and a misspelled `True` or `nul` comes back as **one** span
                // rather than as several single-byte ones, because what the reader needs to see is
                // the word, and a parser reports it invalid either way.
                let end = scan_literal(bytes, i);
                let word = &bytes[i..end];
                let kind = if matches!(word, b"true" | b"false" | b"null") {
                    HighlightKind::Constant
                } else {
                    HighlightKind::Invalid
                };
                spans.push(Highlight { range: i..end, kind });
                i = end;
            }
            b'{' | b'}' | b'[' | b']' | b',' | b':' => {
                spans.push(Highlight {
                    range: i..i + 1,
                    kind: HighlightKind::Punctuation,
                });
                i += 1;
            }
            b if b.is_ascii_whitespace() => i += 1,
            _ => {
                // A byte that is not JSON. One byte at a time rather than a run, so the span says
                // exactly where the reader's cursor has to go.
                spans.push(Highlight {
                    range: i..i + 1,
                    kind: HighlightKind::Invalid,
                });
                i += 1;
            }
        }
    }
    // Through the contract, so a caller's paint order is guaranteed rather than assumed — even
    // though this scanner emits in document order and cannot overlap by construction. The point is
    // that *every* classifier here goes through it, so a future one that descends a tree cannot
    // skip it and look correct.
    normalize(source.len(), &spans)
}

/// The end of the string that starts at `at` (a `"`), and whether it was closed.
///
/// An **unterminated** string runs to the end of the source and is reported as such: that is what an
/// editor holds most of the time while someone is typing a value, and a tokenizer that dropped it
/// would make the line the reader is on the only unhighlighted one.
fn scan_string(bytes: &[u8], at: usize) -> (usize, bool) {
    let mut i = at + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                // Skip the escape **and whatever it escapes**, so `\"` does not end the string and a
                // trailing `\` does not index past the end.
                i += 2;
            }
            b'"' => return (i + 1, true),
            _ => i += 1,
        }
    }
    (bytes.len().min(i), false)
}

/// The end of the number starting at `at`.
///
/// Deliberately permissive — a sign, digits, one dot, digits, an exponent — because JSON's exact
/// numeric grammar is the parser's business and a tokenizer that refused `01` or `1.` would leave
/// the reader's typo unhighlighted rather than marking it. What it must get right is where the
/// number **stops**: at anything that is not one of those characters.
fn scan_number(bytes: &[u8], at: usize) -> (usize, bool) {
    let mut i = at;
    if i < bytes.len() && bytes[i] == b'-' {
        i += 1;
    }
    let mut seen_dot = false;
    let mut seen_exp = false;
    while i < bytes.len() {
        match bytes[i] {
            b'0'..=b'9' => i += 1,
            b'.' if !seen_dot && !seen_exp => {
                seen_dot = true;
                i += 1;
            }
            b'e' | b'E' if !seen_exp => {
                seen_exp = true;
                i += 1;
                if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                    i += 1;
                }
            }
            _ => break,
        }
    }
    // A lone `-`, or a `-` followed by something that is not a digit, is not a number. The flag is
    // returned rather than decided here so the caller owns the kind.
    let digits = bytes[at.min(bytes.len())..i]
        .iter()
        .any(|b| b.is_ascii_digit());
    if digits {
        (i, true)
    } else {
        (at + 1, false)
    }
}

/// The end of the bare word starting at `at`.
///
/// Any run of ASCII letters, not just the three JSON has, so that a misspelled `nul` or `True` comes
/// back as one `Constant` span the reader can see is wrong rather than as several single-byte
/// `Invalid` ones.
fn scan_literal(bytes: &[u8], at: usize) -> usize {
    let mut i = at;
    while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
        i += 1;
    }
    i.max(at + 1)
}

/// One highlighted run, for a caller that wants the text as well as where it is.
pub fn spans_with_text<'a>(
    source: &'a str,
    spans: &[Highlight],
) -> Vec<(Range<usize>, HighlightKind, &'a str)> {
    spans
        .iter()
        .filter_map(|span| {
            source
                .get(span.range.clone())
                .map(|text| (span.range.clone(), span.kind, text))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kind a span covering `needle` has, for a test that reads as the document does.
    fn kind_of(source: &str, needle: &str) -> HighlightKind {
        let spans = classify_json(source);
        let at = source.find(needle).expect("the needle is in the source");
        spans
            .iter()
            .find(|s| s.range.start <= at && at < s.range.end)
            .unwrap_or_else(|| panic!("no span covers {needle:?}: {spans:?}"))
            .kind
    }

    #[test]
    fn test_a_key_is_not_a_string_and_a_value_is_not_a_key() {
        // The distinction the whole language turns on, and the one a naive scanner gets wrong by
        // colouring every quoted run the same.
        let source = r#"{"name": "value"}"#;
        assert_eq!(kind_of(source, "\"name\""), HighlightKind::Attribute);
        assert_eq!(kind_of(source, "\"value\""), HighlightKind::String);
    }

    #[test]
    fn test_whitespace_between_a_key_and_its_colon_does_not_break_the_key() {
        // `"a" : 1` is a key. A scanner that looked at the very next byte would colour it a string,
        // and the fault would only show in documents formatted with a space before the colon.
        for source in [r#"{"a" : 1}"#, "{\n  \"a\"\n  :\n  1\n}", r#"{"a":1}"#] {
            assert_eq!(
                kind_of(source, "\"a\""),
                HighlightKind::Attribute,
                "{source:?}"
            );
        }
    }

    #[test]
    fn test_a_string_containing_an_escaped_quote_does_not_end_early() {
        // The classic: `"a\"b"` is one string of four characters plus escapes. A scanner that ended
        // at the first `"` would colour the rest of the document as if it were code.
        let source = r#"{"k": "a\"b", "j": 2}"#;
        let spans = classify_json(source);
        // The whole escaped string is one String span.
        let at = source.find("\"a").unwrap();
        let span = spans.iter().find(|s| s.range.start == at).expect("a span");
        assert_eq!(span.kind, HighlightKind::String);
        assert_eq!(
            &source[span.range.clone()],
            r#""a\"b""#,
            "the span does not cover the whole escaped string"
        );
        // ...and the key after it is still recognised, so the escape did not swallow the document.
        assert_eq!(kind_of(source, "\"j\""), HighlightKind::Attribute);
    }

    #[test]
    fn test_a_trailing_backslash_does_not_index_past_the_end() {
        // An editor holds a half-typed escape constantly. `i += 2` on the last byte of the source is
        // a panic in a slicing renderer, so the scan has to tolerate it.
        for source in [r#""a\"#, r#""\"#, r#""a\\"#] {
            let spans = classify_json(source);
            assert!(
                spans.iter().all(|s| s.range.end <= source.len()),
                "{source:?} produced a span past the end: {spans:?}"
            );
        }
    }

    #[test]
    fn test_an_unterminated_string_runs_to_the_end_and_is_the_line_being_typed() {
        // An editor holds this most of the time. Emitting nothing would make the reader's current
        // line the only unhighlighted one on screen.
        let source = "{\"key\": \"half";
        let spans = classify_json(source);
        let last = spans.last().expect("spans");
        assert_eq!(last.kind, HighlightKind::String);
        assert_eq!(last.range.end, source.len());
        // And it is **not** a key, even though nothing follows it: a string with no closing quote
        // cannot be a key, and colouring it as one would claim the document is well formed.
        assert_eq!(kind_of(source, "\"half"), HighlightKind::String);
    }

    #[test]
    fn test_offsets_are_bytes_not_characters() {
        // A multi-byte string is where a character-indexed scanner produces a span that slices a
        // character in half. Checked by asserting the spans **slice the source** — the operation a
        // renderer performs, which panics on a non-boundary.
        let source = r#"{"greeting": "héllo — 世界 🌍", "n": 1}"#;
        let spans = classify_json(source);
        let mut sliced = 0;
        for (range, _, _) in spans_with_text(source, &spans) {
            assert!(
                source.is_char_boundary(range.start) && source.is_char_boundary(range.end),
                "{range:?} is not on a character boundary"
            );
            sliced += 1;
        }
        assert!(sliced >= 4, "only {sliced} spans were sliced");
        // The whole greeting is one String span, whatever its length in bytes.
        let at = source.find("\"héllo").unwrap();
        let span = spans.iter().find(|s| s.range.start == at).expect("a span");
        assert_eq!(&source[span.range.clone()], "\"héllo — 世界 🌍\"");
    }

    #[test]
    fn test_the_three_bare_words_are_constants() {
        let source = r#"[true, false, null]"#;
        assert_eq!(kind_of(source, "true"), HighlightKind::Constant);
        assert_eq!(kind_of(source, "false"), HighlightKind::Constant);
        assert_eq!(kind_of(source, "null"), HighlightKind::Constant);
    }

    #[test]
    fn test_a_misspelled_bare_word_is_one_invalid_span_naming_the_word() {
        // `True` is not JSON, so `Invalid` is the right kind — and the reader needs to see **which
        // word**, which means one span rather than four single bytes. The first version of this test
        // asserted `Constant`, reasoning that a word should be shown as a word; that conflated "one
        // span" with "valid", and the code was right to mark it invalid.
        let source = r#"{"a": True}"#;
        let spans = classify_json(source);
        let at = source.find("True").unwrap();
        let covering: Vec<&Highlight> = spans
            .iter()
            .filter(|s| s.range.start >= at && s.range.end <= at + 4)
            .collect();
        assert_eq!(covering.len(), 1, "the word was shredded: {covering:?}");
        assert_eq!(&source[covering[0].range.clone()], "True");
        assert_eq!(covering[0].kind, HighlightKind::Invalid);
        // And the three real literals are still constants, so the branch did not simply go wrong.
        for (source, word) in [("[true]", "true"), ("[false]", "false"), ("[null]", "null")] {
            assert_eq!(kind_of(source, word), HighlightKind::Constant, "{word}");
        }
    }

    #[test]
    fn test_numbers_are_numbers_including_the_awkward_ones() {
        for (source, expected) in [
            ("[1]", "1"),
            ("[-1]", "-1"),
            ("[1.5]", "1.5"),
            ("[-1.5e10]", "-1.5e10"),
            ("[1E+2]", "1E+2"),
            ("[0]", "0"),
        ] {
            assert_eq!(kind_of(source, expected), HighlightKind::Number, "{source}");
        }
    }

    #[test]
    fn test_a_number_stops_at_what_is_not_a_number() {
        let source = r#"[1, 2]"#;
        let spans = classify_json(source);
        let one = spans
            .iter()
            .find(|s| s.kind == HighlightKind::Number)
            .expect("a number");
        assert_eq!(&source[one.range.clone()], "1", "the number ate its comma");
    }

    #[test]
    fn test_a_lone_minus_is_marked_rather_than_coloured_as_a_number() {
        // `[-]` is not JSON. Reporting the `-` as a Number would colour a typo as valid, which is the
        // worst thing a highlighter can do.
        let source = "[-]";
        let spans = classify_json(source);
        let dash = spans
            .iter()
            .find(|s| s.range.start == 1)
            .expect("the dash is spanned");
        assert_ne!(dash.kind, HighlightKind::Number);
    }

    #[test]
    fn test_punctuation_is_structural_and_nothing_else() {
        let source = r#"{"a": [1, 2]}"#;
        let spans = classify_json(source);
        let punctuation: String = spans
            .iter()
            .filter(|s| s.kind == HighlightKind::Punctuation)
            .map(|s| &source[s.range.clone()])
            .collect();
        // `{`, `:`, `[`, `,`, `]`, `}` — the structural bytes, in document order, and nothing else.
        assert_eq!(punctuation, "{:[,]}");
    }

    #[test]
    fn test_a_byte_that_is_not_json_is_marked_one_at_a_time() {
        let source = r#"{"a": @}"#;
        let spans = classify_json(source);
        let invalid: Vec<&Highlight> = spans
            .iter()
            .filter(|s| s.kind == HighlightKind::Invalid)
            .collect();
        assert_eq!(invalid.len(), 1, "{spans:?}");
        assert_eq!(&source[invalid[0].range.clone()], "@");
    }

    #[test]
    fn test_nested_documents_classify_at_every_depth() {
        let source = r#"{"a": {"b": [{"c": "d"}, 1, true]}}"#;
        assert_eq!(kind_of(source, "\"a\""), HighlightKind::Attribute);
        assert_eq!(kind_of(source, "\"b\""), HighlightKind::Attribute);
        assert_eq!(kind_of(source, "\"c\""), HighlightKind::Attribute);
        assert_eq!(kind_of(source, "\"d\""), HighlightKind::String);
        // The same three keys at three depths, and none of them was mistaken for a value.
        let spans = classify_json(source);
        assert_eq!(
            spans
                .iter()
                .filter(|s| s.kind == HighlightKind::Attribute)
                .count(),
            3
        );
    }

    // ---- the contract ------------------------------------------------------

    #[test]
    fn test_every_result_satisfies_the_contract() {
        // The property the theme's `normalize` exists to guarantee, asserted over documents built to
        // stress it: sorted, in bounds, non-overlapping, and never empty.
        let documents = [
            "{}",
            "[]",
            "null",
            "  ",
            "",
            r#"{"a":1}"#,
            r#"{"a": {"b": {"c": [1, 2, {"d": "e"}]}}}"#,
            r#"{"k": "a\"b", "j": @}"#,
            r#"{"héllo": "世界 🌍", "n": -1.5e10}"#,
            "{\"unter: minated",
            "[\n  1,\n  2,\n]",
            "@@@",
        ];
        for source in documents {
            let spans = classify_json(source);
            let mut last_end = 0usize;
            for span in &spans {
                assert!(
                    span.range.start < span.range.end,
                    "{source:?}: empty or reversed span {span:?}"
                );
                assert!(
                    span.range.end <= source.len(),
                    "{source:?}: span past the end {span:?}"
                );
                assert!(
                    span.range.start >= last_end,
                    "{source:?}: overlaps or is out of order at {span:?}"
                );
                assert!(
                    source.is_char_boundary(span.range.start)
                        && source.is_char_boundary(span.range.end),
                    "{source:?}: {span:?} is not on a character boundary"
                );
                last_end = span.range.end;
            }
        }
    }

    #[test]
    fn test_whitespace_is_a_gap_rather_than_a_span_of_its_own() {
        // The rule from the theme's own doc, checked here because this is the first classifier to
        // obey it: "everything outside those spans is unhighlighted text".
        let source = "{\n  \"a\" : 1\n}";
        let spans = classify_json(source);
        let covered: usize = spans.iter().map(|s| s.range.len()).sum();
        assert!(
            covered < source.len(),
            "every byte is covered, so whitespace was emitted as spans"
        );
        for span in &spans {
            assert!(
                !source[span.range.clone()].trim().is_empty(),
                "a span is whitespace: {span:?}"
            );
        }
    }

    #[test]
    fn test_the_router_answers_by_fence_tag_and_nothing_for_an_unknown_one() {
        // One answer for an unknown tag, an unclaimed tag, and a language not compiled in — which is
        // the point of the router shape.
        assert!(classify(r#"{"a":1}"#, "json").is_some());
        assert!(classify(r#"{"a":1}"#, "JSON").is_some(), "tags are case-insensitive");
        assert!(classify(r#"{"a":1}"#, "  json  ").is_some(), "and trimmed");
        assert!(classify("fn main() {}", "rs").is_none());
        assert!(classify("", "").is_none());
        assert!(classify("", "mermaid").is_none());
    }

    #[test]
    fn test_the_language_list_is_what_the_router_answers_to() {
        // A picker offers `languages()`, so a language the router handles and the list omits is a
        // language nobody can choose — and one the list offers and the router refuses is a choice
        // that does nothing.
        for tag in languages() {
            assert!(
                classify("{}", tag).is_some(),
                "{tag} is offered but not handled"
            );
        }
        assert!(classify("{}", "rs").is_none());
        assert_eq!(languages(), &["json"]);
    }
}
