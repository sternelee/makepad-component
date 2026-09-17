//! `MpOptionCard` — a card you choose between, with the selection ring around it.
//!
//! ## What the page is really showing
//!
//! **The ring is a wrapper border, not a spread shadow.** bezel's reasoning, which is invisible until it is wrong: a shadow's
//! spread grows the rectangle without growing its corner radius, so the halo's corners tighten relative to the frame's and the
//! two visibly drift apart by a pixel at each rounded corner. Concentric borders cannot — each element rounds itself and the
//! outer radius is the inner one plus the gap it sits behind.
//!
//! **And choosing does not move the row.** The ring is always drawn and merely transparent when unchosen, so selecting a card
//! cannot resize it. Comparing the two cards below is the test: their frames are the same size whether or not they are chosen.
//!
//! ## Which also settles a question the components answer differently
//!
//! `mp/collapsible.rs` refuses to swallow its body — a container that swallowed its children would have to re-implement layout
//! for them. This card swallows its preview, and for the opposite reason: **the ring's geometry has to wrap it**, so a caller
//! laying out the ring, the frame and the preview by hand would be re-deriving the same concentric relation at every call
//! site. Neither answer is a matter of taste; each is what the component needs.
//!
//! ## The preview rounds its own corners
//!
//! A caller's preview that paints a background must round it to the card radius. That is the one thing this component cannot
//! do for its caller — the preview is drawn by whoever put it there — and a square-cornered background inside a rounded frame
//! shows at all four corners.

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
    // A preview that paints a background, rounded to the card radius as the rule requires.
    let Swatch = View{
        width: Fill
        height: Fill
        show_bg: true
        draw_bg +: {
            color: mod.mpc.tokens.accent
            border_radius: 10.0
        }
    }

    mod.gallery.pages.option_cards = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Option Cards"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Cards you choose between, with the selection ring wrapped around the chosen one. The ring is a border drawn outside the frame rather than a shadow's spread, because a spread grows the rectangle without growing its corner radius: the halo's corners tighten relative to the frame's and the two drift apart by a pixel at every rounded corner. Concentric borders cannot do that, since each element rounds itself and the outer radius is the inner one plus the gap it sits behind. The ring is always present and only transparent when unchosen, so choosing a card never resizes it or moves the ones beside it. The preview inside must round its own background to the card radius, which is the one thing the component cannot do for its caller."
        }

        Section{
            Caption{ text: "A row: the middle card is chosen, so only its ring is opaque" }
            option_row := mod.mp.MpOptionCardRow{
                card_a := mod.mp.MpOptionCard{
                    preview := Swatch{draw_bg +: {color: #x3E63DD}}
                    caption := Label{text: "Ocean"}
                }
                card_b := mod.mp.MpOptionCard{
                    preview := Swatch{draw_bg +: {color: #x46A758}}
                    caption := Label{text: "Forest"}
                }
                card_c := mod.mp.MpOptionCard{
                    preview := Swatch{draw_bg +: {color: #xE5484D}}
                    caption := Label{text: "Ember"}
                }
            }
        }
    }
}
