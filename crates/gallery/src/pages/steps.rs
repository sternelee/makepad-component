//! `MpStepIndicator` — a numbered path with the current step marked.
//!
//! ## What the page is for
//!
//! The state model is three cases and the page shows all three in one row: a step **behind** the current one, the
//! current one, and the ones **ahead**. Below that, the same component at a different current step, because the rule
//! worth seeing is the connector's — **the line into a step is filled once that step is reached**, so a connector
//! carries the state of the step to its *right* and is not simply "the step before it is passed".
//!
//! ## And two cases the tests cannot settle
//!
//! Whether the number is centred in its circle (a two-digit step would sit left of centre if the centring were guessed
//! rather than measured), and whether a step's title lines up with its circle when the row is a `Fill` width.

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

    mod.gallery.pages.steps = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Steps"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A numbered path with the current step marked. A step is passed, current or upcoming, and that is a function of its index and the current step rather than three stored flags — so exactly one step is current by construction, not by remembering to maintain it. The connector follows a different rule and has its own function: the line into a step is filled once that step is reached, so a connector carries the state of the step to its right, and step 0 has none because there is nothing before it. Eight slots, filled from the front, and the dots read their colours from the theme on every paint — so an appearance change is a redraw rather than a re-declaration."
        }

        Section{
            Caption{ text: "Five steps, third current — passed, current and upcoming all in one row" }
            steps_third := mod.mp.MpStepIndicator{}
        }
        Section{
            Caption{ text: "The same path at step one: everything ahead of it, and only the first connector filled" }
            steps_first := mod.mp.MpStepIndicator{}
        }
        Section{
            Caption{ text: "Ten steps offered to eight slots — the path truncates rather than growing" }
            steps_over := mod.mp.MpStepIndicator{}
        }
    }
}
