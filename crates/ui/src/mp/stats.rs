//! `MpStats` — the meter: how many frames this window is drawing, and what the process costs, while you watch it.
//!
//! ## The rule the component exists for
//!
//! **A meter that drives frames cannot measure frames.** If it asks for the next frame in order to refresh its own digits,
//! then its own asking is what makes the window draw — and a counter that included those draws would report the meter's
//! rate rather than the application's. A window at rest would read 60.
//!
//! makepad answers this directly, which is why this is a port rather than an approximation: `NextFrameEvent` carries
//! `frame: u64`, the window's own frame number, **and `set: HashSet<NextFrame>`, the handles that asked for this frame**. So
//! the count comes from the platform and the meter can tell whether the frame it is looking at is one it asked for. That is
//! the same information gpui's `Painter::woken` gives bezel's meter, and the reason the reading is a rate rather than a
//! self-portrait.
//!
//! ## What is reduced, and it is said rather than glossed
//!
//! bezel's meter shows four numbers: frames, CPU, GPU and memory. This one shows **frames and memory**:
//!
//! - **Memory is real**: `platform::memory_watchdog::process_footprint_bytes()`, which is `Option` because some platforms
//!   hand out no process figure. A `None` is shown as `—` rather than as zero, because zero is a measurement and absence is
//!   not.
//! - **CPU has no measurement point in makepad.** bezel differences process CPU time across its tick; there is no equivalent
//!   helper here, and a `std::time::Instant` measures wall time, not CPU time. Reporting wall time under the label "CPU"
//!   would be a number that looks like the real one and is not.
//! - **GPU likewise.** bezel's is `None` off Metal; here it is `None` everywhere.
//!
//! So three of bezel's four figures become one and one refusal. **That is a reduction in what the component can tell you**,
//! and a port that showed the same four labels with zeros would be claiming a parity it does not have.
//!
//! ## The digits are held between recomputes
//!
//! [`HOLD`]: a rate recomputed on every frame changes too fast to read, and a number that flickers is a number nobody reads.
//! The meter accumulates and re-reads once a second, which is the other half of what makes it usable rather than merely
//! correct.
//!
//! ## Placement is the caller's
//!
//! A meter is an overlay in a corner, and which corner is the application's business — the same choice bezel leaves to its
//! caller. This draws a plate and two figures wherever it is put.

use makepad_widgets::*;

/// How long a reading is held before it is recomputed, in seconds.
///
/// One second: a rate recomputed per frame changes too fast to read, and a number that flickers is a number nobody reads. A
/// whole second is also the shortest span over which a frame count is a rate rather than an artefact of one slow frame.
pub const HOLD: f64 = 1.0;

/// How often the meter looks at the clock, in seconds.
///
/// **Deliberately a quarter of [`HOLD`], and the first version of this had no such constant — it used `HOLD` as the timer
/// period, and the readout stuttered.** With the tick and the hold both at one second, the timer's real period lands a hair
/// under or a hair over the nominal one, so the check `elapsed >= HOLD` alternates: `0.9998s` fails and the reading waits for
/// the next tick, `1.0005s` passes. Measured spans came out `1.0s, 2.0s, 1.0s, 2.0s` — a readout changing at two different
/// rates, which is exactly what holding a reading is supposed to prevent.
///
/// A boundary that random jitter falls on both sides of is not a hold. Looking four times as often as the hold makes the
/// margin 4x rather than 0.01%, so the hold decides and the jitter cannot.
pub const TICK: f64 = 0.25;

/// The meter's padding inside its plate.
pub const PAD: f64 = 8.0;

/// The gap between the two figures.
pub const GAP: f64 = 4.0;

/// The text shown where a figure has no measurement.
pub const UNKNOWN: &str = "\u{2014}";

/// A frame count over a span, and the rule that keeps it honest.
///
/// The bookkeeping is small enough to look trivial and is the whole component: `observed` is called once per frame with the
/// window's frame number and whether this meter asked for that frame, and it answers whether **this** frame counts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Counter {
    last: Option<u64>,
    /// The frame number the current span started at, so the span's **own** length is knowable.
    ///
    /// This exists to make the exclusion measurable rather than asserted: `span` counts every frame the platform drew,
    /// including the ones this meter asked for, while `counted` excludes them. Printing both is how a run shows the rule
    /// working — `excluded > 0` means the exclusion actually fired, where a diagnostic that simply said "exclusion on" would
    /// be formatting its own input.
    started: Option<u64>,
    /// Frames counted since the counter was last read.
    counted: u32,
}

