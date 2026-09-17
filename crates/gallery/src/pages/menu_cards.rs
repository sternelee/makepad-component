//! `MpMenuCard` — the panel a menu drops.
//!
//! ## What the page shows that a picture cannot
//!
//! Two of the card's decisions are invisible in a screenshot and are the reason the geometry is a tested function:
//!
//! - **The glyph gutter is reserved only when a row uses it.** The first card has no icons and its labels sit against the
//!   padding; the second has one icon and every label is indented to the gutter. A panel that reserved the column always
//!   would open with an empty stripe down its left — which is what a menu bar's menus look like when it does.
//! - **A described row is two lines tall and widens the panel to a ceiling, not a floor.** The third card's panel is wider
//!   than the others by a fixed amount rather than by the length of its sentence, so a menu does not jump when the data
//!   changes.
//!
//! ## And what it is honest about
//!
//! **The card draws and hit-tests; it does not open.** The cursor is the caller's — see `mp/menu.rs` — so this page sets one
//! directly rather than pretending a click is synthesizable here (it is not; the synthetic pointer does not reach these
//! widgets). The printed lines are the evidence: the panel's width and height, the gutter decision, and where a `y` lands.

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

    mod.gallery.pages.menu_cards = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Menu Cards"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "The panel a menu drops, with one cursor for the pointer and the keyboard — a menu that tracked hover separately could light two rows at once and hang a submenu off a row that is not live. The glyph gutter is reserved only when a row uses it, so a menu of plain labels keeps its labels against the padding instead of opening with an empty stripe down the left. A described row is two lines tall and widens the panel to a ceiling rather than a floor, because a description is a sentence and a panel that grew to the longest one would jump whenever the data changed. The card draws and hit-tests and reports; opening is the caller's, because only the caller knows what opening means."
        }

        Section{
            Caption{ text: "The strip of titles, with File down — the bar's own rules drive which title is lit" }
            menu_strip := mod.mp.MpMenubarStrip{width: Fit}
        }
        Section{
            Caption{ text: "No icons: the gutter is not reserved, and one row is checked" }
            menu_plain := mod.mp.MpMenuCard{}
        }
        Section{
            Caption{ text: "One icon opens the gutter for every row in the panel" }
            menu_glyphs := mod.mp.MpMenuCard{}
        }
        Section{
            Caption{ text: "A described row — two lines tall, and the panel takes its described width" }
            menu_described := mod.mp.MpMenuCard{}
        }
    }
}
