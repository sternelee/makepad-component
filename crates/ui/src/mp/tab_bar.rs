//! `MpTabBar` — a strip of tabs, one of them current, with a close and an add.
//!
//! ## The interesting part is the widths, so the widths are functions
//!
//! A tab bar's whole job is an allocation: `n` titles have to fit an available width, each wants to be wide enough to read
//! and no wider than [`MAX_TAB`], and when there are too many the strip has to overflow and scroll rather than squeeze every
//! tab down to a sliver. All of that is arithmetic on `(count, available)` and none of it needs a window, so it is
//! [`layout`] and it is tested.
//!
//! - **Between [`MIN_TAB`] and [`MAX_TAB`], tabs share the width equally.** Equal shares rather than proportional to their
//!   titles: a bar of equal tabs reads as a set, while one whose widths follow its labels reads as ragged, and a long title
//!   does not need more room than a short one to be recognised once both are truncated anyway.
//! - **Under pressure they all take [`MIN_TAB`] and the strip overflows.** A tab narrower than that is a tab whose title is
//!   unreadable, and a bar that squeezed ten tabs into the window would be showing ten unusable ones. Overflowing is the
//!   honest failure: the caller scrolls, and [`scroll_to_show`] is how the current tab stays visible in it.
//! - **The add button is not a tab.** It is [`ADD_W`] wide, it never shrinks, and it is not part of the equal share — a plus
//!   that changed width as tabs came and went would move under the pointer.
//!
//! ## Why this exists rather than being left to callers
//!
//! Two applications in this workspace grew their own tab bars, one of them 439 lines, and both wanted the same three things
//! this module is: widths that fit, a close region that does not overlap the title, and a current tab that stays on screen
//! when the strip scrolls. A component library that leaves each of those to its callers has not got a tab bar in it.
//!
//! ## The close region is subtracted, not overlaid
//!
//! [`close_rect`] sits inside the tab's trailing edge and the **title's** width is the tab minus it — so a click meant for
//! the title cannot land on the close button, which is the same rule the title bar's control region follows. A close
//! button drawn over a title is a button that closes the wrong tab.

use makepad_widgets::*;

/// The narrowest a tab may be, before the strip overflows instead.
pub const MIN_TAB: f64 = 120.0;

/// The widest a tab grows to when there is room.
pub const MAX_TAB: f64 = 200.0;

/// The trailing add button's width.
///
/// Fixed, and outside the equal share: a plus that changed width as tabs came and went would move under the pointer.
pub const ADD_W: f64 = 34.0;

/// A tab's height.
pub const TAB_HEIGHT: f64 = 36.0;

/// How much of a tab's trailing edge the close button occupies.
pub const CLOSE_W: f64 = 22.0;

/// One tab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    pub title: String,
    /// Whether it can be closed. **A tab that cannot is drawn without a close button and reports no close**, so an
    /// unclosable tab and a closable one differ in what they do rather than only in how they look.
    pub closable: bool,
}

impl Tab {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            closable: true,
        }
    }

    /// The same, not closable.
    pub fn fixed(mut self) -> Self {
        self.closable = false;
        self
    }
}

/// How wide each tab is when `count` of them share `available`.
///
/// Zero tabs have no width. Under `count * MIN_TAB` the answer is [`MIN_TAB`] — the strip overflows and the caller scrolls,
/// rather than squeezing titles into widths that cannot show them.
pub fn tab_width(count: usize, available: f64) -> f64 {
    if count == 0 || !available.is_finite() || available <= 0.0 {
        return 0.0;
    }
    let share = available / count as f64;
    if share < MIN_TAB {
        MIN_TAB
    } else {
        share.min(MAX_TAB)
    }
}

/// A laid-out strip: every tab's width, the add button, and the whole content width.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Strip {
    pub widths: Vec<f64>,
    /// Whether the add button is present, and how wide it is.
    pub add: f64,
    /// The content width — the tabs plus the add button. **Compare this with the available width to know whether it
    /// overflows**, rather than recomputing the same sum at each call site.
    pub content: f64,
}

