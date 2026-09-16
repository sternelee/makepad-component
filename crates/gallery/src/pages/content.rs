//! Group boxes and key caps: the two small things every real screen needs.
//!
//! The page's own checks are the two decisions the components exist to make: that
//! a group box's heading starts where its body does (inside the card's padding),
//! and that a row of key caps lines up because the caps share one face and one
//! height.

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
        width: Fill, height: Fit, flow: Right, spacing: 6, align: Align{y: 0.5}
    }

    mod.gallery.pages.content = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Content"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Two small things. A group box is a style rather than a widget — a titled card, with no state and no hit area, so it is a DSL prototype over the card the surface module already defines; its one decision is that the heading row sits inside the card's padding, so a title starts where its body does. A key cap is a widget because it is shaped rather than styled: a plate, a hairline, a mono face and a height that has to sit on the baseline of the text it annotates, and three things that must agree are three things a call site gets wrong."
        }

        Section{
            Caption{ text: "A group box — a heading, a description, and a body" }
            mod.mp.MpGroupBox{
                width: 460
                header +: {
                    title +: {text: "Terminal"}
                    description +: {text: "What the daemon hosts, and how many of each kind are running.", visible: true}
                }
                body +: {
                    // Named directly, so the app can find it by id from the root —
                    // `ids!` resolves by name anywhere in the tree, and a `this`
                    // reference is not a thing the DSL has.
                    group_list := mod.mp.MpList{}
                }
            }
        }

        Section{
            Caption{ text: "A plain group box — no frame of its own, for grouping inside a card that already has one. The same heading inset, so the two kinds line up when stacked" }
            mod.mp.SurfaceCard{
                width: 460
                flow: Down
                spacing: 12
                mod.mp.MpGroupBoxPlain{
                    header +: {title +: {text: "Inside a card"}}
                    body +: {
                        Label{
                            width: Fill, height: Fit
                            draw_text +: {text_style: body, color: text_muted}
                            text: "A plain group box adds a heading and a rhythm to a card that is already framed, without drawing a second border a few points inside the first."
                        }
                    }
                }
                mod.mp.MpGroupBoxPlain{
                    header +: {title +: {text: "A second one"}}
                    body +: {
                        Label{
                            width: Fill, height: Fit
                            draw_text +: {text_style: body, color: text_muted}
                            text: "Two of them in one card is the shape a settings pane takes."
                        }
                    }
                }
            }
        }

        Section{
            Caption{ text: "Key caps — the caps share a face and a height, which is why a row of them lines up and a row of styled labels does not" }
            Row{
                kbd_cmd := mod.mp.MpKbd{}
                Label{ width: Fit, height: Fit, draw_text +: {text_style: caption, color: text_muted} text: "+" }
                kbd_shift := mod.mp.MpKbd{}
                Label{ width: Fit, height: Fit, draw_text +: {text_style: caption, color: text_muted} text: "+" }
                kbd_p := mod.mp.MpKbd{}
                Label{ width: Fit, height: Fit, draw_text +: {text_style: footnote, color: text} text: "Go to file" }
            }
        }

        Section{
            Caption{ text: "A shortcut list — the shape a menu, a dialog footer and a command palette all end with. The cap is the same component at every row, and the descriptions line up because the caps do" }
            View{
                width: 460
                height: Fit
                flow: Down
                spacing: 8
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 6, align: Align{x: 1.0, y: 0.5}
                    Label{ width: Fill, height: Fit, draw_text +: {text_style: body, color: text} text: "Toggle terminal" }
                    mod.mp.MpKbd{ text: "⌃`" }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 6, align: Align{x: 1.0, y: 0.5}
                    Label{ width: Fill, height: Fit, draw_text +: {text_style: body, color: text} text: "Command palette" }
                    mod.mp.MpKbd{ text: "⇧⌘P" }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 6, align: Align{x: 1.0, y: 0.5}
                    Label{ width: Fill, height: Fit, draw_text +: {text_style: body, color: text} text: "Close window" }
                    mod.mp.MpKbd{ text: "Esc" }
                }
                View{
                    width: Fill, height: Fit, flow: Right, spacing: 6, align: Align{x: 1.0, y: 0.5}
                    Label{ width: Fill, height: Fit, draw_text +: {text_style: body, color: text} text: "A long word in a cap" }
                    mod.mp.MpKbd{ text: "Space" }
                }
            }
        }

        Section{
            Caption{ text: "Small — the chip rung, for a dense row" }
            Row{
                mod.mp.MpKbdSmall{ text: "A" }
                mod.mp.MpKbdSmall{ text: "F1" }
                mod.mp.MpKbdSmall{ text: "⌘" }
                mod.mp.MpKbdSmall{ text: "Esc" }
            }
        }
    }
}
