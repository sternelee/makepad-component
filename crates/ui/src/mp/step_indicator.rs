//! `MpStepIndicator` — a numbered path with the current step marked.
//!
//! ## The state of a step is a function, so it is one
//!
//! A step is one of three things: passed, current, or upcoming. That is not stored anywhere — it is `index <=> current`,
//! and [`state_of`] is where it is decided. Storing three flags per step would be a second place the same fact lives,
//! and the failure that produces is ordinary: a list where two steps say they are current, or none does.
//!
//! The connector has its own rule, and it is **not** the same rule: the line *before* step `i` is filled once step `i`
//! is reached, so a connector carries the state of the step to its right rather than its left. [`connector_passed`] is
//! that rule, and it is separate from [`state_of`] because the two would otherwise look like the same question.
//!
//! ## Eight slots, like the description list
//!
//! A fixed set of slots filled from the front. A step here is a circle, a number and a title with no identity of its
//! own, so filling and hiding eight is simpler than growing a pool — and a wizard with more than eight steps is a form
//! somebody should have split. [`SLOTS`] is public, and [`steps_shown`] is the one place the bound is applied.
//!
//! ## What is deliberately not ported
//!
//! The v2 widget animated its circle between passed, current and upcoming through an animator with `passed.on/off` and
//! `active.on/off`. **This one paints the three states directly from `Theme::of(cx)` on each frame**, which is the
//! convention the rest of this library follows and the reason an appearance change needs a redraw rather than a
//! re-declaration. The cost is that the transition is instant — a real difference, stated rather than hidden, and the
//! fix is an animator over the same two numbers rather than a different design.
//!
//! The v2 `MpSize` was dropped: a five-step size is what the theme's typography is for, and a caller choosing one is a
//! caller choosing a circle diameter without saying so.

use makepad_widgets::*;

/// How many steps the indicator can show.
pub const SLOTS: usize = 8;

/// Where a step stands, relative to the current one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepState {
    /// Behind the current step.
    Passed,
    /// The current step.
    Active,
    /// Ahead of the current step — **the default**, because a widget is constructed before anything has told it where it
    /// is on the path, and "not yet reached" is the only state that is true before that.
    #[default]
    Upcoming,
}

/// Where step `index` stands when the current step is `current`.
///
/// The whole of the state model. `current` past the end is not special-cased: every step is simply passed, which is what
/// a finished wizard looks like.
pub fn state_of(index: usize, current: usize) -> StepState {
    match index.cmp(&current) {
        std::cmp::Ordering::Less => StepState::Passed,
        std::cmp::Ordering::Equal => StepState::Active,
        std::cmp::Ordering::Greater => StepState::Upcoming,
    }
}

/// Whether the connector **before** step `index` is filled, given the current step.
///
/// The line into a step is filled once that step has been reached, so a connector carries the state of the step to its
/// right. Step `0` has no connector before it at all — there is nothing to connect it to — so it is always `false`.
pub fn connector_passed(index: usize, current: usize) -> bool {
    index > 0 && index <= current
}

/// The same bound as [`steps_shown`], named for a caller outside this crate.
///
/// A one-line alias rather than a second rule: the A2UI renderer prints it, and a print that recomputed `min` would be a
/// second place the bound is expressed.
pub fn rows_shown_for(count: usize) -> usize {
    steps_shown(count)
}

