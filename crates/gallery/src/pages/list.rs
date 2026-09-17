//! The list: rows of things, and the three shapes a row takes.
//!
//! The page puts a glyph row, a glyphless row and a detail row next to each other,
//! because the property worth checking is that all three share **one left edge** —
//! the glyph's room is reserved for every row and drawn only by the rows that have
//! one, and a list with two left edges reads as a rendering fault.

use makepad_component::mp::list::ListItem;
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

    let Row = View{
        width: Fill, height: Fit, flow: Right, spacing: 16, align: Align{y: 0.0}
    }

    let Half = View{
        width: Fill, height: Fit, flow: Down, spacing: 6
    }

    mod.gallery.pages.list = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "List"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "The third of the data widgets and the simplest: where a table has columns and a tree has depth, a list has neither — it is the shape a sidebar, a command palette and a picker's options all reduce to. A row is a glyph, a label and a detail pushed to the far edge, which is the shape of every command-palette row and every picker option with a count. One widget paints its rows from data, and the measure-and-clip arithmetic is shared with the table and the tree, so all three clip the same string at the same point."
        }

        Section{
            Caption{ text: "A sidebar — glyphs, and one row with a detail. Every label shares one left edge whether or not its row has a glyph" }
            Row{
                Half{
                    sidebar_list := mod.mp.MpList{}
                }
                Half{
                    Caption{ text: "A picker's options — no glyphs, and the labels keep the same origin, so the two lists would line up if they were stacked" }
                    options_list := mod.mp.MpList{}
                }
            }
        }

        Section{
            Caption{ text: "A command palette — a glyph, a name, and a shortcut pushed to the far edge. The detail claims at most two fifths of the row, so a verbose shortcut cannot leave one character of a name" }
            palette_list := mod.mp.MpList{}
        }

        Section{
            Caption{ text: "An empty list has no height — a widget painted entirely by its own shader states its own size, which is the rule this crate has now hit three times" }
            empty_list := mod.mp.MpList{}
        }
    }
}
