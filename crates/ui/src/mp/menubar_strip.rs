//! `MpMenubarStrip` — the row of titles, and the anchoring between a title and the panel it drops.
//!
//! ## What the strip owns, and what it does not
//!
//! It owns the **geometry**: how wide each title is, where each one starts, which one an `x` lands on, and where the panel
//! below a title should be anchored. It does **not** own the state — which menu is open and where the cursor is are
//! [`crate::mp::menubar::Bar`]'s, and the caller holds that, for the same reason the menu card does not hold its own cursor:
//! only the caller knows what opening means. The strip is told what to draw and reports what the pointer did.
//!
//! So this file is the third of three and the smallest: [`crate::mp::menubar`] has the rules, [`crate::mp::menu_card`] has the
//! panel, and this is the row that connects them — and the connection is one function with a rule in it.
//!
//! ## The rule: the panel hangs off its title, not off the strip
//!
//! [`title_rect`] is the whole reason this is a widget rather than twenty lines in a caller. A menu's panel must start at
//! **its own title's** leading edge, not the strip's — otherwise every menu in the bar opens in the same place and the
//! connection between the title and what it dropped is lost. And a panel near the trailing edge must not run off the window,
//! which the caller can only avoid if it is told where the title actually is.
//!
//! ## Heights are not invented here
//!
//! [`STRIP_HEIGHT`] is 24 points, the in-window strip's own. On macOS the *system* menubar is the platform's and an app
//! should not draw one at all; this is for the in-window bar an app draws for itself and the one every other platform expects
//! — which is the distinction bezel makes about its own bar, kept rather than flattened.

use makepad_widgets::*;

use crate::mp::menubar::Bar;

/// The strip's height.
///
/// 24 points: a strip *inside* the window, shorter than a window's own title bar, because a menubar line is a line of text
/// with a little air and not a control row. It is deliberately not [`crate::mp::titlebar::TITLEBAR_HEIGHT`]: a window's
/// chrome has to match the platform's chrome, while a strip the app draws for itself is its own.
pub const STRIP_HEIGHT: f64 = 24.0;

/// A title's horizontal padding.
pub const TITLE_PAD_X: f64 = 10.0;

/// A title's minimum width.
///
/// A one-letter title — a strip with a "File" and an "À" — would otherwise be a two-point target. Twelve points is small
/// enough to stay out of the way and wide enough to press.
pub const TITLE_MIN: f64 = 12.0;

/// How far below the strip a panel's top edge sits, so the panel does not touch the strip's own bottom border.
pub const PANEL_GAP: f64 = 2.0;

/// The width of one title, from its text's measured width.
pub fn title_width(text_width: f64) -> f64 {
    let text = if text_width.is_finite() && text_width > 0.0 {
        text_width
    } else {
        0.0
    };
    text.max(TITLE_MIN) + TITLE_PAD_X * 2.0
}

/// Where each title starts, given their widths — so `x[i]` is title `i`'s leading edge and `x[last] + w[last]` is the
/// strip's content width.
pub fn title_origins(widths: &[f64]) -> Vec<f64> {
    let mut origins = Vec::with_capacity(widths.len());
    let mut x = 0.0;
    for width in widths {
        origins.push(x);
        // A width that is not a number advances by nothing rather than poisoning every title after it — a `NaN` here would
        // put the whole strip's geometry out of reach.
        x += if width.is_finite() && *width > 0.0 { *width } else { 0.0 };
    }
    origins
}

/// Which title an `x` lands on, `x` being relative to the strip's leading edge.
///
/// **Half-open on both sides**, so the point exactly on two titles' shared edge belongs to the second — which is the one the
/// pointer is entering — and a point one past the last title's end belongs to none. A non-finite `x` is `None`.
pub fn title_at(x: f64, widths: &[f64]) -> Option<usize> {
    if !x.is_finite() || x < 0.0 {
        return None;
    }
    let mut edge = 0.0;
    for (index, width) in widths.iter().enumerate() {
        let width = if width.is_finite() && *width > 0.0 { *width } else { 0.0 };
        if width == 0.0 {
            // A zero-width title can never be the answer, and leaving it in the walk would put the shared edge on the wrong
            // side of it.
            continue;
        }
        if x < edge + width {
            return Some(index);
        }
        edge += width;
    }
    None
}

