//! `MpTabBar` — a strip of tabs, one of them current, with a close and an add.
//!
//! ## The page is an allocation
//!
//! A tab bar's whole job is fitting `n` titles into a width, so the three bars below are the three regimes of that
//! arithmetic, and the printed lines are the arithmetic:
//!
//! - **room**: four tabs share 800 points equally at 200, which is the ceiling — tabs stop growing there;
//! - **a share**: eight tabs at 100 each would be narrower than a tab may be, so **every tab takes the 120-point minimum and
//!   the strip overflows**. That is the honest failure: ten unreadable slivers in the window would be worse than a strip you
//!   scroll;
//! - **the add button is not a tab**: it is 34 points taken off the top, so it never shrinks and never moves under the
//!   pointer.
//!
//! ## What no picture shows
//!
//! That the close button's region is subtracted from the title's, so a click meant for a title cannot close the tab; and
//! that the current tab stays on screen when the strip scrolls — the rule you notice only when it is missing, as you click a
//! tab and watch the bar scroll away from it.

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

    mod.gallery.pages.tabs = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Tabs"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A strip of tabs, one of them current, with a close and an add. Tabs share the width equally between a 120-point minimum and a 200-point ceiling: equal rather than proportional to their titles, because a bar of equal tabs reads as a set and a long title needs no more room than a short one once both are truncated anyway. Under pressure every tab takes the minimum and the strip overflows, which is the honest failure — a bar that squeezed ten tabs into the window would be showing ten unreadable ones. The add button is not a tab: it is taken off the top so it never shrinks and never moves under the pointer. The close button's region is subtracted from the title's, so a click meant for a title cannot close the tab, and the current tab stays on screen when the strip scrolls."
        }

        Section{
            Caption{ text: "Room: four tabs at the 200-point ceiling, with the add button" }
            tabs_room := mod.mp.MpTabBar{width: Fill}
        }
        Section{
            Caption{ text: "Pressure: twelve tabs, all at the 120-point minimum, so the strip overflows" }
            tabs_pressure := mod.mp.MpTabBar{width: Fill}
        }
        Section{
            Caption{ text: "An empty bar: only the add button, which is still reachable" }
            tabs_empty := mod.mp.MpTabBar{width: Fill}
        }
    }
}
