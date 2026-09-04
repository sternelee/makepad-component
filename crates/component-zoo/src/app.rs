use makepad_component::widgets::MpThemeState;
use makepad_component::widgets::MpDescriptionItem;
use makepad_component::widgets::MpDescriptionListWidgetRefExt;
use makepad_component::widgets::MpAvatarWidgetRefExt;
use makepad_component::widgets::MpBadgeWidgetRefExt;
use makepad_component::widgets::MpAttachmentWidgetRefExt;
use makepad_component::widgets::MpBubbleWidgetRefExt;
use makepad_component::widgets::MpColorPickerWidgetRefExt;
use makepad_component::widgets::MpButtonWidgetExt;
use makepad_component::widgets::MpButtonWidgetRefExt;
use makepad_component::widgets::MpCardAction;
use makepad_component::widgets::MpCheckboxWidgetRefExt;
use makepad_component::widgets::MpChipWidgetRefExt;
use makepad_component::widgets::MpCollapsibleTriggerWidgetRefExt;
use makepad_component::widgets::MpContextMenuWidgetRefExt;
use makepad_component::widgets::MpDropdownMenuWidgetRefExt;
use makepad_component::widgets::MpComboboxWidgetRefExt;
use makepad_component::widgets::MpModalAction;
use makepad_component::widgets::MpModalWidgetWidgetRefExt;
use makepad_component::widgets::MpMenuBarWidgetRefExt;
use makepad_component::widgets::MpNotificationWidgetWidgetRefExt;
use makepad_component::widgets::MpPopoverWidgetWidgetRefExt;
use makepad_component::widgets::MpProgressRingWidgetRefExt;
use makepad_component::widgets::MpOptionCardWidgetRefExt;
use makepad_component::widgets::MpProgressWidgetRefExt;
use makepad_component::widgets::MpRadioWidgetRefExt;
use makepad_component::widgets::MpSheetTriggerWidgetRefExt;
use makepad_component::widgets::MpSheetWidgetRefExt;
use makepad_component::widgets::MpSkeletonWidgetWidgetRefExt;
use makepad_component::widgets::MpSliderWidgetRefExt;
use makepad_component::widgets::MpStepIndicatorWidgetRefExt;
use makepad_component::widgets::MpStepperWidgetRefExt;
use makepad_component::widgets::MpSplitPaneWidgetRefExt;
use makepad_component::widgets::MpStepRowWidgetRefExt;
use makepad_component::widgets::MpRatingWidgetRefExt;
use makepad_component::widgets::MpSwitchWidgetRefExt;
use makepad_component::widgets::MpTabWidgetRefExt;
use makepad_component::widgets::MpTableWidgetRefExt;
use makepad_component::widgets::MpTreeWidgetRefExt;
use makepad_component::widgets::MpToggleGroupWidgetRefExt;
use makepad_component::widgets::TableColumn;
use makepad_component::widgets::TreeItem;
use makepad_widgets::*;

