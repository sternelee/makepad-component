//! `MpPage` — the page's measure, its header row, and its subtitle.
//!
//! ## The measure is the component
//!
//! [`MAX_MEASURE`] is 768 points and the column is centred with its auto margins, which is the one thing a page needs that a
//! stack does not: **a bounded line length**. Prose set the full width of a wide window is hard to track back from the end of
//! one line to the start of the next, and every serious reading surface picks a measure for that reason. bezel's number is
//! kept; the reasoning behind the *kind* of number is recorded here because bezel does not state it.
//!
//! ## A count shares the title's baseline, and that is not decoration
//!
//! bezel's `page_header` takes `Option<usize>` and puts it in a **baseline-aligned row** with the title, citing the settings
//! page it came from. A count set on its own line reads as a subtitle — a separate thought — while a count on the title's
//! baseline reads as a count *of* that title, which is what it is. So the offset is computed rather than guessed:
//! [`count_offset`] is the difference between the two faces' baselines, and it exists because makepad has no baseline
//! alignment and drawing both at the same `y` aligns their **tops**, not their baselines.
//!
//! ## The subtitle sits under the header, and the caller composes
//!
//! bezel returns three separate builders rather than a page component, so a page with no subtitle has no gap where one would
//! have been. This keeps that: [`MpPage`] is the column, [`MpPageHeader`] is the row, and the subtitle is a labelled view a
//! caller drops underneath — three small things rather than one that guesses at the shape of every page.

use makepad_widgets::*;

/// The widest the page's content grows.
///
/// A measure, not a layout accident: a line of prose wider than this is hard to track from its end back to its start. bezel's
/// number, kept — the *kind* of number is what matters and the exact value is a house style.
pub const MAX_MEASURE: f64 = 768.0;

/// The page column's horizontal padding.
pub const PAGE_PAD_X: f64 = 24.0;

/// Its top and bottom padding.
///
/// Asymmetric on purpose: a page starts close to the window's edge and ends with room to scroll past its last line, which is
/// why the bottom is twice the top.
pub const PAGE_PAD_TOP: f64 = 32.0;
pub const PAGE_PAD_BOTTOM: f64 = 64.0;

/// The gap between a title and its count.
pub const COUNT_GAP: f64 = 10.0;

/// The subtitle's top margin.
pub const SUBTITLE_GAP: f64 = 4.0;

/// The title's line height, and the count's.
pub const TITLE_LINE: f64 = 26.0;
pub const COUNT_LINE: f64 = 18.0;

/// Where a face's baseline sits within its line, as a fraction of the line height.
///
/// **An approximation, and it is one because makepad reports no baseline metric.** Every face this library uses is a
/// proportional sans, whose baseline sits near four fifths of the line — close enough that the two texts read as sharing one,
/// which is the whole of what baseline alignment is for. A caller using a face with an unusual baseline should set the offset
/// itself rather than trust this.
pub const BASELINE_RATIO: f64 = 0.8;

/// The `y` offset that puts a count's baseline on the title's.
///
/// Zero when the two faces are the same size, positive when the count's face is smaller — which it always is, and which is why
/// drawing both at the same `y` (the obvious thing) aligns their tops and leaves the count visibly riding high.
pub fn count_offset(title_line: f64, count_line: f64) -> f64 {
    // **A nonsense input is no offset, not a zero face.** Two failures led here. Treating a non-finite line as zero let an
    // 18-point count against a nonsense title produce `-14.4`, which would lift the count *above* the title's baseline; and
    // then treating a nonsense *count* as zero produced `20.8`, which would push it far down. Both are the same mistake — a
    // missing measurement quietly becoming a number — so neither input being a measurement means the answer is no offset.
    if !title_line.is_finite() || !count_line.is_finite() {
        return 0.0;
    }
    // And a count is never *above* the title it counts, so the offset is clamped at zero as well.
    ((title_line.max(0.0) - count_line.max(0.0)) * BASELINE_RATIO).max(0.0)
}

/// The page's content width for a viewport.
///
/// The measure, or the viewport less its padding when the window is narrower — never wider than the window, and never
/// negative: a window smaller than two paddings has no content width rather than a negative one.
pub fn content_width(viewport: f64) -> f64 {
    if !viewport.is_finite() || viewport <= 0.0 {
        return 0.0;
    }
    (viewport - PAGE_PAD_X * 2.0).clamp(0.0, MAX_MEASURE)
}

/// The inset a centred column has in a viewport.
///
/// The auto margin, as arithmetic: half of what the viewport has left once the content has taken its measure. Zero when the
/// content fills the window, so a narrow window has no margin rather than a negative one.
pub fn centred_inset(viewport: f64, content: f64) -> f64 {
    if !viewport.is_finite() || !content.is_finite() || viewport <= 0.0 || content <= 0.0 {
        return 0.0;
    }
    ((viewport - content) * 0.5).max(0.0)
}

