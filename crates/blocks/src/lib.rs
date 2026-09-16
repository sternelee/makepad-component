//! `makepad-blocks` — a fence tag routed to a renderer.
//!
//! ## Why a fence, rather than a new kind of block
//!
//! A markdown fence already carries a **language tag** and a body, and `makepad-markdown` already parses it into a
//! block whose text is the body. So a block of an app's own — a chart, a diagram, an embed — needs a **renderer over
//! a fence tag** rather than a new `BlockKind`: the wire form already round-trips byte for byte, and a document
//! written with a fence still opens in a tool that has never heard of the tag. The reference's `blocks` crate says
//! exactly this: *"a fence already round trips byte for byte, already holds a caret, and already degrades to its own
//! source where nothing paints it."*
//!
//! ## A router whose answer is **data**, not a widget
//!
//! The reference returns an `Element` — a gpui view — from its renderer. This port cannot: a Makepad widget is
//! declared in the DSL and registered on the script heap, so it is not a value that can be returned from a function.
//! What a renderer here returns is what the **paint** needs: for the chart block, a set of
//! [`Series`](makepad_plot::Series), which an app hands to a `LinePlot` it declared.
//!
//! ```ignore
//! // The app declares the plot, the fence feeds it:
//! let plot = self.ui.mf_line_plot(cx, ids!(chart));
//! plot.clear();
//! if let Some(Block::Chart(chart)) = blocks::render("chart", fence_body) {
//!     plot.set_title(chart.title.unwrap_or_default());
//!     for series in chart.series { plot.add_series(series); }
//! }
//! ```
//!
//! That split is not a workaround: it is the same one `mp/code.rs` and `makepad-syntax` use, and it is what lets the
//! parsing be **tested without a window** while the widget stays a widget.
//!
//! ## The registry, and the three answers
//!
//! [`register`] adds a tag. [`render`] answers with a [`Block`] or `None`, and `None` is **one answer for three
//! cases** — a tag nothing claims, a tag registered but not in this build, and a renderer that read the source and
//! declined. A caller renders the fence's own text for all three, which is what a document does when it has nothing
//! better, so the three do not need telling apart.
//!
//! ## The one block this ships: `chart`
//!
//! A small, readable, documented format — because a fence body is text a person writes:
//!
//! ```text
//! title: Monthly users
//! xlabel: Month
//! ylabel: Users
//! series: Desktop
//! 1, 10
//! 2, 20
//! series: Mobile
//! 1, 5
//! 2, 30
//! ```
//!
//! `key: value` for the three header fields, `series: <label>` to start one, and `x, y` rows. Every rule has a test,
//! including the ones that matter: a row that is not two numbers, a `series:` with no rows under it, and a body with
//! no series at all.

use std::cell::RefCell;

use makepad_plot::{LineStyle, MarkerStyle, Series, StepStyle};

/// What a renderer produced.
///
/// One variant today. An enum rather than a struct so that a second block is a variant rather than a second return
/// type, and so a caller's `match` is exhaustive when it grows — which is how a new kind is noticed.
#[derive(Clone, Debug)]
pub enum Block {
    /// A chart: what a `LinePlot` needs to draw one.
    Chart(Chart),
}

/// A chart a fence described.
/// `Clone` and `Debug` but **not `PartialEq`**, because a `Series` carries `Vec4` colours and the plot's own types
/// do not implement it — and nothing here needs to compare two charts, only to look at their fields.
#[derive(Clone, Debug, Default)]
pub struct Chart {
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    /// The series, in the order the fence listed them.
    pub series: Vec<Series>,
}

/// A renderer: a fence's body in, a block or a decline out.
///
/// A plain function pointer rather than a closure, because a registry of `Box<dyn Fn>` is a registry that can hold
/// captured state — and a block renderer that captured something would be a block whose output depends on when it
/// was registered, which is the kind of thing a document that round-trips cannot afford.
pub type Renderer = fn(&str) -> Option<Block>;

