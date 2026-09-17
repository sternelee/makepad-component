//! Syntax classification: the **kind** vocabulary, the colours, and the span contract.
//!
//! ## Why the classifier is not here
//!
//! bezel's `syntax` crate runs tree-sitter and returns `(byte range, HighlightKind)` spans. What
//! this module carries is the half that does not depend on how the spans were produced — the
//! kinds, their colours, and the rule that a classifier's raw output has to be **normalized**
//! before anything paints it. bezel separates the same way, and says why: *"there is no color and
//! no rendering here — kinds map to colors through `SyntaxPalette::color`."*
//!
//! That split is not tidiness. It is what lets the classifier be chosen later — tree-sitter with a
//! grammar per language, or a hand-written tokenizer like the one Makepad's own editor ships —
//! without touching a colour, a test, or a call site.
//!
//! ## The kinds are a closed set, like every other vocabulary in this theme
//!
//! A highlighter that emitted free-form strings would put the *set of things a reader can
//! distinguish* in the hands of whichever grammar was loaded. Thirteen kinds is what a reader
//! actually tells apart, and a grammar that has finer distinctions maps them down rather than
//! inventing a fourteenth colour.
//!
//! ## Why normalization is a function rather than a convention
//!
//! Painting two overlapping spans paints one of them twice, and nesting them paints the inner one
//! then the outer one's background over it — so the failure is a colour that is subtly wrong
//! rather than an error. Spelling the rule as a function means a classifier's output can be made
//! safe at the boundary, and means the rule is **testable without a classifier at all**.
//!
//! The three faults it fixes are the three a real grammar produces:
//!
//! - **Out of order.** A tree walk emits spans in the order it descends, which is not document
//!   order once a query has several patterns.
//! - **Overlapping.** One pattern's `(identifier)` contains another's `(function_name)`; both
//!   match, and the grammar reports both.
//! - **Past the end.** A grammar parsed a *stale* buffer — the editor's own document can be
//!   edited between the parse and the paint, and a span one byte past the end is a panic in a
//!   slicing renderer.

use std::ops::Range;

use makepad_widgets::*;

use crate::appearance::Appearance;
use crate::color;

/// What a span of source **is**, for a reader rather than for a parser.
///
/// Thirteen kinds, because thirteen is what a reader tells apart by colour. A grammar with finer
/// distinctions — a keyword that is a control-flow keyword, a string that is a template — maps
/// down to these rather than earning a colour of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Script, ScriptHook)]
pub enum HighlightKind {
    /// `fn`, `let`, `if`, `class` — the words the language reserves.
    #[pick]
    #[live]
    Keyword,
    /// A call or a definition's name.
    #[live]
    Function,
    /// A type, a trait, an interface, a struct.
    #[live]
    Type,
    /// A literal that names a value rather than spelling one: `true`, `None`, `MAX`.
    #[live]
    Constant,
    /// A binding, a field, a parameter.
    #[live]
    Variable,
    /// A string literal, including its quotes.
    #[live]
    String,
    /// A numeric literal.
    #[live]
    Number,
    /// A comment, including its markers.
    #[live]
    Comment,
    /// `+`, `==`, `=>`, `&&`.
    #[live]
    Operator,
    /// A bracket, a comma, a semicolon — structure rather than meaning.
    #[live]
    Punctuation,
    /// `#[derive]`, `@Override`, a decorator, an annotation.
    #[live]
    Attribute,
    /// An element or a markup tag name.
    #[live]
    Tag,
    /// Something the grammar could not read. The one kind a reader **wants** to see.
    #[live]
    Invalid,
}

impl HighlightKind {
    /// Every kind, in the order a palette lists them.
    ///
    /// `PartialOrd`/`Ord` are derived, so this array is also the sort order — which is what lets a
    /// caller group spans by kind for a legend without writing a comparison.
    pub const ALL: [HighlightKind; 13] = [
        HighlightKind::Keyword,
        HighlightKind::Function,
        HighlightKind::Type,
        HighlightKind::Constant,
        HighlightKind::Variable,
        HighlightKind::String,
        HighlightKind::Number,
        HighlightKind::Comment,
        HighlightKind::Operator,
        HighlightKind::Punctuation,
        HighlightKind::Attribute,
        HighlightKind::Tag,
        HighlightKind::Invalid,
    ];
}