script_mod! {
use mod.prelude.widgets_internal.*
use mod.widgets.*
use mod.mpc_theme.*

// ============================================================
// Section Header Component
// ============================================================
let SectionHeader = Label{
    width: Fit, height: Fit,
    draw_text +: {
        text_style: theme.font_bold{ font_size: 18.0 }
        color: TEXT
    }
}

let SubsectionLabel = Label{
    width: Fit, height: Fit,
    draw_text +: {
        text_style: theme.font_regular{ font_size: 12.0 }
        color: TEXT_MUTED
    }
}

// ============================================================
// Category Tab Style
// ============================================================
let CategoryTab = mod.widgets.MpTabPill{
    padding: Inset{ left: 16, right: 16, top: 8, bottom: 8 }
}

// ============================================================
// Animated Shader Canvas - Shadertoy-style fractal
// ============================================================
mod.widgets.ShaderCanvasBase = #(ShaderCanvas::register_widget(vm))
mod.widgets.ShaderCanvas = set_type_default() do mod.widgets.ShaderCanvasBase{
    draw_bg +: {
        // Time uniform driven by animator
        anim_time: instance(0.0)

        // Shadertoy-style fractal shader
        pixel: fn() {
            let resolution = self.rect_size;
            let uv = self.pos;
            let t = self.anim_time;

            // Normalize coordinates: (FC.xy*2.-r)/r.y/.3
            let p = (uv * 2.0 - vec2(1.0, 1.0)) * vec2(resolution.x / resolution.y, 1.0) / 0.3;

            // Output color accumulator
            let mut o = vec4(0.0, 0.0, 0.0, 0.0);

            // Outer loop: i from 1 to 10
            for i in 1..11 {
                let fi = float(i);
                let mut v = p;

                // Inner loop: f from 1 to 9
                for f in 1..10 {
                    let ff = float(f);
                    // v += sin(v.yx * f + i + t) / f
                    let angle = v.y * ff + fi + t;
                    let angle2 = v.x * ff + fi + t;
                    v = v + vec2(sin(angle), sin(angle2)) / ff;
                }

                // o += (cos(i + vec4(0,1,2,3)) + 1) / 6 / length(v)
                let len = max(length(v), 0.001);
                o = o + vec4(
                    (cos(fi + 0.0) + 1.0) / 6.0 / len,
                    (cos(fi + 1.0) + 1.0) / 6.0 / len,
                    (cos(fi + 2.0) + 1.0) / 6.0 / len,
                    (cos(fi + 3.0) + 1.0) / 6.0 / len
                );
            }

            // tanh(o*o) approximation: x / (1 + |x|)
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

    animator: Animator{
        anim: {
            // default must NOT be `on`: Animator::play() early-returns when
            // current_state_id (which falls back to `default`) equals the
            // target state and no track exists yet, so the loop would never
            // start and anim_time would stay pinned at 0 (static shader).
            default: @off
            off: AnimatorState{ from: {all: Forward {duration: 0.0}} }
            on: AnimatorState{
                from: {all: Loop {duration: 10.0, end: 1.0}}
                redraw: true
                apply: {
                    draw_bg: {
                        anim_time: [{time: 0.0, value: 0.0}, {time: 1.0, value: 62.83}]
                    }
                }
            }
        }
    }
}

// ============================================================
// Shader Art Canvas - Observer effect
// ============================================================
mod.widgets.ShaderArtCanvasBase = #(ShaderArtCanvas::register_widget(vm))
mod.widgets.ShaderArtCanvas = set_type_default() do mod.widgets.ShaderArtCanvasBase{
    speed: 1.0

    draw_bg +: {
        anim_time: instance(0.0)
        speed: instance(1.0)

        // Observer shader - glowing lattice effect
        // Original: vec2 p=(FC.xy*2.-r)/r.y/.2,v;
        //   for(float i,l,f;i++<1e1;
        //     o+=.03/max(l=length(v)-i,-l*3.)*(cos(t-i*.4+.1/l+vec4(0,1,2,3))+1.1))
        //     for(v=p,f=0.;f++<9.;v+=sin(ceil(v*f+i*.9)-t/2.)/f);
        //   o=max(tanh(o+(o=texture(b,...))*o),.0);

        pixel: fn() {
            let r = self.rect_size;
            let t = self.anim_time * self.speed;

            // p = (FC.xy*2.-r)/r.y/.2
            let fc = self.pos * r;
            let p = (fc * 2.0 - r) / r.y / 0.2;

            let mut o = vec4(0.0, 0.0, 0.0, 0.0);

            for i in 1..11 {
                let fi = float(i);
                let mut v = p;

                // v += sin(ceil(v*f+i*.9)-t/2.)/f
                for f in 1..10 {
                    let ff = float(f);
                    v = v + sin(ceil(v * ff + fi * 0.9) - t / 2.0) / ff;
                }

                // l = length(v) - i
                let l = length(v) - fi;

                // .03/max(l, -l*3.)
                // When l>0: denom=l, when l<0: denom=3|l|
                // Creates asymmetric glow ring at length(v)==i
                let d = max(l, -l * 3.0);
                let glow = 0.03 / (abs(d) + 0.00005);

                // cos(t - i*.4 + .1/l + vec4(0,1,2,3)) + 1.1
                // Preserve sign of l for correct color phase
                let phase_offset = 0.1 * l / (l * l + 0.005);
                let phase = t - fi * 0.4 + phase_offset;

                o = o + glow * vec4(
                    cos(phase) + 1.1,
                    cos(phase + 1.0) + 1.1,
                    cos(phase + 2.0) + 1.1,
                    cos(phase + 3.0) + 1.1
                );
            }

            // Simulate: tanh(o + prev_frame * o)
            // Without texture feedback, boost with self-multiply for richness
            let rich = o + o * o * 0.12;

            // tanh approximation: x / (1 + |x|)
            let result = vec4(
                rich.x / (1.0 + abs(rich.x)),
                rich.y / (1.0 + abs(rich.y)),
                rich.z / (1.0 + abs(rich.z)),
                1.0
            );

            return max(result, vec4(0.0, 0.0, 0.0, 1.0));
        }
    }

    animator: Animator{
        anim: {
            // default must NOT be `on` (see ShaderCanvas note): play() would
            // early-return and anim_time would never advance.
            default: @off
            off: AnimatorState{ from: {all: Forward {duration: 0.0}} }
            on: AnimatorState{
                from: {all: Loop {duration: 15.0, end: 1.0}}
                redraw: true
                apply: {
                    draw_bg: {
                        anim_time: [{time: 0.0, value: 0.0}, {time: 1.0, value: 94.25}]
                    }
                }
            }
        }
    }
}

// ============================================================
// Shader Art 2 Canvas - FBM noise + HSV color cycling + bitmap text
// ============================================================
mod.widgets.ShaderArt2CanvasBase = #(ShaderArt2Canvas::register_widget(vm))
mod.widgets.ShaderArt2Canvas = set_type_default() do mod.widgets.ShaderArt2CanvasBase{
    speed: 1.0

    draw_bg +: {
        anim_time: instance(0.0)
        speed: instance(1.0)

        // Golden FBM noise + SCRY bitmap text
        // Faithful translation from Shadertoy common code
        pixel: fn() {
            let t = self.anim_time * self.speed;
            let uv = self.pos;
            let ar = self.rect_size.x / self.rect_size.y;

            // === FBM with domain warping (3 passes × 5 octaves) ===
            let mut fbm1 = 0.0;
            let mut a1 = 0.5;
            let mut p1 = uv * 4.0 + vec2(t * 0.08, t * 0.06);
            for oct in 0..5 {
                let i = floor(p1);
                let f = fract(p1);
                let u = f * f * (3.0 - 2.0 * f);
                let h00 = fract(sin(dot(i, vec2(127.1, 311.7))) * 43758.5453);
                let h10 = fract(sin(dot(i + vec2(1.0, 0.0), vec2(127.1, 311.7))) * 43758.5453);
                let h01 = fract(sin(dot(i + vec2(0.0, 1.0), vec2(127.1, 311.7))) * 43758.5453);
                let h11 = fract(sin(dot(i + vec2(1.0, 1.0), vec2(127.1, 311.7))) * 43758.5453);
                fbm1 = fbm1 + a1 * mix(mix(h00, h10, u.x), mix(h01, h11, u.x), u.y);
                p1 = p1 * 2.0;
                a1 = a1 * 0.5;
            }
            let mut fbm2 = 0.0;
            let mut a2 = 0.5;
            let mut p2 = uv * 3.0 + vec2(fbm1 * 2.0 - t * 0.05, t * 0.09);
            for oct in 0..5 {
                let i = floor(p2);
                let f = fract(p2);
                let u = f * f * (3.0 - 2.0 * f);
                let h00 = fract(sin(dot(i, vec2(127.1, 311.7))) * 43758.5453);
                let h10 = fract(sin(dot(i + vec2(1.0, 0.0), vec2(127.1, 311.7))) * 43758.5453);
                let h01 = fract(sin(dot(i + vec2(0.0, 1.0), vec2(127.1, 311.7))) * 43758.5453);
                let h11 = fract(sin(dot(i + vec2(1.0, 1.0), vec2(127.1, 311.7))) * 43758.5453);
                fbm2 = fbm2 + a2 * mix(mix(h00, h10, u.x), mix(h01, h11, u.x), u.y);
                p2 = p2 * 2.0;
                a2 = a2 * 0.5;
            }
            let mut warp = 0.0;
            let mut a3 = 0.5;
            let mut p3 = uv * 2.5 + vec2(fbm2 * 1.5, fbm1 * 1.5) + t * 0.04;
            for oct in 0..5 {
                let i = floor(p3);
                let f = fract(p3);
                let u = f * f * (3.0 - 2.0 * f);
                let h00 = fract(sin(dot(i, vec2(127.1, 311.7))) * 43758.5453);
                let h10 = fract(sin(dot(i + vec2(1.0, 0.0), vec2(127.1, 311.7))) * 43758.5453);
                let h01 = fract(sin(dot(i + vec2(0.0, 1.0), vec2(127.1, 311.7))) * 43758.5453);
                let h11 = fract(sin(dot(i + vec2(1.0, 1.0), vec2(127.1, 311.7))) * 43758.5453);
                warp = warp + a3 * mix(mix(h00, h10, u.x), mix(h01, h11, u.x), u.y);
                p3 = p3 * 2.0;
                a3 = a3 * 0.5;
            }

            // === Golden HSV -> RGB ===
            let hue = 0.06 + warp * 0.08 + fbm1 * 0.04;
            let sat = clamp(0.4 + fbm2 * 0.4, 0.3, 0.85);
            let val = clamp(0.5 + warp * 0.5 + fbm1 * 0.2, 0.2, 1.0);
            let px = abs(fract(hue + 1.0) * 6.0 - 3.0);
            let py = abs(fract(hue + 0.6667) * 6.0 - 3.0);
            let pz = abs(fract(hue + 0.3333) * 6.0 - 3.0);
            let mut col = vec3(
                val * mix(1.0, clamp(px - 1.0, 0.0, 1.0), sat),
                val * mix(1.0, clamp(py - 1.0, 0.0, 1.0), sat),
                val * mix(1.0, clamp(pz - 1.0, 0.0, 1.0), sat)
            );

            // === slogo: "SCRY" bitmap text ===
            // Faithful translation of slogo(uv, ar, size=8)
            // size = 240./8. = 30.
            // suv = uv; suv.x = 1-suv.x
            // suv *= 240./5.25/30. = 1.5238
            // suv -= 0.4; suv.x *= ar*1.75; suv.y *= 1.04
            // suv.x = 5 - suv.x
            let mut suv = uv;
            suv.x = 1.0 - suv.x;
            suv = suv * 1.5238;
            suv = suv - 0.4;
            suv.x = suv.x * ar * 1.75;
            suv.y = suv.y * 1.04;

            // ul = length(vec2(suv.x*0.5, suv.y) - 0.5) before transforms
            let ul = length(vec2(suv.x * 0.5, suv.y) - 0.5);

            suv.x = 5.0 - suv.x;

            // bitm: exact original math
            // uv_b = floor(vec2(uv.x*3, uv.y*5)) / vec2(3,3)
            // cc = uv_b.x + uv_b.y * 3
            // bit = mod(floor(code / exp2(ceil(cc*3 - 0.6))), 2)
            // bounds: step(0,uv_b.x)*step(0,uv_b.y)*step(0,-uv_b.x+0.99)*step(0,-uv_b.y+1.6)

            // Char S (29671)
            let bv1 = floor(vec2(suv.x * 3.0, suv.y * 5.0)) / 3.0;
            let cc1 = bv1.x + bv1.y * 3.0;
            let b1c = floor(29671.0 / exp2(ceil(cc1 * 3.0 - 0.6)));
            let b1 = b1c - 2.0 * floor(b1c / 2.0);
            let m1 = step(0.0, bv1.x) * step(0.0, bv1.y)
                   * step(0.0, -bv1.x + 0.99) * step(0.0, -bv1.y + 1.6);

            // Char C (29263) — suv.x -= 4/3
            let sx2 = suv.x - 1.333;
            let bv2 = floor(vec2(sx2 * 3.0, suv.y * 5.0)) / 3.0;
            let cc2 = bv2.x + bv2.y * 3.0;
            let b2c = floor(29263.0 / exp2(ceil(cc2 * 3.0 - 0.6)));
            let b2 = b2c - 2.0 * floor(b2c / 2.0);
            let m2 = step(0.0, bv2.x) * step(0.0, bv2.y)
                   * step(0.0, -bv2.x + 0.99) * step(0.0, -bv2.y + 1.6);

            // Char R (31469) — suv.x -= 8/3
            let sx3 = suv.x - 2.666;
            let bv3 = floor(vec2(sx3 * 3.0, suv.y * 5.0)) / 3.0;
            let cc3 = bv3.x + bv3.y * 3.0;
            let b3c = floor(31469.0 / exp2(ceil(cc3 * 3.0 - 0.6)));
            let b3 = b3c - 2.0 * floor(b3c / 2.0);
            let m3 = step(0.0, bv3.x) * step(0.0, bv3.y)
                   * step(0.0, -bv3.x + 0.99) * step(0.0, -bv3.y + 1.6);

            // Char Y (23186) — suv.x -= 4
            let sx4 = suv.x - 4.0;
            let bv4 = floor(vec2(sx4 * 3.0, suv.y * 5.0)) / 3.0;
            let cc4 = bv4.x + bv4.y * 3.0;
            let b4c = floor(23186.0 / exp2(ceil(cc4 * 3.0 - 0.6)));
            let b4 = b4c - 2.0 * floor(b4c / 2.0);
            let m4 = step(0.0, bv4.x) * step(0.0, bv4.y)
                   * step(0.0, -bv4.x + 0.99) * step(0.0, -bv4.y + 1.6);

            let b = clamp(b1 * m1 + b2 * m2 + b3 * m3 + b4 * m4, 0.0, 1.0);

            // Text region bounding box (after all char offsets)
            // Original uses last suv state (after -= 4.0)
            let bvr = bv4;
            let rr = step(0.0, bvr.x + 0.333 * 13.0)
                   * step(0.0, bvr.y + 0.2)
                   * step(0.0, -bvr.x + 0.333 * 4.0)
                   * step(0.0, -bvr.y + 0.2 * 6.0);

            // Original slogo compositing:
            // l = hsv2rgb(vec3(b + iTime/40, 0.1, rr - b*1.9)) * rr
            // l -= 0.1 - clamp(ul*0.1, rr*1-b, 0.1)
            // return vec3(l.x, clamp(l.x,0,1)-l.x, clamp(-l.x,0,1))

            // HSV: hue = b + t/40, sat = 0.1, val = rr - b*1.9
            let logo_hue = b + t * 0.025;
            let logo_val = rr - b * 1.9;
            let lh = abs(fract(logo_hue + 1.0) * 6.0 - 3.0);
            let logo_rgb = logo_val * mix(1.0, clamp(lh - 1.0, 0.0, 1.0), 0.1);
            let l_raw = logo_rgb * rr;
            let l = l_raw - (0.1 - clamp(ul * 0.1, rr * 1.0 - b, 0.1));

            // slogo returns: vec3(l.x, clamp(l.x,0,1)-l.x, clamp(-l.x,0,1))
            let logo = vec3(l, clamp(l, 0.0, 1.0) - l, clamp(-l, 0.0, 1.0));

            // Composite: blend logo over FBM background
            // Logo positive = warm tint, logo negative = dark cutout
            let logo_strength = abs(l) * 2.0 * rr;
            col = mix(col, col * (1.0 + logo * 2.5), clamp(logo_strength, 0.0, 1.0));

            // Light vignette
            let dist = length((uv - 0.5) * vec2(ar, 1.0));
            let vignette = smoothstep(1.2, 0.2, dist);
            col = col * vignette;

            return vec4(
                clamp(col.x, 0.0, 1.0),
                clamp(col.y, 0.0, 1.0),
                clamp(col.z, 0.0, 1.0),
                1.0
            );
        }
    }

    animator: Animator{
        anim: {
            // default must NOT be `on` (see ShaderCanvas note): play() would
            // early-return and anim_time would never advance.
            default: @off
            off: AnimatorState{ from: {all: Forward {duration: 0.0}} }
            on: AnimatorState{
                from: {all: Loop {duration: 20.0, end: 1.0}}
                redraw: true
                apply: {
                    draw_bg: {
                        anim_time: [{time: 0.0, value: 0.0}, {time: 1.0, value: 125.66}]
                    }
                }
            }
        }
    }
}

// ============================================================
// Shader Math Canvas - Jellyfish point-cloud (forward mapping)
// ============================================================
mod.widgets.ShaderMathCanvasBase = #(ShaderMathCanvas::register_widget(vm))
mod.widgets.ShaderMathCanvas = set_type_default() do mod.widgets.ShaderMathCanvasBase{
    speed: 1.0

    draw_bg +: {
        anim_time: instance(0.0)
        speed: instance(1.0)

        // Jellyfish point-cloud shader (minimal GPU version)
        pixel: fn() {
            let t = self.anim_time * self.speed;
            let aspect = self.rect_size.x / self.rect_size.y;

            let px = (self.pos.x - 0.5) * 900.0 * aspect;
            let py = (self.pos.y - 0.5) * 900.0;

            // Deep-sea background
            let depth = self.pos.y;
            let caustic = sin(self.pos.x * 25.0 + t * 0.4)
                        * sin(self.pos.y * 18.0 - t * 0.25) * 0.008;
            let mut o = vec4(
                0.005 + caustic,
                0.012 + depth * 0.015 + caustic,
                0.04  + depth * 0.03  + caustic * 2.0,
                1.0
            );

            // Single jellyfish: 10×12 = 120 points flat loop
            for i in 0..120 {
                let fi = float(i);
                let ix = fi - 10.0 * floor(fi / 10.0);
                let iy = floor(fi / 10.0);

                let x = ix * 17.0;
                let y = iy * 15.5;

                let k = 5.0 * cos(x / 14.0) * cos(y / 30.0);
                let e = y / 8.0 - 13.0;
                let d = (k * k + e * e) / 59.0 + 4.0;

                let bell = 1.0 + 0.8 * exp(-(d - 4.0));

                let q = 60.0 - 3.0 * sin(atan2(k, e))
                      + k * (3.0 + 4.0 / d * sin(d * d - 2.0 * t));
                let c = d / 2.0 + e / 99.0 - t / 18.0;

                let u = 3.0 * q * sin(c) * bell + sin(t * 0.04) * 25.0;
                let v = 3.0 * (q + 9.0 * d) * cos(c) * bell + cos(t * 0.035) * 20.0;

                let dx = u - px;
                let dy = v - py;
                let dist2 = dx * dx + dy * dy;

                if dist2 < 600.0 {
                    let glow = exp(-dist2 / 8.0) * 0.35
                             + exp(-dist2 / 50.0) * 0.08
                             + exp(-dist2 / 250.0) * 0.02;

                    let hue = y * 0.016 + k * 0.12
                            + atan2(k, e) * 0.25 + d * 0.06;
                    o = o + vec4(
                        glow * (0.5 + 0.5 * cos(6.2832 * hue)),
                        glow * (0.5 + 0.5 * cos(6.2832 * (hue - 0.33))),
                        glow * (0.5 + 0.5 * cos(6.2832 * (hue - 0.67))),
                        0.0
                    );
                }
            }

            // Tone mapping
            return vec4(
                o.x / (1.0 + o.x),
                o.y / (1.0 + o.y),
                o.z / (1.0 + o.z),
                1.0
            );
        }
    }

    animator: Animator{
        anim: {
            // default must NOT be `on` (see ShaderCanvas note): play() would
            // early-return and anim_time would never advance.
            default: @off
            off: AnimatorState{ from: {all: Forward {duration: 0.0}} }
            on: AnimatorState{
                from: {all: Loop {duration: 20.0, end: 1.0}}
                redraw: true
                apply: {
                    draw_bg: {
                        anim_time: [{time: 0.0, value: 0.0}, {time: 1.0, value: 125.66}]
                    }
                }
            }
        }
    }
}

// ============================================================
// SplashDemo - Natural Language UI Generation
// ============================================================
mod.widgets.SplashDemoBase = #(SplashDemo::register_widget(vm))
mod.widgets.SplashDemo = set_type_default() do mod.widgets.SplashDemoBase{
    width: Fill, height: Fill,
    flow: Down,
    spacing: 20,
    padding: Inset{ left: 24, right: 24, top: 24, bottom: 100 }

    show_bg: true
    draw_bg +: { color: #x1e1e2eff }

    // Header
    View {
        width: Fill, height: Fit,
        flow: Down,
        spacing: 8,

        SectionHeader{
            draw_text +: { color: #xcdd6f4ff }
            text: "Natural Language UI Generation"
        }

        Label {
            width: Fill, height: Fit,
            draw_text +: {
                text_style: theme.font_regular{ font_size: 13.0 }
                color: #xa6adc8ff
            }
            text: "Type commands to dynamically generate UI widgets in real-time."
        }
    }

    mod.widgets.MpDivider { draw_bg +: { color: #x313244ff } }

    // Command Input Section
    View {
        width: Fill, height: Fit,
        flow: Down,
        spacing: 12,

        Label {
            draw_text +: {
                text_style: theme.font_bold{ font_size: 14.0 }
                color: #x89b4faff
            }
            text: "Command Input"
        }

        // Example commands
        View {
            width: Fill, height: Fit,
            padding: 12,
            show_bg: true
            draw_bg +: { color: #x313244ff }

            Label {
                width: Fill, height: Fit,
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 12.0 }
                    color: #x6c7086ff
                }
                text: "Commands: \"add button Submit\" | \"add label Hello World\" | \"add card User Profile\" | \"add progress 75\" | \"add switch Dark Mode\" | \"clear\""
            }
        }

        View {
            width: Fill, height: Fit,
            flow: Right,
            spacing: 12,
            align: Align{ y: 0.5 }

            command_input := TextInput{
                width: Fill, height: Fit,
                padding: 12,
                empty_text: "Type a command... e.g. 'add button Click Me'"
                draw_bg +: {
                    color: #x313244ff
                }
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 14.0 }
                    color: #xcdd6f4ff
                }
            }

            generate_btn := mod.widgets.MpButtonProminent{ text: "Generate" }
            clear_btn := mod.widgets.MpButtonGhost{
                draw_text +: { color: #xf38ba8ff }
                text: "Clear All"
            }
        }
    }

    mod.widgets.MpDivider { draw_bg +: { color: #x313244ff } }

    // Generated UI Section
    View {
        width: Fill, height: Fit,
        flow: Right,
        align: Align{ y: 0.5 }

        Label {
            draw_text +: {
                text_style: theme.font_bold{ font_size: 14.0 }
                color: #x89b4faff
            }
            text: "Generated UI"
        }

        View { width: Fill, height: 1 }

        widget_count_label := Label{
            draw_text +: {
                text_style: theme.font_bold{ font_size: 14.0 }
                color: #xa6e3a1ff
            }
            text: "0 widgets"
        }
    }

    // Dynamic PortalList for generated widgets
    generated_list := PortalList{
        width: Fill, height: 400,
        flow: Down,

        // Button template
        let GenButton = View{
            width: Fill, height: Fit,
            padding: 8,
            margin: Inset{ bottom: 8 }

            gen_button := mod.widgets.MpButtonProminent{
                width: Fit
                text: "Button"
            }
        }

        // Label template
        let GenLabel = View{
            width: Fill, height: Fit,
            padding: Inset{ left: 12, right: 12, top: 16, bottom: 16 }
            margin: Inset{ bottom: 8 }
            show_bg: true
            draw_bg +: { color: #x313244ff }

            gen_label := Label{
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 14.0 }
                    color: #xcdd6f4ff
                }
                text: "Label"
            }
        }

        // Card template
        let GenCard = mod.widgets.MpCard{
            width: Fill, height: Fit,
            margin: Inset{ bottom: 8 }
            padding: 16,

            View {
                width: Fill, height: Fit,
                flow: Down,
                spacing: 8,

                card_title := Label{
                    draw_text +: {
                        text_style: theme.font_bold{ font_size: 16.0 }
                        color: #xcdd6f4ff
                    }
                    text: "Card Title"
                }

                Label {
                    draw_text +: {
                        text_style: theme.font_regular{ font_size: 13.0 }
                        color: #xa6adc8ff
                    }
                    text: "This is a dynamically generated card widget."
                }
            }
        }

        // Progress template
        let GenProgress = View{
            width: Fill, height: Fit,
            padding: 12,
            margin: Inset{ bottom: 8 }
            show_bg: true
            draw_bg +: { color: #x313244ff }
            flow: Down,
            spacing: 8,

            progress_label := Label{
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 12.0 }
                    color: #xa6adc8ff
                }
                text: "Progress: 50%"
            }

            gen_progress := mod.widgets.MpProgress{
                width: Fill, height: 8,
                value: 50
            }
        }

        // Switch template
        let GenSwitch = View{
            width: Fill, height: Fit,
            padding: 12,
            margin: Inset{ bottom: 8 }
            show_bg: true
            draw_bg +: { color: #x313244ff }
            flow: Right,
            align: Align{ y: 0.5 }
            spacing: 12,

            switch_label := Label{
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 14.0 }
                    color: #xcdd6f4ff
                }
                text: "Toggle"
            }

            View { width: Fill, height: 1 }

            gen_switch := mod.widgets.MpSwitch{}
        }

        // Input template
        let GenInput = View{
            width: Fill, height: Fit,
            padding: 8,
            margin: Inset{ bottom: 8 }
            flow: Down,
            spacing: 8,

            input_label := Label{
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 12.0 }
                    color: #xa6adc8ff
                }
                text: "Input Field"
            }

            gen_input := TextInput{
                width: Fill, height: Fit,
                padding: 10,
                empty_text: "Enter text..."
                draw_bg +: { color: #x45475aff }
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 14.0 }
                    color: #xcdd6f4ff
                }
            }
        }
    }
}

// ============================================================
// JsonRenderDemo - JSON-based Dynamic UI Generation
// ============================================================
mod.widgets.JsonRenderDemoBase = #(JsonRenderDemo::register_widget(vm))
mod.widgets.JsonRenderDemo = set_type_default() do mod.widgets.JsonRenderDemoBase{
    width: Fill, height: Fill,
    flow: Down,
    spacing: 20,
    padding: Inset{ left: 24, right: 24, top: 24, bottom: 100 }

    show_bg: true
    draw_bg +: { color: #x1e1e2eff }

    // Header
    View {
        width: Fill, height: Fit,
        flow: Down,
        spacing: 8,

        SectionHeader{
            draw_text +: { color: #xcdd6f4ff }
            text: "JSON Render - A2UI Protocol"
        }

        Label {
            width: Fill, height: Fit,
            draw_text +: {
                text_style: theme.font_regular{ font_size: 13.0 }
                color: #xa6adc8ff
            }
            text: "Parse JSON schema to dynamically render Makepad UI components. Supports nested layouts and component properties."
        }
    }

    mod.widgets.MpDivider { draw_bg +: { color: #x313244ff } }

    // Main content area - two columns
    View {
        width: Fill, height: Fill,
        flow: Right,
        spacing: 20,

        // Left: JSON Editor
        View {
            width: Fill, height: Fill,
            flow: Down,
            spacing: 12,

            Label {
                draw_text +: {
                    text_style: theme.font_bold{ font_size: 14.0 }
                    color: #x89b4faff
                }
                text: "JSON Schema"
            }

            json_input := TextInput{
                width: Fill, height: Fill,
                padding: 12,
                empty_text: "Enter JSON UI schema..."
                draw_bg +: { color: #x313244ff }
                draw_text +: {
                    text_style: theme.font_regular{ font_size: 12.0 }
                    color: #xcdd6f4ff
                }
            }

            View {
                width: Fill, height: Fit,
                flow: Right,
                spacing: 12,

                render_btn := mod.widgets.MpButtonProminent{ text: "Render" }
                clear_render_btn := mod.widgets.MpButtonGhost{
                    draw_text +: { color: #xf38ba8ff }
                    text: "Clear"
                }
                View { width: Fill }
                load_example_btn := mod.widgets.MpButtonGhost{ text: "Basic Example" }
                load_raycast_btn := mod.widgets.MpButtonGhost{ text: "Raycast Examples" }
            }
        }

        // Right: Rendered Preview
        View {
            width: Fill, height: Fill,
            flow: Down,
            spacing: 12,

            View {
                width: Fill, height: Fit,
                flow: Right,
                align: Align{ y: 0.5 }

                Label {
                    draw_text +: {
                        text_style: theme.font_bold{ font_size: 14.0 }
                        color: #x89b4faff
                    }
                    text: "Rendered Preview"
                }

                View { width: Fill }

                render_status := Label{
                    draw_text +: {
                        text_style: theme.font_bold{ font_size: 12.0 }
                        color: #xa6e3a1ff
                    }
                    text: "Ready"
                }
            }

            // Preview container with border
            preview_container := View{
                width: Fill, height: Fill,
                padding: 16,
                show_bg: true
                draw_bg +: { color: #x11111bff }

                // Dynamic content rendered via PortalList
                json_list := PortalList{
                    width: Fill, height: Fill,
                    flow: Down,

                    // View container template
                    let JsonView = View{
                        width: Fill, height: Fit,
                        padding: 8,
                        margin: Inset{ bottom: 4 }
                        show_bg: true
                        draw_bg +: { color: #x1e1e2eff }
                        flow: Down,
                        spacing: 8,
                    }

                    // HStack template
                    let JsonHStack = View{
                        width: Fill, height: Fit,
                        padding: 8,
                        margin: Inset{ bottom: 4 }
                        flow: Right,
                        spacing: 8,
                    }

                    // Label template
                    let JsonLabel = View{
                        width: Fill, height: Fit,
                        padding: 8,
                        margin: Inset{ bottom: 4 }

                        json_label_text := Label{
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 14.0 }
                                color: #xcdd6f4ff
                            }
                            text: "Label"
                        }
                    }

                    // Button template
                    let JsonButton = View{
                        width: Fit, height: Fit,
                        margin: Inset{ bottom: 4 }

                        json_button := mod.widgets.MpButtonProminent{
                            text: "Button"
                        }
                    }

                    // Card template
                    let JsonCard = mod.widgets.MpCard{
                        width: Fill, height: Fit,
                        margin: Inset{ bottom: 8 }
                        padding: 16,

                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8,

                            json_card_title := Label{
                                draw_text +: {
                                    text_style: theme.font_bold{ font_size: 16.0 }
                                    color: #xcdd6f4ff
                                }
                                text: "Card"
                            }

                            json_card_desc := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 13.0 }
                                    color: #xa6adc8ff
                                }
                                text: "Card description"
                            }
                        }
                    }

                    // Progress template
                    let JsonProgress = View{
                        width: Fill, height: Fit,
                        padding: 12,
                        margin: Inset{ bottom: 4 }
                        show_bg: true
                        draw_bg +: { color: #x313244ff }
                        flow: Down,
                        spacing: 8,

                        json_progress_label := Label{
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: #xa6adc8ff
                            }
                            text: "Progress"
                        }

                        json_progress := mod.widgets.MpProgress{
                            width: Fill, height: 8,
                            value: 50
                        }
                    }

                    // Switch template
                    let JsonSwitch = View{
                        width: Fill, height: Fit,
                        padding: 12,
                        margin: Inset{ bottom: 4 }
                        show_bg: true
                        draw_bg +: { color: #x313244ff }
                        flow: Right,
                        align: Align{ y: 0.5 }
                        spacing: 12,

                        json_switch_label := Label{
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 14.0 }
                                color: #xcdd6f4ff
                            }
                            text: "Switch"
                        }

                        View { width: Fill }

                        json_switch := mod.widgets.MpSwitch{}
                    }

                    // TextInput template
                    let JsonInput = View{
                        width: Fill, height: Fit,
                        padding: 8,
                        margin: Inset{ bottom: 4 }
                        flow: Down,
                        spacing: 4,

                        json_input_label := Label{
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: #xa6adc8ff
                            }
                            text: "Input"
                        }

                        json_text_input := TextInput{
                            width: Fill, height: Fit,
                            padding: 10,
                            empty_text: "Enter text..."
                            draw_bg +: { color: #x45475aff }
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 14.0 }
                                color: #xcdd6f4ff
                            }
                        }
                    }

                    // Image placeholder template
                    let JsonImage = View{
                        width: Fill, height: 120,
                        margin: Inset{ bottom: 4 }
                        show_bg: true
                        draw_bg +: { color: #x313244ff }
                        align: Align{ x: 0.5, y: 0.5 }

                        Label {
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: #x6c7086ff
                            }
                            text: "[Image Placeholder]"
                        }
                    }

                    // Divider template
                    let JsonDivider = mod.widgets.MpDivider{
                        margin: Inset{ top: 8, bottom: 8 }
                        draw_bg +: { color: #x313244ff }
                    }
                }
            }
        }
    }
}

// App is registered via app_main! macro
startup() do #(App::script_component(vm)){
    ui: Root{
        main_window := Window{
            window.title: "Component Zoo"
            window.inner_size: vec2(1280, 900)

            show_bg: true
            draw_bg +: { color: BG }

            body := View{
                width: Fill,
                height: Fill,
                flow: Overlay,

                // Main content area
                main_content := View{
                    width: Fill,
                    height: Fill,
                    flow: Down,

                // Header area
                View {
                    width: Fill, height: Fit,
                    flow: Down,
                    padding: Inset{ left: 24, right: 24, top: 24, bottom: 16 },
                    spacing: 8,

                    Label {
                        draw_text +: {
                            text_style: theme.font_bold{ font_size: 24.0 }
                            color: TEXT
                        }
                        text: "Component Zoo"
                    }

                    Label {
                        draw_text +: {
                            text_style: theme.font_regular{ font_size: 14.0 }
                            color: TEXT_MUTED
                        }
                        text: "A showcase of makepad-component widgets"
                    }
                }

                // Category Tab Bar
                View {
                    width: Fill, height: Fit,
                    padding: Inset{ left: 24, right: 24, bottom: 16 },

                    mod.widgets.MpTabBarPill {
                        cat_shadcn := CategoryTab{ text: "Shadcn" }
                        cat_form := CategoryTab{ text: "Form" }
                        cat_display := CategoryTab{ text: "Display" }
                        cat_nav := CategoryTab{ text: "Navigation" }
                        cat_feedback := CategoryTab{ text: "Feedback" }
                        cat_data := CategoryTab{ text: "Data" }
                        cat_shader := CategoryTab{ text: "Shader" }
                        cat_shader_art := CategoryTab{ text: "Shader Art" }
                        cat_shader_art2 := CategoryTab{ text: "Shader FBM" }
                        cat_shader_math := CategoryTab{ text: "Shader Math" }
                        cat_splash := CategoryTab{ text: "Splash" }
                        cat_json := CategoryTab{ text: "JSON Render" }
                    }
                }

                mod.widgets.MpDivider {}

                // Content area with PageFlip
                category_pages := PageFlip{
                    width: Fill,
                    height: Fill,
                    active_page: @page_shadcn,

                    // ============================================================
                    // Form Controls Page
                    // ============================================================
                    page_form := ScrollYView{
                        width: Fill, height: Fill,
                        flow: Down,
                        spacing: 24,
                        padding: Inset{ left: 24, right: 24, top: 24, bottom: 200 }

                        show_bg: true
                        draw_bg +: { color: #xe2e8f0ff }

                        // ===== Button Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Button" }

                            // Button Variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 12,

                                    btn_primary := mod.widgets.MpButtonProminent{ text: "Primary" }
                                    btn_secondary := mod.widgets.MpButtonGhost{ text: "Secondary" }
                                    btn_danger := mod.widgets.MpButtonDestructive{ text: "Danger" }
                                    btn_ghost := mod.widgets.MpButtonGhost{ text: "Ghost" }
                                }
                            }

                            // Button Sizes
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 12,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpButtonSmall { text: "Small" }
                                    mod.widgets.MpButton { text: "Medium" }
                                    mod.widgets.MpButtonLarge { text: "Large" }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Checkbox Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Checkbox" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 12,

                                checkbox1 := mod.widgets.MpCheckbox{ text: "Option 1" }
                                checkbox2 := mod.widgets.MpCheckbox{ text: "Option 2", checked: true }
                                checkbox3 := mod.widgets.MpCheckbox{ text: "Option 3" }
                                mod.widgets.MpCheckbox{ text: "Disabled", disabled: true }
                                mod.widgets.MpCheckbox{ text: "Disabled on", checked: true, disabled: true }
                                mod.widgets.MpCheckbox{ size: MpSize.Large, text: "Large" }
                            }

                            checkbox_status := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 12.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Selected: Option 2"
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Switch Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Switch" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 12,

                                // Built-in labels: text + label_side, like the
                                // gpui-component Switch label
                                switch_wifi := mod.widgets.MpSwitch{ text: "Wi-Fi" }
                                switch_bluetooth := mod.widgets.MpSwitch{ on: true, text: "Bluetooth" }
                                switch_notifications := mod.widgets.MpSwitch{ text: "Notifications", label_side: MpLabelSide.Left }
                                mod.widgets.MpSwitch{ disabled: true, text: "Disabled" }
                                mod.widgets.MpSwitch{ on: true, disabled: true, text: "Disabled on" }
                            }

                            // Size system (track height 14..26)
                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 12,
                                align: Align{ y: 0.5 }

                                mod.widgets.MpSwitch{ size: MpSize.XSmall, text: "XS" }
                                mod.widgets.MpSwitch{ size: MpSize.Small, text: "S" }
                                mod.widgets.MpSwitch{ size: MpSize.Medium, text: "M" }
                                mod.widgets.MpSwitch{ size: MpSize.Large, text: "L" }
                                mod.widgets.MpSwitch{ size: MpSize.XLarge, text: "XL" }
                            }

                            // Multiple switches
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "All On" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSwitch { on: true }
                                    mod.widgets.MpSwitch { on: true }
                                    mod.widgets.MpSwitch { on: true }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Radio Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Radio" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 12,

                                radio_small := mod.widgets.MpRadio{ text: "Small" }
                                radio_medium := mod.widgets.MpRadio{ text: "Medium", checked: true }
                                radio_large := mod.widgets.MpRadio{ text: "Large" }
                                mod.widgets.MpRadio{ text: "Disabled", disabled: true }
                                mod.widgets.MpRadio{ text: "Disabled on", checked: true, disabled: true }
                            }

                            radio_status := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 12.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Selected: Medium"
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Dropdown Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Dropdown" }

                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Basic" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    dropdown_basic := mod.widgets.MpDropdown{
                                        width: 200,
                                        labels: ["Apple", "Banana", "Cherry", "Date", "Elderberry"]
                                    }

                                    dropdown_status := Label{
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "Selected: Apple"
                                    }
                                }
                            }

                            // Dropdown variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Variants" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,

                                    mod.widgets.MpDropdown {
                                        width: 180,
                                        labels: ["Default", "Option 2", "Option 3"]
                                    }

                                    mod.widgets.MpDropdownOutline {
                                        width: 180,
                                        labels: ["Outline", "Option 2", "Option 3"]
                                    }

                                    mod.widgets.MpDropdownGhost {
                                        width: 180,
                                        labels: ["Ghost", "Option 2", "Option 3"]
                                    }
                                }
                            }

                            // Dropdown sizes
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpDropdownSmall {
                                        width: 140,
                                        labels: ["Small", "Option 2"]
                                    }

                                    mod.widgets.MpDropdown {
                                        width: 150,
                                        labels: ["Medium", "Option 2"]
                                    }

                                    mod.widgets.MpDropdownLarge {
                                        width: 160,
                                        labels: ["Large", "Option 2"]
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Slider Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Slider" }

                            // Default Slider
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Default" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    slider_default := mod.widgets.MpSlider{
                                        width: 300,
                                        min: 0.0, max: 100.0, value: 50.0, step: 1.0,
                                    }

                                    slider_default_label := Label{
                                        width: 100, height: Fit,
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT
                                        }
                                        text: "Value: 50"
                                    }
                                }
                            }

                            // Slider Colors
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Colors" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    mod.widgets.MpSlider {
                                        width: 300,
                                        min: 0.0, max: 100.0, value: 60.0, step: 1.0,
                                    }

                                    mod.widgets.MpSliderSuccess {
                                        width: 300,
                                        min: 0.0, max: 100.0, value: 80.0, step: 1.0,
                                    }

                                    mod.widgets.MpSliderWarning {
                                        width: 300,
                                        min: 0.0, max: 100.0, value: 40.0, step: 1.0,
                                    }

                                    mod.widgets.MpSliderDanger {
                                        width: 300,
                                        min: 0.0, max: 100.0, value: 20.0, step: 1.0,
                                    }
                                }
                            }

                            // Slider Sizes
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 10,

                                    mod.widgets.MpSlider {
                                        width: 300,
                                        size: MpSize.XSmall,
                                        min: 0.0, max: 100.0, value: 20.0, step: 1.0,
                                    }
                                    mod.widgets.MpSlider {
                                        width: 300,
                                        size: MpSize.Medium,
                                        min: 0.0, max: 100.0, value: 50.0, step: 1.0,
                                    }
                                    mod.widgets.MpSlider {
                                        width: 300,
                                        size: MpSize.XLarge,
                                        min: 0.0, max: 100.0, value: 80.0, step: 1.0,
                                    }
                                }
                            }

                            // Vertical Slider
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Vertical" }

                                View {
                                    width: Fill, height: 150,
                                    flow: Right,
                                    spacing: 16,

                                    slider_vert := mod.widgets.MpSliderVertical{
                                        height: Fill,
                                        min: 0.0, max: 100.0, value: 30.0, step: 1.0,
                                    }

                                    slider_vert_label := Label{
                                        width: 120, height: Fit,
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT
                                        }
                                        text: "Vertical value: 30"
                                    }
                                }
                            }

                            // Range Slider
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Range Slider" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    slider_range := mod.widgets.MpSlider{
                                        width: 300,
                                        min: 0.0, max: 100.0,
                                        value_start: 20.0, value: 80.0,
                                        range_mode: true, step: 1.0,
                                    }

                                    slider_range_label := Label{
                                        width: 150, height: Fit,
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT
                                        }
                                        text: "Range: 20 - 80"
                                    }
                                }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    slider_range_success := mod.widgets.MpSliderSuccess{
                                        width: 300,
                                        min: 0.0, max: 100.0,
                                        value_start: 30.0, value: 70.0,
                                        range_mode: true, step: 5.0,
                                    }

                                    slider_range_success_label := Label{
                                        width: 150, height: Fit,
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT
                                        }
                                        text: "Range: 30 - 70 (step 5)"
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Input Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Input" }

                            // Input variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Variants" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,

                                    mod.widgets.MpInput {
                                        width: 200,
                                        empty_text: "Default input"
                                    }

                                    mod.widgets.MpInputBorderless {
                                        width: 200,
                                        empty_text: "Borderless input"
                                    }
                                }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,

                                    mod.widgets.MpInputPassword {
                                        width: 200,
                                        input +: { empty_text: "Password input" }
                                    }

                                    mod.widgets.MpInputNumeric {
                                        width: 200,
                                        empty_text: "Numbers only"
                                    }
                                }
                            }

                            // Input sizes
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpInputSmall {
                                        width: 150,
                                        empty_text: "Small"
                                    }

                                    mod.widgets.MpInput {
                                        width: 150,
                                        empty_text: "Medium"
                                    }

                                    mod.widgets.MpInputLarge {
                                        width: 150,
                                        empty_text: "Large"
                                    }
                                }
                            }

                            // Interactive input
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Interactive" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    input_interactive := mod.widgets.MpInput{
                                        width: 250,
                                        empty_text: "Type something..."
                                    }

                                    input_status := Label{
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "Value: (empty)"
                                    }
                                }
                            }
                        }

                    }

                    // ============================================================
                    // Display Page
                    // ============================================================
                    page_display := ScrollYView{
                        width: Fill, height: Fill,
                        flow: Down,
                        spacing: 24,
                        padding: Inset{ left: 24, right: 24, top: 24, bottom: 100 }

                        show_bg: true
                        draw_bg +: { color: #xbbf7d0ff }

                        // ===== Label Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Label" }

                            // Size variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Size Variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 1.0 }

                                    mod.widgets.MpLabelXs { text: "Extra Small" }
                                    mod.widgets.MpLabelSm { text: "Small" }
                                    mod.widgets.MpLabel { text: "Medium (default)" }
                                    mod.widgets.MpLabelLg { text: "Large" }
                                    mod.widgets.MpLabelXl { text: "Extra Large" }
                                }
                            }

                            // Color variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Color Variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,

                                    mod.widgets.MpLabel { text: "Default" }
                                    mod.widgets.MpLabelMuted { text: "Muted" }
                                    mod.widgets.MpLabelPrimary { text: "Primary" }
                                    mod.widgets.MpLabelSuccess { text: "Success" }
                                    mod.widgets.MpLabelWarning { text: "Warning" }
                                    mod.widgets.MpLabelDanger { text: "Danger" }
                                    mod.widgets.MpLabelInfo { text: "Info" }
                                }
                            }

                            // Headings
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Headings" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    mod.widgets.MpHeading1 { text: "Heading 1" }
                                    mod.widgets.MpHeading2 { text: "Heading 2" }
                                    mod.widgets.MpHeading3 { text: "Heading 3" }
                                    mod.widgets.MpHeading4 { text: "Heading 4" }
                                    mod.widgets.MpHeading5 { text: "Heading 5" }
                                    mod.widgets.MpHeading6 { text: "Heading 6" }
                                }
                            }

                            // Secondary text
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "With Secondary Text" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    mod.widgets.MpLabel {
                                        text: "Username"
                                        secondary: "(required)"
                                    }
                                    mod.widgets.MpLabel {
                                        text: "Email"
                                        secondary: "optional"
                                    }
                                }
                            }

                            // Masked text
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Masked Text (Password)" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,

                                    mod.widgets.MpLabel {
                                        text: "password123"
                                        masked: true
                                    }
                                    mod.widgets.MpLabel {
                                        text: "secret"
                                        masked: true
                                    }
                                }
                            }

                            // Highlighted text
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Text Highlighting (Search)" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    mod.widgets.MpLabel {
                                        text: "The quick brown fox jumps over the lazy dog"
                                        highlight: "fox"
                                    }
                                    mod.widgets.MpLabel {
                                        text: "Hello World, Hello Universe"
                                        highlight: "hello"
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Text Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Text" }

                            // Paragraph text
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Paragraph Text (Word Wrap)" }

                                View {
                                    width: 400, height: Fit,
                                    padding: 16,
                                    show_bg: true,
                                    draw_bg +: { color: #xffffffff }

                                    mod.widgets.MpText {
                                        text: "This is a paragraph of text that demonstrates word wrapping. When the text is too long to fit on a single line, it automatically wraps to the next line. This is useful for displaying longer content like descriptions or articles."
                                    }
                                }
                            }

                            // Size variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Size Variants" }

                                View {
                                    width: 400, height: Fit,
                                    flow: Down,
                                    spacing: 12,
                                    padding: 16,
                                    show_bg: true,
                                    draw_bg +: { color: #xffffffff }

                                    mod.widgets.MpTextXs { text: "Extra small text for fine print" }
                                    mod.widgets.MpTextSm { text: "Small text for captions" }
                                    mod.widgets.MpText { text: "Medium text (default body)" }
                                    mod.widgets.MpTextLg { text: "Large text for emphasis" }
                                    mod.widgets.MpTextXl { text: "Extra large text for intro" }
                                }
                            }

                            // Color variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Color Variants" }

                                View {
                                    width: 400, height: Fit,
                                    flow: Down,
                                    spacing: 8,
                                    padding: 16,
                                    show_bg: true,
                                    draw_bg +: { color: #xffffffff }

                                    mod.widgets.MpText { text: "Default text color" }
                                    mod.widgets.MpTextMuted { text: "Muted text for secondary info" }
                                    mod.widgets.MpTextPrimary { text: "Primary colored text" }
                                    mod.widgets.MpTextSuccess { text: "Success message text" }
                                    mod.widgets.MpTextWarning { text: "Warning message text" }
                                    mod.widgets.MpTextDanger { text: "Danger/error message text" }
                                }
                            }

                            // Special variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Special Variants" }

                                View {
                                    width: 500, height: Fit,
                                    flow: Down,
                                    spacing: 16,
                                    padding: 16,
                                    show_bg: true,
                                    draw_bg +: { color: #xffffffff }

                                    // Lead text
                                    mod.widgets.MpTextLead {
                                        text: "This is lead text, perfect for introductory paragraphs that need to stand out."
                                    }

                                    // Inline code
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 4,
                                        align: Align{ y: 0.5 }

                                        mod.widgets.MpTextInline { text: "Use the " }
                                        mod.widgets.MpTextCode { text: "println!()" }
                                        mod.widgets.MpTextInline { text: " macro to print output." }
                                    }

                                    // Caption
                                    mod.widgets.MpTextCaption {
                                        text: "Caption: This is a small caption text often used below images or figures."
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Markdown Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Markdown" }

                            View {
                                width: 460, height: Fit,
                                flow: Down,
                                spacing: 8,
                                padding: 16,
                                show_bg: true,
                                draw_bg +: { color: #xffffffff }

                                demo_markdown := mod.widgets.MpMarkdown{
                                    body: ""
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Badge Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Badge" }

                            // Badge with count (wrapping content)
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Badge with Count" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.5 }

                                    // Default (red)
                                    mod.widgets.MpBadge {
                                        count: 5
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Messages" }
                                        }
                                    }

                                    // Success (green)
                                    mod.widgets.MpBadgeSuccess {
                                        count: 3
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Completed" }
                                        }
                                    }

                                    // Warning (orange)
                                    mod.widgets.MpBadgeWarning {
                                        count: 2
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Pending" }
                                        }
                                    }
                                }
                            }

                            // Badge color variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Color Variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpBadge {
                                        count: 9
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Default" }
                                        }
                                    }

                                    mod.widgets.MpBadgeInfo {
                                        count: 12
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Info" }
                                        }
                                    }

                                    mod.widgets.MpBadgeSecondary {
                                        count: 7
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Secondary" }
                                        }
                                    }
                                }
                            }

                            // Dot badges
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Dot Badges" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpBadgeDot {
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Notifications" }
                                        }
                                    }

                                    mod.widgets.MpBadgeDotSuccess {
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Online" }
                                        }
                                    }

                                    mod.widgets.MpBadgeDotWarning {
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Away" }
                                        }
                                    }
                                }
                            }

                            // Badge sizes
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpBadge {
                                        count: 5
                                        size: MpSize.XSmall
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "XS" }
                                        }
                                    }

                                    mod.widgets.MpBadge {
                                        count: 5
                                        size: MpSize.Small
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Small" }
                                        }
                                    }

                                    mod.widgets.MpBadge {
                                        count: 5
                                        size: MpSize.Medium
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Medium" }
                                        }
                                    }

                                    mod.widgets.MpBadge {
                                        count: 5
                                        size: MpSize.Large
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Large" }
                                        }
                                    }

                                    mod.widgets.MpBadge {
                                        count: 5
                                        size: MpSize.XLarge
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "XL" }
                                        }
                                    }
                                }
                            }

                            // Standalone badges
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Standalone (inline)" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 12,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpBadgeStandalone {
                                        label +: { text: "5" }
                                    }
                                    mod.widgets.MpBadgeStandaloneSuccess {
                                        label +: { text: "New" }
                                    }
                                    mod.widgets.MpBadgeStandaloneWarning {
                                        label +: { text: "99+" }
                                    }
                                    mod.widgets.MpBadgeStandaloneInfo {
                                        label +: { text: "Beta" }
                                    }
                                }
                            }

                            // Interactive badge
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Interactive Count" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    badge_dec_btn := mod.widgets.MpButtonGhost{ text: "-" }
                                    interactive_badge := mod.widgets.MpBadge{
                                        count: 5
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Items" }
                                        }
                                    }
                                    badge_inc_btn := mod.widgets.MpButtonGhost{ text: "+" }

                                    badge_count_label := Label{
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 12.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "Count: 5"
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Avatar Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Avatar" }

                            // Avatar sizes
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpAvatarXSmall { label +: { text: "XS" } }
                                    mod.widgets.MpAvatarSmall { label +: { text: "SM" } }
                                    mod.widgets.MpAvatar { label +: { text: "MD" } }
                                    mod.widgets.MpAvatarLarge { label +: { text: "LG" } }
                                    mod.widgets.MpAvatarXLarge { label +: { text: "XL" } }
                                }
                            }

                            // Avatar colors
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Colors" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 12,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpAvatar { label +: { text: "JD" } }
                                    mod.widgets.MpAvatarPrimary { label +: { text: "AB" } }
                                    mod.widgets.MpAvatarSuccess { label +: { text: "CD" } }
                                    mod.widgets.MpAvatarDanger { label +: { text: "EF" } }
                                    mod.widgets.MpAvatarWarning { label +: { text: "GH" } }
                                }
                            }

                            // Dynamic avatar
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Dynamic (click to change)" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 12,
                                    align: Align{ y: 0.5 }

                                    dynamic_avatar := mod.widgets.MpAvatar{ label +: { text: "??" } }
                                    avatar_change_btn := mod.widgets.MpButtonGhost{ text: "Random Name" }
                                    avatar_name_label := Label{
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "Click button..."
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Card Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Card" }

                            View {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 16,

                                // Basic Card
                                mod.widgets.MpCard {
                                    width: 250,
                                    mod.widgets.MpCardHeader {
                                        mod.widgets.MpCardTitle { text: "Card Title" }
                                        mod.widgets.MpCardDescription { text: "Card description text." }
                                    }
                                    mod.widgets.MpCardContent {
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 14.0 }
                                                color: TEXT
                                            }
                                            text: "This is the card content area."
                                        }
                                    }
                                    mod.widgets.MpCardFooter {
                                        mod.widgets.MpButtonGhost { text: "Cancel" }
                                        mod.widgets.MpButtonProminent { text: "Save" }
                                    }
                                }

                                // Shadow Card
                                mod.widgets.MpCardShadow {
                                    width: 250,
                                    mod.widgets.MpCardHeader {
                                        mod.widgets.MpCardTitle { text: "Shadow Card" }
                                        mod.widgets.MpCardDescription { text: "Card with shadow effect." }
                                    }
                                    mod.widgets.MpCardContent {
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 14.0 }
                                                color: TEXT
                                            }
                                            text: "Shadow creates depth."
                                        }
                                    }
                                }
                            }

                            // Card color variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Color Variants" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 12,

                                    mod.widgets.MpCardSuccess {
                                        width: 180,
                                        padding: 12,
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 14.0 }
                                                color: SUCCESS
                                            }
                                            text: "Success Card"
                                        }
                                    }

                                    mod.widgets.MpCardDanger {
                                        width: 180,
                                        padding: 12,
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 14.0 }
                                                color: DANGER
                                            }
                                            text: "Danger Card"
                                        }
                                    }

                                    mod.widgets.MpCardWarning {
                                        width: 180,
                                        padding: 12,
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 14.0 }
                                                color: #xb45309ff
                                            }
                                            text: "Warning Card"
                                        }
                                    }

                                    mod.widgets.MpCardInfo {
                                        width: 180,
                                        padding: 12,
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 14.0 }
                                                color: INFO
                                            }
                                            text: "Info Card"
                                        }
                                    }
                                }
                            }

                            // Clickable Card
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Clickable Card (hover to see effect)" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 12,

                                    clickable_card_1 := mod.widgets.MpCardClickable{
                                        width: 200,
                                        mod.widgets.MpCardHeader {
                                            mod.widgets.MpCardTitle { text: "Click Me" }
                                            mod.widgets.MpCardDescription { text: "Hover and click" }
                                        }
                                    }

                                    clickable_card_2 := mod.widgets.MpCardClickable{
                                        width: 200,
                                        mod.widgets.MpCardHeader {
                                            mod.widgets.MpCardTitle { text: "Interactive" }
                                            mod.widgets.MpCardDescription { text: "With hover effect" }
                                        }
                                    }

                                    card_click_status := Label{
                                        width: Fit, height: Fit,
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "Click a card..."
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Divider Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Divider" }

                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 20,

                                SubsectionLabel{ text: "Horizontal (default)" }
                                mod.widgets.MpDivider {}

                                SubsectionLabel{ text: "With text" }
                                mod.widgets.MpDividerWithLabel { text: "OR" }

                                SubsectionLabel{ text: "Thick" }
                                mod.widgets.MpDividerWithMargin {}
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Skeleton Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Skeleton" }

                            // Interactive Skeleton Demo
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 8,
                                    align: Align{ y: 0.5 }

                                    skeleton_toggle_btn := mod.widgets.MpButtonProminent{ text: "Toggle Loading" }

                                    skeleton_status := Label{
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "Status: Loading"
                                    }
                                }

                                // Interactive skeleton widget
                                interactive_skeleton := mod.widgets.MpSkeletonWidget{
                                    width: Fill,
                                    height: Fit,

                                    skeleton := View{
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 8

                                        mod.widgets.MpSkeletonRounded { width: 150, height: 20 }
                                        mod.widgets.MpSkeletonRounded { width: Fill, height: 14 }
                                        mod.widgets.MpSkeletonRounded { width: 200, height: 14 }
                                    }

                                    content := View{
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 8

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_bold{ font_size: 16.0 }
                                                color: TEXT
                                            }
                                            text: "Content Loaded!"
                                        }
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 14.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "This is the actual content that appears after loading."
                                        }
                                    }
                                }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Basic shapes" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSkeleton {
                                        width: 200, height: 20
                                    }

                                    mod.widgets.MpSkeletonCircle {
                                        width: 48, height: 48
                                    }
                                }
                            }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Card skeleton" }

                                mod.widgets.MpSkeletonCard {}
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Spinner Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Spinner" }

                            // Size variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Size variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSpinnerXs {}
                                    mod.widgets.MpSpinnerSm {}
                                    mod.widgets.MpSpinnerMd {}
                                    mod.widgets.MpSpinnerLg {}
                                    mod.widgets.MpSpinnerXl {}
                                }
                            }

                            // Color variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Color variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSpinnerPrimary {}
                                    mod.widgets.MpSpinnerSuccess {}
                                    mod.widgets.MpSpinnerWarning {}
                                    mod.widgets.MpSpinnerDanger {}
                                }
                            }

                            // Style variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Style variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSpinnerThin {}
                                    mod.widgets.MpSpinner {}
                                    mod.widgets.MpSpinnerThick {}
                                    mod.widgets.MpSpinnerNoTrack {}
                                }
                            }

                            // Speed variants
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Speed variants" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSpinnerSlow {}
                                    mod.widgets.MpSpinner {}
                                    mod.widgets.MpSpinnerFast {}
                                }
                            }

                            // Alternative styles
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Alternative styles" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSpinnerDots {}
                                    mod.widgets.MpSpinnerPulse {}
                                }
                            }

                            // With label
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "With label" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 32,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpSpinnerWithLabel {}
                                    mod.widgets.MpSpinnerWithLabelVertical {}
                                }
                            }

                            // Orbs (bezel thinking loaders)
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Orbs" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.5 }

                                    mod.widgets.MpOrbCluster {}
                                    mod.widgets.MpOrbRing {}
                                    mod.widgets.MpOrbConverge {}
                                    mod.widgets.MpOrbBloom {}
                                    mod.widgets.MpLoadingWord {}
                                }
                            }
                        }
                    }

                    // ============================================================
                    // Navigation Page
                    // ============================================================
                    page_nav := ScrollYView{
                        width: Fill, height: Fill,
                        flow: Down,
                        spacing: 24,
                        padding: Inset{ left: 24, right: 24, top: 24, bottom: 100 }

                        show_bg: true
                        draw_bg +: { color: #xbfdbfeff }

                        // ===== Tab Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Tab" }

                            // Default tabs
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Default" }

                                mod.widgets.MpTabBar {
                                    tab_home := mod.widgets.MpTab{ text: "Home" }
                                    tab_profile := mod.widgets.MpTab{ text: "Profile" }
                                    tab_settings := mod.widgets.MpTab{ text: "Settings" }
                                }
                            }

                            // Underline tabs
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Underline" }

                                mod.widgets.MpTabBarUnderline {
                                    tab_u_overview := mod.widgets.MpTabUnderline{ text: "Overview" }
                                    tab_u_analytics := mod.widgets.MpTabUnderline{ text: "Analytics" }
                                    tab_u_reports := mod.widgets.MpTabUnderline{ text: "Reports" }
                                }
                            }

                            // Pill tabs
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Pill" }

                                mod.widgets.MpTabBarPill {
                                    tab_p_all := mod.widgets.MpTabPill{ text: "All" }
                                    tab_p_active := mod.widgets.MpTabPill{ text: "Active" }
                                    tab_p_completed := mod.widgets.MpTabPill{ text: "Completed" }
                                }
                            }

                            // Outline tabs
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Outline" }

                                mod.widgets.MpTabBarOutline {
                                    tab_o_day := mod.widgets.MpTabOutline{ text: "Day" }
                                    tab_o_week := mod.widgets.MpTabOutline{ text: "Week" }
                                    tab_o_month := mod.widgets.MpTabOutline{ text: "Month" }
                                }
                            }

                            // Segmented tabs
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Segmented" }

                                mod.widgets.MpTabBarSegmented {
                                    tab_s_list := mod.widgets.MpTabSegmented{ text: "List" }
                                    tab_s_grid := mod.widgets.MpTabSegmented{ text: "Grid" }
                                    tab_s_map := mod.widgets.MpTabSegmented{ text: "Map" }
                                }
                            }

                            tab_status := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 12.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Selected: Home"
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== PageFlip Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "PageFlip" }

                            Label {
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 14.0 }
                                    color: TEXT_MUTED
                                }
                                text: "PageFlip enables switching between different pages/views."
                            }

                            // Page navigation buttons
                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 8,

                                page_btn_a := mod.widgets.MpButtonProminent{ text: "Page A" }
                                page_btn_b := mod.widgets.MpButtonGhost{ text: "Page B" }
                                page_btn_c := mod.widgets.MpButtonGhost{ text: "Page C" }
                            }

                            // PageFlip container
                            View {
                                width: Fill, height: 120,
                                show_bg: true,
                                draw_bg +: {
                                    color: (SURFACE)
                                }

                                demo_page_flip := PageFlip{
                                    width: Fill, height: Fill,
                                    active_page: @page_a,

                                    page_a := View{
                                        width: Fill, height: Fill,
                                        align: Align{ x: 0.5, y: 0.5 }
                                        show_bg: true
                                        draw_bg +: { color: #xdbeafeff }
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_bold{ font_size: 24.0 }
                                                color: ACCENT
                                            }
                                            text: "Page A Content"
                                        }
                                    }

                                    page_b := View{
                                        width: Fill, height: Fill,
                                        align: Align{ x: 0.5, y: 0.5 }
                                        show_bg: true
                                        draw_bg +: { color: #xdcfce7ff }
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_bold{ font_size: 24.0 }
                                                color: SUCCESS
                                            }
                                            text: "Page B Content"
                                        }
                                    }

                                    page_c := View{
                                        width: Fill, height: Fill,
                                        align: Align{ x: 0.5, y: 0.5 }
                                        show_bg: true
                                        draw_bg +: { color: #xfee2e2ff }
                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_bold{ font_size: 24.0 }
                                                color: DANGER
                                            }
                                            text: "Page C Content"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ============================================================
                    // Feedback Page
                    // ============================================================
                    page_feedback := ScrollYView{
                        width: Fill, height: Fill,
                        flow: Down,
                        spacing: 24,
                        padding: Inset{ left: 24, right: 24, top: 24, bottom: 100 }

                        show_bg: true
                        draw_bg +: { color: #xfde68aff }

                        // ===== Tooltip Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Tooltip" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Positions" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    padding: Inset{ top: 40, bottom: 40 }

                                    mod.widgets.MpTooltipTop {
                                        tip: "Tooltip on top"
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Top" }
                                        }
                                    }

                                    mod.widgets.MpTooltipBottom {
                                        tip: "Tooltip on bottom"
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Bottom" }
                                        }
                                    }

                                    mod.widgets.MpTooltipLeft {
                                        tip: "Tooltip on left"
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Left" }
                                        }
                                    }

                                    mod.widgets.MpTooltipRight {
                                        tip: "Tooltip on right"
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Right" }
                                        }
                                    }
                                }
                            }

                            // Delay examples
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Show Delay" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    padding: Inset{ top: 20, bottom: 20 }

                                    mod.widgets.MpTooltipTop {
                                        tip: "Instant tooltip (0s delay)"
                                        show_delay: 0.0
                                        content +: {
                                            mod.widgets.MpButtonOutline { text: "Instant" }
                                        }
                                    }

                                    mod.widgets.MpTooltipTop {
                                        tip: "Default delay (0.3s)"
                                        content +: {
                                            mod.widgets.MpButtonOutline { text: "Default 0.3s" }
                                        }
                                    }

                                    mod.widgets.MpTooltipTop {
                                        tip: "Slow tooltip (1s delay)"
                                        show_delay: 1.0
                                        content +: {
                                            mod.widgets.MpButtonOutline { text: "Slow 1s" }
                                        }
                                    }

                                    mod.widgets.MpTooltipTop {
                                        tip: "Very slow tooltip (2s delay)"
                                        show_delay: 2.0
                                        content +: {
                                            mod.widgets.MpButtonOutline { text: "Very Slow 2s" }
                                        }
                                    }
                                }
                            }

                            // Tooltip on different components
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "On Components" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.5 }
                                    padding: Inset{ top: 20, bottom: 20 }

                                    // Tooltip on Checkbox
                                    mod.widgets.MpTooltipTop {
                                        tip: "Check this to enable feature"
                                        content +: {
                                            mod.widgets.MpCheckbox {
                                                text: "Checkbox"
                                            }
                                        }
                                    }

                                    // Tooltip on Switch
                                    mod.widgets.MpTooltipTop {
                                        tip: "Toggle to turn on/off"
                                        content +: {
                                            View {
                                                width: Fit, height: Fit,
                                                flow: Right,
                                                spacing: 8,
                                                align: Align{ y: 0.5 }
                                                mod.widgets.MpSwitch {}
                                                Label {
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 14.0 }
                                                        color: TEXT
                                                    }
                                                    text: "Switch"
                                                }
                                            }
                                        }
                                    }

                                    // Tooltip on Radio
                                    mod.widgets.MpTooltipTop {
                                        tip: "Select this option"
                                        content +: {
                                            mod.widgets.MpRadio {
                                                text: "Radio"
                                            }
                                        }
                                    }

                                    // Tooltip on Icon/Label
                                    mod.widgets.MpTooltipTop {
                                        tip: "This is an info icon with tooltip"
                                        content +: {
                                            Label {
                                                draw_text +: {
                                                    text_style: theme.font_regular{ font_size: 20.0 }
                                                    color: ACCENT
                                                }
                                                text: "ℹ️"
                                            }
                                        }
                                    }
                                }
                            }

                            // Long text tooltip
                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Long Text" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    padding: Inset{ top: 20, bottom: 20 }

                                    mod.widgets.MpTooltipTop {
                                        tip: "This is a longer tooltip text that provides more detailed information about the element being hovered."
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Long tooltip" }
                                        }
                                    }

                                    mod.widgets.MpTooltipBottom {
                                        tip: "Tooltips can contain helpful hints, keyboard shortcuts, or additional context for users."
                                        content +: {
                                            mod.widgets.MpButtonGhost { text: "Helpful hints" }
                                        }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Progress Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Progress" }

                            // Progress variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                SubsectionLabel{ text: "Values" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    mod.widgets.MpProgress { width: 300, value: 25.0 }
                                    mod.widgets.MpProgress { width: 300, value: 50.0 }
                                    mod.widgets.MpProgress { width: 300, value: 75.0 }
                                    mod.widgets.MpProgress { width: 300, value: 100.0 }
                                }
                            }

                            // Progress colors
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Colors" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    mod.widgets.MpProgress { width: 300, value: 60.0 }
                                    mod.widgets.MpProgressSuccess { width: 300, value: 60.0 }
                                    mod.widgets.MpProgressDanger { width: 300, value: 60.0 }
                                    mod.widgets.MpProgressWarning { width: 300, value: 60.0 }
                                }
                            }

                            // Progress sizes
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 10,

                                    mod.widgets.MpProgress { width: 300, value: 60.0, size: MpSize.XSmall }
                                    mod.widgets.MpProgress { width: 300, value: 60.0, size: MpSize.Small }
                                    mod.widgets.MpProgress { width: 300, value: 60.0, size: MpSize.Medium }
                                    mod.widgets.MpProgress { width: 300, value: 60.0, size: MpSize.Large }
                                    mod.widgets.MpProgress { width: 300, value: 60.0, size: MpSize.XLarge }
                                }
                            }

                            // Progress widths
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Widths" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    mod.widgets.MpProgress { width: 150, value: 50.0 }
                                    mod.widgets.MpProgress { width: 250, value: 50.0 }
                                    mod.widgets.MpProgress { width: 350, value: 50.0 }
                                }
                            }

                            // Interactive progress
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Interactive" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{ y: 0.5 }

                                    progress_dec_btn := mod.widgets.MpButtonGhost{ text: "-10" }
                                    interactive_progress := mod.widgets.MpProgress{ width: 200, value: 50.0 }
                                    progress_inc_btn := mod.widgets.MpButtonGhost{ text: "+10" }

                                    progress_label := Label{
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT
                                        }
                                        text: "50%"
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Alert Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Alert" }

                            // Alert Variants
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Variants" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    mod.widgets.MpAlert {
                                        content +: {
                                            message +: { text: "This is a default alert message." }
                                        }
                                    }

                                    mod.widgets.MpAlertInfo {
                                        content +: {
                                            message +: { text: "This is an info alert for general information." }
                                        }
                                    }

                                    mod.widgets.MpAlertSuccess {
                                        content +: {
                                            message +: { text: "Operation completed successfully!" }
                                        }
                                    }

                                    mod.widgets.MpAlertWarning {
                                        content +: {
                                            message +: { text: "Please review your input before continuing." }
                                        }
                                    }

                                    mod.widgets.MpAlertError {
                                        content +: {
                                            message +: { text: "Something went wrong. Please try again." }
                                        }
                                    }

                                    mod.widgets.MpAlert {
                                        size: MpSize.Small
                                        content +: {
                                            title_wrapper +: { title +: { text: "Small" } }
                                            message +: { text: "Small alert with compact padding and fonts." }
                                        }
                                    }

                                    mod.widgets.MpAlert {
                                        size: MpSize.Medium
                                        content +: {
                                            title_wrapper +: { title +: { text: "Medium" } }
                                            message +: { text: "Medium alert (the default look)." }
                                        }
                                    }

                                    mod.widgets.MpAlert {
                                        size: MpSize.Large
                                        content +: {
                                            title_wrapper +: { title +: { text: "Large" } }
                                            message +: { text: "Large alert with roomier padding and fonts." }
                                        }
                                    }
                                }
                            }

                            // Alert with Title
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "With Title" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    mod.widgets.MpAlertInfo {
                                        content +: {
                                            title_wrapper +: { visible: true, title +: { text: "Information" } }
                                            message +: { text: "This alert has a title for more context." }
                                        }
                                    }

                                    mod.widgets.MpAlertSuccess {
                                        content +: {
                                            title_wrapper +: { visible: true, title +: { text: "Success!" } }
                                            message +: { text: "Your changes have been saved successfully." }
                                        }
                                    }

                                    mod.widgets.MpAlertError {
                                        content +: {
                                            title_wrapper +: { visible: true, title +: { text: "Error" } }
                                            message +: { text: "Failed to connect to the server. Check your network." }
                                        }
                                    }
                                }
                            }

                            // Closable Alert
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Closable" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    closable_alert := mod.widgets.MpAlertInfo{
                                        closable: true
                                        content +: {
                                            message +: { text: "This alert can be closed. Click the X button." }
                                        }
                                    }

                                    closable_alert_warning := mod.widgets.MpAlertWarning{
                                        closable: true
                                        content +: {
                                            title_wrapper +: { visible: true, title +: { text: "Warning" } }
                                            message +: { text: "This is a closable warning with title." }
                                        }
                                    }
                                }
                            }

                            // Banner Alerts
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Banner Style" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 0,

                                    mod.widgets.MpAlertBannerInfo {
                                        content +: {
                                            message +: { text: "Info banner - full width, no border radius" }
                                        }
                                    }

                                    mod.widgets.MpAlertBannerSuccess {
                                        closable: true
                                        content +: {
                                            message +: { text: "Success banner with close button" }
                                        }
                                    }

                                    mod.widgets.MpAlertBannerWarning {
                                        content +: {
                                            message +: { text: "Warning banner alert" }
                                        }
                                    }

                                    mod.widgets.MpAlertBannerError {
                                        closable: true
                                        content +: {
                                            message +: { text: "Error banner - something needs attention!" }
                                        }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Notification Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Notification" }

                            // Interactive Notification Demo
                            View {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8,

                                show_success_notif := mod.widgets.MpButtonProminent{ text: "Success" }
                                show_error_notif := mod.widgets.MpButtonDestructive{ text: "Error" }
                                show_warning_notif := mod.widgets.MpButtonGhost{ text: "Warning" }
                                show_info_notif := mod.widgets.MpButtonProminent{ text: "Info" }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            Label {
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 14.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Notification previews (static):"
                            }

                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                mod.widgets.MpNotification {
                                    content +: {
                                        title +: { text: "Notification" }
                                        message +: { text: "This is a default notification message." }
                                    }
                                }

                                mod.widgets.MpNotificationSuccess {
                                    content +: {
                                        title +: { text: "Success" }
                                        message +: { text: "Operation completed successfully!" }
                                    }
                                }

                                mod.widgets.MpNotificationError {
                                    content +: {
                                        title +: { text: "Error" }
                                        message +: { text: "Something went wrong. Please try again." }
                                    }
                                }

                                mod.widgets.MpNotificationWarning {
                                    content +: {
                                        title +: { text: "Warning" }
                                        message +: { text: "Please review your input before continuing." }
                                    }
                                }

                                mod.widgets.MpNotificationInfo {
                                    content +: {
                                        title +: { text: "Info" }
                                        message +: { text: "Here's some helpful information." }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Modal Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Modal" }

                            // Interactive Modal Demo
                            View {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 16,
                                align: Align{ y: 0.5 }

                                open_modal_btn := mod.widgets.MpButtonProminent{ text: "Open Modal" }

                                modal_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 14.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Click button to open modal"
                                }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            Label {
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 14.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Modal previews (static):"
                            }

                            // Basic Modal preview
                            mod.widgets.MpModal {
                                width: 350,
                                header +: {
                                    title +: { text: "Modal Title" }
                                }
                                body +: {
                                    Label {
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "This is the modal content area."
                                    }
                                }
                                footer +: {
                                    mod.widgets.MpButtonGhost { text: "Cancel" }
                                    mod.widgets.MpButtonProminent { text: "Confirm" }
                                }
                            }

                            // Alert Dialog preview
                            mod.widgets.MpAlertDialog {
                                width: 320,
                                header +: {
                                    title +: { text: "Are you sure?" }
                                }
                                body +: {
                                    Label {
                                        draw_text +: {
                                            text_style: theme.font_regular{ font_size: 14.0 }
                                            color: TEXT_MUTED
                                        }
                                        text: "This action cannot be undone."
                                    }
                                }
                                footer +: {
                                    mod.widgets.MpButtonGhost { text: "Cancel" }
                                    mod.widgets.MpButtonDestructive { text: "Delete" }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Popover Section (Ant Design Style) =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 24,

                            SectionHeader{ text: "Popover" }

                            // ===== Basic Usage =====
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Basic" }
                                Label {
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 13.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "The most basic example. The size of the floating layer depends on the contents region."
                                }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,

                                    mod.widgets.MpPopoverBottom {
                                        trigger: mod.widgets.MpPopoverTrigger.Hover
                                        mod.widgets.MpButtonProminent { text: "Hover me" }
                                        content +: {
                                            Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                        }
                                    }
                                }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            // ===== Trigger Types (Ant Design: hover, focus, click) =====
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Three trigger modes" }
                                Label {
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 13.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Mouse to click, focus and hover."
                                }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 12,

                                    // Hover trigger
                                    mod.widgets.MpPopoverBottom {
                                        trigger: mod.widgets.MpPopoverTrigger.Hover
                                        mod.widgets.MpButton { text: "Hover me" }
                                        content +: {
                                            Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                        }
                                    }

                                    // Focus trigger
                                    mod.widgets.MpPopoverBottom {
                                        trigger: mod.widgets.MpPopoverTrigger.Focus
                                        mod.widgets.MpButton { text: "Focus me" }
                                        content +: {
                                            Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                        }
                                    }

                                    // Click trigger
                                    mod.widgets.MpPopoverBottom {
                                        trigger: mod.widgets.MpPopoverTrigger.Focus
                                        mod.widgets.MpButton { text: "Click me" }
                                        content +: {
                                            Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                        }
                                    }
                                }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            // ===== Placement (12 positions - Ant Design style layout) =====
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Placement" }
                                Label {
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 13.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "There are 12 placement options available."
                                }

                                // Ant Design style placement grid
                                View {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,
                                    align: Align{ x: 0.5 }
                                    padding: Inset{ top: 16 }

                                    // Top row: TL, Top, TR
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 8,

                                        mod.widgets.MpPopoverTopLeft {
                                            mod.widgets.MpButton { width: 80, text: "TL" }
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }

                                        mod.widgets.MpPopoverTop {
                                            mod.widgets.MpButton { width: 80, text: "Top" }
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }

                                        mod.widgets.MpPopoverTopRight {
                                            mod.widgets.MpButton { width: 80, text: "TR" }
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }
                                    }

                                    // Middle section with Left/Right columns
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 156,  // Space between left and right columns

                                        // Left column: LT, Left, LB
                                        View {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 8,

                                            mod.widgets.MpPopoverLeftTop {
                                                mod.widgets.MpButton { width: 80, text: "LT" }
                                                content +: {
                                                    Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                }
                                            }

                                            mod.widgets.MpPopoverLeft {
                                                mod.widgets.MpButton { width: 80, text: "Left" }
                                                content +: {
                                                    Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                }
                                            }

                                            mod.widgets.MpPopoverLeftBottom {
                                                mod.widgets.MpButton { width: 80, text: "LB" }
                                                content +: {
                                                    Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                }
                                            }
                                        }

                                        // Right column: RT, Right, RB
                                        View {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 8,

                                            mod.widgets.MpPopoverRightTop {
                                                mod.widgets.MpButton { width: 80, text: "RT" }
                                                content +: {
                                                    Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                }
                                            }

                                            mod.widgets.MpPopoverRight {
                                                mod.widgets.MpButton { width: 80, text: "Right" }
                                                content +: {
                                                    Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                }
                                            }

                                            mod.widgets.MpPopoverRightBottom {
                                                mod.widgets.MpButton { width: 80, text: "RB" }
                                                content +: {
                                                    Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                }
                                            }
                                        }
                                    }

                                    // Bottom row: BL, Bottom, BR
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 8,

                                        mod.widgets.MpPopoverBottomLeft {
                                            mod.widgets.MpButton { width: 80, text: "BL" }
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }

                                        mod.widgets.MpPopoverBottom {
                                            mod.widgets.MpButton { width: 80, text: "Bottom" }
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }

                                        mod.widgets.MpPopoverBottomRight {
                                            mod.widgets.MpButton { width: 80, text: "BR" }
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }
                                    }
                                }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            // ===== Arrow (Show/Hide) =====
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Arrow" }
                                Label {
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 13.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "You can display an arrow pointing to the target element."
                                }

                                // Arrow variants display (static)
                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 24,

                                    // Arrow pointing up
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 4,

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 11.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "Arrow Up"
                                        }
                                        mod.widgets.MpPopoverArrowUp {
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }
                                    }

                                    // Arrow pointing down
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 4,

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 11.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "Arrow Down"
                                        }
                                        mod.widgets.MpPopoverArrowDown {
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }
                                    }

                                    // Arrow pointing left
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 4,

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 11.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "Arrow Left"
                                        }
                                        mod.widgets.MpPopoverArrowLeft {
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }
                                    }

                                    // Arrow pointing right
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 4,

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 11.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "Arrow Right"
                                        }
                                        mod.widgets.MpPopoverArrowRight {
                                            content +: {
                                                Label { draw_text +: { text_style: theme.font_bold{ font_size: 14.0 }, color: TEXT }, text: "Title" }
                                                Label { draw_text +: { text_style: theme.font_regular{ font_size: 13.0 }, color: TEXT_MUTED }, text: "Content" }
                                            }
                                        }
                                    }
                                }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            // ===== Controlling the close of the dialog =====
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Controlling the close of the dialog" }
                                Label {
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 13.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Use open prop to control the display of the card."
                                }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Overlay,

                                    popover_trigger_btn := mod.widgets.MpButtonProminent{ text: "Click me" }

                                    View {
                                        width: Fit, height: Fit,
                                        margin: Inset{ top: 44 }

                                        interactive_popover := mod.widgets.MpPopoverWidget{
                                            content := mod.widgets.MpPopoverBase{
                                                width: 200, height: Fit,
                                                padding: 12,
                                                flow: Down,
                                                spacing: 8,

                                                Label {
                                                    draw_text +: {
                                                        text_style: theme.font_bold{ font_size: 14.0 }
                                                        color: TEXT
                                                    }
                                                    text: "Title"
                                                }
                                                Label {
                                                    width: Fill,
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 13.0 }
                                                        color: TEXT_MUTED
                                                    }
                                                    text: "Content"
                                                }
                                                Label {
                                                    width: Fill,
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 13.0 }
                                                        color: TEXT_MUTED
                                                    }
                                                    text: "Content"
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            mod.widgets.MpDivider { margin: Inset{ top: 8, bottom: 8 } }

                            // ===== Popover Content Styles =====
                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Content Styles (Static Preview)" }
                                Label {
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 13.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Different content styles for popover."
                                }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 24,
                                    align: Align{ y: 0.0 }

                                    // Basic Popover
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 4,

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 11.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "Basic"
                                        }
                                        mod.widgets.MpPopover {
                                            width: 180,
                                            Label {
                                                width: Fill,
                                                height: Fit,
                                                draw_text +: {
                                                    text_style: theme.font_regular{ font_size: 13.0 }
                                                    color: TEXT
                                                }
                                                text: "Content"
                                            }
                                        }
                                    }

                                    // Popover with Header
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 4,

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 11.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "With Title"
                                        }
                                        mod.widgets.MpPopoverWithHeader {
                                            width: 200,
                                            header +: {
                                                title_label +: { text: "Title" }
                                            }
                                            body +: {
                                                desc_label +: { text: "Content" }
                                            }
                                        }
                                    }

                                    // Menu Popover
                                    View {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 4,

                                        Label {
                                            draw_text +: {
                                                text_style: theme.font_regular{ font_size: 11.0 }
                                                color: TEXT_MUTED
                                            }
                                            text: "Menu"
                                        }
                                        mod.widgets.MpPopoverMenu {
                                            width: 160,
                                            mod.widgets.MpPopoverMenuItem { label +: { text: "Edit" } }
                                            mod.widgets.MpPopoverMenuItem { label +: { text: "Duplicate" } }
                                            mod.widgets.MpPopoverMenuDivider {}
                                            mod.widgets.MpPopoverMenuItemDanger { label +: { text: "Delete" } }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ============================================================
                    // Data Page
                    // ============================================================
                    page_data := ScrollYView{
                        width: Fill, height: Fill,
                        flow: Down,
                        spacing: 24,
                        padding: Inset{ left: 24, right: 24, top: 24, bottom: 100 }

                        show_bg: true
                        draw_bg +: { color: #xfbcfe8ff }

                        // ===== List Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "List" }

                            View {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 24,

                                // Basic List
                                View {
                                    width: 280, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    SubsectionLabel{ text: "Basic List" }

                                    mod.widgets.MpListDivided {
                                        mod.widgets.MpListItem {
                                            mod.widgets.MpListItemContent {
                                                mod.widgets.MpListItemTitle { text: "List Item 1" }
                                                mod.widgets.MpListItemDescription { text: "Description for item 1" }
                                            }
                                        }
                                        mod.widgets.MpListDividerFull {}
                                        mod.widgets.MpListItem {
                                            mod.widgets.MpListItemContent {
                                                mod.widgets.MpListItemTitle { text: "List Item 2" }
                                                mod.widgets.MpListItemDescription { text: "Description for item 2" }
                                            }
                                        }
                                        mod.widgets.MpListDividerFull {}
                                        mod.widgets.MpListItem {
                                            mod.widgets.MpListItemContent {
                                                mod.widgets.MpListItemTitle { text: "List Item 3" }
                                                mod.widgets.MpListItemDescription { text: "Description for item 3" }
                                            }
                                        }
                                    }
                                }

                                // List with avatars
                                View {
                                    width: 280, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    SubsectionLabel{ text: "With Avatar" }

                                    mod.widgets.MpListDivided {
                                        mod.widgets.MpListItem {
                                            mod.widgets.MpListItemLeading {
                                                mod.widgets.MpAvatarSmall { label +: { text: "JD" } }
                                            }
                                            mod.widgets.MpListItemContent {
                                                mod.widgets.MpListItemTitle { text: "John Doe" }
                                                mod.widgets.MpListItemDescription { text: "Software Engineer" }
                                            }
                                        }
                                        mod.widgets.MpListDividerFull {}
                                        mod.widgets.MpListItem {
                                            mod.widgets.MpListItemLeading {
                                                mod.widgets.MpAvatarSmall { label +: { text: "AS" } }
                                            }
                                            mod.widgets.MpListItemContent {
                                                mod.widgets.MpListItemTitle { text: "Alice Smith" }
                                                mod.widgets.MpListItemDescription { text: "Product Manager" }
                                            }
                                        }
                                        mod.widgets.MpListDividerFull {}
                                        mod.widgets.MpListItem {
                                            mod.widgets.MpListItemLeading {
                                                mod.widgets.MpAvatarSmall { label +: { text: "BJ" } }
                                            }
                                            mod.widgets.MpListItemContent {
                                                mod.widgets.MpListItemTitle { text: "Bob Johnson" }
                                                mod.widgets.MpListItemDescription { text: "Designer" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Accordion Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Accordion" }

                            View {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 24,

                                // Basic Accordion
                                View {
                                    width: 320, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    SubsectionLabel{ text: "Basic" }

                                    mod.widgets.MpAccordion {
                                        mod.widgets.MpAccordionItem {
                                            header := mod.widgets.MpAccordionHeader{
                                                label +: { text: "Section 1" }
                                            }
                                            body +: {
                                                Label {
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 14.0 }
                                                        color: TEXT_MUTED
                                                    }
                                                    text: "Content for section 1."
                                                }
                                            }
                                        }

                                        mod.widgets.MpAccordionDivider {}

                                        mod.widgets.MpAccordionItem {
                                            header := mod.widgets.MpAccordionHeader{
                                                label +: { text: "Section 2" }
                                            }
                                            body +: {
                                                Label {
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 14.0 }
                                                        color: TEXT_MUTED
                                                    }
                                                    text: "Content for section 2."
                                                }
                                            }
                                        }

                                        mod.widgets.MpAccordionDivider {}

                                        mod.widgets.MpAccordionItem {
                                            header := mod.widgets.MpAccordionHeader{
                                                label +: { text: "Section 3" }
                                            }
                                            body +: {
                                                Label {
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 14.0 }
                                                        color: TEXT_MUTED
                                                    }
                                                    text: "Content for section 3."
                                                }
                                            }
                                        }
                                    }
                                }

                                // Bordered Accordion
                                View {
                                    width: 320, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    SubsectionLabel{ text: "Bordered" }

                                    mod.widgets.MpAccordionBordered {
                                        mod.widgets.MpAccordionItemBordered {
                                            header := mod.widgets.MpAccordionHeader{
                                                label +: { text: "FAQ Item 1" }
                                            }
                                            body +: {
                                                Label {
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 14.0 }
                                                        color: TEXT_MUTED
                                                    }
                                                    text: "Answer to FAQ 1."
                                                }
                                            }
                                        }

                                        mod.widgets.MpAccordionItemBordered {
                                            header := mod.widgets.MpAccordionHeader{
                                                label +: { text: "FAQ Item 2" }
                                            }
                                            body +: {
                                                Label {
                                                    draw_text +: {
                                                        text_style: theme.font_regular{ font_size: 14.0 }
                                                        color: TEXT_MUTED
                                                    }
                                                    text: "Answer to FAQ 2."
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Interactive Demo =====
                        View {
                            width: Fit, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Interactive Demo" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 16,
                                align: Align{ y: 0.5 }

                                counter_btn := mod.widgets.MpButtonProminent{ text: "Click me!" }

                                counter_label := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 14.0 }
                                        color: TEXT
                                    }
                                    text: "Clicked: 0 times"
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Table Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Table" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Sortable + Selectable" }

                                demo_table := mod.widgets.MpTable{
                                    width: 640, height: 220
                                }

                                table_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Selected row: none"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    demo_table_sm := mod.widgets.MpTable{
                                        width: 640, height: 130
                                        size: MpSize.Small
                                    }

                                    demo_table_lg := mod.widgets.MpTable{
                                        width: 640, height: 170
                                        size: MpSize.Large
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Tree Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Tree" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Expand / Collapse" }

                                demo_tree := mod.widgets.MpTree{
                                    width: 360, height: 240
                                }

                                tree_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Selected item: none"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    demo_tree_sm := mod.widgets.MpTree{
                                        width: 360, height: 110
                                        size: MpSize.Small
                                    }

                                    demo_tree_lg := mod.widgets.MpTree{
                                        width: 360, height: 140
                                        size: MpSize.Large
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Combobox Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Combobox" }

                            View {
                                width: 360, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Type to filter" }

                                demo_combobox := mod.widgets.MpCombobox{
                                    width: Fill
                                }

                                combobox_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Selected: none"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                demo_combobox_sm := mod.widgets.MpComboboxSmall{
                                    width: Fill
                                }

                                demo_combobox_lg := mod.widgets.MpComboboxLarge{
                                    width: Fill
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Stepper Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Stepper" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Numeric +/-" }

                                demo_stepper := mod.widgets.MpStepper{
                                    value: 10.0
                                    step: 1.0
                                    min: 0.0
                                    max: 50.0
                                }

                                stepper_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Value: 10"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Right, spacing: 16,
                                    align: Align{y: 0.5},

                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        mod.widgets.MpStepperSmall{ value: 5.0, step: 1.0, min: 0.0, max: 10.0 }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Small" }
                                    }
                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        mod.widgets.MpStepper{ value: 5.0, step: 1.0, min: 0.0, max: 10.0 }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Medium" }
                                    }
                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        mod.widgets.MpStepperLarge{ value: 5.0, step: 1.0, min: 0.0, max: 10.0 }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Large" }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Context Menu Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Context Menu" }

                            View {
                                width: 480, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Right-click the area" }

                                demo_context_menu := mod.widgets.MpContextMenu{
                                    width: Fill
                                }

                                context_menu_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Selected: none"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    demo_context_menu_sm := mod.widgets.MpContextMenu{
                                        width: Fill
                                        size: MpSize.Small
                                    }

                                    demo_context_menu_lg := mod.widgets.MpContextMenu{
                                        width: Fill
                                        size: MpSize.Large
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Kbd Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Kbd" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 8,

                                mod.widgets.MpKbd{ text: "⌘" }
                                mod.widgets.MpKbd{ text: "K" }
                                mod.widgets.MpKbd{ text: "⇧⌘P" }
                                mod.widgets.MpKbd{ text: "Esc" }
                            }

                            SubsectionLabel{ text: "Sizes" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 12,
                                align: Align{y: 0.5},

                                mod.widgets.MpKbd{ text: "⌘K", size: MpSize.XSmall }
                                mod.widgets.MpKbd{ text: "⌘K", size: MpSize.Small }
                                mod.widgets.MpKbd{ text: "⌘K", size: MpSize.Medium }
                                mod.widgets.MpKbd{ text: "⌘K", size: MpSize.Large }
                                mod.widgets.MpKbd{ text: "⌘K", size: MpSize.XLarge }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Dropdown Menu Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Dropdown Menu" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                demo_dropdown_menu := mod.widgets.MpDropdownMenu{}

                                dropdown_menu_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Selected: none"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: Align{y: 0.5},

                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        demo_dropdown_menu_sm := mod.widgets.MpDropdownMenu{ size: MpSize.Small }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Small" }
                                    }
                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        demo_dropdown_menu_lg := mod.widgets.MpDropdownMenu{ size: MpSize.Large }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Large" }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Bubble Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Bubble" }

                            View {
                                width: 460, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Conversation" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    align: Align{x: 0.0},

                                    bubble_assistant := mod.widgets.MpBubble{
                                        message: "Hi! I can help you set up your project. What are you building?"
                                        variant: mod.widgets.MpBubbleVariant.Secondary
                                    }
                                }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    align: Align{x: 1.0},

                                    bubble_user := mod.widgets.MpBubble{
                                        message: "A Rust CLI that syncs files to S3."
                                        variant: mod.widgets.MpBubbleVariant.Filled
                                    }
                                }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    align: Align{x: 0.0},

                                    mod.widgets.MpBubble{
                                        message: "Got it — here's a starter template."
                                        variant: mod.widgets.MpBubbleVariant.Secondary
                                    }
                                }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    align: Align{x: 1.0},

                                    mod.widgets.MpBubble{
                                        message: "Upload failed. Please retry."
                                        variant: mod.widgets.MpBubbleVariant.Destructive
                                    }
                                }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    align: Align{x: 0.0},

                                    mod.widgets.MpBubble{
                                        message: "Outlined variant for quiet system messages."
                                        variant: mod.widgets.MpBubbleVariant.Outline
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Attachment Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Attachment" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 12,

                                demo_attachment := mod.widgets.MpAttachment{
                                    filename: "quarterly-report.pdf"
                                    meta: "2.4 MB"
                                }

                                demo_attachment_img := mod.widgets.MpAttachment{
                                    filename: "banner.png"
                                    meta: "480 KB"
                                }
                            }

                            attachment_status := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 12.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Removed: none"
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Description List Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Description List" }

                            View {
                                width: 420, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Bordered" }

                                demo_description_list := mod.widgets.MpDescriptionList{}

                                SubsectionLabel{ text: "Plain (unbordered)" }

                                demo_description_list_plain := mod.widgets.MpDescriptionList{
                                    bordered: false
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Group Box Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Group Box" }

                            View {
                                width: 420, height: Fit,
                                flow: Down,
                                spacing: 12,

                                SubsectionLabel{ text: "Variants" }

                                mod.widgets.MpGroupBox{
                                    title +: { text: "Normal" }
                                    content +: {
                                        Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0}, color: TEXT_MUTED } text: "Card background with a border." }
                                    }
                                }

                                mod.widgets.MpGroupBox{
                                    title +: { text: "With Size (Large)" }
                                    size: MpSize.Large
                                    content +: {
                                        Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0}, color: TEXT_MUTED } text: "Padding and title font scale with the size system." }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Step Indicator Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Step Indicator" }

                            View {
                                width: 520, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Click a step to jump" }

                                demo_step_indicator := mod.widgets.MpStepIndicator{
                                    step: 1
                                }

                                step_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Step: 2"
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Stat Cards Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Stat Cards" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 12,

                                mod.widgets.MpStatCard{}
                                stat_card_2 := mod.widgets.MpStatCard{}
                                stat_card_3 := mod.widgets.MpStatCard{}
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Empty State Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Empty State" }

                            empty_state_demo := mod.widgets.MpEmptyState{
                                height: 160
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Menu Bar Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Menu Bar" }

                            View {
                                width: 480, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Horizontal menu bar" }

                                demo_menu_bar := mod.widgets.MpMenuBar{
                                    width: Fill
                                }

                                menu_bar_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Selected: none"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    demo_menu_bar_sm := mod.widgets.MpMenuBar{
                                        width: Fill
                                        size: MpSize.Small
                                    }

                                    demo_menu_bar_lg := mod.widgets.MpMenuBar{
                                        width: Fill
                                        size: MpSize.Large
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Split Pane Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Split Pane" }

                            View {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Drag the divider" }

                                demo_split := mod.widgets.MpSplitPane{
                                    width: Fill, height: 200
                                    left := View{
                                        width: Fill, height: Fill,
                                        flow: Down,
                                        padding: 12,
                                        show_bg: true
                                        draw_bg +: { color: ELEMENT_HOVER }
                                        Label { text: "Left pane" }
                                    }
                                    right := View{
                                        width: Fill, height: Fill,
                                        flow: Down,
                                        padding: 12,
                                        show_bg: true
                                        draw_bg +: { color: SURFACE_RAISED }
                                        Label { text: "Right pane" }
                                    }
                                }

                                split_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Left width: 200"
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Rating Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Rating" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Click a star" }

                                demo_rating := mod.widgets.MpRating{
                                    value: 3
                                }

                                rating_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Rating: 3"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 20,
                                    align: Align{y: 0.5},

                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        mod.widgets.MpRating{ value: 3, size: MpSize.Small }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Small" }
                                    }
                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        mod.widgets.MpRating{ value: 3 }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Medium" }
                                    }
                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        mod.widgets.MpRating{ value: 3, size: MpSize.Large }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Large" }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Color Picker Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Color Picker" }

                            View {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 8,

                                SubsectionLabel{ text: "Pick a swatch" }

                                demo_color_picker := mod.widgets.MpColorPicker{}

                                color_status := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{ font_size: 12.0 }
                                        color: TEXT_MUTED
                                    }
                                    text: "Picked: none"
                                }

                                SubsectionLabel{ text: "Sizes" }

                                View {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 20,
                                    align: Align{y: 0.5},

                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        demo_color_picker_sm := mod.widgets.MpColorPicker{ size: MpSize.Small }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Small" }
                                    }
                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        demo_color_picker_md := mod.widgets.MpColorPicker{}
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Medium" }
                                    }
                                    View {
                                        width: Fit, height: Fit, flow: Down, spacing: 4,
                                        demo_color_picker_lg := mod.widgets.MpColorPicker{ size: MpSize.Large }
                                        Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Large" }
                                    }
                                }
                            }
                        }

                        mod.widgets.MpDivider {}

                        // ===== Chips Section =====
                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 16,

                            SectionHeader{ text: "Chips" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 8,

                                demo_chip1 := mod.widgets.MpChip{ text: "Rust" }
                                demo_chip2 := mod.widgets.MpChip{ text: "Makepad" }
                                demo_chip3 := mod.widgets.MpChip{ text: "GPU" }
                            }

                            SubsectionLabel{ text: "Sizes" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 12,
                                align: Align{y: 0.5},

                                mod.widgets.MpChip{ text: "XS", size: MpSize.XSmall }
                                mod.widgets.MpChip{ text: "Small", size: MpSize.Small }
                                mod.widgets.MpChip{ text: "Medium", size: MpSize.Medium }
                                mod.widgets.MpChip{ text: "Large", size: MpSize.Large }
                                mod.widgets.MpChip{ text: "XLarge", size: MpSize.XLarge }
                            }

                            chips_status := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 12.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Removed: none"
                            }
                        }
                    }

                    mod.widgets.MpDivider {}

                    // ===== Progress Ring Section =====
                    View {
                        width: Fill, height: Fit,
                        flow: Down,
                        spacing: 16,

                        SectionHeader{ text: "Progress Ring" }

                        View {
                            width: Fit, height: Fit,
                            flow: Down,
                            spacing: 8,

                            SubsectionLabel{ text: "Circular progress" }

                            demo_progress_ring := mod.widgets.MpProgressRing{
                                progress: 0.65
                            }

                            ring_status := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 12.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Progress: 65%"
                            }

                            SubsectionLabel{ text: "Sizes" }

                            View {
                                width: Fit, height: Fit,
                                flow: Right,
                                spacing: 16,
                                align: Align{y: 0.5},

                                mod.widgets.MpProgressRing{ progress: 0.65, size: MpSize.XSmall }
                                mod.widgets.MpProgressRing{ progress: 0.65, size: MpSize.Small }
                                mod.widgets.MpProgressRing{ progress: 0.65, size: MpSize.Medium }
                                mod.widgets.MpProgressRing{ progress: 0.65, size: MpSize.Large }
                                mod.widgets.MpProgressRing{ progress: 0.65, size: MpSize.XLarge }
                            }
                        }
                    }

                    mod.widgets.MpDivider {}

                    // ===== Toggle Group Section =====
                    View {
                        width: Fill, height: Fit,
                        flow: Down,
                        spacing: 16,

                        SectionHeader{ text: "Toggle Group" }

                        View {
                            width: Fit, height: Fit,
                            flow: Down,
                            spacing: 8,

                            SubsectionLabel{ text: "Segmented single-select" }

                            demo_toggle_group := mod.widgets.MpToggleGroup{}

                            toggle_group_status := Label{
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 12.0 }
                                    color: TEXT_MUTED
                                }
                                text: "Selected: none"
                            }

                            SubsectionLabel{ text: "Sizes" }

                            View {
                                width: Fill, height: Fit,
                                flow: Right, spacing: 16,
                                align: Align{y: 0.5},

                                View {
                                    width: Fit, height: Fit, flow: Down, spacing: 4,
                                    mod.widgets.MpToggleGroup{ size: MpSize.Small, }
                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Small" }
                                }
                                View {
                                    width: Fit, height: Fit, flow: Down, spacing: 4,
                                    mod.widgets.MpToggleGroup{ size: MpSize.Medium, }
                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Medium" }
                                }
                                View {
                                    width: Fit, height: Fit, flow: Down, spacing: 4,
                                    mod.widgets.MpToggleGroup{ size: MpSize.Large, }
                                    Label { draw_text +: { text_style: theme.font_regular{ font_size: 11.0 }, color: TEXT_FAINT } text: "Large" }
                                }
                            }
                        }
                    }

                    // ===== Scaffolding Section =====
                    View {
                        width: Fill, height: Fit,
                        flow: Down,
                        spacing: 16,

                        SectionHeader{ text: "Scaffolding" }

                        View {
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8,

                            demo_page_header := mod.widgets.MpPageHeader{
                                title := Label{
                                    draw_text +: {
                                        text_style: theme.font_bold{font_size: 20.0}
                                        color: TEXT
                                    }
                                    text: "Project Settings"
                                }
                                subtitle := Label{
                                    draw_text +: {
                                        text_style: theme.font_regular{font_size: 13.0}
                                        color: TEXT_MUTED
                                    }
                                    text: "Configure how this project builds and deploys"
                                }
                            }
                        }

                        mod.widgets.MpGroupBox{
                            width: 480

                            content := View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8,

                                option_card_1 := mod.widgets.MpOptionCard{
                                    width: 150
                                    text: "Standard"
                                    meta_text: "2 vCPU / 4 GB"
                                }
                                option_card_2 := mod.widgets.MpOptionCard{
                                    width: 150
                                    text: "Performance"
                                    meta_text: "4 vCPU / 16 GB"
                                }
                                option_card_3 := mod.widgets.MpOptionCard{
                                    width: 150
                                    text: "Enterprise"
                                    meta_text: "Custom"
                                }
                            }
                        }

                        scaffolding_status := Label{
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: TEXT_MUTED
                            }
                            text: "Selected: none"
                        }
                    }

                    // ===== Status Section =====
                    View{
                        width: Fill, height: Fit,
                        flow: Down,
                        spacing: 16,

                        SectionHeader{ text: "Status" }

                        demo_step_row_1 := mod.widgets.MpStepRow{}
                        demo_step_row_2 := mod.widgets.MpStepRow{}

                        demo_step_output := mod.widgets.MpStepOutput{
                            output_label := {
                                text: "$ cargo build --release\n   Compiling makepad-component v0.1.0\n    Finished release in 42.31s"
                            }
                        }

                        demo_error_strip := mod.widgets.MpErrorStrip{
                            message_label := {
                                text: "Build failed: unresolved import `gemini_live` in crates/gemini-talker"
                            }
                        }

                        demo_warning_strip := mod.widgets.MpWarningStrip{
                            message_label := {
                                text: "4 warnings emitted (unused imports)"
                            }
                        }
                    }

                    // ===== Control Bar Section =====
                    View{
                        width: Fill, height: Fit,
                        flow: Down,
                        spacing: 16,

                        SectionHeader{ text: "Control Bar" }

                        mod.widgets.MpControlBarRounded{
                            leading := {
                                mod.widgets.MpButtonGhost{ text: "Back" }
                                mod.widgets.MpButtonGhost{ text: "Forward" }
                            }
                            center := {
                                mod.widgets.MpButtonProminent{ text: "Play" }
                                Label{
                                    draw_text +: { text_style: theme.font_regular{font_size: 12.5} color: TEXT_MUTED }
                                    text: "0:42 / 3:51"
                                }
                            }
                            trailing := {
                                mod.widgets.MpButtonGhost{ text: "Shuffle" }
                                mod.widgets.MpButtonGhost{ text: "Volume" }
                            }
                        }
                    }

                    // ============================================================
                    // Shader Page - Shadertoy-style fractal effect
                    // ============================================================
                    page_shader := View{
                        width: Fill, height: Fill,
                        flow: Down,
                        padding: 24,
                        spacing: 16,

                        show_bg: true
                        draw_bg +: { color: #x1a1a2eff }

                        SectionHeader{
                            draw_text +: { color: #xffffffff }
                            text: "Shader Art"
                        }

                        Label {
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: #xa0a0a0ff
                            }
                            text: "Code golf shader - fractal rainbow flow"
                        }

                        // Shader display area with animated time
                        shader_canvas := mod.widgets.ShaderCanvas{
                            width: Fill, height: Fill,
                        }
                    }

                    // ============================================================
                    // Shader Art Page - Observer effect
                    // ============================================================
                    page_shader_art := View{
                        width: Fill, height: Fill,
                        flow: Down,
                        padding: 24,
                        spacing: 16,

                        show_bg: true
                        draw_bg +: { color: #x0a0a0fff }

                        SectionHeader{
                            draw_text +: { color: #xffffffff }
                            text: "Shader Art - Observer"
                        }

                        Label {
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: #xa0a0a0ff
                            }
                            text: "Code golf shader - glowing lattice observer effect"
                        }

                        // Speed control
                        View {
                            width: Fill, height: Fit,
                            flow: Right,
                            spacing: 16,
                            align: Align{ y: 0.5 }

                            Label {
                                width: Fit,
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 14.0 }
                                    color: #xffffffff
                                }
                                text: "Speed:"
                            }

                            shader_art_speed := mod.widgets.MpSlider{
                                width: 200, height: 24,
                                min: 0.1,
                                max: 3.0,
                                value: 1.0,
                                step: 0.1,
                            }

                            shader_art_speed_label := Label{
                                width: 60,
                                draw_text +: {
                                    text_style: theme.font_bold{ font_size: 14.0 }
                                    color: #x89b4faff
                                }
                                text: "1.0x"
                            }
                        }

                        // Shader display area
                        shader_art_canvas := mod.widgets.ShaderArtCanvas{
                            width: Fill, height: Fill,
                        }
                    }

                    // ============================================================
                    // Shader FBM Page - Domain warped noise art
                    // ============================================================
                    page_shader_art2 := View{
                        width: Fill, height: Fill,
                        flow: Down,
                        padding: 24,
                        spacing: 16,

                        show_bg: true
                        draw_bg +: { color: #x0a0a0fff }

                        SectionHeader{
                            draw_text +: { color: #xffffffff }
                            text: "Shader Art - FBM Noise"
                        }

                        Label {
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: #xa0a0a0ff
                            }
                            text: "Domain-warped FBM noise with HSV cycling and bitmap text"
                        }

                        // Speed control
                        View {
                            width: Fill, height: Fit,
                            flow: Right,
                            spacing: 16,
                            align: Align{ y: 0.5 }

                            Label {
                                width: Fit,
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 14.0 }
                                    color: #xffffffff
                                }
                                text: "Speed:"
                            }

                            shader_art2_speed := mod.widgets.MpSlider{
                                width: 200, height: 24,
                                min: 0.1,
                                max: 3.0,
                                value: 1.0,
                                step: 0.1,
                            }

                            shader_art2_speed_label := Label{
                                width: 60,
                                draw_text +: {
                                    text_style: theme.font_bold{ font_size: 14.0 }
                                    color: #x89b4faff
                                }
                                text: "1.0x"
                            }
                        }

                        // Shader display area
                        shader_art2_canvas := mod.widgets.ShaderArt2Canvas{
                            width: Fill, height: Fill,
                        }
                    }

                    // ============================================================
                    // Shader Math Page - Parametric flow field
                    // ============================================================
                    page_shader_math := View{
                        width: Fill, height: Fill,
                        flow: Down,
                        padding: 24,
                        spacing: 16,

                        show_bg: true
                        draw_bg +: { color: #x080812ff }

                        SectionHeader{
                            draw_text +: { color: #xffffffff }
                            text: "Shader Math - Jellyfish"
                        }

                        Label {
                            draw_text +: {
                                text_style: theme.font_regular{ font_size: 12.0 }
                                color: #xa0a0a0ff
                            }
                            text: "Point-cloud forward mapping: (x,y) -> (u,v) via parametric formulas"
                        }

                        // Speed control
                        View {
                            width: Fill, height: Fit,
                            flow: Right,
                            spacing: 16,
                            align: Align{ y: 0.5 }

                            Label {
                                width: Fit,
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 14.0 }
                                    color: #xffffffff
                                }
                                text: "Speed:"
                            }

                            shader_math_speed := mod.widgets.MpSlider{
                                width: 200, height: 24,
                                min: 0.1,
                                max: 3.0,
                                value: 1.0,
                                step: 0.1,
                            }

                            shader_math_speed_label := Label{
                                width: 60,
                                draw_text +: {
                                    text_style: theme.font_bold{ font_size: 14.0 }
                                    color: #x89b4faff
                                }
                                text: "1.0x"
                            }
                        }

                        // Shader display area
                        shader_math_canvas := mod.widgets.ShaderMathCanvas{
                            width: Fill, height: Fill,
                        }
                    }

                    // ============================================================
                    // Splash Page - Dynamic scripting showcase
                    // ============================================================
                    page_splash := mod.widgets.SplashDemo{}
                    page_json := mod.widgets.JsonRenderDemo{}
                    page_shadcn := ScrollYView{
                        width: Fill, height: Fill,
                        flow: Down,
                        padding: Inset{left: 24, right: 24, top: 16, bottom: 200},
                        spacing: 20.0,

                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 20.0} color: TEXT }
                            text: "Shadcn-Style Components"
                        }

                        // ============================================================
                        // Theme Switcher
                        // ============================================================
                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Right,
                            spacing: 12.0,
                            align: Align{y: 0.5},
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 10, bottom: 10}

                            Label{
                                draw_text +: { text_style: theme.font_bold{font_size: 14.0} color: TEXT }
                                text: "Theme:"
                            }
                            theme_toggle_btn := mod.widgets.MpButtonOutline{
                                text: "Toggle Dark Mode"
                            }
                            theme_status := Label{
                                draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT_MUTED }
                                text: "Light"
                            }
                        }

                        // ============================================================
                        // Collapsible
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Collapsible"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 4, right: 4, top: 4, bottom: 4}

                            demo_collapsible_1 := View{
                                width: Fill, height: Fit,
                                flow: Down,

                                collapsible_trigger := mod.widgets.MpCollapsibleTrigger{ label: { text: "What is shadcn/ui?" } }
                                collapsible_content := mod.widgets.MpCollapsibleContent{
                                    visible: false
                                    Label{
                                        draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT_MUTED }
                                        text: "shadcn/ui is a collection of re-usable components built with Radix UI and Tailwind CSS. This Makepad port brings the same patterns to native desktop apps."
                                    }
                                }
                            }

                            demo_collapsible_2 := View{
                                width: Fill, height: Fit,
                                flow: Down,

                                collapsible_trigger2 := mod.widgets.MpCollapsibleTrigger{ label: { text: "How does it work?" } }
                                collapsible_content2 := mod.widgets.MpCollapsibleContent{
                                    visible: false
                                    Label{
                                        draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT_MUTED }
                                        text: "Each component follows macOS design conventions while maintaining the composability and variant system from shadcn. Components are built with Makepad's script_mod! DSL and Rust widget structs."
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // Link
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Link"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}

                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Default:" }
                                demo_link := mod.widgets.MpLink{ text: "Click here" }
                            }
                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}

                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Muted:" }
                                mod.widgets.MpLinkMuted{ text: "Learn more" }
                            }
                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}

                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Ghost:" }
                                mod.widgets.MpLinkGhost{ text: "Visit site" }
                            }
                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}

                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Destructive:" }
                                mod.widgets.MpLinkDestructive{ text: "Delete account" }
                            }
                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}

                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Small / Large:" }
                                mod.widgets.MpLinkSmall{ text: "small" }
                                mod.widgets.MpLinkLarge{ text: "LARGE" }
                            }
                        }

                        // ============================================================
                        // Breadcrumb
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Breadcrumb"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            demo_breadcrumb := View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 4.0,
                                align: Align{y: 0.5}

                                mod.widgets.MpBreadcrumbItem{ text: "Home" }
                                mod.widgets.MpBreadcrumbSeparator{ text: "/" }
                                mod.widgets.MpBreadcrumbItem{ text: "Components" }
                                mod.widgets.MpBreadcrumbSeparator{ text: "/" }
                                mod.widgets.MpBreadcrumbItem{ text: "UI" }
                                mod.widgets.MpBreadcrumbSeparator{ text: "/" }
                                mod.widgets.MpBreadcrumbItem{ active: true, text: "Collapsible" }
                            }
                        }

                        // ============================================================
                        // Textarea
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Textarea"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Default:" }
                            demo_textarea := mod.widgets.MpTextArea{ placeholder: "Type your message here..." }

                            Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Small / Medium / Large:" }
                            mod.widgets.MpTextAreaSmall{ placeholder: "Small textarea" }
                            mod.widgets.MpTextArea{ placeholder: "Medium textarea" }
                            mod.widgets.MpTextAreaLarge{ placeholder: "Large textarea" }

                            Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "XLarge:" }
                            mod.widgets.MpTextArea{ size: MpSize.XLarge, placeholder: "Extra large textarea" }
                        }

                        // ============================================================
                        // Sheet
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Sheet"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}

                                demo_sheet_trigger := mod.widgets.MpSheetTrigger{ 0: mod.widgets.MpButtonProminent{ text: "Open Sheet" } }
                            }
                        }

                        // Sheet panel (overlay + content)
                        demo_sheet := mod.widgets.MpSheet{}

                        // ============================================================
                        // Dialog
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Dialog"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            View{
                                width: Fill, height: Fit, flow: Right, spacing: 8.0, align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Click to open:" }
                                demo_dialog_trigger := mod.widgets.MpButtonProminent{ text: "Open Dialog" }
                            }
                        }

                        // ============================================================
                        // Select
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Select"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            View{
                                width: Fill, height: Fit, flow: Right, spacing: 8.0, align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Framework:" }
                                demo_select := mod.widgets.MpSelect{
                                    trigger +: {
                                        label: { text: "Select a framework" }
                                        // placeholder_text removed (not valid in DSL)
                                    }
                                    dropdown +: {
                                        demo_opt_react := mod.widgets.MpSelectOption{ value: "React", label: { text: "React" } }
                                        demo_opt_vue := mod.widgets.MpSelectOption{ value: "Vue", label: { text: "Vue" } }
                                        demo_opt_svelte := mod.widgets.MpSelectOption{ value: "Svelte", label: { text: "Svelte" } }
                                        demo_opt_solid := mod.widgets.MpSelectOption{ value: "Solid", label: { text: "Solid" } }
                                    }
                                }
                            }

                            SubsectionLabel{ text: "Sizes" }

                            View{
                                width: Fill, height: Fit, flow: Down, spacing: 8.0,

                                mod.widgets.MpSelect{
                                    size: MpSize.Small
                                    trigger +: { label: { text: "Small select" } }
                                    dropdown +: {
                                        mod.widgets.MpSelectOption{ value: "A", label: { text: "Option A" } }
                                        mod.widgets.MpSelectOption{ value: "B", label: { text: "Option B" } }
                                    }
                                }

                                mod.widgets.MpSelect{
                                    size: MpSize.Large
                                    trigger +: { label: { text: "Large select" } }
                                    dropdown +: {
                                        mod.widgets.MpSelectOption{ value: "A", label: { text: "Option A" } }
                                        mod.widgets.MpSelectOption{ value: "B", label: { text: "Option B" } }
                                    }
                                }
                            }
                        }
                        // Dialog overlay (hidden unless opened)
                        demo_dialog := mod.widgets.MpDialog{
                            content +: {
                                dialog +: {
                                    header +: {
                                        title +: { text: "Welcome to shadcn Dialog!" }
                                        description +: { text: "This dialog has smooth fade-in animation." }
                                    }
                                    body +: {
                                        Label{ draw_text +: { text_style: theme.font_regular{font_size: 14.0} color: TEXT } text: "Dialog content with custom children." }
                                    }
                                    footer +: {
                                        dialog_close_btn := mod.widgets.MpButtonGhost{ text: "Cancel" }
                                        dialog_confirm_btn := mod.widgets.MpButtonProminent{ text: "Confirm" }
                                    }
                                }
                            }
                        }

                        // Command palette overlay (hidden unless opened via ⌘K)
                        demo_command := mod.widgets.MpCommandPalette{
                            panel +: {
                                list +: {
                                    cmd_group := View{ width: Fill, height: Fit, flow: Down
                                        cmd_search := mod.widgets.MpCommandItem{ label: { text: "Search files..." } shortcut: { text: "⌘K" } }
                                        cmd_new := mod.widgets.MpCommandItem{ label: { text: "New document" } shortcut: { text: "⌘N" } }
                                        cmd_open := mod.widgets.MpCommandItem{ label: { text: "Open project..." } shortcut: { text: "⌘O" } }
                                    }
                                    cmd_group2 := View{ width: Fill, height: Fit, flow: Down
                                        cmd_settings := mod.widgets.MpCommandItem{ label: { text: "Settings" } shortcut: { text: "⌘," } }
                                        cmd_logout := mod.widgets.MpCommandItem{ label: { text: "Sign out" } }
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // Separator
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Separator"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 12.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Default:" }
                            mod.widgets.MpSeparator{}

                            Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "With label:" }
                            mod.widgets.MpSeparatorWithLabel{ label: { text: "OR" } }

                            Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Subtle:" }
                            mod.widgets.MpSeparatorSubtle{}

                            Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Vertical:" }
                            View{ width: Fill, height: 60, flow: Right, spacing: 8.0, align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT } text: "Left" }
                                mod.widgets.MpSeparatorVertical{}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT } text: "Right" }
                            }
                        }

                        // ============================================================
                        // Toggle
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Toggle"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Default:" }
                                demo_toggle_bold := mod.widgets.MpToggle{ text: "Bold" }
                                mod.widgets.MpToggle{ text: "Italic" }
                                mod.widgets.MpToggle{ text: "Underline" }
                            }
                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Ghost:" }
                                mod.widgets.MpToggleGhost{ text: "Bold" }
                                mod.widgets.MpToggleGhost{ text: "Italic" }
                            }
                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "Sizes:" }
                                demo_toggle_sm := mod.widgets.MpToggleSmall{ text: "Small" }
                                mod.widgets.MpToggle{ text: "Medium" }
                                demo_toggle_lg := mod.widgets.MpToggleLarge{ text: "Large" }
                                mod.widgets.MpToggle{ size: MpSize.XSmall, text: "XS" }
                                mod.widgets.MpToggle{ size: MpSize.XLarge, text: "XL" }
                            }
                            View{
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 8.0,
                                align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 12.0} color: TEXT_MUTED } text: "States:" }
                                mod.widgets.MpToggle{ text: "Active", active: true }
                                mod.widgets.MpToggle{ text: "Disabled", disabled: true }
                                mod.widgets.MpToggleGhost{ text: "Ghost off", disabled: true }
                            }
                        }

                        // ============================================================
                        // Pagination
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "Pagination"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            View{
                                width: Fill, height: Fit,
                                align: Align{x: 0.5},

                                mod.widgets.MpPagination{
                                    demo_prev := mod.widgets.MpPaginationPrev{}
                                    demo_page1 := mod.widgets.MpPaginationItem{ text: "1", active: true }
                                    demo_page2 := mod.widgets.MpPaginationItem{ text: "2" }
                                    demo_page3 := mod.widgets.MpPaginationItem{ text: "3" }
                                    mod.widgets.MpPaginationEllipsis{}
                                    demo_page10 := mod.widgets.MpPaginationItem{ text: "10" }
                                    demo_next := mod.widgets.MpPaginationNext{}
                                }
                            }

                            SubsectionLabel{ text: "Sizes" }

                            View{
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,
                                align: Align{x: 0.5},

                                mod.widgets.MpPagination{
                                    size: MpSize.Small
                                    mod.widgets.MpPaginationPrev{}
                                    mod.widgets.MpPaginationItem{ text: "1", active: true }
                                    mod.widgets.MpPaginationItem{ text: "2" }
                                    mod.widgets.MpPaginationItem{ text: "3" }
                                    mod.widgets.MpPaginationNext{}
                                }

                                mod.widgets.MpPagination{
                                    size: MpSize.Large
                                    mod.widgets.MpPaginationPrev{}
                                    mod.widgets.MpPaginationItem{ text: "1", active: true }
                                    mod.widgets.MpPaginationItem{ text: "2" }
                                    mod.widgets.MpPaginationItem{ text: "3" }
                                    mod.widgets.MpPaginationNext{}
                                }
                            }

                            View{
                                width: Fill, height: Fit,
                                align: Align{x: 0.5},

                                mod.widgets.MpPagination{
                                    demo_prev2 := mod.widgets.MpPaginationPrev{}
                                    mod.widgets.MpPaginationItem{ text: "1" }
                                    mod.widgets.MpPaginationItem{ text: "2" }
                                    mod.widgets.MpPaginationItem{ text: "3" }
                                    mod.widgets.MpPaginationItem{ text: "4" }
                                    mod.widgets.MpPaginationItem{ text: "5", active: true }
                                    demo_next2 := mod.widgets.MpPaginationNext{}
                                }
                            }
                        }

                        // ============================================================
                        // HoverCard
                        // ============================================================
                        Label{
                            draw_text +: { text_style: theme.font_bold{font_size: 16.0} color: TEXT }
                            text: "HoverCard"
                        }

                        RoundedView{
                            width: Fill, height: Fit,
                            flow: Down,
                            spacing: 8.0,
                            draw_bg +: { color: SURFACE_CARD, border_radius: 8.0, border_color: BORDER }
                            padding: Inset{left: 16, right: 16, top: 12, bottom: 12}

                            View{
                                width: Fill, height: Fit, flow: Right, spacing: 8.0, align: Align{y: 0.5}
                                Label{ draw_text +: { text_style: theme.font_regular{font_size: 13.0} color: TEXT } text: "Hover 🔍 over this text to see more details (hover_card requires app-level integration)" }
                            }
                        }
                    }
                } // close PageFlip
                } // close main_content

                // Modal overlay - must be after main_content to appear on top
                demo_modal := mod.widgets.MpModalWidget{
                content +: {
                    dialog := mod.widgets.MpModal{
                        width: 400,
                        header +: {
                            title +: { text: "Interactive Modal" }
                        }
                        body +: {
                            Label {
                                width: Fill,
                                height: Fit,
                                draw_text +: {
                                    text_style: theme.font_regular{ font_size: 14.0 }
                                    color: TEXT_MUTED
                                }
                                text: "This is an interactive modal dialog. Click the X button or the backdrop to close it."
                            }
                        }
                        footer +: {
                            modal_cancel_btn := mod.widgets.MpButtonGhost{ text: "Cancel" }
                            modal_confirm_btn := mod.widgets.MpButtonProminent{ text: "Confirm" }
                        }
                    }
                }
            } // close demo_modal

                // Theme state provider
                theme_state := mod.widgets.MpThemeProvider{
                    dark_mode: false,
                }

                // Notification overlay - positioned at top-right
                View {
                    width: Fill,
                    height: Fill,
                    align: Align{ x: 1.0, y: 0.0 }
                    padding: Inset{ top: 20, right: 20 }

                    demo_notification := mod.widgets.MpNotificationWidget{
                        content +: {
                            title +: { text: "Notification" }
                            message +: { text: "This is an interactive notification!" }
                        }
                    }
                }
            } // close body (Overlay)
        }
    }
    }
}

