//! `MpCollapsibleHeader` — a section's header row, and the chevron that says which way it goes.
//!
//! ## What the page shows, and the rule it is really showing
//!
//! Two sections: one open with its body drawn, one collapsed with its body **hidden**. That second one is the point —
//! **collapsed means hidden, not clipped.** A body at zero height is still laid out and still receiving events, so a scroll
//! view inside a collapsed section would scroll and a button inside it could be clicked through the header. The page hides
//! the collapsed body rather than shrinking it, which is what the module doc tells a caller to do.
//!
//! ## Why this is a header and not a container
//!
//! bezel's reason, kept: *a container that swallowed its children would have to re-implement layout for them.* That is
//! exactly what `MpSplitPane` and `MpTabBar` **do** do — and they are right to, because a split must set its panes' widths
//! and a tab bar must place its tabs, neither of which a flow can express. A collapsible needs none of that: the body is
//! simply there or not, laid out by whatever was going to lay it out anyway. So this reports `Toggled` and the caller owns
//! `expanded` — which is also why the action carries no payload: the header does not know the new state.

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
    let Body = View{
        width: Fill, height: Fit, flow: Down, spacing: 4
        padding: Inset{left: 24, right: 8, top: 2, bottom: 8}
    }

    mod.gallery.pages.collapsible = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Collapsible"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A section header and the chevron that says which way it goes. It is a header rather than a container, deliberately: a container that swallowed its children would have to re-implement layout for them, and a collapsible needs nothing of the sort — the body is simply there or not, laid out by whatever was going to lay it out anyway. This is the opposite of the split pane and the tab bar, which do take their children because a split must set its panes' widths and a tab bar must place its tabs, and no flow can express either. So the header reports a toggle and the caller owns the state, which is why the action carries no payload: the header does not know the new state. The chevron points down when expanded and right when collapsed — the direction the section moved. And a collapsed section's body must be hidden rather than clipped, because a body at zero height is still laid out and still receiving events."
        }

        Section{
            Caption{ text: "Open: the chevron points down and the body is drawn" }
            head_open := mod.mp.MpCollapsibleHeader{title: "Columns"}
            body_open := Body{
                Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: body, color: text}
                    text: "id · integer · primary key"
                }
                Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: body, color: text_muted}
                    text: "name · text · not null"
                }
            }
        }
        Section{
            Caption{ text: "Collapsed: the chevron points right, and the body is hidden rather than clipped" }
            head_closed := mod.mp.MpCollapsibleHeader{title: "Indexes"}
            body_closed := Body{
                visible: false
                Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: body, color: text}
                    text: "idx_name · on name"
                }
                Label{
                    width: Fill, height: Fit
                    draw_text +: {text_style: body, color: text_muted}
                    text: "This is hidden, not shrunk: a zero-height body would still take events."
                }
            }
        }
    }
}
