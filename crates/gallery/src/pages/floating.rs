//! The draggable panel, and the two rules that make it draggable.
//!
//! A panel the reader positions rather than the layout does: a meter, an inspector, a
//! detached preview.
//!
//! ## The press is heard on the box and the moves are heard on the layer
//!
//! A press has to be **inside** the panel, or a click anywhere on the page would start dragging
//! it. A move has to be heard on something **larger** than the box, or the gesture stalls the
//! first time the pointer outruns a frame — the pointer lands outside a box-sized hitbox, the
//! panel stops hearing moves, and the box is stranded behind the cursor. So `MpFloating` takes
//! the press itself and the app forwards the moves, naming the layer they are heard on:
//!
//! ```ignore
//! self.ui.mp_floating(cx, ids!(float_panel)).drag(cx, event, layer);
//! ```
//!
//! The page's layer is the whole content area, which is the point — it is much bigger than the
//! panel, so there is nowhere inside it the pointer can outrun.
//!
//! ## The grab offset is what stops the panel jumping
//!
//! A drag that put the panel's top-left at the pointer would snap the moment it started: grab a
//! panel by its title bar and it would jump so the pointer held its corner. The press records
//! where *inside the panel* it landed and every move keeps that point under the pointer — which
//! is also why nothing is clamped. A panel dragged half off the window stays there, and because
//! the offset is preserved it can always be dragged back.
//!
//! ## What the page can and cannot show
//!
//! A synthetic pointer produces no press in this app, so the gesture is driven from
//! `GALLERY_FLOAT` — a list of `press:x,y`, `move:x,y` and `release` steps run through the same
//! machine the widget hands its events to. Every decision is printed, because "the panel did not
//! move" and "the panel moved by the right amount" are indistinguishable in a screenshot of a
//! box.

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

    mod.gallery.pages.floating = View{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.0, y: 0.0}

        // The layer: the whole content area, and the surface the drag is heard on. Full size
        // on purpose — a layer the size of the panel would let the pointer outrun it.
        float_layer := mod.mp.MpFloatingLayer{
            float_body := View{
                width: Fill
                height: Fill
                flow: Down
                spacing: 20

                Label{
                    width: Fit, height: Fit
                    draw_text +: {text_style: title2, color: text}
                    text: "Floating Panel"
                }
                Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: footnote, color: text_muted}
                    text: "A panel that floats over a page and is dragged around it. The drag is heard on the layer rather than on the box, because a panel that listens on itself stalls the moment the pointer outruns a frame: the pointer lands outside a box-sized hitbox, the panel stops hearing moves, and the box is stranded behind the cursor. The grip is a separate strip along the top edge rather than the whole panel, so a panel can hold a control — the first click on a button inside it would otherwise move the panel instead."
                }

                Section{
                    Caption{ text: "The gesture decisions. Set GALLERY_FLOAT to a list of press:x,y;move:x,y;release steps separated by semicolons, the default presses, makes a sub-threshold move that must do nothing, drags properly, and drags back. The panel below is at wherever the script finished" }
                    float_log := Label{
                        width: Fill
                        height: Fit
                        draw_text +: {text_style: caption, color: text_faint}
                        text: "(log)"
                    }
                    float_state := Label{
                        width: Fill
                        height: Fit
                        draw_text +: {text_style: caption, color: text_faint}
                        text: "(state)"
                    }
                }
            }

            float_panel := mod.mp.MpFloating{
                floating_title := Label{text: "Inspector"}
                floating_content := mod.mp.Column{
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: footnote, color: text_muted}
                        text: "This box is placed at its own position and dragged there by the pointer, not laid out by its container. Drag the grip."
                    }
                    mod.mp.MpButtonSmall{text: "A button, which the grip makes clickable"}
                }
            }
        }
    }
}
