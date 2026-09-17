//! `MpDialog` — a full-screen modal with a card in the middle.
//!
//! ## What the page shows
//!
//! The honest picture: a dimmed window with a card in it, the card's header/body/footer laid out at their own paddings, and
//! the backdrop's opacity at the partial value a modal uses so the window it is modal *to* is still legible behind it.
//!
//! ## And the rule a picture cannot show
//!
//! **Showing it twice changes nothing the second time.** Re-playing the entry animation while the dialog is already up makes
//! it flash, so a caller that opens on every unrelated event would produce a strobing dialog. The two operations report
//! whether they changed anything and the animator is played only when they did. That is printed.
//!
//! ## The duration is the catalog's
//!
//! The backdrop animates over `mod.motion.dialog_in.duration` — 180ms on `EASE`, from `makepad_motion` — and not a number
//! written in the DSL. The widget this replaces had `duration: 0.2` inline; a test reads this module's source and fails if a
//! bare duration reappears in the animator, because the rule is otherwise only visible to a reader who knows where to look.

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

    mod.gallery.pages.dialogs = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Dialogs"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A full-screen modal with a card in the middle. Rust's half of it is one boolean and the animation that boolean drives, so the module says so rather than pretending to have logic it does not: the rest is the tree, which is the caller's to fill. Showing it twice changes nothing the second time, because re-playing the entry animation while the dialog is already up makes it flash — a caller that opened on every unrelated event would get a strobing dialog. A click on the backdrop closes it and a click on the card does not, which is why the hit is taken on the backdrop's own area: the dialog fills the window, so a hit on the dialog as a whole would fire on a press inside it. The backdrop is dimmed to 0.8 and not to 1.0, since a modal that hides the window completely loses the context it is modal to."
        }

        Section{
            Caption{ text: "A form dialog, open: the backdrop is partial so the page behind stays legible" }
            dialog_form := mod.mp.MpDialog{
                content +: {
                    dialog := mod.mp.MpDialogLarge{
                        header +: {
                            title +: {text: "New Connection"}
                            description +: {text: "Point at a database to browse." visible: true}
                        }
                        body +: {
                            View{
                                width: Fill, height: Fit, flow: Down, spacing: 8
                                Label{
                                    width: Fill, height: Fit
                                    draw_text +: {text_style: body, color: text}
                                    text: "Host: localhost"
                                }
                                Label{
                                    width: Fill, height: Fit
                                    draw_text +: {text_style: body, color: text_muted}
                                    text: "Port: 5432"
                                }
                            }
                        }
                        footer +: {
                            View{
                                width: Fit, height: Fit, flow: Right, spacing: 8
                                cancel := mod.mp.MpButton{text: "Cancel"}
                                connect := mod.mp.MpButton{text: "Connect"}
                            }
                        }
                    }
                }
            }
        }
        Section{
            Caption{ text: "An alert dialog: a narrow card with its header and footer centred" }
            dialog_alert := mod.mp.MpDialog{
                content +: {
                    dialog := mod.mp.MpAlertDialog{
                        header +: {
                            title +: {text: "Discard changes?"}
                        }
                        body +: {
                            description +: {text: "This cannot be undone."}
                        }
                        footer +: {
                            View{
                                width: Fit, height: Fit, flow: Right, spacing: 12
                                keep := mod.mp.MpButton{text: "Keep"}
                                discard := mod.mp.MpButton{text: "Discard"}
                            }
                        }
                    }
                }
            }
        }
        Section{
            Caption{ text: "A dialog nothing has opened" }
            dialog_closed := mod.mp.MpDialog{}
        }
    }
}
