//! Undo and redo, shown as a stack.
//!
//! An undo stack is the piece of an editor that is **wrong in a way nobody reports**: the
//! button lights up and the document goes somewhere it was never in. Every fault is a
//! state-order fault, so none of them shows in a screenshot and none of them throws.
//!
//! Two are worth seeing rather than reading about, and both are visible below:
//!
//! - **A push after an undo abandons the redo branch.** Undo, then type: the forward
//!   history is gone. The steps the page ran are printed, so the branch disappearing is a
//!   line you can read rather than a claim in a doc comment.
//! - **A bounded history drops the oldest state and the cursor has to move with it.** The
//!   `depth` readout is what shows it: a cursor left behind makes undo skip a step, which
//!   is the one fault here that produces a wrong document rather than a wrong button.
//!
//! The list marks the state the cursor is on, and the buttons are wired to the same type
//! the script drives — so the page is the component, not a picture of it.

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

    mod.gallery.pages.history = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "History"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "One Vec with a cursor, which is the shape that makes both faults expressible as arithmetic on a single index. Two stacks — an undo one and a redo one — would make the truncation rule an operation on two things that must stay consistent, and the bounded case an operation on one of them while the other holds the states that were just dropped. A Vec plus a cursor keeps the current state, the undoable depth and the redoable depth as three readings of one number."
        }

        Section{
            Caption{ text: "A scripted session. Set GALLERY_HISTORY to drive it: a comma-separated list of push:X, undo and redo, applied through the same type the buttons below are wired to. The default is a session that edits a document, undoes, then edits again — which abandons the redo branch, and the printed steps show the branch going away" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 20
                align: Align{x: 0.0, y: 0.0}

                View{
                    width: 420, height: Fit, flow: Down, spacing: 8
                    View{
                        width: Fill, height: Fit, flow: Right, spacing: 6
                        history_undo := mod.mp.MpButtonSmall{
                            text: "Undo"
                            glyph: "\u{f0e2}"
                        }
                        history_redo := mod.mp.MpButtonSmall{
                            text: "Redo"
                            glyph: "\u{f01e}"
                        }
                    }
                    history_states := mod.mp.MpMenu{}
                }

                View{
                    width: Fill, height: Fit, flow: Down, spacing: 10
                    history_depth := Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: caption, color: text_muted}
                        text: "(depth)"
                    }
                    history_note := Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: caption, color: text_faint}
                        text: "(note)"
                    }
                    Caption{ text: "The list is the retained buffer, oldest first; the row marked current is where the cursor is. A push after an undo truncates everything after the cursor — not overwrites it, which is what a longer abandoned branch would leave behind — and the capacity is what makes the oldest row fall off the front." }
                }
            }
        }
    }
}