impl Counter {
    /// Note a frame, and answer whether it is one of the application's.
    ///
    /// A frame the meter itself provoked is **not** counted — see the module doc. A frame number that went backwards (a
    /// window recreated, a `u64` wrap) restarts the span rather than counting a negative number of frames or a huge
    /// positive one.
    pub fn observe(&mut self, frame: u64, own_tick: bool) -> bool {
        match self.last {
            // A frame that is not ahead of the last one is not a frame: it is a restart, and counting the difference would
            // report either a negative count or `u64::MAX` of them.
            Some(last) if frame <= last => {
                self.last = Some(frame);
                return false;
            }
            Some(_) => {}
            // The first frame seen establishes the span; there is no interval to have counted.
            None => {
                self.last = Some(frame);
                self.started = Some(frame);
                return false;
            }
        }
        self.last = Some(frame);
        if own_tick {
            return false;
        }
        self.counted = self.counted.saturating_add(1);
        true
    }

    /// Frames counted since the last read.
    pub fn counted(&self) -> u32 {
        self.counted
    }

    /// Take the count and start a new span.
    pub fn take(&mut self) -> u32 {
        // The new span starts where the old one ended, so a frame is never in two spans or in none.
        self.started = self.last;
        core::mem::take(&mut self.counted)
    }

    /// How many frames the platform drew in the current span, **the excluded ones included**.
    pub fn span(&self) -> u64 {
        match (self.started, self.last) {
            (Some(started), Some(last)) if last >= started => last - started,
            _ => 0,
        }
    }

    /// Whether held long enough to read, and if so how many frames were in the span.
    ///
    /// `None` means "keep accumulating": the span is shorter than [`HOLD`], or the clock did not advance (a zero or
    /// non-finite span would otherwise divide into a rate, which is where an `f32::INFINITY` reading comes from).
    pub fn read_if_due(&mut self, elapsed: f64) -> Option<u32> {
        if !elapsed.is_finite() || elapsed < HOLD {
            return None;
        }
        Some(self.take())
    }
}

/// Frames per second over `elapsed` seconds.
///
/// **Zero when the span is not a positive, finite number**, rather than an infinity or a `NaN` — a rate is a quotient and
/// this is the one place in the component where a division happens.
pub fn rate(frames: u32, elapsed: f64) -> f32 {
    if !elapsed.is_finite() || elapsed <= 0.0 {
        return 0.0;
    }
    (frames as f64 / elapsed) as f32
}

/// The frame rate as it is printed: a whole number, because a rate quoted to a decimal place is a precision the measurement
/// does not have.
pub fn format_fps(fps: Option<f32>) -> String {
    let Some(fps) = fps else {
        return UNKNOWN.to_string();
    };
    if !fps.is_finite() || fps < 0.0 {
        return UNKNOWN.to_string();
    }
    format!("{:.0} fps", fps)
}

/// A byte count as it is printed: whole units, one decimal place above a kilobyte.
///
/// Binary units, because this is a resident footprint and the platform reports bytes — a "MB" that meant 10^6 next to a
/// figure the operating system shows in MiB is a number that disagrees with the system's own by 5%.
///
/// `None` prints [`UNKNOWN`], **not zero**: zero is a measurement and an absent one is not.
pub fn format_bytes(bytes: Option<u64>) -> String {
    let Some(bytes) = bytes else {
        return UNKNOWN.to_string();
    };
    const KIB: f64 = 1024.0;
    let b = bytes as f64;
    if b < KIB {
        return format!("{bytes} B");
    }
    // **The KiB rung, which my first version did not have**: without it the smallest above-byte figure was a megabyte, and
    // 1024 bytes printed as `0.0 MiB` — a rounded-away measurement shown as a zero. A test expecting `1.0 MiB` found it; the
    // right answer was neither, and the ladder was the thing that was wrong.
    let kib = b / KIB;
    if kib < KIB {
        return format!("{kib:.1} KiB");
    }
    let mib = kib / KIB;
    if mib < KIB {
        return format!("{mib:.1} MiB");
    }
    format!("{:.2} GiB", mib / KIB)
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.DrawMpStatsPlate = #(DrawMpStatsPlate::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 6.0
        plate: #x00000000
        border_color: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.5, 0.5, sz.x - 1.0, sz.y - 1.0, self.radius)
            sdf.fill_keep(self.plate)
            sdf.stroke(self.border_color, 1.0)
            return sdf.result
        }
    }

    mod.mp.MpStatsBase = #(MpStats::register_widget(vm))

    mod.mp.MpStats = set_type_default() do mod.mp.MpStatsBase{
        width: Fit
        height: Fit
        // **`true`, and the first run proved why it has to be stated.** A `bool` `#[live]` field with no DSL value is
        // `false`, so both meters on the page were silent — the interval never started, no reading was ever taken, and the
        // run printed no `STATS counted` lines at all. It looked like a working widget showing `—`, which is exactly what a
        // widget that has not read yet shows.
        ticking: true

        draw_figure +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text}
        draw_label +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_muted}
    }
}