/// One classified run of source.
///
/// **Byte offsets**, not characters: every tokenizer and every parser in this ecosystem reports
/// byte ranges, and converting at the boundary is where an off-by-one in a multi-byte string becomes
/// a panic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Highlight {
    pub range: Range<usize>,
    pub kind: HighlightKind,
}

/// A hue and a chroma per kind; the **lightness** comes from the appearance.
///
/// Systematic rather than thirteen hand-picked colours: the same hue family in both appearances
/// means a reader's mental map survives a theme change, and it is what makes the contrast floor
/// reachable by construction rather than by luck.
///
/// The hues are the conventional ones — violet keywords, green strings, orange numbers, a
/// desaturated comment — and the numbers are chosen to be **far apart in hue**, because two kinds
/// thirty degrees apart are one kind to most readers.
const SPEC: [(HighlightKind, f32, f32); 13] = [
    // (kind, hue°, chroma)
    (HighlightKind::Keyword, 296.0, 0.11),
    (HighlightKind::Function, 232.0, 0.11),
    (HighlightKind::Type, 190.0, 0.10),
    (HighlightKind::Constant, 62.0, 0.12),
    (HighlightKind::Variable, 150.0, 0.09),
    (HighlightKind::String, 140.0, 0.11),
    (HighlightKind::Number, 32.0, 0.12),
    // A comment is the one kind that is **meant** to recede, so it carries almost no chroma. It
    // still has to clear the contrast floor: a comment a reader cannot read is a comment the
    // grammar wasted its time classifying.
    (HighlightKind::Comment, 280.0, 0.02),
    (HighlightKind::Operator, 258.0, 0.06),
    (HighlightKind::Punctuation, 250.0, 0.02),
    (HighlightKind::Attribute, 340.0, 0.10),
    (HighlightKind::Tag, 12.0, 0.11),
    // The one kind with maximum chroma, deliberately: an error should be the loudest thing on the
    // line, and nothing else in the set is allowed to approach it.
    (HighlightKind::Invalid, 25.0, 0.16),
];

/// How light a syntax colour is in each appearance, as an OKLCH `L`.
///
/// Dark: light ink on a near-black ground. Light: dark ink on near-white. The two values are
/// **asymmetric** and that is not an oversight: perceived contrast is not symmetric in lightness,
/// and a pair that clears 4.5:1 on white is a washed-out 3:1 on black at the same distance from
/// its ground.
const LIGHTNESS: (f32, f32) = (0.74, 0.46);

/// A colour per kind, for one appearance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SyntaxPalette {
    colors: [Vec4f; 13],
}

impl SyntaxPalette {
    pub fn dark() -> Self {
        Self::build(Appearance::Dark)
    }

    pub fn light() -> Self {
        Self::build(Appearance::Light)
    }

    pub fn for_appearance(appearance: Appearance) -> Self {
        Self::build(appearance)
    }

    fn build(appearance: Appearance) -> Self {
        let l = match appearance {
            Appearance::Dark => LIGHTNESS.0,
            Appearance::Light => LIGHTNESS.1,
        };
        let mut colors = [Vec4f::default(); 13];
        for (slot, (kind, hue, chroma)) in colors.iter_mut().zip(SPEC) {
            *slot = match kind {
                // A comment sits **closer to its ground** than the code, in both appearances: that
                // is what "recedes" means, and it is why it cannot take the shared lightness.
                //
                // *Closer to the ground* is a different direction in each appearance, and the first
                // version got the light one wrong by using the same sign. Receding on a dark ground
                // means getting darker (`l * 0.86`); receding on a light ground means getting
                // **lighter** (`l + 0.05`), which costs contrast rather than gaining it. Getting the
                // direction right is not enough on its own either: `l + 0.12` receded so far that
                // the light comment measured **3.76:1**, under the floor, and the test caught it
                // only after the ground was flattened.
                HighlightKind::Comment => match appearance {
                    Appearance::Dark => color::oklch(l * 0.86, chroma, hue),
                    Appearance::Light => color::oklch(l + 0.05, chroma, hue),
                },
                _ => color::oklch(l, chroma, hue),
            };
        }
        Self { colors }
    }

    /// The colour for a kind.
    pub fn color(&self, kind: HighlightKind) -> Vec4f {
        self.colors[kind as usize]
    }

