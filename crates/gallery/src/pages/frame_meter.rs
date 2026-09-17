//! `MpStats` — the meter: what this window is drawing, and what the process costs.
//!
//! ## The rule, and why this page can prove it
//!
//! **A meter that drives frames cannot measure frames.** Refreshing its own digits means asking for the next frame, and if
//! the count included those frames the meter would be measuring itself — a window at rest would report the refresh rate,
//! which is the most confidently wrong a number can be.
//!
//! makepad makes the distinction available: `NextFrameEvent` carries the window's `frame` number **and** `set`, the handles
//! that asked for that frame. So the count comes from the platform and the meter can recognise its own asking. That is the
//! same information gpui's `Painter::woken` gives bezel's meter.
//!
//! ## What this page can show that a picture cannot
//!
//! The two figures themselves. Run the gallery with `MP_STATS_DEBUG=1` and every reading prints with the span it covered and
//! the frame number it came from — so a meter that had counted its own ticks would be visible as a plausible-looking rate on
//! a window that is doing nothing. **A screenshot of a meter proves nothing at all**: `60 fps` and `0 fps` look identical in
//! their confidence.
//!
//! ## The reduction, said rather than glossed
//!
//! bezel's meter shows four numbers; this one shows two. Memory is real
//! (`memory_watchdog::process_footprint_bytes`). **CPU and GPU are absent because makepad has no measurement point for
//! them** — `Instant` measures wall time, not CPU time, and reporting wall time under the label "CPU" would be a number that
//! looks like the real one and is not. A port that printed all four labels with two zeros would be claiming a parity it does
//! not have.

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

    mod.gallery.pages.frame_meter = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Frame Meter"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "What this window is drawing and what the process costs. The rule it exists for is that a meter which drives frames cannot measure them: refreshing its own digits means asking for the next frame, and a count that included those would report the meter's rate rather than the application's — a window at rest reading 60. makepad distinguishes the two, and the exclusion is why the number means anything. A reading is held for a whole second because a rate recomputed per frame flickers, and a number that flickers is a number nobody reads. Memory is a real process footprint; CPU and GPU are absent rather than zero, because makepad has no measurement point for either and reporting wall time under the label CPU would be a number that looks like the real one and is not."
        }

        Section{
            Caption{ text: "The meter — it ticks, so the reading below changes as you watch" }
            stats_meter := mod.mp.MpStats{}
        }
        Section{
            Caption{ text: "Not ticking: a plate a caller wants without the cost of a frame request" }
            stats_still := mod.mp.MpStats{ticking: false}
        }
    }
}
