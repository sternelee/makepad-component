//! Scaffolding: `MpGroupBox` and `MpKbd`.
//!
//! Two small things that every real screen needs and neither the surface ladder
//! nor the status set covers.
//!
//! ## `MpGroupBox` is a style, not a widget
//!
//! It is a titled card: a surface with a heading, an optional description, and a
//! body. There is no state, no action and no hit area, so it is a **DSL prototype
//! over the card the surface module already defines** rather than a widget —
//! exactly as [`mp::surface`](crate::mp::surface) argues for every container. Its
//! one decision is that the title row is *inside* the card's padding, so a group
//! box's heading starts where its body does; a heading that overhangs the
//! container's own inset reads as a section label that happens to sit near a box.
//!
//! ## `MpKbd` is a widget because it is shaped, not styled
//!
//! A key cap is a plate with a border, a mono face, and a size that has to sit on
//! the baseline of the text it annotates — three things that have to agree, and
//! three things a call site writes differently every time. Small enough to be
//! tempting to inline, which is why it is not.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    // ---- the group box ----
    //
    // A card with a heading. The heading row is inside the card's padding, so the
    // title starts where the body does.
    mod.mp.MpGroupBox = mod.mp.SurfaceCard{
        width: Fill
        height: Fit
        flow: Down
        spacing: 12

        header := View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 2

            title := Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: title3
                    color: text
                }
                text: "Group"
            }
            description := Label{
                width: Fill
                height: Fit
                visible: false
                draw_text +: {
                    text_style: footnote
                    color: text_muted
                }
                text: ""
            }
        }

        // The caller's children land here, so a group box without a body is a
        // heading with nothing under it — which is why `body` is a container
        // rather than the box itself: the heading must come first in the flow and
        // the caller's content cannot be asked to.
        body := View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 8
        }
    }

    // A group box with no visible frame, for grouping inside a card that already
    // has one.
    mod.mp.MpGroupBoxPlain = mod.mp.MpGroupBox{
        draw_bg +: {
            color: instance(vec4(0.0, 0.0, 0.0, 0.0))
            border_size: instance(0.0)
        }
        padding: Inset{left: 0, right: 0, top: 0, bottom: 0}
    }

    // ---- the stat card ----
    //
    // A labelled metric. No Rust: it is a card, three labels and a badge, and the
    // only decision is which of them the eye reaches first — the *value* takes the
    // title rung and the label takes the caption one, so a row of stat cards is
    // read by number and confirmed by label rather than the other way round.
    //
    // The delta is an `MpBadge` rather than a coloured label, so a rise and a fall
    // use the same six tones every other status in the library uses, and the tone
    // carries the semantics rather than the arrow's colour.
    mod.mp.MpStatCard = mod.mp.SurfaceCard{
        width: Fill
        height: Fit
        flow: Down
        spacing: 6

        stat_label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: caption
                color: text_muted
            }
            text: "Metric"
        }
        stat_value := Label{
            width: Fill
            height: Fit
            draw_text +: {
                // Deliberately a rung above the title the card's heading would
                // take: a stat card's number is the thing being read.
                text_style: title
                color: text
            }
            text: "0"
        }
        stat_trend := View{
            width: Fill
            height: Fit
            flow: Right
            spacing: 6
            align: Align{y: 0.5}
            stat_delta := mod.mp.MpBadgeSmall{
                tone: mod.mp.StatusTone.Neutral
                text: "—"
            }
            stat_note := Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: caption
                    color: text_faint
                }
                text: ""
            }
        }
    }

    // A row of them, which is how stat cards are actually used — the grid is part
    // of the component because a single stat card is a strange object and four in
    // a row are a dashboard.
    mod.mp.MpStatRow = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 12
        align: Align{y: 0.0}
        stat_a := mod.mp.MpStatCard{}
        stat_b := mod.mp.MpStatCard{}
        stat_c := mod.mp.MpStatCard{}
        stat_d := mod.mp.MpStatCard{}
    }

    // ---- the empty state ----
    //
    // A glyph, a heading, a sentence and an optional action. No Rust: it is the
    // one thing a list, a pane and a search result all need when there is nothing
    // to show, and the reason it is a component rather than a call site's
    // three labels is that the *rhythm* is the point — glyph, then space, then
    // heading, then a tighter space, then the sentence, and the action set apart
    // from all of it.
    mod.mp.MpEmptyState = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 8
        align: Align{x: 0.5, y: 0.0}
        padding: Inset{left: 24, right: 24, top: 32, bottom: 32}

        empty_glyph := mod.mp.MpIconLarge{
            width: 28
            height: 28
            glyph: "\u{f0f6}"
        }
        empty_title := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: title3
                color: text
            }
            text: "Nothing here"
        }
        empty_body := Label{
            width: Fill
            height: Fit
            align: Align{x: 0.5, y: 0.0}
            draw_text +: {
                text_style: footnote
                color: text_muted
            }
            text: ""
        }
        // The action is set apart from the prose by its own container, because
        // the gap above it is larger than the gaps inside the prose.
        empty_action := View{
            width: Fit
            height: Fit
            // A `Fill`-width container aligned to the centre, so the button inside
            // is centred without the container stretching.
            align: Align{x: 0.5, y: 0.0}
            padding: Inset{left: 0, right: 0, top: 8, bottom: 0}
            empty_action_slot := View{
                width: Fit
                height: Fit
                empty_action_btn := mod.mp.MpButton{
                    style: mod.mp.ButtonStyle.Prominent
                    text: "Create"
                }
            }
        }
    }

    // ---- the key cap ----
    set_type_default() do #(DrawMpKbd::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 1.0
        radius: 4.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, self.rect_size.y - bw * 2.0, max(1.0, self.radius))
            // `fill_keep`: the stroke below is the same box.
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }
            return sdf.result
        }
    }

    mod.mp.MpKbdBase = #(MpKbd::register_widget(vm))

    mod.mp.MpKbd = set_type_default() do mod.mp.MpKbdBase{
        width: Fit
        height: 18
        padding: Inset{left: 5, right: 5, top: 0, bottom: 0}
        align: Align{x: 0.5, y: 0.5}

        text: "K"
        draw_text +: {
            // The mono face, because a key cap names a key and a key is a glyph
            // rather than a word: `⌘` and `A` have to be the same width in a row
            // of caps, which a proportional face will not give.
            text_style: theme.font_code{font_size: 10.0}
            color: #x00000000
        }
    }

    mod.mp.MpKbdSmall = mod.mp.MpKbd{ height: 15 }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpKbd {
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
}

