//! The menu: a popover of commands, and the two things that make it a menu.
//!
//! A menu is a select with commands instead of values, so it uses the same
//! template — a face in the flow, a popover in the page's overlay region, a list
//! of rows inside it, and the app joining them. What is genuinely different is
//! where the *lines* go, and it is the reason `MpList` grew two flags:
//!
//! - **Lines only at group boundaries.** A hairline between every row reads as a
//!   table; between groups it reads as sections. `show_row_lines: false` plus a
//!   `starts_group()` row gives that, and it is one flag rather than a second
//!   widget because a menu *is* a list of rows that happen to be commands.
//! - **Destructive rows.** A menu is scanned before it is read, so a destructive
//!   action has to be visible as one before its label is: the row's label takes
//!   the danger tone, not a red mark beside it.
//!
//! The shortcuts are the list's existing `detail`, pushed to the far edge. Nothing
//! new was needed for them, which is the composition paying for itself.

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
        width: Fill, height: Fit, flow: Right, spacing: 10, align: Align{y: 0.5}
    }

    // A menu's face: a label and a disclosure chevron. The same `MpButton` shape a
    // select's face uses, because a menu button and a select button are the same
    // object with different contents.
    let Face = mod.mp.MpButton{
        glyph: "\u{f0d7}"
        glyph_trailing: true
        text: "Menu"
    }

    mod.gallery.pages.menu = View{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        menu_body := View{
            width: Fill
            height: Fill
            flow: Down
            spacing: 20

            Label{
                width: Fit, height: Fit
                draw_text +: {text_style: title2, color: text}
                text: "Menu"
            }
            Label{
                width: Fill, height: Fit
                draw_text +: {text_style: footnote, color: text_muted}
                text: "A select with commands instead of values, so it is the same composition: a face in the flow, a popover in the page's overlay region, a list of rows inside it, and the app joining them. What is different is where the lines go — a hairline between every row reads as a table, between groups it reads as sections, which is `show_row_lines: false` plus a `starts_group()` row — and that a destructive action has to be visible as one before it is read, which is the row's label taking the danger tone rather than a red mark beside it."
            }

            Section{
                Caption{ text: "A grouped menu — three groups, a separator where each one starts, and a destructive row in the last. The shortcuts are the list's existing detail, pushed to the far edge; nothing new was needed for them" }
                Row{
                    Label{
                        width: 120, height: Fit
                        draw_text +: {text_style: body, color: text}
                        text: "Terminal"
                    }
                    menu_face_a := Face{}
                }
            }

            Section{
                Caption{ text: "A flat menu — no groups, so no lines at all. The same component with no separators set" }
                Row{
                    menu_face_b := Face{text: "Layout"}
                }
            }

            Section{
                Caption{ text: "The readout — what a selection reports. The app closes the menu and shows what was chosen, which is the whole wiring a menu needs" }
                Row{
                    menu_face_c := Face{text: "Choose"}
                    menu_readout := Label{
                        width: Fit, height: Fit
                        draw_text +: {text_style: footnote, color: text_muted}
                        text: "nothing chosen yet"
                    }
                }
            }
        }

        menu_panel_a := mod.mp.MpPopover{
            panel +: {
                width: 260
                menu_list_a := mod.mp.MpMenu{}
            }
        }
        menu_panel_b := mod.mp.MpPopover{
            panel +: {
                width: 220
                menu_list_b := mod.mp.MpMenu{}
            }
        }
        menu_panel_c := mod.mp.MpPopover{
            panel +: {
                width: 240
                menu_list_c := mod.mp.MpMenu{}
            }
        }
    }
}
