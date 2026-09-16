//! Layout primitives: the sibling gap, and the line between things.
//!
//! No Rust. These are styles — a row and a column differ by one DSL field — and
//! in Makepad a style belongs where the theme namespaces are.
//!
//! ## Why a stack exists at all
//!
//! Bezel's `ui::stack::row()` is the system gap, and the reason it is a function
//! rather than a habit is that a call site which wants *the* gap then writes no
//! number at all. `Row{}` is 8pt because
//! [`NSStackView().spacing`](makepad_theme::Theme::SPACE) is 8; a caller who
//! wants 12 writes `spacing: 12`, which is a deviation the way
//! `VStack(spacing: 12)` is one. Without the prototype every call site writes a
//! number, and the numbers drift apart because nothing names the default.
//!
//! `Row` aligns on the centre line by default. A row of controls of differing
//! heights that aligns on its *tops* looks like a mistake, and a stack that
//! leaves `align` unset inherits whatever the parent had.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.layout.*

    /// Children left to right, on the system gap, centred across each other.
    mod.mp.Row = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: space
        align: Align{x: 0.0, y: 0.5}
    }

    /// The same row that takes the whole width of whatever it is in, rather
    /// than hugging its content.
    mod.mp.RowFill = mod.mp.Row{
        height: Fill
    }

    /// Children top to bottom, on the system gap.
    mod.mp.Column = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: space
        align: Align{x: 0.0, y: 0.0}
    }

    /// A column that fills its parent's height — a page body, a sidebar.
    mod.mp.ColumnFill = mod.mp.Column{
        height: Fill
    }

    /// The gap itself, for the rare place a caller needs to push two things
    /// apart without a container.
    mod.mp.Spacer = View{
        width: Fill
        height: Fill
    }

    // ---- the line between things ----
    //
    // A divider is a hairline in `divider`, which is the same tone as `border`:
    // the palette does not distinguish "the edge of a thing" from "the seam
    // inside it", and neither should the DSL. What differs is the geometry, so
    // that is what the two prototypes name.
    //
    // Not a widget: there is no state, no hit area and no action, and a widget
    // that has none of those is a `View` with a colour.
    mod.mp.Divider = mod.widgets.View{
        width: Fill
        height: 1
        show_bg: true
        draw_bg +: {
            color: instance(divider)
        }
    }

    /// A divider between two things side by side. `Fill` height so it reaches
    /// whatever it is separating rather than being a dash in the middle of it.
    mod.mp.DividerVertical = mod.widgets.View{
        width: 1
        height: Fill
        show_bg: true
        draw_bg +: {
            color: instance(divider)
        }
    }

    /// A divider with a label in the middle of it — "or", "more", a section
    /// break. The label sits on the surface's own background so the line reads
    /// as passing behind it.
    mod.mp.DividerLabelled = mod.mp.Row{
        width: Fill
        height: Fit
        spacing: 10

        left_line := mod.mp.Divider{}
        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: mod.mpc.type.caption
                color: text_faint
            }
            text: "or"
        }
        right_line := mod.mp.Divider{}
    }
}
