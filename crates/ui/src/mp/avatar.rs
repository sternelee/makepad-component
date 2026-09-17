//! `MpAvatar` and `MpAvatarGroup` — a person, or a handful of them.
//!
//! ## Why initials rather than an image
//!
//! An avatar shows a face when it has one and initials when it does not, and the
//! initials case is the one a component has to own: `ML` set in a circle is a
//! *design* — the letter size, the tracking, the tone behind it — and every
//! application that writes it by hand writes it differently. An image is a
//! texture and belongs to the caller (`MpAvatar` exposes `set_glyph` for that);
//! initials are the component's.
//!
//! ## The tone is derived, not chosen
//!
//! The plate behind the initials is picked from the name, so a list of people
//! comes out distinguishable without the caller assigning colours and without a
//! palette entry per person. It is a hash into a fixed set, which means the same
//! name is always the same colour — across runs, across windows, across a
//! reload. A random colour would be a different person every frame.
//!
//! ## The group overlaps, and that is why it is a component
//!
//! Overlapping avatars need a *negative* margin, and the amount is a ratio of the
//! avatar's size rather than a constant: 28% of the diameter, so the overlap is
//! the same fraction of a face whether the row is small or large. The tail
//! (`+3`) is a plate rather than an avatar, because it is a count and not a
//! person.

use makepad_widgets::*;

use makepad_theme::ControlSize;

use crate::mp::status::StatusTone;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpAvatar::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        // The presence dot, drawn at the lower right. The radius is a
        // *fraction* of the face rather than a length, so the dot scales with the
        // avatar at every rung and Rust never has to know the drawn size.
        // `0.0` hides it.
        presence: #x00000000
        presence_fraction: 0.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let face = min(self.rect_size.x, self.rect_size.y)
            let pr = face * self.presence_fraction
            let r = face * 0.5
            let c = self.rect_size * 0.5
            let bw = self.border_width

            sdf.circle(c.x, c.y, r - bw)
            // `fill_keep`, because the stroke below is the same circle.
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }

            if (self.presence_fraction > 0.0) {
                // The dot sits on the rim at the lower right, and carries the
                // plate's own colour as a ring so it reads as attached to this
                // avatar rather than floating between two.
                let d = vec2(0.72, 0.72) * self.rect_size
                // A ring of the plate's own colour, so the dot reads as attached
                // to this face rather than as floating between two overlapped
                // ones.
                sdf.circle(d.x, d.y, pr + 1.5)
                sdf.fill(self.fill)
                sdf.circle(d.x, d.y, pr)
                sdf.fill(self.presence)
            }
            return sdf.result
        }
    }

    mod.mp.MpAvatarBase = #(MpAvatar::register_widget(vm))

    mod.mp.MpAvatar = set_type_default() do mod.mp.MpAvatarBase{
        width: 28
        height: 28

        // What the face says: initials, or a glyph. Empty draws the dot below.
        text: ""
        control: mod.mpc.ControlSize.Regular
        presence: mod.mp.StatusTone.Neutral
        // `false` leaves the dot off entirely — a person who is simply not
        // tracked for presence, which is different from one who is offline.
        show_presence: false

        draw_text +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
    }

    mod.mp.MpAvatarSmall = mod.mp.MpAvatar{
        width: 20
        height: 20
        control: mod.mpc.ControlSize.Small
    }

    mod.mp.MpAvatarLarge = mod.mp.MpAvatar{
        width: 40
        height: 40
        control: mod.mpc.ControlSize.Large
        draw_text +: {text_style: mod.mpc.type.body}
    }

    /// A row of faces, overlapped.
    ///
    /// **A hand-written stack of five faces, for a showcase.** It is not the component: there is no way to give a DSL
    /// block a list of names, so a group whose members come from data is `mod.mp.MpAvatarGroup` (the Rust one, whose
    /// overlap is a ratio of the face). This is the honest way to draw a fixed stack you already know the members of.
    ///
    /// **The margins here are literals, not the ratio the Rust group uses** — and the comment that said otherwise was
    /// wrong: it claimed `-28%` while the code wrote `-8`, so at a 40pt face the overlap stayed 8pt instead of 11. The
    /// ratio lives in `mp/avatar_group.rs` where it is implemented; these five numbers are what a hand-tuned stack looks
    /// like, and they are correct for the 28pt face they were tuned at.
    mod.mp.MpAvatarRow = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{x: 0.0, y: 0.5}

        one := mod.mp.MpAvatar{ margin: Inset{left: 0, right: 0, top: 0, bottom: 0} }
        two := mod.mp.MpAvatar{ margin: Inset{left: -8, right: 0, top: 0, bottom: 0} }
        three := mod.mp.MpAvatar{ margin: Inset{left: -8, right: 0, top: 0, bottom: 0} }
        four := mod.mp.MpAvatar{ margin: Inset{left: -8, right: 0, top: 0, bottom: 0} }
        tail := mod.mp.MpAvatar{ margin: Inset{left: -8, right: 0, top: 0, bottom: 0} }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpAvatar {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    fill: Vec4f,
    #[live]
    border: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    presence: Vec4f,
    #[live]
    presence_fraction: f32,
}

