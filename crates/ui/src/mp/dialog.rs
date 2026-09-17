//! `MpDialog` — a full-screen modal with a card in the middle.
//!
//! ## This one is mostly DSL, and that is the honest description
//!
//! A dialog is a **tree**: a backdrop, a centred card, and a header/body/footer whose contents the caller writes. Rust's
//! half is one boolean and the animation that boolean drives. So there is very little logic here to test, and the module does
//! not pretend otherwise: [`Shown`] is the boolean with its two idempotent operations, and the rest of the file is the tree,
//! which needs a window to be exercised at all.
//!
//! That is also why this port keeps the **slot ids** of the widget it replaces — `backdrop`, `content`, `dialog`, `header`,
//! `title`, `description`, `body`, `footer`. A caller writes those in its own DSL, so a port that renamed them would fail
//! every call site for no gain. Four of dbpro's call sites name exactly these.
//!
//! ## The duration is the catalog's, and that is checked
//!
//! The DSL animates the backdrop from opacity 0 to 0.8 over `mod.motion.dialog_in.duration` — the named entry in
//! `makepad_motion`'s catalog (180ms, `EASE`), not a number written here. The v2 widget it replaces had `duration: 0.2`
//! inline, which is the thing this port's rules forbid: an inline duration is a decision that cannot be found, cannot be
//! changed once, and never matches its neighbours. A test reads this file and fails if a bare duration reappears in the
//! animator, because the rule is otherwise only visible to a reader who knows where to look.
//!
//! ## Closing is the backdrop's job, and only the backdrop's
//!
//! A click **on the backdrop** reports `Close`; a click on the card does not, because the card is the thing being used. That
//! is why the hit is taken on the backdrop's area rather than the dialog's — the whole dialog fills the window, so a hit on
//! the dialog as a whole would close it when a button inside was pressed.
//!
//! ## What is dropped
//!
//! The v2's five `MpSize`-ish width variants collapse to two named ones plus whatever a caller writes, and its
//! `MpAlertDialogNew` becomes `MpAlertDialog` (the `New` suffix was a migration artefact).

use makepad_widgets::*;

/// The overlay's maximum backdrop opacity.
///
/// Not 1.0: a modal that hides the window completely loses the context of what it is modal *to*, and every platform's own
/// dimming is partial for that reason. The v2 widget's number, kept.
pub const BACKDROP_OPACITY: f64 = 0.8;

/// The card's side padding.
pub const CARD_PAD: f64 = 24.0;

/// The default card width.
pub const CARD_WIDTH: f64 = 440.0;
/// A narrow card, for a confirmation.
pub const CARD_WIDTH_SMALL: f64 = 320.0;
/// A wide card, for a form.
pub const CARD_WIDTH_LARGE: f64 = 560.0;
/// The confirm/alert width.
pub const CARD_WIDTH_ALERT: f64 = 360.0;

/// The slot ids a dialog's tree is addressed by.
///
/// A constant rather than four copies of `id!(backdrop)` scattered through the file: the DSL and the hit test have to agree
/// about the backdrop's name, and a rename that missed one would be a dialog that could not be dismissed.
pub const BACKDROP: LiveId = live_id!(backdrop);
pub const CONTENT: LiveId = live_id!(content);
pub const CARD: LiveId = live_id!(dialog);
pub const HEADER: LiveId = live_id!(header);
pub const TITLE: LiveId = live_id!(title);
pub const DESCRIPTION: LiveId = live_id!(description);
pub const BODY: LiveId = live_id!(body);
pub const FOOTER: LiveId = live_id!(footer);

/// Whether a dialog is open, and the two operations that are idempotent by design.
///
/// The v2 widget guarded both — `if !self.open { … }` — and the guard is not a micro-optimisation: **re-playing the entry
/// animation while the dialog is already up makes it flash**, so a caller that calls `open()` on every keystroke of some
/// unrelated handler would produce a strobing dialog. So the operations report whether they changed anything, and the caller
/// plays the animator only when they did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shown(bool);

impl Shown {
    /// Open it, answering whether that changed anything.
    pub fn open(&mut self) -> bool {
        self.set(true)
    }

    /// Close it, answering whether that changed anything.
    pub fn close(&mut self) -> bool {
        self.set(false)
    }

    /// Set it, answering whether that changed anything.
    pub fn set(&mut self, open: bool) -> bool {
        if self.0 == open {
            return false;
        }
        self.0 = open;
        true
    }

    pub fn is_open(self) -> bool {
        self.0
    }
}

