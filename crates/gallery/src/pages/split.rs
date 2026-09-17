//! `MpSplitPane` — two panes and the divider you drag between them.
//!
//! ## The whole component is one number
//!
//! A split holds one piece of state: how wide the leading pane is. So this page is really a picture of an arithmetic — and two
//! of its rules are things a picture cannot show, which is why they are printed.
//!
//! - **The divider is grabbed wider than it is drawn.** 6 points of ink, 12 points of target: the difference is a divider you
//!   can take hold of without aiming. The printed hit tests show the region's edges.
//! - **A drag is measured from the press, not accumulated frame by frame.** An accumulated drag drifts the moment a clamp
//!   bites: the frames spent against the limit are discarded from the sum, so dragging past the minimum and back leaves the
//!   pane short of where the pointer is. The print walks exactly that — slam past the minimum, then come back to a position
//!   inside the range, and the pane is where the pointer is.
//!
//! ## And a degenerate window draws two narrow panes
//!
//! A window narrower than both minimums plus the divider cannot satisfy both, so the leading pane takes what is left rather
//! than a negative width — a pane running backwards off the window is the alternative.

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
    let Pane = View{
        width: Fill, height: Fill, flow: Down, show_bg: true
        padding: 12
        draw_bg +: {color: mod.mpc.tokens.surface}
    }

    mod.gallery.pages.split = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Split Pane"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Two panes and the divider between them. A split holds one number — how wide the leading pane is — so the module is mostly the arithmetic of a drag, extracted into functions rather than written inside an event handler. Both panes keep a minimum width and the divider's own width is inside that budget, since a pane measured to the divider's centre would let the divider hang off the window's edge. The divider is grabbed over 12 points and drawn in 6, because a hairline is easy to see and hard to hit. A drag measured from the press comes straight back when you overshoot a limit; one accumulated frame by frame does not."
        }

        Section{
            Caption{ text: "A 300-point sidebar, fill on the right, and the divider between them" }
            split_a := mod.mp.MpSplitPane{
                height: 220
                left := Pane{
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: body, color: text}
                        text: "Sidebar"
                    }
                }
                right := Pane{
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: body, color: text}
                        text: "Details"
                    }
                }
            }
        }
        Section{
            Caption{ text: "The same, with a wider sidebar" }
            split_b := mod.mp.MpSplitPane{
                height: 160
                left_width: 420
                left := Pane{}
                right := Pane{}
            }
        }
    }
}
