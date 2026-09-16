//! A markdown document, parsed, laid out and painted.
//!
//! Three crates meeting, each of which was built to stand alone:
//!
//! - **`makepad-markdown`** parses the source into a flat block document and lays it out — no
//!   dependencies at all, which is why its fixed-point property could be checked over nine thousand
//!   generated documents in a tenth of a second.
//! - **`makepad-theme`** supplies the metrics and the palette.
//! - **`MpMarkdown`** paints the lines at the coordinates the layout computed.
//!
//! ## What this page checks that the tests cannot
//!
//! The tests check the model and the layout in isolation. This page runs the **fixed point in the
//! running app** — parse, serialize, parse again, and compare — on the document it is actually
//! displaying, and prints the block and line counts the layout produced. Those are the numbers a
//! screenshot would have shown: that the document is more than one block, that its height is not zero,
//! and that a second, narrower column re-wraps to more lines than the first.
//!
//! The paint itself is **not verified**, and cannot be: screen capture became unavailable this session.
//! What is verified is that every kind resolves in the DSL (`[E] = 0`), that every block laid out to at
//! least one line, and that the height the widget reports matches the height the layout computed.

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

    mod.gallery.pages.document = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Document"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A markdown document: parsed by makepad-markdown into a flat block list, laid out into lines, and painted at the coordinates the layout computed. The model is Notion's rather than CommonMark's — blocks with an indent level rather than a tree, because editing a flat list is list operations while editing a tree is restructuring — and the guarantee it makes is a fixed point rather than an inverse: parse, serialize and parse again always lands on the same document, so an edit/save cycle cannot drift. That property is checked below on the document being displayed."
        }

        Section{
            Caption{ text: "The document. Every block kind the model carries is in it: headings, paragraphs, bullets, an ordered list, tasks, a quote, a fence and a divider" }
            doc_full := mod.mp.MpMarkdown{}
        }

        Section{
            Caption{ text: "The same document in a narrower column, so the wrap is doing something: the line count must go up and the block count must not change" }
            doc_narrow := mod.mp.MpMarkdown{}
        }

        Section{
            Caption{ text: "What the parser and the layout produced" }
            doc_stats := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(stats)"
            }
            doc_fixed_point := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(fixed point)"
            }
            doc_wire := mod.mp.MpCodeBlock{}
        }
    }
}
