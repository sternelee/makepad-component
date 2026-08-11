use makepad_widgets::*;

/// World↔screen camera for the infinite canvas.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    /// World coordinate at the center of the viewport.
    pub pan: Vec2d,
    /// Screen pixels per world pixel.
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pan: Vec2d::default(),
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn world_to_screen(&self, p: Vec2d, viewport: Vec2d) -> Vec2d {
        (p - self.pan) * self.zoom as f64 + viewport * 0.5
    }

    pub fn screen_to_world(&self, p: Vec2d, viewport: Vec2d) -> Vec2d {
        (p - viewport * 0.5) / self.zoom as f64 + self.pan
    }

    pub fn world_rect_to_screen(&self, r: Rect, viewport: Vec2d) -> Rect {
        let pos = self.world_to_screen(r.pos, viewport);
        Rect {
            pos,
            size: r.size * self.zoom as f64,
        }
    }

    /// Zoom around `screen_point`, keeping the world point under the cursor fixed.
    pub fn zoom_at(&mut self, factor: f32, screen_point: Vec2d, viewport: Vec2d) {
        let world_before = self.screen_to_world(screen_point, viewport);
        let new_zoom = (self.zoom * factor).clamp(0.1, 8.0);
        self.zoom = new_zoom;
        let world_after = self.screen_to_world(screen_point, viewport);
        self.pan += world_before - world_after;
    }

    pub fn pan_by(&mut self, delta: Vec2d, viewport: Vec2d) {
        let _ = viewport;
        self.pan -= delta / self.zoom as f64;
    }
}
