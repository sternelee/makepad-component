//! The loaders, running.
//!
//! This page is the only way to check a loader: a period and a stagger read off
//! a table are numbers, and a wave that is wrong is wrong in a way you have to
//! watch. Everything here is driven by a looping animator, so nothing on this
//! page is a static picture.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    let Caption = Label{
        width: Fit, height: Fit
        draw_text +: {text_style: caption, color: text_muted}
    }

    let Section = View{
        width: Fill, height: Fit, flow: Down, spacing: 10
    }

    mod.gallery.pages.loaders = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Loaders"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Every number on this page comes from motion::phase — the period, the resting opacity, the per-cell stagger — injected into the shaders as instances from Rust rather than restated in the DSL, because a stagger that drifts between two loaders is invisible until they sit side by side."
        }

        Section{
            Caption{ text: "Spinner — 750ms, linear, looped. The comet's tail is the falloff, so the head reads as the present and the tail as where it has been" }
            mod.mp.Row{
                mod.mp.MpSpinnerSmall{}
                mod.mp.MpSpinner{}
                mod.mp.MpSpinnerLarge{}
                spinner_gap := View{ width: 24, height: 1 }
                Label{
                    width: Fit, height: Fit
                    draw_text +: {text_style: body, color: text_muted}
                    text: "Loading…"
                }
            }
        }

        Section{
            Caption{ text: "Pulse — 2400ms, three cells, staggered. Slower on purpose: these breathe rather than tick, and a tick at this size reads as impatience" }
            mod.mp.Row{
                mod.mp.MpPulse{}
                Label{
                    width: Fit, height: Fit
                    draw_text +: {text_style: body, color: text_muted}
                    text: "Thinking…"
                }
            }
        }

        Section{
            Caption{ text: "Progress — determinate. There is no determinate spinner, because a spinner that knows how far along it is, is this" }
            Column{
                width: Fill, height: Fit, flow: Down, spacing: 10
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                    Label{ width: 40, height: Fit, draw_text +: {text_style: caption, color: text_faint} text: "0%" }
                    mod.mp.MpProgress{ value: 0.0 }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                    Label{ width: 40, height: Fit, draw_text +: {text_style: caption, color: text_faint} text: "25%" }
                    mod.mp.MpProgress{ value: 0.25 }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                    Label{ width: 40, height: Fit, draw_text +: {text_style: caption, color: text_faint} text: "62%" }
                    mod.mp.MpProgress{ value: 0.62 }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                    Label{ width: 40, height: Fit, draw_text +: {text_style: caption, color: text_faint} text: "100%" }
                    mod.mp.MpProgress{ value: 1.0 }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                    Label{ width: 40, height: Fit, draw_text +: {text_style: caption, color: text_faint} text: "4pt" }
                    mod.mp.MpProgress{ value: 0.5, height: 4 }
                }
            }
        }
    }
}
