//! The editing surface: keys, a caret, a selection, and undo.
//!
//! This is the half of an editor that needs a window — bezel splits its own the same way and says why: *"`markdown`
//! holds the document, its markdown wire form, and the painting — all of it testable without a window. What lives
//! here is the half that needs one."*
//!
//! Everything below the surface was built to be testable without a window and is:
//!
//! | part | where | tests |
//! |---|---|---|
//! | the document, `parse`/`serialize`, the layout | `makepad-markdown` | 79 |
//! | what a key **does** to a document | `makepad-markdown::edit` | in the 79 |
//! | undo, and what counts as one step | `makepad-editor` | 28 |
//! | **which key means which shortcut, the caret, the click, the paint** | here | 8 |
//!
//! ## How this page is checked
//!
//! Typing cannot be delivered by the screenshot script — a synthetic pointer produces no hit in this app — so the
//! session is driven from `GALLERY_EDITOR`: a list of `type:hello`, `enter`, `backspace`, `tab`, `left`, `undo` and
//! so on, applied through **the same methods a keypress applies**. The page then prints the document's markdown
//! after every step and reports it beside the editor, so the check is a line of stdout and a rendered document
//! rather than a reading of pixels.
//!
//! ## Not built, and named rather than implied
//!
//! No clipboard (⌘C/⌘V need a pasteboard; `makepad-clipboard` is in this workspace and is where that goes), no IME
//! composition display, no menus, no block handles, no comments, no links, and no horizontal scrolling. Each is a
//! named absence so a reader comparing this to the reference can see what is missing without reading the file.

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

    mod.gallery.pages.editor = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Editor"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "An editing surface over the document model: keys, a caret, a selection and undo. Everything under it is testable without a window and is tested — the model and its layout, what a key does to a document, and what counts as one undo step — so what is left here is deliberately the small part: which key means which shortcut, where the caret is, where a click lands, and painting it. Click in the text to place the caret, or drag to select; the selection is drawn before the text and the caret after it, so neither is painted over.\n\nType / at the start of a block for the slash menu: it opens showing every kind, narrows as you type, walks with the arrow keys, and Enter turns the block into the row you chose. The menu's rows are not painted yet — its state, its key routing and the edit it commits are, and the session below prints each step and the document after it."
        }

        Section{
            Caption{ text: "The surface. Set GALLERY_EDITOR to drive it from a run — typing cannot be delivered by the screenshot script, so the session is applied through the same methods a keypress uses. Its default session ends with the slash menu: /, a query, two steps down, one up, and Enter" }
            editor_surface := mod.mp.MpEditor{}
        }

        Section{
            Caption{ text: "What the surface holds, as markdown — the wire form, which is what a caller saves" }
            editor_wire := mod.mp.MpCodeBlock{}
            editor_state := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(state)"
            }
            editor_script := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(script)"
            }
        }
    }
}
