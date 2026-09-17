//! JSON Canvas, parsed, validated and painted.
//!
//! [JSON Canvas](https://jsoncanvas.org) is an open format for infinite canvases — **a published spec with a
//! version and a date** (1.0, 2024-03-11), which is why this is a model rather than a bespoke one. The workspace's
//! `canvas-terminal` already has a 7000-line interactive canvas with its **own** save format; the useful
//! contribution is not a second canvas but **interchange**.
//!
//! ## What the page shows
//!
//! - The document, painted read-only: nodes at their positions, edges as elbows, groups behind their contents, and
//!   the six preset colours from the theme.
//! - The **fixed point**, on the document being displayed: parse, serialize, parse again and compare — the same
//!   guarantee `makepad-markdown` makes, for the same reason.
//! - The **validation report**, on a deliberately broken document as well as the good one, because a report that
//!   only ever prints nothing is indistinguishable from a report that does not work.
//! - The wire form, so the round trip is visible as text as well as asserted.
//!
//! ## The three spec details this page exists to check by eye
//!
//! 1. **`fromEnd` defaults to `none` and `toEnd` to `arrow`** — the two ends of an edge do not share a default, so
//!    an arrow at a connection's *start* is unusual and the format says so. The arrowheads below are only at the
//!    ends.
//! 2. **The presets are red, orange, yellow, green, cyan, purple** — `"1"` through `"6"`, not the rainbow order you
//!    would guess. Four nodes below use presets and two use hex, and they must be told apart.
//! 3. **Nodes are in ascending z-order**, so the array's order is data rather than presentation.

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

    mod.gallery.pages.canvas = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Canvas"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A JSON Canvas document: nodes at positions, edges between them, and the format's six preset colours. Read-only on purpose — canvas-terminal in this workspace already has an interactive infinite canvas with a camera, a hit test and its own persistence, so a second one here would be a second copy of all of it. What this is for is interchange: a canvas document another tool can read. The model has no dependencies and its guarantee is a fixed point, checked below on the document being displayed."
        }

        Section{
            Caption{ text: "The document. A group behind four nodes, four presets and two hex colours, and three edges — the arrowheads are only at the ends, because the spec's fromEnd defaults to none while its toEnd defaults to arrow" }
            canvas_view := mod.mp.MpCanvas{}
        }

        Section{
            Caption{ text: "What the model made of it" }
            canvas_stats := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(stats)"
            }
            canvas_fixed_point := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(fixed point)"
            }
            canvas_problems := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(problems)"
            }
            canvas_broken := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(a broken document)"
            }
        }

        Section{
            Caption{ text: "The wire form — the file, which is the whole point of an interchange format" }
            canvas_wire := mod.mp.MpCodeBlock{}
        }
    }
}
