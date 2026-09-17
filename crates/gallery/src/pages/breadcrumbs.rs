//! `MpBreadcrumb` — where you are, and the way back.
//!
//! ## What the page shows, and the three rules in it
//!
//! - **The current crumb is the last one, derived rather than passed.** bezel takes a `current` flag per crumb, which allows
//!   two at once or none; a trail means the last one, so it is an invariant of the type.
//! - **A separator sits between crumbs, so the last has none.** A trailing chevron points at nothing — the same rule this
//!   library's description list follows with its hairline.
//! - **The current crumb keeps its width and the ancestors share what is left.** A trail is read from its end: the crumb you
//!   are on must be legible, and the ancestors are recognisable when elided. Under pressure every crumb takes its minimum and
//!   the trail overflows, the same honest failure the tab bar makes.
//!
//! ## And the rule a picture cannot show
//!
//! **The crumb you are on is not a link.** A click on it is a click on where the reader already is, and reporting it would
//! have a caller push a second copy of the page it is showing. The printed hit tests show a point on the current crumb
//! reporting nothing while points on the others report their index.

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

    mod.gallery.pages.breadcrumbs = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Breadcrumbs"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "Where you are, and the way back. The current crumb is the last one and is derived rather than passed, because a trail means the last one: a flag per crumb would allow two to be current at once, or none. A chevron sits between crumbs and not after the last, since a trailing chevron points at nothing — the same rule this library's description list follows with its hairline. The current crumb keeps the width it asked for and the ancestors share what is left, because a trail is read from its end: the crumb you are on has to be legible, and the ones behind it are recognisable when elided. And the crumb you are on is not a link: a click on it is a click on where you already are, so it reports nothing."
        }

        Section{
            Caption{ text: "A trail: the last crumb is current and does not look clickable" }
            crumbs_trail := mod.mp.MpBreadcrumb{width: Fill}
        }
        Section{
            Caption{ text: "Too many crumbs for the width: the ancestors take the 24-point minimum and the trail overflows" }
            crumbs_pressure := mod.mp.MpBreadcrumb{width: 320}
        }
        Section{
            Caption{ text: "One crumb: it is current, so it has no chevron and nothing to navigate to" }
            crumbs_single := mod.mp.MpBreadcrumb{width: Fill}
        }
    }
}