impl Strip {
    pub fn count(&self) -> usize {
        self.widths.len()
    }

    /// Whether the content is wider than the space it was laid out for.
    pub fn overflows(&self, available: f64) -> bool {
        self.content > available
    }

    /// Where each tab starts, the add button included as one more position.
    pub fn origins(&self) -> Vec<f64> {
        let mut origins = Vec::with_capacity(self.widths.len() + 1);
        let mut x = 0.0;
        for width in &self.widths {
            origins.push(x);
            x += width;
        }
        if self.add > 0.0 {
            origins.push(x);
        }
        origins
    }

    /// The width of each tab **minus its close region**, which is what a title may occupy.
    pub fn title_widths(&self) -> Vec<f64> {
        self.widths
            .iter()
            .map(|width| (width - CLOSE_W).max(0.0))
            .collect()
    }
}

/// Lay out `tabs` across `available`, with the add button when `show_add`.
pub fn layout(tabs: &[Tab], available: f64, show_add: bool) -> Strip {
    let add = if show_add { ADD_W } else { 0.0 };
    // **The add button is taken off the top**, so the tabs share what is actually left for them rather than shrinking to
    // make room for a button that is not one.
    let for_tabs = if available.is_finite() { (available - add).max(0.0) } else { 0.0 };
    let width = tab_width(tabs.len(), for_tabs);
    let widths = vec![width; tabs.len()];
    let content = width * tabs.len() as f64 + add;
    Strip {
        widths,
        add,
        content,
    }
}

/// Which tab an `x` (relative to the strip's leading edge, already scrolled) lands on.
///
/// **Only tabs**: an `x` on the add button is `None`, so a click there is the caller's `add_clicked` rather than a tab
/// selection. `None` for a point past the end and for a non-finite one — the rule every hit test in this port carries.
pub fn tab_at(x: f64, strip: &Strip) -> Option<usize> {
    if !x.is_finite() || x < 0.0 {
        return None;
    }
    let mut edge = 0.0;
    for (index, width) in strip.widths.iter().enumerate() {
        if *width <= 0.0 {
            continue;
        }
        if x < edge + width {
            return Some(index);
        }
        edge += width;
    }
    None
}

/// Whether an `x` is on the add button.
pub fn on_add(x: f64, strip: &Strip) -> bool {
    if !x.is_finite() || strip.add <= 0.0 {
        return false;
    }
    let start: f64 = strip.widths.iter().sum();
    x >= start && x < start + strip.add
}

/// The close button's rect inside a tab, for a tab that can be closed.
///
/// Inside the tab's trailing edge, vertically centred. `None` for a tab that cannot be closed — which is what makes
/// "unclosable" a difference in behaviour rather than in paint.
pub fn close_rect(tab: &Tab, x: f64, width: f64, y: f64) -> Option<Rect> {
    if !tab.closable || !width.is_finite() || width < CLOSE_W {
        return None;
    }
    Some(Rect {
        pos: dvec2(x + width - CLOSE_W, y + (TAB_HEIGHT - CLOSE_W) * 0.5),
        size: dvec2(CLOSE_W, CLOSE_W),
    })
}

/// How far the strip must scroll for tab `active` to be on screen.
///
/// **The current tab stays visible when the strip overflows** — that is the whole reason a scrolling tab bar exists, and it
/// is the rule a caller with ten open tabs notices only when it is missing: you click a tab, and the bar scrolls away from
/// it. Returns the offset to scroll to, never negative and never past the content's end.
pub fn scroll_to_show(active: usize, strip: &Strip, available: f64) -> f64 {
    if !available.is_finite() || available <= 0.0 || strip.overflows(available) == false {
        return 0.0;
    }
    let origins = strip.origins();
    let Some(&start) = origins.get(active) else {
        return 0.0;
    };
    let Some(&width) = strip.widths.get(active) else {
        return 0.0;
    };
    let end = start + width;
    // Scrolled as little as possible: bring the far edge into view if it is past it, the near edge if it is before it, and
    // otherwise leave the strip alone.
    let offset = if end > available { end - available } else { 0.0 };
    let max = (strip.content - available).max(0.0);
    let candidate = if start < offset { start } else { offset };
    candidate.clamp(0.0, max)
}

