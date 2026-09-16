//! The tree: a flat list that knows its hierarchy, and the disclosure that
//! follows from it.
//!
//! The page shows three states side by side because they are the three things
//! worth checking: nothing collapsed (the whole shape visible), one subtree
//! collapsed (its run hidden, its siblings untouched), and a deep list where the
//! indent has to be right or a child's label sits under its parent's chevron.

use makepad_component::mp::tree::TreeItem;
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

    mod.gallery.pages.tree = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Tree"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Stored as a flat list where each item carries its depth, with the collapsed set owned by the widget. Every operation a tree supports is a list operation on a flat list — expand, hide a run of deeper items, step to the next visible row — and each of those is a restructure on a nested one. `has_children` is derived from the next item's depth rather than stored, and one function answers which rows are on screen, so the painter, the hover, the click and the arrow keys cannot disagree."
        }

        Section{
            Caption{ text: "Fully expanded — every level, with the chevron only where there is something to disclose" }
            full_tree := mod.mp.MpTree{}
        }

        Section{
            Caption{ text: "One subtree collapsed — its run is hidden and its siblings are untouched, which is the fault a naive walk has" }
            collapsed_tree := mod.mp.MpTree{}
        }

        Section{
            Caption{ text: "Deep — five levels, and the label of each sits past its own chevron at every depth" }
            deep_tree := mod.mp.MpTree{}
        }

        Section{
            Caption{ text: "Long names — a label wider than its row is cut rather than running past the plate" }
            long_tree := mod.mp.MpTree{}
        }
    }
}
