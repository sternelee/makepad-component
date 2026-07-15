use crate::a2ui::message::UserAction;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*

    DrawA2uiImage: mod.std.set_type_default() do #(DrawA2uiImage::script_shader(vm)){
        ..mod.draw.DrawQuad
        image: texture_2d(float)
        border_radius: instance(4.0)

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size);
            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.border_radius);
            let img_color = self.image.sample_as_bgra(self.pos);
            sdf.fill(img_color);
            return sdf.result;
        }
    }

    DrawA2uiChartLine: mod.std.set_type_default() do #(DrawA2uiChartLine::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let uv = self.pos;
            let p1 = vec2(self.x1, self.y1);
            let p2 = vec2(self.x2, self.y2);
            let p = uv;
            let line_vec = p2 - p1;
            let line_len = length(line_vec);
            if line_len < 0.001 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }
            let t = clamp(dot(p - p1, line_vec) / (line_len * line_len), 0.0, 1.0);
            let closest = p1 + t * line_vec;
            let dist = length(p - closest);
            let half_width = self.line_width * 0.5;
            let aa = 0.02;
            let alpha = 1.0 - smoothstep(half_width - aa, half_width + aa, dist);
            if alpha < 0.01 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }
            return vec4(self.color.rgb * self.color.a * alpha, self.color.a * alpha);
        }
    }

    DrawA2uiArc: mod.std.set_type_default() do #(DrawA2uiArc::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let two_pi_val = 6.28318530;
            let px = self.pos.x - 0.5;
            let py = self.pos.y - 0.5;
            let distance = sqrt(px * px + py * py);
            let inner_rad = self.inner_radius * 0.5;
            let outer_rad = 0.5;
            let dist_mask = step(inner_rad, distance) * step(distance, outer_rad);
            let pixel_ang = atan2(py, px);
            let sweep_val = self.end_angle - self.start_angle;
            let rel_ang = pixel_ang - self.start_angle;
            let norm_ang = rel_ang + two_pi_val * 4.0;
            let wrap_ang = norm_ang - two_pi_val * floor(norm_ang / two_pi_val);
            let ang_mask = step(wrap_ang, sweep_val) * step(0.001, sweep_val);
            let final_mask = dist_mask * ang_mask;
            let edge_aa = 0.008;
            let outer_aa = 1.0 - smoothstep(outer_rad - edge_aa, outer_rad + edge_aa, distance);
            let inner_aa = smoothstep(inner_rad - edge_aa, inner_rad + edge_aa, distance);
            let aa_alpha = outer_aa * inner_aa;
            let alpha_val = final_mask * aa_alpha;
            return vec4(self.color.rgb * alpha_val, self.color.a * alpha_val);
        }
    }

    DrawA2uiQuad: mod.std.set_type_default() do #(DrawA2uiQuad::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let p = self.pos * self.rect_size;
            let p0 = vec2(self.p0x, self.p0y);
            let p1 = vec2(self.p1x, self.p1y);
            let p2 = vec2(self.p2x, self.p2y);
            let p3 = vec2(self.p3x, self.p3y);
            let e0 = p1 - p0; let v0 = p - p0;
            let e1 = p2 - p1; let v1 = p - p1;
            let e2 = p3 - p2; let v2 = p - p2;
            let e3 = p0 - p3; let v3 = p - p3;
            let c0 = e0.x * v0.y - e0.y * v0.x;
            let c1 = e1.x * v1.y - e1.y * v1.x;
            let c2 = e2.x * v2.y - e2.y * v2.x;
            let c3 = e3.x * v3.y - e3.y * v3.x;
            let all_pos = step(0.0, c0) * step(0.0, c1) * step(0.0, c2) * step(0.0, c3);
            let all_neg = step(c0, 0.0) * step(c1, 0.0) * step(c2, 0.0) * step(c3, 0.0);
            let inside = min(all_pos + all_neg, 1.0);
            if inside < 0.5 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }
            let len0 = max(length(e0), 0.001);
            let len2 = max(length(e2), 0.001);
            let d0 = abs(c0) / len0;
            let d2 = abs(c2) / len2;
            let min_dist = min(d0, d2);
            let aa = smoothstep(0.0, 1.5, min_dist);
            let alpha = aa * self.opacity;
            return vec4(self.color.rgb * self.color.a * alpha, self.color.a * alpha);
        }
    }

    DrawAudioBars: mod.std.set_type_default() do #(DrawAudioBars::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size);

            // ── Fractal rainbow background (code golf shader) ──
            // Speed modulated by audio: idle=slow, playing=fast, amplitude boosts energy
            let amp = clamp(self.amplitude * 3.0, 0.0, 1.0);
            let use_real_amp = step(0.001, self.amplitude);
            let speed = mix(0.6, mix(1.5, 1.5 + amp * 2.0, use_real_amp), self.is_playing);
            let t = self.draw_pass.time * speed;
            let ar = self.rect_size.x / max(self.rect_size.y, 1.0);

            // Fractal: (FC.xy*2.-r)/r.y/.3 with amplitude-reactive zoom
            let zoom = mix(0.3, 0.25 + amp * 0.15, self.is_playing);
            let p = (self.pos * 2.0 - vec2(1.0, 1.0)) * vec2(ar, 1.0) / zoom;

            let mut o = vec4(0.0, 0.0, 0.0, 0.0);
            // Reduced iterations (6 instead of 10) for performance in small widget
            for i in 1..7 {
                let fi = float(i);
                let mut v = p;
                for f in 1..8 {
                    let ff = float(f);
                    let a1 = v.y * ff + fi + t;
                    let a2 = v.x * ff + fi + t;
                    v = v + vec2(sin(a1), sin(a2)) / ff;
                }
                let len = max(length(v), 0.001);
                o = o + vec4(
                    (cos(fi + 0.0) + 1.0) / 6.0 / len,
                    (cos(fi + 1.0) + 1.0) / 6.0 / len,
                    (cos(fi + 2.0) + 1.0) / 6.0 / len,
                    0.0
                );
            }
            // tanh approximation
            let o2 = o * o;
            let fractal = vec3(
                o2.x / (1.0 + abs(o2.x)),
                o2.y / (1.0 + abs(o2.y)),
                o2.z / (1.0 + abs(o2.z))
            );

            // Blend fractal brightness: dim when idle, vibrant when playing
            let fractal_bright = mix(0.35, 0.7 + amp * 0.3, self.is_playing);
            let bg_color = vec4(fractal * fractal_bright, 0.95);

            // Draw rounded background with fractal fill
            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
            sdf.fill(bg_color);

            // ── Audio bars on top ──
            let num_bars = 11.0;
            let gap = 2.5;
            let padding = 8.0;
            let usable_width = self.rect_size.x - padding * 2.0;
            let bar_width = (usable_width - gap * (num_bars - 1.0)) / num_bars;
            let bar_max_height = self.rect_size.y - padding * 2.0;

            for i in 0..11 {
                let fi = float(i);
                let x = padding + fi * (bar_width + gap);
                let phase = fi * 0.7;

                // Idle: gentle breathing
                let idle_wave = sin(self.draw_pass.time * 2.5 + phase) * 0.3 + 0.45;
                let idle_sub = sin(self.draw_pass.time * 1.3 + phase * 1.7) * 0.1;
                let idle_total = idle_wave + idle_sub;

                // Playing: energetic multi-frequency
                let play_wave1 = sin(self.draw_pass.time * 6.0 + phase) * 0.35;
                let play_wave2 = sin(self.draw_pass.time * 3.5 + phase * 2.1) * 0.2;
                let play_base = play_wave1 + play_wave2 + 0.55;

                let real_wave = play_base * amp;
                let active_wave = mix(play_base, real_wave, use_real_amp);
                let wave = mix(idle_total, active_wave, self.is_playing);
                let height = max(clamp(wave, 0.08, 1.0) * bar_max_height, 3.0);

                // Semi-transparent white/cyan bars that let fractal show through
                let bar_t = fi / 10.0;
                let bar_alpha = mix(0.6, 0.85, self.is_playing);
                let bar_color = vec3(
                    mix(0.7, 1.0, bar_t),
                    mix(0.95, 0.85, bar_t * bar_t),
                    1.0
                );

                let y = self.rect_size.y - padding - height;
                sdf.box(x, y, bar_width, height, 2.5);
                sdf.fill(vec4(bar_color * bar_alpha, bar_alpha));
            }
            return sdf.result;
        }
    }

    // ============================================================
    // Shader Stage Effects (5 shaders with amplitude modulation)
    // ============================================================

    // Aurora - volumetric light through cosine lattice
    DrawAurora: mod.std.set_type_default() do #(DrawAurora::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let t = self.draw_pass.time * (0.3 + self.amplitude * 2.0) * self.speed;
            let rx = self.rect_size.x;
            let ry = self.rect_size.y;
            let fc = vec3(self.pos.x * rx, (1.0 - self.pos.y) * ry, 0.0);
            let mut o = vec4(0.0, 0.0, 0.0, 0.0);
            let mut z = 0.0;
            let mut d = 0.1;
            for _i in 0..100 {
                let ray = (fc * 2.0 - vec3(rx, ry, ry)) / ry / self.zoom;
                let p = ray * z + vec3(1.0, 1.0, 1.0);
                let mut w = p;
                for _j in 0..5 {
                    let f = float(_j) + 1.0;
                    let phase = -9.0 * exp(-d / 0.1) + t;
                    w = w + sin(vec3(w.z, w.x, w.y) * f + vec3(phase, phase, phase)) / f;
                }
                let blended = mix(p, w, 0.1);
                let cs = self.color_shift;
                let offsets = vec4(cs, cs + 1.0, cs + 2.0, cs + 3.0) / 100.0;
                let vals = abs(vec4(blended.y, blended.y, blended.y, blended.y) + offsets);
                let safe_vals = max(vals, vec4(0.0001, 0.0001, 0.0001, 0.0001));
                let glow_intensity = (0.03 + self.amplitude * 0.02) * self.glow;
                o = o + glow_intensity / safe_vals * d;
                d = 0.3 * (length(cos(vec2(p.x, p.z))) - 0.4);
                z = z + d;
            }
            let e2x = exp(2.0 * o);
            let ones = vec4(1.0, 1.0, 1.0, 1.0);
            let result = (e2x - ones) / (e2x + ones);
            return vec4(result.x, result.y, result.z, 1.0);
        }
    }

    // Reef - underwater coral reef
    DrawReef: mod.std.set_type_default() do #(DrawReef::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let t = self.draw_pass.time * 0.3 * self.speed;
            let rx = self.rect_size.x;
            let ry = self.rect_size.y;
            let fc = vec3(self.pos.x * rx, (1.0 - self.pos.y) * ry, 0.0);
            let mut o = vec4(0.0, 0.0, 0.0, 0.0);
            let mut z = 0.0;
            let mut d = 0.0;
            let zoom = (1.0 + self.amplitude * 0.3) * self.zoom;
            for _i in 0..50 {
                let i = float(_i) + 1.0;
                let raw = fc * 2.0 - vec3(rx, ry, rx);
                let rd = normalize(raw);
                let mut p = rd * z * zoom;
                let mut dd = 0.0;
                for _j in 0..9 {
                    dd = dd + 1.0;
                    let swing = vec3(p.y, p.z, p.x) * dd - vec3(z, z, z) + vec3(t + i, t + i, t + i);
                    p = p + 0.4 * sin(swing) / dd + vec3(0.5, 0.5, 0.5);
                }
                d = dd;
                let v1 = abs(p.y + p.z * 0.5);
                let sp = sin(p - vec3(z, z, z)) / 7.0;
                let dist_vec = vec4(v1, sp.x, sp.y, sp.z);
                d = length(dist_vec) / (4.0 + z * z / 100.0);
                let phase = i * 0.1;
                let sat = 0.9 + self.amplitude * 0.3;
                let cs = self.color_shift;
                let color_wave = sat + vec4(
                    sin(phase - 6.0 + cs),
                    sin(phase - 1.0 + cs),
                    sin(phase - 2.0 + cs),
                    sin(phase + cs)
                ) * self.glow;
                let safe_z = max(z, 0.001);
                let safe_d = max(d, 0.0001);
                let term1 = color_wave / (safe_d * safe_d * safe_z);
                let term2 = d * z / vec4(4.0, 2.0, 1.0, 1.0);
                o = o + term1 + term2;
                z = z + d;
            }
            let scaled = o / 2000.0;
            let e2x = exp(2.0 * scaled);
            let result = (e2x - vec4(1.0, 1.0, 1.0, 1.0)) / (e2x + vec4(1.0, 1.0, 1.0, 1.0));
            return vec4(result.x, result.y, result.z, 1.0);
        }
    }

    // Fractal Rainbow - code golf style rainbow fractal
    DrawFractalRainbow: mod.std.set_type_default() do #(DrawFractalRainbow::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let t = self.draw_pass.time * (0.8 + self.amplitude * 1.5) * self.speed;
            let ar = self.rect_size.x / max(self.rect_size.y, 1.0);
            let zoom = (0.3 - self.amplitude * 0.1) * self.zoom;
            let p = (self.pos * 2.0 - vec2(1.0, 1.0)) * vec2(ar, 1.0) / zoom;
            let mut o = vec4(0.0, 0.0, 0.0, 0.0);
            for i in 1..11 {
                let fi = float(i);
                let mut v = p;
                for f in 1..10 {
                    let ff = float(f);
                    let a1 = v.y * ff + fi + t;
                    let a2 = v.x * ff + fi + t;
                    v = v + vec2(sin(a1), sin(a2)) / ff;
                }
                let len = max(length(v), 0.001);
                let cs = self.color_shift;
                let gl = self.glow;
                o = o + vec4(
                    (cos(fi + 0.0 + cs) + 1.0) / 6.0 / len * gl,
                    (cos(fi + 1.0 + cs) + 1.0) / 6.0 / len * gl,
                    (cos(fi + 2.0 + cs) + 1.0) / 6.0 / len * gl,
                    (cos(fi + 3.0 + cs) + 1.0) / 6.0 / len * gl
                );
            }
            let o2 = o * o;
            let result = vec4(
                o2.x / (1.0 + abs(o2.x)),
                o2.y / (1.0 + abs(o2.y)),
                o2.z / (1.0 + abs(o2.z)),
                1.0
            );
            return result;
        }
    }

    // Glowing Lattice - observer effect
    DrawGlowingLattice: mod.std.set_type_default() do #(DrawGlowingLattice::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let r = self.rect_size;
            let t = self.draw_pass.time * (0.8 + self.amplitude * 1.5) * self.speed;
            let fc = self.pos * r;
            let p = (fc * 2.0 - r) / r.y / (0.2 * self.zoom);
            let mut o = vec4(0.0, 0.0, 0.0, 0.0);
            for i in 1..11 {
                let fi = float(i);
                let mut v = p;
                for f in 1..10 {
                    let ff = float(f);
                    v = v + sin(ceil(v * ff + fi * 0.9) - t / 2.0) / ff;
                }
                let l = length(v) - fi;
                let d = max(l, -l * 3.0);
                let glow_base = (0.03 + self.amplitude * 0.05) * self.glow;
                let glow = glow_base / (abs(d) + 0.00005);
                let phase_offset = 0.1 * l / (l * l + 0.005);
                let cs = self.color_shift;
                let phase = t - fi * 0.4 + phase_offset;
                o = o + glow * vec4(
                    cos(phase + cs) + 1.1,
                    cos(phase + 1.0 + cs) + 1.1,
                    cos(phase + 2.0 + cs) + 1.1,
                    cos(phase + 3.0 + cs) + 1.1
                );
            }
            let rich = o + o * o * 0.12;
            let result = vec4(
                rich.x / (1.0 + abs(rich.x)),
                rich.y / (1.0 + abs(rich.y)),
                rich.z / (1.0 + abs(rich.z)),
                1.0
            );
            return max(result, vec4(0.0, 0.0, 0.0, 1.0));
        }
    }

    // Jellyfish - point-cloud bioluminescent jellyfish
    DrawJellyfish: mod.std.set_type_default() do #(DrawJellyfish::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let t = self.draw_pass.time * 0.8 * self.speed;
            let aspect = self.rect_size.x / self.rect_size.y;
            let view_scale = 900.0 / self.zoom;
            let px = (self.pos.x - 0.5) * view_scale * aspect;
            let py = (self.pos.y - 0.5) * view_scale;
            let depth = self.pos.y;
            let caustic_boost = 1.0 + self.amplitude * 2.0;
            let caustic = sin(self.pos.x * 25.0 + t * 0.4)
                        * sin(self.pos.y * 18.0 - t * 0.25) * 0.008 * caustic_boost;
            let mut o = vec4(
                0.005 + caustic,
                0.012 + depth * 0.015 + caustic,
                0.04  + depth * 0.03  + caustic * 2.0,
                1.0
            );
            let bell_expand = 1.0 + self.amplitude * 0.5;
            for i in 0..120 {
                let fi = float(i);
                let ix = fi - 10.0 * floor(fi / 10.0);
                let iy = floor(fi / 10.0);
                let x = ix * 17.0;
                let y = iy * 15.5;
                let k = 5.0 * cos(x / 14.0) * cos(y / 30.0);
                let e = y / 8.0 - 13.0;
                let d = (k * k + e * e) / 59.0 + 4.0;
                let bell = (1.0 + 0.8 * exp(-(d - 4.0))) * bell_expand;
                let q = 60.0 - 3.0 * sin(atan2(k, e))
                      + k * (3.0 + 4.0 / d * sin(d * d - 2.0 * t));
                let c = d / 2.0 + e / 99.0 - t / 18.0;
                let u = 3.0 * q * sin(c) * bell + sin(t * 0.04) * 25.0;
                let v = 3.0 * (q + 9.0 * d) * cos(c) * bell + cos(t * 0.035) * 20.0;
                let dx = u - px;
                let dy = v - py;
                let dist2 = dx * dx + dy * dy;
                if dist2 < 600.0 {
                    let glow = (exp(-dist2 / 8.0) * 0.35
                             + exp(-dist2 / 50.0) * 0.08
                             + exp(-dist2 / 250.0) * 0.02) * self.glow;
                    let hue = y * 0.016 + k * 0.12
                            + atan2(k, e) * 0.25 + d * 0.06
                            + self.color_shift / 6.2832;
                    o = o + vec4(
                        glow * (0.5 + 0.5 * cos(6.2832 * hue)),
                        glow * (0.5 + 0.5 * cos(6.2832 * (hue - 0.33))),
                        glow * (0.5 + 0.5 * cos(6.2832 * (hue - 0.67))),
                        0.0
                    );
                }
            }
            return vec4(
                o.x / (1.0 + o.x),
                o.y / (1.0 + o.y),
                o.z / (1.0 + o.z),
                1.0
            );
        }
    }

    // Turbulence Fire - Xor-style layered sine-wave fluid fire
    // Technique: multi-octave turbulence with golden-angle rotation,
    // volumetric glow accumulation, and tanh tonemapping.
    DrawTurbulenceFire: mod.std.set_type_default() do #(DrawTurbulenceFire::script_shader(vm)){
        ..mod.draw.DrawQuad
        pixel: fn() {
            let t = self.draw_pass.time * (0.6 + self.amplitude * 2.0) * self.speed;
            let ar = self.rect_size.x / max(self.rect_size.y, 1.0);

            // Normalized coords: bottom-center origin, y goes up
            let scale = 2.5 / self.zoom;
            let mut uv = vec2(
                (self.pos.x - 0.5) * ar * scale,
                (1.0 - self.pos.y) * scale - 0.3
            );

            // --- Turbulence distortion (Xor technique) ---
            // 10 octaves of rotated sine waves simulate fluid motion
            let mut freq = 2.0;
            // Golden-angle rotation matrix (~137.5 deg) — breaks tiling patterns
            let mut rx = 0.22252093;
            let mut ry = -0.97492791;
            let mut rz = 0.97492791;
            let mut rw = 0.22252093;
            let turb_amp = 0.7 + self.amplitude * 0.5;

            for i in 0..10 {
                let fi = float(i);
                // Rotate uv by current rotation matrix
                let ru = rx * uv.x + ry * uv.y;
                // Phase: scroll upward along rotated y
                let phase = freq * ru + t * (0.3 + fi * 0.02);
                // Perpendicular offset (the rotation's x-axis)
                uv = uv + turb_amp * vec2(rz, rw) * sin(phase) / freq;
                // Compose rotation: multiply by golden-angle matrix again
                let nrx = rx * 0.22252093 + ry * 0.97492791;
                let nry = rx * (-0.97492791) + ry * 0.22252093;
                let nrz = rz * 0.22252093 + rw * 0.97492791;
                let nrw = rz * (-0.97492791) + rw * 0.22252093;
                rx = nrx;
                ry = nry;
                rz = nrz;
                rw = nrw;
                freq = freq * 1.45;
            }

            // --- Fire shape ---
            // Vertical gradient: base flame intensity fades upward
            let flame_height = 1.8 + self.amplitude * 0.6;
            let y_fade = max(0.0, 1.0 - self.pos.y / flame_height);
            // Horizontal: narrower at top, wider at base
            let x_width = 0.6 + (1.0 - self.pos.y) * 0.8;
            let x_fade = max(0.0, 1.0 - abs(uv.x) / x_width);
            let shape = y_fade * x_fade;

            // --- Density field from turbulence ---
            let density = shape * shape * (1.5 + self.amplitude);

            // --- Fire coloring with volumetric glow ---
            // Hot core: white-yellow, outer: orange-red, edge: dark red
            let hot = smoothstep(0.0, 1.5, density);
            let gl = self.glow;
            let cs = self.color_shift;
            // Color shift rotates the fire palette: 0=warm, ~2=green, ~4=blue
            let r_val = min(1.0, density * 1.8 * gl) * (0.5 + 0.5 * cos(cs));
            let g_val = density * density * 0.7 * gl * (0.5 + 0.5 * cos(cs + 2.094));
            let b_val = density * density * density * 0.4 * gl * (0.5 + 0.5 * cos(cs + 4.189));

            // Add blue base for gas flame effect at high amplitude
            let blue_fire = self.amplitude * 0.3 * max(0.0, 1.0 - self.pos.y * 2.0);
            let b_boost = b_val + blue_fire * density;

            let col = vec3(r_val, g_val, b_boost);

            // --- Ember particles (Xor efficient chaos - 3 layers) ---
            let mut ember = 0.0;
            let mut ec = vec2(self.pos.x * 8.0, (1.0 - self.pos.y) * 12.0 + t * 0.5);
            for i in 0..3 {
                // Golden-angle rotate each layer
                let tmp = ec.x * 0.22252093 - ec.y * 0.97492791;
                ec.y = ec.x * 0.97492791 + ec.y * 0.22252093;
                ec.x = tmp;
                let fi = float(i);
                let p = ec + fi * 2.618;
                let cell = p - vec2(2.0, 2.0) * floor(p / vec2(2.0, 2.0)) - vec2(1.0, 1.0);
                let len = length(cell);
                ember = ember + max(0.0, 1.0 - len) / max(len, 0.01) * 0.005;
            }
            let ember_color = vec3(1.0, 0.6, 0.1) * ember * y_fade;

            // --- Background: dark with subtle heat haze ---
            let bg_heat = sin(self.pos.x * 20.0 + t * 2.0) * sin(self.pos.y * 15.0 - t) * 0.003;
            let bg = vec3(0.01 + bg_heat, 0.005, 0.015);

            let final_col = bg + col + ember_color;

            // Tanh tonemapping (Xor recommended)
            let mapped = vec3(
                final_col.x / (1.0 + final_col.x),
                final_col.y / (1.0 + final_col.y),
                final_col.z / (1.0 + final_col.z)
            );

            return vec4(mapped.x, mapped.y, mapped.z, 1.0);
        }
    }

    // Liquid Glass Taiji - rotates with audio rhythm
    DrawTaiji: mod.std.set_type_default() do #(DrawTaiji::script_shader(vm)){
        ..mod.draw.DrawQuad
        background: fn(uv: vec2, time: float) {
            let t = time * 0.5;
            let r1 = sin(uv.x * 10.0 + t) * 0.5 + 0.5;
            let g1 = sin(uv.y * 8.0 - t * 0.7) * 0.5 + 0.5;
            let b1 = sin((uv.x + uv.y) * 6.0 + t * 1.3) * 0.5 + 0.5;
            let r2 = cos(uv.y * 12.0 + t * 0.8) * 0.3;
            let g2 = cos(uv.x * 9.0 - t * 0.5) * 0.3;
            let b2 = sin(uv.x * 7.0 + uv.y * 5.0 + t) * 0.3;
            return vec4(0.3 + r1 * 0.4 + r2, 0.3 + g1 * 0.4 + g2, 0.4 + b1 * 0.4 + b2, 1.0);
        }

        pixel: fn() {
            let c = self.rect_size * 0.5;
            let r = min(c.x, c.y) - 4.0;
            let p = self.pos * self.rect_size;

            // Rotation driven by anim parameter (0..1 wrapping)
            let angle = self.anim * 2.0 * PI;
            let ca = cos(angle);
            let sa = sin(angle);
            let ox = p.x - c.x;
            let oy = p.y - c.y;
            let rx = ox * ca - oy * sa;
            let ry = ox * sa + oy * ca;

            let dist = sqrt(rx * rx + ry * ry);
            let h = r * 0.5;
            let dotrad = r * 0.13;

            // Raw background
            let raw_bg = self.background(self.pos, self.anim);

            // Smooth circle mask
            let aa = 1.5;
            let circle_mask = 1.0 - smoothstep(r - aa, r + aa, dist);
            if (circle_mask < 0.01) {
                return raw_bg;
            }

            // Yin/Yang regions with anti-aliasing
            let uy = ry + h;
            let ly = ry - h;
            let udist = sqrt(rx * rx + uy * uy);
            let ldist = sqrt(rx * rx + ly * ly);

            let in_udot = 1.0 - smoothstep(dotrad - aa, dotrad + aa, udist);
            let in_ldot = 1.0 - smoothstep(dotrad - aa, dotrad + aa, ldist);
            let in_usemi = 1.0 - smoothstep(h - aa, h + aa, udist);
            let in_lsemi = 1.0 - smoothstep(h - aa, h + aa, ldist);
            let in_left = 1.0 - smoothstep(-aa, aa, rx);

            let yy0 = in_left;
            let yy1 = mix(yy0, 0.0, in_usemi);
            let yy2 = mix(yy1, 1.0, in_lsemi);
            let yy3 = mix(yy2, 1.0, in_udot);
            let isyin = mix(yy3, 0.0, in_ldot);

            // Glass surface normal (dome shape)
            let norm_dist = dist / r;
            let nz = sqrt(max(0.01, 1.0 - norm_dist * norm_dist));
            let nx = -rx / r;
            let ny = -ry / r;

            // Chromatic aberration - RGB channel separation
            let base_refract = 0.04;
            let edge_factor = pow(norm_dist, 2.0);
            let r_offset = vec2(nx, ny) * base_refract / 1.45 * (1.0 + edge_factor);
            let g_offset = vec2(nx, ny) * base_refract / 1.50 * (1.0 + edge_factor * 1.2);
            let b_offset = vec2(nx, ny) * base_refract / 1.55 * (1.0 + edge_factor * 1.5);

            let r_sample = self.background(self.pos + r_offset, self.anim).x;
            let g_sample = self.background(self.pos + g_offset, self.anim).y;
            let b_sample = self.background(self.pos + b_offset, self.anim).z;
            let refracted = vec4(r_sample, g_sample, b_sample, 1.0);

            // Edge dispersion (rainbow at edges)
            let edge_dist = r - dist;
            let edge_zone = smoothstep(15.0, 0.0, edge_dist);
            let edge_angle = atan2(ry, rx);
            let rainbow_r = sin(edge_angle * 2.0 + self.anim * 3.0) * 0.5 + 0.5;
            let rainbow_g = sin(edge_angle * 2.0 + self.anim * 3.0 + 2.094) * 0.5 + 0.5;
            let rainbow_b = sin(edge_angle * 2.0 + self.anim * 3.0 + 4.188) * 0.5 + 0.5;
            let rainbow = vec4(rainbow_r, rainbow_g, rainbow_b, 1.0);
            let with_dispersion = mix(refracted, rainbow, edge_zone * 0.4);

            // Glass tint
            let yin_color = vec4(0.08, 0.08, 0.12, 1.0);
            let yang_color = vec4(0.95, 0.95, 1.0, 1.0);
            let blend = 0.55;
            let yin_glass = mix(with_dispersion, yin_color, blend);
            let yang_glass = mix(with_dispersion, yang_color, blend * 0.4);
            let tinted = mix(yang_glass, yin_glass, isyin);

            // Fresnel reflection
            let fresnel = pow(1.0 - nz, 4.0);
            let reflection = vec4(0.9, 0.95, 1.0, 1.0) * fresnel * 0.3;

            // Dynamic specular highlights
            let lx = cos(self.anim * 1.2) * 0.6;
            let ly2 = sin(self.anim * 0.9) * 0.6;
            let lz = 0.8;
            let ldot = max(0.0, nx * lx + ny * ly2 + nz * lz);
            let spec1 = pow(ldot, 64.0) * 0.6;
            let spec2 = pow(ldot, 16.0) * 0.2;
            let l2x = cos(self.anim * 0.8 + 3.14) * 0.5;
            let l2y = sin(self.anim * 1.1 + 3.14) * 0.5;
            let l2dot = max(0.0, nx * l2x + ny * l2y + nz * 0.7);
            let spec3 = pow(l2dot, 32.0) * 0.3;
            let specular = spec1 + spec2 + spec3;

            // Edge highlight
            let rim_width = 3.0;
            let rim = smoothstep(r - rim_width, r, dist) * (1.0 - smoothstep(r, r + aa, dist));
            let rim_color = vec4(1.0, 1.0, 1.0, 1.0) * rim * 0.8;

            // Final composite
            let glass = tinted + reflection;
            let with_spec = glass + vec4(specular, specular, specular * 1.1, 0.0);
            let with_rim = with_spec + rim_color;

            return mix(raw_bg, with_rim, circle_mask);
        }
    }
}

