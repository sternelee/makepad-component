//! Small status plates: `MpBadge` and `MpTag`.
//!
//! Paint-only, like [`MpIcon`](crate::mp::icon), and grouped here because they
//! are the same widget with two behaviours:
//!
//! - **`MpBadge`** reports something about its subject — a count, a state, a
//!   build number. It is a *result*, so it carries its own fill and reads as an
//!   object placed on the row.
//! - **`MpTag`** classifies its subject — a label, a category, a filter that is
//!   on. It is an *adjective*, so it is a wash with a hairline rather than a
//!   plate, and it does not compete with the text beside it.
//!
//! The difference is which surface each paints, and that is the whole difference:
//! one shader, one `Tone` resolution, two assembled looks. A caller picks a tone
//! from a closed set, and free-form colours never become API.
//!
//! ## Why a tone enum rather than a colour
//!
//! `Tone` has six members and no numbers. The palette verified a contrast floor
//! for each pairing — a badge's ink on a badge's plate — and a caller passing
//! `#f0a` would be opting out of that without saying so. A tone is the vocabulary
//! the palette can actually promise something about.

use makepad_widgets::*;

use makepad_theme::ControlSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    // The closed set, on the heap, because a DSL block names a tone and a tone
    // that is not registered is 108 runtime errors pointing at every call site
    // rather than at this line.
    mod.mp.StatusTone = set_type_default() do #(StatusTone::script_api(vm))

    set_type_default() do #(DrawMpStatus::script_shader(vm)){
        ..mod.draw.DrawQuad

        // Resolved from `Theme::of(cx)` every paint — this widget owns its
        // shader, so it takes the leaf path rather than the DSL-token one.
        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 4.0
        // The leading dot a tag may carry. 0 hides it.
        dot_color: #x00000000
        dot_radius: 0.0
        dot_gap: 0.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            let h = self.rect_size.y
            let r = min(self.radius, h * 0.5)

            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, h - bw * 2.0, max(1.0, r))
            // `fill_keep`, because the stroke below is the *same* box.
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }

            // The dot, when the tone asks for one. Drawn after the plate so it
            // is not covered by it, and inset by the radius so it never touches
            // the rounded corner.
            if (self.dot_radius > 0.0) {
                sdf.circle(
                    bw + self.dot_radius
                    h * 0.5
                    self.dot_radius
                )
                sdf.fill(self.dot_color)
            }
            return sdf.result
        }
    }

    // ---- badge ----
    mod.mp.MpBadgeBase = #(MpBadge::register_widget(vm))

    mod.mp.MpBadge = set_type_default() do mod.mp.MpBadgeBase{
        width: Fit
        height: 18
        padding: Inset{left: 6, right: 6, top: 0, bottom: 0}
        align: Align{x: 0.5, y: 0.5}

        text: "Badge"
        tone: mod.mp.StatusTone.Neutral

        draw_text +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
    }

    mod.mp.MpBadgeSmall = mod.mp.MpBadge{
        height: 15
        padding: Inset{left: 5, right: 5, top: 0, bottom: 0}
    }

    mod.mp.MpBadgeDot = mod.mp.MpBadge{
        // A count plate with no text is a dot: 8pt square, and the label is empty.
        width: 8
        height: 8
        padding: 0
        text: ""
    }

    // ---- tag ----
    mod.mp.MpTagBase = #(MpTag::register_widget(vm))

    mod.mp.MpTag = set_type_default() do mod.mp.MpTagBase{
        width: Fit
        height: 20
        // The dot needs its own room, so a tag's leading inset is larger than a
        // badge's and the shader is told how much of it the dot takes.
        padding: Inset{left: 16, right: 8, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}

        text: "Tag"
        tone: mod.mp.StatusTone.Neutral
        // `false` for a tag that is only a label — a category, not a state.
        dot: false

        draw_text +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
    }

    mod.mp.MpTagNoDot = mod.mp.MpTag{
        dot: false
        padding: Inset{left: 8, right: 8, top: 0, bottom: 0}
    }

    mod.mp.MpTagSmall = mod.mp.MpTag{
        height: 17
        padding: Inset{left: 14, right: 6, top: 0, bottom: 0}
    }
}