/// One key, as a cap.
#[derive(Script, ScriptHook, Widget)]
pub struct MpKbd {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[live]
    text: ArcStringMut,
    #[redraw]
    #[live]
    draw_bg: DrawMpKbd,
    #[live]
    draw_text: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
}

impl MpKbd {
    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }
}

impl Widget for MpKbd {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // A key cap is a label about a key, not the key: it does not take a
        // press, and the shortcut it names is handled wherever that shortcut
        // belongs.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (fill, border, ink) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            // A recessed plate with a hairline: a cap reads as a key because it is
            // *pressed into* the surface, not because it is raised off it. `band`
            // is the recessed strip token, which is exactly this.
            (p.band, p.border_strong, p.text_muted)
        };
        self.draw_bg.fill = fill;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_text.color = ink;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl MpKbdRef {
    pub fn text(&self) -> String {
        self.borrow().map(|inner| inner.text().to_string()).unwrap_or_default()
    }

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shapes a key cap has to hold: one glyph, a word, and a chord.
    const CAPS: [&str; 6] = ["A", "F1", "⌘", "Esc", "⌥⌘S", "Space"];

    #[test]
    fn test_every_cap_shape_is_a_single_line() {
        // A cap is one line by construction; a caption that wrapped would make the
        // whole row grow, and a shortcut row is dense by design.
        for cap in CAPS {
            assert!(!cap.contains('\n'), "{cap}");
        }
    }

    // Two tests used to live here asserting that `text::width` gives a modifier glyph
    // and a capital the same estimate, "because the mono face makes a row of caps
    // align". **The proxy was the wrong thing to assert, and it came apart.**
    //
    // `MpKbd` does not measure anything. It draws its text with `Walk::fit()` and lets
    // Makepad lay it out in `theme.font_code`, so the advance that aligns a row of caps
    // is the **font's** and this crate's estimator is not involved. When the estimator
    // gained a symbol correction — because a *proportional* placement was overshooting,
    // as `mp/text.rs` records — both tests went red while the property they stood for
    // was untouched. The estimator was never the reason a `MpKbd` lined up.
    //
    // The property itself cannot be asserted from here: it is a fact about a font file
    // and about Makepad's text layout, neither of which a unit test in this crate can
    // see. What is asserted instead is in `mp/text.rs`, where the estimator's own
    // behaviour belongs.
}