app_main!(App);

// ============================================================
// ShaderCanvas - Animated shader widget
// ============================================================
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct ShaderCanvas {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    area: Area,
    #[rust]
    start_time: f64,
    #[rust]
    next_frame: Option<NextFrame>,
}

impl Widget for ShaderCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // Self-driven next-frame loop keeps the shader animating independent of
        // the Animator track (which can fail to start when its default state is
        // the same as the target state — see play() early-return).
        if let Some(nf) = &self.next_frame {
            if nf.is_event(event).is_some() {
                self.next_frame = Some(cx.new_next_frame());
                self.redraw(cx);
            }
        }
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Drive anim_time straight from the draw clock. The animator is kept as
        // a secondary redraw source, but we don't rely on it for the time value:
        // pushing any instance via script_apply(Animate) rewrites the whole
        // instance set, which would otherwise reset anim_time to 0 each frame.
        let now = cx.time();
        if self.start_time < 0.0 {
            self.start_time = now;
        }
        let anim_time = (now - self.start_time) * 2.0 * std::f64::consts::PI;
        cx.with_vm(|vm| {
            let obj = vm.bx.heap.new_object();
            vm.bx
                .heap
                .set_value(obj, id!(anim_time).into(), anim_time.into(), NoTrap);
            self.draw_bg
                .script_apply(vm, &Apply::Animate, &mut Scope::default(), obj.into());
        });
        if !self.animator_in_state(cx, ids!(anim.on)) {
            self.animator_play(cx, ids!(anim.on));
        }
        // Always re-arm the next-frame loop on every draw (matches the
        // A2uiSurface shader-stage pattern). Guarding on is_none() could leave
        // a stale token behind when NextFrame events stop while the window is
        // occluded/minimized/suspended, so the loop would never restart after
        // the window becomes visible again (frozen canvas).
        self.next_frame = Some(cx.new_next_frame());
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