// ============================================================================
// A2UI Theme Colors
// ============================================================================

/// Theme colors for A2UI surface and its components
#[derive(Clone, Copy, Debug)]
pub struct A2uiThemeColors {
    /// Background color for the surface
    pub bg_surface: Vec4,
    /// Background color for cards
    pub bg_card: Vec4,
    /// Border color for cards, inputs, etc.
    pub border_color: Vec4,
    /// Primary text color
    pub text_primary: Vec4,
    /// Secondary/muted text color
    pub text_secondary: Vec4,
    /// Accent/primary button color
    pub accent: Vec4,
    /// Accent hover color
    pub accent_hover: Vec4,
    /// Accent pressed color
    pub accent_pressed: Vec4,
    /// Input field background color
    pub input_bg: Vec4,
    /// Slider track color
    pub slider_track: Vec4,
    /// Checkbox/slider fill color
    pub control_fill: Vec4,
}

impl Default for A2uiThemeColors {
    fn default() -> Self {
        // Default dark purple theme
        Self {
            bg_surface: vec4(0.102, 0.102, 0.180, 1.0),     // #1a1a2e
            bg_card: vec4(0.165, 0.227, 0.353, 1.0),        // #2a3a5a
            border_color: vec4(0.333, 0.533, 0.733, 1.0),   // #5588bb
            text_primary: vec4(1.0, 1.0, 1.0, 1.0),         // #FFFFFF
            text_secondary: vec4(0.533, 0.533, 0.533, 1.0), // #888888
            accent: vec4(0.231, 0.51, 0.965, 1.0),          // #3B82F6
            accent_hover: vec4(0.145, 0.388, 0.922, 1.0),   // slightly darker
            accent_pressed: vec4(0.114, 0.306, 0.847, 1.0), // even darker
            input_bg: vec4(0.165, 0.227, 0.353, 1.0),       // #2a3a5a
            slider_track: vec4(0.227, 0.290, 0.416, 1.0),   // #3a4a6a
            control_fill: vec4(0.231, 0.51, 0.965, 1.0),    // #3B82F6
        }
    }
}

