//! Particle dithering background with interactive mouse/click effects.
//!
//! Pipeline:
//!   Image → Floyd-Steinberg dithering → particle positions (x,y Float32 grid)
//!   → physics update (mouse repulsion + click shockwave)
//!   → RGBA pixel render (particles + comet trail + ripple rings)
//!   → PNG upload to Makepad Image widget

#![allow(dead_code)]

use image::{DynamicImage, GenericImageView};
use std::collections::VecDeque;
use std::io::Cursor;

// ── Canvas ──────────────────────────────────────────────────────────────────
/// Virtual canvas dimensions. Image widget scales this to fill scene_area.
pub const CANVAS_W: u32 = 480;
pub const CANVAS_H: u32 = 320;

// ── Physics ─────────────────────────────────────────────────────────────────
const DOT_STEP: usize = 3;       // dither sample stride (pixels)
const MOUSE_RADIUS: f32 = 72.0;
const MOUSE_RADIUS_SQ: f32 = MOUSE_RADIUS * MOUSE_RADIUS;
const MOUSE_FORCE_PEAK: f32 = 34.0;
const EASING: f32 = 0.13;
const SNAP: f32 = 0.05;

// Shockwave (click ripple)
const SHOCK_SPEED: f32 = 210.0;   // px/s
const SHOCK_WIDTH: f32 = 26.0;
const SHOCK_STRENGTH: f32 = 22.0;
const SHOCK_DURATION: f64 = 0.65; // seconds

// ── Visuals ──────────────────────────────────────────────────────────────────
const BG: [u8; 3] = [10, 11, 22];           // deep blue-black background
const DOT_COL: [u8; 3] = [115, 150, 210];   // base particle colour
const DOT_GLOW: [u8; 3] = [200, 225, 255];  // displaced particle glow
const COMET_COL: [u8; 3] = [170, 205, 255]; // comet trail colour
const RIPPLE_COL: [u8; 3] = [145, 190, 255];// ripple ring colour

const COMET_MAX: usize = 30;     // max stored trail positions
const COMET_LIFE_S: f64 = 0.07;  // seconds between positions before fade

// ── Data types ───────────────────────────────────────────────────────────────
pub struct Particle {
    pub bx: f32, pub by: f32, // base (resting) position
    pub dx: f32, pub dy: f32, // displacement from base
}

pub struct Ripple {
    pub cx: f32, pub cy: f32, // click origin in canvas coords
    pub start: f64,           // timestamp (secs)
}

pub struct CometPt {
    pub x: f32, pub y: f32,
    pub t: f64, // timestamp
}

// ── Main system ──────────────────────────────────────────────────────────────
pub struct ParticleBackground {
    pub particles: Vec<Particle>,
    pub ripples:   Vec<Ripple>,
    pub comet:     VecDeque<CometPt>,
    pub width:     u32,
    pub height:    u32,

    // mouse state (canvas coords)
    pub mx: f32, pub my: f32,
    pub mouse_in: bool,
}

