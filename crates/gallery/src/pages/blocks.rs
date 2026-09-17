//! A fenced block: a ```chart fence routed to a plot.
//!
//! ## Why a fence rather than a new kind of block
//!
//! A fence already carries a **language tag** and a body, and `makepad-markdown` already parses it into a block whose
//! text is the body. So a block of an app's own — a chart, a diagram, an embed — needs a **renderer over a fence tag**
//! rather than a new `BlockKind`: the wire form already round-trips byte for byte, and a document written with a fence
//! still opens in a tool that has never heard of the tag.
//!
//! ## The router's answer is data, not a widget
//!
//! The reference returns an `Element` — a gpui view — from its renderer. This port cannot: a Makepad widget is
//! declared in the DSL and registered on the script heap, so it is not a value a function can return. What a renderer
//! returns here is what the **paint** needs: [`Series`](makepad_plot::Series), which this page hands to a `LinePlot` it
//! declared. The same split as `mp/code.rs` and `makepad-syntax` uses, and the reason the parsing can be tested
//! without a window while the widget stays a widget.
//!
//! ## What the page shows
//!
//! The fence's own text, the chart it parsed into (labels and series, printed), the plot it drove, and — the rule that
//! makes a `chart` fence safe to enable everywhere — a second fence that is **prose**, which the block **declines**,
//! so it stays code. A block that claimed every fence would turn a document's shell session into an empty chart.

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

    mod.gallery.pages.blocks = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Blocks"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A fence tag routed to a renderer: a markdown fence whose body is a small table, parsed into series a chart draws. A fence already round-trips byte for byte and already degrades to its own source where nothing paints it, so a block of an app's own is a renderer over a tag rather than a new kind of block. The router answers with data rather than a widget, because a Makepad widget is declared in the DSL and is not a value a function can return — and that split is what lets the parsing be tested without a window."
        }

        Section{
            Caption{ text: "The fence a person would write — key/value headers, a series per group, x, y rows" }
            blocks_fence := mod.mp.MpCodeBlock{}
        }

        Section{
            Caption{ text: "The chart it parsed into, drawn by makepad-plot. The fence drove this: the block returned series, the page handed them to the plot" }
            View{
                width: Fill
                height: 300
                plot := mod.widgets.LinePlot{}
            }
        }

        Section{
            Caption{ text: "What the router answered" }
            blocks_stats := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(stats)"
            }
            blocks_declined := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(declined)"
            }
        }
    }
}
