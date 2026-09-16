//! The overlay: a tooltip that has to paint above everything below it.
//!
//! **This page currently shows nothing.** The tooltip is implemented and the app
//! runs clean, but the plate never draws — see `mp/tooltip.rs` for the state and
//! the three places to look next. The page is kept in the rail rather than hidden
//! so the gallery reports the truth about what works.
//!
//! The page is built so the overlay either works or is obviously broken. The
//! tooltip is declared *first* in the tree — before the sections, the cards and
//! the buttons it has to cover — so a version that relied on tree order would
//! render underneath them. It is the `DrawList2d` that puts it on top, and that
//! is the only thing being demonstrated.

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

    mod.gallery.pages.overlay = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        // First in the tree, and it has to draw last.
        tip := mod.mp.MpTooltip{}

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Overlay"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "NOT WORKING YET. The tooltip is the first thing in this page's tree, before the cards and buttons it has to cover, and it would draw on top of them by owning a DrawList2d bracketed with begin_overlay_reuse — the framework composites that pass after the normal tree. The mechanism is right; the plumbing is not, and the plate never appears. Hovering a button below does nothing. See mp/tooltip.rs for the three places to look next, and docs/WIDGETS_PROGRESS_CN.md for the two bugs the v2 set's geometry hit-testing produced."
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Hover a button — the plate hangs below and right of it, in the inverted plate the palette keeps for exactly this" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                tip_primary := mod.mp.MpButton{ style: mod.mp.ButtonStyle.Prominent, text: "Continue" }
                tip_default := mod.mp.MpButton{ text: "Save" }
                tip_ghost := mod.mp.MpButton{ style: mod.mp.ButtonStyle.Ghost, text: "Cancel" }
                tip_danger := mod.mp.MpButton{ style: mod.mp.ButtonStyle.Destructive, text: "Delete" }
            }
        }

        // A card *after* the tooltip, and overlapping where the plate lands, so
        // the overlay is drawn over something opaque rather than over the page.
        mod.mp.SurfaceCard{
            tip_card_label := Label{
                width: Fit, height: Fit
                draw_text +: {text_style: body, color: text}
                text: "A card below the tooltip. Its edge and its fill are both opaque, so if the plate is visible over this, the overlay pass is real."
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "On the rail — the tooltip is anchored from an Area, not from a coordinate, so a call site does not do arithmetic on rectangles" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                tip_one := mod.mp.MpButton{ text: "Anchored here" }
                mod.mp.MpButton{ text: "Not this one" }
            }
        }
    }
}
