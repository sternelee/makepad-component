//! The overlay: a tooltip that has to paint above everything below it.
//!
//! **Hovering a button on this page shows nothing, and that is a known gap.**
//! The page detects the hover in the *app*, which is the wrong place: Makepad
//! resolves one hit per event and the button consumes it, so a second
//! `event.hits` on the same area from the app reports nothing. Hovering belongs
//! in the trigger, which is why the component that should exist is an
//! `MpTooltipArea` container rather than a page-level lookup. See
//! `mp/tooltip.rs`.
//!
//! What the page *does* verify, with `GALLERY_TOOLTIP=1`, is the mechanism: the
//! plate is drawn over the card and the buttons below it — siblings that follow
//! it in the tree — anchored from a trigger's `Area`.
//!
//! The arrangement matters and is the page's other lesson: the tooltip is a
//! `Fill`/`Fill` overlay *sibling* of the content, not a child of it. Inside the
//! content's column it would compete for height; and a zero-sized tooltip — the
//! first attempt — has nothing to composite at all, so its overlay pass is empty
//! and the plate never appears.
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

    // The page is an `Overlay` flow: the content sits in a normal column, and
    // the tooltip is its *sibling*, sized to the whole viewport. That is the
    // arrangement a full-size overlay needs — put it inside the column and it
    // competes with the content for height, which is what a tooltip must not do.
    mod.gallery.pages.overlay = View{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        overlay_body := View{
            width: Fill
            height: Fill
            flow: Down
            spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Overlay"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Hover a button. The tooltip is a Fill/Fill overlay sibling of this page\'s content, so it draws over the card below it — the card is opaque, which is what makes that checkable rather than assumed. It draws there because it owns a DrawList2d bracketed with begin_overlay_reuse, not because of where it sits in the tree; the v2 set instead managed an overlay pass by hand and resolved z-order with geometry hit-tests, which produced the two hardest bugs in docs/WIDGETS_PROGRESS_CN.md: a dropdown painted and then covered by a following section, and a sheet whose close button never received its hit."
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Hover a button — the plate hangs under the trigger, in the inverted plate the palette keeps for exactly this. It draws over the card below, which is opaque, so the overlay pass is real rather than assumed" }
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
            Caption{ text: "Anchored from a widget's Area — the call site says which widget, not which rectangle" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                tip_one := mod.mp.MpButton{ text: "Anchored here" }
                mod.mp.MpButton{ text: "Not this one" }
            }
        }
        }

        // Second in the tree, and it has to draw last: the draw list is what
        // puts it over `overlay_body` regardless of tree order.
        tip := mod.mp.MpTooltip{}
    }
}