/// What a strip reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpMenubarStripAction {
    /// A title was clicked. The caller feeds this to `Bar::toggle`.
    Toggle(usize),
    /// The pointer is on a title. The caller feeds this to `Bar::hover_switch`, **which ignores it unless a menu is already
    /// down** — that asymmetry is the bar's rule and lives in the bar, not here.
    Hover(usize),
    /// The pointer left every title.
    HoverOut,
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.DrawMpMenubarStrip = #(DrawMpMenubarStrip::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 0.0
        plate: #x00000000
        open_color: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.0, 0.0, sz.x, sz.y, self.radius)
            sdf.fill_keep(mix(self.plate, self.open_color, self.live))
            return sdf.result
        }
    }

    mod.mp.MpMenubarStripBase = #(MpMenubarStrip::register_widget(vm))

    mod.mp.MpMenubarStrip = set_type_default() do mod.mp.MpMenubarStripBase{
        width: Fit
        height: 24.0

        draw_title +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text}
    }
}

/// The strip's own plate, with a live flag for the open or hovered title.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpMenubarStrip {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    live: f32,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
    #[live]
    open_color: Vec4f,
}

/// A row of titles that reports what the pointer did to it.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpMenubarStrip {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    /// The animator the control signals need — see `mp/titlebar.rs` on why a non-animated widget still carries one.
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawMpMenubarStrip,
    #[live]
    draw_title: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// The titles, which are the caller's.
    #[rust]
    titles: Vec<String>,
    /// Which title is down, and which the pointer is on. Drawn, not decided here.
    #[rust]
    open: Option<usize>,
    #[rust]
    hovered: Option<usize>,
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl MpMenubarStrip {
    pub fn set_titles(&mut self, cx: &mut Cx, titles: &[String]) {
        self.titles = titles.to_vec();
        self.open = None;
        self.hovered = None;
        self.redraw(cx);
    }

    pub fn titles(&self) -> &[String] {
        &self.titles
    }

    /// Tell the strip which menu is down, so it can draw that title as open.
    pub fn set_open(&mut self, cx: &mut Cx, open: Option<usize>) {
        if self.open != open {
            self.open = open;
            self.redraw(cx);
        }
    }

    pub fn open(&self) -> Option<usize> {
        self.open
    }

    fn widths(&self, cx: &mut Cx) -> Vec<f64> {
        let draw = &self.draw_title;
        self.titles
            .iter()
            .map(|title| title_width(crate::mp::text::measured_width(draw, cx, title)))
            .collect()
    }

    /// The strip's content width for the current titles.
    pub fn content_width(&self, cx: &mut Cx) -> f64 {
        self.widths(cx).iter().sum()
    }

    /// Where title `index` sits in screen coordinates — **what a caller anchors that menu's panel to.**
    pub fn title_rect(&self, cx: &mut Cx, index: usize) -> Option<Rect> {
        let widths = self.widths(cx);
        if index >= widths.len() {
            return None;
        }
        let origins = title_origins(&widths);
        Some(Rect {
            pos: dvec2(self.origin.x + origins[index], self.origin.y),
            size: dvec2(widths[index], STRIP_HEIGHT),
        })
    }

    /// Which title a screen `x` lands on.
    pub fn title_at_x(&self, cx: &mut Cx, x: f64) -> Option<usize> {
        title_at(x - self.origin.x, &self.widths(cx))
    }

    /// The state [`Bar`] would set this strip to, so a caller can draw without threading it by hand.
    pub fn sync_from(&mut self, cx: &mut Cx, bar: &Bar) {
        self.set_open(cx, bar.open());
    }
}