// ============================================================
// ShaderArtCanvas - Observer shader widget
// ============================================================
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct ShaderArtCanvas {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    area: Area,
    #[live]
    speed: f64,
    #[rust]
    start_time: f64,
    #[rust]
    next_frame: Option<NextFrame>,
}

impl Widget for ShaderArtCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Some(nf) = &self.next_frame {
            if nf.is_event(event).is_some() {
                self.next_frame = Some(cx.new_next_frame());
                self.redraw(cx);
            }
        }
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Drive anim_time from the draw clock so the shader keeps moving. The
        // per-frame speed push below uses script_apply(Animate), which rewrites
        // the whole instance set and would otherwise reset anim_time to 0.
        let now = cx.time();
        if self.start_time < 0.0 {
            self.start_time = now;
        }
        let anim_time = (now - self.start_time) * 2.0 * std::f64::consts::PI;
        cx.with_vm(|vm| {
            let obj = vm.bx.heap.new_object();
            vm.bx
                .heap
                .set_value(obj, id!(anim_time).into(), anim_time.into(), NoTrap);
            vm.bx
                .heap
                .set_value(obj, id!(speed).into(), self.speed.into(), NoTrap);
            self.draw_bg
                .script_apply(vm, &Apply::Animate, &mut Scope::default(), obj.into());
        });
        if !self.animator_in_state(cx, ids!(anim.on)) {
            self.animator_play(cx, ids!(anim.on));
        }
        // Always re-arm the next-frame loop on every draw (matches the
        // A2uiSurface shader-stage pattern). Guarding on is_none() could leave
        // a stale token behind when NextFrame events stop while the window is
        // occluded/minimized/suspended, so the loop would never restart after
        // the window becomes visible again (frozen canvas).
        self.next_frame = Some(cx.new_next_frame());
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