/// How many of `count` steps are shown.
///
/// The single place the bound is applied, so filling the slots and hiding the rest cannot disagree about which is last.
pub fn steps_shown(count: usize) -> usize {
    count.min(SLOTS)
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// One step: a circle with its number, and the title under it.
    ///
    /// **Self-drawing, with its own shader and its own two `DrawText`s** — the shape `mp/checkbox.rs` uses, and the
    /// reason is the convention this library follows: colours are written from Rust on every paint, and a shader owned
    /// by a child DSL `View` cannot be typed at from Rust. The first version of this component kept the circle as a
    /// child `View` and tried to reach its instance fields generically, which is not a thing the DSL exposes — so the
    /// dot became a `#[live]` field here instead.
    mod.mp.DrawMpStepDot = #(DrawMpStepDot::script_shader(vm)){
        ..mod.draw.DrawQuad

        // Written from Rust each paint, read from `Theme::of(cx)` — so an appearance change is a redraw rather than a
        // re-declaration. Same shape as `mp/button.rs`'s plate.
        passed: 0.0
        active: 0.0
        fill_solid: #x00000000
        fill_accent: #x00000000
        ring: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let r = self.rect_size.x * 0.5
            sdf.circle(r, r, r)
            let bg = mix(#x0000, self.fill_solid, self.passed)
            let bg = mix(bg, self.fill_accent, self.active)
            let border = mix(self.ring, bg, max(self.passed, self.active))
            sdf.fill(bg)
            sdf.stroke(border, 1.0)
            return sdf.result
        }
    }

    mod.mp.MpStepItemBase = #(MpStepItem::register_widget(vm))

    mod.mp.MpStepItem = set_type_default() do mod.mp.MpStepItemBase{
        width: Fit
        height: Fit
        draw_number +: {
            text_style: caption
            color: #x00000000
        }
        draw_title +: {
            text_style: caption
            color: #x00000000
        }
    }

    /// The line between two steps. A track with a filled line over it.
    mod.mp.MpStepConnector = mod.widgets.View{
        width: Fill
        height: 2
        flow: Overlay

        track := View{
            width: Fill
            height: Fill
            show_bg: true
            draw_bg +: {
                color: instance(border)
            }
        }
        checked_line := View{
            width: Fill
            height: Fill
            visible: false
            show_bg: true
            draw_bg +: {
                color: instance(solid)
            }
        }
    }

    mod.mp.MpStepIndicatorBase = #(MpStepIndicator::register_widget(vm))

    mod.mp.MpStepIndicator = set_type_default() do mod.mp.MpStepIndicatorBase{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8
        align: Align{y: 0.0}

        slot0 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector0 := mod.mp.MpStepConnector{visible: false}, item0 := mod.mp.MpStepItem{} }
        slot1 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector1 := mod.mp.MpStepConnector{}, item1 := mod.mp.MpStepItem{} }
        slot2 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector2 := mod.mp.MpStepConnector{}, item2 := mod.mp.MpStepItem{} }
        slot3 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector3 := mod.mp.MpStepConnector{}, item3 := mod.mp.MpStepItem{} }
        slot4 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector4 := mod.mp.MpStepConnector{}, item4 := mod.mp.MpStepItem{} }
        slot5 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector5 := mod.mp.MpStepConnector{}, item5 := mod.mp.MpStepItem{} }
        slot6 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector6 := mod.mp.MpStepConnector{}, item6 := mod.mp.MpStepItem{} }
        slot7 := View{ width: Fill, height: Fit, flow: Right, spacing: 8, connector7 := mod.mp.MpStepConnector{}, item7 := mod.mp.MpStepItem{} }
    }
}

/// The step's circle.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpStepDot {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    passed: f32,
    #[live]
    active: f32,
    #[live]
    fill_solid: Vec4f,
    #[live]
    fill_accent: Vec4f,
    #[live]
    ring: Vec4f,
}

/// The circle's diameter, and the gap to the title under it.
///
/// Named rather than inline, and **not** a `#[live]` field: a caller choosing a step's diameter is a caller choosing a
/// size the theme should be choosing, which is the kind of knob this port's rules forbid.
const DOT_DIAMETER: f64 = 28.0;
const TITLE_GAP: f64 = 6.0;

