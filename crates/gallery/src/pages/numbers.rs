//! `MpNumberInput` — a number, with a field and a step cluster.
//!
//! ## What the page is for
//!
//! Three of the four things below are cases where **the widget has to disagree with what it was handed**, and each is
//! invisible in a picture of the happy path:
//!
//! - a value **outside** its bounds, which is clamped — and shown clamped, because the widget holds one number rather
//!   than "the field says X but the value is Y";
//! - **bounds passed the wrong way round**, which are ordered — the widget this replaces **panicked** on that, because
//!   `f64::clamp` asserts `min <= max`;
//! - a **fractional step with two decimals**, where the grid and the formatting have to agree: stepping `2.50` by `0.5`
//!   must land on `3.00` and not on `2.9999999999999996`.
//!
//! ## And the one a picture cannot show at all
//!
//! That pressing the arrows moves the value by the step and **stops at the bounds** — a stepper that can walk past its
//! own maximum is not bounded.

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

    mod.gallery.pages.numbers = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Numbers"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A number with a field and a step cluster. The widget holds one number and the four things that constrain it — bounds, step, decimals — and nothing else: no second copy in the field to keep in step. The field shows the value's own formatting, so a caller and the widget cannot disagree about what is on screen. The arrows move by the step onto the grid and stop at the bounds, and text typed into the field is read back only when it parses, because a field showing 12 while somebody types 12.5 holds text that is not yet a number. Bounds passed the wrong way round are ordered rather than trusted — the widget this replaces panicked on that, because f64::clamp asserts min <= max — and a step of zero means no grid rather than the NaN it used to produce."
        }

        Section{
            Caption{ text: "The ordinary case: 0 to 100, step 1 — press the arrows" }
            numbers_plain := mod.mp.MpNumberInput{}
        }
        Section{
            Caption{ text: "A fractional step with two decimals: 0 to 10 by 0.5, and the grid and the formatting have to agree" }
            numbers_fractional := mod.mp.MpNumberInput{}
        }
        Section{
            Caption{ text: "Handed 999 with a maximum of 10 — the value is clamped, and the field shows the clamped value" }
            numbers_clamped := mod.mp.MpNumberInput{}
        }
        Section{
            Caption{ text: "Handed its bounds backwards — ordered rather than trusted, where the v2 widget panicked" }
            numbers_backwards := mod.mp.MpNumberInput{}
        }
    }
}