impl ParticleBackground {
    /// Build from a source image using Floyd-Steinberg dithering.
    /// The image is letterboxed into CANVAS_W × CANVAS_H.
    pub fn from_image(img: &DynamicImage) -> Self {
        let (iw, ih) = img.dimensions();

        // Fit image into canvas (letterbox)
        let scale = (CANVAS_W as f32 / iw as f32).min(CANVAS_H as f32 / ih as f32);
        let fw = (iw as f32 * scale) as u32;
        let fh = (ih as f32 * scale) as u32;
        let ox = (CANVAS_W.saturating_sub(fw)) / 2;
        let oy = (CANVAS_H.saturating_sub(fh)) / 2;

        // Resize and grayscale
        let resized = img.resize_exact(fw, fh, image::imageops::FilterType::Lanczos3);
        let gray = resized.to_luma8();
        let gw = fw as usize;
        let gh = fh as usize;

        // Floyd-Steinberg error diffusion (in-place)
        let mut buf: Vec<f32> = gray.pixels().map(|p| p[0] as f32).collect();
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let old = buf[i];
                let new = if old > 128.0 { 255.0 } else { 0.0 };
                let err = old - new;
                buf[i] = new;
                let spread = |b: &mut Vec<f32>, nx: isize, ny: isize, w: f32| {
                    if nx >= 0 && nx < gw as isize && ny >= 0 && ny < gh as isize {
                        let j = ny as usize * gw + nx as usize;
                        b[j] = (b[j] + err * w).clamp(0.0, 255.0);
                    }
                };
                let xi = x as isize;
                let yi = y as isize;
                spread(&mut buf, xi + 1, yi,     7.0 / 16.0);
                spread(&mut buf, xi - 1, yi + 1, 3.0 / 16.0);
                spread(&mut buf, xi,     yi + 1, 5.0 / 16.0);
                spread(&mut buf, xi + 1, yi + 1, 1.0 / 16.0);
            }
        }

        // Collect "on" pixels, sampled at DOT_STEP stride
        let mut particles = Vec::new();
        for sy in (0..gh).step_by(DOT_STEP) {
            for sx in (0..gw).step_by(DOT_STEP) {
                if buf[sy * gw + sx] > 128.0 {
                    particles.push(Particle {
                        bx: (ox as usize + sx) as f32,
                        by: (oy as usize + sy) as f32,
                        dx: 0.0, dy: 0.0,
                    });
                }
            }
        }

        log::info!(
            "ParticleBackground: {} particles from {}×{} source",
            particles.len(), iw, ih
        );

        Self {
            particles,
            ripples: Vec::new(),
            comet:   VecDeque::new(),
            width:   CANVAS_W,
            height:  CANVAS_H,
            mx: 0.0, my: 0.0,
            mouse_in: false,
        }
    }

    /// Update mouse position (in canvas coords).
    pub fn set_mouse(&mut self, x: f32, y: f32, inside: bool) {
        self.mx = x; self.my = y; self.mouse_in = inside;
    }

    /// Record a comet trail point. Skips if mouse barely moved.
    pub fn push_comet(&mut self, x: f32, y: f32, now: f64) {
        if let Some(last) = self.comet.back() {
            let dx = x - last.x;
            let dy = y - last.y;
            if dx * dx + dy * dy < 4.0 { return; }
        }
        self.comet.push_back(CometPt { x, y, t: now });
        while self.comet.len() > COMET_MAX { self.comet.pop_front(); }
    }

    /// Add a click shockwave at canvas coords.
    pub fn add_ripple(&mut self, x: f32, y: f32, now: f64) {
        self.ripples.push(Ripple { cx: x, cy: y, start: now });
    }

    /// Physics step. Returns true when any motion is still happening.
    pub fn update(&mut self, now: f64) -> bool {
        // Expire finished ripples
        self.ripples.retain(|r| now - r.start < SHOCK_DURATION);

        let mut any = self.mouse_in || !self.ripples.is_empty() || !self.comet.is_empty();

        for p in &mut self.particles {
            let mut fx = 0.0_f32;
            let mut fy = 0.0_f32;

            // Mouse repulsion (cubic falloff)
            if self.mouse_in {
                let vx = (p.bx + p.dx) - self.mx;
                let vy = (p.by + p.dy) - self.my;
                let d2 = vx * vx + vy * vy;
                if d2 > 0.1 && d2 < MOUSE_RADIUS_SQ {
                    let d = d2.sqrt();
                    let f = (1.0 - d / MOUSE_RADIUS).powi(3) * MOUSE_FORCE_PEAK;
                    fx += (vx / d) * f;
                    fy += (vy / d) * f;
                }
            }

            // Shockwave rings (expanding ripple from click)
            for r in &self.ripples {
                let elapsed = (now - r.start) as f32;
                let radius  = elapsed * SHOCK_SPEED;
                let life    = 1.0 - elapsed as f64 / SHOCK_DURATION;
                let sx = p.bx - r.cx;
                let sy = p.by - r.cy;
                let d  = (sx * sx + sy * sy).sqrt();
                if d > 0.1 {
                    let band = (d - radius).abs();
                    if band < SHOCK_WIDTH {
                        let f = (1.0 - band / SHOCK_WIDTH) * life as f32 * SHOCK_STRENGTH;
                        fx += (sx / d) * f;
                        fy += (sy / d) * f;
                    }
                }
            }

            // Spring easing back to base position
            p.dx += (fx - p.dx) * EASING;
            p.dy += (fy - p.dy) * EASING;
            if p.dx.abs() < SNAP { p.dx = 0.0; }
            if p.dy.abs() < SNAP { p.dy = 0.0; }
            if p.dx != 0.0 || p.dy != 0.0 { any = true; }
        }
        any
    }

    /// Render the current frame to PNG bytes (ready for Makepad Image upload).
    pub fn render_png(&self, now: f64) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut px = vec![0u8; w * h * 4];

        // Background fill
        for i in 0..w * h {
            px[i * 4]     = BG[0];
            px[i * 4 + 1] = BG[1];
            px[i * 4 + 2] = BG[2];
            px[i * 4 + 3] = 255;
        }

        // ── Comet trail (oldest → newest, fading in) ──────────────────────
        let pts: Vec<_> = self.comet.iter().collect();
        let n = pts.len();
        if n >= 2 {
            let t_newest = pts[n - 1].t;
            let window = COMET_LIFE_S * COMET_MAX as f64;
            for i in 0..n - 1 {
                // Normalised age: 0 = oldest visible, 1 = newest
                let age = ((pts[i + 1].t - t_newest + window) / window).clamp(0.0, 1.0) as f32;
                let alpha = age.powf(1.8) * 0.9;
                if alpha > 0.01 {
                    draw_line(&mut px, w, h,
                        pts[i].x, pts[i].y, pts[i + 1].x, pts[i + 1].y,
                        2.2, COMET_COL, alpha);
                }
            }
            // Glowing head at the newest point
            if let Some(head) = pts.last() {
                draw_soft_circle(&mut px, w, h, head.x, head.y, 4.5, COMET_COL, 1.0);
                draw_soft_circle(&mut px, w, h, head.x, head.y, 2.0, [220, 240, 255], 1.0);
            }
        }

        // ── Ripple rings ──────────────────────────────────────────────────
        for r in &self.ripples {
            let elapsed = now - r.start;
            if elapsed >= SHOCK_DURATION { continue; }
            let life   = (1.0 - elapsed / SHOCK_DURATION) as f32;
            let radius = elapsed as f32 * SHOCK_SPEED;
            // Outer ring
            draw_ring(&mut px, w, h, r.cx, r.cy, radius, 2.5, RIPPLE_COL, life * life * 0.75);
            // Inner secondary ring (smaller, more faded)
            if radius > 20.0 {
                draw_ring(&mut px, w, h, r.cx, r.cy, radius * 0.55, 1.5, RIPPLE_COL, life * 0.35);
            }
        }

        // ── Particles ─────────────────────────────────────────────────────
        for p in &self.particles {
            let x = p.bx + p.dx;
            let y = p.by + p.dy;
            let disp  = (p.dx * p.dx + p.dy * p.dy).sqrt();
            let glow  = (disp / 14.0).min(1.0);
            let r = lerp_u8(DOT_COL[0], DOT_GLOW[0], glow);
            let g = lerp_u8(DOT_COL[1], DOT_GLOW[1], glow);
            let b = lerp_u8(DOT_COL[2], DOT_GLOW[2], glow);
            draw_dot(&mut px, w, h, x, y, [r, g, b], 0.95);
        }

        // ── Encode PNG ───────────────────────────────────────────────────
        let rgba = image::RgbaImage::from_raw(self.width, self.height, px).unwrap();
        let dyn_img = image::DynamicImage::ImageRgba8(rgba);
        let mut out = Vec::new();
        dyn_img
            .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }
}

