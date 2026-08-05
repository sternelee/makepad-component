//! Holographic point cloud background.
//!
//! Pipeline:
//!   Image → sampled at stride → colored particles retaining original RGB
//!   → physics (mouse repulsion + click shockwave)
//!   → RGBA render: colored point cloud + holographic glow + comet + ripple
//!   → PNG → Makepad Image widget

#![allow(dead_code)]

use image::{DynamicImage, GenericImageView};
use std::collections::VecDeque;
use std::io::Cursor;

// ── Canvas ───────────────────────────────────────────────────────────────────
pub const CANVAS_W: u32 = 480;
pub const CANVAS_H: u32 = 320;

// ── Sampling ─────────────────────────────────────────────────────────────────
/// Sample every N pixels in x and y → controls point cloud density.
const DOT_STEP: usize = 2;
/// Minimum alpha to include a pixel as a particle.
const ALPHA_THRESHOLD: u8 = 30;

// ── Physics ───────────────────────────────────────────────────────────────────
const MOUSE_RADIUS: f32 = 40.0;
const MOUSE_RADIUS_SQ: f32 = MOUSE_RADIUS * MOUSE_RADIUS;
const MOUSE_FORCE_PEAK: f32 = 28.0;
const EASING: f32 = 0.12;
const SNAP: f32 = 0.04;

const SHOCK_SPEED: f32 = 200.0;
const SHOCK_WIDTH: f32 = 28.0;
const SHOCK_STRENGTH: f32 = 24.0;
const SHOCK_DURATION: f64 = 0.65;

// ── Holographic visuals ───────────────────────────────────────────────────────
/// Background: deep near-black
const BG: [u8; 3] = [8, 9, 18];

/// How much to shift colors toward holographic cyan when displaced (0..1)
const HOLO_GLOW_STRENGTH: f32 = 0.85;
/// Scanline darkening factor (0 = no scanlines, 1 = full dark)
const SCANLINE_FACTOR: f32 = 0.18;

const COMET_COL: [u8; 3] = [160, 220, 255];
const RIPPLE_COL: [u8; 3] = [130, 200, 255];
const COMET_MAX: usize = 12;
const COMET_TOTAL_S: f64 = 0.30;

// ── Data types ────────────────────────────────────────────────────────────────
pub struct Particle {
    pub bx: f32,
    pub by: f32, // base position
    pub dx: f32,
    pub dy: f32, // displacement
    pub r: u8,
    pub g: u8,
    pub b: u8, // original image color
}

pub struct Ripple {
    pub cx: f32,
    pub cy: f32,
    pub start: f64,
}

pub struct CometPt {
    pub x: f32,
    pub y: f32,
    pub t: f64,
}

// ── Main struct ───────────────────────────────────────────────────────────────
pub struct ParticleBackground {
    pub particles: Vec<Particle>,
    pub ripples: Vec<Ripple>,
    pub comet: VecDeque<CometPt>,
    pub width: u32,
    pub height: u32,
    pub mx: f32,
    pub my: f32,
    pub mouse_in: bool,
}