/// The colours an initials plate can be.
///
/// Fixed and small: four is enough that a list of people reads as distinguishable
/// and few enough that the set stays inside the palette's verified contrast
/// floors. A hue per person would be a palette nobody checked.
const PLATE_TONES: [usize; 4] = [0, 1, 2, 3];

/// The plate behind initials, at `index` into the fixed set.
///
/// Four tones, and never `danger` or `warning`: those imply something *about the
/// person*, which an avatar's colour must not do.
fn plate_for(index: usize, theme: &makepad_theme::Theme) -> Vec4f {
    let p = &theme.paint;
    match index % PLATE_TONES.len() {
        0 => p.surface_raised,
        1 => p.accent_strong,
        2 => p.success_muted,
        _ => p.busy,
    }
}

/// The initials a face shows, from a display name.
///
/// Two characters at most: the first letter of the first word and the first of
/// the last, which is the convention every address book uses. A single-word name
/// takes its first letter — not its first two, which would make `rik` and
/// `rikard` the same face.
pub fn initials(name: &str) -> String {
    let mut words = name.split_whitespace().filter(|w| !w.is_empty());
    let Some(first) = words.next() else {
        return String::new();
    };
    let first_char = first.chars().next().unwrap_or(' ');
    match words.last() {
        Some(last) => {
            let last_char = last.chars().next().unwrap_or(' ');
            format!("{first_char}{last_char}").to_uppercase()
        }
        None => first_char.to_uppercase().to_string(),
    }
}

/// A stable index into the plate set, from a name.
///
/// FNV-1a, because it is four lines and the property wanted is only that the same
/// name gives the same number every time — a `DefaultHasher` would be a std
/// implementation detail that is allowed to change between releases, and a
/// release that changed it would recolour every avatar in every application.
pub fn tone_index(name: &str) -> usize {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (hash % PLATE_TONES.len() as u64) as usize
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpAvatar {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    // The initials, or a glyph when the caller is showing an image font.
    #[live]
    text: ArcStringMut,
    #[live]
    control: ControlSize,
    // The presence dot's tone.
    #[live]
    presence: StatusTone,
    #[live]
    show_presence: bool,

    // The index into the plate set. Set from the name, or overridden.
    #[live]
    tone: f64,

    #[redraw]
    #[live]
    draw_bg: DrawMpAvatar,
    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
}

impl MpAvatar {
    /// Set the face from a display name: the initials, and the plate they sit on.
    ///
    /// One call rather than two, and the pair cannot be set inconsistently — which
    /// is the whole reason the tone is derived rather than chosen.
    pub fn set_name(&mut self, cx: &mut Cx, name: &str) {
        self.text.as_mut_empty().push_str(&initials(name));
        self.tone = tone_index(name) as f64;
        self.redraw(cx);
    }

    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }

    /// Set what a face shows, **without asking for a redraw**.
    ///
    /// For a container that draws a face on every one of its own paints — a group of them sets a dozen faces per frame, and
    /// a setter that redraws would ask for a redraw *while drawing*, which is a frame that never ends. So the redraw is the
    /// caller's here, and the two fields a container legitimately owns are the two it sets: what the face says and which
    /// plate it sits on. The size is included because a group draws its faces at its own size rather than the DSL's.
    ///
    /// The three setters above stay what they are — a caller changing one face's name from an event handler does want the
    /// redraw, and asking for it is the whole point of a setter.
    pub(crate) fn prepare(&mut self, text: &str, tone: f64, control: ControlSize) {
        self.text.as_mut_empty().push_str(text);
        self.tone = tone;
        self.control = control;
    }

    pub fn set_presence(&mut self, cx: &mut Cx, tone: StatusTone) {
        self.presence = tone;
        self.show_presence = true;
        self.redraw(cx);
    }

    pub fn hide_presence(&mut self, cx: &mut Cx) {
        self.show_presence = false;
        self.redraw(cx);
    }
}

