//! The surface planes: the ladder every container is built on, and the glass.
//!
//! This page exists because the surfaces were used by every *other* page and
//! documented by none — and, more concretely, because `mp.SurfaceGlass` had never
//! been rendered since it was written. It is the one surface in the library built
//! on Makepad's real backdrop blur, and a surface that has never drawn is a
//! surface whose numbers are unverified.

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
        width: Fill, height: Fit, flow: Right, spacing: 12, align: Align{y: 0.0}
    }

    // A tile of nameable size, so the planes can sit side by side and be compared
    // rather than described.
    let Tile = View{
        width: Fit, height: Fit, flow: Down, spacing: 4
        tile_caption := Caption{ text: "" }
    }

    mod.gallery.pages.surface = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 24

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Surface"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "The planes every container is built on. Each is a prototype over a rounded box, and a caller names one rather than a colour — so a card on a shell on a page reads as three levels without anyone choosing a number. The corner radii are ratios of one base, which is what makes branding the radius move all of them together."
        }

        Section{
            Caption{ text: "The opaque ladder — page, panel, card, raised, sunken" }
            Row{
                Tile{
                    tile_caption +: {text: "MpSurfacePage"}
                    mod.mp.SurfacePage{ width: 130, height: 64 }
                }
                Tile{
                    tile_caption +: {text: "SurfacePanel"}
                    mod.mp.SurfacePanel{ width: 130, height: 64 }
                }
                Tile{
                    tile_caption +: {text: "SurfaceCard"}
                    mod.mp.SurfaceCard{ width: 130, height: 64 }
                }
                Tile{
                    tile_caption +: {text: "MpSurfaceRaised"}
                    mod.mp.SurfaceRaised{ width: 130, height: 64 }
                }
                Tile{
                    tile_caption +: {text: "SurfaceSunken"}
                    mod.mp.SurfaceSunken{ width: 130, height: 64 }
                }
            }
        }

        Section{
            Caption{ text: "The floating pair — a dialog's plane and a popover's, each with a stronger edge than a card because they sit over content rather than in it" }
            Row{
                Tile{
                    tile_caption +: {text: "MpSurfaceDialog"}
                    mod.mp.SurfaceDialog{ width: 170, height: 72 }
                }
                Tile{
                    tile_caption +: {text: "MpSurfaceOverlay"}
                    mod.mp.SurfaceOverlay{ width: 170, height: 72 }
                }
            }
        }

        Section{
            Caption{ text: "The radii are one base times a ratio: 6 control, 8 button, 10 panel, 12 surface, 16 bubble. Branding the radius moves all five" }
            Row{
                mod.mp.SurfaceRaised{ width: 74, height: 48, draw_bg +: {border_radius: instance(6.0)} }
                mod.mp.SurfaceRaised{ width: 74, height: 48, draw_bg +: {border_radius: instance(8.0)} }
                mod.mp.SurfaceRaised{ width: 74, height: 48, draw_bg +: {border_radius: instance(10.0)} }
                mod.mp.SurfaceRaised{ width: 74, height: 48, draw_bg +: {border_radius: instance(12.0)} }
                mod.mp.SurfaceRaised{ width: 74, height: 48, draw_bg +: {border_radius: instance(16.0)} }
            }
        }

        // The glass, over something opaque so the blur has a backdrop to bend.
        // A glass surface over a flat page shows almost nothing; over content it
        // shows what it is for.
        Section{
            Caption{ text: "Glass over maximum-contrast stripes. The backdrop has to be high-contrast for this to prove anything: over flat or near-toned plates a blur is invisible by construction, and a tinted rectangle would look identical. Sharp black and white under the plate, soft grey through it, is the only reading that distinguishes a real backdrop blur from a fill." }
            View{
                width: Fill, height: 132, flow: Overlay, align: Align{x: 0.0, y: 0.0}
                // The backdrop: **maximum-contrast stripes**, because that is the
                // only backdrop on which a blur can be told from a fill. The
                // first version used five plates from the surface ladder, which
                // are within a few levels of each other — the glass looked the
                // same whether it blurred or not, so the page proved nothing.
                View{
                    width: Fill, height: Fill, flow: Right, spacing: 0
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(text), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(bg), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(text), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(bg), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(text), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(bg), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(text), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(bg), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(text), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: 46, height: Fill, draw_bg +: {color: instance(bg), border_radius: instance(0.0)} }
                    mod.mp.Surface{ width: Fill, height: Fill, draw_bg +: {color: instance(text), border_radius: instance(0.0)} }
                }
                glass_tile := mod.mp.SurfaceGlass{
                    width: 260
                    height: 96
                    padding: Inset{left: 16, right: 16, top: 16, bottom: 16}
                    Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: body, color: text}
                        text: "Sharp stripes, or soft grey?"
                    }
                }
            }
        }
    }
}
