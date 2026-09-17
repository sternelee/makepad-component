//! `MpSheet` — a panel pinned to an edge, which travels in rather than growing in place.
//!
//! ## What the page shows
//!
//! A sheet pinned to the window's left edge and one pinned to the bottom, both open. The two are the argument for the whole
//! component: **the extent is read *across* the edge it is pinned to**, so on the left it is a width and on the bottom it is a
//! height — which is why the parameter is not called a width.
//!
//! ## And the two things a picture cannot show
//!
//! - **Only the free corners are rounded.** A panel attached to an edge is part of the frame there: rounding its pinned
//!   corners shows the page behind through notches that were never notches. makepad's `border_radius` rounds all four, so the
//!   plate is drawn **past the pinned edge by its own radius** and the clip removes the corners that must stay square.
//! - **It travels rather than grows**, which is why `makepad_motion` gained a `sheet_in` entry instead of this borrowing the
//!   dialog's: a travel read at a grow's duration arrives before the eye has followed it.
//!
//! The printed lines are the geometry at each stage of the travel, the corner rule, and the dismissal test — all of which a
//! screenshot would show only as a panel sitting on an edge.

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
    let Panel = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 6
        padding: 16
    }

    mod.gallery.pages.sheets = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Sheets"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A panel pinned to an edge, travelling in from it. The argument it takes is an extent rather than a width, because it is read across the edge it is pinned to: on the left it is a width and on the bottom it is a height, so a parameter called width would be wrong for one of the three sides in a way a caller only discovers by measuring. Its motion is its own catalog entry and not the dialog's, since a sheet travels while a dialog grows, and a travel read at a grow duration arrives before the eye has followed it. Only the corners away from the pinned edge round, because a panel attached to an edge is part of the frame there — and since makepad rounds all four corners, the plate is drawn past the pinned edge so the ones that must stay square fall outside the clip."
        }

        Section{
            Caption{ text: "Pinned left: 320 across, spanning the height" }
            sheet_left := mod.mp.MpSheet{
                side: 0.0
                extent: 320.0
                content := Panel{
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: headline, color: text}
                        text: "Connections"
                    }
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: body, color: text_muted}
                        text: "The panel travels in from the edge it is pinned to."
                    }
                }
            }
        }
        Section{
            Caption{ text: "Pinned bottom: 240 across, spanning the width — the same argument, read as a height" }
            sheet_bottom := mod.mp.MpSheet{
                side: 2.0
                extent: 240.0
                content := Panel{
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: headline, color: text}
                        text: "Row actions"
                    }
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: body, color: text_muted}
                        text: "A bottom sheet is what a narrow window wants instead of a side panel."
                    }
                }
            }
        }
        Section{
            Caption{ text: "Closed: off its own edge, and taking no events" }
            sheet_closed := mod.mp.MpSheet{
                side: 1.0
                extent: 280.0
                content := Panel{}
            }
        }
    }
}
