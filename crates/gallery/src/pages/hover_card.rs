//! The hover card, and the four ways its timing can go wrong.
//!
//! gpui owns this timing: a tooltip there has a 500ms delay built in and stays alive while the
//! pointer is inside it, which is why bezel's `hover_card.rs` is content with no timing in it.
//! **Makepad owns nothing**, so this port had to write the machine — and it is the substance,
//! because all four failure modes are visible the moment a reader moves a mouse:
//!
//! 1. A card that opens instantly **flickers**: the pointer crossing the window passes dozens
//!    of triggers and each one that opens is a flash nobody asked for.
//! 2. A card that opens on a *passing* pointer must not, which is the same delay from the
//!    other side: leaving before it elapses cancels rather than merely not-opening-yet.
//! 3. A card the pointer can enter must not close when the pointer **enters it** — the pointer
//!    leaves the trigger and arrives at the card, and closing on "left the trigger" closes the
//!    thing being reached for.
//! 4. A card must not flicker when the pointer crosses the **gap**, which belongs to neither.
//!
//! All four are checked by the module's tests with no window, and this page shows the machine's
//! decisions as a log: a synthetic pointer cannot produce a hover event in this app, so a
//! screenshot of a hover card is only possible with it pinned, and the *timing* is not
//! photographable at all. The log is the evidence.
//!
//! The card is in the overlay region and pinned with `GALLERY_POPOVER=1`, so the visual is
//! checkable too — but the picture is of a card that was told to open, not of one that decided
//! to.

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

    mod.gallery.pages.hover_card = View{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        hover_card_body := View{
            width: Fill
            height: Fill
            flow: Down
            spacing: 20

            Label{
                width: Fit, height: Fit
                draw_text +: {text_style: title2, color: text}
                text: "Hover Card"
            }
            Label{
                width: Fill, height: Fit
                draw_text +: {text_style: footnote, color: text_muted}
                text: "A card that opens on hover and can itself be hovered, for a preview the pointer can travel into. The card is the easy part — it is a title, a body and a quiet line in a wider box than a tooltip, because this one carries prose. The state machine is the module: a delay so a pointer crossing the window does not open anything, a cancellation when the pointer leaves before the delay elapses, an arm that keeps the card open while the pointer is inside it, and a grace period for the strip between the trigger and the card that belongs to neither."
            }

            Section{
                Caption{ text: "The decisions, from a scripted pointer. Set GALLERY_HOVER to a list of presence:milliseconds steps — trigger, card, outside — and every change the machine decides is printed. The default is a sweep that must amount to nothing, then a rest that opens, then a gap crossing, then reading the card, then leaving" }
                hover_log := Label{
                    width: Fill
                    height: Fit
                    draw_text +: {text_style: caption, color: text_faint}
                    text: "(log)"
                }
                hover_replay := Label{
                    width: Fill
                    height: Fit
                    draw_text +: {text_style: caption, color: text_faint}
                    text: "(replay)"
                }
            }

            Section{
                Caption{ text: "The numbers, and where they come from. 500ms is gpui's own tooltip delay, so a control here opens when the same control would open under the reference implementation — the same convention as every other number that reached the theme because a platform named it. The 150ms grace is chosen: it only has to cover a hand crossing a small gap" }
                hover_numbers := Label{
                    width: Fill
                    height: Fit
                    draw_text +: {text_style: caption, color: text_faint}
                    text: "(numbers)"
                }
            }

            Section{
                Caption{ text: "What a screenshot can and cannot show here. A synthetic pointer produces no hover event in this app, so the card below is drawn because it was told to open — it is in the overlay region and pinned by GALLERY_POPOVER=1. The picture proves the card's shape; the log proves the timing. Neither would be enough on its own, which is why both are here" }
            }
        }

        hover_card_anchor := View{
            width: 1
            height: 1
            show_bg: true
            draw_bg +: {color: #x00000000}
        }

        hover_card := mod.mp.MpHoverCard{
            margin: Inset{left: 40.0, top: 300.0}
            hovercard_title := Label{text: "clearloop"}
            hovercard_body := Label{
                text: "Builds desktop software in Rust. Maintainer of a component library that ports a design system to a second UI toolkit, and of the tooling around it."
            }
            hovercard_meta := Label{text: "Maintainer \u{b7} 412 commits \u{b7} last active today"}
        }
    }
}