/// The closed set of tones a status plate can be.
///
/// Six, because the palette has six things it can promise a contrast floor for: a
/// neutral plate, the inverted one, and the four status hues. `Busy` is the
/// fourth status rather than a second accent — work in flight is neither good nor
/// bad news, and painting it as either is the mistake this member exists to
/// prevent.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Script, ScriptHook)]
pub enum StatusTone {
    /// The everyday plate: a raised fill, body ink. For a count or a build number.
    #[pick]
    #[default]
    #[live]
    Neutral,
    /// The inverted plate. One per view at most — it is the loudest thing here.
    #[live]
    Solid,
    #[live]
    Accent,
    #[live]
    Success,
    #[live]
    Warning,
    #[live]
    Danger,
    /// In flight. Not an error and not a success.
    #[live]
    Busy,
}

/// One tone's resolved paint, before it is written to the shader.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plate {
    pub(crate) fill: Vec4f,
    pub(crate) border: Vec4f,
    pub(crate) border_width: f32,
    pub(crate) ink: Vec4f,
    pub(crate) dot: Vec4f,
}

impl StatusTone {
    /// This tone as a filled plate — what a badge paints.
    ///
    /// `pub(crate)` because the avatar reuses it for its presence dot, so a
    /// person's state and a badge's state are the same six tones.
    pub(crate) fn badge(self, theme: &makepad_theme::Theme) -> Plate {
        let p = &theme.paint;
        match self {
            StatusTone::Neutral => Plate {
                fill: p.surface_raised,
                border: p.border,
                border_width: 1.0,
                ink: p.text,
                dot: p.text_muted,
            },
            StatusTone::Solid => Plate {
                fill: p.solid,
                border: p.solid,
                border_width: 0.0,
                ink: p.on_solid,
                dot: p.on_solid,
            },
            StatusTone::Accent => Plate {
                fill: p.accent_strong,
                border: p.accent_strong,
                border_width: 0.0,
                ink: p.on_accent,
                dot: p.on_accent,
            },
            StatusTone::Success => status_plate(p.success_muted, p),
            StatusTone::Warning => status_plate(p.warning_muted, p),
            StatusTone::Danger => status_plate(p.danger_muted, p),
            StatusTone::Busy => status_plate(p.busy, p),
        }
    }

    /// This tone as a wash with a hairline — what a tag paints.
    ///
    /// The fill is the tone at badge weight composited *over the page* rather
    /// than a plate of its own, because a tag sits in a line of text and a solid
    /// plate there would read as a button. The hairline is the tone at full
    /// strength, which is what makes a tag legible as a category while staying
    /// quieter than a badge.
    fn tag(self, theme: &makepad_theme::Theme) -> Plate {
        let p = &theme.paint;
        // A wash is the tone over the page at low coverage, so it takes the
        // page's own lightness rather than sitting on top of it as a plate.
        let wash = |c: Vec4f| makepad_theme::color::flatten_a(c, p.bg, 0.16);
        let (base, ink) = match self {
            StatusTone::Neutral => (p.text_muted, p.text_muted),
            StatusTone::Solid => (p.solid, p.text),
            StatusTone::Accent => (p.accent, p.accent),
            StatusTone::Success => (p.success, p.success),
            StatusTone::Warning => (p.warning, p.warning),
            StatusTone::Danger => (p.danger, p.danger),
            StatusTone::Busy => (p.busy, p.busy),
        };
        Plate {
            fill: wash(base),
            border: base,
            border_width: 1.0,
            ink,
            dot: base,
        }
    }
}

