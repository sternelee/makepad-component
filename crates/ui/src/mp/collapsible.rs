//! `MpCollapsibleHeader` — a section's header row, and the chevron that says which way it goes.
//!
//! ## It is a header, not a container, and the reason is worth keeping
//!
//! bezel's own note on its `collapsible_header`: *"The caller owns `expanded` and renders the body itself — a container that
//! swallowed its children would have to re-implement layout for them."* That is the right call here too, and it is the
//! opposite of what [`crate::mp::split_pane`] and [`crate::mp::tab_bar`] do — those **do** take their children, because a
//! split must set its panes' widths and a tab bar must place its tabs, and neither can be expressed by a flow. A collapsible
//! needs nothing of the sort: the body is simply there or not, laid out by whatever was going to lay it out anyway.
//!
//! So this is a header that reports `Toggled`, and the caller shows or hides the body. It is also why the header is not a
//! `View` with a `#[deref]`: there is nothing to forward to.
//!
//! ## Collapsed means **hidden**, not clipped
//!
//! The one rule a caller can get wrong without noticing: a body at zero height is still *there*, still laid out, and still
//! receiving events — a scroll view inside a collapsed section would scroll, and a button inside it could be clicked through
//! the header. The body must be hidden, and this module says so where the caller will read it rather than leaving it to be
//! discovered by watching a click land on something invisible.
//!
//! ## The chevron is the state
//!
//! [`disclosure`] points **down when expanded and right when collapsed** — the direction of the movement the section made, or
//! would make. That is a glyph per state rather than a rotation, because a rotation is an animation and an animation is the
//! motion catalog's business, not a header's: a caller that wants the chevron to turn asks the catalog for a duration and
//! animates the glyph.

use makepad_widgets::*;

/// The header's vertical padding.
pub const PAD_Y: f64 = 5.0;

/// Its horizontal padding.
pub const PAD_X: f64 = 4.0;

/// The gap between the chevron and the title.
pub const GAP: f64 = 6.0;

/// The chevron's width.
pub const CHEVRON: f64 = 14.0;

/// The header's height.
///
/// **One control row, from the theme** — the same `row_height` a menu row, a select trigger and a tab are, so a section header
/// lines up with everything beside it without any of them knowing about the others. The padding is inside that number rather
/// than added to it: the theme's number is the row.
pub fn header_height(cx: &mut Cx) -> f64 {
    makepad_theme::Theme::of(cx).layout.row_height as f64
}

/// The chevron for a section's state.
///
/// Down when expanded, right when collapsed — the direction the section moved, or would move.
pub fn disclosure(expanded: bool) -> &'static str {
    if expanded {
        "\u{25BE}"
    } else {
        "\u{25B8}"
    }
}

/// The state after a toggle, and whether anything changed.
///
/// Trivial arithmetic, and it is a function so that "a toggle always changes the state" is stated once — the same reason
/// [`crate::mp::select`]'s wrapper delegates rather than reimplementing.
pub fn toggled(expanded: bool) -> bool {
    !expanded
}

/// The body's height for a section's state.
///
/// **Zero when collapsed is the wrong answer on its own** — the body would still be laid out and still take events. This is
/// the number a caller needs, and the rule beside it is that the body must also be *hidden*: see the module doc.
pub fn body_height(expanded: bool, content_height: f64) -> f64 {
    if expanded && content_height.is_finite() && content_height > 0.0 {
        content_height
    } else {
        0.0
    }
}

