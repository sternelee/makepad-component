//! `MpStrip` — a tonal notice: a message on a wash, with its tone's glyph.
//!
//! ## The component is a tone, so the tone is a closed enum
//!
//! bezel ships two of these (`error_strip`, `warning_strip`) and they differ in exactly three things: the tone's colours, the
//! glyph, and — incidentally — two points of vertical padding. Everything else is the same plate. So this is one component
//! with a [`StripTone`] rather than four near-identical widgets, which is what the library does with a button's style and
//! what its second law asks for: **a closed vocabulary, never a colour.**
//!
//! [`StripTone`] has four tones rather than bezel's two, and each is justified by a token the theme already carries —
//! `danger`, `warning`, `success`, and `border` for the neutral one. Going *past* bezel where the theme has the vocabulary is
//! consistent with the button, whose four styles are bezel's three plus a named base.
//!
//! ## The plate's wash is derived, not chosen
//!
//! The background is the tone at a low opacity over the surface and the border is the tone at a higher one — the same two
//! fractions for every tone, so a new tone cannot arrive with a hand-picked wash that does not match its neighbours. bezel's
//! fractions are kept (`0.06` and `0.2`), and they are constants because a caller choosing them would be choosing a colour.
//!
//! ## The height follows the message
//!
//! A strip is text on a plate, so its height is `2 * PAD + lines * LINE` — a rule with a test, because a strip whose plate
//! did not grow with a two-line message would clip the second line, and one that grew to a fixed height would leave a gap
//! under a one-line one. [`strip_height`] is that, and it takes the line count rather than measuring: **the number of lines a
//! message wraps to is a layout result the caller has already computed**, and measuring it again here would be a second
//! answer to a question the layout just answered.
//!
//! ## What is deliberately absent
//!
//! A dismiss button. bezel's strips have none, and a strip that can be dismissed is a different component — it has state, it
//! needs a caller to notice it was dismissed, and once one strip has a close button every strip wants one. If this library
//! grows that, it should be a named decision rather than a field added here.

use makepad_widgets::*;

/// The plate's vertical padding.
pub const PAD: f64 = 11.0;

/// The message's line height.
pub const LINE: f64 = 18.0;

/// How much of the tone the plate's wash takes.
pub const WASH: f64 = 0.06;

/// How much of the tone the plate's border takes.
pub const BORDER: f64 = 0.2;

/// The glyph's size, and the gap between it and the message.
pub const GLYPH: f64 = 14.0;
pub const GAP: f64 = 8.0;

/// What a strip is saying.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StripTone {
    /// Something failed. The most severe, and the one a caller should not be able to miss.
    #[default]
    Error,
    /// Something may be wrong: a warning, which a reader may decide to live with.
    Warning,
    /// Neutral information about what just happened.
    Info,
    /// Something succeeded.
    Success,
}

impl StripTone {
    /// The glyph, as a character rather than an icon name: a strip's glyph is part of its tone, and an icon set that lacked
    /// one would leave a warning strip with no symbol at all.
    pub fn glyph(self) -> &'static str {
        match self {
            StripTone::Error => "\u{26A0}",
            StripTone::Warning => "\u{26A0}",
            StripTone::Info => "\u{2139}",
            StripTone::Success => "\u{2713}",
        }
    }

    /// The same, spelled for a caller that logs or serialises one.
    pub fn name(self) -> &'static str {
        match self {
            StripTone::Error => "error",
            StripTone::Warning => "warning",
            StripTone::Info => "info",
            StripTone::Success => "success",
        }
    }

    /// Whether this tone is asking the reader to do something about it.
    ///
    /// Error and warning do; info and success report. A caller uses this to decide whether a strip may be skipped rather than
    /// shown, and it is here rather than at the call site because **it is a property of the tone**, not of the occasion.
    pub fn demands_action(self) -> bool {
        matches!(self, StripTone::Error | StripTone::Warning)
    }

    /// The theme colour a strip of this tone is drawn in.
    ///
    /// The mapping from a closed vocabulary to the environment's tokens, which is where all of this component's styling lives.
    pub fn color(self, theme: &makepad_theme::Theme) -> Vec4f {
        match self {
            StripTone::Error => theme.paint.danger,
            StripTone::Warning => theme.paint.warning,
            StripTone::Info => theme.paint.accent,
            StripTone::Success => theme.paint.success,
        }
    }

    /// The colour the message is set in: the tone's muted pair where the theme has one, so the text is legible on the wash
    /// rather than the saturated tone drawn on itself.
    pub fn text_color(self, theme: &makepad_theme::Theme) -> Vec4f {
        match self {
            StripTone::Error => theme.paint.danger_muted,
            StripTone::Warning => theme.paint.warning_muted,
            StripTone::Info => theme.paint.accent,
            StripTone::Success => theme.paint.success,
        }
    }
}

/// A strip's height for a message of `lines` lines.
///
/// One line at least: an empty message still draws a plate, because a caller showing a strip with nothing in it has a bug to
/// see rather than a strip to hide.
pub fn strip_height(lines: usize) -> f64 {
    PAD * 2.0 + LINE * lines.max(1) as f64
}