impl A2uiThemeColors {
    /// Create dark purple theme colors (default)
    pub fn dark_purple() -> Self {
        Self::default()
    }

    /// Create light iOS-like theme colors
    pub fn light() -> Self {
        Self {
            bg_surface: vec4(1.0, 1.0, 1.0, 1.0),           // #FFFFFF
            bg_card: vec4(0.96, 0.96, 0.97, 1.0),           // #f5f5f8
            border_color: vec4(0.85, 0.85, 0.87, 1.0),      // #d9d9de
            text_primary: vec4(0.11, 0.11, 0.118, 1.0),     // #1c1c1e
            text_secondary: vec4(0.557, 0.557, 0.576, 1.0), // #8e8e93
            accent: vec4(0.0, 0.478, 1.0, 1.0),             // #007AFF
            accent_hover: vec4(0.0, 0.4, 0.85, 1.0),        // slightly darker
            accent_pressed: vec4(0.0, 0.35, 0.75, 1.0),     // even darker
            input_bg: vec4(0.95, 0.95, 0.97, 1.0),          // light gray
            slider_track: vec4(0.9, 0.9, 0.92, 1.0),        // light gray
            control_fill: vec4(0.0, 0.478, 1.0, 1.0),       // #007AFF
        }
    }

    /// Create soft gray mid-tone theme colors
    pub fn soft() -> Self {
        Self {
            bg_surface: vec4(0.533, 0.553, 0.588, 1.0),     // #888d96
            bg_card: vec4(0.6, 0.62, 0.66, 1.0),            // slightly lighter
            border_color: vec4(0.7, 0.72, 0.76, 1.0),       // light border
            text_primary: vec4(1.0, 1.0, 1.0, 1.0),         // #FFFFFF
            text_secondary: vec4(0.2, 0.2, 0.25, 1.0),      // dark gray for contrast
            accent: vec4(0.231, 0.51, 0.965, 1.0),          // #3B82F6 (vibrant blue)
            accent_hover: vec4(0.145, 0.388, 0.922, 1.0),   // slightly darker
            accent_pressed: vec4(0.114, 0.306, 0.847, 1.0), // even darker
            input_bg: vec4(0.5, 0.52, 0.56, 1.0),           // medium gray
            slider_track: vec4(0.45, 0.47, 0.51, 1.0),      // darker gray
            control_fill: vec4(0.231, 0.51, 0.965, 1.0),    // #3B82F6 (vibrant blue)
        }
    }
}