/// The meter's own plate.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpStatsPlate {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
    #[live]
    border_color: Vec4f,
}

/// The in-window meter.
#[derive(Script, ScriptHook, Widget)]
pub struct MpStats {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_bg: DrawMpStatsPlate,
    /// The figures.
    #[live]
    draw_figure: DrawText,
    /// Their captions.
    #[live]
    draw_label: DrawText,
    /// Whether the meter asks for frames at all.
    ///
    /// **Off, the meter never refreshes** — it draws whatever it last read, for a caller that wants the plate without the
    /// cost of a ticking timer. On, it asks for the next frame and the exclusion rule in [`Counter`] is what keeps the
    /// asking out of the count.
    #[live]
    ticking: bool,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    counter: Counter,
    #[rust]
    since: Option<f64>,
    /// The handle of the meter's own next-frame request, so a frame can be recognised as one it asked for.
    #[rust]
    next: Option<NextFrame>,
    /// The last frame rate. `Option` for the same reason `mem` is: **a meter that has not read yet has no rate**, and
    /// printing `0 fps` would be reporting a measurement it never took.
    #[rust]
    fps: Option<f32>,
    #[rust]
    mem: Option<u64>,
    /// The one-second interval that drives the reading. Only started when `ticking`, and started once.
    #[rust]
    timer: Option<Timer>,
    #[rust]
    area: Area,
}

impl MpStats {
    /// The last reading, or `None` before the first.
    pub fn fps(&self) -> Option<f32> {
        self.fps
    }

    /// The last memory reading, or `None` where the platform has none.
    pub fn memory(&self) -> Option<u64> {
        self.mem
    }

    /// What the meter prints for both figures.
    pub fn reading(&self) -> (String, String) {
        (format_fps(self.fps), format_bytes(self.mem))
    }

    /// Take a reading if one is due, answering whether the digits changed.
    fn tick(&mut self, now: f64) -> bool {
        let Some(since) = self.since else {
            self.since = Some(now);
            return false;
        };
        // **The span is read before the counter is taken**, because `take` starts the next span where this one ended — so a
        // diagnostic that measured afterwards would report a span of zero. My first version did exactly that and printed
        // `span_frames=0` for every reading, which is the shape of a diagnostic measuring after the mutation it should
        // measure before.
        let span_frames = self.counter.span();
        if let Some(frames) = self.counter.read_if_due(now - since) {
            self.fps = Some(rate(frames, now - since));
            // **Sampled at the tick rather than differenced across one**: a footprint is a level, not a rate, so measuring
            // how much it changed would report `0 B` for a process holding a steady gigabyte.
            self.mem = makepad_platform::memory_watchdog::process_footprint_bytes();
            self.since = Some(now);
            // **`MP_STATS_DEBUG=1` is how this component is verified at all.** A meter's whole claim is about what it counts,
            // and no screenshot can show whether its own ticks were excluded — only the numbers can. The frame number comes
            // from the platform and the count from `Counter`, so a reading of `60 fps` on a window at rest would be visible
            // here as the meter measuring itself.
            if std::env::var("MP_STATS_DEBUG").is_ok() {
                // **`span_frames` against `counted`, not a claim about the exclusion.** The platform's frame number
                // advanced by `span` while the meter counted `counted`; the difference is the meter's own ticks. A run
                // printing `excluded=0` would mean the rule never fired — which a line saying "own_ticks_excluded=yes"
                // could not have told anyone, because it was a string I wrote rather than a number the code produced.
                println!(
                    "STATS counted={frames} span_frames={span_frames} excluded={} span={:.3}s fps={:.1} mem={:?}",
                    span_frames.saturating_sub(frames as u64),
                    now - since,
                    self.fps.unwrap_or(0.0),
                    self.mem,
                );
            }
            return true;
        }
        false
    }
}