impl Widget for MpMenubarStrip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = crate::mp::control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        let hovered = match (event, signals.hover_out) {
            (_, true) => Some(None),
            (Event::MouseMove(me), _) => {
                let widths = self.widths(cx);
                Some(title_at(me.abs.x - self.origin.x, &widths).filter(|_| self.contains_x(cx, me.abs.x)))
            }
            _ => None,
        };
        if let Some(hovered) = hovered {
            if self.hovered != hovered {
                self.hovered = hovered;
                match hovered {
                    Some(index) => cx.widget_action(self.uid, MpMenubarStripAction::Hover(index)),
                    None => cx.widget_action(self.uid, MpMenubarStripAction::HoverOut),
                }
                self.redraw(cx);
            }
        }
        if signals.down {
            if let Some(index) = self.title_at_x(cx, signals.pointer.x) {
                cx.widget_action(self.uid, MpMenubarStripAction::Toggle(index));
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (plate, open_color, ink, muted) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (
                Vec4f::default(),
                theme.paint.element_active,
                theme.paint.text,
                theme.paint.text_muted,
            )
        };
        // The strip measures itself: a self-drawing widget has no intrinsic size, and the width is its titles.
        let widths = self.widths(cx.cx);
        let width: f64 = widths.iter().sum();
        let placed = cx.walk_turtle(Walk {
            width: Size::Fixed(width),
            height: Size::Fixed(STRIP_HEIGHT),
            ..walk
        });
        self.origin = placed.pos;
        self.area = self.draw_bg.area();

        let origins = title_origins(&widths);
        let line = makepad_theme::Theme::of(cx.cx).layout.row_height;
        let baseline = placed.pos.y + (STRIP_HEIGHT - line as f64) * 0.5;
        for (index, title) in self.titles.iter().enumerate() {
            // **Its own plate per title**, drawn at that title's rect: the open title is marked where it is, which is what
            // connects it to the panel hanging below it.
            let rect = Rect {
                pos: dvec2(placed.pos.x + origins[index], placed.pos.y),
                size: dvec2(widths[index], STRIP_HEIGHT),
            };
            let live = self.open == Some(index) || (self.open.is_none() && self.hovered == Some(index));
            self.draw_bg.live = if live { 1.0 } else { 0.0 };
            self.draw_bg.plate = plate;
            self.draw_bg.open_color = open_color;
            self.draw_bg.radius = 4.0;
            self.draw_bg.draw_abs(cx, rect);
            // An open title is full ink; a closed one is muted until it is the live one, which is how a menubar reads as
            // one control rather than five labels.
            self.draw_title.color = if live { ink } else { muted };
            self.draw_title
                .draw_abs(cx, dvec2(rect.pos.x + TITLE_PAD_X, baseline), title);
        }
        DrawStep::done()
    }
}

impl MpMenubarStrip {
    fn contains_x(&self, cx: &mut Cx, x: f64) -> bool {
        let width = self.content_width(cx);
        x >= self.origin.x && x < self.origin.x + width
    }
}

