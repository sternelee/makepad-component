//! `MpTitlebar` — the bar you drag a chromeless window by, and the conventions on it.
//!
//! ## The widget reports the drag; the caller moves the window
//!
//! A widget in this library cannot move a window: `Window::reposition` needs a `&Window`, which
//! `Widget::handle_event` does not get. That is not a limitation to work around — it is the shape every reporting widget
//! here already has. [`MpTitlebar`] reports [`MpTitlebarAction::DragBy`] with the pointer's movement, and the application,
//! which owns the window, applies it. The same as `MpMenuCard` reporting a choice and the caller dispatching it.
//!
//! ## A title bar's rules are conventions, and each is a decision
//!
//! - **A double click toggles the window's maximised state.** On macOS that is "zoom", on Windows "maximise"; both are the
//!   same gesture on the same region, and a title bar that ignored it would feel broken to anyone who has used a window.
//! - **The drag region is not the whole bar.** A bar with trailing controls — a close button, a search field — must not be
//!   draggable under them, or a press meant for a button moves the window instead. [`drag_region`] is that subtraction,
//!   and it is the one piece of geometry here with a rule in it rather than an extent.
//! - **A movement that is not a finite number is not a movement.** A `NaN` delta handed to `Window::reposition` puts the
//!   window somewhere unreachable, and on a backend that fits the request to the attached displays it would come back
//!   somewhere the user did not ask for — so a non-finite delta is dropped rather than clamped. This port has paid for a
//!   propagating `NaN` five times now; here the cost would be a window the user cannot get back.
//!
//! ## Wayland is asked about rather than assumed
//!
//! makepad reports `Window::uses_wayland_client_side_decorations`, and on such a compositor the window is moved by the
//! compositor's own protocol rather than by a position we choose. A caller has to branch on it — the widget cannot, since it
//! has no window — so it is named here rather than discovered later.

use makepad_widgets::*;

/// The bar's height.
///
/// 28 points: macOS's own title bar, which is what a custom one has to match to look like it belongs. The theme has a
/// `row_height` of 32, and using that would make every window's chrome 4 points taller than the platform's — a difference
/// nobody could name and everybody would feel.
pub const TITLEBAR_HEIGHT: f64 = 28.0;

/// Where the title's baseline sits from the top of the bar.
pub const TITLE_PAD: f64 = 8.0;

/// How wide a region at the trailing edge is reserved for controls.
///
/// The default is zero — a caller with no controls wants the whole bar draggable — and a caller with any passes the width
/// it needs. A *constant* here rather than a live field, because the width of a caller's controls is the caller's business.
pub const CONTROLS_DEFAULT: f64 = 0.0;

/// The draggable part of a bar `width` wide with `controls` points reserved at its trailing edge.
///
/// **The subtraction that keeps a press meant for a button from moving the window.** Clamped at zero: a caller whose control
/// region is wider than the bar has no draggable bar at all, which is a degenerate layout rather than a negative rect that
/// would be draggable *backwards*.
pub fn drag_region(width: f64, controls: f64) -> (f64, f64) {
    if !width.is_finite() || width <= 0.0 {
        return (0.0, 0.0);
    }
    let reserved = if controls.is_finite() && controls > 0.0 {
        controls.min(width)
    } else if controls.is_finite() {
        0.0
    } else {
        // A non-finite reservation is "unknown", not "everything": reserving the whole bar because a caller passed `NaN`
        // would silently make the window undraggable.
        0.0
    };
    (0.0, width - reserved)
}

/// Whether a point is in the draggable region, `x` being relative to the bar's leading edge.
pub fn is_draggable(x: f64, width: f64, controls: f64) -> bool {
    if !x.is_finite() || x < 0.0 {
        return false;
    }
    let (start, end) = drag_region(width, controls);
    x >= start && x < end
}

/// What a title bar reports.
///
/// `Default` because the `#[default]` variant is `None`, and `PartialEq` so a caller and a test can compare reports — but
/// not `Eq`, because a delta is a pair of floats and no float is equal to itself when it is a `NaN`.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MpTitlebarAction {
    /// The pointer moved while dragging the bar, by this much **since the last report**.
    ///
    /// A delta rather than a target position, because the bar cannot know where the window is. A caller applies it with
    /// `window.reposition(cx, window.get_position(cx) + delta)` — and on a Wayland compositor with client-side decorations
    /// it starts its own move instead; see the module doc.
    DragBy(DVec2),
    /// The bar was double-clicked: toggle the window's maximised state.
    ToggleMaximize,
    /// The pointer pressed the bar and began a drag — the point at which a caller on Wayland should hand the move to the
    /// compositor, since it will not be getting a useful `DragBy`.
    DragBegan,
    #[default]
    None,
}