impl Widget for MpStats {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // **The interval, not a per-frame request.** bezel's meter re-reads once a second and excludes the single render that
        // reading provoked; a meter that asked for *every* frame would have its handle in every frame's `set`, so the
        // exclusion would throw away the application's frames along with its own and a busy window would read zero. One
        // request per **reading** — not per look at the clock — is what makes the exclusion an exclusion rather than a mute.
        if let Event::Timer(te) = event {
            if self.timer.as_ref().is_some_and(|t| t.0 == te.timer_id) {
                if let Some(time) = te.time {
                    if std::env::var("MP_STATS_DEBUG").is_ok() {
                        // The interval's own cadence, printed rather than assumed — because the reading came out at two
                        // seconds with the hold declared as one, and a doubled duration is a fact to measure. This print is
                        // what turned the stutter from a suspicion into `delta=Some(0.9998091250000001)`: a hair under the
                        // hold, so the reading waited for the next tick.
                        println!(
                            "STATS timer time={time:.3} delta={:?}",
                            self.since.map(|since| time - since),
                        );
                    }
                    // **A frame is requested only when there is something new to show.** One self-provoked frame per
                    // reading, not one per look at the clock: the meter's own cost should be one frame a second, and a
                    // request per tick would be four.
                    if self.tick(time) {
                        self.next = Some(cx.new_next_frame());
                        self.redraw(cx);
                    }
                }
            }
        }
        if let Event::NextFrame(ne) = event {
            // **The rule, in one line.** `set` is the handles that asked for this frame, so the meter knows whether the frame
            // in front of it is one of the application's or one it asked for itself. Without this the reading would be the
            // meter's own rate: a window at rest would report the refresh rate.
            // **Taken, not peeked**: the handle identifies one frame, and leaving it set would exclude every later frame the
            // window happened to draw while it was stale.
            let own_tick = self
                .next
                .take()
                .is_some_and(|mine| ne.set.contains(&mine));
            self.counter.observe(ne.frame, own_tick);
            self.redraw(cx);
        }
        let _ = event.hits(cx, self.area);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (plate, border, ink, muted) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (
                theme.paint.surface_overlay,
                theme.paint.border,
                theme.paint.text,
                theme.paint.text_muted,
            )
        };
        // Started on the first paint, once: a timer needs a context and a widget has none at construction.
        if self.ticking && self.timer.is_none() {
            self.timer = Some(cx.cx.start_interval(TICK));
        }
        let fps = format_fps(self.fps);
        let mem = format_bytes(self.mem);
        let fps_w = crate::mp::text::measured_width(&self.draw_figure, cx.cx, &fps);
        let mem_w = crate::mp::text::measured_width(&self.draw_figure, cx.cx, &mem);
        let caption = crate::mp::text::measured_width(&self.draw_label, cx.cx, "memory");
        let width = PAD * 2.0 + fps_w.max(caption) + GAP + mem_w.max(caption);
        let height = PAD * 2.0 + 34.0;

        let placed = cx.walk_turtle(Walk {
            width: Size::Fixed(width),
            height: Size::Fixed(height),
            ..walk
        });
        self.area = self.draw_bg.area();
        self.draw_bg.plate = plate;
        self.draw_bg.border_color = border;
        self.draw_bg.radius = 6.0;
        self.draw_bg.draw_abs(cx, placed);

        let y = placed.pos.y + PAD;
        self.draw_figure.color = ink;
        self.draw_figure.draw_abs(cx, dvec2(placed.pos.x + PAD, y), &fps);
        self.draw_label.color = muted;
        self.draw_label
            .draw_abs(cx, dvec2(placed.pos.x + PAD, y + 18.0), "frames");
        let x = placed.pos.x + PAD + fps_w.max(caption) + GAP;
        self.draw_figure.draw_abs(cx, dvec2(x, y), &mem);
        self.draw_label
            .draw_abs(cx, dvec2(x, y + 18.0), "memory");
        DrawStep::done()
    }
}

impl MpStatsRef {
    pub fn reading(&self) -> (String, String) {
        self.borrow()
            .map(|inner| inner.reading())
            .unwrap_or_else(|| (UNKNOWN.to_string(), UNKNOWN.to_string()))
    }

    pub fn fps(&self) -> Option<f32> {
        self.borrow().and_then(|inner| inner.fps())
    }

