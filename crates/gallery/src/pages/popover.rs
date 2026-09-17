//! The popover: a panel that opens under its trigger.
//!
//! **The panels are opened by `GALLERY_POPOVER=1` rather than by clicking.** Not
//! because clicking is broken here, but because a synthetic pointer produces no
//! hit at all in this app: every `event.hits` in a run reports `Nothing`, for
//! every control, over 100k calls. So a capture script cannot deliver a click,
//! and this is how the panel, its anchoring and its dismissal are checked.
//!
//! ## The overlay region is part of the page's structure
//!
//! The three panels are `Fill`/`Fill` **siblings of the content**, declared at
//! this page's root — not children of the rows that hold their triggers. That is
//! a requirement, not a layout preference: an overlay draw list clips to its
//! widget's rectangle, so a `Fill` panel inside a `Fit` row has nowhere to draw
//! and the popover silently does nothing. The first version of this page nested
//! them in their rows and the panels did not appear.
//!
//! A real app declares this region once at the window root, so every floating
//! surface in it — tooltips, popovers, menus, selects — shares one layer. A page
//! declares its own at its root.

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
        width: Fill, height: Fit, flow: Down, spacing: 6
    }

    let Row = View{
        width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
    }

    mod.gallery.pages.popover = View{
        // The overlay region. `Fill`/`Fill` and `Overlay`, so a panel's rectangle
        // contains the surface it draws at.
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        // ---- the content ----
        popover_body := View{
            width: Fill
            height: Fill
            flow: Down
            spacing: 20

            Label{
                width: Fit, height: Fit
                draw_text +: {text_style: title2, color: text}
                text: "Popover"
            }
            Label{
                width: Fill, height: Fit
                draw_text +: {text_style: footnote, color: text_muted}
                text: "Click a trigger. A popover is the tooltip's recipe with the caller's content instead of a label — Fill/Fill in an Overlay flow so its rectangle contains the panel, a Fit-sized plate inside it, and the panel positioned from the trigger's Area. It dismisses three ways: the trigger again, a press anywhere outside the panel, and Escape."
            }

            Section{
                Caption{ text: "A form — the panel holds controls, and they work while it is open" }
                Row{
                    pop_form := mod.mp.MpButton{ text: "New terminal" }
                }
            }

            Section{
                Caption{ text: "A short list — a menu is a popover that happens to contain rows" }
                Row{
                    pop_menu := mod.mp.MpButton{ text: "Actions" }
                }
            }

            Section{
                Caption{ text: "A tall panel — the plate is measured from its content, so it does not care how tall that makes it" }
                Row{
                    pop_tall := mod.mp.MpButton{ text: "Details" }
                }
            }
        }

        // ---- the floating surfaces ----
        //
        // Siblings of the content, at the root. See the module doc for why they
        // cannot live in the rows above.

        pop_form_panel := mod.mp.MpPopover{
            panel +: {
                // The width is named, and it has to be: this panel's rows are
                // `Fill`, and a `Fill` child contributes **nothing** to a `Fit`
                // parent's width — so a `Fit` panel measures to its widest
                // *intrinsic* child, which here is one label. The panel was 143pt
                // wide with a 260pt field inside it, and everything past the
                // label was clipped. A panel of `Fill` rows must say how wide it
                // is, exactly as `pop_menu_panel` and `pop_tall_panel` do.
                width: 300
                Label{ draw_text +: {text_style: body, color: text} text: "Spawn a terminal" }
                View{
                    width: Fill, height: Fit, flow: Down, spacing: 6
                    Caption{ text: "Working directory" }
                    mod.mp.MpTextInput{ width: 260, empty_text: "~/www" }
                }
                View{
                    width: Fill, height: Fit, flow: Down, spacing: 6
                    Caption{ text: "Options" }
                    mod.mp.MpCheckbox{ text: "Open in a new space", checked: true }
                    mod.mp.MpCheckbox{ text: "Follow the agent" }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{x: 1.0, y: 0.5}
                    mod.mp.MpButton{ style: mod.mp.ButtonStyle.Ghost, text: "Cancel" }
                    mod.mp.MpButton{ style: mod.mp.ButtonStyle.Prominent, text: "Spawn" }
                }
            }
        }

        pop_menu_panel := mod.mp.MpPopover{
            panel +: {
                width: 200
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Ghost, text: "Rename" }
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Ghost, text: "Duplicate" }
                mod.mp.Divider{}
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Destructive, text: "Delete" }
            }
        }

        pop_tall_panel := mod.mp.MpPopover{
            panel +: {
                width: 300
                Label{ draw_text +: {text_style: body, color: text} text: "Receipt" }
                mod.mp.Divider{}
                Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: caption, color: text_muted}
                    text: "A panel taller than its trigger does not care: the plate is measured from the content, not from the anchor."
                }
                mod.mp.MpProgress{ value: 0.62 }
            }
        }
    }
}