/// One step, drawn by itself.
#[derive(Script, ScriptHook, Widget)]
pub struct MpStepItem {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_dot: DrawMpStepDot,
    #[live]
    draw_number: DrawText,
    #[live]
    draw_title: DrawText,
    #[rust]
    state: StepState,
    /// The step's zero-based position, so the circle shows a number rather than the caller supplying one — a caller
    /// numbering its own steps is a caller that can number them wrongly.
    #[rust]
    position: usize,
    #[rust]
    title: String,
    #[walk]
    walk: Walk,
}

impl Widget for MpStepItem {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // **The theme, read this paint.** The three states are three sets of tokens rather than three declared
        // palettes, which is what lets an appearance change re-colour a step with no re-declaration.
        let (passed, active, number_ink, title_ink) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            match self.state {
                StepState::Passed => (1.0, 0.0, p.on_solid, p.text_muted),
                StepState::Active => (1.0, 1.0, p.on_accent, p.text),
                StepState::Upcoming => (0.0, 0.0, p.text_muted, p.text_faint),
            }
        };
        self.draw_dot.passed = passed;
        self.draw_dot.active = active;
        {
            let theme = makepad_theme::Theme::of(cx.cx);
            self.draw_dot.fill_solid = theme.paint.solid;
            self.draw_dot.fill_accent = theme.paint.accent;
            self.draw_dot.ring = theme.paint.border;
        }
        self.draw_number.color = number_ink;
        self.draw_title.color = title_ink;

        // The title's line box, from the theme's row height rather than a literal: a step's title is one line of a
        // known height, and taking the number from the same place the rest of the library does keeps a step's height
        // in step with everything else.
        let line = {
            let theme = makepad_theme::Theme::of(cx.cx);
            theme.layout.row_height as f64
        };
        // **A self-drawing widget declares its own size** — `Fit` measures zero for anything that does not walk a child
        // and report it, and this draws a circle and two strings by hand. So the height is the circle plus the gap plus
        // one line, and the width is whatever the title needs, measured rather than guessed.
        let title_width = if self.title.is_empty() {
            0.0
        } else {
            crate::mp::text::measured_width(&self.draw_title, cx.cx, &self.title)
        };
        let width = DOT_DIAMETER.max(title_width);
        let height = DOT_DIAMETER + TITLE_GAP + line;
        let walk = Walk {
            width: Size::Fixed(width),
            height: Size::Fixed(height),
            ..walk
        };
        // `walk_turtle` answers a `Rect`, not a position: the walk is placed and the rect is where it landed.
        let placed = cx.walk_turtle(walk);
        let origin = placed.pos;
        let dot = Rect {
            pos: origin,
            size: dvec2(DOT_DIAMETER, DOT_DIAMETER),
        };
        self.draw_dot.draw_abs(cx, dot);

        // The number, centred in the circle. Centred from the **measured** width, not from a guess about digit width —
        // a step numbered 10 would otherwise sit left of centre.
        let number = self.draw_number.layout(cx, 0.0, 0.0, None, false, Align::default(), &self.number());
        let number_size = dvec2(
            number.size_in_lpxs.width as f64 * crate::mp::text::DPI,
            number.size_in_lpxs.height as f64 * crate::mp::text::DPI,
        );
        self.draw_number.draw_abs(
            cx,
            dvec2(
                origin.x + (DOT_DIAMETER - number_size.x) * 0.5,
                origin.y + (DOT_DIAMETER - number_size.y) * 0.5,
            ),
            &self.number(),
        );

        if !self.title.is_empty() {
            self.draw_title.draw_abs(
                cx,
                dvec2(origin.x, origin.y + DOT_DIAMETER + TITLE_GAP),
                &self.title,
            );
        }
        DrawStep::done()
    }
}

impl MpStepItem {
    /// The step's number, as text: one-based, because a path's first step is step one.
    fn number(&self) -> String {
        format!("{}", self.position + 1)
    }

    /// Set the zero-based position, which is what the circle displays.
    pub fn set_position(&mut self, cx: &mut Cx, position: usize) {
        if self.position == position {
            return;
        }
        self.position = position;
        self.redraw(cx);
    }