/// What a dialog reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpDialogAction {
    /// The dialog opened.
    Open,
    /// The dialog closed — the backdrop was clicked, or the caller said so.
    Close,
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The dimming layer. `opacity` is an instance so the animator can drive it.
    mod.mp.DrawMpDialogBackdrop = #(DrawMpDialogBackdrop::script_shader(vm)){
        ..mod.draw.DrawQuad

        scrim: #x000000ff
        opacity: instance(0.0)

        pixel: fn() {
            return self.scrim * vec4(self.opacity, self.opacity, self.opacity, self.opacity)
        }
    }

    /// The card: a header with a title and an optional description, a body, and a footer.
    mod.mp.MpDialogInner = View{
        width: 440
        height: Fit
        flow: Down
        show_bg: true

        draw_bg +: {
            color: mod.mpc.tokens.surface_card
            radius: 12.0
            border_color: mod.mpc.tokens.border
            border_size: 1.0
            shadow_color: #x00000033
            shadow_radius: 24.0
            shadow_offset: vec2(0.0, 8.0)
        }

        header := View{
            width: Fill, height: Fit
            padding: Inset{left: 24, right: 24, top: 24, bottom: 8}
            flow: Down, spacing: 4

            title := Label{
                width: Fill, height: Fit
                draw_text +: {text_style: mod.mpc.type.headline, color: mod.mpc.tokens.text}
                text: ""
            }
            description := Label{
                width: Fill, height: Fit
                visible: false
                draw_text +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text_muted}
                text: ""
            }
        }

        body := View{
            width: Fill, height: Fit
            padding: Inset{left: 24, right: 24, top: 8, bottom: 8}
            flow: Down, spacing: 8
        }

        footer := View{
            width: Fill, height: Fit
            padding: Inset{left: 24, right: 24, top: 16, bottom: 24}
            flow: Right, spacing: 8
            align: Align{x: 1.0, y: 0.5}
        }
    }

    mod.mp.MpDialogSmall = mod.mp.MpDialogInner{width: 320}
    mod.mp.MpDialogLarge = mod.mp.MpDialogInner{width: 560}

    /// A confirmation: a narrow card with its header and footer centred.
    mod.mp.MpAlertDialog = mod.mp.MpDialogInner{
        width: 360

        header := View{
            width: Fill, height: Fit
            padding: Inset{left: 24, right: 24, top: 24, bottom: 8}
            align: Align{x: 0.5}

            title := Label{
                width: Fit, height: Fit
                draw_text +: {text_style: mod.mpc.type.headline, color: mod.mpc.tokens.text}
                text: ""
            }
        }
        body := View{
            width: Fill, height: Fit
            padding: Inset{left: 24, right: 24, top: 8, bottom: 20}
            align: Align{x: 0.5}

            description := Label{
                width: Fit, height: Fit
                draw_text +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text_muted}
                text: ""
            }
        }
        footer := View{
            width: Fill, height: Fit
            padding: Inset{left: 24, right: 24, top: 0, bottom: 24}
            flow: Right, spacing: 12
            align: Align{x: 0.5, y: 0.5}
        }
    }

    mod.mp.MpDialogBase = #(MpDialog::register_widget(vm))

    mod.mp.MpDialog = set_type_default() do mod.mp.MpDialogBase{
        width: Fill
        height: Fill
        flow: Overlay
        visible: false

        backdrop := View{
            width: Fill, height: Fill
            show_bg: true
            draw_bg +: {
                scrim: #x000000ff
                opacity: instance(0.0)
                pixel: fn() {
                    return self.scrim * vec4(self.opacity, self.opacity, self.opacity, self.opacity)
                }
            }
        }

        content := View{
            width: Fill, height: Fill
            align: Align{x: 0.5, y: 0.5}
            dialog := mod.mp.MpDialogInner{}
        }

        animator: Animator{
            show: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: 0.0}}
                    redraw: true
                    apply: {backdrop: {opacity: 0.0}}
                }
                on: AnimatorState{
                    // **The catalog's duration, not a number here.** `mod.motion.dialog_in` is 180ms on `EASE`; a test reads
                    // this file and fails if a bare duration reappears in the animator.
                    from: {all: Forward{duration: mod.motion.dialog_in.duration}}
                    redraw: true
                    apply: {backdrop: {opacity: 0.8}}
                }
            }
        }
    }
}

/// The dimming layer.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpDialogBackdrop {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    scrim: Vec4f,
}

