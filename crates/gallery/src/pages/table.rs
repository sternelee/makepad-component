//! The table: columns, rows, and the arithmetic that puts a cell where it belongs.
//!
//! The page is the check for the two things a test cannot see: that a table fills
//! the width it is given (flexible columns sharing the remainder exactly, with no
//! gap at the right edge), and that a cell clips its text rather than overrunning
//! its neighbour.

use makepad_component::mp::table::TableColumn;
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

    mod.gallery.pages.table = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Table"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A table cannot be a widget per cell — a hundred rows of six columns would be six hundred widgets with six hundred areas for content that is a grid of strings — so the table owns the layout and paints its cells. One function answers where every row and column is, and the painter, the hover and the click all read it: writing that arithmetic once per consumer is how a row ends up clickable where it is not drawn."
        }

        Section{
            Caption{ text: "A flexible first column and three fixed ones. `priority` and `count` are right-aligned, because a column of figures set flush left is the most common table mistake there is" }
            build_table := mod.mp.MpTable{}
        }

        Section{
            Caption{ text: "An empty table is its header, not nothing — a table with no rows still has columns to name" }
            empty_table := mod.mp.MpTable{}
        }
    }
}