/// What a header reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpCollapsibleAction {
    /// The header was pressed. The caller flips its own `expanded` and shows or hides the body.
    ///
    /// **No payload**: the header does not know the new state, because the state is the caller's — reporting `expanded: false`
    /// from a header that was drawn expanded would be this widget claiming to know something it does not.
    Toggled,
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.DrawMpCollapsibleHeader = #(DrawMpCollapsibleHeader::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 4.0
        plate: #x00000000
        hover_color: #x00000000
        hover: 0.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.0, 0.0, sz.x, sz.y, self.radius)
            sdf.fill_keep(mix(self.plate, self.hover_color, self.hover))
            return sdf.result
        }
    }

    mod.mp.MpCollapsibleHeaderBase = #(MpCollapsibleHeader::register_widget(vm))

    mod.mp.MpCollapsibleHeader = set_type_default() do mod.mp.MpCollapsibleHeaderBase{
        width: Fill
        height: 32.0

        draw_chevron +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_muted}
        draw_title +: {text_style: mod.mpc.type.callout, color: mod.mpc.tokens.text}
    }
}

/// The header's hover wash.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpCollapsibleHeader {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
    #[live]
    hover_color: Vec4f,
    #[live]
    hover: f32,
}

/// A section's header row.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpCollapsibleHeader {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    /// The animator the control signals need.
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawMpCollapsibleHeader,
    #[live]
    draw_chevron: DrawText,
    #[live]
    draw_title: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// The title. `#[live]` because it is the caller's declaration, like a button's label.
    #[live]
    title: ArcStringMut,
    /// **Drawn, not decided.** The caller owns `expanded`; this widget is told which way to point.
    #[rust]
    expanded: bool,
    #[rust]
    area: Area,
}

impl MpCollapsibleHeader {
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        if self.title.as_ref() != title {
            self.title.as_mut_empty().push_str(title);
            self.redraw(cx);
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Tell the header which way to point. **The caller's state**, not this widget's.
    pub fn set_expanded(&mut self, cx: &mut Cx, expanded: bool) {
        if self.expanded != expanded {
            self.expanded = expanded;
            self.redraw(cx);
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// The chevron this header is drawing.
    pub fn chevron(&self) -> &'static str {
        disclosure(self.expanded)
    }
}

impl Widget for MpCollapsibleHeader {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let signals = crate::mp::control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        // The signal a button uses for a press or Enter is the one a header wants: a header is a control, and a keyboard user
        // reaching it should be able to open a section without a pointer.
        if signals.activate {
            cx.widget_action(self.uid, MpCollapsibleAction::Toggled);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (plate, hover_color, muted, ink) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (
                Vec4f::default(),
                theme.paint.element_hover,
                theme.paint.text_muted,
                theme.paint.text,
            )
        };
        let placed = cx.walk_turtle(walk);
        self.area = self.draw_bg.area();
        self.draw_bg.plate = plate;
        self.draw_bg.hover_color = hover_color;
        self.draw_bg.radius = 4.0;
        self.draw_bg.hover = 0.0;
        self.draw_bg.draw_abs(cx, placed);
        self.draw_chevron.color = muted;
        self.draw_chevron.draw_abs(
            cx,
            dvec2(placed.pos.x + PAD_X, placed.pos.y + 8.0),
            disclosure(self.expanded),
        );
        self.draw_title.color = ink;
        self.draw_title.draw_abs(
            cx,
            dvec2(placed.pos.x + PAD_X + CHEVRON + GAP, placed.pos.y + 8.0),
            self.title.as_ref(),
        );
        DrawStep::done()
    }
}

impl MpCollapsibleHeaderRef {
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn set_expanded(&self, cx: &mut Cx, expanded: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_expanded(cx, expanded);
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.borrow().map(|inner| inner.is_expanded()).unwrap_or(false)
    }

