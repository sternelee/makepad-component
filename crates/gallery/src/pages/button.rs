//! The button page: the four shipped looks, at each control size.


use makepad_widgets::*;
script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    let Caption = Label{
        width: Fit
        height: Fit
        draw_text +: {text_style: caption, color: text_muted}
    }

    mod.gallery.pages.button = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Button"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Four looks, because four is how many there are. The v2 set shipped nine and most were not looks: Accent was Prominent in another hue, which is a brand decision, and Link and Text were Ghost with tighter padding, which is a content decision the caller already owns."
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Default — the everyday plate" }
            default_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                mod.mp.MpButton{ text: "Save" }
                mod.mp.MpButton{ text: "Disabled" disabled: true }
                mod.mp.MpButtonSmall{ text: "Small" }
                mod.mp.MpButtonLarge{ text: "Large" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Prominent — one per view, because its job is to be the only one" }
            prominent_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Prominent, text: "Continue" }
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Prominent, text: "Disabled" disabled: true }
                mod.mp.MpButtonSmall{ style: mod.mp.ButtonStyle.Prominent, text: "Small" }
                mod.mp.MpButtonLarge{ style: mod.mp.ButtonStyle.Prominent, text: "Large" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Ghost — the dismiss, the cancel, the third action" }
            ghost_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Ghost, text: "Cancel" }
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Ghost, text: "Disabled" disabled: true }
                mod.mp.MpButtonSmall{ style: mod.mp.ButtonStyle.Ghost, text: "Small" }
                mod.mp.MpButtonLarge{ style: mod.mp.ButtonStyle.Ghost, text: "Large" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Destructive — the semantics are in the paint, not the label" }
            destructive_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Destructive, text: "Delete" }
                mod.mp.MpButton{ style: mod.mp.ButtonStyle.Destructive, text: "Disabled" disabled: true }
                mod.mp.MpButtonSmall{ style: mod.mp.ButtonStyle.Destructive, text: "Small" }
                mod.mp.MpButtonLarge{ style: mod.mp.ButtonStyle.Destructive, text: "Large" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Focus — Tab to reach them, Enter or Space to fire" }
            focus_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                focus_one := mod.mp.MpButton{ text: "First" }
                focus_two := mod.mp.MpButton{ text: "Second" }
                focus_three := mod.mp.MpButton{ text: "Third" }
            }
        }

        View{
            width: Fill, height: Fit, flow: Down, spacing: 6
            Caption{ text: "Clicks" }
            click_row := View{
                width: Fill, height: Fit, flow: Right, spacing: 8, align: Align{y: 0.5}
                click_me := mod.mp.MpButton{ style: mod.mp.ButtonStyle.Prominent, text: "Click me" }
                click_count := Label{
                    width: Fit, height: Fit
                    draw_text +: {text_style: body, color: text_muted}
                    text: "no clicks yet"
                }
            }
        }
    }
}

/// The button page's own counter, so the page proves the action path end to end
/// rather than only painting.
pub struct ButtonPage;

impl ButtonPage {
    /// The DSL path of the button the page listens to.
    pub fn button_path() -> &'static [LiveId] {
        ids!(click_me)
    }
}