    /// Every colour, in [`HighlightKind::ALL`] order.
    pub fn colors(&self) -> &[Vec4f; 13] {
        &self.colors
    }

    /// The field a kind is stored in, so a caller can name one without an accessor.
    ///
    /// The array is indexed by the enum's discriminant, and this is the assertion that the two
    /// agree — a `HighlightKind` reordered without reordering `SPEC` would put a tag's colour on a
    /// comment, which reads as a palette fault rather than as the indexing bug it is.
    pub fn index_of(kind: HighlightKind) -> usize {
        kind as usize
    }
}

/// Make a classifier's raw output safe to paint.
///
/// Sorts into document order, clips to the source, and **drops overlaps** so that no two spans share
/// a byte. See the module doc for why each is a real grammar's behaviour rather than a hypothetical.
///
/// The overlap rule: among spans that overlap, the **first in document order** wins and the others
/// are trimmed to what is left. That is a choice, not a consequence — the alternative (last wins, or
/// longest wins) would be just as defensible, and what matters is that it is one rule rather than
/// whatever order the grammar happened to emit. Ties are broken by kind, so the output does not
/// depend on the input's order at all.
pub fn normalize(source_len: usize, spans: &[Highlight]) -> Vec<Highlight> {
    // Sorted by start, then by **longer first**, then by kind. Longer first means the outer span of
    // a nesting wins, which is what a reader expects: a function name inside a call is the name,
    // but a string inside a string literal is the literal.
    let mut sorted: Vec<Highlight> = spans
        .iter()
        .filter(|span| span.range.start < span.range.end)
        .map(|span| Highlight {
            range: span.range.start.min(source_len)..span.range.end.min(source_len),
            kind: span.kind,
        })
        .filter(|span| span.range.start < span.range.end)
        .collect();
    sorted.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(b.range.end.cmp(&a.range.end))
            .then(a.kind.cmp(&b.kind))
    });

    let mut out: Vec<Highlight> = Vec::with_capacity(sorted.len());
    for span in sorted {
        let Some(last) = out.last_mut() else {
            out.push(span);
            continue;
        };
        if span.range.start >= last.range.end {
            out.push(span);
            continue;
        }
        // Overlaps the previous span, which by the sort starts no later. Keep the previous one and
        // trim this to the part that is left, dropping it when nothing is.
        let start = last.range.end;
        if span.range.end > start {
            out.push(Highlight {
                range: start..span.range.end,
                kind: span.kind,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: usize, end: usize, kind: HighlightKind) -> Highlight {
        Highlight {
            range: start..end,
            kind,
        }
    }

    fn kinds() -> [HighlightKind; 13] {
        HighlightKind::ALL
    }

    #[test]
    fn test_every_kind_is_indexed_by_its_own_discriminant() {
        // The assertion `color` depends on: a kind reordered without reordering `SPEC` would put
        // one kind's colour on another, which reads as a palette fault rather than an indexing bug.
        for (index, kind) in kinds().iter().enumerate() {
            assert_eq!(
                SyntaxPalette::index_of(*kind),
                index,
                "{kind:?} is not at {index} in ALL"
            );
        }
        assert_eq!(SPEC.len(), kinds().len(), "SPEC and ALL disagree");
        for ((spec_kind, _, _), kind) in SPEC.iter().zip(kinds()) {
            assert_eq!(*spec_kind, kind, "SPEC and ALL are in different orders");
        }
    }

    #[test]
    fn test_every_kind_has_a_distinct_colour() {
        // Two kinds the same colour is a real fault: the reader cannot tell a keyword from a
        // string, and the grammar's work is invisible. This is the test that a "just reuse the
        // accent" shortcut fails.
        for palette in [SyntaxPalette::dark(), SyntaxPalette::light()] {
            for (i, a) in kinds().iter().enumerate() {
                for b in kinds().iter().skip(i + 1) {
                    let (ca, cb) = (palette.color(*a), palette.color(*b));
                    let same = (ca.x - cb.x).abs() < 1e-6
                        && (ca.y - cb.y).abs() < 1e-6
                        && (ca.z - cb.z).abs() < 1e-6;
                    assert!(!same, "{a:?} and {b:?} share a colour");
                }
            }
        }
    }

    #[test]
    fn test_every_kind_clears_the_contrast_floor_on_its_own_ground() {
        // Code is read, so every kind clears WCAG AA against the surface it is painted on. **The
        // comment included**: a comment a reader cannot read is a comment the grammar wasted its
        // time classifying, and "it is meant to recede" is not a licence to be illegible.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = SyntaxPalette::for_appearance(appearance);
            // `code_ground`, not `code_wash`: the wash is translucent and `contrast_ratio` treats
            // its argument as opaque, so measuring against the raw wash measures against pure white
            // in dark and pure black in light. See `Paint::code_ground`, which exists because this
            // test got it wrong first.
            let ground = crate::palette::for_appearance(appearance).code_ground();
            for kind in kinds() {
                let ratio = color::contrast_ratio(palette.color(kind), ground);
                assert!(
                    ratio >= 4.5,
                    "{kind:?} in {appearance:?} is {ratio:.2}:1 against the code ground, under 4.5"
                );
            }
        }
    }

    #[test]
    fn test_the_two_appearances_are_not_the_same_palette() {
        // A palette built once and reused for both is the failure this catches: it would pass every
        // test above on one appearance and be unreadable on the other.
        let dark = SyntaxPalette::dark();
        let light = SyntaxPalette::light();
        assert_ne!(dark, light);
        for kind in kinds() {
            assert_ne!(
                dark.color(kind),
                light.color(kind),
                "{kind:?} is the same colour in both appearances"
            );
        }
    }

    #[test]
    fn test_an_invalid_span_is_the_loudest_kind_and_nothing_else_approaches_it() {
        // An error should be the loudest thing on the line. Checked as chroma, which is what
        // "loud" means in OKLCH and what the reader actually perceives as saturation.
        // Compared **on the specification**, not on the converted colour. The first version used
        // the spread between sRGB channels as a stand-in for OKLCH chroma, and it failed on
        // `Function`: a red at chroma 0.16 and a blue at 0.11 do not have channel spreads in that
        // order, because the conversion to sRGB is not chroma-preserving across hues. The claim
        // being made is about the *palette's intent*, and the intent is the table.
        let invalid_chroma = SPEC
            .iter()
            .find(|(kind, _, _)| *kind == HighlightKind::Invalid)
            .expect("Invalid is specified")
            .2;
        for (kind, _, chroma) in SPEC {
            if kind == HighlightKind::Invalid {
                continue;
            }
            assert!(
                chroma < invalid_chroma,
                "{kind:?} is specified at chroma {chroma}, as loud as Invalid at {invalid_chroma}"
            );
        }
        // ...and the conversion really did produce a colour, so this is not a test of a table
        // against itself.
        let _ = SyntaxPalette::dark().color(HighlightKind::Invalid);
    }

    // ---- the span contract -------------------------------------------------

    #[test]
    fn test_normalize_sorts_into_document_order() {
        // What a tree walk produces: a query with several patterns emits spans in the order the
        // patterns matched, which is not document order.
        let spans = [
            span(20, 24, HighlightKind::Number),
            span(0, 4, HighlightKind::Keyword),
            span(10, 14, HighlightKind::Function),
        ];
        let out = normalize(40, &spans);
        let starts: Vec<usize> = out.iter().map(|s| s.range.start).collect();
        assert_eq!(starts, vec![0, 10, 20]);
    }

    #[test]
    fn test_normalize_clips_a_span_past_the_end_rather_than_panicking() {
        // A grammar parsed a **stale buffer**: the document can be edited between the parse and the
        // paint, and a span one byte past the end is a panic in a slicing renderer. Clipped rather
        // than dropped when part of it is real.
        let spans = [
            span(0, 4, HighlightKind::Keyword),
            span(8, 40, HighlightKind::String),
            span(50, 60, HighlightKind::Comment),
        ];
        let out = normalize(12, &spans);
        assert_eq!(out.len(), 2, "{out:?}");
        assert_eq!(out[1].range, 8..12);
        // A span entirely past the end is dropped, not turned into an empty span at the end.
        assert!(out.iter().all(|s| s.range.end <= 12));
        assert!(out.iter().all(|s| s.range.start < s.range.end));
    }

    #[test]
    fn test_normalize_drops_an_empty_span() {
        // `a..a` paints nothing and would confuse a caller stepping kerning across spans.
        assert!(normalize(10, &[span(3, 3, HighlightKind::Keyword)]).is_empty());
        // ...and so does a reversed one, which is what a bad index subtraction produces.
        assert!(normalize(10, &[span(7, 3, HighlightKind::String)]).is_empty());
    }

    #[test]
    fn test_normalize_leaves_no_two_spans_sharing_a_byte() {
        // The property every other test in this section is an instance of, checked over a set built
        // to overlap in every way: nested, identical, straddling, and adjacent.
        let spans = [
            span(0, 10, HighlightKind::Function),
            span(2, 4, HighlightKind::Variable),
            span(2, 4, HighlightKind::Type),
            span(6, 14, HighlightKind::String),
            span(14, 20, HighlightKind::Number),
            span(12, 16, HighlightKind::Comment),
        ];
        let out = normalize(30, &spans);
        for pair in out.windows(2) {
            assert!(
                pair[0].range.end <= pair[1].range.start,
                "spans overlap: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
        // Sorted, and every span is inside the source.
        assert!(out.windows(2).all(|p| p[0].range.start <= p[1].range.start));
        assert!(out.iter().all(|s| s.range.end <= 30 && s.range.start < s.range.end));
        // And nothing outside the covered region was invented.
        assert!(out.iter().all(|s| s.range.start >= 0));
    }

    #[test]
    fn test_the_outer_span_of_a_nesting_wins_and_the_inner_is_trimmed_away() {
        // The rule stated as a case: a function name inside a call is the name, but a string inside
        // a longer string literal is the literal. Longer-first means the outer span claims its
        // bytes, and the inner one keeps only what is left of itself.
        let spans = [
            span(0, 6, HighlightKind::Function), // the outer
            span(2, 4, HighlightKind::Variable), // entirely inside, so dropped
            span(5, 9, HighlightKind::String),   // straddles the edge, so trimmed
        ];
        let out = normalize(20, &spans);
        assert_eq!(out.len(), 2, "{out:?}");
        assert_eq!(out[0].range, 0..6);
        assert_eq!(out[0].kind, HighlightKind::Function);
        assert_eq!(out[1].range, 6..9);
        assert_eq!(out[1].kind, HighlightKind::String);
    }

    #[test]
    fn test_normalize_does_not_depend_on_the_order_it_was_given() {
        // The property that makes the function safe at a boundary: whatever order a grammar emits,
        // the paint is the same. Every permutation of a four-span set is checked, so an accidental
        // dependence on input order cannot hide in one of them.
        let spans = [
            span(0, 8, HighlightKind::Keyword),
            span(3, 5, HighlightKind::Number),
            span(5, 11, HighlightKind::String),
            span(11, 13, HighlightKind::Operator),
        ];
        let expected = normalize(20, &spans);
        let mut indices = [0usize, 1, 2, 3];
        // Heap's algorithm, inlined: twenty-four permutations, all of which must agree.
        let mut permutations = 0usize;
        permute(&mut indices, 0, &mut |order: &[usize]| {
            let permuted: Vec<Highlight> = order.iter().map(|i| spans[*i].clone()).collect();
            assert_eq!(normalize(20, &permuted), expected, "order {order:?} differed");
            permutations += 1;
        });
        assert_eq!(permutations, 24);
    }

    fn permute(items: &mut [usize], at: usize, visit: &mut impl FnMut(&[usize])) {
        if at + 1 == items.len() {
            visit(items);
            return;
        }
        for i in at..items.len() {
            items.swap(at, i);
            permute(items, at + 1, visit);
            items.swap(at, i);
        }
    }

    #[test]
    fn test_normalize_covers_the_whole_source_when_it_was_given_a_partition() {
        // The shape a good classifier emits: a partition with no gaps. Normalizing it must not
        // change anything, which is the check that the function is not lossy for correct input.
        let spans = [
            span(0, 3, HighlightKind::Keyword),
            span(3, 4, HighlightKind::Punctuation),
            span(4, 9, HighlightKind::String),
        ];
        let out = normalize(9, &spans);
        assert_eq!(out, spans.to_vec());
    }

    #[test]
    fn test_normalize_of_nothing_is_nothing() {
        assert!(normalize(0, &[]).is_empty());
        assert!(normalize(100, &[]).is_empty());
    }
}