// ── Drawing primitives ────────────────────────────────────────────────────────

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

/// Alpha-blend a single pixel into the RGBA buffer.
fn blend(buf: &mut [u8], w: usize, x: i32, y: i32, r: u8, g: u8, b: u8, a: f32) {
    let h = buf.len() / (w * 4);
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 { return; }
    let i = (y as usize * w + x as usize) * 4;
    let a = a.clamp(0.0, 1.0);
    buf[i]     = lerp_u8(buf[i],     r, a);
    buf[i + 1] = lerp_u8(buf[i + 1], g, a);
    buf[i + 2] = lerp_u8(buf[i + 2], b, a);
}

/// 2×2 dot with subtle soft edge.
fn draw_dot(buf: &mut [u8], w: usize, h: usize, x: f32, y: f32, c: [u8; 3], a: f32) {
    let _ = h;
    let xi = x as i32;
    let yi = y as i32;
    blend(buf, w, xi,     yi,     c[0], c[1], c[2], a);
    blend(buf, w, xi + 1, yi,     c[0], c[1], c[2], a * 0.75);
    blend(buf, w, xi,     yi + 1, c[0], c[1], c[2], a * 0.75);
    blend(buf, w, xi + 1, yi + 1, c[0], c[1], c[2], a * 0.55);
}

