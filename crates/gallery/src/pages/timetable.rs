//! `MpCalendar` — a configurable banded grid. A timetable, despite the name.
//!
//! ## What the page shows, and the rule it shows it with
//!
//! Three grids with **three different colour hints between them**, because the rule in this component is that a row's height
//! and its meaning come from a *string* the caller supplies:
//!
//! - a `header` row is 55pt and shows the **column headers** rather than data — so the same grid reads as dates in one band
//!   and as sessions in the next;
//! - a `budget` row is 40pt, the compact one;
//! - anything else is 70pt, the default.
//!
//! A page where every hint were the same would set one row height three times and prove nothing.
//!
//! ## The name is misleading and is said out loud
//!
//! This has nothing to do with a month calendar. It is the shape the A2UI protocol calls `Calendar`: a column header row and
//! one band per row, each band divided into equal columns. `mp/date.rs` is where month grids live.
//!
//! ## And what a picture cannot show
//!
//! That a click in the title band is **not** a cell, that a click past the last column is not one either, and that the cell
//! drawn and the cell a click reports come from one arithmetic. The printed lines are the hit tests.

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

    mod.gallery.pages.timetable = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Timetable"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A configurable banded grid — the shape the A2UI protocol calls Calendar, and not a month view. Columns are equal shares with no gutter, so a cell is found by counting bands and dividing rather than by searching hit areas, which is what makes the click test exact. A row's height and its meaning come from a colour hint string: header is 55 points and shows the column names rather than data, budget is 40, and anything else is 70. A config with no columns or no rows draws nothing at all rather than dividing by zero columns, and a cell matrix shorter than the headers promise is grown with empty cells rather than indexed past its end."
        }

        Section{
            Caption{ text: "A timetable: a header band, a compact row, and a full row" }
            grid_a := mod.mp.MpCalendar{grid_width: 640}
        }
        Section{
            Caption{ text: "No title and no footer: the grid is only as tall as its bands" }
            grid_b := mod.mp.MpCalendar{grid_width: 420}
        }
    }
}
