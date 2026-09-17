//! `MpHoverCard` — a card that opens on hover and can itself be hovered.
//!
//! ## The state machine is the module
//!
//! gpui owns this: a tooltip there has a 500ms delay built in and stays alive while the
//! pointer is inside it, which is why bezel's `hover_card.rs` is 120 lines of *content* with
//! no timing in it at all. **Makepad owns nothing**, so the timing is a thing this port has to
//! write — and it is the substance, because every one of the four ways it goes wrong is
//! visible the moment the reader moves a mouse:
//!
//! 1. **A card that opens instantly flickers.** The pointer crossing the window on its way
//!    somewhere else passes over dozens of triggers; each one that opens is a flash of
//!    content nobody asked for. So there is a **delay**.
//! 2. **A card that opens on a passing pointer must not.** Which is the same delay, from the
//!    other side: leaving before it elapses has to cancel, not merely not-open-yet.
//! 3. **A card the pointer can enter must not close when the pointer enters it.** This is the
//!    one that separates a hover card from a tooltip: the pointer leaves the *trigger* and
//!    arrives at the *card*, and a machine that closes on "left the trigger" closes the thing
//!    the reader is reaching for.
//! 4. **A card must not flicker when the pointer crosses the gap.** Between the trigger and
//!    the card there is a strip belonging to neither, and a machine with no grace period
//!    closes there.
//!
//! All four are properties of one small machine, so all four are tested with no window at
//! all — which matters here more than usual, because **a synthetic pointer cannot produce a
//! hover event in this app**, so no screenshot can check any of it.
//!
//! ## Where the numbers come from
//!
//! [`DEFAULT_DELAY_MS`] is **500**, which is gpui's own tooltip delay and therefore the
//! reference this port is measured against — the same convention as every other number that
//! reached the theme because a platform named it. [`DEFAULT_GRACE_MS`] is **150**, which is
//! chosen: it only has to cover a hand crossing a small gap, and the cost of it being too long
//! is a card that lingers after the reader has left.

use makepad_widgets::*;

/// How long the pointer must rest on a trigger before the card opens.
///
/// gpui's own tooltip delay, so a control in this library opens when the same control would
/// open under the reference implementation.
pub const DEFAULT_DELAY_MS: f64 = 500.0;

/// How long the card stays open after the pointer has left both the trigger and the card.
///
/// Chosen, not measured. It exists to cover the strip of window between the trigger and the
/// card, which belongs to neither — long enough for a hand to cross it, short enough that a
/// card the reader has actually left goes away promptly.
pub const DEFAULT_GRACE_MS: f64 = 150.0;

/// Where the pointer is, as far as the card is concerned.
///
/// Three states rather than a `hovered: bool`, because **"inside the card" and "inside the
/// trigger" have to be told apart**: one of them is what stops the card closing on the way in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    /// Over neither the trigger nor the card.
    Outside,
    /// Over the thing the card belongs to.
    OnTrigger,
    /// Inside the card itself.
    OnCard,
}

/// What a tick of the machine decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Change {
    /// Nothing changed.
    #[default]
    Nothing,
    /// The card should open.
    Opened,
    /// The card should close.
    Closed,
}

/// When a hover card opens and closes.
///
/// One small machine, driven by `update` with where the pointer is and how much time has
/// passed. It draws nothing and owns nothing; the caller does what [`Change`] says. That is
/// what makes four interaction rules testable without a window.
#[derive(Clone, Debug, PartialEq)]
pub struct HoverIntent {
    delay_ms: f64,
    grace_ms: f64,
    /// How long the pointer has rested on the trigger.
    dwell_ms: f64,
    /// How long the pointer has been outside both, while open.
    away_ms: f64,
    open: bool,
}

impl Default for HoverIntent {
    fn default() -> Self {
        Self::new(DEFAULT_DELAY_MS, DEFAULT_GRACE_MS)
    }
}

