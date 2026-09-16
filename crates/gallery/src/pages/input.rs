//! The text field: the ladder, the placeholder, and the states.
//!
//! A field's states are the whole point of looking at it — an unfocused well and
//! a focused one differ by an edge and a plane, and neither is checkable in a
//! test that has no window.

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

    let Out = Label{
        width: Fill, height: Fit
        draw_text +: {text_style: caption, color: text_faint}
        text: "—"
    }

    mod.gallery.pages.input = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Text field"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "The one component here with no Rust. The caret, the selection, the IME, the scroll-into-view and the platform keys are Makepad's — reimplementing them would be the largest and least interesting file in the crate — so this is a table of token assignments across four layers and fourteen colour slots, which is the part that is actually missing from a raw TextInput."
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "The ladder — small, regular and large are the same three rungs a button has, so a row of them lines up without anyone naming a height" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                mod.mp.MpTextInputSmall{ empty_text: "Small" }
                mod.mp.MpTextInput{ empty_text: "Regular" }
                mod.mp.MpTextInputLarge{ empty_text: "Large" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Placeholder — `text_faint` at rest, `text_muted` on hover, so an empty field says what it wants without looking like content" }
            typed := mod.mp.MpTextInput{ text: "Typed into", empty_text: "Type here" }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "The form field — a label above, the input below. `MpField` is the pair" }
            field_one := mod.mp.MpField{
                field_label +: {text: "Repository"}
                field_input +: {text: "makepad-component"}
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Search — a well rather than a plate: the edge stays faint on focus, and the caret alone says where the keyboard is" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                search_box := mod.mp.MpTextInputSearch{}
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Prominent, text: "Go" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Read-only and password — behaviour props, untouched by the theme" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                mod.mp.MpTextInput{ text: "Read only", is_read_only: true }
                mod.mp.MpTextInput{ text: "hunter2", is_password: true }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "What it reports — `Changed` on every keystroke, `Returned` on Enter" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                echo_input := mod.mp.MpTextInput{ empty_text: "Type and watch" }
                echo_out := Out{}
            }
        }
    }
}
