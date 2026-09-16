//! The slider: continuous, stepped, ranged, and the readouts that prove the
//! value path.
//!
//! Each row is written out rather than built from a parametrised prototype.
//! A `let Row = View{ slider := MpSlider{} }` looks like it would work, and its
//! use site `Row{ slider +: {value: 0.35} }` does not: a named child (`:=`) goes
//! into the object's *vector* part and `+:` merges into its *map*, so the merge
//! reports "field slider not found in type-check". The rows are documentation
//! anyway, and written out they read as what they are.

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

    let Readout = Label{
        width: 90, height: Fit
        draw_text +: {text_style: footnote, color: text}
        text: "—"
    }

    mod.gallery.pages.slider = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Slider"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Grab anywhere on the track, not only the knob — a press sets the value to that point and the drag continues from there. Arrows nudge by a step, Home and End go to the ends, and Tab reaches it. The pixel-to-value arithmetic is four free functions in mp::slider with tests, rather than a method that needs a live Area to run."
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Continuous — the default. No step, so an arrow key moves a hundredth of the range" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                read_continuous := mod.mp.MpSlider{ value: 0.35 }
                out_continuous := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Stepped — 0..1 by 0.25. The value clicks onto ticks" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                read_stepped := mod.mp.MpSlider{ value: 0.5, step: 0.25 }
                out_stepped := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "A range that does not divide by its step — 0..1 by 0.3 offers 0, 0.3, 0.6, 0.9 and 1" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                read_thirds := mod.mp.MpSlider{ value: 0.9, step: 0.3 }
                out_thirds := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Anchored at min, not at zero — 5..10 by 2 offers 5, 7, 9" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                read_anchor := mod.mp.MpSlider{ value: 7.0, min: 5.0, max: 10.0, step: 2.0 }
                out_anchor := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Negative range — -3..3 by 1" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                read_negative := mod.mp.MpSlider{ value: -1.0, min: -3.0, max: 3.0, step: 1.0 }
                out_negative := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Small — the chip rung of the control ladder" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                read_small := mod.mp.MpSlider{
                    value: 0.6
                    control: mod.mpc.ControlSize.Small
                    height: mod.mpc.layout.control.small.height
                }
                out_small := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Disabled — no hit, no focus, and Tab skips it" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                mod.mp.MpSlider{ value: 0.4, disabled: true }
                Readout{ text: "disabled" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "A slider driving something — the value in, the progress bar out" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                drive_source := mod.mp.MpSlider{ value: 0.25, step: 0.05 }
                drive_bar := mod.mp.MpProgress{ value: 0.25, width: 200 }
            }
        }
    }
}
