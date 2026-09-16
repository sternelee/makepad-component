//! The controls page: the boolean family and the radio group.
//!
//! The page is also the family's behaviour test at runtime. Each row has a
//! label that reports the control's current value, so a missed action or a
//! component that paints but does not fire shows up on screen rather than only
//! in a log — and the radio group proves the caller-owned-group contract by
//! clearing its siblings from `App::handle_actions`.


use makepad_widgets::*;
script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    let Caption = Label{
        width: Fit
        height: Fit
        draw_text +: {text_style: caption, color: text_muted}
    }

    let Readout = Label{
        width: Fit
        height: Fit
        draw_text +: {text_style: footnote, color: text_muted}
        text: "—"
    }

    let Row = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 16
        align: Align{y: 0.5}
    }

    mod.gallery.pages.controls = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Controls"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Four controls, one shared contract. hover, press, focus and disabled come from mod.mp.ControlAnimator with the catalog's durations, and the pointer and keyboard behaviour comes from mp::control — so a control declares only what makes it itself. Tab to reach them; Enter or Space fires; a click takes focus."
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 10

            Caption{ text: "Checkbox — two independent states, and activating flips them" }
            Row{
                checkbox_one := mod.mp.MpCheckbox{ text: "Include dependencies" }
                mod.mp.MpCheckbox{ text: "Preselected", checked: true }
                mod.mp.MpCheckbox{ text: "Disabled", disabled: true }
                mod.mp.MpCheckbox{ text: "Disabled, checked", checked: true, disabled: true }
                mod.mp.MpCheckboxSmall{ text: "Small" }
            }
            Row{
                checkbox_readout := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 10

            Caption{ text: "Switch — the same value as a position, committed instantly" }
            Row{
                switch_one := mod.mp.MpSwitch{}
                switch_two := mod.mp.MpSwitch{ checked: true }
                mod.mp.MpSwitch{ disabled: true }
                mod.mp.MpSwitchSmall{}
                switch_readout := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 10

            Caption{ text: "Radio — one choice among several; the group is the caller's" }
            Row{
                radio_0 := mod.mp.MpRadio{ text: "Every day" }
                radio_1 := mod.mp.MpRadio{ text: "Weekly" }
                radio_2 := mod.mp.MpRadio{ text: "Never", selected: true }
                radio_3 := mod.mp.MpRadio{ text: "Disabled", disabled: true }
            }
            Row{
                radio_readout := Readout{}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 10

            Caption{ text: "Layout — the box is the row's own height, so a control is a row rather than a box with a label beside it, and a caller can size either independently" }
            Row{
                mod.mp.MpCheckbox{ text: "Default height" }
                mod.mp.MpSwitch{ height: 32 }
                mod.mp.MpSwitch{ width: 64, height: 32 }
                mod.mp.MpSwitchSmall{}
            }
        }
    }
}