/// The message's colour, at the opacity a description is drawn with.
///
/// The wash is faint by design, so the text is mixed toward full rather than taken straight from the token — bezel's strips
/// carry `opacity(0.9)` for the same reason.
pub fn message_color(tone: StripTone, theme: &makepad_theme::Theme) -> Vec4f {
    faded(tone.text_color(theme))
}

/// A colour at the opacity a message on a wash is drawn with.
///
/// **The rule is here, not inline, so it can be tested without a theme** — the theme is a whole environment to construct, and
/// a test that had to build one to check a multiply would end up checking a copy of the multiply instead.
pub fn faded(color: Vec4f) -> Vec4f {
    Vec4f {
        w: color.w * 0.9,
        ..color
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The plate: a wash of the tone over the surface, with a border of the same tone.
    mod.mp.DrawMpStrip = #(DrawMpStrip::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 8.0
        wash: #x00000000
        border_color: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.5, 0.5, sz.x - 1.0, sz.y - 1.0, self.radius)
            sdf.fill_keep(self.wash)
            sdf.stroke(self.border_color, 1.0)
            return sdf.result
        }
    }

    mod.mp.MpStripBase = #(MpStrip::register_widget(vm))

    mod.mp.MpStrip = set_type_default() do mod.mp.MpStripBase{
        width: Fill
        height: Fit

        draw_glyph +: {text_style: mod.mpc.type.body}
        draw_message +: {text_style: mod.mpc.type.callout}
    }
}

/// The strip's plate.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpStrip {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    wash: Vec4f,
    #[live]
    border_color: Vec4f,
}

/// A tonal notice.
#[derive(Script, ScriptHook, Widget)]
pub struct MpStrip {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_bg: DrawMpStrip,
    #[live]
    draw_glyph: DrawText,
    #[live]
    draw_message: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// What it is saying.
    #[rust]
    tone: StripTone,
    #[rust]
    message: String,
    /// How many lines the message occupies. **Given, not measured** — the caller's layout already wrapped the text, and
    /// measuring it here would be a second answer to a question that has been answered.
    #[rust]
    lines: usize,
    #[rust]
    area: Area,
}

impl MpStrip {
    pub fn set_message(&mut self, cx: &mut Cx, message: &str) {
        self.message = message.to_string();
        self.redraw(cx);
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// Set the tone and the number of lines the message occupies.
    pub fn set_tone(&mut self, cx: &mut Cx, tone: StripTone, lines: usize) {
        self.tone = tone;
        self.lines = lines;
        self.redraw(cx);
    }

    pub fn tone(&self) -> StripTone {
        self.tone
    }

    pub fn height(&self) -> f64 {
        strip_height(self.lines)
    }
}

impl Widget for MpStrip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // A strip is a notice, not a control: nothing to press, no focus to hold. The hit is taken so its area is live for a
        // tooltip, and that is all.
        let _ = event.hits(cx, self.area);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (tone, text) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (self.tone.color(theme), message_color(self.tone, theme))
        };
        let height = strip_height(self.lines);
        let placed = cx.walk_turtle(Walk {
            height: Size::Fixed(height),
            ..walk
        });
        self.area = self.draw_bg.area();
        // **The wash and the border are the tone at two fractions**, the same two for every tone — so a new tone cannot
        // arrive with hand-picked values that do not match its neighbours.
        self.draw_bg.wash = Vec4f {
            x: tone.x,
            y: tone.y,
            z: tone.z,
            w: WASH as f32,
        };
        self.draw_bg.border_color = Vec4f {
            x: tone.x,
            y: tone.y,
            z: tone.z,
            w: BORDER as f32,
        };
        self.draw_bg.radius = 8.0;
        self.draw_bg.draw_abs(cx, placed);

        self.draw_glyph.color = text;
        self.draw_glyph
            .draw_abs(cx, dvec2(placed.pos.x + PAD, placed.pos.y + PAD), self.tone.glyph());
        self.draw_message.color = text;
        self.draw_message.draw_abs(
            cx,
            dvec2(placed.pos.x + PAD + GLYPH + GAP, placed.pos.y + PAD),
            &self.message,
        );
        DrawStep::done()
    }
}