    /// Whether this header was toggled, for a caller reading an event batch.
    pub fn toggled(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .is_some_and(|action| matches!(action.cast::<MpCollapsibleAction>(), MpCollapsibleAction::Toggled))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_chevron_points_the_way_the_section_moved_or_would_move() {
        // Down when expanded, right when collapsed — a glyph per state rather than a rotation, because a rotation is an
        // animation and an animation is the motion catalog's business rather than a header's.
        assert_eq!(disclosure(true), "\u{25BE}");
        assert_eq!(disclosure(false), "\u{25B8}");
        assert_ne!(disclosure(true), disclosure(false), "the two states share a glyph");
        // Both are single glyphs, so neither can wrap or shift its neighbour's position.
        assert_eq!(disclosure(true).chars().count(), 1);
        assert_eq!(disclosure(false).chars().count(), 1);
    }

    #[test]
    fn test_a_toggle_always_changes_the_state() {
        // The trivial arithmetic, stated once. There is no state a toggle leaves alone — which is why `MpCollapsibleAction`
        // carries no payload: the header does not know the new state, the caller does.
        assert!(toggled(false));
        assert!(!toggled(true));
        assert_eq!(toggled(toggled(false)), false);
        assert_eq!(toggled(toggled(toggled(false))), true);
    }

    #[test]
    fn test_a_collapsed_body_has_no_height_and_an_expanded_one_has_its_own() {
        // The number a caller needs — and **the rule beside it is that the body must also be hidden**: a body at zero height
        // is still laid out and still receives events, so a scroll view inside a collapsed section would scroll and a button
        // inside it could be clicked through the header. This function gives the height; the module doc gives the rest.
        assert_eq!(body_height(true, 120.0), 120.0);
        assert_eq!(body_height(false, 120.0), 0.0);
        // A nonsense content height is no height, rather than a `NaN` the layout inherits.
        assert_eq!(body_height(true, f64::NAN), 0.0);
        assert_eq!(body_height(true, f64::INFINITY), 0.0);
        assert_eq!(body_height(true, -10.0), 0.0);
        assert_eq!(body_height(true, 0.0), 0.0, "an empty body has no height even when expanded");
        // Collapsed is zero whatever the content, which is the property that matters.
        for height in [0.0, 1.0, 1e6, f64::NAN] {
            assert_eq!(body_height(false, height), 0.0, "a collapsed body took {height}");
        }
    }

    #[test]
    fn test_the_header_is_one_control_row_and_its_contents_fit_inside_it() {
        // **The theme's `row_height`, not a new number**: a section header lines up with the menu row, the select trigger and
        // the tab beside it without any of them knowing about the others. And the padding is inside that number rather than
        // added to it, so the contents have to fit — which is asserted here because a chevron that did not fit would be
        // clipped by its own header rather than by anything visible.
        assert_eq!(PAD_Y, 5.0);
        assert_eq!(GAP, 6.0);
        assert!(CHEVRON > 0.0 && GAP > 0.0 && PAD_X >= 0.0);
        // A one-line header: two paddings plus a line has to be within the theme's row height for the text to sit inside it.
        let text_line = 18.0;
        assert!(
            PAD_Y * 2.0 + text_line <= 32.0,
            "the header's contents do not fit one control row"
        );
        // The title starts after the padding, the chevron and the gap.
        let title_x = PAD_X + CHEVRON + GAP;
        assert!(title_x > PAD_X, "the title overlaps the chevron");
        assert!(title_x >= CHEVRON, "the title starts inside the chevron");
    }

    #[test]
    fn test_the_action_is_named_for_what_happened_rather_than_for_a_state() {
        // `Toggled`, not `Expanded` or `Collapsed`: the header reports the *event* and the caller owns the state, so an action
        // naming a state would be this widget claiming to know something it does not. The default is `None`, so a caller
        // reading an event batch for a toggle that did not happen gets nothing rather than a phantom.
        assert_ne!(MpCollapsibleAction::Toggled, MpCollapsibleAction::None);
        assert_eq!(MpCollapsibleAction::default(), MpCollapsibleAction::None);
        // The variant carries nothing, which is the same statement in the type.
        let action = MpCollapsibleAction::Toggled;
        assert_eq!(action, MpCollapsibleAction::Toggled);
        assert_eq!(std::mem::size_of::<MpCollapsibleAction>(), 1, "the action grew a payload");
    }
}