impl HoverIntent {
    pub fn new(delay_ms: f64, grace_ms: f64) -> Self {
        Self {
            delay_ms: delay_ms.max(0.0),
            grace_ms: grace_ms.max(0.0),
            dwell_ms: 0.0,
            away_ms: 0.0,
            open: false,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn delay_ms(&self) -> f64 {
        self.delay_ms
    }

    pub fn grace_ms(&self) -> f64 {
        self.grace_ms
    }

    /// How long the pointer has rested on the trigger, for a caller drawing a progress ring.
    pub fn dwell_ms(&self) -> f64 {
        self.dwell_ms
    }

    /// Advance the machine by `dt_ms`, with the pointer `presence`.
    ///
    /// Returns what the caller should do. A `dt_ms` that is not a positive finite number is
    /// ignored rather than treated as zero, so a caller that has not computed a frame delta
    /// yet cannot accidentally open a card by passing a `NaN`.
    pub fn update(&mut self, presence: Presence, dt_ms: f64) -> Change {
        if !dt_ms.is_finite() || dt_ms <= 0.0 {
            return Change::Nothing;
        }
        match presence {
            Presence::OnTrigger => {
                // Inside either side cancels a pending close.
                self.away_ms = 0.0;
                if self.open {
                    return Change::Nothing;
                }
                self.dwell_ms += dt_ms;
                if self.dwell_ms >= self.delay_ms {
                    self.open = true;
                    return Change::Opened;
                }
                Change::Nothing
            }
            Presence::OnCard => {
                // Inside *either* side cancels a pending close — this is the arm that makes a
                // hover card hoverable. On its own it never opens one: a card the pointer
                // could reach without ever having rested on the trigger would be a card that
                // appears out of nowhere.
                self.away_ms = 0.0;
                Change::Nothing
            }
            Presence::Outside => {
                if self.open {
                    self.away_ms += dt_ms;
                    if self.away_ms >= self.grace_ms {
                        self.open = false;
                        self.dwell_ms = 0.0;
                        self.away_ms = 0.0;
                        return Change::Closed;
                    }
                    return Change::Nothing;
                }
                // Never opened: the pointer did not rest long enough. **Reset rather than
                // merely stop counting**, so a pointer that sweeps across a row of triggers
                // accumulates nothing — otherwise four quick passes would add up to one delay
                // and open a card nobody rested on.
                self.dwell_ms = 0.0;
                Change::Nothing
            }
        }
    }

    /// Close at once, for a caller that knows the card must go — the window lost focus, the
    /// page changed, the trigger scrolled away.
    pub fn reset(&mut self) {
        self.open = false;
        self.dwell_ms = 0.0;
        self.away_ms = 0.0;
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    // The paint tokens and the type rungs this card reads. Without them the DSL reports
    // `variable surface_dialog not found in scope` — a *variable* rather than a property,
    // which is the shape that says the module was never imported rather than the name being
    // wrong.
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The card itself: initials, a title, a body, and a quiet line under it.
    ///
    /// `surface_dialog` rather than `surface_card`, because a hover card floats over the
    /// window like a popover and a palette do — the same rung, so every floating thing in the
    /// library reads as the same distance from the page.
    ///
    /// A **wider box than a tooltip on purpose**: this one carries prose, and prose on one
    /// line is a tooltip with a taller border.
    mod.mp.MpHoverCard = View{
        width: 320
        height: Fit
        flow: Down
        spacing: 8
        padding: Inset{left: 14.0, right: 14.0, top: 12.0, bottom: 12.0}

        draw_bg +: {
            color: surface_dialog
            border_color: border
            border_size: 1.0
            border_radius: 8.0
        }

        hovercard_head := mod.mp.Row{
            width: Fill
            height: Fit
            spacing: 10

            hovercard_avatar := mod.mp.MpAvatar{
                width: 32
                height: 32
            }
            hovercard_title := Label{
                width: Fill
                height: Fit
                draw_text +: {text_style: body, color: text}
                text: ""
            }
        }
        hovercard_body := Label{
            width: Fill
            height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: ""
        }
        hovercard_meta := Label{
            width: Fill
            height: Fit
            draw_text +: {text_style: caption, color: text_faint}
            text: ""
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive the machine for `total_ms` in `step`-sized ticks at a fixed presence, and report
    /// everything it decided. Ticks rather than one big jump, because a real caller ticks per
    /// frame and a machine that only works for one jump is not the machine being shipped.
    fn run(
        intent: &mut HoverIntent,
        presence: Presence,
        total_ms: f64,
        step_ms: f64,
    ) -> Vec<Change> {
        let mut changes = Vec::new();
        let mut elapsed = 0.0;
        while elapsed < total_ms {
            let dt = step_ms.min(total_ms - elapsed);
            changes.push(intent.update(presence, dt));
            elapsed += dt;
        }
        changes
    }

    #[test]
    fn test_a_pointer_that_rests_opens_the_card_after_the_delay() {
        let mut intent = HoverIntent::new(500.0, 150.0);
        let changes = run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
        assert!(intent.is_open());
        assert_eq!(
            changes.iter().filter(|c| **c == Change::Opened).count(),
            1,
            "it opened more than once: {changes:?}"
        );
        // And not before the delay elapsed.
        let mut early = HoverIntent::new(500.0, 150.0);
        run(&mut early, Presence::OnTrigger, 400.0, 16.0);
        assert!(!early.is_open(), "it opened before its delay");
    }

    #[test]
    fn test_a_pointer_that_leaves_before_the_delay_never_opens() {
        // The flicker case: the pointer crosses the window on its way somewhere else, and a
        // card that opens is a flash of content nobody asked for.
        let mut intent = HoverIntent::new(500.0, 150.0);
        run(&mut intent, Presence::OnTrigger, 300.0, 16.0);
        assert!(!intent.is_open());
        let changes = run(&mut intent, Presence::Outside, 1000.0, 16.0);
        assert!(!intent.is_open(), "a card opened after the pointer left");
        assert!(
            !changes.contains(&Change::Opened),
            "it opened while the pointer was outside"
        );
    }

    #[test]
    fn test_the_dwell_does_not_accumulate_across_separate_passes() {
        // **Reset rather than merely stop counting.** Four quick passes across the same
        // trigger must not add up to one delay — otherwise a card opens for a pointer that
        // never rested, which is the flicker the delay exists to prevent, just slower.
        let mut intent = HoverIntent::new(500.0, 150.0);
        for _ in 0..4 {
            run(&mut intent, Presence::OnTrigger, 200.0, 16.0);
            run(&mut intent, Presence::Outside, 100.0, 16.0);
            assert!(!intent.is_open());
        }
        assert_eq!(intent.dwell_ms(), 0.0, "the dwell survived a departure");
    }

    #[test]
    fn test_the_card_stays_open_while_the_pointer_is_inside_it() {
        // **The property that makes a hover card a hover card.** The pointer leaves the
        // trigger and arrives at the card; a machine that closed on "left the trigger" would
        // close the thing the reader is reaching for.
        let mut intent = HoverIntent::new(500.0, 150.0);
        run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
        assert!(intent.is_open());
        // Away from the trigger for far longer than the grace, but inside the card.
        let changes = run(&mut intent, Presence::OnCard, 5000.0, 16.0);
        assert!(intent.is_open(), "the card closed while the pointer was in it");
        assert!(!changes.contains(&Change::Closed));
    }

    #[test]
    fn test_crossing_the_gap_between_trigger_and_card_does_not_close_the_card() {
        // The strip between the two belongs to neither, and a machine with no grace period
        // closes there — which is what makes a hover card unusable: it disappears exactly as
        // the reader moves toward it.
        let mut intent = HoverIntent::new(500.0, 150.0);
        run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
        // A gap crossing shorter than the grace.
        let changes = run(&mut intent, Presence::Outside, 100.0, 16.0);
        assert!(intent.is_open(), "it closed in the gap");
        assert!(!changes.contains(&Change::Closed));
        run(&mut intent, Presence::OnCard, 16.0, 16.0);
        assert!(intent.is_open());
    }

    #[test]
    fn test_leaving_both_closes_the_card_after_the_grace() {
        let mut intent = HoverIntent::new(500.0, 150.0);
        run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
        let changes = run(&mut intent, Presence::Outside, 200.0, 16.0);
        assert!(!intent.is_open());
        assert_eq!(
            changes.iter().filter(|c| **c == Change::Closed).count(),
            1,
            "it closed more than once, or never: {changes:?}"
        );
    }

    #[test]
    fn test_re_entering_during_the_grace_cancels_the_close() {
        // The reader leaves, changes their mind, and comes back before the grace is up. The
        // card must still be there — closing and re-opening it would be a flicker, and
        // re-opening would cost another full delay.
        let mut intent = HoverIntent::new(500.0, 150.0);
        run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
        run(&mut intent, Presence::Outside, 100.0, 16.0);
        assert!(intent.is_open());
        run(&mut intent, Presence::OnCard, 16.0, 16.0);
        // Well past the grace from here, still inside.
        let changes = run(&mut intent, Presence::OnCard, 400.0, 16.0);
        assert!(intent.is_open());
        assert!(!changes.contains(&Change::Closed));
    }

    #[test]
    fn test_a_card_that_closes_needs_the_full_delay_to_open_again() {
        // After a close the machine is where it started, so a pointer still resting does not
        // reopen instantly — which would be the flicker again.
        let mut intent = HoverIntent::new(500.0, 150.0);
        run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
        run(&mut intent, Presence::Outside, 200.0, 16.0);
        assert!(!intent.is_open());
        run(&mut intent, Presence::OnTrigger, 400.0, 16.0);
        assert!(!intent.is_open(), "it reopened without waiting");
        run(&mut intent, Presence::OnTrigger, 200.0, 16.0);
        assert!(intent.is_open());
    }

    #[test]
    fn test_being_inside_the_card_never_opens_one() {
        // A card the pointer could reach without ever having rested on the trigger would be a
        // card that appears out of nowhere. `OnCard` cancels a close; it never opens.
        let mut intent = HoverIntent::new(500.0, 150.0);
        let changes = run(&mut intent, Presence::OnCard, 5000.0, 16.0);
        assert!(!intent.is_open());
        assert!(!changes.contains(&Change::Opened));
    }

    #[test]
    fn test_a_zero_delay_opens_on_the_first_tick_and_a_zero_grace_closes_at_once() {
        // The degenerate configuration, which is what a caller gets by asking for a card with
        // no delay — the behaviour a tooltip library would call "immediate".
        let mut intent = HoverIntent::new(0.0, 0.0);
        assert_eq!(intent.update(Presence::OnTrigger, 1.0), Change::Opened);
        assert_eq!(intent.update(Presence::Outside, 1.0), Change::Closed);
        assert!(!intent.is_open());
    }

    #[test]
    fn test_a_delta_that_is_not_a_positive_number_changes_nothing() {
        // A caller that has not computed a frame delta yet passes a zero, and one that has not
        // laid out passes a `NaN`. Neither may open a card: a `NaN` fails every comparison, so
        // a machine that tested `dwell >= delay` without guarding would leave `dwell` a `NaN`
        // and the card would never open *or* close again.
        let mut intent = HoverIntent::new(500.0, 150.0);
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(intent.update(Presence::OnTrigger, bad), Change::Nothing);
        }
        assert!(!intent.is_open());
        assert_eq!(intent.dwell_ms(), 0.0);
        // ...and the machine still works afterwards.
        assert_eq!(intent.update(Presence::OnTrigger, 500.0), Change::Opened);
    }

    #[test]
    fn test_a_negative_delay_or_grace_is_clamped_rather_than_kept() {
        // A negative grace would mean "close before the pointer left", which is not a
        // configuration anyone wants and would make the machine's own tests read as nonsense.
        let intent = HoverIntent::new(-100.0, -100.0);
        assert_eq!(intent.delay_ms(), 0.0);
        assert_eq!(intent.grace_ms(), 0.0);
    }

    #[test]
    fn test_reset_closes_at_once_from_any_state() {
        // For a caller that knows the card must go: the window lost focus, the page changed,
        // the trigger scrolled away.
        for open in [false, true] {
            let mut intent = HoverIntent::new(500.0, 150.0);
            if open {
                run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
            }
            intent.reset();
            assert!(!intent.is_open());
            assert_eq!(intent.dwell_ms(), 0.0);
        }
    }

    #[test]
    fn test_a_real_approach_reads_the_way_a_hover_card_behaves() {
        // The session in the order it actually happens, tick by tick, so the four rules are
        // exercised as one sequence rather than as four isolated ones: sweep past, rest,
        // open, move into the card, read it, leave.
        let mut intent = HoverIntent::new(500.0, 150.0);
        // 1. A sweep: three short passes that must all amount to nothing.
        for _ in 0..3 {
            run(&mut intent, Presence::OnTrigger, 150.0, 16.0);
            run(&mut intent, Presence::Outside, 60.0, 16.0);
            assert!(!intent.is_open(), "a sweep opened a card");
        }
        // 2. A rest: the card opens.
        let changes = run(&mut intent, Presence::OnTrigger, 500.0, 16.0);
        assert!(intent.is_open());
        assert_eq!(changes.iter().filter(|c| **c == Change::Opened).count(), 1);
        // 3. Moving into the card, with a gap crossing partway.
        run(&mut intent, Presence::Outside, 80.0, 16.0);
        assert!(intent.is_open(), "it closed in the gap");
        run(&mut intent, Presence::OnCard, 2000.0, 16.0);
        assert!(intent.is_open(), "it closed while being read");
        // 4. Leaving for good.
        let changes = run(&mut intent, Presence::Outside, 200.0, 16.0);
        assert!(!intent.is_open());
        assert_eq!(changes.iter().filter(|c| **c == Change::Closed).count(), 1);
    }
}