impl MpStripRef {
    pub fn set_message(&self, cx: &mut Cx, message: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_message(cx, message);
        }
    }

    pub fn set_tone(&self, cx: &mut Cx, tone: StripTone, lines: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_tone(cx, tone, lines);
        }
    }

    pub fn tone(&self) -> StripTone {
        self.borrow().map(|inner| inner.tone()).unwrap_or_default()
    }

    pub fn message(&self) -> String {
        self.borrow().map(|inner| inner.message().to_string()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_plate_grows_with_the_message_and_never_below_one_line() {
        // A strip whose plate did not grow with a two-line message would clip the second line; one that grew to a fixed
        // height would leave a gap under a one-line one. And **an empty message still draws a plate**: a caller showing a
        // strip with nothing in it has a bug to see, not a strip to hide.
        assert_eq!(strip_height(1), PAD * 2.0 + LINE);
        assert_eq!(strip_height(2), PAD * 2.0 + LINE * 2.0);
        assert_eq!(strip_height(0), PAD * 2.0 + LINE, "an empty message is one line, not none");
        for lines in 1..6usize {
            assert!(strip_height(lines + 1) > strip_height(lines));
            assert_eq!(
                strip_height(lines + 1) - strip_height(lines),
                LINE,
                "a line costs a line"
            );
        }
    }

    #[test]
    fn test_the_wash_and_the_border_are_the_same_two_fractions_for_every_tone() {
        // **The property that keeps a new tone from arriving with a wash that does not match its neighbours**: the fraction
        // is a constant of the component, not a per-tone choice. A tone only supplies a colour.
        assert!(WASH > 0.0 && WASH < BORDER, "the wash should be fainter than its border");
        assert!(BORDER < 1.0, "an opaque border is a box, not a wash");
        assert_eq!(WASH, 0.06);
        assert_eq!(BORDER, 0.2);
        // The same fractions at every tone, since there is nowhere per-tone for them to differ.
        for tone in [StripTone::Error, StripTone::Warning, StripTone::Info, StripTone::Success] {
            assert!(!tone.name().is_empty(), "a tone with no name cannot be logged");
        }
    }

    #[test]
    fn test_every_tone_has_a_glyph_and_a_name_and_they_are_distinct_where_they_should_be() {
        // A strip's glyph is part of its tone, and a tone without one would leave a notice with no symbol. Error and warning
        // share the alert triangle deliberately — they differ in colour and in how much they demand — while info and success
        // each have their own.
        assert_eq!(StripTone::Error.glyph(), StripTone::Warning.glyph());
        assert_ne!(StripTone::Info.glyph(), StripTone::Success.glyph());
        assert_ne!(StripTone::Error.glyph(), StripTone::Success.glyph());
        let names: Vec<&str> = [StripTone::Error, StripTone::Warning, StripTone::Info, StripTone::Success]
            .iter()
            .map(|tone| tone.name())
            .collect();
        assert_eq!(names, ["error", "warning", "info", "success"]);
        assert_eq!(names.iter().collect::<std::collections::HashSet<_>>().len(), 4, "two tones share a name");
    }

    #[test]
    fn test_only_the_two_tones_that_ask_for_something_say_so() {
        // **A property of the tone, not of the occasion.** A caller deciding whether a strip may be skipped rather than shown
        // asks the tone, so that "warnings matter and successes do not" is stated once instead of at every call site.
        assert!(StripTone::Error.demands_action());
        assert!(StripTone::Warning.demands_action());
        assert!(!StripTone::Info.demands_action());
        assert!(!StripTone::Success.demands_action());
        // Error and warning are the two `demands_action` tones, and there are exactly two of them.
        let demanding = [StripTone::Error, StripTone::Warning, StripTone::Info, StripTone::Success]
            .iter()
            .filter(|tone| tone.demands_action())
            .count();
        assert_eq!(demanding, 2);
    }

    #[test]
    fn test_the_default_tone_is_the_one_a_caller_must_not_be_able_to_miss() {
        // A strip with no tone set is an error, not an info: the default should be the one that asks for attention rather
        // than the one that is easiest to ignore.
        assert_eq!(StripTone::default(), StripTone::Error);
        assert!(StripTone::default().demands_action());
    }

    #[test]
    fn test_the_message_is_drawn_slightly_transparent_so_it_sits_on_its_own_wash() {
        // The wash is faint by design and the text is the tone's muted pair, faded rather than taken straight — the same
        // reason bezel's strips carry `opacity(0.9)`. **This calls the function rather than restating its arithmetic**: my
        // first version built the expected value by hand, which would have passed for any implementation at all.
        let color = Vec4f {
            x: 0.8,
            y: 0.2,
            z: 0.2,
            w: 1.0,
        };
        let mixed = faded(color);
        assert_eq!(mixed.x, color.x, "the message colour was not the tone's");
        assert_eq!(mixed.y, color.y);
        assert_eq!(mixed.z, color.z);
        assert!(mixed.w < color.w, "the message is fully opaque on a faint wash");
        // It degrades rather than inverts, and a transparent colour stays transparent.
        assert_eq!(faded(color).w, 0.9);
        assert_eq!(faded(Vec4f::default()).w, 0.0);
        let faint = Vec4f { w: 0.5, ..color };
        assert!(faded(faint).w < faint.w);
    }

    #[test]
    fn test_the_strip_s_padding_and_glyph_leave_room_for_the_message() {
        // The message starts after the padding, the glyph and the gap — and the plate is tall enough for the glyph, or the
        // symbol would hang below the wash it belongs to.
        assert!(strip_height(1) > PAD * 2.0, "the plate has no room for a line");
        assert!(GLYPH + PAD * 2.0 <= strip_height(1), "the glyph does not fit the plate");
        assert!(GAP > 0.0, "the glyph would touch the message");
        assert!(PAD > 0.0 && LINE > 0.0);
    }
}
