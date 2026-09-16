//! The scroll container, and the chrome it replaced.
//!
//! This page exists for a reason the others do not: the component it documents was
//! already in use on **every** page before it existed. Each page scrolled inside a
//! bare Makepad `ScrollYView`, whose handle is painted from Makepad's own theme —
//! so the one piece of chrome on every page was the one piece not designed here,
//! and it was invisible because a scroll bar is chrome and nobody looks at chrome.
//!
//! So the page shows the *other* axis, which nothing else in the gallery uses: a
//! wide block inside `MpScrollBoth`. The vertical bar the whole gallery scrolls
//! through is above and below this page and every other one.

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

    mod.gallery.pages.scroll = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Scroll"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A scroll bar is one of the few places the platform names nothing, so these numbers are chosen rather than measured — and chosen from the palette rather than invented. The handle is the three-rung ink ladder every other piece of chrome uses, text_faint at rest, text_muted on hover, text while dragging, so a scroll bar recedes and comes forward the way a border does. No bevel: a hairline around a 6pt bar is most of the bar. And a square-ish corner, because a 6pt handle with a 3pt radius is a capsule, and a capsule reads as a control rather than as a position."
        }

        Section{
            Caption{ text: "Both axes — the one shape nothing else in the gallery needs, for a canvas or a wide table. Drag a bar, or drag the content" }
            mod.mp.MpScrollBoth{
                width: 520
                height: 260
                mod.mp.SurfaceSunken{
                    width: 1100
                    height: 640
                    draw_bg +: {border_radius: instance(8.0)}
                    Label{
                        width: Fill
                        height: Fit
                        draw_text +: {text_style: body, color: text_muted}
                        text: "1100 × 640 inside 520 × 260. Both handles are drawn from the same three ink rungs, and both reserve a 12pt track with a 6pt handle in it — so the bar can be grabbed without taking a column of layout away from the content."
                    }
                }
            }
        }

        Section{
            Caption{ text: "A vertical one on its own, for the case where a horizontal bar would be a bug. The whole gallery scrolls through one of these" }
            mod.mp.MpScroll{
                width: 520
                height: 200
                View{
                    width: Fill
                    height: 520
                    flow: Down
                    spacing: 8
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: body, color: text}
                        text: "Four hundred points of content in two hundred points of box."
                    }
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: footnote, color: text_muted}
                        text: "A separate prototype from the two-axis one rather than a flag on it, because a horizontal scroll bar on a page of prose is a bug — so the default has to be the one that cannot do it."
                    }
                }
            }
        }
    }
}