/// The count as the header prints it, or `None` when there is no count.
///
/// `Some(0)` prints `"0"` — **a count of none is a count**, and a page that hid it would be hiding the answer to "how many are
/// there". A caller with nothing to count passes `None` instead, which is a different statement.
pub fn count_text(count: Option<usize>) -> Option<String> {
    count.map(|count| count.to_string())
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The page's column: centred, bounded to the measure, with room above and below.
    mod.mp.MpPage = View{
        width: Fill
        height: Fit
        flow: Down
        align: Align{x: 0.5}
        padding: Inset{left: 24.0, right: 24.0, top: 32.0, bottom: 64.0}

        page_content := View{
            width: 768.0
            height: Fit
            flow: Down
            spacing: 24.0
        }
    }

    mod.mp.MpPageHeaderBase = #(MpPageHeader::register_widget(vm))

    mod.mp.MpPageHeader = set_type_default() do mod.mp.MpPageHeaderBase{
        width: Fit
        height: Fit

        draw_title +: {text_style: mod.mpc.type.title2, color: mod.mpc.tokens.text}
        draw_count +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text_muted}
    }

    /// The subtitle: a line under the header, and nothing at all when there is nothing to say.
    mod.mp.MpPageSubtitle = Label{
        width: Fill, height: Fit
        margin: Inset{left: 0, right: 0, top: 4.0, bottom: 0}
        draw_text +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text_muted}
    }
}

/// A page's header row: the title, and how many of the thing there are.
#[derive(Script, ScriptHook, Widget)]
pub struct MpPageHeader {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_title: DrawText,
    #[live]
    draw_count: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    title: String,
    /// `None` is "there is no count" and `Some(0)` is "there are none" — two different statements.
    #[rust]
    count: Option<usize>,
    #[rust]
    area: Area,
}

impl MpPageHeader {
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        if self.title != title {
            self.title = title.to_string();
            self.redraw(cx);
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the count, or `None` for a page that has nothing to count.
    pub fn set_count(&mut self, cx: &mut Cx, count: Option<usize>) {
        if self.count != count {
            self.count = count;
            self.redraw(cx);
        }
    }

    pub fn count(&self) -> Option<usize> {
        self.count
    }
}

impl Widget for MpPageHeader {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // A header is a label, not a control: it takes no pointer and holds no focus.
        let _ = event.hits(cx, self.area);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (ink, muted) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (theme.paint.text, theme.paint.text_muted)
        };
        let placed = cx.walk_turtle(Walk {
            height: Size::Fixed(TITLE_LINE),
            ..walk
        });
        self.area = self.draw_title.area();
        self.draw_title.color = ink;
        self.draw_title
            .draw_abs(cx, placed.pos, self.title.as_str());
        if let Some(count) = count_text(self.count) {
            let width = crate::mp::text::measured_width(&self.draw_title, cx.cx, &self.title);
            self.draw_count.color = muted;
            // **On the title's baseline**, computed rather than guessed — see `count_offset`. Drawing both at `placed.pos.y`
            // would align their tops and leave the count riding high, which is the difference between a count *of* the title
            // and a second thought beside it.
            self.draw_count.draw_abs(
                cx,
                dvec2(
                    placed.pos.x + width + COUNT_GAP,
                    placed.pos.y + count_offset(TITLE_LINE, COUNT_LINE),
                ),
                &count,
            );
        }
        DrawStep::done()
    }
}

