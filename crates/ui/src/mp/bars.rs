//! The chrome bars — `MpTitlebar`, `MpControlBar`, `MpMenubar`.
//!
//! ## Why they are one module
//!
//! All three are horizontal strips that hold other things, and a strip's only real
//! decisions are **how tall it is, how much air it has at the sides, and where its
//! content sits vertically.** The three differ in what they carry, not in how they
//! are built, so they are three prototypes here rather than three modules — the
//! rule this crate has been using since `mp/control.rs`: two widgets needing the
//! same arithmetic is one module, not two copies.
//!
//! ## The numbers are the theme's, and they already existed
//!
//! A titlebar is one of the few pieces of chrome with a platform-defined shape, and
//! this port's layout tokens already carried the three numbers it needs —
//! `layout.titlebar_height`, `layout.titlebar_top_pad` and
//! `layout.traffic_light_inset` — from the bezel reference, with the source noted
//! where they are defined. So nothing here is chosen: the titlebar reads them, and
//! the two bars that have no platform height take **`Fit`**, which is to say their
//! height is their content's rather than a number invented for them.
//!
//! That last point is the reason `MpControlBar` and `MpMenubar` do not have a
//! height constant. A control bar is 32pt in one app and 44pt in another, and a
//! library that picks one is a library that will be overridden. A bar that sizes to
//! what it holds is right at both.
//!
//! ## The inset is a *spacer*, not padding
//!
//! macOS draws its window buttons inside the titlebar's own rectangle, so content
//! has to step around them. `layout.traffic_light_inset` is a leading spacer rather
//! than `padding.left`, because padding would move the bar's own background too —
//! and the background is supposed to run under the buttons, behind them.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// A window title bar: an inset for the platform's window buttons, a title, and
    /// a trailing place for actions.
    ///
    /// The title is **left-aligned after the inset** rather than centred. A true
    /// optical centre needs equal-width columns on both sides, and the trailing
    /// slot is whatever the caller puts in it — so a centring that worked for one
    /// app's action set would drift in another's. Left after the inset is what a
    /// document window does and it is stable under any trailing content.
    mod.mp.MpTitlebar = View{
        width: Fill
        height: mod.mpc.layout.titlebar_height
        flow: Right
        spacing: 8
        // Vertical centring is the row's; `titlebar_top_pad` is the bar's own
        // allowance for the platform's taller button row, applied as padding so it
        // moves the bar rather than the text inside it.
        align: Align{x: 0.0, y: 0.5}
        padding: Inset{top: mod.mpc.layout.titlebar_top_pad, bottom: 0.0}

        draw_bg +: {
            color: surface
            border_color: divider
            border_size: 1.0
        }

        // The platform's window buttons live *inside* this rectangle, so content
        // steps around them. A spacer rather than padding, because padding would
        // move the background too.
        titlebar_inset := View{
            width: mod.mpc.layout.traffic_light_inset
            height: Fill
        }
        titlebar_title := Label{
            width: Fit
            height: Fit
            draw_text +: {text_style: caption, color: text_muted}
            text: "Window"
        }
        // Pushes the trailing slot to the far edge, so a caller can drop actions in
        // without also having to write a spacer.
        titlebar_spacer := View{
            width: Fill
            height: Fill
        }
        titlebar_actions := mod.mp.Row{
            width: Fit
            height: Fit
            spacing: 4
        }
    }

    /// A strip of controls below a titlebar — view modes, filters, a search field.
    ///
    /// `Fit` height on purpose: this bar has no platform height, and its children
    /// are controls that already know their own. The side air is the layout's
    /// `space`, so it lines up with every other inset in the library.
    mod.mp.MpControlBar = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8
        align: Align{x: 0.0, y: 0.5}
        padding: Inset{left: mod.mpc.layout.space, right: mod.mpc.layout.space, top: 6.0, bottom: 6.0}

        draw_bg +: {
            color: bg
            border_color: divider
            border_size: 1.0
        }

        controlbar_leading := mod.mp.Row{
            // **`Fit`, and there must be only one `Fill` among a bar's children.**
            //
            // A segmented control painted two of its three labels, and the cause was structural rather than in
            // the control: this group and the spacer below were **both** `Fill`, so they **split** the bar's
            // remaining space — the group got half, and a child drawing wider than its cell is clipped to it.
            // Which it is: `draw_abs` from inside a widget is clipped to the parent's cell as well as `begin`,
            // so nothing the child draws can escape it.
            //
            // `Fit` and a declared width is the combination that works: `mp/segmented.rs` sets its own
            // `Fixed` walk in `set_segments`, which runs between frames, so the parent's next layout pass reads a
            // number instead of a `Fit` it cannot resolve.
            width: Fit
            height: Fit
            spacing: 4
        }
        // The **only** `Fill` in the bar. A second one splits the leftover space with the leading group.
        controlbar_spacer := View{
            width: Fill
            height: Fill
        }
        controlbar_trailing := mod.mp.Row{
            width: Fit
            height: Fit
            spacing: 4
        }
    }

    /// A horizontal row of menu triggers.
    ///
    /// The triggers are `MpButton` ghosts rather than labels, which is a decision
    /// worth stating: a menubar trigger is *clickable* and *focusable*, so it should
    /// get the press, focus and hover behaviour every other control in the library
    /// already has, instead of a second implementation of the same three states.
    mod.mp.MpMenubar = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 2
        align: Align{x: 0.0, y: 0.5}
        padding: Inset{left: 6.0, right: 6.0, top: 3.0, bottom: 3.0}

        draw_bg +: {
            color: surface
            border_color: divider
            border_size: 1.0
        }

        menubar_items := mod.mp.Row{
            width: Fit
            height: Fit
            spacing: 2
        }
    }
}