// ============================================================
// ShaderArt2Canvas - FBM noise art widget
// ============================================================
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct ShaderArt2Canvas {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    area: Area,
    #[live]
    speed: f64,
    #[rust]
    start_time: f64,
    #[rust]
    next_frame: Option<NextFrame>,
}

impl Widget for ShaderArt2Canvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Some(nf) = &self.next_frame {
            if nf.is_event(event).is_some() {
                self.next_frame = Some(cx.new_next_frame());
                self.redraw(cx);
            }
        }
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Drive anim_time from the draw clock so the shader keeps moving. The
        // per-frame speed push below uses script_apply(Animate), which rewrites
        // the whole instance set and would otherwise reset anim_time to 0.
        let now = cx.time();
        if self.start_time < 0.0 {
            self.start_time = now;
        }
        let anim_time = (now - self.start_time) * 2.0 * std::f64::consts::PI;
        cx.with_vm(|vm| {
            let obj = vm.bx.heap.new_object();
            vm.bx
                .heap
                .set_value(obj, id!(anim_time).into(), anim_time.into(), NoTrap);
            vm.bx
                .heap
                .set_value(obj, id!(speed).into(), self.speed.into(), NoTrap);
            self.draw_bg
                .script_apply(vm, &Apply::Animate, &mut Scope::default(), obj.into());
        });
        if !self.animator_in_state(cx, ids!(anim.on)) {
            self.animator_play(cx, ids!(anim.on));
        }
        // Always re-arm the next-frame loop on every draw (matches the
        // A2uiSurface shader-stage pattern). Guarding on is_none() could leave
        // a stale token behind when NextFrame events stop while the window is
        // occluded/minimized/suspended, so the loop would never restart after
        // the window becomes visible again (frozen canvas).
        self.next_frame = Some(cx.new_next_frame());
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

