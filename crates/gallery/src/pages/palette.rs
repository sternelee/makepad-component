//! The command palette, live.
//!
//! Every other page shows a component working; this one shows the *interaction*
//! working, which is a different claim. The field and the list were both already on
//! other pages. What is new here is the part in between: a query that re-ranks the
//! list on every keystroke, and a **cursor that follows its own row** while the list
//! shrinks underneath it.
//!
//! That second thing is the reason `mp/palette.rs` exists rather than a note telling
//! callers to wire a field to a list. A cursor held as a *position* is correct until
//! the first query that filters its row out, at which point it has silently moved
//! onto a different command — and running the wrong command leaves no trace in any
//! log. So the cursor here is held as an **original index** and remapped by identity.
//!
//! ## Why the path is `command_palette`
//!
//! The first version registered `mod.gallery.pages.palette` — which the **colour**
//! palette page already owned, so this module was silently overwriting that page's
//! DSL tree. Nothing errored: the colour page's rail row would simply have rendered
//! a command palette. The crate's own uniqueness test caught it, which is the whole
//! reason those hand-maintained slot tables have tests.
//!
//! ## How this page was checked
//!
//! Typing is not deliverable by the screenshot script (synthetic pointers do not hit
//! in this app), so the query comes from `GALLERY_PALETTE_QUERY` and the app runs the
//! *same* `apply_palette` the `changed` handler runs. The app also prints the query,
//! the ranked view and the active original index on every change, so the check is a
//! line of stdout rather than a reading of pixels — the first page in this port whose
//! evidence is not a screenshot, and it is stronger for it.

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

    mod.gallery.pages.command_palette = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Command Palette"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A query field over a filtered list, with the cursor live underneath. The contract that matters is that a selection is reported as an index into the ORIGINAL command list, never into the filtered view — rank() already returns original indices for exactly that reason, and a caller matching on a filtered position would run the wrong command the moment a query is typed. The cursor is held as an original index too, and remapped by identity, so the row the reader is on stays under them while the list shrinks. When that row is genuinely filtered out the cursor falls back to the top, which is rank's best answer to the new query."
        }

        Section{
            Caption{ text: "The panel, live. surface_dialog rather than surface_card, because a palette is the topmost thing on screen — the same rung every other overlay uses, so a palette and a dialog read as the same distance from the page" }
            View{
                width: Fill
                height: Fit
                flow: Right
                spacing: 20
                align: Align{x: 0.0, y: 0.0}

                mod.mp.MpPalette{
                    width: 460
                }

                View{
                    width: Fill
                    height: Fit
                    flow: Down
                    spacing: 10
                    palette_state := Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: caption, color: text_muted}
                        text: "(state)"
                    }
                    palette_cursor_note := Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: caption, color: text_faint}
                        text: "(cursor)"
                    }
                }
            }
        }

        Section{
            Caption{ text: "Set GALLERY_PALETTE_QUERY to drive it from a run: the app fills the field, re-ranks and remaps the cursor through the same apply_palette the changed handler calls. Try “de”, then “del” — the second filters the row the cursor was on out of the view, and the cursor must land on a row that exists" }
            Caption{ text: "Commands: New Terminal · Toggle Terminal · Go to File · Command Palette · Split Right · Delete · Duplicate · Move to Space · Rename… · Close Window" }
        }
    }
}