/// A drag in progress: where the pointer was when it last reported.
///
/// Kept as the last pointer position rather than as an accumulated total, because a delta is what a caller applies and a
/// total computed by adding every move to a start point drifts on a long drag — the same reason a scroll offset is kept as a
/// position and not as a sum of deltas.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragState {
    last: Option<DVec2>,
}

impl DragState {
    /// Begin a drag at the pointer's current position.
    pub fn begin(&mut self, pointer: DVec2) {
        self.last = if is_finite(pointer) { Some(pointer) } else { None };
    }

    /// End it.
    pub fn end(&mut self) {
        self.last = None;
    }

    /// Whether a drag is in progress.
    pub fn is_dragging(&self) -> bool {
        self.last.is_some()
    }

    /// The movement since the last report, and advances.
    ///
    /// `None` when there is no drag, and `None` when the movement is not a finite number — **dropped rather than clamped**,
    /// because a clamped `NaN` is still a window somewhere the user did not ask for. A zero movement is also `None`: a
    /// report of nothing is a repaint of nothing.
    pub fn take_delta(&mut self, pointer: DVec2) -> Option<DVec2> {
        let last = self.last?;
        if !is_finite(pointer) {
            // The position is kept, so a single bad sample does not end the drag.
            return None;
        }
        let delta = pointer - last;
        self.last = Some(pointer);
        if delta.x == 0.0 && delta.y == 0.0 {
            return None;
        }
        Some(delta)
    }
}

fn is_finite(v: DVec2) -> bool {
    v.x.is_finite() && v.y.is_finite()
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.DrawMpTitlebar = #(DrawMpTitlebar::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 0.0
        plate: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.0, 0.0, sz.x, sz.y, self.radius)
            sdf.fill_keep(self.plate)
            return sdf.result
        }
    }

    mod.mp.MpTitlebarBase = #(MpTitlebar::register_widget(vm))

    mod.mp.MpTitlebar = set_type_default() do mod.mp.MpTitlebarBase{
        width: Fill
        height: 28.0

        // The title is centred, which is the platform convention for a window's own bar — a leading-aligned title reads as
        // a toolbar's heading rather than as the window's name.
        draw_title +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text}
    }
}

/// The bar's own plate.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpTitlebar {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
}

