//! The pagination: the window onto a long list, at the shapes it has to hold.
//!
//! The arithmetic is the component's content, so it is exhaustively tested, and
//! the four cases on this page are the ones a still frame can check: the row must
//! not change width as the current page moves, and the ends must always be present.
//!
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

    mod.gallery.pages.pagination = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Pagination"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "The whole component is one function: given the current page, the page count and how many neighbours to show, produce the row of numbers and ellipses. Everything else is the shape that output is drawn in. So the arithmetic is pure and tested exhaustively, because page arithmetic is where pagination is actually wrong — a duplicated page, a missing last page, an ellipsis where a number fits — and every one of those is a set property rather than a drawing property. The first and last pages are always shown; the window is centred where it can be and pushed against an end where it cannot, so the row never changes width as you walk through it; and an ellipsis only ever stands in for a run of at least two hidden pages, because one hidden page drawn as an ellipsis reads as a mistake."
        }

        Section{
            Caption{ text: "A list that fits — every page, no ellipsis. An ellipsis is never worth a number's place when there is room for the number" }
            pag_short := mod.mp.MpPagination{}
        }

        Section{
            Caption{ text: "A long list at the first page — the window is pushed right, so the left run is real numbers rather than an ellipsis standing in for one page" }
            pag_first := mod.mp.MpPagination{}
        }

        Section{
            Caption{ text: "The same list in the middle — the window is centred, and the row is the same width as the one above" }
            pag_middle := mod.mp.MpPagination{}
        }

        Section{
            Caption{ text: "And at the last page — pushed left, the same width again" }
            pag_last := mod.mp.MpPagination{}
        }

        Section{
            Caption{ text: "Two neighbours either side — the wider window, which shifts where the ellipses fall" }
            pag_wide := mod.mp.MpPagination{}
        }
    }
}
