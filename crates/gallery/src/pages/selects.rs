//! `MpSelect` — a trigger that shows the chosen option and a panel that lists them.
//!
//! ## A thin component, shown as a thin page
//!
//! A select is a combobox that is never typed into, so this needs no model of its own — the list, the chosen index, the
//! walking highlight and the commit all exist in `mp/combobox.rs` with their own tests. Writing a second model would have
//! meant two lists and two indices, and eventually a trigger saying one thing while the panel highlighted another.
//!
//! ## What the page shows, and what it deliberately does not
//!
//! The trigger with its chosen label, and beside it **the rows the panel would show** drawn as ordinary labels from the
//! model's own `rows()`. That is not the panel: **a floating surface cannot be composed into its trigger** — the overlay's
//! draw list is clipped to the widget's own rect, so a panel drawn inside a trigger is cut off at the trigger's bottom edge.
//! The real panel is a sibling in the page's `Overlay` area, which is the rule `mp/popover.rs` and `mp/combobox.rs` both
//! document. The preview exists so the *contents* are visible; the positioning is not this page's to show.
//!
//! ## And the rules printed
//!
//! That a select never filters (so choosing an option does not narrow the panel), that setting an index the list no longer
//! covers drops it rather than clamping, and what opening does to the highlight.

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
    let PanelPreview = View{
        width: 260, height: Fit, flow: Down, show_bg: true
        padding: 4
        draw_bg +: {
            color: mod.mpc.tokens.surface_card
            border_color: mod.mpc.tokens.border
            border_size: 1.0
            border_radius: 8.0
        }
    }

    mod.gallery.pages.selects = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Selects"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A trigger that shows the chosen option, and the options themselves. A select is a combobox that cannot be typed into, so it needs no model of its own — the list, the chosen index, the walking highlight and the commit all come from the combobox with its own tests. What the wrapper adds is that a select never filters: its panel shows every option, which here is not a special case but a consequence of never typing, and the consequence is asserted rather than left to coincide. Opening highlights the chosen row, or the first when nothing is chosen, so pressing the trigger and pressing down cannot do different things. The panel a select really opens is a sibling in the page's overlay rather than a child of the trigger, because a floating surface is clipped to its parent's rectangle — what is drawn beside the trigger is the rows the panel would contain."
        }

        Section{
            Caption{ text: "The trigger, with SQLite chosen" }
            select_trigger := mod.mp.MpSelectTrigger{width: 260}
        }
        Section{
            Caption{ text: "The rows the panel would show: three options, PostgreSQL current" }
            select_preview := PanelPreview{
                row_0 := Label{width: Fill, height: Fit, draw_text +: {text_style: body, color: text}, text: ""}
                row_1 := Label{width: Fill, height: Fit, draw_text +: {text_style: body, color: text}, text: ""}
                row_2 := Label{width: Fill, height: Fit, draw_text +: {text_style: body, color: text_muted}, text: ""}
            }
        }
    }
}