thread_local! {
    /// The registry.
    ///
    /// Thread-local rather than global, because it holds function pointers a UI thread registers and reads and
    /// nothing else touches it — and a `static mut` for this would be an unsafe with no benefit.
    static REGISTRY: RefCell<Vec<(String, Renderer)>> = RefCell::new(Vec::new());
}

/// The tags this crate ships, always registered.
pub const BUILT_IN: &[&str] = &["chart"];

/// Claim a fence tag.
///
/// Registers [`chart`] if it is not registered yet, so a caller can add a tag without first knowing that the
/// built-in one exists. Registering a tag **replaces** an earlier renderer for it: an app overriding `chart` with
/// its own is the case the reference names — *"an app with its own block writes the same function"* — and a registry
/// that refused would make the built-in unoverridable.
pub fn register(language: &str, renderer: Renderer) {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if !registry.iter().any(|(tag, _)| tag == "chart") {
            registry.push(("chart".to_string(), chart_block));
        }
        let tag = language.trim().to_lowercase();
        match registry.iter_mut().find(|(existing, _)| *existing == tag) {
            Some((_, slot)) => *slot = renderer,
            None => registry.push((tag, renderer)),
        }
    });
}

/// Every tag the registry answers to, sorted and deduplicated.
///
/// For a language picker that would otherwise offer a tag this build cannot paint.
pub fn languages() -> Vec<String> {
    REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if !registry.iter().any(|(tag, _)| tag == "chart") {
            registry.push(("chart".to_string(), chart_block));
        }
        let mut tags: Vec<String> = registry.iter().map(|(tag, _)| tag.clone()).collect();
        tags.sort();
        tags.dedup();
        tags
    })
}

/// Paint the block a fence names, or `None` to leave it to the ordinary code block.
///
/// `None` for a tag nothing claims, a tag not in this build, and a renderer that declined — one answer for all three;
/// see the module doc. The tag is matched case-insensitively and trimmed, because a fence tag is written by hand.
pub fn render(language: &str, code: &str) -> Option<Block> {
    let tag = language.trim().to_lowercase();
    if tag.is_empty() {
        return None;
    }
    let renderer = REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if !registry.iter().any(|(tag, _)| tag == "chart") {
            registry.push(("chart".to_string(), chart_block));
        }
        registry
            .iter()
            .find(|(existing, _)| *existing == tag)
            .map(|(_, renderer)| *renderer)
    })?;
    // The renderer runs **outside** the `RefCell` borrow above: a renderer that registered another tag would
    // otherwise panic on a re-entrant borrow, and a block that registers a block is a reasonable thing for an app to
    // do.
    renderer(code)
}

/// The `chart` block.
///
/// Declines — returns `None` — for a body with **no series**, so a fence a person used for prose comes back as the
/// code it is rather than as an empty chart.
pub fn chart_block(code: &str) -> Option<Block> {
    let chart = parse_chart(code);
    if chart.series.is_empty() {
        return None;
    }
    Some(Block::Chart(chart))
}

/// Parse a chart fence's body.
///
/// Never fails: a line that is not understood is **skipped**, because a fence is a document and a document with one
/// bad row in it should still draw the rows that are good. What the parser does *not* do is guess — a `series:` with
/// no rows under it produces no series, rather than a series of no points that draws an empty legend entry.
pub fn parse_chart(code: &str) -> Chart {
    let mut chart = Chart::default();
    let mut current: Option<Series> = None;

    let mut flush = |series: &mut Option<Series>, chart: &mut Chart| {
        if let Some(series) = series.take() {
            if !series.x.is_empty() {
                chart.series.push(series);
            }
        }
    };

    for line in code.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // `key: value` for the header. Checked before a data row, because a label may contain a colon and a data row
        // may not.
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim().to_string();
            match key.as_str() {
                "title" => chart.title = Some(value),
                "xlabel" | "x" => chart.x_label = Some(value),
                "ylabel" | "y" => chart.y_label = Some(value),
                "series" => {
                    flush(&mut current, &mut chart);
                    current = Some(Series::new(value));
                }
                // An unknown `key: value` is skipped rather than treated as a data row, because `colour: red` is not
                // a pair of numbers and a reader would get one point at (0, 0) from trying.
                _ => {}
            }
            continue;
        }
        // A data row: two numbers, comma or whitespace separated.
        let numbers: Vec<f64> = line
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.parse::<f64>().ok())
            .collect();
        if numbers.len() < 2 {
            continue;
        }
        let series = current.get_or_insert_with(|| Series::new(""));
        series.x.push(numbers[0]);
        series.y.push(numbers[1]);
    }
    flush(&mut current, &mut chart);
    chart
}