    /// Mark the step passed, current or upcoming.
    pub fn set_state(&mut self, cx: &mut Cx, state: StepState) {
        self.state = state;
        self.redraw(cx);
    }

    /// Set the title shown under the circle.
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        if self.title == title {
            return;
        }
        self.title.clear();
        self.title.push_str(title);
        self.redraw(cx);
    }
}

impl MpStepItemRef {
    pub fn set_state(&self, cx: &mut Cx, state: StepState) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_state(cx, state);
        }
    }

    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn set_position(&self, cx: &mut Cx, position: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_position(cx, position);
        }
    }
}

/// The indicator: eight slots, each a connector and a step.
#[derive(Script, ScriptHook, Widget)]
pub struct MpStepIndicator {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// The current step, zero-based. `#[rust]` rather than `#[live]`, because a `#[live]` field is written back to its
    /// declared value whenever the script is re-applied — so a step kept there would be forgotten by a theme change.
    #[rust]
    current: usize,
    /// The titles, which the slots mirror.
    #[rust]
    titles: Vec<String>,
}

/// The id path of slot `index`.
fn slot_id(index: usize) -> LiveId {
    LiveId::from_str(&format!("slot{index}"))
}

/// The id path of a slot's connector or item.
fn part_id(index: usize, part: &str) -> [LiveId; 2] {
    let part = if part == "connector" {
        LiveId::from_str(&format!("connector{index}"))
    } else {
        LiveId::from_str(&format!("item{index}"))
    };
    [slot_id(index), part]
}

impl Widget for MpStepIndicator {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpStepIndicator {
    /// Set the titles, step by step.
    pub fn set_items(&mut self, cx: &mut Cx, titles: &[String]) {
        self.titles = titles.to_vec();
        self.apply(cx);
    }

    /// Move to step `current`.
    pub fn set_step(&mut self, cx: &mut Cx, current: usize) {
        if self.current == current {
            return;
        }
        self.current = current;
        self.apply(cx);
    }

    /// The current step.
    pub fn step(&self) -> usize {
        self.current
    }

