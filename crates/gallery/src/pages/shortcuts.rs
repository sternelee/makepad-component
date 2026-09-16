//! A shortcut sheet whose labels are resolved, not typed.
//!
//! Every list below is filled from a `Keymap` declared in the app, and each row's
//! trailing chord is `Keymap::label(action, platform)`. That is the whole claim: **a
//! hand-typed accelerator is a claim nothing checks** — bind `cmd-b` elsewhere and a
//! button that says `⌘B` in a string literal goes on saying it. Here the label cannot
//! drift from the binding, because there is only one of them.
//!
//! Both notations are shown for one keymap, which is the second half of the argument: two
//! hand-maintained columns would be two things to keep right, and Apple's modifier order
//! (`⌃⌥⇧⌘`) is not Windows' (`Win+Ctrl+Alt+Shift`), so the two columns can't be a font
//! swap anyway.
//!
//! The conflict list is built from a **deliberately broken** keymap, because a report that
//! only ever prints nothing is indistinguishable from a report that does not work.

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

    mod.gallery.pages.shortcuts = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Shortcuts"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Every shortcut the library prints — a menu row's trailing ⌘T, a list item's detail, a tooltip — is a claim about the keymap. A claim written by hand is one that nothing verifies: bind cmd-b to something else and the button still says ⌘B. So the label is resolved from a declared keymap rather than typed beside the thing it describes, and an action with nothing declared prints nothing rather than a chord that is no longer true."
        }

        Section{
            Caption{ text: "macOS. The modifier order here is Apple's — control, option, shift, command — and is applied by the formatter, never taken from how the chord was written. So cmd+shift+p prints ⇧⌘P and not ⌘⇧P, which is what preserving the input order gives and which looks fine until two shortcuts sit side by side" }
            keys_mac := mod.mp.MpMenu{}
        }

        Section{
            Caption{ text: "Windows, from the same keymap — not a second list. Its order is pinned by two examples that disagree with each other: Win+Shift+S is why the platform key leads, Ctrl+Alt+Del is why Ctrl comes before Alt. Neither order is Apple's and neither is alphabetical, so this is not one formatter with the letters swapped" }
            keys_other := mod.mp.MpMenu{}
        }

        Section{
            Caption{ text: "The conflicts in a deliberately broken keymap. Two actions on one chord is a real bug: one of them silently loses, and which one depends on dispatch order, so the app cannot tell the user which will fire. Note that the pairs below are spelled differently from each other — a config spelling and a glyph run — and are still seen as one chord" }
            keys_conflicts := mod.mp.MpMenu{}
        }

        Section{
            Caption{ text: "What the keymap says about itself" }
            keys_note := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: caption, color: text_faint}
                text: "(note)"
            }
        }
    }
}
