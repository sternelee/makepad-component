//! The motion catalog, playing.
//!
//! The one page that has to be live: a duration read off a table is a number,
//! and a duration *seen* against its neighbours is the only way to tell whether
//! the catalog is right. Each row replays on click.


use makepad_widgets::*;
script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*
    use mod.motion.*

    // A track and a head: the head slides the width of the track on the row's
    // curve and duration, so two rows next to each other are directly
    // comparable — which is the whole reason to look at this page.
    let Track = View{
        width: 220
        height: 24
        align: Align{x: 0.0, y: 0.5}

        groove := mod.mp.SurfaceSunken{
            width: Fill, height: 4
            draw_bg +: {
                color: instance(element_active)
                border_size: instance(0.0)
                border_radius: instance(2.0)
            }
        }

        head := View{
            width: Fit, height: Fit
            mod.mp.SurfaceRaised{
                width: 16, height: 16
                draw_bg +: {border_radius: instance(8.0)}
            }
        }
    }

    let MotionRow = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 12
        align: Align{y: 0.5}

        label := Label{
            width: 150
            height: Fit
            draw_text +: {text_style: footnote, color: text}
            text: "spec"
        }
        track := Track{}
        timing := Label{
            width: 160
            height: Fit
            draw_text +: {text_style: caption, color: text_muted}
            text: ""
        }
    }

    mod.gallery.pages.motion = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Motion"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "One catalog, no inline durations. Every curve here is a CSS cubic-bezier evaluated exactly and handed to the animator as Ease.Bezier — not snapped to the nearest named ease. Click a row to replay it."
        }

        // The catalog, longest-lived first so the eye calibrates on the slowest.
        rows := View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            row_pulse := MotionRow{ label +: {text: "pulse"}, timing +: {text: "2400ms — a loader, it repeats"} }
            row_fade_in := MotionRow{ label +: {text: "fade_in"}, timing +: {text: "500ms expo-out — an entrance"} }
            row_scroll_glide := MotionRow{ label +: {text: "scroll_glide"}, timing +: {text: "500ms in-out — a long travel"} }
            row_resize := MotionRow{ label +: {text: "resize"}, timing +: {text: "200ms out — a pane settling"} }
            row_chevron := MotionRow{ label +: {text: "chevron"}, timing +: {text: "200ms — a disclosure turning"} }
            row_dialog_in := MotionRow{ label +: {text: "dialog_in"}, timing +: {text: "180ms — it takes the window"} }
            row_collapse := MotionRow{ label +: {text: "collapse"}, timing +: {text: "180ms out — closing"} }
            row_hover_fade := MotionRow{ label +: {text: "hover_fade"}, timing +: {text: "150ms tailwind — the state change"} }
            row_tab_slide := MotionRow{ label +: {text: "tab_slide"}, timing +: {text: "150ms out — a selection moving"} }
            row_menu_in := MotionRow{ label +: {text: "menu_in"}, timing +: {text: "140ms — a popover arriving"} }
            row_menu_out := MotionRow{ label +: {text: "menu_out"}, timing +: {text: "100ms — leaving, faster than arriving"} }
            row_press := MotionRow{ label +: {text: "press"}, timing +: {text: "90ms — the plate answers first"} }
        }

        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: caption, color: text_faint}
            text: "Every duration above is scaled by motion::speed_scale(), one process-wide multiplier — which is where a Reduce Motion setting hooks in, and why a test can run a two-second animation without waiting two seconds."
        }
    }
}