// ============================================================
// ShaderMathCanvas - Parametric flow field widget
// ============================================================
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct ShaderMathCanvas {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,
    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    area: Area,
    #[live]
    speed: f64,
    #[rust]
    start_time: f64,
    #[rust]
    next_frame: Option<NextFrame>,
}

impl Widget for ShaderMathCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if let Some(nf) = &self.next_frame {
            if nf.is_event(event).is_some() {
                self.next_frame = Some(cx.new_next_frame());
                self.redraw(cx);
            }
        }
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Drive anim_time from the draw clock so the shader keeps moving. The
        // per-frame speed push below uses script_apply(Animate), which rewrites
        // the whole instance set and would otherwise reset anim_time to 0.
        let now = cx.time();
        if self.start_time < 0.0 {
            self.start_time = now;
        }
        let anim_time = (now - self.start_time) * 2.0 * std::f64::consts::PI;
        cx.with_vm(|vm| {
            let obj = vm.bx.heap.new_object();
            vm.bx
                .heap
                .set_value(obj, id!(anim_time).into(), anim_time.into(), NoTrap);
            vm.bx
                .heap
                .set_value(obj, id!(speed).into(), self.speed.into(), NoTrap);
            self.draw_bg
                .script_apply(vm, &Apply::Animate, &mut Scope::default(), obj.into());
        });
        if !self.animator_in_state(cx, ids!(anim.on)) {
            self.animator_play(cx, ids!(anim.on));
        }
        // Always re-arm the next-frame loop on every draw (matches the
        // A2uiSurface shader-stage pattern). Guarding on is_none() could leave
        // a stale token behind when NextFrame events stop while the window is
        // occluded/minimized/suspended, so the loop would never restart after
        // the window becomes visible again (frozen canvas).
        self.next_frame = Some(cx.new_next_frame());
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

// ============================================================
// SplashDemo - Natural Language UI Generation
// ============================================================

// Widget type enum for dynamic generation
#[derive(Clone, Debug)]
pub enum GeneratedWidget {
    Button { text: String },
    Label { text: String },
    Card { title: String },
    Progress { value: f64 },
    Switch { label: String },
    Input { placeholder: String },
}

#[derive(Script, Widget)]
pub struct SplashDemo {
    #[deref]
    view: View,
    #[rust]
    widgets: Vec<GeneratedWidget>,
}

impl ScriptHook for SplashDemo {}

impl SplashDemo {
    // Parse natural language command and return widget type
    fn parse_command(&self, input: &str) -> Option<GeneratedWidget> {
        let input = input.trim().to_lowercase();

        // Parse "add <type> <content>" pattern
        if let Some(rest) = input.strip_prefix("add ") {
            let rest = rest.trim();

            // Button: "add button Submit"
            if let Some(text) = rest.strip_prefix("button ") {
                return Some(GeneratedWidget::Button {
                    text: text.trim().to_string(),
                });
            }
            if rest == "button" {
                return Some(GeneratedWidget::Button {
                    text: "Button".to_string(),
                });
            }

            // Label: "add label Hello World"
            if let Some(text) = rest.strip_prefix("label ") {
                return Some(GeneratedWidget::Label {
                    text: text.trim().to_string(),
                });
            }
            if rest == "label" {
                return Some(GeneratedWidget::Label {
                    text: "Label".to_string(),
                });
            }

            // Card: "add card User Profile"
            if let Some(title) = rest.strip_prefix("card ") {
                return Some(GeneratedWidget::Card {
                    title: title.trim().to_string(),
                });
            }
            if rest == "card" {
                return Some(GeneratedWidget::Card {
                    title: "Card".to_string(),
                });
            }

            // Progress: "add progress 75"
            if let Some(val) = rest.strip_prefix("progress ") {
                if let Ok(v) = val.trim().parse::<f64>() {
                    return Some(GeneratedWidget::Progress {
                        value: (v / 100.0).clamp(0.0, 1.0),
                    });
                }
            }
            if rest == "progress" {
                return Some(GeneratedWidget::Progress { value: 0.5 });
            }

            // Switch: "add switch Dark Mode"
            if let Some(label) = rest.strip_prefix("switch ") {
                return Some(GeneratedWidget::Switch {
                    label: label.trim().to_string(),
                });
            }
            if rest == "switch" {
                return Some(GeneratedWidget::Switch {
                    label: "Toggle".to_string(),
                });
            }

            // Input: "add input Email address"
            if let Some(placeholder) = rest.strip_prefix("input ") {
                return Some(GeneratedWidget::Input {
                    placeholder: placeholder.trim().to_string(),
                });
            }
            if rest == "input" {
                return Some(GeneratedWidget::Input {
                    placeholder: "Enter text...".to_string(),
                });
            }
        }

        None
    }
}

impl Widget for SplashDemo {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Handle button actions
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Generate button clicked
        if self
            .view
            .mp_button(cx, ids!(generate_btn))
            .clicked(&actions)
        {
            let input_text = self.view.text_input(cx, ids!(command_input)).text();

            if input_text.trim().to_lowercase() == "clear" {
                self.widgets.clear();
            } else if let Some(widget) = self.parse_command(&input_text) {
                self.widgets.push(widget);
            }

            // Update count label
            self.view
                .label(cx, ids!(widget_count_label))
                .set_text(cx, &format!("{} widgets", self.widgets.len()));

            // Clear input
            self.view
                .text_input(cx, ids!(command_input))
                .set_text(cx, "");
            self.redraw(cx);
        }

        // Clear button clicked
        if self.view.mp_button(cx, ids!(clear_btn)).clicked(&actions) {
            self.widgets.clear();
            self.view
                .label(cx, ids!(widget_count_label))
                .set_text(cx, "0 widgets");
            self.redraw(cx);
        }

        // Handle Enter key in text input
        if let Event::KeyDown(ke) = event {
            if ke.key_code == KeyCode::ReturnKey {
                let input_text = self.view.text_input(cx, ids!(command_input)).text();

                if input_text.trim().to_lowercase() == "clear" {
                    self.widgets.clear();
                } else if let Some(widget) = self.parse_command(&input_text) {
                    self.widgets.push(widget);
                }

                self.view
                    .label(cx, ids!(widget_count_label))
                    .set_text(cx, &format!("{} widgets", self.widgets.len()));
                self.view
                    .text_input(cx, ids!(command_input))
                    .set_text(cx, "");
                self.redraw(cx);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.widgets.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if let Some(widget_def) = self.widgets.get(item_id) {
                        match widget_def {
                            GeneratedWidget::Button { text } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenButton));
                                item_widget.mp_button(cx, ids!(gen_button)).set_text(text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Label { text } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenLabel));
                                item_widget.label(cx, ids!(gen_label)).set_text(cx, text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Card { title } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenCard));
                                item_widget.label(cx, ids!(card_title)).set_text(cx, title);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Progress { value } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenProgress));
                                let percent = (*value * 100.0) as u32;
                                item_widget
                                    .label(cx, ids!(progress_label))
                                    .set_text(cx, &format!("Progress: {}%", percent));
                                item_widget
                                    .mp_progress(cx, ids!(gen_progress))
                                    .set_value(cx, percent as f64);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Switch { label } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenSwitch));
                                item_widget
                                    .label(cx, ids!(switch_label))
                                    .set_text(cx, label);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Input { placeholder } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenInput));
                                item_widget
                                    .label(cx, ids!(input_label))
                                    .set_text(cx, placeholder);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                        }
                    }
                }
            }
        }
        DrawStep::done()
    }
}

// ============================================================
// JsonRenderDemo - JSON-based Dynamic UI Generation
// ============================================================

use serde::{Deserialize, Serialize};