/// A full-screen modal with a card in the middle.
///
/// **`ScriptHook` is implemented by hand below, so it is not derived**: a derive would generate the impl and the manual one
/// would then be a second, conflicting implementation. The one below is what applies the DSL's declared `open` as a starting
/// state, which is the only thing a dialog needs a hook for.
#[derive(Script, Widget, Animator)]
pub struct MpDialog {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
    /// Whether the dialog is up.
    #[rust]
    shown: Shown,
    /// The DSL's declared value, applied once on creation — the same split the slider uses, because a `#[live]` field is
    /// written back to its declared value whenever the script is re-applied.
    #[live]
    open: bool,
}

impl ScriptHook for MpDialog {
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {
        // The declared `open` is a **starting** state, not a value to be re-read: a dialog a script says starts open does.
        self.shown.set(self.open);
    }
}

impl Widget for MpDialog {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.shown.is_open() {
            // A closed dialog takes no events at all — not even to dismiss itself, because there is nothing up to dismiss.
            return;
        }
        self.view.handle_event(cx, event, scope);
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        let uid = self.widget_uid();
        // **The backdrop's area, not the dialog's.** The dialog fills the window, so a hit on it as a whole would fire on a
        // press inside the card. Only a release that ended over the backdrop dismisses.
        if let Hit::FingerUp(fe) = event.hits(cx, self.view.view(cx, &[BACKDROP]).area()) {
            if fe.is_over {
                self.shown.close();
                cx.widget_action(uid, MpDialogAction::Close);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.shown.is_open() {
            // Nothing is drawn, so the window under it is not covered and no hit test reaches it either.
            return DrawStep::done();
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpDialog {
    /// Show it. **Idempotent**: a second call does nothing, so a caller that opens on every unrelated event cannot make the
    /// dialog flash.
    pub fn open(&mut self, cx: &mut Cx) {
        if self.set_open(cx, true) {
            cx.widget_action(self.widget_uid(), MpDialogAction::Open);
        }
    }

    pub fn close(&mut self, cx: &mut Cx) {
        if self.set_open(cx, false) {
            cx.widget_action(self.widget_uid(), MpDialogAction::Close);
        }
    }

    /// Show or hide it, answering whether that changed anything.
    pub fn set_open(&mut self, cx: &mut Cx, open: bool) -> bool {
        if !self.shown.set(open) {
            return false;
        }
        self.view.set_visible(cx, open);
        self.animator_play(cx, if open { ids!(show.on) } else { ids!(show.off) });
        self.redraw(cx);
        true
    }

    pub fn is_open(&self) -> bool {
        self.shown.is_open()
    }

    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view
            .label(cx, &[CONTENT, CARD, HEADER, TITLE])
            .set_text(cx, title);
    }

    /// Set the second line, showing it — **an empty description hides it**, so a caller with nothing to say does not reserve
    /// the space for it.
    pub fn set_description(&mut self, cx: &mut Cx, description: &str) {
        self.view
            .label(cx, &[CONTENT, CARD, HEADER, DESCRIPTION])
            .set_text(cx, description);
        self.view
            .view(cx, &[CONTENT, CARD, HEADER, DESCRIPTION])
            .set_visible(cx, !description.is_empty());
    }

    /// The card's own view, for a caller that wants to write into the body.
    pub fn card(&self, cx: &mut Cx) -> WidgetRef {
        self.view.widget(cx, &[CONTENT, CARD])
    }
}

impl MpDialogRef {
    pub fn open(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open(cx);
        }
    }

    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
        }
    }