/// `text` cut to fit `width`, with an ellipsis when anything was cut.
///
/// **Measured, not estimated.** This could have used the port's width *estimator* on the grounds that cutting a title short
/// does not need to be exact — but the exact measure is already here (`text::measured_width`), so there is no reason to spend
/// accuracy on it, and a clip that is right for the face it is drawn in beats one that is right for an average character.
pub fn clip_to(text: &str, width: f64, cx: &mut Cx, draw: &DrawText) -> String {
    if !width.is_finite() || width <= 0.0 {
        return String::new();
    }
    if crate::mp::text::measured_width(draw, cx, text) <= width {
        return text.to_string();
    }
    let mut out = String::new();
    for character in text.chars() {
        let mut candidate = out.clone();
        candidate.push(character);
        candidate.push('\u{2026}');
        if crate::mp::text::measured_width(draw, cx, &candidate) > width {
            break;
        }
        out.push(character);
    }
    if out.is_empty() {
        // Nothing fits, not even the ellipsis on its own: an empty label is honest, a single glyph running past the tab is
        // not.
        return String::new();
    }
    out.push('\u{2026}');
    out
}

/// What a tab bar reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpTabBarAction {
    /// A tab was chosen, by click or by the keyboard.
    Activated(usize),
    /// A closable tab's close button was pressed.
    Closed(usize),
    /// The trailing add button was pressed.
    AddClicked,
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// A tab's plate. `active` is the raised current tab; `hover` brightens an inactive one.
    mod.mp.DrawMpTabPlate = #(DrawMpTabPlate::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 6.0
        plate: #x00000000
        active_plate: #x00000000
        hover_color: #x00000000
        underline: #x00000000
        active: 0.0
        hover: 0.0
        underline_width: 2.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.0, 0.0, sz.x, sz.y - 2.0, self.radius)
            sdf.fill_keep(mix(mix(self.plate, self.hover_color, self.hover), self.active_plate, self.active))
            // A rule under the current tab, which is what marks it when the plate is too subtle to read.
            if (self.active > 0.5) {
                sdf.box(0.0, sz.y - self.underline_width, sz.x, self.underline_width, 0.0)
                sdf.fill(self.underline)
            }
            return sdf.result
        }
    }

    mod.mp.MpTabBarBase = #(MpTabBar::register_widget(vm))

    mod.mp.MpTabBar = set_type_default() do mod.mp.MpTabBarBase{
        width: Fill
        height: 36.0

        draw_label +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text}
        draw_close +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_muted}
        draw_add +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text_muted}
    }
}

/// A tab's plate.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpTabPlate {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
    #[live]
    active_plate: Vec4f,
    #[live]
    hover_color: Vec4f,
    #[live]
    underline: Vec4f,
    #[live]
    active: f32,
    #[live]
    hover: f32,
    #[live]
    underline_width: f32,
}

