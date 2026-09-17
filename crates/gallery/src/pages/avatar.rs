//! Avatars: initials, the size ladder, presence, and the overlapped group.
//!
//! The page checks three things a test cannot: that the initials are optically
//! centred in the face at every rung, that a presence dot reads as *attached* to
//! its avatar rather than floating between two overlapped ones, and that an
//! overlapped row reads as one object.

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

    mod.gallery.pages.avatar = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Avatar"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Initials are the case a component has to own — `ML` set in a circle is a design, and every application that writes it by hand writes it differently. The plate behind them is derived from the name, so a list of people comes out distinguishable without the caller assigning colours; the same name is always the same colour, because a random tone would make someone a different person every frame. An image, when there is one, is the caller's: `set_text` takes a glyph."
        }

        Section{
            Caption{ text: "The ladder — 20, 28 and 40pt. Initials scale with the face, and the presence dot is 11% of its diameter at every rung" }
            Row{
                avatar_small := mod.mp.MpAvatarSmall{}
                avatar_regular := mod.mp.MpAvatar{}
                avatar_large := mod.mp.MpAvatarLarge{}
            }
        }

        Section{
            Caption{ text: "Derived colours — five names, and the same five again below in the same order. A name keeps its colour across runs" }
            Row{
                av_a := mod.mp.MpAvatar{}
                av_b := mod.mp.MpAvatar{}
                av_c := mod.mp.MpAvatar{}
                av_d := mod.mp.MpAvatar{}
                av_e := mod.mp.MpAvatar{}
            }
            Row{
                av_a2 := mod.mp.MpAvatar{}
                av_b2 := mod.mp.MpAvatar{}
                av_c2 := mod.mp.MpAvatar{}
                av_d2 := mod.mp.MpAvatar{}
                av_e2 := mod.mp.MpAvatar{}
            }
        }

        Section{
            Caption{ text: "Presence — the same six tones a badge uses, so a person's state and a badge's state are one vocabulary" }
            Row{
                pres_off := mod.mp.MpAvatar{}
                pres_ok := mod.mp.MpAvatar{}
                pres_away := mod.mp.MpAvatar{}
                pres_failed := mod.mp.MpAvatar{}
                pres_busy := mod.mp.MpAvatar{}
                pres_none := mod.mp.MpAvatar{}
            }
        }

        Section{
            Caption{ text: "An overlapped group — the gap is a fraction of a face, so the group reads as one object at any size, and the tail is a count rather than a person" }
            avatar_row := mod.mp.MpAvatarRow{}
            avatar_group := mod.mp.MpAvatarGroup{}
        }

        Section{
            Caption{ text: "In a row of content — which is where the plate and the initials have to earn their keep" }
            View{
                width: Fill, height: Fit, flow: Down, spacing: 10
                Row{
                    av_row_a := mod.mp.MpAvatar{}
                    Label{ width: Fit, height: Fit, draw_text +: {text_style: body, color: text} text: "Ada Lovelace" }
                    mod.mp.MpTagSmall{ tone: mod.mp.StatusTone.Neutral, text: "owner" }
                    mod.mp.MpBadgeSmall{ tone: mod.mp.StatusTone.Success, text: "online" }
                }
                Row{
                    av_row_b := mod.mp.MpAvatar{}
                    Label{ width: Fit, height: Fit, draw_text +: {text_style: body, color: text} text: "Grace Brewster Hopper" }
                    mod.mp.MpTagSmall{ tone: mod.mp.StatusTone.Neutral, text: "reviewer" }
                    mod.mp.MpBadgeSmall{ tone: mod.mp.StatusTone.Busy, text: "in a call" }
                }
                Row{
                    av_row_c := mod.mp.MpAvatar{}
                    Label{ width: Fit, height: Fit, draw_text +: {text_style: body, color: text} text: "Alan Turing" }
                    mod.mp.MpTagSmall{ tone: mod.mp.StatusTone.Neutral, text: "contributor" }
                    mod.mp.MpBadgeSmall{ tone: mod.mp.StatusTone.Neutral, text: "away" }
                }
            }
        }
    }
}
