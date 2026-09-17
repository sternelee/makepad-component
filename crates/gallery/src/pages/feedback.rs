//! The two shapes that say "not yet": a determinate ring and a skeleton.
//!
//! The page has to be a *still* frame of two animations, which is exactly what
//! makes it worth looking at: the skeleton's sweep is at some phase when the
//! capture is taken, and the ring is the only one of the two whose value is not
//! moving. A skeleton frozen at the left edge and one mid-sweep look like two
//! different components, so the page shows several plates — their positions in
//! the cycle differ, and a sweep that never entered the plate would be obvious.

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

    let Row = View{
        width: Fill, height: Fit, flow: Right, spacing: 16, align: Align{y: 0.5}
    }

    mod.gallery.pages.feedback = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Feedback"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A spinner says wait; a skeleton says wait, and this is the shape of what is coming. That difference is why both exist rather than one, and the ring is the third answer for where a bar does not fit — a toolbar, an avatar, a title, where a horizontal bar needs a column's worth of width and a ring needs a square the height of the row. The ring draws no arc: it paints the whole circle and chooses per fragment whether it is the filled part or the track, from the fragment's own angle, so there is no seam where the arc starts."
        }

        Section{
            Caption{ text: "The ring at six values. 0 is a full track, 1 a full ring — and 1 has no notch at the top, which is why the shader special-cases it" }
            Row{
                ring_0 := mod.mp.MpProgressRing{}
                ring_25 := mod.mp.MpProgressRing{}
                ring_50 := mod.mp.MpProgressRing{}
                ring_75 := mod.mp.MpProgressRing{}
                ring_90 := mod.mp.MpProgressRing{}
                ring_100 := mod.mp.MpProgressRing{}
            }
        }

        Section{
            Caption{ text: "The ladder — 18, 28 and 44pt. The band is a fraction of the radius, so the ring keeps its proportions at every size" }
            Row{
                ring_small := mod.mp.MpProgressRingSmall{}
                ring_regular := mod.mp.MpProgressRing{}
                ring_large := mod.mp.MpProgressRingLarge{}
            }
        }

        Section{
            Caption{ text: "A ring in a row of content, which is where it belongs — beside a label rather than under one" }
            Row{
                ring_row := mod.mp.MpProgressRing{}
                Label{
                    width: Fit, height: Fit
                    draw_text +: {text_style: body, color: text}
                    text: "Indexing workspace"
                }
                mod.mp.MpBadgeSmall{ tone: mod.mp.StatusTone.Busy, text: "working" }
            }
        }

        Section{
            Caption{ text: "Named skeleton shapes — a skeleton's whole job is to be the shape of the content, so the shapes are named rather than sized at every call site" }
            View{
                width: Fill, height: Fit, flow: Down, spacing: 10
                mod.mp.SurfaceCard{
                    width: 420
                    flow: Down
                    spacing: 10
                    mod.mp.MpSkeletonTitle{}
                    mod.mp.MpSkeletonText{}
                    mod.mp.MpSkeletonText{}
                    mod.mp.MpSkeletonShort{}
                }
            }
        }

        Section{
            Caption{ text: "A card of skeletons beside its own avatar — the two shapes the file-list row and the detail pane both need" }
            Row{
                mod.mp.SurfaceCard{
                    width: 300
                    flow: Down
                    spacing: 10
                    View{
                        width: Fill, height: Fit, flow: Right, spacing: 10, align: Align{y: 0.0}
                        mod.mp.MpSkeletonSquare{}
                        View{
                            width: Fill, height: Fit, flow: Down, spacing: 8
                            mod.mp.MpSkeletonTitle{}
                            mod.mp.MpSkeletonText{}
                        }
                    }
                    mod.mp.MpSkeletonText{}
                    mod.mp.MpSkeletonShort{}
                }
                mod.mp.SurfaceCard{
                    width: 300
                    flow: Down
                    spacing: 10
                    mod.mp.MpSkeleton{ width: 120, height: 16 }
                    mod.mp.MpSkeletonText{}
                    mod.mp.MpSkeletonText{}
                    mod.mp.MpSkeletonText{}
                    mod.mp.MpSkeletonShort{}
                }
            }
        }
    }
}