// ============================================================================
// A2UI Surface Actions
// ============================================================================

/// Actions emitted by A2uiSurface widget
#[derive(Clone, Debug, Default)]
pub enum A2uiSurfaceAction {
    #[default]
    None,
    /// User triggered an action (e.g., button click)
    UserAction(UserAction),
    /// Data model value changed (two-way binding)
    DataModelChanged {
        surface_id: String,
        path: String,
        value: serde_json::Value,
    },
    /// Audio player play button clicked - open URL in browser
    PlayAudio {
        component_id: String,
        url: String,
        title: String,
    },
}

// ============================================================================
// DrawA2uiImage - for rendering images with border radius
// ============================================================================

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawA2uiImage {
    #[deref]
    draw_super: DrawQuad,
}

// ============================================================================
// DrawA2uiChartLine - for rendering line chart segments (chord chart)
// ============================================================================

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawA2uiChartLine {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    pub color: Vec4,
    #[live(0.0)]
    pub x1: f32,
    #[live(0.0)]
    pub y1: f32,
    #[live(1.0)]
    pub x2: f32,
    #[live(1.0)]
    pub y2: f32,
    #[live(0.05)]
    pub line_width: f32,
}

// ============================================================================
// DrawA2uiArc - for rendering pie chart slices
// ============================================================================

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawA2uiArc {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    pub color: Vec4,
    #[live(0.0)]
    pub start_angle: f32,
    #[live(1.0)]
    pub end_angle: f32,
    #[live(0.0)]
    pub inner_radius: f32,
}