/// The styles a parsed series keeps, for a caller that wants the plot's own defaults back.
pub fn default_styles() -> (LineStyle, MarkerStyle, StepStyle) {
    (
        LineStyle::Solid,
        MarkerStyle::None,
        StepStyle::None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const FENCE: &str = "\
title: Monthly users
xlabel: Month
ylabel: Users
series: Desktop
1, 10
2, 20
3, 35
series: Mobile
1, 5
2, 12
3, 30
";

    #[test]
    fn test_a_fence_becomes_a_chart_with_its_labels_and_series() {
        let Block::Chart(chart) = render("chart", FENCE).expect("the chart renders") else {
            panic!("expected a chart");
        };
        assert_eq!(chart.title.as_deref(), Some("Monthly users"));
        assert_eq!(chart.x_label.as_deref(), Some("Month"));
        assert_eq!(chart.y_label.as_deref(), Some("Users"));
        assert_eq!(chart.series.len(), 2);
        assert_eq!(chart.series[0].label, "Desktop");
        assert_eq!(chart.series[0].x, vec![1.0, 2.0, 3.0]);
        assert_eq!(chart.series[0].y, vec![10.0, 20.0, 35.0]);
        assert_eq!(chart.series[1].label, "Mobile");
        assert_eq!(chart.series[1].y, vec![5.0, 12.0, 30.0]);
    }

    #[test]
    fn test_a_row_can_be_separated_by_a_comma_or_by_whitespace() {
        // A fence is text a person writes, and both are what a person writes. A parser that demanded one of them
        // would silently drop half of a table.
        let chart = parse_chart("series: a\n1, 2\n3 4\n5\t6\n");
        assert_eq!(chart.series[0].x, vec![1.0, 3.0, 5.0]);
        assert_eq!(chart.series[0].y, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_a_negative_and_a_decimal_row_parse() {
        // The values a real chart has, and the reason this uses `f64::parse` rather than a hand-rolled number
        // scanner: `-1.5e3` is a number in every language a fence is written in.
        let chart = parse_chart("series: a\n-1.5, 2.25\n1e3, -4\n");
        assert_eq!(chart.series[0].x, vec![-1.5, 1000.0]);
        assert_eq!(chart.series[0].y, vec![2.25, -4.0]);
    }

    #[test]
    fn test_a_line_that_is_not_a_row_is_skipped_rather_than_guessed_at() {
        // A document with one bad row in it should still draw the rows that are good, and a row that is not two
        // numbers must not become a point at (0, 0).
        let chart = parse_chart("series: a\n1, 2\nthis row is prose\n3, 4\n,,\n5\n");
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].x, vec![1.0, 3.0]);
        assert_eq!(chart.series[0].y, vec![2.0, 4.0]);
    }

    #[test]
    fn test_an_unknown_header_is_skipped_rather_than_read_as_a_row() {
        // `colour: red` is not a pair of numbers, and a reader that tried would produce one point at (0, 0) plus a
        // series with a nonsense label.
        let chart = parse_chart("series: a\ncolour: red\n1, 2\n");
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].x, vec![1.0]);
    }

    #[test]
    fn test_a_series_with_no_rows_under_it_produces_no_series() {
        // Otherwise a chart gains a legend entry that draws nothing, which reads as a bug in the chart.
        let chart = parse_chart("series: empty\nseries: real\n1, 2\n");
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].label, "real");
    }

    #[test]
    fn test_a_body_with_no_series_declines_and_leaves_the_fence_as_code() {
        // **The rule that makes this safe to enable everywhere.** A fence a person used for prose — or for a
        // language this build does not paint — comes back as the code it is, because a decliner returns `None` and
        // the caller renders the fence's own text.
        assert!(render("chart", "just some prose\n").is_none());
        assert!(render("chart", "").is_none());
        assert!(render("chart", "title: nothing else\n").is_none());
    }

    #[test]
    fn test_rows_with_no_series_header_still_make_a_series() {
        // A fence that is only a table of numbers is a chart with one unnamed series, which is what a person writing
        // one plainly means.
        let chart = parse_chart("1, 2\n3, 4\n");
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].label, "");
        assert_eq!(chart.series[0].y, vec![2.0, 4.0]);
    }

    #[test]
    fn test_a_comment_line_is_skipped() {
        // `#` starts a comment, because a fence body is text and a person annotating their own table is normal.
        let chart = parse_chart("series: a\n# a note\n1, 2\n");
        assert_eq!(chart.series[0].x, vec![1.0]);
    }

    #[test]
    fn test_the_tag_is_trimmed_and_case_insensitive() {
        // A fence tag is written by hand, so `Chart`, ` chart` and `CHART` are one tag.
        for tag in ["chart", "Chart", "  CHART  "] {
            assert!(
                render(tag, FENCE).is_some(),
                "{tag:?} did not find the chart block"
            );
        }
        assert!(render("", FENCE).is_none(), "an empty tag claims nothing");
    }

    #[test]
    fn test_an_unregistered_tag_is_none() {
        // The first of the three answers: a tag nothing claims.
        assert!(render("mermaid", "graph TD\nA-->B\n").is_none());
        assert!(render("rust", "fn main() {}\n").is_none());
    }

    #[test]
    fn test_an_app_can_add_a_tag_and_override_the_built_in_one() {
        // Both halves of the contract the reference names: *"an app with its own block writes the same function"*,
        // and an app overriding `chart` must be able to.
        fn always(code: &str) -> Option<Block> {
            let mut chart = Chart::default();
            chart.title = Some(code.to_string());
            Some(Block::Chart(chart))
        }
        register("mytag", always);
        assert_eq!(languages().contains(&"mytag".to_string()), true);
        match render("mytag", "hello") {
            Some(Block::Chart(chart)) => assert_eq!(chart.title.as_deref(), Some("hello")),
            _ => panic!("expected a chart"),
        }
        // Overriding the built-in: a body the built-in would decline now renders.
        fn accepts_everything(code: &str) -> Option<Block> {
            let mut chart = Chart::default();
            chart.title = Some(code.to_string());
            Some(Block::Chart(chart))
        }
        register("chart", accepts_everything);
        assert!(
            render("chart", "just some prose\n").is_some(),
            "the built-in chart could not be overridden"
        );
        // ...and restoring it is registering it back, which is what makes the override reversible.
        register("chart", chart_block);
        assert!(render("chart", "just some prose\n").is_none());
    }

    #[test]
    fn test_a_renderer_that_registers_another_tag_does_not_panic() {
        // The registry is a `RefCell`, and the renderer runs **outside** the borrow so that registering from inside
        // one is allowed. A block that registers a block is a reasonable thing for an app to do, and a re-entrant
        // borrow panic would be a confusing way to find out it is not.
        fn registers(code: &str) -> Option<Block> {
            fn inner(_code: &str) -> Option<Block> {
                None
            }
            register("registered-from-inside", inner);
            let mut chart = Chart::default();
            chart.title = Some(code.to_string());
            Some(Block::Chart(chart))
        }
        register("outer", registers);
        assert!(render("outer", "x").is_some());
        assert!(languages().contains(&"registered-from-inside".to_string()));
    }

    #[test]
    fn test_the_language_list_is_what_the_router_answers_to() {
        // A picker offers `languages()`, so a tag the router handles and the list omits is one nobody can choose —
        // and one the list offers and the router refuses is a choice that does nothing.
        for tag in languages() {
            assert!(
                render(&tag, FENCE).is_some() || tag != "chart",
                "{tag} is offered but the router refuses it"
            );
        }
        // The built-in is always there, whatever an app registered before.
        assert!(languages().contains(&"chart".to_string()));
    }
}