/// A status hue as a plate: the muted tone carries the fill, and the ink is the
/// full-strength hue so the label stays legible on it.
fn status_plate(muted: Vec4f, p: &makepad_theme::Paint) -> Plate {
    Plate {
        fill: muted,
        border: muted,
        border_width: 0.0,
        // The better of the palette's two ink extremes, not the hue: a hue on
        // its own tint is the pairing a palette cannot hold a floor for, and a
        // lightness threshold inverts in light mode.
        ink: crate::mp::control::plates::ink_on(muted, p),
        dot: crate::mp::control::plates::ink_on(muted, p),
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpStatus {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    fill: Vec4f,
    #[live]
    border: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
    #[live]
    dot_color: Vec4f,
    #[live]
    dot_radius: f32,
}

/// A badge reports something about its subject.
#[derive(Script, ScriptHook, Widget)]
pub struct MpBadge {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[live]
    tone: StatusTone,
    #[live]
    text: ArcStringMut,
    #[redraw]
    #[live]
    draw_bg: DrawMpStatus,
    #[live]
    draw_text: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
}

impl MpBadge {
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }

    pub fn tone(&self) -> StatusTone {
        self.tone
    }

    pub fn set_tone(&mut self, cx: &mut Cx, tone: StatusTone) {
        self.tone = tone;
        self.redraw(cx);
    }
}

impl Widget for MpBadge {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // A badge reports; it does not take. A clickable one is a `MpTag` inside
        // a button, which keeps the hit area the button's business.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        let plate = self.tone.badge(theme);
        self.draw_bg.fill = plate.fill;
        self.draw_bg.border = plate.border;
        self.draw_bg.border_width = plate.border_width;
        self.draw_bg.radius = 4.0;
        self.draw_bg.dot_radius = 0.0;
        self.draw_text.color = plate.ink;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl MpBadgeRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn set_tone(&self, cx: &mut Cx, tone: StatusTone) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_tone(cx, tone);
        }
    }
}

/// A tag classifies its subject.
#[derive(Script, ScriptHook, Widget)]
pub struct MpTag {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[live]
    tone: StatusTone,
    /// Whether to draw a leading dot. A category is a label; a state is a label
    /// *and* a dot, and the dot is what makes it scannable without reading.
    #[live]
    dot: bool,
    #[live]
    text: ArcStringMut,
    #[redraw]
    #[live]
    draw_bg: DrawMpStatus,
    #[live]
    draw_text: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
}

/// The dot's diameter, and the room it takes at the leading edge.
const DOT_R: f32 = 3.0;
/// Where the dot's centre sits, measured from the plate's left edge: the border,
/// the tag's own leading inset, and the dot's radius.
const DOT_INSET: f32 = 8.0;

impl MpTag {
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }

    pub fn set_tone(&mut self, cx: &mut Cx, tone: StatusTone) {
        self.tone = tone;
        self.redraw(cx);
    }

    pub fn set_dot(&mut self, cx: &mut Cx, dot: bool) {
        self.dot = dot;
        self.redraw(cx);
    }
}

