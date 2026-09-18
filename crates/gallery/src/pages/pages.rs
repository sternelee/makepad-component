//! `MpPage` — the page's measure, its header row, and its subtitle.
//!
//! ## The three pieces, and why there are three
//!
//! bezel returns three builders rather than one page component, so a page with no subtitle has no gap where one would have
//! been. This keeps that: [`MpPage`] is the centred column with its measure, [`MpPageHeader`] is a title and the count *of*
//! that title sharing a baseline, and the subtitle is a label a caller drops underneath. One component that guessed at the
//! shape of every page would have to reserve space for parts a page does not have.
//!
//! ## What the page shows
//!
//! A header with a count, a subtitle under it, and the measure bounding both — 768 points, centred, with the window's own
//! padding on a narrow window instead. The count sits on the title's baseline rather than at its top, which is the difference
//! between a count *of* the title and a second thought beside it.
//!
//! ## And what the print shows
//!
//! That the measure and the padding divide the widest of windows the same way; that the two insets plus the content are
//! exactly the viewport; that a count of `None` and a count of zero are different statements; and the baseline offset that
//! makepad has no alignment for.

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

    mod.gallery.pages.pages = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Pages"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "The page measure, the header row and the subtitle — three small things rather than one page component, so a page with no subtitle has no gap where one would have been. The measure is the point of the column: 768 points, centred, because a line of prose set the full width of a wide window is hard to track from its end back to its start, and every serious reading surface picks a measure for that reason. On a narrower window the padding is what bounds the content instead, and neither ever pushes the content off its own edge. The header's count shares the title's baseline rather than its top, because a count on its own line reads as a subtitle while one on the title's baseline reads as a count of that title."
        }

        Section{
            Caption{ text: "A page: the measure bounds the column, and the count sits on the title's baseline" }
            demo_header := mod.mp.MpPageHeader{}
            demo_subtitle := mod.mp.MpPageSubtitle{
                text: "Everything in this workspace that has a name."
            }
        }
        Section{
            Caption{ text: "A header with no count, and one counting zero — two different statements" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 32, align: Align{y: 0.0}
                demo_none := mod.mp.MpPageHeader{}
                demo_zero := mod.mp.MpPageHeader{}
            }
        }
    }
}