/// A draggable bar that reports what the pointer did to it.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpTitlebar {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    /// The animator the control signals need. A bar has no animation; the field is here because `control::handle` is what
    /// turns a pointer into `down`/`moved`/`up`, and re-implementing that hit test in a bar would give this port a second
    /// one.
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawMpTitlebar,
    /// The window's name.
    #[live]
    draw_title: DrawText,
    /// The title shown.
    #[live]
    title: ArcStringMut,
    /// How wide the trailing controls are, in points. The whole bar is draggable when this is zero.
    #[live]
    controls: f64,
    /// Whether a double click reports `ToggleMaximize`. A caller drawing a bar for a utility window that cannot be maximised
    /// turns it off rather than ignoring the report.
    #[live]
    zooms: bool,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    drag: DragState,
    /// The bar's top-left, captured during the last paint so a pointer position can be made relative to it — the shape
    /// every self-drawn widget here uses.
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl MpTitlebar {
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        if self.title.as_ref() != title {
            self.title.as_mut_empty().push_str(title);
            self.redraw(cx);
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Whether a point is in the draggable part of the bar, `x` being in screen coordinates.
    pub fn accepts_drag(&self, cx: &mut Cx, x: f64) -> bool {
        is_draggable(x - self.origin.x, self.area.rect(cx).size.x, self.controls)
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_dragging()
    }
}

impl Widget for MpTitlebar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // **The double click, read from the hit rather than from the control signals.** `tap_count` on the press is the
        // platform's own answer to "is this a double click", and the convention on every platform with a title bar is that a
        // double click on it toggles the window's maximised state.
        if let Hit::FingerDown(fe) = event.hits(cx, self.area) {
            if self.zooms && fe.tap_count == 2 {
                cx.widget_action(self.uid, MpTitlebarAction::ToggleMaximize);
            }
        }

        // The gesture, from the control signals — the same source a slider takes a drag from, so a bar and a slider cannot
        // disagree about what a press and a move are.
        let signals = crate::mp::control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        if signals.down {
            // **Only within the draggable region.** A press meant for a trailing control must not begin a window drag.
            let x = signals.pointer.x - self.origin.x;
            if is_draggable(x, self.area.rect(cx).size.x, self.controls) {
                self.drag.begin(dvec2(signals.pointer.x, signals.pointer.y));
                cx.widget_action(self.uid, MpTitlebarAction::DragBegan);
            }
            return;
        }
        if signals.moved {
            if let Some(delta) = self.drag.take_delta(dvec2(signals.pointer.x, signals.pointer.y)) {
                cx.widget_action(self.uid, MpTitlebarAction::DragBy(delta));
            }
            return;
        }
        if signals.up {
            self.drag.end();
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (plate, ink) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (theme.paint.surface_raised, theme.paint.text)
        };
        let placed = cx.walk_turtle(walk);
        self.origin = placed.pos;
        self.area = self.draw_bg.area();
        self.draw_bg.plate = plate;
        self.draw_bg.radius = 0.0;
        self.draw_bg.draw_abs(cx, placed);

        // Centred: see the DSL comment on why the title is not leading-aligned.
        let text = self.title.as_ref();
        if !text.is_empty() {
            let measured = crate::mp::text::measured_width(&self.draw_title, cx.cx, text);
            self.draw_title.color = ink;
            self.draw_title.draw_abs(
                cx,
                dvec2(
                    placed.pos.x + (placed.size.x - measured) * 0.5,
                    placed.pos.y + TITLE_PAD,
                ),
                text,
            );
        }
        DrawStep::done()
    }
}

impl MpTitlebarRef {
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn title(&self) -> String {
        self.borrow().map(|inner| inner.title().to_string()).unwrap_or_default()
    }