/// JSON Widget types for A2UI protocol
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JsonWidget {
    View {
        #[serde(default)]
        props: JsonViewProps,
        #[serde(default)]
        children: Vec<JsonWidget>,
    },
    HStack {
        #[serde(default)]
        props: JsonStackProps,
        #[serde(default)]
        children: Vec<JsonWidget>,
    },
    VStack {
        #[serde(default)]
        props: JsonStackProps,
        #[serde(default)]
        children: Vec<JsonWidget>,
    },
    Label {
        #[serde(default)]
        props: JsonLabelProps,
    },
    Button {
        #[serde(default)]
        props: JsonButtonProps,
    },
    Card {
        #[serde(default)]
        props: JsonCardProps,
    },
    Progress {
        #[serde(default)]
        props: JsonProgressProps,
    },
    Switch {
        #[serde(default)]
        props: JsonSwitchProps,
    },
    TextInput {
        #[serde(default)]
        props: JsonInputProps,
    },
    Image {
        #[serde(default)]
        props: JsonImageProps,
    },
    Divider,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonViewProps {
    #[serde(default)]
    pub padding: Option<f64>,
    #[serde(default)]
    pub spacing: Option<f64>,
    #[serde(default)]
    pub background: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonStackProps {
    #[serde(default)]
    pub spacing: Option<f64>,
    #[serde(default)]
    pub align: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonLabelProps {
    #[serde(default)]
    pub text: String,
    #[serde(rename = "fontSize", default)]
    pub font_size: Option<f64>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub bold: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonButtonProps {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonCardProps {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonProgressProps {
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonSwitchProps {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub checked: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonInputProps {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JsonImageProps {
    #[serde(default)]
    pub src: Option<String>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
}

/// Flattened widget for PortalList rendering
#[derive(Clone, Debug)]
pub enum FlatWidget {
    Label { text: String },
    Button { text: String },
    Card { title: String, description: String },
    Progress { value: f64, label: String },
    Switch { label: String },
    Input { label: String, placeholder: String },
    Image,
    Divider,
}

#[derive(Script, Widget)]
pub struct JsonRenderDemo {
    #[deref]
    view: View,
    #[rust]
    flat_widgets: Vec<FlatWidget>,
}

impl ScriptHook for JsonRenderDemo {}

impl JsonRenderDemo {
    /// Parse JSON string into JsonWidget tree
    fn parse_json(&self, json: &str) -> Result<JsonWidget, String> {
        // Try to extract JSON from markdown code block
        let json_str = if json.contains("```json") {
            json.split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .map(|s| s.trim())
                .unwrap_or(json.trim())
        } else if json.contains("```") {
            json.split("```")
                .nth(1)
                .map(|s| s.trim())
                .unwrap_or(json.trim())
        } else {
            json.trim()
        };

        serde_json::from_str(json_str).map_err(|e| format!("JSON Parse Error: {}", e))
    }

    /// Flatten widget tree for PortalList rendering
    fn flatten_widgets(widget: &JsonWidget, result: &mut Vec<FlatWidget>) {
        match widget {
            JsonWidget::View { children, .. }
            | JsonWidget::VStack { children, .. }
            | JsonWidget::HStack { children, .. } => {
                for child in children {
                    Self::flatten_widgets(child, result);
                }
            }
            JsonWidget::Label { props } => {
                result.push(FlatWidget::Label {
                    text: props.text.clone(),
                });
            }
            JsonWidget::Button { props } => {
                result.push(FlatWidget::Button {
                    text: props.text.clone(),
                });
            }
            JsonWidget::Card { props } => {
                result.push(FlatWidget::Card {
                    title: props.title.clone(),
                    description: props.description.clone().unwrap_or_default(),
                });
            }
            JsonWidget::Progress { props } => {
                result.push(FlatWidget::Progress {
                    value: props.value,
                    label: props
                        .label
                        .clone()
                        .unwrap_or_else(|| format!("{}%", props.value as i32)),
                });
            }
            JsonWidget::Switch { props } => {
                result.push(FlatWidget::Switch {
                    label: props.label.clone(),
                });
            }
            JsonWidget::TextInput { props } => {
                result.push(FlatWidget::Input {
                    label: props.label.clone().unwrap_or_default(),
                    placeholder: props
                        .placeholder
                        .clone()
                        .unwrap_or_else(|| "Enter text...".to_string()),
                });
            }
            JsonWidget::Image { .. } => {
                result.push(FlatWidget::Image);
            }
            JsonWidget::Divider => {
                result.push(FlatWidget::Divider);
            }
        }
    }

    /// Get example JSON for demonstration
    fn get_example_json() -> &'static str {
        r#"{
  "type": "VStack",
  "props": { "spacing": 16 },
  "children": [
    {
      "type": "Card",
      "props": {
        "title": "User Profile",
        "description": "Dynamically generated card"
      }
    },
    {
      "type": "Label",
      "props": { "text": "Welcome to JSON Render!", "bold": true }
    },
    {
      "type": "HStack",
      "props": { "spacing": 12 },
      "children": [
        { "type": "Button", "props": { "text": "Submit" } },
        { "type": "Button", "props": { "text": "Cancel" } }
      ]
    },
    {
      "type": "Progress",
      "props": { "value": 75, "label": "Loading: 75%" }
    },
    {
      "type": "Switch",
      "props": { "label": "Dark Mode" }
    },
    {
      "type": "TextInput",
      "props": { "label": "Email", "placeholder": "Enter your email" }
    },
    { "type": "Divider" },
    {
      "type": "Label",
      "props": { "text": "Generated via A2UI Protocol" }
    }
  ]
}"#
    }

    /// Get Raycast-style example JSON (simplified format for Component Zoo)
    fn get_raycast_example_json() -> &'static str {
        r#"{"type":"VStack","props":{"spacing":16},"children":[
{"type":"Card","props":{"title":"Create New User","description":"Raycast form"}},
{"type":"Label","props":{"text":"Username"}},
{"type":"TextInput","props":{"label":"Username","placeholder":"Enter username"}},
{"type":"Label","props":{"text":"Password"}},
{"type":"TextInput","props":{"label":"Password","placeholder":"Enter password"}},
{"type":"HStack","props":{"spacing":12},"children":[
{"type":"Switch","props":{"label":"Admin"}},
{"type":"Switch","props":{"label":"Active"}}
]},
{"type":"Divider"},
{"type":"HStack","props":{"spacing":12},"children":[
{"type":"Button","props":{"text":"Create"}},
{"type":"Button","props":{"text":"Cancel"}}
]},
{"type":"Progress","props":{"value":45,"label":"Progress: 45%"}},
{"type":"Divider"},
{"type":"Label","props":{"text":"Use A2UI Demo for full Raycast components"}}
]}"#
    }
}

impl Widget for JsonRenderDemo {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Render button clicked
        if self.view.mp_button(cx, ids!(render_btn)).clicked(&actions) {
            let json_text = self.view.text_input(cx, ids!(json_input)).text();

            match self.parse_json(&json_text) {
                Ok(widget_tree) => {
                    self.flat_widgets.clear();
                    Self::flatten_widgets(&widget_tree, &mut self.flat_widgets);

                    self.view
                        .label(cx, ids!(render_status))
                        .set_text(cx, &format!("{} widgets rendered", self.flat_widgets.len()));
                }
                Err(e) => {
                    self.view
                        .label(cx, ids!(render_status))
                        .set_text(cx, &format!("Error: {}", e));
                }
            }

            self.redraw(cx);
        }

        // Clear button clicked
        if self
            .view
            .mp_button(cx, ids!(clear_render_btn))
            .clicked(&actions)
        {
            self.flat_widgets.clear();
            self.view.text_input(cx, ids!(json_input)).set_text(cx, "");
            self.view
                .label(cx, ids!(render_status))
                .set_text(cx, "Ready");
            self.redraw(cx);
        }

        // Load example button clicked
        if self
            .view
            .mp_button(cx, ids!(load_example_btn))
            .clicked(&actions)
        {
            self.view
                .text_input(cx, ids!(json_input))
                .set_text(cx, Self::get_example_json());
            self.view
                .label(cx, ids!(render_status))
                .set_text(cx, "Basic example loaded");
            self.redraw(cx);
        }

        // Load Raycast example button clicked
        if self
            .view
            .mp_button(cx, ids!(load_raycast_btn))
            .clicked(&actions)
        {
            self.view
                .text_input(cx, ids!(json_input))
                .set_text(cx, Self::get_raycast_example_json());
            self.view
                .label(cx, ids!(render_status))
                .set_text(cx, "Raycast example loaded");
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.flat_widgets.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if let Some(widget_def) = self.flat_widgets.get(item_id) {
                        match widget_def {
                            FlatWidget::Label { text } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonLabel));
                                item_widget
                                    .label(cx, ids!(json_label_text))
                                    .set_text(cx, text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Button { text } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonButton));
                                item_widget.mp_button(cx, ids!(json_button)).set_text(text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Card { title, description } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonCard));
                                item_widget
                                    .label(cx, ids!(json_card_title))
                                    .set_text(cx, title);
                                item_widget
                                    .label(cx, ids!(json_card_desc))
                                    .set_text(cx, description);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Progress { value, label } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonProgress));
                                item_widget
                                    .label(cx, ids!(json_progress_label))
                                    .set_text(cx, label);
                                item_widget
                                    .mp_progress(cx, ids!(json_progress))
                                    .set_value(cx, *value);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Switch { label } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonSwitch));
                                item_widget
                                    .label(cx, ids!(json_switch_label))
                                    .set_text(cx, label);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Input { label, placeholder: _ } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonInput));
                                item_widget
                                    .label(cx, ids!(json_input_label))
                                    .set_text(cx, label);
                                item_widget
                                    .text_input(cx, ids!(json_text_input))
                                    .set_text(cx, "");
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Image => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonImage));
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Divider => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonDivider));
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                        }
                    }
                }
            }
        }
        DrawStep::done()
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    counter: usize,
    #[rust]
    current_page: usize,
    #[rust]
    current_category: usize,
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.counter = 0;
        self.current_category = 0;

        // Set initial category tab as selected
        self.ui.mp_tab(cx, ids!(cat_form)).set_selected(cx, true);

        // Initialize skeleton in loading state
        self.ui
            .mp_skeleton_widget(cx, ids!(interactive_skeleton))
            .set_loading(cx, true);

        // Populate table demo
        self.ui
            .mp_table(cx, ids!(demo_table))
            .set_columns(vec![
                TableColumn::new("Name", 220.0, true),
                TableColumn::new("Language", 140.0, true),
                TableColumn::new("Stars", 100.0, true),
                TableColumn::new("License", 180.0, false),
            ]);
        self.ui.mp_table(cx, ids!(demo_table)).set_rows(
            cx,
            vec![
                vec!["makepad".to_string(), "Rust".to_string(), "12400".to_string(), "MIT".to_string()],
                vec!["tokio".to_string(), "Rust".to_string(), "27800".to_string(), "MIT".to_string()],
                vec!["serde".to_string(), "Rust".to_string(), "9100".to_string(), "Apache-2.0".to_string()],
                vec!["tauri".to_string(), "Rust".to_string(), "86200".to_string(), "MIT".to_string()],
                vec!["wgpu".to_string(), "Rust".to_string(), "13100".to_string(), "MIT OR Apache-2.0".to_string()],
                vec!["bevy".to_string(), "Rust".to_string(), "37400".to_string(), "MIT OR Apache-2.0".to_string()],
            ],
        );

        // Populate tree demo
        let tree_items = vec![
            TreeItem::new("crates", 0),
            TreeItem::new("ui", 1),
            TreeItem::new("widgets", 2),
            TreeItem::new("button.rs", 3),
            TreeItem::new("table.rs", 3),
            TreeItem::new("tree.rs", 3),
            TreeItem::new("a2ui", 2),
            TreeItem::new("processor.rs", 3),
            TreeItem::new("theme", 1),
            TreeItem::new("palette.rs", 2),
            TreeItem::new("color.rs", 2),
            TreeItem::new("motion", 1),
            TreeItem::new("lib.rs", 2),
            TreeItem::new("Cargo.toml", 0),
        ];
        self.ui.mp_tree(cx, ids!(demo_tree)).set_items(cx, tree_items.clone());

        // Populate tree size demos (same structure, fewer rows fit)
        for tree_id in [ids!(demo_tree_sm), ids!(demo_tree_lg)] {
            self.ui.mp_tree(cx, tree_id).set_items(cx, tree_items.clone());
        }

        // Populate table size demos (same columns/rows as the main demo)
        let size_demo_columns = vec![
            TableColumn::new("Name", 160.0, false),
            TableColumn::new("Language", 120.0, false),
        ];
        let size_demo_rows = vec![
            vec!["makepad".to_string(), "Rust".to_string()],
            vec!["tokio".to_string(), "Rust".to_string()],
            vec!["serde".to_string(), "Rust".to_string()],
        ];
        for table_id in [ids!(demo_table_sm), ids!(demo_table_lg)] {
            self.ui.mp_table(cx, table_id).set_columns(size_demo_columns.clone());
            self.ui.mp_table(cx, table_id).set_rows(cx, size_demo_rows.clone());
        }

        // Populate color picker demos: a warm-to-cool palette grid
        let swatch_palette = vec![
            "#1C1917", "#57534E", "#A8A29E", "#E7E5E4", "#FAFAF9",
            "#7F1D1D", "#DC2626", "#F87171", "#FECACA", "#FEF2F2",
            "#9A3412", "#EA580C", "#FB923C", "#FED7AA", "#FFF7ED",
            "#A16207", "#CA8A04", "#FACC15", "#FDE68A", "#FEFCE8",
            "#166534", "#16A34A", "#4ADE80", "#BBF7D0", "#F0FDF4",
            "#155E75", "#0891B2", "#22D3EE", "#A5F3FC", "#ECFEFF",
            "#1E40AF", "#2563EB", "#60A5FA", "#BFDBFE", "#EFF6FF",
            "#5B21B6", "#7C3AED", "#A78BFA", "#DDD6FE", "#F5F3FF",
            "#831843", "#DB2777", "#F472B6", "#FBCFE8", "#FDF2F8",
        ]
        .iter()
        .filter_map(|s| {
            let hex = s.trim_start_matches('#');
            u32::from_str_radix(hex, 16).ok().map(|v| Vec4f {
                x: ((v >> 16) & 0xff) as f32 / 255.0,
                y: ((v >> 8) & 0xff) as f32 / 255.0,
                z: (v & 0xff) as f32 / 255.0,
                w: 1.0,
            })
        })
        .collect::<Vec<_>>();
        self.ui.mp_color_picker(cx, ids!(demo_color_picker)).set_colors(cx, swatch_palette.clone());
        self.ui.mp_color_picker(cx, ids!(demo_color_picker)).set_columns(cx, 5);
        self.ui.mp_color_picker(cx, ids!(demo_color_picker)).set_selected(cx, Some(7));
        for picker_id in [ids!(demo_color_picker_sm), ids!(demo_color_picker_md), ids!(demo_color_picker_lg)] {
            self.ui.mp_color_picker(cx, picker_id).set_colors(cx, swatch_palette.clone());
        }

        // Populate combobox demo
        self.ui
            .mp_combobox(cx, ids!(demo_combobox))
            .set_items(
                cx,
                vec![
                    "Apple".to_string(),
                    "Banana".to_string(),
                    "Cherry".to_string(),
                    "Durian".to_string(),
                    "Elderberry".to_string(),
                    "Fig".to_string(),
                    "Grape".to_string(),
                    "Honeydew".to_string(),
                    "Kiwi".to_string(),
                    "Lemon".to_string(),
                ],
            );

        // Populate combobox size-variant demos (same fruit list)
        for combobox_id in [ids!(demo_combobox_sm), ids!(demo_combobox_lg)] {
            self.ui
                .mp_combobox(cx, combobox_id)
                .set_items(
                    cx,
                    vec![
                        "Apple".to_string(),
                        "Banana".to_string(),
                        "Cherry".to_string(),
                        "Durian".to_string(),
                        "Elderberry".to_string(),
                        "Fig".to_string(),
                        "Grape".to_string(),
                        "Honeydew".to_string(),
                        "Kiwi".to_string(),
                        "Lemon".to_string(),
                    ],
                );
        }

        // Populate markdown demo
        self.ui.markdown(cx, ids!(demo_markdown)).set_text(
            cx,
            "# Markdown rendering\n\nA **bold** word, an *italic* one, and `inline code`.\n\n## Lists\n\n- One\n- Two\n- Three\n\n1. First\n2. Second\n3. Third\n\n## Code block\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\n## Quote\n\n> A blockquote with a border.\n\n## Table\n\n| Name | Type |\n| ---- | ---- |\n| a    | b    |\n\n---\n\n[Open makepad](https://github.com/makepad/makepad)",
        );

        // Populate context menu demo
        self.ui
            .mp_context_menu(cx, ids!(demo_context_menu))
            .set_items(
                cx,
                vec![
                    "Cut".to_string(),
                    "Copy".to_string(),
                    "Paste".to_string(),
                    "Duplicate".to_string(),
                    "Delete".to_string(),
                    "Rename…".to_string(),
                ],
            );

        // Populate context menu size demos (same action list)
        for context_menu_id in [ids!(demo_context_menu_sm), ids!(demo_context_menu_lg)] {
            self.ui
                .mp_context_menu(cx, context_menu_id)
                .set_items(
                    cx,
                    vec![
                        "Cut".to_string(),
                        "Copy".to_string(),
                        "Paste".to_string(),
                        "Duplicate".to_string(),
                        "Delete".to_string(),
                        "Rename…".to_string(),
                    ],
                );
        }

        // Populate dropdown menu demo
        self.ui
            .mp_dropdown_menu(cx, ids!(demo_dropdown_menu))
            .set_items(
                cx,
                &vec![
                    "New file".to_string(),
                    "Open…".to_string(),
                    "Save".to_string(),
                    "Save as…".to_string(),
                    "Export".to_string(),
                ],
            );

        // Populate dropdown menu size demos (same action list)
        for dropdown_menu_id in [ids!(demo_dropdown_menu_sm), ids!(demo_dropdown_menu_lg)] {
            self.ui
                .mp_dropdown_menu(cx, dropdown_menu_id)
                .set_items(
                    cx,
                    &vec![
                        "New file".to_string(),
                        "Open…".to_string(),
                        "Save".to_string(),
                        "Save as…".to_string(),
                        "Export".to_string(),
                    ],
                );
        }

        // Populate menu bar demo
        self.ui
            .mp_menu_bar(cx, ids!(demo_menu_bar))
            .set_trigger_labels(cx, &["File".to_string(), "Edit".to_string(), "View".to_string()]);
        self.ui
            .mp_menu_bar(cx, ids!(demo_menu_bar))
            .set_items(cx, 0, &["New File".to_string(), "Open…".to_string(), "Save".to_string()]);
        self.ui
            .mp_menu_bar(cx, ids!(demo_menu_bar))
            .set_items(cx, 1, &["Undo".to_string(), "Redo".to_string(), "Cut".to_string()]);
        self.ui
            .mp_menu_bar(cx, ids!(demo_menu_bar))
            .set_items(cx, 2, &["Zoom In".to_string(), "Zoom Out".to_string()]);

        // Populate menu bar size demos (same menu structure)
        for menu_bar_id in [ids!(demo_menu_bar_sm), ids!(demo_menu_bar_lg)] {
            self.ui
                .mp_menu_bar(cx, menu_bar_id)
                .set_trigger_labels(cx, &["File".to_string(), "Edit".to_string(), "View".to_string()]);
            self.ui
                .mp_menu_bar(cx, menu_bar_id)
                .set_items(cx, 0, &["New File".to_string(), "Open…".to_string(), "Save".to_string()]);
            self.ui
                .mp_menu_bar(cx, menu_bar_id)
                .set_items(cx, 1, &["Undo".to_string(), "Redo".to_string(), "Cut".to_string()]);
            self.ui
                .mp_menu_bar(cx, menu_bar_id)
                .set_items(cx, 2, &["Zoom In".to_string(), "Zoom Out".to_string()]);
        }

        // Populate description list demos
        let project_info = vec![
            MpDescriptionItem::new("Name", "makepad-component"),
            MpDescriptionItem::new("Language", "Rust"),
            MpDescriptionItem::new("Framework", "Makepad"),
            MpDescriptionItem::new("License", "MIT OR Apache-2.0"),
        ];
        self.ui
            .mp_description_list(cx, ids!(demo_description_list))
            .set_items(cx, &project_info);
        self.ui
            .mp_description_list(cx, ids!(demo_description_list_plain))
            .set_items(cx, &project_info);

        // Populate step indicator demo
        self.ui
            .mp_step_indicator(cx, ids!(demo_step_indicator))
            .set_items(
                cx,
                vec![
                    "Cart".to_string(),
                    "Shipping".to_string(),
                    "Payment".to_string(),
                    "Review".to_string(),
                ],
            );

        // Rating demo: initial value
        self.ui.mp_rating(cx, ids!(demo_rating)).set_value(cx, 3);

        self.ui.mp_step_row(cx, ids!(demo_step_row_1)).set_step(
            cx,
            "○",
            "Install dependencies",
            "cargo fetch · crates.io",
            "3.2s",
        );
        self.ui.mp_step_row(cx, ids!(demo_step_row_2)).set_step(
            cx,
            "✕",
            "Run tests",
            "makepad-component::a2ui",
            "1 failed",
        );

        // Toggle group demo: items + initial selection
        self.ui
            .mp_toggle_group(cx, ids!(demo_toggle_group))
            .set_items(
                cx,
                &[
                    "Day".to_string(),
                    "Week".to_string(),
                    "Month".to_string(),
                    "Year".to_string(),
                ],
            );
        self.ui
            .mp_toggle_group(cx, ids!(demo_toggle_group))
            .set_selected(cx, 1);

        // Option card demo: preselect first plan
        self.ui
            .mp_option_card(cx, ids!(option_card_1))
            .set_selected(cx, true);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle category tab clicks
        if self.ui.mp_tab(cx, ids!(cat_form)).clicked(actions) {
            self.select_category(cx, 0);
        }
        if self.ui.mp_tab(cx, ids!(cat_display)).clicked(actions) {
            self.select_category(cx, 1);
        }
        if self.ui.mp_tab(cx, ids!(cat_nav)).clicked(actions) {
            self.select_category(cx, 2);
        }
        if self.ui.mp_tab(cx, ids!(cat_feedback)).clicked(actions) {
            self.select_category(cx, 3);
        }
        if self.ui.mp_tab(cx, ids!(cat_data)).clicked(actions) {
            self.select_category(cx, 4);
        }
        if self.ui.mp_tab(cx, ids!(cat_shader)).clicked(actions) {
            self.select_category(cx, 5);
        }
        if self.ui.mp_tab(cx, ids!(cat_shader_art)).clicked(actions) {
            self.select_category(cx, 6);
        }
        if self.ui.mp_tab(cx, ids!(cat_shader_art2)).clicked(actions) {
            self.select_category(cx, 7);
        }
        if self.ui.mp_tab(cx, ids!(cat_shader_math)).clicked(actions) {
            self.select_category(cx, 8);
        }
        if self.ui.mp_tab(cx, ids!(cat_splash)).clicked(actions) {
            self.select_category(cx, 9);
        }
        if self.ui.mp_tab(cx, ids!(cat_json)).clicked(actions) {
            self.select_category(cx, 10);
        }
        if self.ui.mp_tab(cx, ids!(cat_shadcn)).clicked(actions) {
            self.select_category(cx, 11);
        }

        // Handle counter button
        if self.ui.mp_button(cx, ids!(counter_btn)).clicked(actions) {
            self.counter += 1;
            self.ui
                .label(cx, ids!(counter_label))
                .set_text(cx, &format!("Clicked: {} times", self.counter));
        }

        // Handle PageFlip navigation
        if self.ui.mp_button(cx, ids!(page_btn_a)).clicked(actions) {
            self.ui
                .page_flip(cx, ids!(demo_page_flip))
                .set_active_page(cx, id!(page_a));
            self.current_page = 0;
            self.update_page_buttons(cx);
        }
        if self.ui.mp_button(cx, ids!(page_btn_b)).clicked(actions) {
            self.ui
                .page_flip(cx, ids!(demo_page_flip))
                .set_active_page(cx, id!(page_b));
            self.current_page = 1;
            self.update_page_buttons(cx);
        }
        if self.ui.mp_button(cx, ids!(page_btn_c)).clicked(actions) {
            self.ui
                .page_flip(cx, ids!(demo_page_flip))
                .set_active_page(cx, id!(page_c));
            self.current_page = 2;
            self.update_page_buttons(cx);
        }

        // Handle checkbox changes
        if self
            .ui
            .mp_checkbox(cx, ids!(checkbox1))
            .changed(actions)
            .is_some()
        {
            self.update_checkbox_status(cx);
        }
        if self
            .ui
            .mp_checkbox(cx, ids!(checkbox2))
            .changed(actions)
            .is_some()
        {
            self.update_checkbox_status(cx);
        }
        if self
            .ui
            .mp_checkbox(cx, ids!(checkbox3))
            .changed(actions)
            .is_some()
        {
            self.update_checkbox_status(cx);
        }

        // Handle switch changes
        if let Some(on) = self.ui.mp_switch(cx, ids!(switch_wifi)).changed(actions) {
            log!("Wi-Fi: {}", if on { "ON" } else { "OFF" });
        }
        if let Some(on) = self
            .ui
            .mp_switch(cx, ids!(switch_bluetooth))
            .changed(actions)
        {
            log!("Bluetooth: {}", if on { "ON" } else { "OFF" });
        }
        if let Some(on) = self
            .ui
            .mp_switch(cx, ids!(switch_notifications))
            .changed(actions)
        {
            log!("Notifications: {}", if on { "ON" } else { "OFF" });
        }

        // Handle radio changes (mutually exclusive)
        if self
            .ui
            .mp_radio(cx, ids!(radio_small))
            .changed(actions)
            .is_some()
        {
            self.ui
                .mp_radio(cx, ids!(radio_medium))
                .set_checked(cx, false);
            self.ui
                .mp_radio(cx, ids!(radio_large))
                .set_checked(cx, false);
            self.ui
                .label(cx, ids!(radio_status))
                .set_text(cx, "Selected: Small");
        }
        if self
            .ui
            .mp_radio(cx, ids!(radio_medium))
            .changed(actions)
            .is_some()
        {
            self.ui
                .mp_radio(cx, ids!(radio_small))
                .set_checked(cx, false);
            self.ui
                .mp_radio(cx, ids!(radio_large))
                .set_checked(cx, false);
            self.ui
                .label(cx, ids!(radio_status))
                .set_text(cx, "Selected: Medium");
        }
        if self
            .ui
            .mp_radio(cx, ids!(radio_large))
            .changed(actions)
            .is_some()
        {
            self.ui
                .mp_radio(cx, ids!(radio_small))
                .set_checked(cx, false);
            self.ui
                .mp_radio(cx, ids!(radio_medium))
                .set_checked(cx, false);
            self.ui
                .label(cx, ids!(radio_status))
                .set_text(cx, "Selected: Large");
        }

        // Handle progress buttons
        if self
            .ui
            .mp_button(cx, ids!(progress_inc_btn))
            .clicked(actions)
        {
            let current = self.ui.mp_progress(cx, ids!(interactive_progress)).value();
            let new_value = (current + 10.0).min(100.0);
            self.ui
                .mp_progress(cx, ids!(interactive_progress))
                .set_value(cx, new_value);
            self.ui
                .label(cx, ids!(progress_label))
                .set_text(cx, &format!("{}%", new_value as i32));
        }
        if self
            .ui
            .mp_button(cx, ids!(progress_dec_btn))
            .clicked(actions)
        {
            let current = self.ui.mp_progress(cx, ids!(interactive_progress)).value();
            let new_value = (current - 10.0).max(0.0);
            self.ui
                .mp_progress(cx, ids!(interactive_progress))
                .set_value(cx, new_value);
            self.ui
                .label(cx, ids!(progress_label))
                .set_text(cx, &format!("{}%", new_value as i32));
        }

        // Handle slider changes
        if let Some(value) = self.ui.mp_slider(cx, ids!(slider_default)).changed(actions) {
            let v = value.end();
            self.ui
                .label(cx, ids!(slider_default_label))
                .set_text(cx, &format!("Value: {}", v as i32));
        }

        if let Some(value) = self.ui.mp_slider(cx, ids!(slider_vert)).changed(actions) {
            let v = value.end();
            self.ui
                .label(cx, ids!(slider_vert_label))
                .set_text(cx, &format!("Vertical value: {}", v as i32));
        }

        // Handle range slider changes
        if let Some(value) = self.ui.mp_slider(cx, ids!(slider_range)).changed(actions) {
            let start = value.start() as i32;
            let end = value.end() as i32;
            self.ui
                .label(cx, ids!(slider_range_label))
                .set_text(cx, &format!("Range: {} - {}", start, end));
        }

        if let Some(value) = self
            .ui
            .mp_slider(cx, ids!(slider_range_success))
            .changed(actions)
        {
            let start = value.start() as i32;
            let end = value.end() as i32;
            self.ui
                .label(cx, ids!(slider_range_success_label))
                .set_text(cx, &format!("Range: {} - {} (step 5)", start, end));
        }

        // Handle shader art speed slider
        if let Some(value) = self
            .ui
            .mp_slider(cx, ids!(shader_art_speed))
            .changed(actions)
        {
            let speed = value.end();
            self.ui
                .label(cx, ids!(shader_art_speed_label))
                .set_text(cx, &format!("{:.1}x", speed));
            let mut shader_view = self.ui.view(cx, ids!(shader_art_canvas));
            script_apply_eval!(cx, shader_view, {
                speed: #(speed as f32)
            });
        }

        // Handle shader art2 speed slider
        if let Some(value) = self
            .ui
            .mp_slider(cx, ids!(shader_art2_speed))
            .changed(actions)
        {
            let speed = value.end();
            self.ui
                .label(cx, ids!(shader_art2_speed_label))
                .set_text(cx, &format!("{:.1}x", speed));
            let mut shader_view = self.ui.view(cx, ids!(shader_art2_canvas));
            script_apply_eval!(cx, shader_view, {
                speed: #(speed as f32)
            });
        }

        // Handle shader math speed slider
        if let Some(value) = self
            .ui
            .mp_slider(cx, ids!(shader_math_speed))
            .changed(actions)
        {
            let speed = value.end();
            self.ui
                .label(cx, ids!(shader_math_speed_label))
                .set_text(cx, &format!("{:.1}x", speed));
            let mut shader_view = self.ui.view(cx, ids!(shader_math_canvas));
            script_apply_eval!(cx, shader_view, {
                speed: #(speed as f32)
            });
        }

        // Handle input changes
        if let Some(text) = self
            .ui
            .text_input(cx, ids!(input_interactive))
            .changed(actions)
        {
            let display = if text.is_empty() {
                "Value: (empty)".to_string()
            } else {
                format!("Value: {}", text)
            };
            self.ui.label(cx, ids!(input_status)).set_text(cx, &display);
        }

        // Handle badge buttons
        if self.ui.mp_button(cx, ids!(badge_inc_btn)).clicked(actions) {
            let current = self.ui.mp_badge(cx, ids!(interactive_badge)).count();
            let new_count = current + 1;
            self.ui
                .mp_badge(cx, ids!(interactive_badge))
                .set_count(cx, new_count);
            self.ui
                .label(cx, ids!(badge_count_label))
                .set_text(cx, &format!("Count: {}", new_count));
        }
        if self.ui.mp_button(cx, ids!(badge_dec_btn)).clicked(actions) {
            let current = self.ui.mp_badge(cx, ids!(interactive_badge)).count();
            let new_count = (current - 1).max(0);
            self.ui
                .mp_badge(cx, ids!(interactive_badge))
                .set_count(cx, new_count);
            self.ui
                .label(cx, ids!(badge_count_label))
                .set_text(cx, &format!("Count: {}", new_count));
        }

        // Handle avatar change button
        if self
            .ui
            .mp_button(cx, ids!(avatar_change_btn))
            .clicked(actions)
        {
            let names = [
                "Alice Wang",
                "Bob Smith",
                "Carol Lee",
                "David Kim",
                "Emma Chen",
                "Frank Zhang",
            ];
            let idx = (cx.event_id() as usize) % names.len();
            let name = names[idx];
            self.ui
                .mp_avatar(cx, ids!(dynamic_avatar))
                .set_initials_from_name(cx, name);
            self.ui
                .label(cx, ids!(avatar_name_label))
                .set_text(cx, name);
        }

        // Handle clickable card clicks using as_widget_action().cast() pattern
        for action in actions {
            if let MpCardAction::Clicked = action.as_widget_action().cast() {
                self.ui
                    .label(cx, ids!(card_click_status))
                    .set_text(cx, "Card clicked!");
            }
            // Handle modal close request (backdrop or X button)
            if let MpModalAction::CloseRequested = action.as_widget_action().cast() {
                self.ui.mp_modal_widget(cx, ids!(demo_modal)).close(cx);
                self.ui
                    .label(cx, ids!(modal_status))
                    .set_text(cx, "Modal closed");
            }
        }

        // Handle open modal button
        if self.ui.mp_button(cx, ids!(open_modal_btn)).clicked(actions) {
            self.ui.mp_modal_widget(cx, ids!(demo_modal)).open(cx);
            self.ui
                .label(cx, ids!(modal_status))
                .set_text(cx, "Modal opened");
        }

        // Handle modal cancel button
        if self
            .ui
            .mp_button(cx, ids!(modal_cancel_btn))
            .clicked(actions)
        {
            self.ui.mp_modal_widget(cx, ids!(demo_modal)).close(cx);
            self.ui
                .label(cx, ids!(modal_status))
                .set_text(cx, "Cancelled");
        }

        // Handle modal confirm button
        if self
            .ui
            .mp_button(cx, ids!(modal_confirm_btn))
            .clicked(actions)
        {
            self.ui.mp_modal_widget(cx, ids!(demo_modal)).close(cx);
            self.ui
                .label(cx, ids!(modal_status))
                .set_text(cx, "Confirmed!");
        }

        // Handle popover toggle button
        if self
            .ui
            .mp_button(cx, ids!(popover_trigger_btn))
            .clicked(actions)
        {
            self.ui
                .mp_popover_widget(cx, ids!(interactive_popover))
                .toggle(cx);
        }

        // Handle skeleton toggle button
        if self
            .ui
            .mp_button(cx, ids!(skeleton_toggle_btn))
            .clicked(actions)
        {
            let skeleton = self.ui.mp_skeleton_widget(cx, ids!(interactive_skeleton));
            let is_loading = skeleton.is_loading();
            skeleton.set_loading(cx, !is_loading);
            let status = if !is_loading { "Loading" } else { "Loaded" };
            self.ui
                .label(cx, ids!(skeleton_status))
                .set_text(cx, &format!("Status: {}", status));
        }

        // Handle notification buttons
        if self
            .ui
            .mp_button(cx, ids!(show_success_notif))
            .clicked(actions)
        {
            self.ui
                .mp_notification_widget(cx, ids!(demo_notification))
                .show_message(cx, "Success!", "Operation completed successfully!");
        }
        if self
            .ui
            .mp_button(cx, ids!(show_error_notif))
            .clicked(actions)
        {
            self.ui
                .mp_notification_widget(cx, ids!(demo_notification))
                .show_message(cx, "Error", "Something went wrong. Please try again.");
        }
        if self
            .ui
            .mp_button(cx, ids!(show_warning_notif))
            .clicked(actions)
        {
            self.ui
                .mp_notification_widget(cx, ids!(demo_notification))
                .show_message(cx, "Warning", "Please review your input before continuing.");
        }
        if self
            .ui
            .mp_button(cx, ids!(show_info_notif))
            .clicked(actions)
        {
            self.ui
                .mp_notification_widget(cx, ids!(demo_notification))
                .show_message(cx, "Info", "Here's some helpful information for you.");
        }

        // Handle dropdown changes
        let labels = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
        if let Some(idx) = self
            .ui
            .drop_down(cx, ids!(dropdown_basic))
            .selected(actions)
        {
            let label = labels.get(idx).unwrap_or(&"Unknown");
            self.ui
                .label(cx, ids!(dropdown_status))
                .set_text(cx, &format!("Selected: {}", label));
        }

        // Handle Tab clicks - Default style
        if self.ui.mp_tab(cx, ids!(tab_home)).clicked(actions) {
            self.select_tab(cx, "default", 0, "Home");
        }
        if self.ui.mp_tab(cx, ids!(tab_profile)).clicked(actions) {
            self.select_tab(cx, "default", 1, "Profile");
        }
        if self.ui.mp_tab(cx, ids!(tab_settings)).clicked(actions) {
            self.select_tab(cx, "default", 2, "Settings");
        }

        // Handle Tab clicks - Underline style
        if self.ui.mp_tab(cx, ids!(tab_u_overview)).clicked(actions) {
            self.select_tab(cx, "underline", 0, "Overview");
        }
        if self.ui.mp_tab(cx, ids!(tab_u_analytics)).clicked(actions) {
            self.select_tab(cx, "underline", 1, "Analytics");
        }
        if self.ui.mp_tab(cx, ids!(tab_u_reports)).clicked(actions) {
            self.select_tab(cx, "underline", 2, "Reports");
        }

        // Handle Tab clicks - Pill style
        if self.ui.mp_tab(cx, ids!(tab_p_all)).clicked(actions) {
            self.select_tab(cx, "pill", 0, "All");
        }
        if self.ui.mp_tab(cx, ids!(tab_p_active)).clicked(actions) {
            self.select_tab(cx, "pill", 1, "Active");
        }
        if self.ui.mp_tab(cx, ids!(tab_p_completed)).clicked(actions) {
            self.select_tab(cx, "pill", 2, "Completed");
        }

        // Handle Tab clicks - Outline style
        if self.ui.mp_tab(cx, ids!(tab_o_day)).clicked(actions) {
            self.select_tab(cx, "outline", 0, "Day");
        }
        if self.ui.mp_tab(cx, ids!(tab_o_week)).clicked(actions) {
            self.select_tab(cx, "outline", 1, "Week");
        }
        if self.ui.mp_tab(cx, ids!(tab_o_month)).clicked(actions) {
            self.select_tab(cx, "outline", 2, "Month");
        }

        // Handle Tab clicks - Segmented style
        if self.ui.mp_tab(cx, ids!(tab_s_list)).clicked(actions) {
            self.select_tab(cx, "segmented", 0, "List");
        }
        if self.ui.mp_tab(cx, ids!(tab_s_grid)).clicked(actions) {
            self.select_tab(cx, "segmented", 1, "Grid");
        }
        if self.ui.mp_tab(cx, ids!(tab_s_map)).clicked(actions) {
            self.select_tab(cx, "segmented", 2, "Map");
        }

        // Handle collapsible toggles
        if self
            .ui
            .mp_collapsible_trigger(cx, ids!(collapsible_trigger))
            .toggled(actions)
        {
            let current = self.ui.view(cx, ids!(collapsible_content)).visible();
            self.ui
                .view(cx, ids!(collapsible_content))
                .set_visible(cx, !current);
            self.ui.redraw(cx);
        }
        if self
            .ui
            .mp_collapsible_trigger(cx, ids!(collapsible_trigger2))
            .toggled(actions)
        {
            let current = self.ui.view(cx, ids!(collapsible_content2)).visible();
            self.ui
                .view(cx, ids!(collapsible_content2))
                .set_visible(cx, !current);
            self.ui.redraw(cx);
        }

        // Handle sheet trigger
        if self
            .ui
            .mp_sheet_trigger(cx, ids!(demo_sheet_trigger))
            .opened(actions)
        {
            self.ui.mp_sheet(cx, ids!(demo_sheet)).set_open(cx, true);
            self.ui
                .mp_sheet(cx, ids!(demo_sheet))
                .set_title(cx, "Sheet Demo");
        }
        self.ui.mp_sheet(cx, ids!(demo_sheet)).closed(actions);

        // Handle dialog open/close
        if self
            .ui
            .mp_button(cx, ids!(demo_dialog_trigger))
            .clicked(actions)
        {
            if let Some(mut w) = self
                .ui
                .widget(cx, ids!(demo_dialog))
                .borrow_mut::<makepad_component::widgets::MpDialog>()
            {
                w.open(cx);
            }
        }
        if self
            .ui
            .mp_button(cx, ids!(dialog_close_btn))
            .clicked(actions)
        {
            if let Some(mut w) = self
                .ui
                .widget(cx, ids!(demo_dialog))
                .borrow_mut::<makepad_component::widgets::MpDialog>()
            {
                w.close(cx);
            }
        }
        if self
            .ui
            .mp_button(cx, ids!(dialog_confirm_btn))
            .clicked(actions)
        {
            if let Some(mut w) = self
                .ui
                .widget(cx, ids!(demo_dialog))
                .borrow_mut::<makepad_component::widgets::MpDialog>()
            {
                w.close(cx);
            }
        }

        // Handle theme toggle button
        if self
            .ui
            .mp_button(cx, ids!(theme_toggle_btn))
            .clicked(actions)
        {
            let is_dark = self
                .ui
                .widget(cx, ids!(theme_state))
                .borrow_mut::<MpThemeState>()
                .map(|ts| ts.is_dark())
                .unwrap_or(false);
            if let Some(mut ts) = self
                .ui
                .widget(cx, ids!(theme_state))
                .borrow_mut::<MpThemeState>()
            {
                ts.toggle(cx)
            }
            let new_mode = if is_dark { "Light" } else { "Dark" };
            self.ui.label(cx, ids!(theme_status)).set_text(cx, new_mode);
        }

        // Table demo: row selection + sorting feedback
        if let Some(row) = self.ui.mp_table(cx, ids!(demo_table)).row_selected(actions) {
            self.ui
                .label(cx, ids!(table_status))
                .set_text(cx, &format!("Selected row: {}", row));
        }
        let table = self.ui.mp_table(cx, ids!(demo_table));
        if let Some((col, dir)) = table.sorted_column(actions) {
            let dir = match dir {
                makepad_component::widgets::SortDirection::Ascending => "asc",
                makepad_component::widgets::SortDirection::Descending => "desc",
                _ => "none",
            };
            self.ui
                .label(cx, ids!(table_status))
                .set_text(cx, &format!("Sort: column {} ({})", col, dir));
        }

        // Tree demo: item selection
        if let Some(item) = self.ui.mp_tree(cx, ids!(demo_tree)).item_selected(actions) {
            self.ui
                .label(cx, ids!(tree_status))
                .set_text(cx, &format!("Selected item: {}", item));
        }

        // Combobox demo: selection
        if let Some(selected) = self
            .ui
            .mp_combobox(cx, ids!(demo_combobox))
            .selected(actions)
        {
            self.ui
                .label(cx, ids!(combobox_status))
                .set_text(cx, &format!("Selected: {}", selected));
        }

        // Stepper demo: value change
        if let Some(value) = self.ui.mp_stepper(cx, ids!(demo_stepper)).changed(actions) {
            self.ui
                .label(cx, ids!(stepper_status))
                .set_text(cx, &format!("Value: {}", value));
        }

        // Step indicator demo: step selection
        if let Some(step) = self
            .ui
            .mp_step_indicator(cx, ids!(demo_step_indicator))
            .selected(actions)
        {
            self.ui
                .label(cx, ids!(step_status))
                .set_text(cx, &format!("Step: {}", step + 1));
        }

        // Attachment demo: removal
        for attachment_id in [ids!(demo_attachment), ids!(demo_attachment_img)] {
            if let Some(removed) = self.ui.mp_attachment(cx, attachment_id).removed(actions) {
                self.ui
                    .label(cx, ids!(attachment_status))
                    .set_text(cx, &format!("Removed: {}", removed));
            }
        }

        // Context menu demo: item selection
        if let Some(selected) = self
            .ui
            .mp_context_menu(cx, ids!(demo_context_menu))
            .item_selected(actions)
        {
            self.ui
                .label(cx, ids!(context_menu_status))
                .set_text(cx, &format!("Selected: {}", selected));
        }

        // Dropdown menu demo: item selection
        if let Some(selected) = self
            .ui
            .mp_dropdown_menu(cx, ids!(demo_dropdown_menu))
            .item_selected(actions)
        {
            self.ui
                .label(cx, ids!(dropdown_menu_status))
                .set_text(cx, &format!("Selected: {}", selected));
        }

        // Menu bar demo: item selection
        if let Some((menu, item)) = self.ui.mp_menu_bar(cx, ids!(demo_menu_bar)).selected(actions) {
            self.ui
                .label(cx, ids!(menu_bar_status))
                .set_text(cx, &format!("Menu {}: {}", menu, item));
        }

        // Split pane demo: divider resize
        if let Some(width) = self.ui.mp_split_pane(cx, ids!(demo_split)).resized(actions) {
            self.ui
                .label(cx, ids!(split_status))
                .set_text(cx, &format!("Left width: {:.0}", width));
        }

        // Toggle group demo: selection change
        if let Some(index) = self.ui.mp_toggle_group(cx, ids!(demo_toggle_group)).selected(actions) {
            self.ui
                .label(cx, ids!(toggle_group_status))
                .set_text(cx, &format!("Selected: {}", index));
        }

        // Option card demo: exclusive selection
        for (i, name) in ["option_card_1", "option_card_2", "option_card_3"]
            .iter()
            .enumerate()
        {
            let key = makepad_widgets::LiveId::from_str(name);
            if self.ui.mp_option_card(cx, &[key]).clicked(actions) {
                for (j, name2) in ["option_card_1", "option_card_2", "option_card_3"]
                    .iter()
                    .enumerate()
                {
                    let key2 = makepad_widgets::LiveId::from_str(name2);
                    self.ui
                        .mp_option_card(cx, &[key2])
                        .set_selected(cx, i == j);
                }
                self.ui
                    .label(cx, ids!(scaffolding_status))
                    .set_text(cx, &format!("Selected: {}", name.replace("option_card_", "Plan ")));
            }
        }

        // Rating demo: star change
        if let Some(value) = self.ui.mp_rating(cx, ids!(demo_rating)).changed(actions) {
            self.ui
                .label(cx, ids!(rating_status))
                .set_text(cx, &format!("Rating: {}", value));
        }

        // Color picker demo: swatch picked
        if let Some(color) = self.ui.mp_color_picker(cx, ids!(demo_color_picker)).picked(actions) {
            self.ui
                .label(cx, ids!(color_status))
                .set_text(
                    cx,
                    &format!(
                        "Picked: rgb({:.0}, {:.0}, {:.0})",
                        color.x * 255.0,
                        color.y * 255.0,
                        color.z * 255.0
                    ),
                );
        }

        // Chips demo: remove
        for name in ["demo_chip1", "demo_chip2", "demo_chip3"] {
            let key = makepad_widgets::LiveId::from_str(name);
            if let Some(removed) = self.ui.mp_chip(cx, &[key]).removed(actions) {
                self.ui
                    .label(cx, ids!(chips_status))
                    .set_text(cx, &format!("Removed: {}", removed));
            }
        }
    }
}