// ============================================================================
// DrawA2uiQuad - arbitrary convex quadrilateral with AA edges (chord chart)
// DrawAudioBars - for rendering audio waveform visualization
// ============================================================================

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawA2uiQuad {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    pub color: Vec4,
    #[live(0.3)]
    pub opacity: f32,
    // Corner positions in pixel coords relative to the draw quad origin
    #[live(0.0)]
    pub p0x: f32,
    #[live(0.0)]
    pub p0y: f32,
    #[live(1.0)]
    pub p1x: f32,
    #[live(0.0)]
    pub p1y: f32,
    #[live(1.0)]
    pub p2x: f32,
    #[live(1.0)]
    pub p2y: f32,
    #[live(0.0)]
    pub p3x: f32,
    #[live(1.0)]
    pub p3y: f32,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawAudioBars {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub is_playing: f32,
    #[live(0.0)]
    pub amplitude: f32,
}

// ============================================================================
// Shader Stage draw types (5 effects with amplitude modulation)
// ============================================================================

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawAurora {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub amplitude: f32,
    #[live(1.0)]
    pub speed: f32,
    #[live(1.0)]
    pub zoom: f32,
    #[live(1.0)]
    pub glow: f32,
    #[live(0.0)]
    pub color_shift: f32,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawReef {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub amplitude: f32,
    #[live(1.0)]
    pub speed: f32,
    #[live(1.0)]
    pub zoom: f32,
    #[live(1.0)]
    pub glow: f32,
    #[live(0.0)]
    pub color_shift: f32,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawFractalRainbow {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub amplitude: f32,
    #[live(1.0)]
    pub speed: f32,
    #[live(1.0)]
    pub zoom: f32,
    #[live(1.0)]
    pub glow: f32,
    #[live(0.0)]
    pub color_shift: f32,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawGlowingLattice {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub amplitude: f32,
    #[live(1.0)]
    pub speed: f32,
    #[live(1.0)]
    pub zoom: f32,
    #[live(1.0)]
    pub glow: f32,
    #[live(0.0)]
    pub color_shift: f32,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawJellyfish {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub amplitude: f32,
    #[live(1.0)]
    pub speed: f32,
    #[live(1.0)]
    pub zoom: f32,
    #[live(1.0)]
    pub glow: f32,
    #[live(0.0)]
    pub color_shift: f32,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawTurbulenceFire {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub amplitude: f32,
    #[live(1.0)]
    pub speed: f32,
    #[live(1.0)]
    pub zoom: f32,
    #[live(1.0)]
    pub glow: f32,
    #[live(0.0)]
    pub color_shift: f32,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawTaiji {
    #[deref]
    draw_super: DrawQuad,
    #[live(0.0)]
    pub anim: f32,
}