/// A strip of tabs, one of them current.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpTabBar {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_plate: DrawMpTabPlate,
    #[live]
    draw_label: DrawText,
    #[live]
    draw_close: DrawText,
    #[live]
    draw_add: DrawText,
    /// Whether the trailing add button is drawn.
    #[live]
    show_add: bool,
    /// How far the strip is scrolled. The caller may set it; [`MpTabBar::show_tab`] computes it.
    #[live]
    offset: f64,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    tabs: Vec<Tab>,
    #[rust]
    active: Option<usize>,
    #[rust]
    hovered: Option<usize>,
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl MpTabBar {
    /// Replace the tabs.
    pub fn set_tabs(&mut self, cx: &mut Cx, tabs: Vec<Tab>) {
        self.tabs = tabs;
        // A current tab that the new list does not cover is dropped, not clamped — the rule every list in this port keeps,
        // for the same reason: a bar showing the fourth tab as current when it has three would be naming a tab nobody can
        // choose.
        if self.active.is_some_and(|active| active >= self.tabs.len()) {
            self.active = None;
        }
        self.redraw(cx);
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn set_active(&mut self, cx: &mut Cx, active: Option<usize>) {
        self.active = active.filter(|index| *index < self.tabs.len());
        self.redraw(cx);
    }

    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// Scroll the strip as little as possible to put `index` on screen.
    pub fn show_tab(&mut self, cx: &mut Cx, index: usize, available: f64) {
        let strip = layout(&self.tabs, available, self.show_add);
        let offset = scroll_to_show(index, &strip, available);
        if offset != self.offset {
            self.offset = offset;
            self.redraw(cx);
        }
    }

    pub fn strip(&self, available: f64) -> Strip {
        layout(&self.tabs, available, self.show_add)
    }
}

impl Widget for MpTabBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = crate::mp::control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        let available = self.area.rect(cx).size.x;
        let strip = self.strip(available);
        if signals.down {
            // The strip's own coordinates are the pointer's minus the origin **plus the scroll offset**, so a click lands
            // on the tab it is over rather than on the tab that would be there unscrolled.
            let x = signals.pointer.x - self.origin.x + self.offset;
            if on_add(x, &strip) {
                cx.widget_action(self.uid, MpTabBarAction::AddClicked);
                return;
            }
            if let Some(index) = tab_at(x, &strip) {
                // **The close button first.** It sits inside the tab, so a press there is the close and not the selection —
                // and the title's own width is the tab minus the close region, which is what makes that unambiguous.
                if let Some(close) = self
                    .tabs
                    .get(index)
                    .and_then(|tab| close_rect(tab, 0.0, strip.widths[index], 0.0))
                {
                    let local = x - (strip.origins()[index]);
                    if local >= close.pos.x {
                        if self.tabs[index].closable {
                            cx.widget_action(self.uid, MpTabBarAction::Closed(index));
                        }
                        return;
                    }
                }
                self.active = Some(index);
                self.redraw(cx);
                cx.widget_action(self.uid, MpTabBarAction::Activated(index));
            }
            return;
        }
        if let Event::MouseMove(me) = event {
            let x = me.abs.x - self.origin.x + self.offset;
            let hovered = tab_at(x, &strip);
            if self.hovered != hovered {
                self.hovered = hovered;
                self.redraw(cx);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (plate, active_plate, hover_color, underline, ink, muted) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (
                Vec4f::default(),
                theme.paint.surface_raised,
                theme.paint.element_hover,
                theme.paint.accent,
                theme.paint.text,
                theme.paint.text_muted,
            )
        };
        let placed = cx.walk_turtle(walk);
        self.origin = placed.pos;
        self.area = self.draw_plate.area();
        let available = placed.size.x;
        let strip = self.strip(available);
        let origins = strip.origins();
        let titles = strip.title_widths();
        let y = placed.pos.y;

        for (index, tab) in self.tabs.iter().enumerate() {
            // Scrolled: a tab entirely outside the visible span is **not drawn at all**, which is what keeps a hundred-tab
            // bar from costing a hundred draws.
            let x = placed.pos.x + origins[index] - self.offset;
            let width = strip.widths[index];
            if x + width < placed.pos.x || x > placed.pos.x + available {
                continue;
            }
            self.draw_plate.plate = plate;
            self.draw_plate.active_plate = active_plate;
            self.draw_plate.hover_color = hover_color;
            self.draw_plate.underline = underline;
            self.draw_plate.radius = 6.0;
            self.draw_plate.underline_width = 2.0;
            self.draw_plate.active = if self.active == Some(index) { 1.0 } else { 0.0 };
            self.draw_plate.hover = if self.hovered == Some(index) { 1.0 } else { 0.0 };
            self.draw_plate.draw_abs(
                cx,
                Rect {
                    pos: dvec2(x, y),
                    size: dvec2(width, TAB_HEIGHT),
                },
            );
            self.draw_label.color = if self.active == Some(index) { ink } else { muted };
            // **The title is truncated to its own width** — the tab minus the close region — so a long title cannot run
            // under the close button or into the next tab. This is the one job the port's width *estimator* is allowed to
            // do: measuring something to cut it short does not need to be exact, while positioning does.
            let clipped = clip_to(&tab.title, titles[index], cx.cx, &self.draw_label);
            self.draw_label.draw_abs(cx, dvec2(x + 10.0, y + 10.0), &clipped);
            if let Some(close) = close_rect(tab, x, width, y) {
                self.draw_close.color = muted;
                self.draw_close.draw_abs(cx, close.pos + dvec2(6.0, 3.0), "\u{00D7}");
            }
        }
        if strip.add > 0.0 {
            let x = placed.pos.x + origins[origins.len() - 1] - self.offset;
            self.draw_add.color = muted;
            self.draw_add.draw_abs(cx, dvec2(x + 11.0, y + 10.0), "+");
        }
        DrawStep::done()
    }
}

