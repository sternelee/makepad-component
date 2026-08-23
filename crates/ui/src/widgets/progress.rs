use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    mod.widgets.MpProgressBase = #(MpProgress::register_widget(vm))

    set_type_default() do #(DrawProgress::script_shader(vm)){
        ..mod.draw.DrawQuad

        progress: 0.0
        track_color: SURFACE_RAISED
        fill_color: ACCENT

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            let r = sz.y * 0.5

            // Draw track (background capsule)
            sdf.circle(r, r, r)
            sdf.rect(r, 0.0, sz.x - sz.y, sz.y)
            sdf.circle(sz.x - r, r, r)
            sdf.fill(self.track_color)

            // Draw fill using position check
            let fill_end = sz.x * self.progress
            let px = self.pos.x * sz.x

            // If current pixel is within progress range, draw fill color
            let in_fill = step(px, fill_end)

            // Re-draw capsule shape for fill area
            let sdf2 = Sdf2d.viewport(self.pos * self.rect_size)
            sdf2.circle(r, r, r)
            sdf2.rect(r, 0.0, sz.x - sz.y, sz.y)
            sdf2.circle(sz.x - r, r, r)
            sdf2.fill(self.fill_color)

            // Blend based on whether we're in fill region
            let result = mix(sdf.result, sdf2.result, in_fill * sdf2.result.w)
            return result
        }
    }

    // Progress bar component
    mod.widgets.MpProgress = set_type_default() do mod.widgets.MpProgressBase{
        width: Fill
        height: 8.0
    }

    // Progress variants
    mod.widgets.MpProgressSuccess = mod.widgets.MpProgress{
        draw_bg +: { fill_color: SUCCESS }
    }

    mod.widgets.MpProgressWarning = mod.widgets.MpProgress{
        draw_bg +: { fill_color: WARNING }
    }

    mod.widgets.MpProgressDanger = mod.widgets.MpProgress{
        draw_bg +: { fill_color: DANGER }
    }
}

/// Background shader for MpProgress. Custom instance fields are written from
/// Rust directly (the 2.0 replacement for `apply_over`).
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawProgress {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    progress: f32,
    #[live]
    track_color: Vec4f,
    #[live]
    fill_color: Vec4f,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpProgress {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawProgress,

    #[walk]
    walk: Walk,

    #[live(0.0)]
    value: f64,
}

impl Widget for MpProgress {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Set progress by writing the shader instance field directly
        let progress = (self.value / 100.0).clamp(0.0, 1.0);
        self.draw_bg.progress = progress as f32;

        self.draw_bg.draw_walk(cx, walk);
        DrawStep::done()
    }
}

impl MpProgress {
    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        self.value = value.clamp(0.0, 100.0);
        self.redraw(cx);
    }
}

impl MpProgressRef {
    pub fn value(&self) -> f64 {
        if let Some(inner) = self.borrow() {
            inner.value
        } else {
            0.0
        }
    }

    pub fn set_value(&self, cx: &mut Cx, value: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }
}