impl MpPageHeaderRef {
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn set_count(&self, cx: &mut Cx, count: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_count(cx, count);
        }
    }

    pub fn title(&self) -> String {
        self.borrow().map(|inner| inner.title().to_string()).unwrap_or_default()
    }

    pub fn count(&self) -> Option<usize> {
        self.borrow().and_then(|inner| inner.count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_measure_bounds_a_line_and_the_padding_bounds_a_narrow_window() {
        // **The component is the measure.** A line of prose set the full width of a wide window is hard to track from its end
        // back to its start, which is why every serious reading surface picks one. On a narrow window the padding is what
        // bounds it, and neither is ever negative.
        assert_eq!(content_width(2000.0), MAX_MEASURE);
        assert_eq!(content_width(MAX_MEASURE + PAGE_PAD_X * 2.0), MAX_MEASURE);
        assert_eq!(content_width(600.0), 600.0 - PAGE_PAD_X * 2.0, "a narrow window loses its padding");
        assert_eq!(content_width(10.0), 0.0, "a window narrower than its padding has no content");
        assert_eq!(content_width(0.0), 0.0);
        assert_eq!(content_width(-100.0), 0.0);
        assert_eq!(content_width(f64::NAN), 0.0);
        // The content is never wider than the window it is in, at any viewport.
        for viewport in [100.0, 400.0, 816.0, 4000.0] {
            assert!(content_width(viewport) <= viewport.max(0.0), "viewport {viewport}");
        }
    }

    #[test]
    fn test_the_column_is_centred_by_the_arithmetic_rather_than_by_the_layout() {
        // The auto margin, as arithmetic: half of what the viewport has left once the content has taken its measure — and
        // zero on a narrow window rather than a negative inset that would push the content off its own edge.
        assert_eq!(centred_inset(2000.0, MAX_MEASURE), (2000.0 - MAX_MEASURE) * 0.5);
        assert_eq!(centred_inset(768.0, MAX_MEASURE), 0.0);
        assert_eq!(centred_inset(400.0, MAX_MEASURE), 0.0, "a narrow window has no margin");
        assert_eq!(centred_inset(0.0, MAX_MEASURE), 0.0);
        assert_eq!(centred_inset(f64::NAN, MAX_MEASURE), 0.0);
        assert_eq!(centred_inset(2000.0, f64::NAN), 0.0);
        // And the margins add up: content plus both insets is the viewport, when the content fits.
        let viewport = 1600.0;
        let content = content_width(viewport);
        let inset = centred_inset(viewport, content);
        assert!((content + inset * 2.0 - viewport).abs() < 1e-9, "the centring does not add up");
    }

    #[test]
    fn test_a_count_of_none_is_a_different_statement_from_a_count_of_zero() {
        // **`Some(0)` prints "0" and `None` prints nothing.** A page that hid a zero would be hiding the answer to "how many
        // are there"; a caller with nothing to count passes `None`, which is a different statement about a different page.
        assert_eq!(count_text(Some(0)), Some("0".to_string()));
        assert_eq!(count_text(Some(4)), Some("4".to_string()));
        assert_eq!(count_text(Some(1234)), Some("1234".to_string()));
        assert_eq!(count_text(None), None);
        assert_ne!(count_text(None), count_text(Some(0)));
        // The count is a plain decimal, not a thousands-separated rendering: a count beside a title is read at a glance and
        // its grouping is the locale's business rather than a header's.
        assert_eq!(count_text(Some(1000)), Some("1000".to_string()));
    }

    #[test]
    fn test_the_count_sits_on_the_title_s_baseline_and_drawing_it_at_the_same_y_does_not() {
        // **The rule bezel cites its own settings page for.** A count on its own line reads as a subtitle — a separate
        // thought — while one on the title's baseline reads as a count *of* that title. makepad has no baseline alignment, so
        // the offset is computed; drawing both at the same `y` aligns their **tops**, which is the mistake this function
        // exists to prevent.
        let offset = count_offset(TITLE_LINE, COUNT_LINE);
        assert!(offset > 0.0, "a smaller face drawn at the same y rides high, not low");
        assert_eq!(offset, (TITLE_LINE - COUNT_LINE) * BASELINE_RATIO);
        assert_eq!(count_offset(18.0, 18.0), 0.0, "two equal faces already share a baseline");
        // The offset scales with the difference and never goes negative for a count that is not larger than its title.
        assert!(count_offset(26.0, 14.0) > count_offset(26.0, 20.0));
        // A nonsense line height offsets by nothing rather than by a `NaN` the draw would inherit.
        assert_eq!(count_offset(f64::NAN, 18.0), 0.0);
        assert_eq!(count_offset(26.0, f64::NAN), 0.0);
        assert_eq!(count_offset(-5.0, 18.0), 0.0);
    }

    #[test]
    fn test_the_page_s_padding_is_asymmetric_on_purpose() {
        // A page starts close to the window's edge and ends with room to scroll past its last line, which is why the bottom is
        // twice the top. Asserted so a later edit cannot make them equal without saying so.
        assert_eq!(PAGE_PAD_TOP * 2.0, PAGE_PAD_BOTTOM);
        assert!(PAGE_PAD_X > 0.0 && PAGE_PAD_TOP > 0.0);
        // And the DSL's own numbers agree with the constants, so neither can be changed alone.
        let source = include_str!("page.rs");
        let dsl = &source[..source.find("#[cfg(test)]").unwrap_or(source.len())];
        assert!(dsl.contains("width: 768.0"), "the page's measure is not in the DSL");
        assert!(dsl.contains("top: 32.0"), "the page's top padding is not in the DSL");
        assert!(dsl.contains("bottom: 64.0"), "the page's bottom padding is not in the DSL");
        assert_eq!(MAX_MEASURE, 768.0);
        assert_eq!(PAGE_PAD_TOP, 32.0);
        assert_eq!(PAGE_PAD_BOTTOM, 64.0);
        assert_eq!(SUBTITLE_GAP, 4.0);
        assert!(dsl.contains("top: 4.0"), "the subtitle's gap is not in the DSL");
    }
}
