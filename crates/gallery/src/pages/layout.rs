//! The layout primitives: the sibling gap, and the line between things.

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

    // A tile, so the gap between tiles is what the eye measures.
    let Tile = mod.mp.SurfaceRaised{
        width: 72, height: 40
        align: Align{x: 0.5, y: 0.5}
        label := Label{
            width: Fit, height: Fit
            draw_text +: {text_style: caption, color: text_muted}
            text: "tile"
        }
    }

    mod.gallery.pages.layout = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Layout"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Row and Column carry the system gap, which is 8pt because NSStackView().spacing, the visual format's '-', and constraint(equalToSystemSpacingAfter:) all report it. A call site that wants the standard gap writes no number at all — and a call site that writes `spacing: 20` is a deviation the way VStack(spacing: 20) is one."
        }

        Section{
            Caption{ text: "Row — 8pt, centred across each other" }
            mod.mp.Row{
                Tile{}
                Tile{}
                Tile{}
                Tile{}
            }
        }

        Section{
            Caption{ text: "Column — the same 8pt, top to bottom" }
            Column{
                width: Fill, height: Fit, flow: Down, spacing: 8
                Tile{ width: Fill, height: 32 }
                Tile{ width: Fill, height: 32 }
                Tile{ width: Fill, height: 32 }
            }
        }

        Section{
            Caption{ text: "A deviation is written, not implied — spacing: 20" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 20
                Tile{}
                Tile{}
                Tile{}
            }
        }

        Section{
            Caption{ text: "Dividers — one tone, two geometries" }
            Column{
                width: Fill, height: Fit, flow: Down, spacing: 12
                mod.mp.Divider{}
                mod.mp.DividerLabelled{}
                View{
                    width: Fill, height: 56, flow: Right, spacing: 12
                    align: Align{y: 0.5}
                    Tile{}
                    mod.mp.DividerVertical{}
                    Tile{}
                    mod.mp.DividerVertical{}
                    Tile{}
                }
            }
        }

        Section{
            Caption{ text: "Spacer — pushes two things apart without a container" }
            mod.mp.Row{
                Tile{}
                mod.mp.Spacer{}
                Tile{}
            }
        }
    }
}
