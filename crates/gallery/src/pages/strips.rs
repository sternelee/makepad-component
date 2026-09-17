//! `MpStrip` — a tonal notice: a message on a wash, with its tone's glyph.
//!
//! ## What the page shows
//!
//! The four tones side by side, because the component **is** its tone vocabulary: bezel ships two strips that differ only in
//! colour and glyph, so this is one component with a `StripTone` rather than four near-identical widgets. Each tone's plate
//! is its colour at the same two fractions — 6% for the wash and 20% for the border — so a new tone cannot arrive with
//! hand-picked values that do not match its neighbours.
//!
//! ## And what the print shows
//!
//! The mapping from the closed vocabulary to the environment: each tone's glyph, its theme colour, whether it
//! **demands action** (a property of the tone rather than of the occasion, so a caller deciding whether a strip may be
//! skipped asks the tone), and the height for a one-line and a two-line message.
//!
//! ## Deliberately absent
//!
//! A dismiss button. bezel's strips have none, and a strip that can be dismissed is a different component: it has state,
//! it needs a caller to notice, and once one strip has a close button every strip wants one. If the library grows that it
//! should be a named decision rather than a field.

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

    mod.gallery.pages.strips = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Strips"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A tonal notice: a message on a wash of its tone, with the tone's glyph beside it. bezel ships two of these and they differ only in colour and symbol, so this is one component with a closed tone rather than four widgets — the library's rule that a caller chooses a name and never a colour. Each plate is its tone at the same two fractions, six percent for the wash and twenty for the border, so a tone cannot arrive with hand-picked values that do not match its neighbours. The height follows the message's line count, which the caller has already computed: measuring the wrap again here would be a second answer to a question the layout just answered. There is deliberately no dismiss button."
        }

        Section{
            Caption{ text: "Error and warning — the two tones that ask the reader to do something" }
            strip_error := mod.mp.MpStrip{}
            strip_warning := mod.mp.MpStrip{}
        }
        Section{
            Caption{ text: "Info and success — the two that report" }
            strip_info := mod.mp.MpStrip{}
            strip_success := mod.mp.MpStrip{}
        }
        Section{
            Caption{ text: "A two-line message: the plate grows by exactly one line" }
            strip_two_lines := mod.mp.MpStrip{}
        }
    }
}
