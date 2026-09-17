//! The syntax stack, end to end.
//!
//! Three pieces that were built to be separable, shown working together:
//!
//! - **`makepad-syntax`** classifies a JSON document into spans.
//! - **`makepad-theme::syntax`** supplies the kind vocabulary and the palette.
//! - **`MpCodeBlock`** paints the runs — one `DrawText` call per run, because `DrawText` paints one
//!   string in one colour and a code block is read by lines.
//!
//! The separability is the reason the page is worth looking at rather than a claim in a doc: the
//! widget takes spans and does not classify, the classifier does not know a colour, and the palette
//! does not know a source language. Each of the three would still work if the other two were
//! replaced, which is what makes the classifier — the one part bezel implements with tree-sitter and
//! this port does not — a decision that can be revisited.
//!
//! ## What is painted here
//!
//! The document is the **A2UI message shape** this workspace's demo actually exchanges, so the
//! highlighter has a reader rather than a fixture. The second block is a legend drawn **by the same
//! widget**: one word per kind, each word spanned with that kind — so the legend cannot fall out of
//! step with the palette the way a hand-coloured list would.
//!
//! The traps the widget is built around are visible here too: the comment's `//` runs across the
//! block, the string contains an escaped quote, and one key is deliberately misspelled so the
//! `Invalid` kind is on screen rather than only in a test.

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

    mod.gallery.pages.code = View{
        width: Fill
        height: Fill
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Code"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A JSON document classified by makepad-syntax, coloured by the theme's syntax palette, and painted by MpCodeBlock — three pieces built to be separable, shown together. The widget takes spans and does not classify; the classifier does not know a colour; the palette does not know a source language. That is what makes the classifier the one part that can be replaced: bezel implements it with tree-sitter and this port does not, and swapping it changes neither of the other two."
        }

        Section{
            Caption{ text: "An A2UI message — the shape this workspace's demo exchanges, so the highlighter has a reader rather than a fixture. Note the comment markers this JSON does not have: the document is mutated below to put a multi-line token, an escaped quote and a misspelling on screen, because those are the three cases the widget is built around" }
            code_json := mod.mp.MpCodeBlock{}
        }

        Section{
            Caption{ text: "The legend, drawn by the same widget: one word per kind, each spanned with that kind. A hand-coloured legend would fall out of step with the palette; this one cannot" }
            code_legend := mod.mp.MpCodeBlock{}
        }

        Section{
            Caption{ text: "What the classifier returned" }
            code_stats := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(stats)"
            }
            code_traps := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(traps)"
            }
        }
    }
}