    /// Re-apply the held titles and the current step.
    ///
    /// The parent sets **three things per slot and nothing else**: whether the slot exists, where the step stands, and
    /// what it is called. The circle, the number and the title are the item's own business — it draws all three — so
    /// there is no walking into a child's children here, which is what the first version of this did.
    pub fn apply(&mut self, cx: &mut Cx) {
        let shown = steps_shown(self.titles.len());
        for index in 0..SLOTS {
            let slot = self.view.view(cx, &[slot_id(index)]);
            if index >= shown {
                slot.set_visible(cx, false);
                continue;
            }
            slot.set_visible(cx, true);
            let item = self.view.mp_step_item(cx, &part_id(index, "item"));
            item.set_position(cx, index);
            item.set_title(cx, &self.titles[index]);
            item.set_state(cx, state_of(index, self.current));

            // **The connector carries the state of the step to its right**, and step 0's is hidden: there is nothing
            // before it to connect to.
            self.view
                .view(cx, &part_id(index, "connector"))
                .set_visible(cx, index > 0);
            self.view
                .view(cx, &[
                    part_id(index, "connector")[0],
                    part_id(index, "connector")[1],
                    id!(checked_line),
                ])
                .set_visible(cx, connector_passed(index, self.current));
        }
        self.redraw(cx);
    }
}

impl MpStepIndicatorRef {
    pub fn set_items(&self, cx: &mut Cx, titles: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, titles);
        }
    }

    pub fn set_step(&self, cx: &mut Cx, current: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_step(cx, current);
        }
    }

    pub fn step(&self) -> usize {
        self.borrow().map(|inner| inner.current).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_step_is_passed_current_or_upcoming_and_exactly_one_is_current() {
        // **The state is a function of the index and the current step, not three stored flags** — which is what makes
        // "exactly one current" true by construction rather than by remembering to maintain it.
        let current = 2;
        let states: Vec<StepState> = (0..5).map(|index| state_of(index, current)).collect();
        assert_eq!(
            states,
            vec![
                StepState::Passed,
                StepState::Passed,
                StepState::Active,
                StepState::Upcoming,
                StepState::Upcoming,
            ]
        );
        assert_eq!(
            states.iter().filter(|s| **s == StepState::Active).count(),
            1,
            "not exactly one step is current"
        );
    }

    #[test]
    fn test_the_first_step_is_current_at_zero_and_none_is_passed() {
        // The case the `<=` versus `<` in a hand-written comparison gets wrong.
        assert_eq!(state_of(0, 0), StepState::Active);
        assert_eq!(state_of(1, 0), StepState::Upcoming);
        assert!((0..SLOTS).all(|index| state_of(index, 0) != StepState::Passed));
    }

    #[test]
    fn test_a_current_step_past_the_end_leaves_every_step_passed() {
        // What a finished wizard looks like, and it needs no special case: every index is simply less than `current`.
        assert!((0..SLOTS).all(|index| state_of(index, SLOTS + 3) == StepState::Passed));
    }

    #[test]
    fn test_the_connector_before_a_step_is_filled_once_that_step_is_reached() {
        // **Not the same rule as the step's own state**, which is why it is its own function: the line into a step is
        // filled when the step is reached, so a connector carries the state of the step to its **right**.
        let current = 2;
        // Step 0 has nothing before it.
        assert!(!connector_passed(0, current));
        // The line into step 1 is filled, because step 1 is passed.
        assert!(connector_passed(1, current));
        // ...and into step 2, which is current: the path reaches it.
        assert!(connector_passed(2, current));
        // ...but not into step 3, which is ahead.
        assert!(!connector_passed(3, current));
        // The inversion worth naming: a **passed** step whose connector is filled, and an **upcoming** step whose is not
        // — so the connector is never simply `state == Passed`.
        assert_eq!(state_of(1, current), StepState::Passed);
        assert!(connector_passed(1, current));
        assert_eq!(state_of(2, current), StepState::Active);
        assert!(connector_passed(2, current), "the line into the current step is not filled");
    }

    #[test]
    fn test_no_connector_is_filled_before_the_first_step() {
        // At step zero the path has not started, so nothing is filled — including the connector into step 0, which is
        // hidden anyway. Checked at zero separately because it is the state a wizard opens in.
        assert!((0..SLOTS).all(|index| !connector_passed(index, 0)));
    }

    #[test]
    fn test_the_bound_is_applied_in_one_place() {
        assert_eq!(steps_shown(0), 0);
        assert_eq!(steps_shown(3), 3);
        assert_eq!(steps_shown(SLOTS), SLOTS);
        assert_eq!(steps_shown(SLOTS + 4), SLOTS, "the bound is not applied");

        // **What this test used to assert, and why that was wrong.** It looped over hidden slots and required
        // `connector_passed(index, count)` to be false — which treats `count` as if it were the current step. Those are
        // **two independent questions**: how many steps exist decides which slots are drawn, and which step is current
        // decides which connectors are filled. At count=1 with a current of 1 the connector of slot 1 is filled, and
        // slot 1 is not drawn at all — so the assertion was about a value nothing reads.
        //
        // The real guarantee is in `apply`, which `continue`s past a hidden slot **before touching its connector**, and a
        // pure function cannot see that. So this checks the independence instead — the count does not enter
        // `connector_passed` at all, which is visible in its signature and worth pinning because a version that took the
        // count "to be safe" is exactly how the two questions get entangled again.
        for current in 0..(SLOTS + 3) {
            for index in 0..(SLOTS + 3) {
                assert_eq!(
                    connector_passed(index, current),
                    index > 0 && index <= current,
                    "the connector rule stopped being about the current step at index={index} current={current}"
                );
            }
        }
    }
}