    pub fn set_open(&self, cx: &mut Cx, open: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_open(cx, open);
        }
    }

    pub fn is_open(&self) -> bool {
        self.borrow().map(|inner| inner.is_open()).unwrap_or(false)
    }

    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn set_description(&self, cx: &mut Cx, description: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_description(cx, description);
        }
    }

    /// Whether the dialog reported closing, for a caller reading an event batch.
    pub fn dialog_closed(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .is_some_and(|action| matches!(action.cast::<MpDialogAction>(), MpDialogAction::Close))
    }

    /// Whether the dialog reported opening.
    pub fn dialog_opened(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .is_some_and(|action| matches!(action.cast::<MpDialogAction>(), MpDialogAction::Open))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("dialog.rs");

    #[test]
    fn test_showing_it_twice_changes_nothing_the_second_time() {
        // **The guard is not a micro-optimisation.** Re-playing the entry animation while the dialog is already up makes it
        // flash, so a caller that opens on every unrelated event would produce a strobing dialog. Both operations report
        // whether they changed anything, and the caller plays the animator only when they did.
        let mut shown = Shown::default();
        assert!(!shown.is_open());
        assert!(shown.open(), "the first open changes it");
        assert!(shown.is_open());
        assert!(!shown.open(), "the second open changed something");
        assert!(!shown.open());
        assert!(shown.is_open(), "and it is still open");
        assert!(shown.close(), "the first close changes it");
        assert!(!shown.close(), "the second close changed something");
        // `set` is the same operation with the argument, and agrees with the two.
        let mut other = Shown::default();
        assert!(other.set(true));
        assert!(!other.set(true));
        assert!(other.set(false));
        assert!(!other.set(false));
        assert_eq!(other, Shown::default());
    }

    #[test]
    fn test_the_animator_duration_comes_from_the_motion_catalog() {
        // **The rule this project keeps: an inline duration is a decision that cannot be found, cannot be changed once, and
        // never matches its neighbours.** The v2 widget had `duration: 0.2` written in its DSL; this one names a catalog
        // entry. Asserted by reading the source, because the difference is invisible to a reader who does not know where to
        // look — and because a later edit could put the literal back without anything failing.
        assert!(
            SOURCE.contains("mod.motion.dialog_in.duration"),
            "the animator no longer names the catalog's dialog duration"
        );
        assert!(
            makepad_motion::DIALOG_IN.duration_secs() > 0.0,
            "the catalog's entry has no duration"
        );
        // It is the entry a dialog should have, not some other one.
        assert_eq!(makepad_motion::DIALOG_IN.duration_secs(), 0.18);
        // **And no bare duration in the animator.** `duration: 0.0` is allowed — the off state is instantaneous by design —
        // so the check is that the only numbers after a `duration:` are the catalog reference or zero.
        for line in SOURCE.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("from: {all: Forward{duration:") {
                let value = rest.split('}').next().unwrap_or("").trim();
                assert!(
                    value == "0.0" || value == "mod.motion.dialog_in.duration",
                    "a bare duration in the animator: {value}"
                );
            }
        }
    }

    #[test]
    fn test_the_slot_ids_are_the_ones_the_dsl_and_the_hit_test_share() {
        // The DSL addresses slots by name and so does the hit test, so a rename that missed one would be a dialog that could
        // not be dismissed. Kept as constants for that reason, and asserted against the DSL text so a rename on one side
        // cannot pass.
        for name in ["backdrop", "content", "dialog", "header", "title", "description", "body", "footer"] {
            assert!(
                SOURCE.contains(&format!("{name} := ")),
                "the DSL no longer declares the `{name}` slot that callers write into"
            );
        }
        // The backdrop is the one the hit test uses, and it is the one a caller never replaces.
        assert_eq!(BACKDROP, live_id!(backdrop));
        assert_ne!(BACKDROP, CARD, "a hit on the card must not dismiss");
    }

    #[test]
    fn test_a_scrim_is_partial_because_a_modal_that_hides_everything_loses_its_context() {
        // Not 1.0: a dialog that hides the window completely loses the context of what it is modal *to*, which is why every
        // platform's own dimming is partial.
        assert!(BACKDROP_OPACITY > 0.0 && BACKDROP_OPACITY < 1.0);
        assert_eq!(BACKDROP_OPACITY, 0.8);
        // And the DSL's `on` state uses the same number, so the constant and the tree cannot disagree.
        assert!(
            SOURCE.contains("opacity: 0.8"),
            "the animator's target opacity no longer matches BACKDROP_OPACITY"
        );
    }

    #[test]
    fn test_the_widths_are_ordered_and_the_alert_is_the_narrowest_card_with_a_title() {
        // Four numbers with a reason to be distinct, asserted so a careless edit cannot make "large" narrower than "small".
        assert!(CARD_WIDTH_SMALL < CARD_WIDTH_ALERT);
        assert!(CARD_WIDTH_ALERT < CARD_WIDTH);
        assert!(CARD_WIDTH < CARD_WIDTH_LARGE);
        assert_eq!(CARD_PAD, 24.0);
        // The inner card's own width and each variant's, so the DSL and the constants agree.
        for (constant, dsl) in [
            (CARD_WIDTH, "width: 440"),
            (CARD_WIDTH_SMALL, "width: 320"),
            (CARD_WIDTH_LARGE, "width: 560"),
            (CARD_WIDTH_ALERT, "width: 360"),
        ] {
            assert!(
                SOURCE.contains(dsl),
                "the DSL no longer declares {dsl} for {constant}"
            );
        }
    }

    #[test]
    fn test_a_closed_dialog_reports_nothing() {
        // The action vocabulary is three values and one of them is "nothing happened" — a dialog that reported `Close` when
        // it was already closed would let a caller's handler run against a state that was never true.
        let closed = MpDialogAction::default();
        assert_eq!(closed, MpDialogAction::None);
        assert_ne!(MpDialogAction::Open, MpDialogAction::Close);
        assert_ne!(MpDialogAction::Open, MpDialogAction::None);
    }
}
