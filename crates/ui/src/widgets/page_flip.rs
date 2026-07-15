use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpPageFlip - Page switching container
    // ============================================================

    // Base PageFlip - simple page container
    mod.widgets.MpPageFlip = mod.widgets.PageFlip{
        width: Fill
        height: Fill
    }

    // PageFlip with background
    // NOTE: Makepad 2.0's PageFlip no longer has show_bg/draw_bg fields,
    // so it cannot paint a background itself anymore. Give the pages
    // inside it a backgrounded style (e.g. MpPageWithBg) instead.
    mod.widgets.MpPageFlipWithBg = mod.widgets.PageFlip{
        width: Fill
        height: Fill
    }

    // ============================================================
    // Page container styles - use these inside PageFlip
    // ============================================================

    // Basic page container
    mod.widgets.MpPage = mod.widgets.View{
        width: Fill
        height: Fill
        flow: Down
        padding: 16
    }

    // Page with centered content
    mod.widgets.MpPageCentered = mod.widgets.View{
        width: Fill
        height: Fill
        flow: Down
        align: Align{x: 0.5, y: 0.5}
        padding: 16
    }

    // Page with background
    // (SolidView: its draw_bg shader actually paints `color`,
    // a plain View's stock shader ignores it)
    mod.widgets.MpPageWithBg = mod.widgets.SolidView{
        width: Fill
        height: Fill
        flow: Down
        padding: 16
        draw_bg +: {
            color: CARD
        }
    }

    // Page with rounded card style
    mod.widgets.MpPageCard = mod.widgets.RoundedView{
        width: Fill
        height: Fill
        flow: Down
        padding: 24
        margin: 16

        draw_bg +: {
            color: CARD
            border_radius: 8.0
            border_size: 1.0
            border_color: BORDER
        }
    }

    // ============================================================
    // Pre-styled color pages for quick demos
    // ============================================================

    // Primary colored page
    mod.widgets.MpPagePrimary = mod.widgets.SolidView{
        width: Fill
        height: Fill
        flow: Down
        align: Align{x: 0.5, y: 0.5}
        padding: 16
        draw_bg +: {
            color: PRIMARY
        }
    }

    // Secondary colored page
    mod.widgets.MpPageSecondary = mod.widgets.SolidView{
        width: Fill
        height: Fill
        flow: Down
        align: Align{x: 0.5, y: 0.5}
        padding: 16
        draw_bg +: {
            color: SECONDARY
        }
    }

    // Muted colored page
    mod.widgets.MpPageMuted = mod.widgets.SolidView{
        width: Fill
        height: Fill
        flow: Down
        align: Align{x: 0.5, y: 0.5}
        padding: 16
        draw_bg +: {
            color: MUTED
        }
    }

    // Accent colored page
    mod.widgets.MpPageAccent = mod.widgets.SolidView{
        width: Fill
        height: Fill
        flow: Down
        align: Align{x: 0.5, y: 0.5}
        padding: 16
        draw_bg +: {
            color: ACCENT
        }
    }
}

// Re-export PageFlip types from makepad_widgets for convenience
pub use makepad_widgets::page_flip::PageFlipRef;