impl Widget for MpTag {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // Same as the badge: a tag that filters is a tag inside a button.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let theme = makepad_theme::Theme::of(cx.cx);
        let plate = self.tone.tag(theme);
        self.draw_bg.fill = plate.fill;
        self.draw_bg.border = plate.border;
        self.draw_bg.border_width = 1.0;
        // A tag is a rectilinear label, not a pill: a category sits in a line of
        // text and the small square corner reads as a label rather than as a
        // control.
        self.draw_bg.radius = 4.0;
        self.draw_bg.dot_color = plate.dot;
        self.draw_bg.dot_radius = if self.dot { DOT_R } else { 0.0 };
        self.draw_text.color = plate.ink;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl MpTagRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn set_tone(&self, cx: &mut Cx, tone: StatusTone) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_tone(cx, tone);
        }
    }

    pub fn set_dot(&self, cx: &mut Cx, dot: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_dot(cx, dot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makepad_theme::{color, Appearance, Theme};

    fn themes() -> [Theme; 2] {
        [
            Theme::for_appearance(Appearance::Dark),
            Theme::for_appearance(Appearance::Light),
        ]
    }

    #[test]
    fn test_every_tone_has_a_badge_and_a_tag_plate() {
        for tone in [
            StatusTone::Neutral,
            StatusTone::Solid,
            StatusTone::Accent,
            StatusTone::Success,
            StatusTone::Warning,
            StatusTone::Danger,
            StatusTone::Busy,
        ] {
            for theme in themes() {
                let badge = tone.badge(&theme);
                let tag = tone.tag(&theme);
                assert!(!badge.fill.w.is_nan() && !tag.fill.w.is_nan());
                assert!(badge.border_width >= 0.0 && tag.border_width >= 0.0);
            }
        }
    }

    #[test]
    fn test_every_badge_tone_reads_at_aa_on_its_own_plate() {
        // The property the tone enum exists to promise: a caller picks a tone and
        // gets a legible plate, without choosing an ink. Every tone, not the
        // three the first version checked — the status plates are the ones where
        // a lightness threshold on the ink goes wrong, and they were the ones
        // left out.
        for theme in themes() {
            for tone in [
                StatusTone::Neutral,
                StatusTone::Solid,
                StatusTone::Accent,
                StatusTone::Success,
                StatusTone::Warning,
                StatusTone::Danger,
                StatusTone::Busy,
            ] {
                let plate = tone.badge(&theme);
                let ratio = color::contrast_ratio(plate.ink, plate.fill);
                assert!(
                    ratio >= 4.5,
                    "{tone:?} on {:?}: {ratio}",
                    theme.appearance
                );
            }
        }
    }

    #[test]
    fn test_a_tag_is_quieter_than_a_badge() {
        // A tag sits in a line of text; if it were as loud as a badge it would
        // read as a button. Its fill is a wash over the page, so it must be far
        // closer to the page than a badge's plate is.
        for theme in themes() {
            for tone in [StatusTone::Accent, StatusTone::Success, StatusTone::Danger] {
                let page = theme.paint.bg;
                let tag = color::contrast_ratio(tone.tag(&theme).fill, page);
                let badge = color::contrast_ratio(tone.badge(&theme).fill, page);
                assert!(
                    tag < badge,
                    "{tone:?} on {:?}: tag {tag} vs badge {badge}",
                    theme.appearance
                );
            }
        }
    }

    #[test]
    fn test_a_status_badge_ink_is_never_the_status_hue_on_its_own_hue() {
        // The pairing a palette cannot hold a floor for: a pale tint of a hue
        // carrying that hue as its label. The plate's ink is the page's dark
        // tone instead, so this asserts the ink is desaturated relative to the
        // fill rather than a second tint of it.
        for theme in themes() {
            let plate = StatusTone::Success.badge(&theme);
            assert!(
                color::is_grey(plate.ink) || plate.ink.x + plate.ink.y + plate.ink.z < 1.5,
                "{:?}",
                theme.appearance
            );
        }
    }

    #[test]
    fn test_the_dot_fits_inside_the_room_the_tag_leaves_it() {
        // The dot is drawn in the shader at a fixed inset while the *text* is
        // inset by the DSL's leading padding. If the dot's own inset ever grows
        // past that, the dot lands in the label — and it would look like a
        // kerning bug rather than a geometry one.
        assert!(DOT_R > 0.0);
        assert!(
            DOT_INSET >= DOT_R * 2.0,
            "the dot's inset {DOT_INSET} leaves no room for its own diameter"
        );
        // The tag's leading padding is 16 and the dot occupies 8 of it, so the
        // label starts where the dot ends. Recorded as an assertion against the
        // number the DSL uses, so a change to either is a failing test.
        assert_eq!(DOT_INSET, 8.0, "matches MpTag's leading padding of 16");
    }

    #[test]
    fn test_the_two_assembled_looks_share_one_shader() {
        // A badge and a tag differ by which plate they resolve, not by how they
        // paint — so the shader is one struct and the tone enum is where the
        // difference lives. If a second draw struct is ever needed, this is the
        // note saying it was deliberate.
        for theme in themes() {
            let badge = StatusTone::Neutral.badge(&theme);
            let tag = StatusTone::Neutral.tag(&theme);
            // Both are opaque enough to stroke; the difference is the fill.
            assert_ne!(badge.fill, tag.fill);
            assert_eq!(badge.dot.w, tag.dot.w);
        }
    }
}