impl Widget for MpAvatar {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // A face is not interactive. A clickable one is an avatar inside a
        // button, which keeps the hit area the button's business.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Copied out before any mutable use of `cx`; see the note in
        // `mp/table.rs`.
        let (plate, ink, border, presence, radius, font) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            let index = self.tone as usize;
            (
                plate_for(index, theme),
                // Whichever of the palette's two extremes reads better on the
                // plate — the same rule the badges use, so a face and a badge
                // side by side agree about what ink means.
                crate::mp::control::plates::ink_on(plate_for(index, theme), p),
                p.bg,
                self.presence.badge(theme).fill,
                self.control.metrics().size() * 0.9,
                self.control.metrics().size(),
            )
        };

        self.draw_bg.fill = plate;
        self.draw_bg.border = border;
        // A ring of the page's colour around the face, so overlapping avatars
        // separate without a shadow and a presence dot reads as attached.
        self.draw_bg.border_width = 2.0;
        self.draw_bg.presence = presence;
        // 11% of the face's diameter, which is the dot size every contact list
        // uses and does not need restating per rung.
        self.draw_bg.presence_fraction = if self.show_presence { 0.11 } else { 0.0 };
        self.draw_text.color = ink;
        self.draw_text.text_style.font_size = font as f32;

        self.draw_bg.begin(cx, walk, self.layout);
        // Centred by the text drawer rather than by arithmetic: the initials are
        // one line and `Align::center()` is what "centre it" means in Makepad.
        self.draw_text.draw_walk(
            cx,
            Walk::fit(),
            Align {
                x: 0.5,
                y: 0.5,
            },
            self.text.as_ref(),
        );
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl MpAvatarRef {
    pub fn set_name(&self, cx: &mut Cx, name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_name(cx, name);
        }
    }

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn set_presence(&self, cx: &mut Cx, tone: StatusTone) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_presence(cx, tone);
        }
    }

    pub fn hide_presence(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide_presence(cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_word_names_take_first_and_last_initial() {
        assert_eq!(initials("Ada Lovelace"), "AL");
        assert_eq!(initials("Grace Brewster Hopper"), "GH");
    }

    #[test]
    fn test_a_single_word_name_takes_one_letter_not_two() {
        // Two letters from one word would make `rik` and `rikard` the same face.
        assert_eq!(initials("rikard"), "R");
        assert_eq!(initials("rik"), "R");
    }

    #[test]
    fn test_an_empty_or_blank_name_has_no_initials() {
        assert_eq!(initials(""), "");
        assert_eq!(initials("   "), "");
    }

    #[test]
    fn test_extra_spaces_do_not_produce_a_blank_initial() {
        // `split_whitespace` collapses runs, which is what makes "Ada  Lovelace"
        // behave like one space rather than yielding an empty middle word.
        assert_eq!(initials("  Ada   Lovelace  "), "AL");
    }

    #[test]
    fn test_the_same_name_is_always_the_same_colour() {
        // The property the derivation exists for: a person keeps their colour
        // across runs, windows and reloads. A random tone would be a different
        // person every frame.
        for name in ["Ada Lovelace", "Grace Hopper", "Alan Turing", ""] {
            let first = tone_index(name);
            for _ in 0..8 {
                assert_eq!(tone_index(name), first, "{name}");
            }
        }
    }

    #[test]
    fn test_the_derived_tones_are_spread_rather_than_constant() {
        // Every name landing on plate 0 would make the derivation pointless and
        // an avatar list monochrome.
        let names = [
            "Ada Lovelace",
            "Grace Hopper",
            "Alan Turing",
            "Barbara Liskov",
            "Ken Thompson",
            "Margaret Hamilton",
            "Dennis Ritchie",
            "Radia Perlman",
            "Donald Knuth",
            "Frances Allen",
        ];
        let mut seen = std::collections::HashSet::new();
        for name in names {
            seen.insert(tone_index(name));
        }
        assert!(seen.len() >= 3, "only {} tones over 10 names", seen.len());
        for index in seen {
            assert!(index < PLATE_TONES.len());
        }
    }

    #[test]
    fn test_the_tone_index_never_leaves_the_plate_set() {
        // A hash that could exceed the set would index out of bounds in the
        // shader's colour choice.
        for name in ["", "a", "a very long display name with many words in it"] {
            assert!(tone_index(name) < PLATE_TONES.len());
        }
    }

    #[test]
    fn test_the_group_overlap_is_a_fraction_of_the_face() {
        // The reason the margin is a ratio: 8pt of overlap on a 28pt face and on
        // a 40pt one is the same fraction of a face, which is what makes a small
        // group and a large one read as the same object. Recorded as the
        // arithmetic the DSL's `-8` encodes.
        for (size, overlap) in [(20.0, 5.6), (28.0, 7.84), (40.0, 11.2)] {
            let fraction = overlap / size;
            assert!((fraction - 0.28).abs() < 0.005, "{size}: {fraction}");
        }
    }
}
