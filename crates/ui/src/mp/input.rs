//! `MpTextInput` — the text field, and why it is the one component with no Rust.
//!
//! Every other component in this crate owns its shader and resolves the theme in
//! Rust at paint time. A text field cannot: the hard parts of one are the caret,
//! the selection, the IME composition, the scroll-into-view and the platform key
//! handling, and Makepad already ships all of them in `widgets::TextInput`. So
//! the honest thing is to *style* it rather than reimplement it — and a style in
//! Makepad belongs in the DSL, where the theme namespaces are.
//!
//! That makes this module a table of token assignments, and the reason it is
//! worth having is that the assignments are not obvious: Makepad's field has
//! fourteen colour slots across four layers (`draw_bg`, `draw_text`,
//! `draw_selection`, `draw_cursor`), each with `hover`/`focus`/`down`/`empty`/
//! `disabled` variants, and a wrapper that fills in only the base `color` leaves
//! the rest of them the stock theme's.
//!
//! ## What is themed, and what is not
//!
//! | Layer | Themed |
//! |---|---|
//! | `draw_bg` | the well's five states plus the disabled one, the bevel's, and the corner |
//! | `draw_text` | body ink, its hover and focus, the placeholder in both, disabled |
//! | `draw_selection` | the selection band, from the palette's `selection` |
//! | `draw_cursor` | the caret, from the palette's `caret` — the same token a focus ring uses |
//!
//! Not themed: the scroll bar (a multiline field grows one, and it is a
//! component of its own — [`scroll`](crate::mp) has not landed yet) and the
//! numeric/password toggles, which are behaviour rather than style.
//!
//! ## The ladder
//!
//! Height, padding and the text style come from the same `mod.mpc.layout` and
//! `mod.mpc.type` the controls read, so a field and a button sitting in one row
//! are the same height without either naming a number.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.layout.*
    use mod.mpc.type.*

    // The themed field. Everything below is a token assignment; there is no
    // pixel function here on purpose.
    mod.mp.MpTextInput = mod.widgets.TextInputFlat{
        width: Fill
        height: control.regular.height
        padding: Inset{
            left: control.regular.pad_x
            right: control.regular.pad_x
            top: control.regular.pad_y
            bottom: control.regular.pad_y
        }
        // A field is its own row, so it carries no margin of its own; the
        // container's `spacing` is what separates it from its neighbours, and
        // two sources of the same gap is one too many.
        margin: Inset{left: 0, right: 0, top: 0, bottom: 0}

        empty_text: "Placeholder"

        // ---- the well ----
        draw_bg +: {
            border_radius: uniform(8.0)

            color: input_bg
            color_hover: input_bg
            // Focus raises the field off the page rather than tinting it: the
            // ring says "this has the keyboard" and the fill says "this is the
            // plane you are typing on".
            color_focus: surface_card
            color_down: input_bg
            color_empty: input_bg
            color_disabled: input_bg

            border_size: uniform(1.0)
            border_color: border
            border_color_hover: border_strong
            border_color_focus: caret
            border_color_down: border_strong
            border_color_empty: border
            border_color_disabled: border

            // No gradient: the two-stop bevel is a Makepad chrome look, and this
            // palette separates planes by tone and edge instead.
            color_2: vec4(-1)
            border_color_2: vec4(-1)
        }

        // ---- the ink ----
        draw_text +: {
            color: text
            color_hover: text
            color_focus: text
            color_down: text
            color_disabled: text_faint
            color_empty: text_faint
            color_empty_hover: text_muted
            color_empty_focus: text_faint

            text_style: body
        }

        // ---- the selection band ----
        draw_selection +: {
            color: selection
            color_hover: selection
            color_focus: selection
            color_down: selection
            color_empty: selection
            color_disabled: selection
            color_2: vec4(-1)
        }

        // ---- the caret ----
        //
        // `caret` is the same token a focus ring paints, so the two things that
        // say "the keyboard is here" are one colour.
        draw_cursor +: {
            color: uniform(caret)
        }
    }

    mod.mp.MpTextInputSmall = mod.mp.MpTextInput{
        height: control.small.height
        padding: Inset{
            left: control.small.pad_x
            right: control.small.pad_x
            top: control.small.pad_y
            bottom: control.small.pad_y
        }
        draw_text +: {text_style: callout}
    }

    mod.mp.MpTextInputLarge = mod.mp.MpTextInput{
        height: control.large.height
        padding: Inset{
            left: control.large.pad_x
            right: control.large.pad_x
            top: control.large.pad_y
            bottom: control.large.pad_y
        }
        draw_text +: {text_style: title3}
    }

    /// A field with a label above it — the shape a form row actually takes.
    mod.mp.MpField = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 4

        field_label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: caption
                color: text_muted
            }
            text: "Label"
        }
        field_input := mod.mp.MpTextInput{}
    }

    /// A field that fills the rest of its row, for a search box beside a button.
    mod.mp.MpTextInputSearch = mod.mp.MpTextInput{
        empty_text: "Search"
        // Searching is a continuous act, so the field is a well with a soft
        // edge rather than a plate: the border stays faint even on focus, and
        // the caret alone says where the keyboard is.
        draw_bg +: {
            border_color_focus: border_strong
            color_focus: input_bg
        }
    }
}
