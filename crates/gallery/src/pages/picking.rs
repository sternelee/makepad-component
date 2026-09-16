//! `MpColorPicker` — a grid of swatches you pick from.
//!
//! ## What the page is for
//!
//! The hit test, which is the only interesting logic here and is invisible in a picture: **a point in the gap between two
//! swatches picks nothing**, and neither does the space past the last column of a partial row. Clicking in the gaps of the
//! third grid below is the test — a picker that rounded a click to the nearest swatch would pick a colour somebody
//! narrowly missed.
//!
//! ## And one thing a picture settles that arithmetic cannot
//!
//! Whether the selection's inner ring is distinguishable from a hovered neighbour's brightened hairline. Both are drawn
//! from the theme's own ink and the ring is inside the cell, so a hover and a selection on adjacent swatches should read
//! as two different things — which is the sort of claim that is either true when you look or not true at all.
//!
//! ## The colours are the caller's, which is not a theme violation
//!
//! The three palettes below are written out rather than taken from the theme, because **a palette is data**: the list of
//! things a user may choose between. A picker whose swatches came from the theme would offer the theme's own colours, which
//! is a different component. How a swatch is *drawn* — its ring, its radius, its border — is the theme's.

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

    mod.gallery.pages.picking = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Picking"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A grid of swatches. The colours are the caller's because a palette is data — the list of things a user may choose between — while how a swatch is drawn, its ring and radius and border, comes from the theme. One shader instance is reused for the whole grid, so a twenty-swatch picker is twenty draws of one shader rather than twenty widgets, and that is only possible because the cells share everything except their colour and their two flags. Which is also why the hit test is arithmetic and not a search: the gaps between swatches are not swatches, and neither is the space past the last column of a partial row — a click that narrowly misses picks nothing rather than the nearest colour."
        }

        Section{
            Caption{ text: "Nine swatches in rows of eight — one partial row, and clicking in the gaps picks nothing" }
            picking_one_col := mod.mp.MpColorPicker{}
        }
        Section{
            Caption{ text: "The same nine in rows of four: the wrap is the caller's, and the grid is as wide as the columns" }
            picking_four := mod.mp.MpColorPicker{}
        }
        Section{
            Caption{ text: "Rows of one — a list rather than a grid, which a caller may ask for and gets" }
            picking_column := mod.mp.MpColorPicker{}
        }
    }
}