    pub fn memory(&self) -> Option<u64> {
        self.borrow().and_then(|inner| inner.memory())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_frame_the_meter_asked_for_is_not_counted() {
        // **The rule the component exists for.** A meter that counted the frames its own tick provoked would report its own
        // rate — a window at rest would read the refresh rate, which is the most confidently wrong a number can be.
        let mut counter = Counter::default();
        assert!(!counter.observe(100, false), "the first frame establishes the span");
        assert!(counter.observe(101, false), "the application drew");
        assert!(!counter.observe(102, true), "the meter asked for this one");
        assert!(counter.observe(103, false));
        assert_eq!(counter.counted(), 2, "two of the four frames were the application's");
        // And the exclusion is per-frame, not a permanent switch.
        assert!(!counter.observe(104, true));
        assert!(!counter.observe(105, true));
        assert_eq!(counter.counted(), 2, "neither tick counted");
        assert!(counter.observe(106, false));
        assert_eq!(counter.counted(), 3);
    }

    #[test]
    fn test_a_frame_number_that_did_not_advance_is_not_a_frame() {
        // A window recreated, or a `u64` that wrapped: counting the difference would report either a negative number of
        // frames or `u64::MAX` of them, and the rate would be absurd in either direction. The span restarts instead.
        let mut counter = Counter::default();
        counter.observe(500, false);
        assert!(!counter.observe(500, false), "the same frame number");
        assert!(!counter.observe(499, false), "a number that went backwards");
        assert_eq!(counter.counted(), 0, "a restart counted frames");
        // And the span resumes from the new number rather than from the old one.
        assert!(counter.observe(501, false));
        assert_eq!(counter.counted(), 1);
    }

    #[test]
    fn test_a_reading_is_held_until_the_span_is_long_enough_to_read() {
        // **The other half of what makes the meter usable rather than merely correct.** A rate recomputed per frame changes
        // too fast to read, and a number that flickers is a number nobody reads.
        assert_eq!(HOLD, 1.0);
        let mut counter = Counter::default();
        counter.observe(0, false);
        for frame in 1..=30u64 {
            counter.observe(frame, false);
        }
        assert_eq!(counter.read_if_due(0.2), None, "a fifth of a second is too short");
        assert_eq!(counter.counted(), 30, "and nothing was taken");
        assert_eq!(counter.read_if_due(0.999), None);
        assert_eq!(counter.read_if_due(1.0), Some(30), "a whole second is enough");
        assert_eq!(counter.counted(), 0, "the count was taken");
        // A span that is not a positive number does not divide into a rate.
        assert_eq!(counter.read_if_due(0.0), None);
        assert_eq!(counter.read_if_due(-5.0), None);
        assert_eq!(counter.read_if_due(f64::NAN), None);
        assert_eq!(counter.read_if_due(f64::INFINITY), None);
    }

    #[test]
    fn test_the_tick_is_well_inside_the_hold_so_the_readout_cannot_stutter() {
        // **The rule `TICK` exists for, and the stutter it prevents reproduced as a test.** With the timer period equal to
        // the hold, the timer's real period lands a hair under or over the nominal one and the check alternates — measured
        // spans were `1.0s, 2.0s, 1.0s, 2.0s` — so the readout changes at two different rates, which is exactly what holding
        // a reading is supposed to prevent. A boundary that random jitter falls on both sides of is not a hold.
        assert!(TICK > 0.0);
        assert!(TICK < HOLD, "the tick must be shorter than the hold");
        assert!(
            HOLD / TICK >= 2.0,
            "a margin of at least 2x, so a hair of jitter cannot flip the comparison"
        );

        // The failure itself: 60 frames over a span a ten-thousandth under the hold does **not** read, so the digits wait for
        // the next tick. That is the stutter, and it is why the tick is not the hold.
        let mut counter = Counter::default();
        counter.observe(0, false);
        for frame in 1..=60u64 {
            counter.observe(frame, false);
        }
        assert_eq!(
            counter.read_if_due(0.9998),
            None,
            "a hair under the hold did not wait — this is not the stutter"
        );
        assert_eq!(counter.counted(), 60, "and nothing was taken");
        assert_eq!(counter.read_if_due(1.0), Some(60), "the nominal hold reads");
    }

    #[test]
    fn test_a_rate_over_no_time_is_zero_rather_than_an_infinity() {
        // The one division in the component, and the one place an `f32::INFINITY` reading could come from.
        assert_eq!(rate(60, 1.0), 60.0);
        assert_eq!(rate(30, 0.5), 60.0);
        assert_eq!(rate(0, 1.0), 0.0);
        assert_eq!(rate(60, 0.0), 0.0, "no span, no rate");
        assert_eq!(rate(60, -1.0), 0.0);
        assert_eq!(rate(60, f64::NAN), 0.0);
        assert_eq!(rate(60, f64::INFINITY), 0.0);
        // **A forty-frame second and a second that was not a second.** The rate is a quotient, so the same frame count over
        // a longer span is a lower reading — which is the property that makes it a rate at all.
        assert!(rate(40, 1.0) > rate(40, 2.0));
    }

    #[test]
    fn test_the_figures_are_printed_as_whole_numbers_and_an_absent_one_is_not_a_zero() {
        // A rate quoted to a decimal place is a precision the measurement does not have; and **zero is a measurement while
        // absence is not**, so an unknown figure prints as a dash rather than as `0`.
        assert_eq!(format_fps(Some(59.6)), "60 fps");
        assert_eq!(format_fps(Some(0.4)), "0 fps");
        assert_eq!(format_fps(Some(f32::NAN)), UNKNOWN);
        assert_eq!(format_fps(Some(-1.0)), UNKNOWN);
        // **An unread meter is unknown on both rows**, by the same rule as memory: zero is a measurement and absence is not.
        assert_eq!(format_fps(None), UNKNOWN);
        assert_ne!(format_fps(None), format_fps(Some(0.0)), "absence printed as a measurement");
        assert_eq!(format_bytes(None), UNKNOWN);
        assert_ne!(format_bytes(None), format_bytes(Some(0)), "absence printed as a measurement");
        assert_eq!(format_bytes(Some(0)), "0 B");
        assert_eq!(format_bytes(Some(512)), "512 B");
        // Binary units, because a resident footprint is a byte count and the operating system shows MiB — and **every rung
        // of the ladder is present**, so no measurement is rounded down into a zero.
        assert_eq!(format_bytes(Some(1024)), "1.0 KiB");
        assert_eq!(format_bytes(Some(1536)), "1.5 KiB");
        assert_eq!(format_bytes(Some(1024 * 1024)), "1.0 MiB");
        assert_eq!(format_bytes(Some(3 * 512 * 1024)), "1.5 MiB");
        assert_eq!(format_bytes(Some(1024 * 1024 * 1024)), "1.00 GiB");
        // The boundary: a figure just under a rung stays on the smaller one, and one byte more climbs.
        assert_eq!(format_bytes(Some(1023)), "1023 B");
        assert_eq!(format_bytes(Some(1024)), "1.0 KiB");
        assert_eq!(format_bytes(Some(1024 * 1024 - 1)), "1024.0 KiB");
        assert_eq!(format_bytes(Some(1024 * 1024)), "1.0 MiB");
        // And nothing on the ladder prints a measurement as zero.
        for bytes in [1024u64, 1024 * 1024, 1024 * 1024 * 1024] {
            assert!(
                !format_bytes(Some(bytes)).starts_with('0'),
                "{} printed as zero",
                format_bytes(Some(bytes))
            );
        }
    }

    #[test]
    fn test_a_counter_read_at_a_span_boundary_does_not_lose_or_double_count_a_frame() {
        // **The boundary property**: every counted frame is counted once. Read at any point, the total handed out over a run
        // plus the count still held equals the number of application frames observed.
        let mut counter = Counter::default();
        let mut handed_out = 0u32;
        let mut observed = 0u32;
        for frame in 0..100u64 {
            // Every third frame is the meter's own.
            let own = frame % 3 == 0;
            if counter.observe(frame, own) {
                observed += 1;
            }
            if frame % 7 == 0 {
                handed_out += counter.take();
            }
        }
        handed_out += counter.counted();
        assert_eq!(handed_out, observed, "a frame was lost or counted twice");
        // ...and the exclusion held throughout: two of every three frames counted, within one of it.
        assert!((66..=68).contains(&observed), "the exclusion is off: {observed}");
    }

    #[test]
    fn test_holding_a_reading_does_not_drop_the_frames_that_arrive_during_the_hold() {
        // A hold is not a gap: the frames drawn while the digits stand still are the next reading's frames, which is the whole
        // point of accumulating rather than counting per tick.
        let mut counter = Counter::default();
        counter.observe(0, false);
        for frame in 1..=10u64 {
            counter.observe(frame, false);
        }
        assert_eq!(counter.read_if_due(0.5), None);
        // Ten more frames arrive while it is being held.
        for frame in 11..=20u64 {
            counter.observe(frame, false);
        }
        assert_eq!(counter.read_if_due(1.2), Some(20), "the held frames were dropped");
    }
}
