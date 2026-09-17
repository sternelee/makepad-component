//! The select: a face, a panel, and the two lines that join them.
//!
//! ## Why this is a page and not a widget
//!
//! A floating surface cannot be composed into a trigger widget — an overlay draw
//! list clips to its rectangle, so a `Fill` panel inside a trigger-sized box has
//! nowhere to draw. `MpTooltipArea` was an attempt to escape that and it could not
//! work. So a select is a **composition**: a trigger face in the flow, an
//! `MpPopover` in the page's overlay region, an `MpList` of options inside it, and
//! the app joins them.
//!
//! That is the template for the rest of the family — combobox, date picker, menu —
//! and the reason this page exists rather than a `MpSelect` widget: the wiring is
//! three lines, and a widget would have to invent an API for option rows that
//! `MpList` already provides.
//!
//! ```text
//! face := MpButton{ glyph_trailing: true, glyph: chevron, text: <value> }
//! panel := MpPopover{ panel +: { options := MpList{} } }   // in the overlay region
//! ```
//!
//! ## What the page checks
//!
//! The option list is an `MpList`, so the panel gets hover, selection and arrow
//! keys for free — which is what makes this composition worth preferring over a
//! hand-rolled set of buttons. The faces are `MpButton`s with a trailing chevron,
//! which is the reason the button grew a `glyph` property.

use makepad_component::mp::list::ListItem;
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
        width: Fill, height: Fit, flow: Down, spacing: 8
    }

    let Row = View{
        width: Fill, height: Fit, flow: Right, spacing: 10, align: Align{y: 0.5}
    }

    // A select's face: the value, with the disclosure chevron trailing. A button
    // with `glyph_trailing` is exactly that, which is why there is no separate
    // `MpSelectFace` widget.
    let Face = mod.mp.MpButton{
        glyph: "\u{f0d7}"
        glyph_trailing: true
        text: "Choose"
    }

    mod.gallery.pages.select = View{
        // The overlay region. Every panel on this page is a `Fill`/`Fill` sibling
        // of the content, here — a requirement rather than a layout preference.
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        select_body := View{
            width: Fill
            height: Fill
            flow: Down
            spacing: 20

            Label{
                width: Fit, height: Fit
                draw_text +: {text_style: title2, color: text}
                text: "Select"
            }
            Label{
                width: Fill, height: Fit
                draw_text +: {text_style: footnote, color: text_muted}
                text: "A composition rather than a widget, and not by choice: a floating surface cannot live inside its trigger, because an overlay draw list clips to the rectangle it draws in. So a select is a face in the flow, a popover in the page's overlay region, and a list of options inside it — which is the same three things a combobox, a date picker and a menu are, with different contents. The options are an `MpList`, so they get hover, selection and arrow keys for free."
            }

            Section{
                Caption{ text: "A form row — the face shows the value, the panel is the options" }
                Row{
                    Label{
                        width: 120, height: Fit
                        draw_text +: {text_style: body, color: text}
                        text: "Sync every"
                    }
                    select_face_a := Face{}
                }
            }

            Section{
                Caption{ text: "A second one, to show that two selects on a page do not interfere — each face anchors its own panel" }
                Row{
                    Label{
                        width: 120, height: Fit
                        draw_text +: {text_style: body, color: text}
                        text: "Region"
                    }
                    select_face_b := Face{}
                }
            }

            Section{
                Caption{ text: "The value path — what a selection writes back into the face, and into whatever else is listening" }
                Row{
                    select_face_c := Face{}
                    select_readout := Label{
                        width: Fit, height: Fit
                        draw_text +: {text_style: footnote, color: text_muted}
                        text: "nothing chosen yet"
                    }
                }
            }
        }

        // ---- the floating surfaces ----
        select_panel_a := mod.mp.MpPopover{
            panel +: {
                width: 220
                select_list_a := mod.mp.MpList{}
            }
        }
        select_panel_b := mod.mp.MpPopover{
            panel +: {
                width: 220
                select_list_b := mod.mp.MpList{}
            }
        }
        select_panel_c := mod.mp.MpPopover{
            panel +: {
                width: 220
                select_list_c := mod.mp.MpList{}
            }
        }
    }
}
