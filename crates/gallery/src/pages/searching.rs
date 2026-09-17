//! `MpSearchableList` — a list you narrow by typing.
//!
//! ## What the page is for
//!
//! Three things that are only visible while typing, which is why there is a page rather than only tests:
//!
//! - **The count and the rows are different numbers.** The second list has more items than the ten slots, so the line
//!   under it says how many it is hiding — a truncated list that says nothing reads as a complete one.
//! - **A selection survives a filter that no longer shows it.** Type into the third list after clicking a row: the
//!   selected item keeps its **index in the items**, not in the narrowed view, so narrowing cannot move the selection to
//!   a different item. That is the defect the widget this replaces had, and it is invisible until you type.
//! - **The placeholder is a placeholder, not a filter** — the field is empty, and the whole list is shown, which is what
//!   the A2UI renderer got wrong.

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

    mod.gallery.pages.searching = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Searching"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A list narrowed by typing. The selection is an index into the items rather than into the narrowed view, so filtering cannot move it to a different item — which is the defect this widget was written around: its predecessor selected a position in the filtered array, so clicking the third row and then typing made the third row of the remainder the selection, silently. The count of matches and the rows shown are two different numbers, so a list showing ten of twenty-three can say so; a truncated list that says nothing reads as a complete one. Ten slots, filled from the front."
        }

        Section{
            Caption{ text: "Six items — everything fits, so nothing is hidden and there is no count line" }
            searching_six := mod.mp.MpSearchableList{}
        }
        Section{
            Caption{ text: "After clicking a row, type into the field: the selection keeps its item rather than its position" }
            searching_selected := mod.mp.MpSearchableList{}
        }
        Section{
            Caption{ text: "Eighteen items offered to ten slots — narrow it to see the count and the rows diverge" }
            searching_many := mod.mp.MpSearchableList{}
        }
    }
}