impl MpMenubarStripRef {
    pub fn set_titles(&self, cx: &mut Cx, titles: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_titles(cx, titles);
        }
    }

    pub fn set_open(&self, cx: &mut Cx, open: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_open(cx, open);
        }
    }

    pub fn sync_from(&self, cx: &mut Cx, bar: &Bar) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.sync_from(cx, bar);
        }
    }

    pub fn title_rect(&self, cx: &mut Cx, index: usize) -> Option<Rect> {
        self.borrow_mut().and_then(|mut inner| inner.title_rect(cx, index))
    }

    pub fn titles(&self) -> Vec<String> {
        self.borrow().map(|inner| inner.titles.clone()).unwrap_or_default()
    }

    /// Which title the strip is drawing as open.
    pub fn open(&self) -> Option<usize> {
        self.borrow().and_then(|inner| inner.open())
    }

    /// The strip's content width for the current titles.
    pub fn content_width(&self, cx: &mut Cx) -> f64 {
        self.borrow_mut()
            .map(|mut inner| inner.content_width(cx))
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_title_has_a_minimum_width_so_a_short_one_is_still_a_target() {
        // A strip with a "File" and an "À" would otherwise give the second a two-point target. Twelve points is small enough
        // to stay out of the way and wide enough to press.
        assert_eq!(title_width(40.0), 40.0 + TITLE_PAD_X * 2.0);
        assert_eq!(title_width(0.0), TITLE_MIN + TITLE_PAD_X * 2.0);
        assert_eq!(title_width(2.0), TITLE_MIN + TITLE_PAD_X * 2.0, "a narrow title is padded up");
        assert!(title_width(60.0) > title_width(40.0), "a wider title is wider");
        // Nonsense in is a minimum out, not a `NaN` width that would poison the strip.
        assert_eq!(title_width(f64::NAN), TITLE_MIN + TITLE_PAD_X * 2.0);
        assert_eq!(title_width(-5.0), TITLE_MIN + TITLE_PAD_X * 2.0);
        assert_eq!(title_width(f64::INFINITY), TITLE_MIN + TITLE_PAD_X * 2.0);
    }

    #[test]
    fn test_the_origins_are_cumulative_and_a_bad_width_does_not_shift_the_rest() {
        // **The property that keeps a title's rect and the hit test from disagreeing**: the origins are the running sum of
        // the widths, so title `i`'s origin plus its width is title `i + 1`'s origin.
        let widths = [60.0, 40.0, 50.0];
        assert_eq!(title_origins(&widths), vec![0.0, 60.0, 100.0]);
        for i in 0..widths.len() - 1 {
            assert_eq!(
                title_origins(&widths)[i] + widths[i],
                title_origins(&widths)[i + 1],
                "title {i} and {} do not meet",
                i + 1
            );
        }
        assert_eq!(title_origins(&[]), Vec::<f64>::new());
        // A width that is not a number advances by nothing rather than poisoning everything after it — a `NaN` origin would
        // put the whole strip's geometry out of reach.
        let poisoned = title_origins(&[60.0, f64::NAN, 50.0]);
        assert_eq!(poisoned, vec![0.0, 60.0, 60.0]);
        assert!(poisoned.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn test_a_point_lands_on_one_title_and_the_shared_edge_belongs_to_the_second() {
        // **Half-open on both sides**, so the point exactly on two titles' shared edge belongs to the one the pointer is
        // entering — and one point past the last title belongs to none, which is what makes the strip's own end an end.
        let widths = [60.0, 40.0, 50.0];
        assert_eq!(title_at(0.0, &widths), Some(0));
        assert_eq!(title_at(59.9, &widths), Some(0));
        assert_eq!(title_at(60.0, &widths), Some(1), "the shared edge belongs to the second");
        assert_eq!(title_at(99.9, &widths), Some(1));
        assert_eq!(title_at(100.0, &widths), Some(2));
        assert_eq!(title_at(149.9, &widths), Some(2));
        assert_eq!(title_at(150.0, &widths), None, "one past the last title");
        assert_eq!(title_at(1000.0, &widths), None);
        assert_eq!(title_at(-1.0, &widths), None);
        assert_eq!(title_at(f64::NAN, &widths), None, "every comparison against NaN is false");
        assert_eq!(title_at(0.0, &[]), None, "an empty strip holds nothing");
    }

    #[test]
    fn test_a_zero_width_title_can_never_be_the_one_landed_on() {
        // A title that measures nothing is not a target, and leaving it in the walk would put its shared edge on the wrong
        // side — so the title after it would start one point early.
        let widths = [60.0, 0.0, 50.0];
        assert_eq!(title_at(0.0, &widths), Some(0));
        assert_eq!(title_at(59.0, &widths), Some(0));
        assert_eq!(title_at(60.0, &widths), Some(2), "the empty title was skipped, not landed on");
        assert_eq!(title_at(109.0, &widths), Some(2));
        assert_eq!(title_at(110.0, &widths), None);
        // ...which is also where the origins say it starts, so the two agree.
        assert_eq!(title_origins(&widths), vec![0.0, 60.0, 60.0]);
    }

    #[test]
    fn test_every_title_is_reachable_and_the_reachable_span_is_exactly_the_strip() {
        // **The property that matters**: for any set of widths, aiming at each title's centre lands on that title, and the
        // union of the reachable points is exactly `[0, content width)` — no gap and no point outside.
        for widths in [
            vec![60.0, 40.0],
            vec![12.0, 12.0, 12.0],
            vec![100.0],
            vec![60.0, 0.0, 40.0, 90.0],
        ] {
            let origins = title_origins(&widths);
            let total: f64 = widths.iter().sum();
            let mut reachable = 0.0;
            for (index, width) in widths.iter().enumerate() {
                if *width == 0.0 {
                    continue;
                }
                let centre = origins[index] + width * 0.5;
                assert_eq!(title_at(centre, &widths), Some(index), "centre of {index} missed");
                reachable += width;
            }
            assert_eq!(reachable, total, "some of the strip is not reachable");
            // The far edge is outside and the point just under it is inside.
            assert_eq!(title_at(total, &widths), None);
            let last = widths.iter().rposition(|w| *w > 0.0).expect("a title");
            assert_eq!(title_at(total - 0.1, &widths), Some(last));
        }
    }

    #[test]
    fn test_the_panel_gap_is_a_hair_and_the_strip_is_not_a_title_bar() {
        // The two numbers with provenance, asserted so neither drifts into the other: a panel sits a hair below the strip
        // rather than touching it, and the strip is shorter than a window's title bar because it is a line of text rather
        // than a control row.
        assert_eq!(STRIP_HEIGHT, 24.0);
        assert!(PANEL_GAP > 0.0 && PANEL_GAP <= 4.0, "the gap is a hair, not a space");
        assert!(
            STRIP_HEIGHT < crate::mp::titlebar::TITLEBAR_HEIGHT,
            "a strip the app draws is shorter than the platform's chrome"
        );
    }
}
