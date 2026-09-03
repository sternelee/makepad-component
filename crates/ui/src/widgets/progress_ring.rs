use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    mod.widgets.MpProgressRingBase = #(MpProgressRing::register_widget(vm))
    mod.widgets.MpProgressRing = set_type_default() do mod.widgets.MpProgressRingBase{
        width: 64.0
        height: 64.0

        draw_ring +: {
            track_color: instance(SURFACE_RAISED)
            ring_color: instance(ACCENT)
            progress: instance(0.0)
            thickness: instance(5.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = c.x - self.thickness
                let a0 = -1.5708
                let a1 = a0 + self.progress * 6.2832

                // track
                sdf.circle_arc(c.x, c.y, r, self.thickness, 0.0, 6.2832)
                sdf.fill(self.track_color)

                // progress arc
                if (self.progress > 0.001) {
                    sdf.circle_arc(c.x, c.y, r, self.thickness, a0, a1)
                    sdf.fill(self.ring_color)
                }
                return sdf.result
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpProgressRingAction {
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpProgressRing {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_ring: DrawQuad,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live(0.0)]
    progress: f32,
    #[live]
    ring_color: Vec4f,
    #[live]
    track_color: Vec4f,
    #[live(5.0)]
    thickness: f32,

    /// Five-step size driving the ring diameter (Medium = the DSL 64) and
    /// scaling the stroke thickness proportionally.
    #[live]
    size: MpSize,

    /// The user-declared thickness (captured on first draw) that the size
    /// system scales proportionally.
    #[rust]
    base_thickness: f32,

    /// Last size applied (avoids recomputing every draw).
    #[rust]
    applied_size: Option<MpSize>,

    #[rust]
    area: Area,
}

/// Ring diameter for a size step (Medium = the DSL 64).
fn ring_diameter(size: MpSize) -> f64 {
    match size {
        MpSize::XSmall => 40.0,
        MpSize::Small => 48.0,
        MpSize::Medium => 64.0,
        MpSize::Large => 80.0,
        MpSize::XLarge => 96.0,
    }
}

impl Widget for MpProgressRing {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Capture the user-declared thickness once, then scale it
        // proportionally to the ring diameter.
        if self.base_thickness == 0.0 {
            self.base_thickness = self.thickness;
        }
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            let diameter = ring_diameter(self.size);
            self.thickness = self.base_thickness * (diameter / 64.0) as f32;
        }

        let diameter = ring_diameter(self.size);
        let mut walk = walk;
        walk.width = Size::Fixed(diameter);
        walk.height = Size::Fixed(diameter);

        self.draw_ring.begin(cx, walk, self.layout);
        self.draw_ring.end(cx);
        self.area = self.draw_ring.area();
        DrawStep::done()
    }
}

impl MpProgressRing {
    pub fn set_progress(&mut self, cx: &mut Cx, progress: f32) {
        let p = progress.clamp(0.0, 1.0);
        if (self.progress - p).abs() > 0.001 {
            self.progress = p;
            self.redraw(cx);
        }
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }
}

impl MpProgressRingRef {
    pub fn set_progress(&self, cx: &mut Cx, progress: f32) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_progress(cx, progress);
        }
    }

    pub fn progress(&self) -> f32 {
        if let Some(inner) = self.borrow() {
            inner.progress()
        } else {
            0.0
        }
    }

    pub fn size(&self) -> MpSize {
        if let Some(inner) = self.borrow() {
            inner.size()
        } else {
            MpSize::default()
        }
    }

    pub fn set_size(&self, cx: &mut Cx, size: MpSize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_size(cx, size);
        }
    }
}
