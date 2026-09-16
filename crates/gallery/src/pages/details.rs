//! `MpDescriptionList` — key/value rows beside a thing.
//!
//! ## What the page is for
//!
//! Two things are worth looking at here and neither is visible from the API: **the bound** (eight slots, and what
//! happens at and past it) and **the rule that the last visible row has no hairline under it**. The first list below has
//! four rows, the second has exactly [`SLOTS`], and the third has **two more than fit** — so the page shows a truncated
//! list rather than only a fitting one, which is the case a caller is most likely to meet by accident.
//!
//! ## One thing the page settles that a test cannot
//!
//! Whether the label column and the value column line up when the value wraps to two lines. The widths are the widget's
//! own (`120` for the label, `Fill` for the value), so a wrap is the only way the row's alignment can go wrong, and one
//! of the values below is deliberately long enough to wrap.

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

    mod.gallery.pages.details = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Details"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Key/value rows, for the detail beside a thing. The list declares eight row slots and fills the first N, which is a bound rather than a growable pool: a row here is two labels and a hairline with no identity of its own, so filling and hiding eight is simpler to reason about than instantiating them — and a detail list with more than eight rows is a table. The hairline sits between rows, so the last visible row has none, which is a rule and not a detail. Colours come from the theme, so an appearance change re-colours it without re-declaring it."
        }

        Section{
            Caption{ text: "Four rows — inside the bound, so nothing is hidden" }
            details_four := mod.mp.MpDescriptionList{}
        }
        Section{
            Caption{ text: "Exactly eight, the bound — the last slot is used and still has no hairline under it" }
            details_full := mod.mp.MpDescriptionList{}
        }
        Section{
            Caption{ text: "Ten rows offered to eight slots — the list truncates rather than growing, and a value long enough to wrap shows where the columns land" }
            details_over := mod.mp.MpDescriptionList{}
        }
    }
}