impl ParticleBackground {
    /// Build a colored point cloud from the source image.
    /// Samples every DOT_STEP pixels, retaining original RGB colors.
    pub fn from_image(img: &DynamicImage) -> Self {
        let (iw, ih) = img.dimensions();

        // Fit image into canvas (letterbox)
        let scale = (CANVAS_W as f32 / iw as f32).min(CANVAS_H as f32 / ih as f32);
        let fw = (iw as f32 * scale) as u32;
        let fh = (ih as f32 * scale) as u32;
        let ox = (CANVAS_W.saturating_sub(fw)) / 2;
        let oy = (CANVAS_H.saturating_sub(fh)) / 2;

        // Resize to canvas-fitted dimensions
        let resized = img.resize_exact(fw, fh, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();

        let mut particles = Vec::new();
        for sy in (0..fh as usize).step_by(DOT_STEP) {
            for sx in (0..fw as usize).step_by(DOT_STEP) {
                let p = rgba.get_pixel(sx as u32, sy as u32);
                if p[3] < ALPHA_THRESHOLD {
                    continue;
                }
                particles.push(Particle {
                    bx: (ox as usize + sx) as f32,
                    by: (oy as usize + sy) as f32,
                    dx: 0.0,
                    dy: 0.0,
                    r: p[0],
                    g: p[1],
                    b: p[2],
                });
            }
        }

        log::info!(
            "ParticleBackground: {} colored particles from {}×{} source",
            particles.len(),
            iw,
            ih
        );

        Self {
            particles,
            ripples: Vec::new(),
            comet: VecDeque::new(),
            width: CANVAS_W,
            height: CANVAS_H,
            mx: 0.0,
            my: 0.0,
            mouse_in: false,
        }
    }

    pub fn set_mouse(&mut self, x: f32, y: f32, inside: bool) {
        self.mx = x;
        self.my = y;
        self.mouse_in = inside;
    }

    pub fn push_comet(&mut self, x: f32, y: f32, now: f64) {
        if let Some(last) = self.comet.back() {
            let dx = x - last.x;
            let dy = y - last.y;
            if dx * dx + dy * dy < 4.0 {
                return;
            }
        }
        self.comet.push_back(CometPt { x, y, t: now });
        while self.comet.len() > COMET_MAX {
            self.comet.pop_front();
        }
    }

    pub fn add_ripple(&mut self, x: f32, y: f32, now: f64) {
        self.ripples.push(Ripple {
            cx: x,
            cy: y,
            start: now,
        });
    }

    /// Physics step. Returns true when redraw is needed.
    pub fn update(&mut self, now: f64) -> bool {
        self.ripples.retain(|r| now - r.start < SHOCK_DURATION);
        // Auto-expire comet points
        self.comet.retain(|p| now - p.t < COMET_TOTAL_S);

        let mut any = self.mouse_in || !self.ripples.is_empty() || !self.comet.is_empty();

        for p in &mut self.particles {
            let mut fx = 0.0_f32;
            let mut fy = 0.0_f32;

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

            for r in &self.ripples {
                let elapsed = (now - r.start) as f32;
                let radius = elapsed * SHOCK_SPEED;
                let life = 1.0 - elapsed as f64 / SHOCK_DURATION;
                let sx = p.bx - r.cx;
                let sy = p.by - r.cy;
                let d = (sx * sx + sy * sy).sqrt();
                if d > 0.1 {
                    let band = (d - radius).abs();
                    if band < SHOCK_WIDTH {
                        let f = (1.0 - band / SHOCK_WIDTH) * life as f32 * SHOCK_STRENGTH;
                        fx += (sx / d) * f;
                        fy += (sy / d) * f;
                    }
                }
            }

            p.dx += (fx - p.dx) * EASING;
            p.dy += (fy - p.dy) * EASING;
            if p.dx.abs() < SNAP {
                p.dx = 0.0;
            }
            if p.dy.abs() < SNAP {
                p.dy = 0.0;
            }
            if p.dx != 0.0 || p.dy != 0.0 {
                any = true;
            }
        }
        any
    }

    pub fn render_png(&self, now: f64) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut px = vec![0u8; w * h * 4];

        // Background
        for i in 0..w * h {
            px[i * 4] = BG[0];
            px[i * 4 + 1] = BG[1];
            px[i * 4 + 2] = BG[2];
            px[i * 4 + 3] = 255;
        }

        // ── Colored point cloud (bottom layer) ───────────────────────────
        for p in &self.particles {
            let x = p.bx + p.dx;
            let y = p.by + p.dy;

            let disp = (p.dx * p.dx + p.dy * p.dy).sqrt();
            let t = (disp / 18.0).min(1.0); // 0 = resting, 1 = fully displaced

            // Resting color: original image RGB with a very subtle cold tint
            // (+8 on B channel to hint holographic without washing color out)
            let base_r = p.r;
            let base_g = p.g;
            let base_b = p.b.saturating_add(8);

            // Displaced color: lerp toward bright cyan-white
            let glow_r = 200u8;
            let glow_g = 235u8;
            let glow_b = 255u8;

            let r = lerp_u8(base_r, glow_r, t * HOLO_GLOW_STRENGTH);
            let g = lerp_u8(base_g, glow_g, t * HOLO_GLOW_STRENGTH);
            let b = lerp_u8(base_b, glow_b, t * HOLO_GLOW_STRENGTH);

            // Scanline: every 3rd row is slightly dimmer for holographic feel
            let scanline = if (y as usize) % 3 == 0 {
                1.0 - SCANLINE_FACTOR
            } else {
                1.0
            };
            let r = (r as f32 * scanline) as u8;
            let g = (g as f32 * scanline) as u8;
            let b = (b as f32 * scanline) as u8;

            // Brightness boost for displaced particles (they "emit" light)
            let alpha = if t > 0.05 { 1.0 } else { 0.92 };
            draw_dot(&mut px, w, h, x, y, [r, g, b], alpha);

            // Extra soft halo around strongly displaced particles
            if t > 0.35 {
                let halo_a = (t - 0.35) * 0.4;
                draw_soft_circle(&mut px, w, h, x, y, 3.5, [glow_r, glow_g, glow_b], halo_a);
            }
        }

        // ── Ripple rings ─────────────────────────────────────────────────
        for r in &self.ripples {
            let elapsed = now - r.start;
            if elapsed >= SHOCK_DURATION {
                continue;
            }
            let life = (1.0 - elapsed / SHOCK_DURATION) as f32;
            let radius = elapsed as f32 * SHOCK_SPEED;
            draw_ring(
                &mut px,
                w,
                h,
                r.cx,
                r.cy,
                radius,
                2.5,
                RIPPLE_COL,
                life * life * 0.80,
            );
            if radius > 20.0 {
                draw_ring(
                    &mut px,
                    w,
                    h,
                    r.cx,
                    r.cy,
                    radius * 0.55,
                    1.5,
                    RIPPLE_COL,
                    life * 0.32,
                );
            }
        }

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

fn blend(buf: &mut [u8], w: usize, x: i32, y: i32, r: u8, g: u8, b: u8, a: f32) {
    let h = buf.len() / (w * 4);
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return;
    }
    let i = (y as usize * w + x as usize) * 4;
    let a = a.clamp(0.0, 1.0);
    buf[i] = lerp_u8(buf[i], r, a);
    buf[i + 1] = lerp_u8(buf[i + 1], g, a);
    buf[i + 2] = lerp_u8(buf[i + 2], b, a);
}

/// 1×1 single pixel dot — fine point cloud.
fn draw_dot(buf: &mut [u8], w: usize, h: usize, x: f32, y: f32, c: [u8; 3], a: f32) {
    let _ = h;
    blend(buf, w, x as i32, y as i32, c[0], c[1], c[2], a);
}

/// Soft radial glow (comet head / displaced halo).
fn draw_soft_circle(
    buf: &mut [u8],
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    r: f32,
    c: [u8; 3],
    a: f32,
) {
    let _ = h;
    let margin = (r + 2.0) as i32;
    for dy in -margin..=margin {
        for dx in -margin..=margin {
            let dist = (dx as f32).hypot(dy as f32);
            let cov = ((r + 1.5 - dist) / 1.5).clamp(0.0, 1.0);
            if cov > 0.01 {
                blend(
                    buf,
                    w,
                    cx as i32 + dx,
                    cy as i32 + dy,
                    c[0],
                    c[1],
                    c[2],
                    a * cov,
                );
            }
        }
    }
}

/// Anti-aliased thick line (perpendicular-slice stepping).
fn draw_line(
    buf: &mut [u8],
    w: usize,
    h: usize,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    width: f32,
    c: [u8; 3],
    a: f32,
) {
    let _ = h;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let nx = -dy / len;
    let ny = dx / len;
    let hw = width * 0.5;
    let steps = (len * 1.5) as usize + 1;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let mx = x0 + dx * t;
        let my = y0 + dy * t;
        for j in -3_i32..=3 {
            let off = j as f32 * hw / 3.0;
            let cov = ((hw - off.abs() + 0.5) / 1.0).clamp(0.0, 1.0);
            blend(
                buf,
                w,
                (mx + nx * off) as i32,
                (my + ny * off) as i32,
                c[0],
                c[1],
                c[2],
                a * cov,
            );
        }
    }
}

/// Anti-aliased ring.
fn draw_ring(
    buf: &mut [u8],
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    radius: f32,
    width: f32,
    c: [u8; 3],
    a: f32,
) {
    let _ = h;
    if radius <= 0.0 || a <= 0.01 {
        return;
    }
    let margin = (radius + width + 2.0) as i32;
    let cxi = cx as i32;
    let cyi = cy as i32;
    for dy in -margin..=margin {
        for dx in -margin..=margin {
            let d = (dx as f32).hypot(dy as f32);
            let band = (d - radius).abs();
            if band < width + 1.5 {
                let cov = ((width + 1.0 - band) / 1.5).clamp(0.0, 1.0);
                blend(buf, w, cxi + dx, cyi + dy, c[0], c[1], c[2], a * cov);
            }
        }
    }
}