/// Soft radial glow circle (used for comet head).
fn draw_soft_circle(buf: &mut [u8], w: usize, h: usize, cx: f32, cy: f32, r: f32, c: [u8; 3], a: f32) {
    let _ = h;
    let margin = (r + 2.0) as i32;
    for dy in -margin..=margin {
        for dx in -margin..=margin {
            let dist = (dx as f32).hypot(dy as f32);
            let coverage = ((r + 1.5 - dist) / 1.5).clamp(0.0, 1.0);
            if coverage > 0.01 {
                blend(buf, w, cx as i32 + dx, cy as i32 + dy, c[0], c[1], c[2], a * coverage);
            }
        }
    }
}

/// Anti-aliased thick line (perpendicular slice stepping).
fn draw_line(
    buf: &mut [u8], w: usize, h: usize,
    x0: f32, y0: f32, x1: f32, y1: f32,
    width: f32, c: [u8; 3], a: f32,
) {
    let _ = h;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 { return; }
    // Perpendicular unit vector
    let nx = -dy / len;
    let ny =  dx / len;
    let hw = width * 0.5;
    let steps = (len * 1.5) as usize + 1;
    for i in 0..=steps {
        let t  = i as f32 / steps as f32;
        let mx = x0 + dx * t;
        let my = y0 + dy * t;
        // Sample across the width with 7 sub-pixel positions
        for j in -3_i32..=3 {
            let off = j as f32 * hw / 3.0;
            let cov = ((hw - off.abs() + 0.5) / 1.0).clamp(0.0, 1.0);
            blend(buf, w, (mx + nx * off) as i32, (my + ny * off) as i32,
                  c[0], c[1], c[2], a * cov);
        }
    }
}

/// Anti-aliased ring (expanding circle shell).
fn draw_ring(
    buf: &mut [u8], w: usize, h: usize,
    cx: f32, cy: f32, radius: f32, width: f32, c: [u8; 3], a: f32,
) {
    let _ = h;
    if radius <= 0.0 || a <= 0.01 { return; }
    let margin = (radius + width + 2.0) as i32;
    let cxi = cx as i32;
    let cyi = cy as i32;
    for dy in -margin..=margin {
        for dx in -margin..=margin {
            let d    = (dx as f32).hypot(dy as f32);
            let band = (d - radius).abs();
            if band < width + 1.5 {
                let cov = ((width + 1.0 - band) / 1.5).clamp(0.0, 1.0);
                blend(buf, w, cxi + dx, cyi + dy, c[0], c[1], c[2], a * cov);
            }
        }
    }
}
