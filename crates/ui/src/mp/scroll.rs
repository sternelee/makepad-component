//! `MpScroll` — the scroll container, and the scroll bar the rest of the library
//! was silently not using.
//!
//! ## The gap this closes
//!
//! Every page in the gallery scrolls inside a bare Makepad `ScrollYView`, whose
//! handle is painted from **Makepad's own theme** — `theme.color_outset` and its
//! hover and drag siblings — not from this palette. So the one piece of chrome on
//! every page was the one piece that was not designed here, and it was invisible
//! because a scroll bar is chrome and nobody looks at chrome.
//!
//! That is the whole reason this module exists rather than a note telling callers
//! to write a scroll bar override: **a component library that does not theme the
//! parts of the framework it uses has a hole in it exactly where nobody looks.**
//!
//! ## What the numbers are
//!
//! A scroll bar is one of the few places the platform names nothing, so these are
//! chosen rather than measured, and chosen from the palette rather than invented:
//!
//! - **The handle is an ink, not a plate.** `text_faint` at rest, `text_muted` on
//!   hover, `text` while dragging — the three-rung ink ladder every other piece of
//!   chrome in the library uses, so a scroll bar recedes and comes forward the way
//!   a border does.
//! - **No border.** Makepad's default draws a bevel on the handle; a hairline
//!   around a 6pt bar is most of the bar.
//! - **A square-ish corner**, because a 6pt handle with a 3pt radius is a capsule
//!   and a capsule reads as a control rather than as a position.
//!
//! ## Two axes
//!
//! [`MpScroll`] is the vertical one every page uses. [`MpScrollBoth`] shows both
//! bars, for a canvas or a wide table — and it is a separate prototype rather than
//! a flag because a horizontal scroll bar on a page of prose is a bug, so the
//! default has to be the one that cannot do it.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*

    /// Vertical scrolling, with a handle painted from this palette.
    mod.mp.MpScroll = ScrollYView{
        scroll_bars +: {
            show_scroll_x: false
            show_scroll_y: true
            scroll_bar_y +: {
                // Makepad's own default is 10 with a 3pt side margin; this is a
                // wider *track* with a narrower *handle*, which is what lets the
                // handle be grabbed without the bar taking a column of layout.
                bar_size: 12.0
                bar_side_margin: 2.0
                // Longer than the default 30: a handle that shrinks to a stub
                // tells the reader nothing about where they are.
                min_handle_size: 36.0
                drag_scrolling: true
                draw_bg +: {
                    // The three-rung ink ladder. `text_faint` is faint enough to
                    // ignore and visible enough to find; the hover and drag rungs
                    // are the same steps a border takes.
                    color: text_faint
                    color_hover: text_muted
                    color_drag: text
                    // None of the bevel: a hairline is most of a 6pt bar.
                    border_size: 0.0
                    // The drawn thickness, as against the reserved track width.
                    size: 6.0
                    border_radius: 3.0
                }
            }
        }
    }

    /// Scrolling on both axes, for a canvas or a wide table.
    mod.mp.MpScrollBoth = ScrollXYView{
        scroll_bars +: {
            show_scroll_x: true
            show_scroll_y: true
            scroll_bar_x +: {
                bar_size: 12.0
                bar_side_margin: 2.0
                min_handle_size: 36.0
                drag_scrolling: true
                draw_bg +: {
                    color: text_faint
                    color_hover: text_muted
                    color_drag: text
                    border_size: 0.0
                    size: 6.0
                    border_radius: 3.0
                }
            }
            scroll_bar_y +: {
                bar_size: 12.0
                bar_side_margin: 2.0
                min_handle_size: 36.0
                drag_scrolling: true
                draw_bg +: {
                    color: text_faint
                    color_hover: text_muted
                    color_drag: text
                    border_size: 0.0
                    size: 6.0
                    border_radius: 3.0
                }
            }
        }
    }
}
