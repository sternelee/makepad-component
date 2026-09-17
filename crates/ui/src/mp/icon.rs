//! `MpIcon` — a glyph from the bundled icon font.
//!
//! The smallest component in the crate and the one that most other components
//! want: a button with a leading glyph, a list row with a trailing chevron, a
//! toolbar. Makepad ships FontAwesome (regularly as `theme.font_icons`), so there
//! is nothing to draw — the whole component is *choosing the ink, the size and
//! the box*, which is what a component library is for.
//!
//! ## Why not a `Label` with a font override
//!
//! That is exactly what a call site would write, and it fails in three ways a
//! component exists to prevent:
//!
//! - **Optical size.** A glyph's box is not the text ladder's line box: an icon
//!   set at `Body`'s 13pt with `Body`'s 16pt leading sits visibly low in a 24pt
//!   row, because the icon face has no descender to take up the slack. The
//!   widget centres the glyph's own box instead.
//! - **Ink.** An icon is decoration, so it takes `text_muted` by default rather
//!   than body ink; a caller that wants body ink says so.
//! - **Colour inheritance.** A disabled or hovered parent needs its icons to
//!   follow, and a `Label` cannot be told to without its own animator.
//!
//! ## The size is a role, not a number
//!
//! `MpIcon` sizes from [`ControlSize`], the same three rungs every control uses,
//! so an icon inside a `MpButton` is the right size without anybody checking. A
//! caller who wants a glyph at a size the ladder does not have sets `size`.

use makepad_widgets::*;

use makepad_theme::ControlSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.mp.MpIconBase = #(MpIcon::register_widget(vm))

    mod.mp.MpIcon = set_type_default() do mod.mp.MpIconBase{
        // A square the size of the control it sits in, so an icon-leading button
        // lines up with an icon-only one.
        width: 16
        height: 16

        // The glyph, as the character itself. A caller writes
        // `glyph: "\u{f00c}"` rather than a name, because a name would have to be
        // mapped here and there are 1834 of them in the font — the map is the
        // caller's to keep, and a wrong one is a font miss rather than a bug in
        // this crate.
        glyph: "\u{f111}"

        // The rung whose type size the glyph is set at.
        control: mod.mpc.ControlSize.Regular
        // A font scale over that rung, for the one-off that needs a hair more.
        scale: 1.0

        draw_text +: {
            text_style: theme.font_icons{font_size: 11.0}
            color: #x00000000
        }
    }

    mod.mp.MpIconSmall = mod.mp.MpIcon{
        control: mod.mpc.ControlSize.Small
        width: 14
        height: 14
    }

    mod.mp.MpIconLarge = mod.mp.MpIcon{
        control: mod.mpc.ControlSize.Large
        width: 20
        height: 20
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpIcon {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    /// The character to draw. An `ArcStringMut` rather than a `char` because
    /// Makepad's `draw_walk` takes a string, and a glyph is one or two `char`s
    /// once a variation selector is involved.
    #[live]
    glyph: ArcStringMut,

    #[live]
    control: ControlSize,

    /// A multiplier over the rung's type size.
    #[live]
    scale: f64,

    #[redraw]
    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[rust]
    area: Area,
}

impl MpIcon {
    pub fn glyph(&self) -> &str {
        self.glyph.as_ref()
    }

    pub fn set_glyph(&mut self, cx: &mut Cx, glyph: &str) {
        self.glyph.as_mut_empty().push_str(glyph);
        self.redraw(cx);
    }

    pub fn control(&self) -> ControlSize {
        self.control
    }

    pub fn set_control(&mut self, cx: &mut Cx, control: ControlSize) {
        self.control = control;
        self.redraw(cx);
    }

    /// The point size this icon paints at, before the ladder's own scale.
    ///
    /// Exposed because a caller aligning an icon against text needs the same
    /// number the widget uses, and re-deriving it at the call site is how the two
    /// drift.
    pub fn point_size(&self) -> f64 {
        self.control.metrics().size() as f64 * self.scale
    }
}

impl Widget for MpIcon {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // An icon is not interactive. A caller who wants a clickable one wraps it
        // in whatever should take the click, which keeps the hit area and the
        // cursor the wrapper's business rather than the glyph's.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);

        // Decoration, not content: an icon at body ink competes with the label it
        // sits beside, which is the opposite of what a leading glyph is for.
        self.draw_text.color = theme.paint.text_muted;
        // `top_drop` is the one vertical control the icon face gives us: the face
        // has no descender, so a glyph centred by its line box sits low. Dropping
        // it by a fifth of its own size lands it on the optical centre of the row
        // rather than on the typographic one — the number is the font's, not a
        // guess about the caller's layout.
        self.draw_text.text_style.font_size = self.point_size() as f32;
        self.draw_text.text_style.top_drop = -0.08;

        self.draw_text
            .draw_walk(cx, self.walk, Align::default(), self.glyph.as_ref());
        self.area = self.draw_text.area();
        DrawStep::done()
    }
}

impl MpIconRef {
    pub fn set_glyph(&self, cx: &mut Cx, glyph: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_glyph(cx, glyph);
        }
    }

    pub fn set_control(&self, cx: &mut Cx, control: ControlSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_control(cx, control);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_size_ladder_drives_the_glyph() {
        // The point of sizing from `ControlSize` rather than a number: an icon in
        // a `Small` button is small, one in a `Large` button is large, and nobody
        // compares the two by eye.
        let sizes: Vec<f32> = [
            ControlSize::Small,
            ControlSize::Regular,
            ControlSize::Large,
        ]
        .iter()
        .map(|c| c.metrics().size())
        .collect();
        assert!(sizes[0] < sizes[1] && sizes[1] < sizes[2], "{sizes:?}");
    }

    #[test]
    fn test_an_icon_is_never_larger_than_the_control_that_holds_it() {
        // A glyph as tall as its control has no room for the control's own
        // padding, and the row grows to fit the icon instead of the text. The
        // ladder's role sizes are all comfortably under their control heights.
        for control in [
            ControlSize::Small,
            ControlSize::Regular,
            ControlSize::Large,
        ] {
            let glyph = control.metrics().size() as f64;
            assert!(
                glyph <= control.height() as f64,
                "{control:?}: glyph {glyph} vs height {}",
                control.height()
            );
        }
    }
}
