//! The combobox, and the two things it holds that can disagree.
//!
//! A combobox is the only control in the library that holds **two** values: what is typed
//! and what is chosen. Every other control holds one. The two agree after a choice and
//! disagree the moment the reader types, and the question answered on every keystroke is
//! *"is what is in the field still the thing that was chosen?"*
//!
//! Getting it wrong is invisible in the way this port keeps running into: the field shows
//! `Split Right`, the app's value still says `New Terminal`, and nothing anywhere complains
//! — until the next action runs the wrong command.
//!
//! The four readouts below are the state itself rather than a description of it:
//! `query` is the field, `value` is the choice, `offered` is the filtered view in ranked
//! order, and `highlight` is the row Enter would take. The page is driven by an env script
//! because the screenshot script cannot type, and every step is printed so the state
//! machine is checkable from a run rather than from a picture.
//!
//! The panel is the Select page's composition unchanged — an `MpPopover` in the overlay
//! region with an `MpMenu` inside — because a floating surface cannot live inside the widget
//! that opens it: the overlay's draw list is clipped to its own rectangle. What is new here
//! is the state, not the panel.

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

    mod.gallery.pages.combobox = View{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        combobox_body := View{
            width: Fill
            height: Fill
            flow: Down
            spacing: 20

            Label{
                width: Fit, height: Fit
                draw_text +: {text_style: title2, color: text}
                text: "Combobox"
            }
            Label{
                width: Fill, height: Fit
                draw_text +: {text_style: footnote, color: text_muted}
                text: "A choice survives typing only while the text still names it. Type one more character and the value is cleared, because the field no longer says the thing the value claims — and a stale value is worse than no value, since nothing chosen is a state the app can handle and something else chosen is not. A cleared value does not come back by retyping: remembering the last choice would make the value a third thing, neither what the text says nor nothing at all, which is the same fault in a quieter costume."
            }

            View{
                width: 420, height: Fit, flow: Down, spacing: 10
                Caption{ text: "The field. Type into it and the panel offers matches; the app drives it from GALLERY_COMBOBOX in a run, because the screenshot script cannot deliver keystrokes" }
                combo_field := mod.mp.MpCombobox{}
            }

            View{
                width: Fill, height: Fit, flow: Down, spacing: 6
                combo_query := Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: caption, color: text}
                    text: "(query)"
                }
                combo_value := Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: caption, color: text}
                    text: "(value)"
                }
                combo_offered := Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: caption, color: text_faint}
                    text: "(offered)"
                }
                combo_highlight := Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: caption, color: text_faint}
                    text: "(highlight)"
                }
                combo_script := Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: caption, color: text_faint}
                    text: "(script)"
                }
            }

            Caption{ text: "And the rule stated as the one thing that must never happen: the field's text and the value must never name different items. While nothing is chosen the readout says so in as many words, and the moment the text stops naming the choice the value goes — so the invariant is visible rather than asserted." }
        }

        // ---- the floating surface, as a sibling of the content ----
        combo_panel := mod.mp.MpComboboxPanel{
            panel +: {
                width: 300
                combobox_list := mod.mp.MpMenu{}
            }
        }
    }
}