impl MpTabBarRef {
    pub fn set_tabs(&self, cx: &mut Cx, tabs: Vec<Tab>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_tabs(cx, tabs);
        }
    }

    pub fn set_active(&self, cx: &mut Cx, active: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_active(cx, active);
        }
    }

    pub fn active(&self) -> Option<usize> {
        self.borrow().and_then(|inner| inner.active())
    }

    /// The tabs, for a caller reading the bar back.
    pub fn tabs(&self) -> Vec<Tab> {
        self.borrow().map(|inner| inner.tabs.clone()).unwrap_or_default()
    }

    /// The laid-out strip for an available width, so a caller can ask the same question the painter asks.
    pub fn strip(&self, available: f64) -> Strip {
        self.borrow().map(|inner| inner.strip(available)).unwrap_or_default()
    }

    pub fn activated(&self, actions: &Actions) -> Option<usize> {
        actions
            .find_widget_action(self.widget_uid())
            .and_then(|action| match action.cast::<MpTabBarAction>() {
                MpTabBarAction::Activated(index) => Some(index),
                _ => None,
            })
    }

    pub fn closed(&self, actions: &Actions) -> Option<usize> {
        actions
            .find_widget_action(self.widget_uid())
            .and_then(|action| match action.cast::<MpTabBarAction>() {
                MpTabBarAction::Closed(index) => Some(index),
                _ => None,
            })
    }

    pub fn add_clicked(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .is_some_and(|action| matches!(action.cast::<MpTabBarAction>(), MpTabBarAction::AddClicked))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabs(count: usize) -> Vec<Tab> {
        (0..count).map(|index| Tab::new(format!("Tab {}", index + 1))).collect()
    }

    #[test]
    fn test_tabs_share_the_width_equally_between_the_minimum_and_the_maximum() {
        // Equal shares rather than proportional to the titles: a bar of equal tabs reads as a set, and a long title does not
        // need more room than a short one once both are truncated.
        assert_eq!(tab_width(4, 800.0), 200.0);
        assert_eq!(tab_width(4, 600.0), 150.0);
        assert_eq!(tab_width(2, 800.0), MAX_TAB, "wide tabs stop growing");
        // **A share below the minimum is clamped *up*, not taken.** 800/8 is 100, which is narrower than a tab may be, so
        // the answer is the minimum and the strip overflows — the rule this module documents. I first wrote `100.0` here,
        // contradicting the rule in the same file.
        assert_eq!(tab_width(8, 800.0), MIN_TAB);
        assert_eq!(tab_width(8, 800.0 * 2.0), 200.0, "with room, the share is taken");
        // ...and at exactly the minimum.
        assert_eq!(tab_width(4, MIN_TAB * 4.0), MIN_TAB);
    }

    #[test]
    fn test_under_pressure_every_tab_takes_the_minimum_and_the_strip_overflows() {
        // **The honest failure**: a tab narrower than the minimum cannot show its title, so ten tabs in a small window
        // overflow and the caller scrolls rather than showing ten unusable slivers.
        let strip = layout(&tabs(10), 800.0, false);
        assert_eq!(strip.count(), 10);
        assert!(strip.widths.iter().all(|width| *width == MIN_TAB));
        assert_eq!(strip.content, MIN_TAB * 10.0);
        assert!(strip.overflows(800.0), "the strip did not overflow");
        assert!((strip.content - 1200.0).abs() < 1e-9);
    }

    #[test]
    fn test_the_add_button_is_not_a_tab_nor_part_of_the_share() {
        // It is taken off the top, so the tabs share what is actually left — and it never shrinks, because a plus that
        // changed width as tabs came and went would move under the pointer.
        let without = layout(&tabs(4), 800.0, false);
        let with = layout(&tabs(4), 800.0, true);
        assert_eq!(without.add, 0.0);
        assert_eq!(with.add, ADD_W);
        // The tabs are laid out against the width *minus* the button.
        assert_eq!(with.widths[0], tab_width(4, 800.0 - ADD_W));
        assert!(with.widths[0] < without.widths[0], "the add button did not take space from the tabs");
        assert!((with.content - (with.widths[0] * 4.0 + ADD_W)).abs() < 1e-9);
        // And it is reachable as its own region rather than as a tab.
        let x = with.widths.iter().sum::<f64>() + 1.0;
        assert!(on_add(x, &with));
        assert_eq!(tab_at(x, &with), None, "the add button was hit as a tab");
    }

    #[test]
    fn test_a_point_lands_on_one_tab_and_the_add_button_is_not_one() {
        let strip = layout(&tabs(4), 800.0, true);
        // Each tab's centre lands on it.
        let origins = strip.origins();
        for index in 0..strip.count() {
            let centre = origins[index] + strip.widths[index] * 0.5;
            assert_eq!(tab_at(centre, &strip), Some(index), "the centre of {index} missed");
        }
        // The shared edges belong to the tab being entered, and one past the last tab is the add button.
        assert_eq!(tab_at(strip.widths[0], &strip), Some(1));
        assert_eq!(tab_at(strip.widths[0] - 0.1, &strip), Some(0));
        assert_eq!(tab_at(strip.widths.iter().sum::<f64>(), &strip), None);
        // Nonsense points are nowhere.
        assert_eq!(tab_at(f64::NAN, &strip), None);
        assert_eq!(tab_at(-1.0, &strip), None);
        assert!(!on_add(f64::NAN, &strip));
        assert_eq!(tab_at(0.0, &layout(&[], 800.0, false)), None);
    }

    #[test]
    fn test_a_tab_that_cannot_be_closed_has_no_close_region_at_all() {
        // **A difference in behaviour rather than in paint**: an unclosable tab reports no close, so a click near its
        // trailing edge selects it instead. A close button drawn but not honoured would be worse than none.
        let closable = Tab::new("Query 1");
        let fixed = Tab::new("Home").fixed();
        assert!(close_rect(&closable, 0.0, MAX_TAB, 0.0).is_some());
        assert!(close_rect(&fixed, 0.0, MAX_TAB, 0.0).is_none());
        assert!(close_rect(&closable, 0.0, CLOSE_W - 1.0, 0.0).is_none(), "a tab narrower than its close button");
        // The close sits inside the tab's trailing edge, and the title gets what is left.
        let rect = close_rect(&closable, 100.0, MAX_TAB, 0.0).expect("closable");
        assert_eq!(rect.pos.x + rect.size.x, 100.0 + MAX_TAB, "the close button hangs off its tab");
        let strip = layout(&[closable], 800.0, false);
        assert_eq!(strip.title_widths()[0], strip.widths[0] - CLOSE_W);
        assert!(
            strip.title_widths()[0] < strip.widths[0],
            "the title may run under the close button"
        );
    }

    #[test]
    fn test_the_current_tab_stays_on_screen_when_the_strip_scrolls() {
        // **The rule a caller with ten open tabs notices only when it is missing**: you click a tab, and the bar has scrolled
        // away from it. The offset is the smallest that brings the tab into view, and it is bounded by the content.
        let strip = layout(&tabs(10), 800.0, false);
        let available = 800.0;
        // A tab already visible does not scroll the strip.
        assert_eq!(scroll_to_show(0, &strip, available), 0.0);
        assert_eq!(scroll_to_show(3, &strip, available), 0.0);
        // The first tab that runs off the end brings the strip just far enough.
        assert_eq!(scroll_to_show(6, &strip, available), MIN_TAB * 7.0 - available);
        // The last tab, and the bound: the offset never passes the content's own end.
        let last = scroll_to_show(9, &strip, available);
        assert!(last <= strip.content - available + 1e-9, "the strip scrolled past its own end");
        assert!(last > 0.0);
        // And the tab really is on screen at that offset.
        let origin = strip.origins()[9] - last;
        assert!(origin >= 0.0 && origin + strip.widths[9] <= available + 1e-9);
        // A strip that fits does not scroll at all, and an out-of-range index is not a reason to move it.
        let small = layout(&tabs(2), 800.0, false);
        assert_eq!(scroll_to_show(1, &small, available), 0.0);
        assert_eq!(scroll_to_show(99, &strip, available), 0.0);
        assert_eq!(scroll_to_show(0, &strip, 0.0), 0.0);
    }

    #[test]
    fn test_an_empty_bar_lays_out_nothing_and_still_offers_its_add_button() {
        let strip = layout(&[], 800.0, true);
        assert_eq!(strip.count(), 0);
        assert_eq!(strip.content, ADD_W, "an empty bar with an add keeps only the button");
        assert!(on_add(1.0, &strip), "the add button was not reachable on an empty bar");
        assert_eq!(tab_at(1.0, &strip), None);
        let bare = layout(&[], 800.0, false);
        assert_eq!(bare.content, 0.0);
        // Nonsense widths allocate nothing rather than a `NaN` every rect inherits.
        assert_eq!(tab_width(4, 0.0), 0.0);
        assert_eq!(tab_width(4, f64::NAN), 0.0);
        assert_eq!(tab_width(0, 800.0), 0.0);
        assert_eq!(layout(&tabs(3), f64::NAN, false).content, 0.0);
    }

    #[test]
    fn test_every_tab_is_reachable_and_the_strip_tiles_from_zero() {
        // The property tying the layout to the hit test: the origins start at zero, each tab begins where the last ended,
        // and aiming anywhere inside a tab lands on it — so the tab drawn and the tab a click reports agree.
        for (count, available, add) in [(1usize, 800.0f64, false), (4, 800.0, true), (10, 800.0, false), (3, 300.0, true)] {
            let strip = layout(&tabs(count), available, add);
            let origins = strip.origins();
            assert_eq!(origins[0], 0.0, "the strip does not start at zero");
            for index in 1..count {
                assert_eq!(
                    origins[index - 1] + strip.widths[index - 1],
                    origins[index],
                    "tab {} and {index} do not meet",
                    index - 1
                );
            }
            for index in 0..count {
                let start = origins[index] + 0.1;
                let end = origins[index] + strip.widths[index] - 0.1;
                assert_eq!(tab_at(start, &strip), Some(index), "the left edge of {index}");
                assert_eq!(tab_at(end, &strip), Some(index), "the right edge of {index}");
            }
        }
    }
}
