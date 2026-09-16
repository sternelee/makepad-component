//! `MpSurface` — the container every other container is built on.
//!
//! Two things decide how a surface looks, and they are separate on purpose:
//!
//! - **Which plane** it is: the page, a panel, a card, a dialog, a popover, a
//!   recessed strip. Each is its own prototype, because that is how a Makepad
//!   caller names a look — `mod.mp.SurfaceCard{}` reads better than
//!   `role: SurfaceRole.Card`, and it is the shape the rest of the framework
//!   already uses (`mod.widgets.RoundedView`, `mod.widgets.PanelView`).
//! - **How far off the page** it sits, which for a glass surface is the whole
//!   reason it is one.
//!
//! No Rust here at all. A surface is a *style*, and in Makepad a style lives in
//! the DSL where the theme namespaces are — there is no per-frame state to
//! mix, so a widget wrapper would only be a place to lose the theme. What it
//! costs is that an appearance change has to re-apply scripts to reach these
//! values; [`Theme::install`](makepad_theme::Theme::install) does exactly that,
//! which is how Makepad's own `mod.theme` switch works.
//!
//! ## The numbers
//!
//! Corner radii are ratios of one base ([`Theme::panel_radius`] and friends),
//! so branding the radius moves every surface together. The glass numbers come
//! from [`Theme::frost`](makepad_theme::Theme::frost) — the measured SwiftUI
//! scale — rather than from a literal here, so `Material::Thin` means the same
//! thing on a popover as it does in the theme.
//!
//! [`Theme::panel_radius`]: makepad_theme::Theme::panel_radius

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc.tokens.*
    use mod.mpc.layout.*

    // The namespace has to exist before a nested assignment reaches it — the
    // same reason the theme creates `mod.mpc` before `mod.mpc.tokens`. A page
    // that assigns into a missing module fails at runtime, not at compile time.
    mod.mp = {}

    // ---- the opaque surface ----
    //
    // A `RoundedView` with the theme's corner and hairline, and a fill the
    // caller names by picking a prototype below.
    mod.mp.Surface = mod.widgets.RoundedView {
        width: Fill
        height: Fit
        flow: Down
        spacing: space

        draw_bg +: {
            color: instance(surface)
            border_radius: instance(12.0)
            // A hairline in `border`, not `border_strong`: a surface edge
            // separates planes, it does not have to hold against a bright
            // surround the way a focused field does.
            border_size: instance(1.0)
            border_color: instance(border)
        }
    }

    /// The page itself. No edge, because there is nothing behind it to be
    /// separated from.
    mod.mp.SurfacePage = mod.mp.Surface {
        draw_bg +: {
            color: instance(bg)
            border_size: instance(0.0)
            border_radius: instance(0.0)
        }
    }

    /// The shell plane a card sits on — the sidebar, the rail.
    mod.mp.SurfacePanel = mod.mp.Surface {
        draw_bg +: {
            color: instance(surface)
            border_size: instance(0.0)
            border_radius: instance(0.0)
        }
    }

    /// The content card. The everyday surface, and the one a caller reaches for
    /// without thinking.
    mod.mp.SurfaceCard = mod.mp.Surface {
        padding: Inset{left: 16, right: 16, top: 16, bottom: 16}
        draw_bg +: {
            color: instance(surface_card)
            border_radius: instance(12.0)
            border_size: instance(1.0)
            border_color: instance(border)
        }
    }

    /// An opaque plate: a user bubble, a jump-to-bottom pill. Sits directly on
    /// the page with nothing to lift it, so it carries its own tone.
    mod.mp.SurfaceRaised = mod.mp.Surface {
        draw_bg +: {
            color: instance(surface_raised)
            border_radius: instance(10.0)
            border_size: instance(0.0)
        }
    }

    /// A dialog's plane. Above a card, below a popover.
    mod.mp.SurfaceDialog = mod.mp.Surface {
        padding: Inset{left: 20, right: 20, top: 20, bottom: 20}
        draw_bg +: {
            color: instance(surface_dialog)
            border_radius: instance(12.0)
            border_size: instance(1.0)
            border_color: instance(border_strong)
        }
    }

    /// A popover, menu or command palette — the topmost opaque plane.
    mod.mp.SurfaceOverlay = mod.mp.Surface {
        padding: Inset{left: 12, right: 12, top: 12, bottom: 12}
        draw_bg +: {
            color: instance(surface_overlay)
            border_radius: instance(12.0)
            border_size: instance(1.0)
            border_color: instance(border_strong)
        }
    }

    /// A recessed strip: a picker header, a footer bar, a code block. It reads
    /// as *below* the page rather than above it, which is why it is a band
    /// rather than a rung on the surface ladder — the ladder only goes up.
    mod.mp.SurfaceSunken = mod.mp.Surface {
        draw_bg +: {
            color: instance(surface)
            border_radius: instance(6.0)
            border_size: instance(1.0)
            border_color: instance(border)
        }
    }

    // ---- the elevated surface ----
    //
    // Makepad's own Gaussian-backed glass: a real mip-chain blur of whatever
    // is behind it, plus a lit rim, an inner shadow and a drop shadow. This is
    // the direct analogue of bezel's `SurfaceSpec`, and the numbers below are
    // bezel's, read from the theme rather than written here.
    //
    // `fallback_color` is what it paints when the compositor cannot give it a
    // backdrop — an opaque plane one rung above the surface it covers, so a
    // popover over an unsupported window still reads as a popover.
    mod.mp.SurfaceGlass = mod.widgets.GaussRoundedView {
        width: Fill
        height: Fit
        flow: Down
        clip_x: false
        clip_y: false
        padding: Inset{left: 12, right: 12, top: 12, bottom: 12}

        draw_bg +: {
            corner_radius: instance(12.0)
            tint_color: instance(#xffffff)
            // `Clear` measures at 16/255 of coverage; `Regular` at 11/255 over a
            // much harder compression. `Regular` is the everyday one.
            tint_alpha: uniform(0.043)
            border_color: instance(#xffffff)
            border_alpha: instance(0.10)
            border_width: instance(1.0)
            rim_alpha: instance(0.10)
            rim_width: uniform(1.0)
            // A material has nothing to bend, so it needs no lens.
            lensing_effect: uniform(0.0)
            inner_shadow_alpha: instance(0.0)
            shadow_color: instance(#x00000040)
            // Bezel collapses two shadow layers into one Gaussian because
            // Makepad paints one; these are that collapse, at the offset that
            // carries the perceived lift.
            shadow_radius: uniform(14.0)
            shadow_sigma: uniform(5.0)
            shadow_offset: uniform(vec2(0.0, 6.0))
            fallback_color: instance(surface_overlay)
        }
    }

    /// The glass a popover, menu or command palette paints.
    mod.mp.SurfaceGlassPopover = mod.mp.SurfaceGlass {
        draw_bg +: {
            fallback_color: instance(surface_overlay)
        }
    }
}
