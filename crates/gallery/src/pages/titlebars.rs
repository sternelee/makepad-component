//! `MpTitlebar` — the bar you drag a chromeless window by.
//!
//! ## What the page shows
//!
//! Three bars, and the difference between them is the one rule that lives in the component's geometry: **the drag region is
//! the bar minus its trailing controls.** The first bar has no controls and is draggable edge to edge; the second reserves
//! room on the right, so a press there is the control's rather than the window's; the third reserves more than it is wide,
//! which is a degenerate layout and leaves nothing draggable at all rather than a region that runs backwards.
//!
//! ## What it cannot show, and is honest about
//!
//! **The drag itself.** A widget cannot move a window — `Window::reposition` needs a `&Window`, which `handle_event` does not
//! get — so the bar reports `DragBy(delta)` and the application applies it. That also means the synthetic pointer cannot
//! exercise this page here, so the evidence is the printed arithmetic: the same `drag_region`/`is_draggable` the widget calls,
//! plus a scripted `DragState` walk showing that each report is measured from the last one and that a `NaN` movement is
//! dropped rather than clamped.
//!
//! A `NaN` delta handed to `Window::reposition` puts the window somewhere unreachable, and a backend that fits the request to
//! the attached displays brings it back somewhere the user did not ask for. Dropping it keeps the window where it is.

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

    mod.gallery.pages.titlebars = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Title Bars"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "The bar you drag a chromeless window by. It cannot move a window itself — Window::reposition needs a &Window, which handle_event does not get — so it reports the pointer's movement and the application, which owns the window, applies it. The one rule in its geometry is that the drag region is the bar minus whatever its trailing controls reserve, because a press meant for a button must not move the window instead. A double click reports ToggleMaximize, the convention on every platform that has a title bar. The height is 28 points, macOS's own, rather than the theme's 32-point row: a custom bar at the row height would make every window's chrome four points taller than the platform's."
        }

        Section{
            Caption{ text: "No controls — draggable from its leading edge to its trailing one" }
            titlebar_plain := mod.mp.MpTitlebar{width: Fill}
        }
        Section{
            Caption{ text: "80 points reserved for controls on the right, which are not draggable" }
            titlebar_controls := mod.mp.MpTitlebar{width: Fill}
        }
        Section{
            Caption{ text: "Widths of 320 points, to show two bars of different length side by side" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.5}
                titlebar_narrow := mod.mp.MpTitlebar{width: 320}
                titlebar_short := mod.mp.MpTitlebar{width: 180}
            }
        }
    }
}