impl App {
    fn select_category(&mut self, cx: &mut Cx, index: usize) {
        self.current_category = index;

        // Update tab selected states
        self.ui
            .mp_tab(cx, ids!(cat_form))
            .set_selected(cx, index == 0);
        self.ui
            .mp_tab(cx, ids!(cat_display))
            .set_selected(cx, index == 1);
        self.ui
            .mp_tab(cx, ids!(cat_nav))
            .set_selected(cx, index == 2);
        self.ui
            .mp_tab(cx, ids!(cat_feedback))
            .set_selected(cx, index == 3);
        self.ui
            .mp_tab(cx, ids!(cat_data))
            .set_selected(cx, index == 4);
        self.ui
            .mp_tab(cx, ids!(cat_shader))
            .set_selected(cx, index == 5);
        self.ui
            .mp_tab(cx, ids!(cat_shader_art))
            .set_selected(cx, index == 6);
        self.ui
            .mp_tab(cx, ids!(cat_shader_art2))
            .set_selected(cx, index == 7);
        self.ui
            .mp_tab(cx, ids!(cat_shader_math))
            .set_selected(cx, index == 8);
        self.ui
            .mp_tab(cx, ids!(cat_splash))
            .set_selected(cx, index == 9);
        self.ui
            .mp_tab(cx, ids!(cat_json))
            .set_selected(cx, index == 10);
        self.ui
            .mp_tab(cx, ids!(cat_shadcn))
            .set_selected(cx, index == 11);

        // Switch page
        let page_id = match index {
            0 => id!(page_form),
            1 => id!(page_display),
            2 => id!(page_nav),
            3 => id!(page_feedback),
            4 => id!(page_data),
            5 => id!(page_shader),
            6 => id!(page_shader_art),
            7 => id!(page_shader_art2),
            8 => id!(page_shader_math),
            9 => id!(page_splash),
            10 => id!(page_json),
            11 => id!(page_shadcn),
            _ => id!(page_form),
        };
        self.ui
            .page_flip(cx, ids!(category_pages))
            .set_active_page(cx, page_id);
        self.ui.redraw(cx);
    }

    fn update_checkbox_status(&mut self, cx: &mut Cx) {
        let mut selected = Vec::new();

        if self.ui.mp_checkbox(cx, ids!(checkbox1)).is_checked() {
            selected.push("Option 1");
        }
        if self.ui.mp_checkbox(cx, ids!(checkbox2)).is_checked() {
            selected.push("Option 2");
        }
        if self.ui.mp_checkbox(cx, ids!(checkbox3)).is_checked() {
            selected.push("Option 3");
        }

        let status = if selected.is_empty() {
            "Selected: None".to_string()
        } else {
            format!("Selected: {}", selected.join(", "))
        };

        self.ui
            .label(cx, ids!(checkbox_status))
            .set_text(cx, &status);
    }

    fn update_page_buttons(&mut self, cx: &mut Cx) {
        let active_bg = vec4(0.231, 0.510, 0.965, 1.0);
        let active_hover = vec4(0.145, 0.388, 0.859, 1.0);
        let active_pressed = vec4(0.114, 0.310, 0.847, 1.0);
        let active_text = vec4(1.0, 1.0, 1.0, 1.0);

        let inactive_bg = vec4(0.0, 0.0, 0.0, 0.0);
        let inactive_hover = vec4(0.945, 0.961, 0.976, 1.0);
        let inactive_pressed = vec4(0.796, 0.835, 0.820, 1.0);
        let inactive_text = vec4(0.059, 0.090, 0.165, 1.0);

        let buttons = [
            (ids!(page_btn_a), 0),
            (ids!(page_btn_b), 1),
            (ids!(page_btn_c), 2),
        ];

        for (btn_id, page_idx) in buttons {
            let btn = self.ui.widget(cx, btn_id);
            if page_idx == self.current_page {
                let mut btn = btn;
                script_apply_eval!(cx, btn, {
                    draw_bg +: { color: #(active_bg), color_hover: #(active_hover), color_pressed: #(active_pressed) }
                    draw_text +: { color: #(active_text) }
                });
            } else {
                let mut btn = btn;
                script_apply_eval!(cx, btn, {
                    draw_bg +: { color: #(inactive_bg), color_hover: #(inactive_hover), color_pressed: #(inactive_pressed) }
                    draw_text +: { color: #(inactive_text) }
                });
            }
        }

        self.ui.redraw(cx);
    }

    fn select_tab(&mut self, cx: &mut Cx, style: &str, index: usize, label: &str) {
        match style {
            "default" => {
                self.ui
                    .mp_tab(cx, ids!(tab_home))
                    .set_selected(cx, index == 0);
                self.ui
                    .mp_tab(cx, ids!(tab_profile))
                    .set_selected(cx, index == 1);
                self.ui
                    .mp_tab(cx, ids!(tab_settings))
                    .set_selected(cx, index == 2);
            }
            "underline" => {
                self.ui
                    .mp_tab(cx, ids!(tab_u_overview))
                    .set_selected(cx, index == 0);
                self.ui
                    .mp_tab(cx, ids!(tab_u_analytics))
                    .set_selected(cx, index == 1);
                self.ui
                    .mp_tab(cx, ids!(tab_u_reports))
                    .set_selected(cx, index == 2);
            }
            "pill" => {
                self.ui
                    .mp_tab(cx, ids!(tab_p_all))
                    .set_selected(cx, index == 0);
                self.ui
                    .mp_tab(cx, ids!(tab_p_active))
                    .set_selected(cx, index == 1);
                self.ui
                    .mp_tab(cx, ids!(tab_p_completed))
                    .set_selected(cx, index == 2);
            }
            "outline" => {
                self.ui
                    .mp_tab(cx, ids!(tab_o_day))
                    .set_selected(cx, index == 0);
                self.ui
                    .mp_tab(cx, ids!(tab_o_week))
                    .set_selected(cx, index == 1);
                self.ui
                    .mp_tab(cx, ids!(tab_o_month))
                    .set_selected(cx, index == 2);
            }
            "segmented" => {
                self.ui
                    .mp_tab(cx, ids!(tab_s_list))
                    .set_selected(cx, index == 0);
                self.ui
                    .mp_tab(cx, ids!(tab_s_grid))
                    .set_selected(cx, index == 1);
                self.ui
                    .mp_tab(cx, ids!(tab_s_map))
                    .set_selected(cx, index == 2);
            }
            _ => {}
        }

        self.ui
            .label(cx, ids!(tab_status))
            .set_text(cx, &format!("Selected: {}", label));
        self.ui.redraw(cx);
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        makepad_component::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        // Tab / Shift-Tab keyboard focus traversal (bezel focus port)
        makepad_component::widgets::focus::handle_key(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
