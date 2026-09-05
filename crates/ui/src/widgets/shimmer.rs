use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpShimmer - loading shimmer (gpui ShimmerStyle port).
    // The highlight band sweeps across in a pixel shader; the band
    // center is a draw instance animated from Rust via NextFrame.
    // ============================================================

    // Text-family custom draw: registers the DrawText shader with a
    // per-pixel `get_color` so the band can tint glyphs while sweeping.
    set_type_default() do #(DrawMpShimmerText::script_shader(vm)){
        ..mod.draw.DrawText
    }

    // Skeleton block: rounded rect with a moving highlight band.
    mod.widgets.MpShimmerBase = #(MpShimmer::register_widget(vm))
    mod.widgets.MpShimmer = set_type_default() do mod.widgets.MpShimmerBase{
        width: Fill
        height: 16

        set_type_default() do #(DrawMpShimmer::script_shader(vm)){
            ..mod.draw.DrawQuad

            color: INPUT_BG
            highlight_color: ELEMENT_ACTIVE
            radius: 8.0
            shimmer_pos: -0.4
            spread: 0.3

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size
                sdf.box(0.0, 0.0, sz.x, sz.y, self.radius)
                let d = abs(self.pos.x - self.shimmer_pos)
                let band = 1.0 - smoothstep(0.0, self.spread, d)
                sdf.fill(mix(self.color, self.highlight_color, band))
                return sdf.result
            }
        }
    }

    // Text with a sweeping highlight (e.g. "Thinking..." placeholders).
    mod.widgets.MpShimmerTextBase = #(MpShimmerText::register_widget(vm))
    mod.widgets.MpShimmerText = set_type_default() do mod.widgets.MpShimmerTextBase{
        width: Fit
        height: Fit

        draw_text +: {
            text_style: theme.font_regular{font_size: 14.0}
            color: TEXT_MUTED
            highlight_color: TEXT
            shimmer_pos: -0.4
            spread: 0.3

            get_color: fn() {
                let d = abs(self.pos.x - self.shimmer_pos)
                let band = 1.0 - smoothstep(0.0, self.spread, d)
                return mix(self.color, self.highlight_color, band)
            }
        }

        text: ""
    }
}

/// Map normalized animation phase to the band center so the highlight
/// starts fully left of the rect and exits fully right (mirrored when
/// reversed).
fn shimmer_band_center(phase: f64, reverse: bool) -> f32 {
    let t = if reverse {
        1.0 - phase.clamp(0.0, 1.0)
    } else {
        phase.clamp(0.0, 1.0)
    };
    (-0.3 + t * 1.6) as f32
}

// Shimmer block surface: base color + animated highlight band. The custom
// draw struct exposes `shimmer_pos` to Rust so the NextFrame loop can move
// the band; without `script_shader` registration the struct falls back to
// DrawQuad's default transparent pixel.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpShimmer {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    highlight_color: Vec4f,
    #[live]
    radius: f32,
    #[live]
    shimmer_pos: f32,
    #[live]
    spread: f32,
}

// Shimmer text surface: DrawText shader with a per-pixel `get_color` that
// mixes in the highlight band while it sweeps across the glyphs.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpShimmerText {
    #[deref]
    draw_super: DrawText,
    #[live]
    highlight_color: Vec4f,
    #[live]
    shimmer_pos: f32,
    #[live]
    spread: f32,
}

/// Skeleton block with a sweeping highlight. Auto-plays on first draw;
/// `once: true` plays a single sweep and parks the band off-edge.
#[derive(Script, ScriptHook, Widget)]
pub struct MpShimmer {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpShimmer,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// Duration of one full sweep, in seconds.
    #[live(2.0)]
    duration: f64,

    /// Sweep right-to-left instead of left-to-right.
    #[live]
    reverse: bool,

    /// Play a single sweep instead of looping.
    #[live]
    once: bool,

    /// Start animating automatically on first draw.
    #[live(true)]
    auto_play: bool,

    #[rust]
    started: bool,
    #[rust]
    phase: f64,
    #[rust]
    last_time: Option<f64>,
    #[rust]
    next_frame: NextFrame,
    #[rust]
    area: Area,
}

impl Widget for MpShimmer {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Some(nf) = self.next_frame.is_event(event) {
            let dt = self.last_time.map(|t| (nf.time - t).max(0.0)).unwrap_or(0.0);
            self.last_time = Some(nf.time);
            self.phase += dt / self.duration.max(0.001);
            let finished = self.once && self.phase >= 1.0;
            if finished {
                self.phase = 1.0;
            } else if self.phase > 1.0 {
                self.phase -= 1.0;
            }
            self.draw_bg.shimmer_pos = shimmer_band_center(self.phase, self.reverse);
            self.redraw(cx);
            if finished {
                self.started = false;
                self.last_time = None;
                self.phase = 0.0;
            } else {
                self.next_frame = cx.new_next_frame();
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.auto_play && !self.started {
            self.started = true;
            self.phase = 0.0;
            self.last_time = None;
            self.next_frame = cx.new_next_frame();
        }
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpShimmer {
    pub fn area(&self) -> Area {
        self.area
    }
}

/// Text with a sweeping highlight ("Thinking..." placeholders).
#[derive(Script, ScriptHook, Widget)]
pub struct MpShimmerText {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_text: DrawMpShimmerText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// The shimmering text.
    #[live]
    text: ArcStringMut,

    /// Duration of one full sweep, in seconds.
    #[live(2.0)]
    duration: f64,

    /// Sweep right-to-left instead of left-to-right.
    #[live]
    reverse: bool,

    /// Play a single sweep instead of looping.
    #[live]
    once: bool,

    /// Start animating automatically on first draw.
    #[live(true)]
    auto_play: bool,

    #[rust]
    started: bool,
    #[rust]
    phase: f64,
    #[rust]
    last_time: Option<f64>,
    #[rust]
    next_frame: NextFrame,
    #[rust]
    area: Area,
}

impl Widget for MpShimmerText {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Some(nf) = self.next_frame.is_event(event) {
            let dt = self.last_time.map(|t| (nf.time - t).max(0.0)).unwrap_or(0.0);
            self.last_time = Some(nf.time);
            self.phase += dt / self.duration.max(0.001);
            let finished = self.once && self.phase >= 1.0;
            if finished {
                self.phase = 1.0;
            } else if self.phase > 1.0 {
                self.phase -= 1.0;
            }
            self.draw_text.shimmer_pos = shimmer_band_center(self.phase, self.reverse);
            self.redraw(cx);
            if finished {
                self.started = false;
                self.last_time = None;
                self.phase = 0.0;
            } else {
                self.next_frame = cx.new_next_frame();
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.auto_play && !self.started {
            self.started = true;
            self.phase = 0.0;
            self.last_time = None;
            self.next_frame = cx.new_next_frame();
        }
        self.draw_text
            .draw_walk(cx, walk, Align::default(), self.text.as_ref());
        self.area = self.draw_text.area();
        DrawStep::done()
    }
}

impl MpShimmerText {
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().clear();
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }

    pub fn area(&self) -> Area {
        self.area
    }
}

impl MpShimmerTextRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }
}