    pub fn is_dragging(&self) -> bool {
        self.borrow().map(|inner| inner.is_dragging()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_control_region_is_subtracted_from_the_draggable_bar() {
        // **The rule that keeps a press meant for a button from moving the window.** The bar is draggable from its leading
        // edge up to where the controls begin, and not one point past it.
        assert_eq!(drag_region(400.0, 0.0), (0.0, 400.0), "no controls, the whole bar");
        assert_eq!(drag_region(400.0, 80.0), (0.0, 320.0));
        assert!(is_draggable(0.0, 400.0, 80.0));
        assert!(is_draggable(319.9, 400.0, 80.0));
        assert!(!is_draggable(320.0, 400.0, 80.0), "the first point of the control region");
        assert!(!is_draggable(399.0, 400.0, 80.0));
        // A caller whose controls are wider than the bar has a bar that is not draggable at all, rather than a negative
        // region that would be draggable *backwards*.
        assert_eq!(drag_region(100.0, 150.0), (0.0, 0.0));
        assert!(!is_draggable(10.0, 100.0, 150.0));
    }

    #[test]
    fn test_a_bar_with_no_width_or_a_nonsense_one_has_no_draggable_region() {
        assert_eq!(drag_region(0.0, 0.0), (0.0, 0.0));
        assert_eq!(drag_region(-10.0, 0.0), (0.0, 0.0));
        assert_eq!(drag_region(f64::NAN, 0.0), (0.0, 0.0));
        // **A non-finite control width means "unknown", not "everything"**: reserving the whole bar because a caller passed
        // `NaN` would silently make the window undraggable, which is the failure a caller would never think to look for.
        assert_eq!(drag_region(400.0, f64::NAN), (0.0, 400.0));
        assert_eq!(drag_region(400.0, f64::INFINITY), (0.0, 400.0));
        assert_eq!(drag_region(400.0, -80.0), (0.0, 400.0), "a negative reservation reserves nothing");
        // A point that is not a number is not in the region.
        assert!(!is_draggable(f64::NAN, 400.0, 0.0));
        assert!(!is_draggable(f64::INFINITY, 400.0, 0.0));
        assert!(!is_draggable(-1.0, 400.0, 0.0));
    }

    #[test]
    fn test_a_drag_reports_the_movement_since_the_last_report() {
        // A **delta**, not a target: the bar cannot know where the window is, so it reports what changed and the caller adds
        // it to a position it owns.
        let mut drag = DragState::default();
        assert!(!drag.is_dragging());
        assert_eq!(drag.take_delta(dvec2(10.0, 10.0)), None, "no drag to report from");
        drag.begin(dvec2(100.0, 100.0));
        assert!(drag.is_dragging());
        assert_eq!(drag.take_delta(dvec2(104.0, 103.0)), Some(dvec2(4.0, 3.0)));
        // **The second report is from the last one, not from the start** — which is what keeps a long drag from drifting.
        assert_eq!(drag.take_delta(dvec2(110.0, 103.0)), Some(dvec2(6.0, 0.0)));
        assert_eq!(drag.take_delta(dvec2(110.0, 90.0)), Some(dvec2(0.0, -13.0)));
        drag.end();
        assert!(!drag.is_dragging());
        assert_eq!(drag.take_delta(dvec2(200.0, 200.0)), None, "the drag ended");
    }

    #[test]
    fn test_a_movement_that_is_not_a_number_is_dropped_rather_than_clamped() {
        // **A `NaN` delta handed to `Window::reposition` puts the window somewhere unreachable**, and a backend that fits
        // the request to the attached displays would bring it back somewhere the user did not ask for. Dropping it keeps the
        // window where it is; clamping it would still move it.
        let mut drag = DragState::default();
        drag.begin(dvec2(100.0, 100.0));
        assert_eq!(drag.take_delta(dvec2(f64::NAN, 100.0)), None);
        assert_eq!(drag.take_delta(dvec2(100.0, f64::INFINITY)), None);
        assert_eq!(drag.take_delta(dvec2(f64::NEG_INFINITY, f64::NAN)), None);
        // **And the drag survives it** — one bad sample does not end the gesture, so the next real movement still reports from
        // the last good position.
        assert!(drag.is_dragging(), "a bad sample ended the drag");
        assert_eq!(drag.take_delta(dvec2(105.0, 100.0)), Some(dvec2(5.0, 0.0)));
        // Beginning with a bad position is not a drag at all.
        let mut bad = DragState::default();
        bad.begin(dvec2(f64::NAN, 0.0));
        assert!(!bad.is_dragging());
        assert_eq!(bad.take_delta(dvec2(1.0, 1.0)), None);
    }

    #[test]
    fn test_a_movement_of_nothing_is_not_reported() {
        // A report of zero is a repaint of zero — and a pointer that jitters by a fraction of a point generates a great many
        // of them.
        let mut drag = DragState::default();
        drag.begin(dvec2(50.0, 50.0));
        assert_eq!(drag.take_delta(dvec2(50.0, 50.0)), None);
        assert_eq!(drag.take_delta(dvec2(50.0, 50.0)), None);
        assert_eq!(drag.take_delta(dvec2(51.0, 50.0)), Some(dvec2(1.0, 0.0)));
    }

    #[test]
    fn test_a_drag_can_be_restarted_without_ending_the_previous_one() {
        // A second press (a double click, or a press that was never released in a lost event) must not compute its first
        // delta from where the previous gesture ended.
        let mut drag = DragState::default();
        drag.begin(dvec2(0.0, 0.0));
        drag.take_delta(dvec2(500.0, 500.0));
        drag.begin(dvec2(10.0, 10.0));
        assert_eq!(
            drag.take_delta(dvec2(12.0, 10.0)),
            Some(dvec2(2.0, 0.0)),
            "the new drag measured from the old gesture's end"
        );
    }

    #[test]
    fn test_the_height_is_the_platform_s_one_rather_than_the_theme_s_row() {
        // The number has a source: macOS's own title bar is 28 points, and the theme's `row_height` is 32. A custom bar at
        // the theme's height would make every window's chrome four points taller than the platform's — a difference nobody
        // could name and everybody would feel.
        assert_eq!(TITLEBAR_HEIGHT, 28.0);
        assert_ne!(
            TITLEBAR_HEIGHT, 32.0,
            "the title bar is not a row, and must not silently become one"
        );
        assert!(TITLE_PAD > 0.0 && TITLE_PAD < TITLEBAR_HEIGHT);
    }
}
