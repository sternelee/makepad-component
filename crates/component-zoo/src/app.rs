use makepad_widgets::*;
use makepad_component::widgets::MpButtonWidgetRefExt;
use makepad_component::widgets::MpButtonWidgetExt;
use makepad_component::widgets::MpCheckboxWidgetRefExt;
use makepad_component::widgets::MpSwitchWidgetRefExt;
use makepad_component::widgets::MpRadioWidgetRefExt;
use makepad_component::widgets::MpProgressWidgetRefExt;
use makepad_component::widgets::MpSliderWidgetRefExt;
use makepad_component::widgets::MpBadgeWidgetRefExt;
use makepad_component::widgets::MpTabWidgetRefExt;
use makepad_component::widgets::MpCardAction;
use makepad_component::widgets::MpAvatarWidgetRefExt;
use makepad_component::widgets::MpModalAction;
use makepad_component::widgets::MpModalWidgetWidgetRefExt;
use makepad_component::widgets::MpNotificationWidgetWidgetRefExt;
use makepad_component::widgets::MpSkeletonWidgetWidgetRefExt;
use makepad_component::widgets::MpPopoverWidgetWidgetRefExt;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use makepad_component::theme::colors::*;
    use makepad_component::widgets::button::*;
    use makepad_component::widgets::checkbox::*;
    use makepad_component::widgets::switch::*;
    use makepad_component::widgets::divider::*;
    use makepad_component::widgets::radio::*;
    use makepad_component::widgets::progress::*;
    use makepad_component::widgets::slider::*;
    use makepad_component::widgets::input::*;
    use makepad_component::widgets::badge::*;
    use makepad_component::widgets::tooltip::*;
    use makepad_component::widgets::dropdown::*;
    use makepad_component::widgets::page_flip::*;
    use makepad_component::widgets::tab::*;
    use makepad_component::widgets::card::*;
    use makepad_component::widgets::avatar::*;
    use makepad_component::widgets::skeleton::*;
    use makepad_component::widgets::spinner::*;
    use makepad_component::widgets::accordion::*;
    use makepad_component::widgets::list::*;
    use makepad_component::widgets::notification::*;
    use makepad_component::widgets::modal::*;
    use makepad_component::widgets::popover::*;
    use makepad_component::widgets::label::*;
    use makepad_component::widgets::text::*;
    use makepad_component::widgets::alert::*;

    // ============================================================
    // Section Header Component
    // ============================================================
    SectionHeader = <Label> {
        width: Fit, height: Fit,
        draw_text: {
            text_style: <THEME_FONT_BOLD>{ font_size: 18.0 }
            color: (FOREGROUND)
        }
    }

    SubsectionLabel = <Label> {
        width: Fit, height: Fit,
        draw_text: {
            text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
            color: (MUTED_FOREGROUND)
        }
    }

    // ============================================================
    // Category Tab Style
    // ============================================================
    CategoryTab = <MpTabPill> {
        padding: { left: 16, right: 16, top: 8, bottom: 8 }
    }

    // ============================================================
    // Animated Shader Canvas - Shadertoy-style fractal
    // ============================================================
    ShaderCanvas = {{ShaderCanvas}} {
        show_bg: true
        draw_bg: {
            // Time uniform driven by animator
            instance anim_time: 0.0

            // Shadertoy-style fractal shader
            fn pixel(self) -> vec4 {
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

        animator: {
            anim = {
                default: on,
                on = {
                    from: {all: Loop {duration: 10.0, end: 1.0}}
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
    ShaderArtCanvas = {{ShaderArtCanvas}} {
        show_bg: true
        speed: 1.0

        draw_bg: {
            instance anim_time: 0.0
            instance speed: 1.0

            // Observer shader - glowing lattice effect
            // Original: vec2 p=(FC.xy*2.-r)/r.y/.2,v;
            //   for(float i,l,f;i++<1e1;
            //     o+=.03/max(l=length(v)-i,-l*3.)*(cos(t-i*.4+.1/l+vec4(0,1,2,3))+1.1))
            //     for(v=p,f=0.;f++<9.;v+=sin(ceil(v*f+i*.9)-t/2.)/f);
            //   o=max(tanh(o+(o=texture(b,...))*o),.0);

            fn pixel(self) -> vec4 {
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

        animator: {
            anim = {
                default: on,
                on = {
                    from: {all: Loop {duration: 15.0, end: 1.0}}
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
    ShaderArt2Canvas = {{ShaderArt2Canvas}} {
        show_bg: true
        speed: 1.0

        draw_bg: {
            instance anim_time: 0.0
            instance speed: 1.0

            // Golden FBM noise + SCRY bitmap text
            // Faithful translation from Shadertoy common code
            fn pixel(self) -> vec4 {
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
                let b1 = mod(floor(29671.0 / exp2(ceil(cc1 * 3.0 - 0.6))), 2.0);
                let m1 = step(0.0, bv1.x) * step(0.0, bv1.y)
                       * step(0.0, -bv1.x + 0.99) * step(0.0, -bv1.y + 1.6);

                // Char C (29263) — suv.x -= 4/3
                let sx2 = suv.x - 1.333;
                let bv2 = floor(vec2(sx2 * 3.0, suv.y * 5.0)) / 3.0;
                let cc2 = bv2.x + bv2.y * 3.0;
                let b2 = mod(floor(29263.0 / exp2(ceil(cc2 * 3.0 - 0.6))), 2.0);
                let m2 = step(0.0, bv2.x) * step(0.0, bv2.y)
                       * step(0.0, -bv2.x + 0.99) * step(0.0, -bv2.y + 1.6);

                // Char R (31469) — suv.x -= 8/3
                let sx3 = suv.x - 2.666;
                let bv3 = floor(vec2(sx3 * 3.0, suv.y * 5.0)) / 3.0;
                let cc3 = bv3.x + bv3.y * 3.0;
                let b3 = mod(floor(31469.0 / exp2(ceil(cc3 * 3.0 - 0.6))), 2.0);
                let m3 = step(0.0, bv3.x) * step(0.0, bv3.y)
                       * step(0.0, -bv3.x + 0.99) * step(0.0, -bv3.y + 1.6);

                // Char Y (23186) — suv.x -= 4
                let sx4 = suv.x - 4.0;
                let bv4 = floor(vec2(sx4 * 3.0, suv.y * 5.0)) / 3.0;
                let cc4 = bv4.x + bv4.y * 3.0;
                let b4 = mod(floor(23186.0 / exp2(ceil(cc4 * 3.0 - 0.6))), 2.0);
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

        animator: {
            anim = {
                default: on,
                on = {
                    from: {all: Loop {duration: 20.0, end: 1.0}}
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
    ShaderMathCanvas = {{ShaderMathCanvas}} {
        show_bg: true
        speed: 1.0

        draw_bg: {
            instance anim_time: 0.0
            instance speed: 1.0

            // Jellyfish point-cloud shader (minimal GPU version)
            fn pixel(self) -> vec4 {
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
                    let ix = mod(fi, 10.0);
                    let iy = floor(fi / 10.0);

                    let x = ix * 17.0;
                    let y = iy * 15.5;

                    let k = 5.0 * cos(x / 14.0) * cos(y / 30.0);
                    let e = y / 8.0 - 13.0;
                    let d = (k * k + e * e) / 59.0 + 4.0;

                    let bell = 1.0 + 0.8 * exp(-(d - 4.0));

                    let q = 60.0 - 3.0 * sin(atan(k, e))
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
                                + atan(k, e) * 0.25 + d * 0.06;
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

        animator: {
            anim = {
                default: on,
                on = {
                    from: {all: Loop {duration: 20.0, end: 1.0}}
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
    SplashDemo = {{SplashDemo}} {
        width: Fill, height: Fill,
        flow: Down,
        spacing: 20,
        padding: { left: 24, right: 24, top: 24, bottom: 100 }

        show_bg: true
        draw_bg: { color: #1e1e2e }

        // Header
        <View> {
            width: Fill, height: Fit,
            flow: Down,
            spacing: 8,

            <SectionHeader> {
                draw_text: { color: #cdd6f4 }
                text: "Natural Language UI Generation"
            }

            <Label> {
                width: Fill, height: Fit,
                draw_text: {
                    text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                    color: #a6adc8
                    wrap: Word
                }
                text: "Type commands to dynamically generate UI widgets in real-time."
            }
        }

        <MpDivider> { draw_bg: { color: #313244 } }

        // Command Input Section
        <View> {
            width: Fill, height: Fit,
            flow: Down,
            spacing: 12,

            <Label> {
                draw_text: {
                    text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                    color: #89b4fa
                }
                text: "Command Input"
            }

            // Example commands
            <View> {
                width: Fill, height: Fit,
                padding: 12,
                show_bg: true
                draw_bg: { color: #313244 }

                <Label> {
                    width: Fill, height: Fit,
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                        color: #6c7086
                        wrap: Word
                    }
                    text: "Commands: \"add button Submit\" | \"add label Hello World\" | \"add card User Profile\" | \"add progress 75\" | \"add switch Dark Mode\" | \"clear\""
                }
            }

            <View> {
                width: Fill, height: Fit,
                flow: Right,
                spacing: 12,
                align: { y: 0.5 }

                command_input = <TextInput> {
                    width: Fill, height: Fit,
                    padding: 12,
                    empty_text: "Type a command... e.g. 'add button Click Me'"
                    draw_bg: {
                        color: #313244
                    }
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                        color: #cdd6f4
                    }
                }

                generate_btn = <MpButtonPrimary> { text: "Generate" }
                clear_btn = <MpButtonGhost> {
                    draw_text: { color: #f38ba8 }
                    text: "Clear All"
                }
            }
        }

        <MpDivider> { draw_bg: { color: #313244 } }

        // Generated UI Section
        <View> {
            width: Fill, height: Fit,
            flow: Right,
            align: { y: 0.5 }

            <Label> {
                draw_text: {
                    text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                    color: #89b4fa
                }
                text: "Generated UI"
            }

            <View> { width: Fill, height: 1 }

            widget_count_label = <Label> {
                draw_text: {
                    text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                    color: #a6e3a1
                }
                text: "0 widgets"
            }
        }

        // Dynamic PortalList for generated widgets
        generated_list = <PortalList> {
            width: Fill, height: 400,
            flow: Down,

            // Button template
            GenButton = <View> {
                width: Fill, height: Fit,
                padding: 8,
                margin: { bottom: 8 }

                gen_button = <MpButtonPrimary> {
                    width: Fit
                    text: "Button"
                }
            }

            // Label template
            GenLabel = <View> {
                width: Fill, height: Fit,
                padding: { left: 12, right: 12, top: 16, bottom: 16 }
                margin: { bottom: 8 }
                show_bg: true
                draw_bg: { color: #313244 }

                gen_label = <Label> {
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                        color: #cdd6f4
                    }
                    text: "Label"
                }
            }

            // Card template
            GenCard = <MpCard> {
                width: Fill, height: Fit,
                margin: { bottom: 8 }
                padding: 16,

                <View> {
                    width: Fill, height: Fit,
                    flow: Down,
                    spacing: 8,

                    card_title = <Label> {
                        draw_text: {
                            text_style: <THEME_FONT_BOLD>{ font_size: 16.0 }
                            color: #cdd6f4
                        }
                        text: "Card Title"
                    }

                    <Label> {
                        draw_text: {
                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                            color: #a6adc8
                        }
                        text: "This is a dynamically generated card widget."
                    }
                }
            }

            // Progress template
            GenProgress = <View> {
                width: Fill, height: Fit,
                padding: 12,
                margin: { bottom: 8 }
                show_bg: true
                draw_bg: { color: #313244 }
                flow: Down,
                spacing: 8,

                progress_label = <Label> {
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                        color: #a6adc8
                    }
                    text: "Progress: 50%"
                }

                gen_progress = <MpProgress> {
                    width: Fill, height: 8,
                    value: 50
                }
            }

            // Switch template
            GenSwitch = <View> {
                width: Fill, height: Fit,
                padding: 12,
                margin: { bottom: 8 }
                show_bg: true
                draw_bg: { color: #313244 }
                flow: Right,
                align: { y: 0.5 }
                spacing: 12,

                switch_label = <Label> {
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                        color: #cdd6f4
                    }
                    text: "Toggle"
                }

                <View> { width: Fill, height: 1 }

                gen_switch = <MpSwitch> {}
            }

            // Input template
            GenInput = <View> {
                width: Fill, height: Fit,
                padding: 8,
                margin: { bottom: 8 }
                flow: Down,
                spacing: 8,

                input_label = <Label> {
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                        color: #a6adc8
                    }
                    text: "Input Field"
                }

                gen_input = <TextInput> {
                    width: Fill, height: Fit,
                    padding: 10,
                    empty_text: "Enter text..."
                    draw_bg: { color: #45475a }
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                        color: #cdd6f4
                    }
                }
            }
        }
    }

    // ============================================================
    // JsonRenderDemo - JSON-based Dynamic UI Generation
    // ============================================================
    JsonRenderDemo = {{JsonRenderDemo}} {
        width: Fill, height: Fill,
        flow: Down,
        spacing: 20,
        padding: { left: 24, right: 24, top: 24, bottom: 100 }

        show_bg: true
        draw_bg: { color: #1e1e2e }

        // Header
        <View> {
            width: Fill, height: Fit,
            flow: Down,
            spacing: 8,

            <SectionHeader> {
                draw_text: { color: #cdd6f4 }
                text: "JSON Render - A2UI Protocol"
            }

            <Label> {
                width: Fill, height: Fit,
                draw_text: {
                    text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                    color: #a6adc8
                    wrap: Word
                }
                text: "Parse JSON schema to dynamically render Makepad UI components. Supports nested layouts and component properties."
            }
        }

        <MpDivider> { draw_bg: { color: #313244 } }

        // Main content area - two columns
        <View> {
            width: Fill, height: Fill,
            flow: Right,
            spacing: 20,

            // Left: JSON Editor
            <View> {
                width: Fill, height: Fill,
                flow: Down,
                spacing: 12,

                <Label> {
                    draw_text: {
                        text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                        color: #89b4fa
                    }
                    text: "JSON Schema"
                }

                json_input = <TextInput> {
                    width: Fill, height: Fill,
                    padding: 12,
                    empty_text: "Enter JSON UI schema..."
                    draw_bg: { color: #313244 }
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                        color: #cdd6f4
                    }
                }

                <View> {
                    width: Fill, height: Fit,
                    flow: Right,
                    spacing: 12,

                    render_btn = <MpButtonPrimary> { text: "Render" }
                    clear_render_btn = <MpButtonGhost> {
                        draw_text: { color: #f38ba8 }
                        text: "Clear"
                    }
                    <View> { width: Fill }
                    load_example_btn = <MpButtonSecondary> { text: "Basic Example" }
                    load_raycast_btn = <MpButtonSecondary> { text: "Raycast Examples" }
                }
            }

            // Right: Rendered Preview
            <View> {
                width: Fill, height: Fill,
                flow: Down,
                spacing: 12,

                <View> {
                    width: Fill, height: Fit,
                    flow: Right,
                    align: { y: 0.5 }

                    <Label> {
                        draw_text: {
                            text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                            color: #89b4fa
                        }
                        text: "Rendered Preview"
                    }

                    <View> { width: Fill }

                    render_status = <Label> {
                        draw_text: {
                            text_style: <THEME_FONT_BOLD>{ font_size: 12.0 }
                            color: #a6e3a1
                        }
                        text: "Ready"
                    }
                }

                // Preview container with border
                preview_container = <View> {
                    width: Fill, height: Fill,
                    padding: 16,
                    show_bg: true
                    draw_bg: { color: #11111b }

                    // Dynamic content rendered via PortalList
                    json_list = <PortalList> {
                        width: Fill, height: Fill,
                        flow: Down,

                        // View container template
                        JsonView = <View> {
                            width: Fill, height: Fit,
                            padding: 8,
                            margin: { bottom: 4 }
                            show_bg: true
                            draw_bg: { color: #1e1e2e }
                            flow: Down,
                            spacing: 8,
                        }

                        // HStack template
                        JsonHStack = <View> {
                            width: Fill, height: Fit,
                            padding: 8,
                            margin: { bottom: 4 }
                            flow: Right,
                            spacing: 8,
                        }

                        // Label template
                        JsonLabel = <View> {
                            width: Fill, height: Fit,
                            padding: 8,
                            margin: { bottom: 4 }

                            json_label_text = <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                    color: #cdd6f4
                                }
                                text: "Label"
                            }
                        }

                        // Button template
                        JsonButton = <View> {
                            width: Fit, height: Fit,
                            margin: { bottom: 4 }

                            json_button = <MpButtonPrimary> {
                                text: "Button"
                            }
                        }

                        // Card template
                        JsonCard = <MpCard> {
                            width: Fill, height: Fit,
                            margin: { bottom: 8 }
                            padding: 16,

                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 8,

                                json_card_title = <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_BOLD>{ font_size: 16.0 }
                                        color: #cdd6f4
                                    }
                                    text: "Card"
                                }

                                json_card_desc = <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                        color: #a6adc8
                                    }
                                    text: "Card description"
                                }
                            }
                        }

                        // Progress template
                        JsonProgress = <View> {
                            width: Fill, height: Fit,
                            padding: 12,
                            margin: { bottom: 4 }
                            show_bg: true
                            draw_bg: { color: #313244 }
                            flow: Down,
                            spacing: 8,

                            json_progress_label = <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                    color: #a6adc8
                                }
                                text: "Progress"
                            }

                            json_progress = <MpProgress> {
                                width: Fill, height: 8,
                                value: 50
                            }
                        }

                        // Switch template
                        JsonSwitch = <View> {
                            width: Fill, height: Fit,
                            padding: 12,
                            margin: { bottom: 4 }
                            show_bg: true
                            draw_bg: { color: #313244 }
                            flow: Right,
                            align: { y: 0.5 }
                            spacing: 12,

                            json_switch_label = <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                    color: #cdd6f4
                                }
                                text: "Switch"
                            }

                            <View> { width: Fill }

                            json_switch = <MpSwitch> {}
                        }

                        // TextInput template
                        JsonInput = <View> {
                            width: Fill, height: Fit,
                            padding: 8,
                            margin: { bottom: 4 }
                            flow: Down,
                            spacing: 4,

                            json_input_label = <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                    color: #a6adc8
                                }
                                text: "Input"
                            }

                            json_text_input = <TextInput> {
                                width: Fill, height: Fit,
                                padding: 10,
                                empty_text: "Enter text..."
                                draw_bg: { color: #45475a }
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                    color: #cdd6f4
                                }
                            }
                        }

                        // Image placeholder template
                        JsonImage = <View> {
                            width: Fill, height: 120,
                            margin: { bottom: 4 }
                            show_bg: true
                            draw_bg: { color: #313244 }
                            align: { x: 0.5, y: 0.5 }

                            <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                    color: #6c7086
                                }
                                text: "[Image Placeholder]"
                            }
                        }

                        // Divider template
                        JsonDivider = <MpDivider> {
                            margin: { top: 8, bottom: 8 }
                            draw_bg: { color: #313244 }
                        }
                    }
                }
            }
        }
    }

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: {
                    title: "Component Zoo"
                    inner_size: vec2(1280, 900)
                }

                show_bg: true
                draw_bg: { color: (BACKGROUND) }

                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Overlay,

                    // Main content area
                    main_content = <View> {
                        width: Fill,
                        height: Fill,
                        flow: Down,

                    // Header area
                    <View> {
                        width: Fill, height: Fit,
                        flow: Down,
                        padding: { left: 24, right: 24, top: 24, bottom: 16 },
                        spacing: 8,

                        <Label> {
                            draw_text: {
                                text_style: <THEME_FONT_BOLD>{ font_size: 24.0 }
                                color: (FOREGROUND)
                            }
                            text: "Component Zoo"
                        }

                        <Label> {
                            draw_text: {
                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                color: (MUTED_FOREGROUND)
                            }
                            text: "A showcase of makepad-component widgets"
                        }
                    }

                    // Category Tab Bar
                    <View> {
                        width: Fill, height: Fit,
                        padding: { left: 24, right: 24, bottom: 16 },

                        <MpTabBarPill> {
                            cat_form = <CategoryTab> { text: "Form" }
                            cat_display = <CategoryTab> { text: "Display" }
                            cat_nav = <CategoryTab> { text: "Navigation" }
                            cat_feedback = <CategoryTab> { text: "Feedback" }
                            cat_data = <CategoryTab> { text: "Data" }
                            cat_shader = <CategoryTab> { text: "Shader" }
                            cat_shader_art = <CategoryTab> { text: "Shader Art" }
                            cat_shader_art2 = <CategoryTab> { text: "Shader FBM" }
                            cat_shader_math = <CategoryTab> { text: "Shader Math" }
                            cat_splash = <CategoryTab> { text: "Splash" }
                            cat_json = <CategoryTab> { text: "JSON Render" }
                        }
                    }

                    <MpDivider> {}

                    // Content area with PageFlip
                    category_pages = <PageFlip> {
                        width: Fill,
                        height: Fill,
                        active_page: page_form,

                        // ============================================================
                        // Form Controls Page
                        // ============================================================
                        page_form = <ScrollYView> {
                            width: Fill, height: Fill,
                            flow: Down,
                            spacing: 24,
                            padding: { left: 24, right: 24, top: 24, bottom: 200 }

                            show_bg: true
                            draw_bg: { color: #e2e8f0 }

                            // ===== Button Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Button" }

                                // Button Variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,

                                        btn_primary = <MpButtonPrimary> { text: "Primary" }
                                        btn_secondary = <MpButtonSecondary> { text: "Secondary" }
                                        btn_danger = <MpButtonDanger> { text: "Danger" }
                                        btn_ghost = <MpButtonGhost> { text: "Ghost" }
                                    }
                                }

                                // Button Sizes
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Sizes" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,
                                        align: { y: 0.5 }

                                        <MpButtonSmall> { text: "Small" }
                                        <MpButton> { text: "Medium" }
                                        <MpButtonLarge> { text: "Large" }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Checkbox Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Checkbox" }

                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    checkbox1 = <MpCheckbox> { text: "Option 1" }
                                    checkbox2 = <MpCheckbox> { text: "Option 2", checked: true }
                                    checkbox3 = <MpCheckbox> { text: "Option 3" }
                                }

                                checkbox_status = <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                        color: (MUTED_FOREGROUND)
                                    }
                                    text: "Selected: Option 2"
                                }
                            }

                            <MpDivider> {}

                            // ===== Switch Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Switch" }

                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 16,

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,
                                        align: { y: 0.5 }
                                        switch_wifi = <MpSwitch> {}
                                        <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "Wi-Fi"
                                        }
                                    }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,
                                        align: { y: 0.5 }
                                        switch_bluetooth = <MpSwitch> { on: true }
                                        <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "Bluetooth"
                                        }
                                    }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,
                                        align: { y: 0.5 }
                                        switch_notifications = <MpSwitch> {}
                                        <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "Notifications"
                                        }
                                    }
                                }

                                // Multiple switches
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "All On" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpSwitch> { on: true }
                                        <MpSwitch> { on: true }
                                        <MpSwitch> { on: true }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Radio Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Radio" }

                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    radio_small = <MpRadio> { text: "Small" }
                                    radio_medium = <MpRadio> { text: "Medium", checked: true }
                                    radio_large = <MpRadio> { text: "Large" }
                                }

                                radio_status = <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                        color: (MUTED_FOREGROUND)
                                    }
                                    text: "Selected: Medium"
                                }
                            }

                            <MpDivider> {}

                            // ===== Dropdown Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Dropdown" }

                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Basic" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        dropdown_basic = <MpDropdown> {
                                            width: 200,
                                            labels: ["Apple", "Banana", "Cherry", "Date", "Elderberry"]
                                        }

                                        dropdown_status = <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (MUTED_FOREGROUND)
                                            }
                                            text: "Selected: Apple"
                                        }
                                    }
                                }

                                // Dropdown variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Variants" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,

                                        <MpDropdown> {
                                            width: 180,
                                            labels: ["Default", "Option 2", "Option 3"]
                                        }

                                        <MpDropdownOutline> {
                                            width: 180,
                                            labels: ["Outline", "Option 2", "Option 3"]
                                        }

                                        <MpDropdownGhost> {
                                            width: 180,
                                            labels: ["Ghost", "Option 2", "Option 3"]
                                        }
                                    }
                                }

                                // Dropdown sizes
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Sizes" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpDropdownSmall> {
                                            width: 140,
                                            labels: ["Small", "Option 2"]
                                        }

                                        <MpDropdown> {
                                            width: 150,
                                            labels: ["Medium", "Option 2"]
                                        }

                                        <MpDropdownLarge> {
                                            width: 160,
                                            labels: ["Large", "Option 2"]
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Slider Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Slider" }

                                // Default Slider
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Default" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        slider_default = <MpSlider> {
                                            width: 300,
                                            min: 0.0, max: 100.0, value: 50.0, step: 1.0,
                                        }

                                        slider_default_label = <Label> {
                                            width: 100, height: Fit,
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "Value: 50"
                                        }
                                    }
                                }

                                // Slider Colors
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Colors" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 12,

                                        <MpSlider> {
                                            width: 300,
                                            min: 0.0, max: 100.0, value: 60.0, step: 1.0,
                                        }

                                        <MpSliderSuccess> {
                                            width: 300,
                                            min: 0.0, max: 100.0, value: 80.0, step: 1.0,
                                        }

                                        <MpSliderWarning> {
                                            width: 300,
                                            min: 0.0, max: 100.0, value: 40.0, step: 1.0,
                                        }

                                        <MpSliderDanger> {
                                            width: 300,
                                            min: 0.0, max: 100.0, value: 20.0, step: 1.0,
                                        }
                                    }
                                }

                                // Vertical Slider
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Vertical" }

                                    <View> {
                                        width: Fill, height: 150,
                                        flow: Right,
                                        spacing: 16,

                                        slider_vert = <MpSliderVertical> {
                                            height: Fill,
                                            min: 0.0, max: 100.0, value: 30.0, step: 1.0,
                                        }

                                        slider_vert_label = <Label> {
                                            width: 120, height: Fit,
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "Vertical value: 30"
                                        }
                                    }
                                }

                                // Range Slider
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Range Slider" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        slider_range = <MpSlider> {
                                            width: 300,
                                            min: 0.0, max: 100.0,
                                            value_start: 20.0, value: 80.0,
                                            range_mode: true, step: 1.0,
                                        }

                                        slider_range_label = <Label> {
                                            width: 150, height: Fit,
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "Range: 20 - 80"
                                        }
                                    }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        slider_range_success = <MpSliderSuccess> {
                                            width: 300,
                                            min: 0.0, max: 100.0,
                                            value_start: 30.0, value: 70.0,
                                            range_mode: true, step: 5.0,
                                        }

                                        slider_range_success_label = <Label> {
                                            width: 150, height: Fit,
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "Range: 30 - 70 (step 5)"
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Input Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Input" }

                                // Input variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Variants" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,

                                        <MpInput> {
                                            width: 200,
                                            empty_text: "Default input"
                                        }

                                        <MpInputBorderless> {
                                            width: 200,
                                            empty_text: "Borderless input"
                                        }
                                    }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,

                                        <MpInputPassword> {
                                            width: 200,
                                            input = { empty_text: "Password input" }
                                        }

                                        <MpInputNumeric> {
                                            width: 200,
                                            empty_text: "Numbers only"
                                        }
                                    }
                                }

                                // Input sizes
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Sizes" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpInputSmall> {
                                            width: 150,
                                            empty_text: "Small"
                                        }

                                        <MpInput> {
                                            width: 150,
                                            empty_text: "Medium"
                                        }

                                        <MpInputLarge> {
                                            width: 150,
                                            empty_text: "Large"
                                        }
                                    }
                                }

                                // Interactive input
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Interactive" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        input_interactive = <MpInput> {
                                            width: 250,
                                            empty_text: "Type something..."
                                        }

                                        input_status = <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (MUTED_FOREGROUND)
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
                        page_display = <ScrollYView> {
                            width: Fill, height: Fill,
                            flow: Down,
                            spacing: 24,
                            padding: { left: 24, right: 24, top: 24, bottom: 100 }

                            show_bg: true
                            draw_bg: { color: #bbf7d0 }

                            // ===== Label Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Label" }

                                // Size variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Size Variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 1.0 }

                                        <MpLabelXs> { text: "Extra Small" }
                                        <MpLabelSm> { text: "Small" }
                                        <MpLabel> { text: "Medium (default)" }
                                        <MpLabelLg> { text: "Large" }
                                        <MpLabelXl> { text: "Extra Large" }
                                    }
                                }

                                // Color variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Color Variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,

                                        <MpLabel> { text: "Default" }
                                        <MpLabelMuted> { text: "Muted" }
                                        <MpLabelPrimary> { text: "Primary" }
                                        <MpLabelSuccess> { text: "Success" }
                                        <MpLabelWarning> { text: "Warning" }
                                        <MpLabelDanger> { text: "Danger" }
                                        <MpLabelInfo> { text: "Info" }
                                    }
                                }

                                // Headings
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Headings" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 8,

                                        <MpHeading1> { text: "Heading 1" }
                                        <MpHeading2> { text: "Heading 2" }
                                        <MpHeading3> { text: "Heading 3" }
                                        <MpHeading4> { text: "Heading 4" }
                                        <MpHeading5> { text: "Heading 5" }
                                        <MpHeading6> { text: "Heading 6" }
                                    }
                                }

                                // Secondary text
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "With Secondary Text" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 8,

                                        <MpLabel> {
                                            text: "Username"
                                            secondary: "(required)"
                                        }
                                        <MpLabel> {
                                            text: "Email"
                                            secondary: "optional"
                                        }
                                    }
                                }

                                // Masked text
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Masked Text (Password)" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,

                                        <MpLabel> {
                                            text: "password123"
                                            masked: true
                                        }
                                        <MpLabel> {
                                            text: "secret"
                                            masked: true
                                        }
                                    }
                                }

                                // Highlighted text
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Text Highlighting (Search)" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 8,

                                        <MpLabel> {
                                            text: "The quick brown fox jumps over the lazy dog"
                                            highlight: "fox"
                                        }
                                        <MpLabel> {
                                            text: "Hello World, Hello Universe"
                                            highlight: "hello"
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Text Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Text" }

                                // Paragraph text
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Paragraph Text (Word Wrap)" }

                                    <View> {
                                        width: 400, height: Fit,
                                        padding: 16,
                                        show_bg: true,
                                        draw_bg: { color: #ffffff }

                                        <MpText> {
                                            text: "This is a paragraph of text that demonstrates word wrapping. When the text is too long to fit on a single line, it automatically wraps to the next line. This is useful for displaying longer content like descriptions or articles."
                                        }
                                    }
                                }

                                // Size variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Size Variants" }

                                    <View> {
                                        width: 400, height: Fit,
                                        flow: Down,
                                        spacing: 12,
                                        padding: 16,
                                        show_bg: true,
                                        draw_bg: { color: #ffffff }

                                        <MpTextXs> { text: "Extra small text for fine print" }
                                        <MpTextSm> { text: "Small text for captions" }
                                        <MpText> { text: "Medium text (default body)" }
                                        <MpTextLg> { text: "Large text for emphasis" }
                                        <MpTextXl> { text: "Extra large text for intro" }
                                    }
                                }

                                // Color variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Color Variants" }

                                    <View> {
                                        width: 400, height: Fit,
                                        flow: Down,
                                        spacing: 8,
                                        padding: 16,
                                        show_bg: true,
                                        draw_bg: { color: #ffffff }

                                        <MpText> { text: "Default text color" }
                                        <MpTextMuted> { text: "Muted text for secondary info" }
                                        <MpTextPrimary> { text: "Primary colored text" }
                                        <MpTextSuccess> { text: "Success message text" }
                                        <MpTextWarning> { text: "Warning message text" }
                                        <MpTextDanger> { text: "Danger/error message text" }
                                    }
                                }

                                // Special variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Special Variants" }

                                    <View> {
                                        width: 500, height: Fit,
                                        flow: Down,
                                        spacing: 16,
                                        padding: 16,
                                        show_bg: true,
                                        draw_bg: { color: #ffffff }

                                        // Lead text
                                        <MpTextLead> {
                                            text: "This is lead text, perfect for introductory paragraphs that need to stand out."
                                        }

                                        // Inline code
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Right,
                                            spacing: 4,
                                            align: { y: 0.5 }

                                            <MpTextInline> { text: "Use the " }
                                            <MpTextCode> { text: "println!()" }
                                            <MpTextInline> { text: " macro to print output." }
                                        }

                                        // Caption
                                        <MpTextCaption> {
                                            text: "Caption: This is a small caption text often used below images or figures."
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Badge Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Badge" }

                                // Badge with count (wrapping content)
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Badge with Count" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 24,
                                        align: { y: 0.5 }

                                        // Default (red)
                                        <MpBadge> {
                                            count: 5
                                            content = {
                                                <MpButtonSecondary> { text: "Messages" }
                                            }
                                        }

                                        // Success (green)
                                        <MpBadgeSuccess> {
                                            count: 3
                                            content = {
                                                <MpButtonSecondary> { text: "Completed" }
                                            }
                                        }

                                        // Warning (orange)
                                        <MpBadgeWarning> {
                                            count: 2
                                            content = {
                                                <MpButtonSecondary> { text: "Pending" }
                                            }
                                        }
                                    }
                                }

                                // Badge color variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Color Variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 24,
                                        align: { y: 0.5 }

                                        <MpBadge> {
                                            count: 9
                                            content = {
                                                <MpButtonGhost> { text: "Default" }
                                            }
                                        }

                                        <MpBadgeInfo> {
                                            count: 12
                                            content = {
                                                <MpButtonGhost> { text: "Info" }
                                            }
                                        }

                                        <MpBadgeSecondary> {
                                            count: 7
                                            content = {
                                                <MpButtonGhost> { text: "Secondary" }
                                            }
                                        }
                                    }
                                }

                                // Dot badges
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Dot Badges" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 24,
                                        align: { y: 0.5 }

                                        <MpBadgeDot> {
                                            content = {
                                                <MpButtonSecondary> { text: "Notifications" }
                                            }
                                        }

                                        <MpBadgeDotSuccess> {
                                            content = {
                                                <MpButtonSecondary> { text: "Online" }
                                            }
                                        }

                                        <MpBadgeDotWarning> {
                                            content = {
                                                <MpButtonSecondary> { text: "Away" }
                                            }
                                        }
                                    }
                                }

                                // Standalone badges
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Standalone (inline)" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,
                                        align: { y: 0.5 }

                                        <MpBadgeStandalone> {
                                            label = { text: "5" }
                                        }
                                        <MpBadgeStandaloneSuccess> {
                                            label = { text: "New" }
                                        }
                                        <MpBadgeStandaloneWarning> {
                                            label = { text: "99+" }
                                        }
                                        <MpBadgeStandaloneInfo> {
                                            label = { text: "Beta" }
                                        }
                                    }
                                }

                                // Interactive badge
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Interactive Count" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        badge_dec_btn = <MpButtonGhost> { text: "-" }
                                        interactive_badge = <MpBadge> {
                                            count: 5
                                            content = {
                                                <MpButtonSecondary> { text: "Items" }
                                            }
                                        }
                                        badge_inc_btn = <MpButtonGhost> { text: "+" }

                                        badge_count_label = <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                                color: (MUTED_FOREGROUND)
                                            }
                                            text: "Count: 5"
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Avatar Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Avatar" }

                                // Avatar sizes
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Sizes" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpAvatarXSmall> { label = { text: "XS" } }
                                        <MpAvatarSmall> { label = { text: "SM" } }
                                        <MpAvatar> { label = { text: "MD" } }
                                        <MpAvatarLarge> { label = { text: "LG" } }
                                        <MpAvatarXLarge> { label = { text: "XL" } }
                                    }
                                }

                                // Avatar colors
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Colors" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,
                                        align: { y: 0.5 }

                                        <MpAvatar> { label = { text: "JD" } }
                                        <MpAvatarPrimary> { label = { text: "AB" } }
                                        <MpAvatarSuccess> { label = { text: "CD" } }
                                        <MpAvatarDanger> { label = { text: "EF" } }
                                        <MpAvatarWarning> { label = { text: "GH" } }
                                    }
                                }

                                // Dynamic avatar
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Dynamic (click to change)" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,
                                        align: { y: 0.5 }

                                        dynamic_avatar = <MpAvatar> { label = { text: "??" } }
                                        avatar_change_btn = <MpButtonSecondary> { text: "Random Name" }
                                        avatar_name_label = <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (MUTED_FOREGROUND)
                                            }
                                            text: "Click button..."
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Card Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Card" }

                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,

                                    // Basic Card
                                    <MpCard> {
                                        width: 250,
                                        <MpCardHeader> {
                                            <MpCardTitle> { text: "Card Title" }
                                            <MpCardDescription> { text: "Card description text." }
                                        }
                                        <MpCardContent> {
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                    color: (FOREGROUND)
                                                }
                                                text: "This is the card content area."
                                            }
                                        }
                                        <MpCardFooter> {
                                            <MpButtonGhost> { text: "Cancel" }
                                            <MpButtonPrimary> { text: "Save" }
                                        }
                                    }

                                    // Shadow Card
                                    <MpCardShadow> {
                                        width: 250,
                                        <MpCardHeader> {
                                            <MpCardTitle> { text: "Shadow Card" }
                                            <MpCardDescription> { text: "Card with shadow effect." }
                                        }
                                        <MpCardContent> {
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                    color: (FOREGROUND)
                                                }
                                                text: "Shadow creates depth."
                                            }
                                        }
                                    }
                                }

                                // Card color variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Color Variants" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 12,

                                        <MpCardSuccess> {
                                            width: 180,
                                            padding: 12,
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                    color: (SUCCESS)
                                                }
                                                text: "Success Card"
                                            }
                                        }

                                        <MpCardDanger> {
                                            width: 180,
                                            padding: 12,
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                    color: (DANGER)
                                                }
                                                text: "Danger Card"
                                            }
                                        }

                                        <MpCardWarning> {
                                            width: 180,
                                            padding: 12,
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                    color: #b45309
                                                }
                                                text: "Warning Card"
                                            }
                                        }

                                        <MpCardInfo> {
                                            width: 180,
                                            padding: 12,
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                    color: (INFO)
                                                }
                                                text: "Info Card"
                                            }
                                        }
                                    }
                                }

                                // Clickable Card
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Clickable Card (hover to see effect)" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 12,

                                        clickable_card_1 = <MpCardClickable> {
                                            width: 200,
                                            <MpCardHeader> {
                                                <MpCardTitle> { text: "Click Me" }
                                                <MpCardDescription> { text: "Hover and click" }
                                            }
                                        }

                                        clickable_card_2 = <MpCardClickable> {
                                            width: 200,
                                            <MpCardHeader> {
                                                <MpCardTitle> { text: "Interactive" }
                                                <MpCardDescription> { text: "With hover effect" }
                                            }
                                        }

                                        card_click_status = <Label> {
                                            width: Fit, height: Fit,
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (MUTED_FOREGROUND)
                                            }
                                            text: "Click a card..."
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Divider Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Divider" }

                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 20,

                                    <SubsectionLabel> { text: "Horizontal (default)" }
                                    <MpDivider> {}

                                    <SubsectionLabel> { text: "With text" }
                                    <MpDividerWithLabel> { text: "OR" }

                                    <SubsectionLabel> { text: "Thick" }
                                    <MpDividerWithMargin> {}
                                }
                            }

                            <MpDivider> {}

                            // ===== Skeleton Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Skeleton" }

                                // Interactive Skeleton Demo
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 8,
                                        align: { y: 0.5 }

                                        skeleton_toggle_btn = <MpButtonPrimary> { text: "Toggle Loading" }

                                        skeleton_status = <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (MUTED_FOREGROUND)
                                            }
                                            text: "Status: Loading"
                                        }
                                    }

                                    // Interactive skeleton widget
                                    interactive_skeleton = <MpSkeletonWidget> {
                                        width: Fill,
                                        height: Fit,

                                        skeleton = <View> {
                                            width: Fill, height: Fit,
                                            flow: Down,
                                            spacing: 8

                                            <MpSkeletonRounded> { width: 150, height: 20 }
                                            <MpSkeletonRounded> { width: Fill, height: 14 }
                                            <MpSkeletonRounded> { width: 200, height: 14 }
                                        }

                                        content = <View> {
                                            width: Fill, height: Fit,
                                            flow: Down,
                                            spacing: 8

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_BOLD>{ font_size: 16.0 }
                                                    color: (FOREGROUND)
                                                }
                                                text: "Content Loaded!"
                                            }
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "This is the actual content that appears after loading."
                                            }
                                        }
                                    }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Basic shapes" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpSkeleton> {
                                            width: 200, height: 20
                                        }

                                        <MpSkeletonCircle> {
                                            width: 48, height: 48
                                        }
                                    }
                                }

                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Card skeleton" }

                                    <MpSkeletonCard> {}
                                }
                            }

                            <MpDivider> {}

                            // ===== Spinner Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Spinner" }

                                // Size variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Size variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpSpinnerXs> {}
                                        <MpSpinnerSm> {}
                                        <MpSpinnerMd> {}
                                        <MpSpinnerLg> {}
                                        <MpSpinnerXl> {}
                                    }
                                }

                                // Color variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Color variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpSpinnerPrimary> {}
                                        <MpSpinnerSuccess> {}
                                        <MpSpinnerWarning> {}
                                        <MpSpinnerDanger> {}
                                    }
                                }

                                // Style variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Style variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpSpinnerThin> {}
                                        <MpSpinner> {}
                                        <MpSpinnerThick> {}
                                        <MpSpinnerNoTrack> {}
                                    }
                                }

                                // Speed variants
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Speed variants" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        <MpSpinnerSlow> {}
                                        <MpSpinner> {}
                                        <MpSpinnerFast> {}
                                    }
                                }

                                // Alternative styles
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Alternative styles" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 24,
                                        align: { y: 0.5 }

                                        <MpSpinnerDots> {}
                                        <MpSpinnerPulse> {}
                                    }
                                }

                                // With label
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "With label" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 32,
                                        align: { y: 0.5 }

                                        <MpSpinnerWithLabel> {}
                                        <MpSpinnerWithLabelVertical> {}
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // Navigation Page
                        // ============================================================
                        page_nav = <ScrollYView> {
                            width: Fill, height: Fill,
                            flow: Down,
                            spacing: 24,
                            padding: { left: 24, right: 24, top: 24, bottom: 100 }

                            show_bg: true
                            draw_bg: { color: #bfdbfe }

                            // ===== Tab Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Tab" }

                                // Default tabs
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Default" }

                                    <MpTabBar> {
                                        tab_home = <MpTab> { text: "Home" }
                                        tab_profile = <MpTab> { text: "Profile" }
                                        tab_settings = <MpTab> { text: "Settings" }
                                    }
                                }

                                // Underline tabs
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Underline" }

                                    <MpTabBarUnderline> {
                                        tab_u_overview = <MpTabUnderline> { text: "Overview" }
                                        tab_u_analytics = <MpTabUnderline> { text: "Analytics" }
                                        tab_u_reports = <MpTabUnderline> { text: "Reports" }
                                    }
                                }

                                // Pill tabs
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Pill" }

                                    <MpTabBarPill> {
                                        tab_p_all = <MpTabPill> { text: "All" }
                                        tab_p_active = <MpTabPill> { text: "Active" }
                                        tab_p_completed = <MpTabPill> { text: "Completed" }
                                    }
                                }

                                // Outline tabs
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Outline" }

                                    <MpTabBarOutline> {
                                        tab_o_day = <MpTabOutline> { text: "Day" }
                                        tab_o_week = <MpTabOutline> { text: "Week" }
                                        tab_o_month = <MpTabOutline> { text: "Month" }
                                    }
                                }

                                // Segmented tabs
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Segmented" }

                                    <MpTabBarSegmented> {
                                        tab_s_list = <MpTabSegmented> { text: "List" }
                                        tab_s_grid = <MpTabSegmented> { text: "Grid" }
                                        tab_s_map = <MpTabSegmented> { text: "Map" }
                                    }
                                }

                                tab_status = <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                        color: (MUTED_FOREGROUND)
                                    }
                                    text: "Selected: Home"
                                }
                            }

                            <MpDivider> {}

                            // ===== PageFlip Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "PageFlip" }

                                <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                        color: (MUTED_FOREGROUND)
                                    }
                                    text: "PageFlip enables switching between different pages/views."
                                }

                                // Page navigation buttons
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 8,

                                    page_btn_a = <MpButtonPrimary> { text: "Page A" }
                                    page_btn_b = <MpButtonGhost> { text: "Page B" }
                                    page_btn_c = <MpButtonGhost> { text: "Page C" }
                                }

                                // PageFlip container
                                <View> {
                                    width: Fill, height: 120,
                                    show_bg: true,
                                    draw_bg: {
                                        color: (MUTED)
                                    }

                                    demo_page_flip = <PageFlip> {
                                        width: Fill, height: Fill,
                                        active_page: page_a,

                                        page_a = <View> {
                                            width: Fill, height: Fill,
                                            align: { x: 0.5, y: 0.5 }
                                            show_bg: true
                                            draw_bg: { color: #dbeafe }
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_BOLD>{ font_size: 24.0 }
                                                    color: (PRIMARY)
                                                }
                                                text: "Page A Content"
                                            }
                                        }

                                        page_b = <View> {
                                            width: Fill, height: Fill,
                                            align: { x: 0.5, y: 0.5 }
                                            show_bg: true
                                            draw_bg: { color: #dcfce7 }
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_BOLD>{ font_size: 24.0 }
                                                    color: (SUCCESS)
                                                }
                                                text: "Page B Content"
                                            }
                                        }

                                        page_c = <View> {
                                            width: Fill, height: Fill,
                                            align: { x: 0.5, y: 0.5 }
                                            show_bg: true
                                            draw_bg: { color: #fee2e2 }
                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_BOLD>{ font_size: 24.0 }
                                                    color: (DANGER)
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
                        page_feedback = <ScrollYView> {
                            width: Fill, height: Fill,
                            flow: Down,
                            spacing: 24,
                            padding: { left: 24, right: 24, top: 24, bottom: 100 }

                            show_bg: true
                            draw_bg: { color: #fde68a }

                            // ===== Tooltip Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Tooltip" }

                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Positions" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 24,
                                        padding: { top: 40, bottom: 40 }

                                        <MpTooltipTop> {
                                            tip: "Tooltip on top"
                                            content = {
                                                <MpButtonSecondary> { text: "Top" }
                                            }
                                        }

                                        <MpTooltipBottom> {
                                            tip: "Tooltip on bottom"
                                            content = {
                                                <MpButtonSecondary> { text: "Bottom" }
                                            }
                                        }

                                        <MpTooltipLeft> {
                                            tip: "Tooltip on left"
                                            content = {
                                                <MpButtonSecondary> { text: "Left" }
                                            }
                                        }

                                        <MpTooltipRight> {
                                            tip: "Tooltip on right"
                                            content = {
                                                <MpButtonSecondary> { text: "Right" }
                                            }
                                        }
                                    }
                                }

                                // Delay examples
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Show Delay" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        padding: { top: 20, bottom: 20 }

                                        <MpTooltipTop> {
                                            tip: "Instant tooltip (0s delay)"
                                            show_delay: 0.0
                                            content = {
                                                <MpButtonOutline> { text: "Instant" }
                                            }
                                        }

                                        <MpTooltipTop> {
                                            tip: "Default delay (0.3s)"
                                            content = {
                                                <MpButtonOutline> { text: "Default 0.3s" }
                                            }
                                        }

                                        <MpTooltipTop> {
                                            tip: "Slow tooltip (1s delay)"
                                            show_delay: 1.0
                                            content = {
                                                <MpButtonOutline> { text: "Slow 1s" }
                                            }
                                        }

                                        <MpTooltipTop> {
                                            tip: "Very slow tooltip (2s delay)"
                                            show_delay: 2.0
                                            content = {
                                                <MpButtonOutline> { text: "Very Slow 2s" }
                                            }
                                        }
                                    }
                                }

                                // Tooltip on different components
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "On Components" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 24,
                                        align: { y: 0.5 }
                                        padding: { top: 20, bottom: 20 }

                                        // Tooltip on Checkbox
                                        <MpTooltipTop> {
                                            tip: "Check this to enable feature"
                                            content = {
                                                <MpCheckbox> {
                                                    text: "Checkbox"
                                                }
                                            }
                                        }

                                        // Tooltip on Switch
                                        <MpTooltipTop> {
                                            tip: "Toggle to turn on/off"
                                            content = {
                                                <View> {
                                                    width: Fit, height: Fit,
                                                    flow: Right,
                                                    spacing: 8,
                                                    align: { y: 0.5 }
                                                    <MpSwitch> {}
                                                    <Label> {
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                            color: (FOREGROUND)
                                                        }
                                                        text: "Switch"
                                                    }
                                                }
                                            }
                                        }

                                        // Tooltip on Radio
                                        <MpTooltipTop> {
                                            tip: "Select this option"
                                            content = {
                                                <MpRadio> {
                                                    text: "Radio"
                                                }
                                            }
                                        }

                                        // Tooltip on Icon/Label
                                        <MpTooltipTop> {
                                            tip: "This is an info icon with tooltip"
                                            content = {
                                                <Label> {
                                                    draw_text: {
                                                        text_style: <THEME_FONT_REGULAR>{ font_size: 20.0 }
                                                        color: (PRIMARY)
                                                    }
                                                    text: "ℹ️"
                                                }
                                            }
                                        }
                                    }
                                }

                                // Long text tooltip
                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Long Text" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        padding: { top: 20, bottom: 20 }

                                        <MpTooltipTop> {
                                            tip: "This is a longer tooltip text that provides more detailed information about the element being hovered."
                                            content = {
                                                <MpButtonGhost> { text: "Long tooltip" }
                                            }
                                        }

                                        <MpTooltipBottom> {
                                            tip: "Tooltips can contain helpful hints, keyboard shortcuts, or additional context for users."
                                            content = {
                                                <MpButtonGhost> { text: "Helpful hints" }
                                            }
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Progress Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Progress" }

                                // Progress variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 16,

                                    <SubsectionLabel> { text: "Values" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 12,

                                        <MpProgress> { width: 300, value: 25.0 }
                                        <MpProgress> { width: 300, value: 50.0 }
                                        <MpProgress> { width: 300, value: 75.0 }
                                        <MpProgress> { width: 300, value: 100.0 }
                                    }
                                }

                                // Progress colors
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Colors" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 12,

                                        <MpProgress> { width: 300, value: 60.0 }
                                        <MpProgressSuccess> { width: 300, value: 60.0 }
                                        <MpProgressDanger> { width: 300, value: 60.0 }
                                        <MpProgressWarning> { width: 300, value: 60.0 }
                                    }
                                }

                                // Progress widths
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Widths" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 12,

                                        <MpProgress> { width: 150, value: 50.0 }
                                        <MpProgress> { width: 250, value: 50.0 }
                                        <MpProgress> { width: 350, value: 50.0 }
                                    }
                                }

                                // Interactive progress
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Interactive" }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,
                                        align: { y: 0.5 }

                                        progress_dec_btn = <MpButtonGhost> { text: "-10" }
                                        interactive_progress = <MpProgress> { width: 200, value: 50.0 }
                                        progress_inc_btn = <MpButtonGhost> { text: "+10" }

                                        progress_label = <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (FOREGROUND)
                                            }
                                            text: "50%"
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Alert Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Alert" }

                                // Alert Variants
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Variants" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 12,

                                        <MpAlert> {
                                            content = {
                                                message = { text: "This is a default alert message." }
                                            }
                                        }

                                        <MpAlertInfo> {
                                            content = {
                                                message = { text: "This is an info alert for general information." }
                                            }
                                        }

                                        <MpAlertSuccess> {
                                            content = {
                                                message = { text: "Operation completed successfully!" }
                                            }
                                        }

                                        <MpAlertWarning> {
                                            content = {
                                                message = { text: "Please review your input before continuing." }
                                            }
                                        }

                                        <MpAlertError> {
                                            content = {
                                                message = { text: "Something went wrong. Please try again." }
                                            }
                                        }
                                    }
                                }

                                // Alert with Title
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "With Title" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 12,

                                        <MpAlertInfo> {
                                            content = {
                                                title_wrapper = { visible: true, title = { text: "Information" } }
                                                message = { text: "This alert has a title for more context." }
                                            }
                                        }

                                        <MpAlertSuccess> {
                                            content = {
                                                title_wrapper = { visible: true, title = { text: "Success!" } }
                                                message = { text: "Your changes have been saved successfully." }
                                            }
                                        }

                                        <MpAlertError> {
                                            content = {
                                                title_wrapper = { visible: true, title = { text: "Error" } }
                                                message = { text: "Failed to connect to the server. Check your network." }
                                            }
                                        }
                                    }
                                }

                                // Closable Alert
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Closable" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 12,

                                        closable_alert = <MpAlertInfo> {
                                            closable: true
                                            content = {
                                                message = { text: "This alert can be closed. Click the X button." }
                                            }
                                        }

                                        closable_alert_warning = <MpAlertWarning> {
                                            closable: true
                                            content = {
                                                title_wrapper = { visible: true, title = { text: "Warning" } }
                                                message = { text: "This is a closable warning with title." }
                                            }
                                        }
                                    }
                                }

                                // Banner Alerts
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 8,

                                    <SubsectionLabel> { text: "Banner Style" }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Down,
                                        spacing: 0,

                                        <MpAlertBannerInfo> {
                                            content = {
                                                message = { text: "Info banner - full width, no border radius" }
                                            }
                                        }

                                        <MpAlertBannerSuccess> {
                                            closable: true
                                            content = {
                                                message = { text: "Success banner with close button" }
                                            }
                                        }

                                        <MpAlertBannerWarning> {
                                            content = {
                                                message = { text: "Warning banner alert" }
                                            }
                                        }

                                        <MpAlertBannerError> {
                                            closable: true
                                            content = {
                                                message = { text: "Error banner - something needs attention!" }
                                            }
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Notification Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Notification" }

                                // Interactive Notification Demo
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 8,

                                    show_success_notif = <MpButtonSuccess> { text: "Success" }
                                    show_error_notif = <MpButtonDanger> { text: "Error" }
                                    show_warning_notif = <MpButtonWarning> { text: "Warning" }
                                    show_info_notif = <MpButtonPrimary> { text: "Info" }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                        color: (MUTED_FOREGROUND)
                                    }
                                    text: "Notification previews (static):"
                                }

                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <MpNotification> {
                                        content = {
                                            title = { text: "Notification" }
                                            message = { text: "This is a default notification message." }
                                        }
                                    }

                                    <MpNotificationSuccess> {
                                        content = {
                                            title = { text: "Success" }
                                            message = { text: "Operation completed successfully!" }
                                        }
                                    }

                                    <MpNotificationError> {
                                        content = {
                                            title = { text: "Error" }
                                            message = { text: "Something went wrong. Please try again." }
                                        }
                                    }

                                    <MpNotificationWarning> {
                                        content = {
                                            title = { text: "Warning" }
                                            message = { text: "Please review your input before continuing." }
                                        }
                                    }

                                    <MpNotificationInfo> {
                                        content = {
                                            title = { text: "Info" }
                                            message = { text: "Here's some helpful information." }
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Modal Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Modal" }

                                // Interactive Modal Demo
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: { y: 0.5 }

                                    open_modal_btn = <MpButtonPrimary> { text: "Open Modal" }

                                    modal_status = <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                            color: (MUTED_FOREGROUND)
                                        }
                                        text: "Click button to open modal"
                                    }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                <Label> {
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                        color: (MUTED_FOREGROUND)
                                    }
                                    text: "Modal previews (static):"
                                }

                                // Basic Modal preview
                                <MpModal> {
                                    width: 350,
                                    header = {
                                        title = { text: "Modal Title" }
                                    }
                                    body = {
                                        <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (MUTED_FOREGROUND)
                                            }
                                            text: "This is the modal content area."
                                        }
                                    }
                                    footer = {
                                        <MpButtonGhost> { text: "Cancel" }
                                        <MpButtonPrimary> { text: "Confirm" }
                                    }
                                }

                                // Alert Dialog preview
                                <MpAlertDialog> {
                                    width: 320,
                                    header = {
                                        title = { text: "Are you sure?" }
                                    }
                                    body = {
                                        <Label> {
                                            draw_text: {
                                                text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                color: (MUTED_FOREGROUND)
                                            }
                                            text: "This action cannot be undone."
                                        }
                                    }
                                    footer = {
                                        <MpButtonGhost> { text: "Cancel" }
                                        <MpButtonDanger> { text: "Delete" }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Popover Section (Ant Design Style) =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 24,

                                <SectionHeader> { text: "Popover" }

                                // ===== Basic Usage =====
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Basic" }
                                    <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                            color: (MUTED_FOREGROUND)
                                        }
                                        text: "The most basic example. The size of the floating layer depends on the contents region."
                                    }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 16,

                                        <MpPopoverBottom> {
                                            trigger: Hover
                                            <MpButtonPrimary> { text: "Hover me" }
                                            content = {
                                                <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                            }
                                        }
                                    }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                // ===== Trigger Types (Ant Design: hover, focus, click) =====
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Three trigger modes" }
                                    <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                            color: (MUTED_FOREGROUND)
                                        }
                                        text: "Mouse to click, focus and hover."
                                    }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Right,
                                        spacing: 12,

                                        // Hover trigger
                                        <MpPopoverBottom> {
                                            trigger: Hover
                                            <MpButton> { text: "Hover me" }
                                            content = {
                                                <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                            }
                                        }

                                        // Focus trigger
                                        <MpPopoverBottom> {
                                            trigger: Focus
                                            <MpButton> { text: "Focus me" }
                                            content = {
                                                <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                            }
                                        }

                                        // Click trigger
                                        <MpPopoverBottom> {
                                            trigger: Focus
                                            <MpButton> { text: "Click me" }
                                            content = {
                                                <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                            }
                                        }
                                    }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                // ===== Placement (12 positions - Ant Design style layout) =====
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Placement" }
                                    <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                            color: (MUTED_FOREGROUND)
                                        }
                                        text: "There are 12 placement options available."
                                    }

                                    // Ant Design style placement grid
                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Down,
                                        spacing: 8,
                                        align: { x: 0.5 }
                                        padding: { top: 16 }

                                        // Top row: TL, Top, TR
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Right,
                                            spacing: 8,

                                            <MpPopoverTopLeft> {
                                                <MpButton> { width: 80, text: "TL" }
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }

                                            <MpPopoverTop> {
                                                <MpButton> { width: 80, text: "Top" }
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }

                                            <MpPopoverTopRight> {
                                                <MpButton> { width: 80, text: "TR" }
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }
                                        }

                                        // Middle section with Left/Right columns
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Right,
                                            spacing: 156,  // Space between left and right columns

                                            // Left column: LT, Left, LB
                                            <View> {
                                                width: Fit, height: Fit,
                                                flow: Down,
                                                spacing: 8,

                                                <MpPopoverLeftTop> {
                                                    <MpButton> { width: 80, text: "LT" }
                                                    content = {
                                                        <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    }
                                                }

                                                <MpPopoverLeft> {
                                                    <MpButton> { width: 80, text: "Left" }
                                                    content = {
                                                        <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    }
                                                }

                                                <MpPopoverLeftBottom> {
                                                    <MpButton> { width: 80, text: "LB" }
                                                    content = {
                                                        <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    }
                                                }
                                            }

                                            // Right column: RT, Right, RB
                                            <View> {
                                                width: Fit, height: Fit,
                                                flow: Down,
                                                spacing: 8,

                                                <MpPopoverRightTop> {
                                                    <MpButton> { width: 80, text: "RT" }
                                                    content = {
                                                        <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    }
                                                }

                                                <MpPopoverRight> {
                                                    <MpButton> { width: 80, text: "Right" }
                                                    content = {
                                                        <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    }
                                                }

                                                <MpPopoverRightBottom> {
                                                    <MpButton> { width: 80, text: "RB" }
                                                    content = {
                                                        <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                        <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    }
                                                }
                                            }
                                        }

                                        // Bottom row: BL, Bottom, BR
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Right,
                                            spacing: 8,

                                            <MpPopoverBottomLeft> {
                                                <MpButton> { width: 80, text: "BL" }
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }

                                            <MpPopoverBottom> {
                                                <MpButton> { width: 80, text: "Bottom" }
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }

                                            <MpPopoverBottomRight> {
                                                <MpButton> { width: 80, text: "BR" }
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }
                                        }
                                    }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                // ===== Arrow (Show/Hide) =====
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Arrow" }
                                    <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                            color: (MUTED_FOREGROUND)
                                        }
                                        text: "You can display an arrow pointing to the target element."
                                    }

                                    // Arrow variants display (static)
                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 24,

                                        // Arrow pointing up
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 4,

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "Arrow Up"
                                            }
                                            <MpPopoverArrowUp> {
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }
                                        }

                                        // Arrow pointing down
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 4,

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "Arrow Down"
                                            }
                                            <MpPopoverArrowDown> {
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }
                                        }

                                        // Arrow pointing left
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 4,

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "Arrow Left"
                                            }
                                            <MpPopoverArrowLeft> {
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }
                                        }

                                        // Arrow pointing right
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 4,

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "Arrow Right"
                                            }
                                            <MpPopoverArrowRight> {
                                                content = {
                                                    <Label> { draw_text: { text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }, color: (FOREGROUND) }, text: "Title" }
                                                    <Label> { draw_text: { text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }, color: (MUTED_FOREGROUND) }, text: "Content" }
                                                }
                                            }
                                        }
                                    }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                // ===== Controlling the close of the dialog =====
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Controlling the close of the dialog" }
                                    <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                            color: (MUTED_FOREGROUND)
                                        }
                                        text: "Use open prop to control the display of the card."
                                    }

                                    <View> {
                                        width: Fit, height: Fit,
                                        flow: Overlay,

                                        popover_trigger_btn = <MpButtonPrimary> { text: "Click me" }

                                        <View> {
                                            width: Fit, height: Fit,
                                            margin: { top: 44 }

                                            interactive_popover = <MpPopoverWidget> {
                                                content = <MpPopoverBase> {
                                                    width: 200, height: Fit,
                                                    padding: 12,
                                                    flow: Down,
                                                    spacing: 8,

                                                    <Label> {
                                                        draw_text: {
                                                            text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                                                            color: (FOREGROUND)
                                                        }
                                                        text: "Title"
                                                    }
                                                    <Label> {
                                                        width: Fill,
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                                            color: (MUTED_FOREGROUND)
                                                            wrap: Word
                                                        }
                                                        text: "Content"
                                                    }
                                                    <Label> {
                                                        width: Fill,
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                                            color: (MUTED_FOREGROUND)
                                                            wrap: Word
                                                        }
                                                        text: "Content"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                <MpDivider> { margin: { top: 8, bottom: 8 } }

                                // ===== Popover Content Styles =====
                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Down,
                                    spacing: 12,

                                    <SubsectionLabel> { text: "Content Styles (Static Preview)" }
                                    <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                            color: (MUTED_FOREGROUND)
                                        }
                                        text: "Different content styles for popover."
                                    }

                                    <View> {
                                        width: Fill, height: Fit,
                                        flow: Right,
                                        spacing: 24,
                                        align: { y: 0.0 }

                                        // Basic Popover
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 4,

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "Basic"
                                            }
                                            <MpPopover> {
                                                width: 180,
                                                <Label> {
                                                    width: Fill,
                                                    height: Fit,
                                                    draw_text: {
                                                        text_style: <THEME_FONT_REGULAR>{ font_size: 13.0 }
                                                        color: (FOREGROUND)
                                                        wrap: Word
                                                    }
                                                    text: "Content"
                                                }
                                            }
                                        }

                                        // Popover with Header
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 4,

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "With Title"
                                            }
                                            <MpPopoverWithHeader> {
                                                width: 200,
                                                header = {
                                                    title_label = { text: "Title" }
                                                }
                                                body = {
                                                    desc_label = { text: "Content" }
                                                }
                                            }
                                        }

                                        // Menu Popover
                                        <View> {
                                            width: Fit, height: Fit,
                                            flow: Down,
                                            spacing: 4,

                                            <Label> {
                                                draw_text: {
                                                    text_style: <THEME_FONT_REGULAR>{ font_size: 11.0 }
                                                    color: (MUTED_FOREGROUND)
                                                }
                                                text: "Menu"
                                            }
                                            <MpPopoverMenu> {
                                                width: 160,
                                                <MpPopoverMenuItem> { label = { text: "Edit" } }
                                                <MpPopoverMenuItem> { label = { text: "Duplicate" } }
                                                <MpPopoverMenuDivider> {}
                                                <MpPopoverMenuItemDanger> { label = { text: "Delete" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // Data Page
                        // ============================================================
                        page_data = <ScrollYView> {
                            width: Fill, height: Fill,
                            flow: Down,
                            spacing: 24,
                            padding: { left: 24, right: 24, top: 24, bottom: 100 }

                            show_bg: true
                            draw_bg: { color: #fbcfe8 }

                            // ===== List Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "List" }

                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 24,

                                    // Basic List
                                    <View> {
                                        width: 280, height: Fit,
                                        flow: Down,
                                        spacing: 8,

                                        <SubsectionLabel> { text: "Basic List" }

                                        <MpListDivided> {
                                            <MpListItem> {
                                                <MpListItemContent> {
                                                    <MpListItemTitle> { text: "List Item 1" }
                                                    <MpListItemDescription> { text: "Description for item 1" }
                                                }
                                            }
                                            <MpListDividerFull> {}
                                            <MpListItem> {
                                                <MpListItemContent> {
                                                    <MpListItemTitle> { text: "List Item 2" }
                                                    <MpListItemDescription> { text: "Description for item 2" }
                                                }
                                            }
                                            <MpListDividerFull> {}
                                            <MpListItem> {
                                                <MpListItemContent> {
                                                    <MpListItemTitle> { text: "List Item 3" }
                                                    <MpListItemDescription> { text: "Description for item 3" }
                                                }
                                            }
                                        }
                                    }

                                    // List with avatars
                                    <View> {
                                        width: 280, height: Fit,
                                        flow: Down,
                                        spacing: 8,

                                        <SubsectionLabel> { text: "With Avatar" }

                                        <MpListDivided> {
                                            <MpListItem> {
                                                <MpListItemLeading> {
                                                    <MpAvatarSmall> { label = { text: "JD" } }
                                                }
                                                <MpListItemContent> {
                                                    <MpListItemTitle> { text: "John Doe" }
                                                    <MpListItemDescription> { text: "Software Engineer" }
                                                }
                                            }
                                            <MpListDividerFull> {}
                                            <MpListItem> {
                                                <MpListItemLeading> {
                                                    <MpAvatarSmall> { label = { text: "AS" } }
                                                }
                                                <MpListItemContent> {
                                                    <MpListItemTitle> { text: "Alice Smith" }
                                                    <MpListItemDescription> { text: "Product Manager" }
                                                }
                                            }
                                            <MpListDividerFull> {}
                                            <MpListItem> {
                                                <MpListItemLeading> {
                                                    <MpAvatarSmall> { label = { text: "BJ" } }
                                                }
                                                <MpListItemContent> {
                                                    <MpListItemTitle> { text: "Bob Johnson" }
                                                    <MpListItemDescription> { text: "Designer" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Accordion Section =====
                            <View> {
                                width: Fill, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Accordion" }

                                <View> {
                                    width: Fill, height: Fit,
                                    flow: Right,
                                    spacing: 24,

                                    // Basic Accordion
                                    <View> {
                                        width: 320, height: Fit,
                                        flow: Down,
                                        spacing: 8,

                                        <SubsectionLabel> { text: "Basic" }

                                        <MpAccordion> {
                                            <MpAccordionItem> {
                                                header = <MpAccordionHeaderBase> {
                                                    label = { text: "Section 1" }
                                                }
                                                body = {
                                                    <Label> {
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                            color: (MUTED_FOREGROUND)
                                                        }
                                                        text: "Content for section 1."
                                                    }
                                                }
                                            }

                                            <MpAccordionDivider> {}

                                            <MpAccordionItem> {
                                                header = <MpAccordionHeaderBase> {
                                                    label = { text: "Section 2" }
                                                }
                                                body = {
                                                    <Label> {
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                            color: (MUTED_FOREGROUND)
                                                        }
                                                        text: "Content for section 2."
                                                    }
                                                }
                                            }

                                            <MpAccordionDivider> {}

                                            <MpAccordionItem> {
                                                header = <MpAccordionHeaderBase> {
                                                    label = { text: "Section 3" }
                                                }
                                                body = {
                                                    <Label> {
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                            color: (MUTED_FOREGROUND)
                                                        }
                                                        text: "Content for section 3."
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Bordered Accordion
                                    <View> {
                                        width: 320, height: Fit,
                                        flow: Down,
                                        spacing: 8,

                                        <SubsectionLabel> { text: "Bordered" }

                                        <MpAccordionBordered> {
                                            <MpAccordionItemBordered> {
                                                header = <MpAccordionHeaderBase> {
                                                    label = { text: "FAQ Item 1" }
                                                }
                                                body = {
                                                    <Label> {
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                            color: (MUTED_FOREGROUND)
                                                        }
                                                        text: "Answer to FAQ 1."
                                                    }
                                                }
                                            }

                                            <MpAccordionItemBordered> {
                                                header = <MpAccordionHeaderBase> {
                                                    label = { text: "FAQ Item 2" }
                                                }
                                                body = {
                                                    <Label> {
                                                        draw_text: {
                                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                                            color: (MUTED_FOREGROUND)
                                                        }
                                                        text: "Answer to FAQ 2."
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            <MpDivider> {}

                            // ===== Interactive Demo =====
                            <View> {
                                width: Fit, height: Fit,
                                flow: Down,
                                spacing: 16,

                                <SectionHeader> { text: "Interactive Demo" }

                                <View> {
                                    width: Fit, height: Fit,
                                    flow: Right,
                                    spacing: 16,
                                    align: { y: 0.5 }

                                    counter_btn = <MpButtonPrimary> { text: "Click me!" }

                                    counter_label = <Label> {
                                        draw_text: {
                                            text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                            color: (FOREGROUND)
                                        }
                                        text: "Clicked: 0 times"
                                    }
                                }
                            }
                        }

                        // ============================================================
                        // Shader Page - Shadertoy-style fractal effect
                        // ============================================================
                        page_shader = <View> {
                            width: Fill, height: Fill,
                            flow: Down,
                            padding: 24,
                            spacing: 16,

                            show_bg: true
                            draw_bg: { color: #1a1a2e }

                            <SectionHeader> {
                                draw_text: { color: #ffffff }
                                text: "Shader Art"
                            }

                            <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                    color: #a0a0a0
                                }
                                text: "Code golf shader - fractal rainbow flow"
                            }

                            // Shader display area with animated time
                            shader_canvas = <ShaderCanvas> {
                                width: Fill, height: Fill,
                            }
                        }

                        // ============================================================
                        // Shader Art Page - Observer effect
                        // ============================================================
                        page_shader_art = <View> {
                            width: Fill, height: Fill,
                            flow: Down,
                            padding: 24,
                            spacing: 16,

                            show_bg: true
                            draw_bg: { color: #0a0a0f }

                            <SectionHeader> {
                                draw_text: { color: #ffffff }
                                text: "Shader Art - Observer"
                            }

                            <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                    color: #a0a0a0
                                }
                                text: "Code golf shader - glowing lattice observer effect"
                            }

                            // Speed control
                            <View> {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 16,
                                align: { y: 0.5 }

                                <Label> {
                                    width: Fit,
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                        color: #ffffff
                                    }
                                    text: "Speed:"
                                }

                                shader_art_speed = <MpSlider> {
                                    width: 200, height: 24,
                                    min: 0.1,
                                    max: 3.0,
                                    value: 1.0,
                                    step: 0.1,
                                }

                                shader_art_speed_label = <Label> {
                                    width: 60,
                                    draw_text: {
                                        text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                                        color: #89b4fa
                                    }
                                    text: "1.0x"
                                }
                            }

                            // Shader display area
                            shader_art_canvas = <ShaderArtCanvas> {
                                width: Fill, height: Fill,
                            }
                        }

                        // ============================================================
                        // Shader FBM Page - Domain warped noise art
                        // ============================================================
                        page_shader_art2 = <View> {
                            width: Fill, height: Fill,
                            flow: Down,
                            padding: 24,
                            spacing: 16,

                            show_bg: true
                            draw_bg: { color: #0a0a0f }

                            <SectionHeader> {
                                draw_text: { color: #ffffff }
                                text: "Shader Art - FBM Noise"
                            }

                            <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                    color: #a0a0a0
                                }
                                text: "Domain-warped FBM noise with HSV cycling and bitmap text"
                            }

                            // Speed control
                            <View> {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 16,
                                align: { y: 0.5 }

                                <Label> {
                                    width: Fit,
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                        color: #ffffff
                                    }
                                    text: "Speed:"
                                }

                                shader_art2_speed = <MpSlider> {
                                    width: 200, height: 24,
                                    min: 0.1,
                                    max: 3.0,
                                    value: 1.0,
                                    step: 0.1,
                                }

                                shader_art2_speed_label = <Label> {
                                    width: 60,
                                    draw_text: {
                                        text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                                        color: #89b4fa
                                    }
                                    text: "1.0x"
                                }
                            }

                            // Shader display area
                            shader_art2_canvas = <ShaderArt2Canvas> {
                                width: Fill, height: Fill,
                            }
                        }

                        // ============================================================
                        // Shader Math Page - Parametric flow field
                        // ============================================================
                        page_shader_math = <View> {
                            width: Fill, height: Fill,
                            flow: Down,
                            padding: 24,
                            spacing: 16,

                            show_bg: true
                            draw_bg: { color: #080812 }

                            <SectionHeader> {
                                draw_text: { color: #ffffff }
                                text: "Shader Math - Jellyfish"
                            }

                            <Label> {
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR>{ font_size: 12.0 }
                                    color: #a0a0a0
                                }
                                text: "Point-cloud forward mapping: (x,y) -> (u,v) via parametric formulas"
                            }

                            // Speed control
                            <View> {
                                width: Fill, height: Fit,
                                flow: Right,
                                spacing: 16,
                                align: { y: 0.5 }

                                <Label> {
                                    width: Fit,
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                        color: #ffffff
                                    }
                                    text: "Speed:"
                                }

                                shader_math_speed = <MpSlider> {
                                    width: 200, height: 24,
                                    min: 0.1,
                                    max: 3.0,
                                    value: 1.0,
                                    step: 0.1,
                                }

                                shader_math_speed_label = <Label> {
                                    width: 60,
                                    draw_text: {
                                        text_style: <THEME_FONT_BOLD>{ font_size: 14.0 }
                                        color: #89b4fa
                                    }
                                    text: "1.0x"
                                }
                            }

                            // Shader display area
                            shader_math_canvas = <ShaderMathCanvas> {
                                width: Fill, height: Fill,
                            }
                        }

                        // ============================================================
                        // Splash Page - Dynamic scripting showcase
                        // ============================================================
                        page_splash = <SplashDemo> {}
                        page_json = <JsonRenderDemo> {}
                    }
                    } // close main_content

                    // Modal overlay - must be after main_content to appear on top
                    demo_modal = <MpModalWidget> {
                    content = {
                        dialog = <MpModal> {
                            width: 400,
                            header = {
                                title = { text: "Interactive Modal" }
                            }
                            body = {
                                <Label> {
                                    width: Fill,
                                    height: Fit,
                                    draw_text: {
                                        text_style: <THEME_FONT_REGULAR>{ font_size: 14.0 }
                                        color: (MUTED_FOREGROUND)
                                        wrap: Word
                                    }
                                    text: "This is an interactive modal dialog. Click the X button or the backdrop to close it."
                                }
                            }
                            footer = {
                                modal_cancel_btn = <MpButtonGhost> { text: "Cancel" }
                                modal_confirm_btn = <MpButtonPrimary> { text: "Confirm" }
                            }
                        }
                    }
                } // close demo_modal

                    // Notification overlay - positioned at top-right
                    <View> {
                        width: Fill,
                        height: Fill,
                        align: { x: 1.0, y: 0.0 }
                        padding: { top: 20, right: 20 }

                        demo_notification = <MpNotificationWidget> {
                            content = {
                                title = { text: "Notification" }
                                message = { text: "This is an interactive notification!" }
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
#[derive(Live, LiveHook, Widget)]
pub struct ShaderCanvas {
    #[deref] view: View,
    #[animator] animator: Animator,
}

impl Widget for ShaderCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Start time animation
        self.animator_play(cx, ids!(anim.on));
        self.view.draw_walk(cx, scope, walk)
    }
}

// ============================================================
// ShaderArtCanvas - Observer shader widget
// ============================================================
#[derive(Live, LiveHook, Widget)]
pub struct ShaderArtCanvas {
    #[deref] view: View,
    #[animator] animator: Animator,
    #[live] speed: f64,
}

impl Widget for ShaderArtCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Apply speed to shader
        self.view.apply_over(cx, live!{
            draw_bg: { speed: (self.speed) }
        });
        // Start time animation
        self.animator_play(cx, ids!(anim.on));
        self.view.draw_walk(cx, scope, walk)
    }
}


// ============================================================
// ShaderArt2Canvas - FBM noise art widget
// ============================================================
#[derive(Live, LiveHook, Widget)]
pub struct ShaderArt2Canvas {
    #[deref] view: View,
    #[animator] animator: Animator,
    #[live] speed: f64,
}

impl Widget for ShaderArt2Canvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.apply_over(cx, live!{
            draw_bg: { speed: (self.speed) }
        });
        self.animator_play(cx, ids!(anim.on));
        self.view.draw_walk(cx, scope, walk)
    }
}

// ============================================================
// ShaderMathCanvas - Parametric flow field widget
// ============================================================
#[derive(Live, LiveHook, Widget)]
pub struct ShaderMathCanvas {
    #[deref] view: View,
    #[animator] animator: Animator,
    #[live] speed: f64,
}

impl Widget for ShaderMathCanvas {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.apply_over(cx, live!{
            draw_bg: { speed: (self.speed) }
        });
        self.animator_play(cx, ids!(anim.on));
        self.view.draw_walk(cx, scope, walk)
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

#[derive(Live, Widget)]
pub struct SplashDemo {
    #[deref] view: View,
    #[rust] widgets: Vec<GeneratedWidget>,
}

impl LiveHook for SplashDemo {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.widgets = Vec::new();
    }
}

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
                    text: text.trim().to_string()
                });
            }
            if rest == "button" {
                return Some(GeneratedWidget::Button {
                    text: "Button".to_string()
                });
            }

            // Label: "add label Hello World"
            if let Some(text) = rest.strip_prefix("label ") {
                return Some(GeneratedWidget::Label {
                    text: text.trim().to_string()
                });
            }
            if rest == "label" {
                return Some(GeneratedWidget::Label {
                    text: "Label".to_string()
                });
            }

            // Card: "add card User Profile"
            if let Some(title) = rest.strip_prefix("card ") {
                return Some(GeneratedWidget::Card {
                    title: title.trim().to_string()
                });
            }
            if rest == "card" {
                return Some(GeneratedWidget::Card {
                    title: "Card".to_string()
                });
            }

            // Progress: "add progress 75"
            if let Some(val) = rest.strip_prefix("progress ") {
                if let Ok(v) = val.trim().parse::<f64>() {
                    return Some(GeneratedWidget::Progress {
                        value: (v / 100.0).clamp(0.0, 1.0)
                    });
                }
            }
            if rest == "progress" {
                return Some(GeneratedWidget::Progress { value: 0.5 });
            }

            // Switch: "add switch Dark Mode"
            if let Some(label) = rest.strip_prefix("switch ") {
                return Some(GeneratedWidget::Switch {
                    label: label.trim().to_string()
                });
            }
            if rest == "switch" {
                return Some(GeneratedWidget::Switch {
                    label: "Toggle".to_string()
                });
            }

            // Input: "add input Email address"
            if let Some(placeholder) = rest.strip_prefix("input ") {
                return Some(GeneratedWidget::Input {
                    placeholder: placeholder.trim().to_string()
                });
            }
            if rest == "input" {
                return Some(GeneratedWidget::Input {
                    placeholder: "Enter text...".to_string()
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
        if self.view.mp_button(ids!(generate_btn)).clicked(&actions) {
            let input_text = self.view.text_input(ids!(command_input)).text();

            if input_text.trim().to_lowercase() == "clear" {
                self.widgets.clear();
            } else if let Some(widget) = self.parse_command(&input_text) {
                self.widgets.push(widget);
            }

            // Update count label
            self.view.label(ids!(widget_count_label))
                .set_text(cx, &format!("{} widgets", self.widgets.len()));

            // Clear input
            self.view.text_input(ids!(command_input)).set_text(cx, "");
            self.redraw(cx);
        }

        // Clear button clicked
        if self.view.mp_button(ids!(clear_btn)).clicked(&actions) {
            self.widgets.clear();
            self.view.label(ids!(widget_count_label))
                .set_text(cx, "0 widgets");
            self.redraw(cx);
        }

        // Handle Enter key in text input
        if let Event::KeyDown(ke) = event {
            if ke.key_code == KeyCode::ReturnKey {
                let input_text = self.view.text_input(ids!(command_input)).text();

                if input_text.trim().to_lowercase() == "clear" {
                    self.widgets.clear();
                } else if let Some(widget) = self.parse_command(&input_text) {
                    self.widgets.push(widget);
                }

                self.view.label(ids!(widget_count_label))
                    .set_text(cx, &format!("{} widgets", self.widgets.len()));
                self.view.text_input(ids!(command_input)).set_text(cx, "");
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
                                item_widget.mp_button(ids!(gen_button)).set_text(text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Label { text } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenLabel));
                                item_widget.label(ids!(gen_label)).set_text(cx, text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Card { title } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenCard));
                                item_widget.label(ids!(card_title)).set_text(cx, title);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Progress { value } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenProgress));
                                let percent = (*value * 100.0) as u32;
                                item_widget.label(ids!(progress_label))
                                    .set_text(cx, &format!("Progress: {}%", percent));
                                item_widget.mp_progress(ids!(gen_progress)).set_value(cx, percent as f64);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Switch { label } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenSwitch));
                                item_widget.label(ids!(switch_label)).set_text(cx, label);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            GeneratedWidget::Input { placeholder } => {
                                let item_widget = list.item(cx, item_id, live_id!(GenInput));
                                item_widget.label(ids!(input_label)).set_text(cx, placeholder);
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

#[derive(Live, Widget)]
pub struct JsonRenderDemo {
    #[deref] view: View,
    #[rust] flat_widgets: Vec<FlatWidget>,
}

impl LiveHook for JsonRenderDemo {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.flat_widgets = Vec::new();
    }
}

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

        serde_json::from_str(json_str)
            .map_err(|e| format!("JSON Parse Error: {}", e))
    }

    /// Flatten widget tree for PortalList rendering
    fn flatten_widgets(widget: &JsonWidget, result: &mut Vec<FlatWidget>) {
        match widget {
            JsonWidget::View { children, .. } |
            JsonWidget::VStack { children, .. } |
            JsonWidget::HStack { children, .. } => {
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
                    label: props.label.clone().unwrap_or_else(|| format!("{}%", props.value as i32)),
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
                    placeholder: props.placeholder.clone().unwrap_or_else(|| "Enter text...".to_string()),
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
        if self.view.mp_button(ids!(render_btn)).clicked(&actions) {
            let json_text = self.view.text_input(ids!(json_input)).text();

            match self.parse_json(&json_text) {
                Ok(widget_tree) => {
                    self.flat_widgets.clear();
                    Self::flatten_widgets(&widget_tree, &mut self.flat_widgets);

                    self.view.label(ids!(render_status))
                        .set_text(cx, &format!("{} widgets rendered", self.flat_widgets.len()));
                }
                Err(e) => {
                    self.view.label(ids!(render_status))
                        .set_text(cx, &format!("Error: {}", e));
                }
            }

            self.redraw(cx);
        }

        // Clear button clicked
        if self.view.mp_button(ids!(clear_render_btn)).clicked(&actions) {
            self.flat_widgets.clear();
            self.view.text_input(ids!(json_input)).set_text(cx, "");
            self.view.label(ids!(render_status)).set_text(cx, "Ready");
            self.redraw(cx);
        }

        // Load example button clicked
        if self.view.mp_button(ids!(load_example_btn)).clicked(&actions) {
            self.view.text_input(ids!(json_input))
                .set_text(cx, Self::get_example_json());
            self.view.label(ids!(render_status)).set_text(cx, "Basic example loaded");
            self.redraw(cx);
        }

        // Load Raycast example button clicked
        if self.view.mp_button(ids!(load_raycast_btn)).clicked(&actions) {
            self.view.text_input(ids!(json_input))
                .set_text(cx, Self::get_raycast_example_json());
            self.view.label(ids!(render_status)).set_text(cx, "Raycast example loaded");
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
                                item_widget.label(ids!(json_label_text)).set_text(cx, text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Button { text } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonButton));
                                item_widget.mp_button(ids!(json_button)).set_text(text);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Card { title, description } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonCard));
                                item_widget.label(ids!(json_card_title)).set_text(cx, title);
                                item_widget.label(ids!(json_card_desc)).set_text(cx, description);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Progress { value, label } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonProgress));
                                item_widget.label(ids!(json_progress_label)).set_text(cx, label);
                                item_widget.mp_progress(ids!(json_progress)).set_value(cx, *value);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Switch { label } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonSwitch));
                                item_widget.label(ids!(json_switch_label)).set_text(cx, label);
                                item_widget.draw_all(cx, &mut Scope::empty());
                            }
                            FlatWidget::Input { label, placeholder } => {
                                let item_widget = list.item(cx, item_id, live_id!(JsonInput));
                                item_widget.label(ids!(json_input_label)).set_text(cx, label);
                                item_widget.text_input(ids!(json_text_input)).set_text(cx, "");
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

#[derive(Live, LiveHook)]
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

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
        makepad_component::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        self.counter = 0;
        self.current_category = 0;

        // Set initial category tab as selected
        self.ui.mp_tab(ids!(cat_form)).set_selected(cx, true);

        // Initialize skeleton in loading state
        self.ui.mp_skeleton_widget(ids!(interactive_skeleton)).set_loading(cx, true);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle category tab clicks
        if self.ui.mp_tab(ids!(cat_form)).clicked(&actions) {
            self.select_category(cx, 0);
        }
        if self.ui.mp_tab(ids!(cat_display)).clicked(&actions) {
            self.select_category(cx, 1);
        }
        if self.ui.mp_tab(ids!(cat_nav)).clicked(&actions) {
            self.select_category(cx, 2);
        }
        if self.ui.mp_tab(ids!(cat_feedback)).clicked(&actions) {
            self.select_category(cx, 3);
        }
        if self.ui.mp_tab(ids!(cat_data)).clicked(&actions) {
            self.select_category(cx, 4);
        }
        if self.ui.mp_tab(ids!(cat_shader)).clicked(&actions) {
            self.select_category(cx, 5);
        }
        if self.ui.mp_tab(ids!(cat_shader_art)).clicked(&actions) {
            self.select_category(cx, 6);
        }
        if self.ui.mp_tab(ids!(cat_shader_art2)).clicked(&actions) {
            self.select_category(cx, 7);
        }
        if self.ui.mp_tab(ids!(cat_shader_math)).clicked(&actions) {
            self.select_category(cx, 8);
        }
        if self.ui.mp_tab(ids!(cat_splash)).clicked(&actions) {
            self.select_category(cx, 9);
        }
        if self.ui.mp_tab(ids!(cat_json)).clicked(&actions) {
            self.select_category(cx, 10);
        }

        // Handle counter button
        if self.ui.mp_button(ids!(counter_btn)).clicked(&actions) {
            self.counter += 1;
            self.ui.label(ids!(counter_label))
                .set_text(cx, &format!("Clicked: {} times", self.counter));
        }

        // Handle PageFlip navigation
        if self.ui.mp_button(ids!(page_btn_a)).clicked(&actions) {
            self.ui.page_flip(ids!(demo_page_flip)).set_active_page(cx, id!(page_a));
            self.current_page = 0;
            self.update_page_buttons(cx);
        }
        if self.ui.mp_button(ids!(page_btn_b)).clicked(&actions) {
            self.ui.page_flip(ids!(demo_page_flip)).set_active_page(cx, id!(page_b));
            self.current_page = 1;
            self.update_page_buttons(cx);
        }
        if self.ui.mp_button(ids!(page_btn_c)).clicked(&actions) {
            self.ui.page_flip(ids!(demo_page_flip)).set_active_page(cx, id!(page_c));
            self.current_page = 2;
            self.update_page_buttons(cx);
        }

        // Handle checkbox changes
        if self.ui.mp_checkbox(ids!(checkbox1)).changed(&actions).is_some() {
            self.update_checkbox_status(cx);
        }
        if self.ui.mp_checkbox(ids!(checkbox2)).changed(&actions).is_some() {
            self.update_checkbox_status(cx);
        }
        if self.ui.mp_checkbox(ids!(checkbox3)).changed(&actions).is_some() {
            self.update_checkbox_status(cx);
        }

        // Handle switch changes
        if let Some(on) = self.ui.mp_switch(ids!(switch_wifi)).changed(&actions) {
            log!("Wi-Fi: {}", if on { "ON" } else { "OFF" });
        }
        if let Some(on) = self.ui.mp_switch(ids!(switch_bluetooth)).changed(&actions) {
            log!("Bluetooth: {}", if on { "ON" } else { "OFF" });
        }
        if let Some(on) = self.ui.mp_switch(ids!(switch_notifications)).changed(&actions) {
            log!("Notifications: {}", if on { "ON" } else { "OFF" });
        }

        // Handle radio changes (mutually exclusive)
        if self.ui.mp_radio(ids!(radio_small)).changed(&actions).is_some() {
            self.ui.mp_radio(ids!(radio_medium)).set_checked(cx, false);
            self.ui.mp_radio(ids!(radio_large)).set_checked(cx, false);
            self.ui.label(ids!(radio_status)).set_text(cx, "Selected: Small");
        }
        if self.ui.mp_radio(ids!(radio_medium)).changed(&actions).is_some() {
            self.ui.mp_radio(ids!(radio_small)).set_checked(cx, false);
            self.ui.mp_radio(ids!(radio_large)).set_checked(cx, false);
            self.ui.label(ids!(radio_status)).set_text(cx, "Selected: Medium");
        }
        if self.ui.mp_radio(ids!(radio_large)).changed(&actions).is_some() {
            self.ui.mp_radio(ids!(radio_small)).set_checked(cx, false);
            self.ui.mp_radio(ids!(radio_medium)).set_checked(cx, false);
            self.ui.label(ids!(radio_status)).set_text(cx, "Selected: Large");
        }

        // Handle progress buttons
        if self.ui.mp_button(ids!(progress_inc_btn)).clicked(&actions) {
            let current = self.ui.mp_progress(ids!(interactive_progress)).value();
            let new_value = (current + 10.0).min(100.0);
            self.ui.mp_progress(ids!(interactive_progress)).set_value(cx, new_value);
            self.ui.label(ids!(progress_label)).set_text(cx, &format!("{}%", new_value as i32));
        }
        if self.ui.mp_button(ids!(progress_dec_btn)).clicked(&actions) {
            let current = self.ui.mp_progress(ids!(interactive_progress)).value();
            let new_value = (current - 10.0).max(0.0);
            self.ui.mp_progress(ids!(interactive_progress)).set_value(cx, new_value);
            self.ui.label(ids!(progress_label)).set_text(cx, &format!("{}%", new_value as i32));
        }

        // Handle slider changes
        if let Some(value) = self.ui.mp_slider(ids!(slider_default)).changed(&actions) {
            let v = value.end();
            self.ui.label(ids!(slider_default_label)).set_text(cx, &format!("Value: {}", v as i32));
        }

        if let Some(value) = self.ui.mp_slider(ids!(slider_vert)).changed(&actions) {
            let v = value.end();
            self.ui.label(ids!(slider_vert_label)).set_text(cx, &format!("Vertical value: {}", v as i32));
        }

        // Handle range slider changes
        if let Some(value) = self.ui.mp_slider(ids!(slider_range)).changed(&actions) {
            let start = value.start() as i32;
            let end = value.end() as i32;
            self.ui.label(ids!(slider_range_label)).set_text(cx, &format!("Range: {} - {}", start, end));
        }

        if let Some(value) = self.ui.mp_slider(ids!(slider_range_success)).changed(&actions) {
            let start = value.start() as i32;
            let end = value.end() as i32;
            self.ui.label(ids!(slider_range_success_label)).set_text(cx, &format!("Range: {} - {} (step 5)", start, end));
        }

        // Handle shader art speed slider
        if let Some(value) = self.ui.mp_slider(ids!(shader_art_speed)).changed(&actions) {
            let speed = value.end();
            self.ui.label(ids!(shader_art_speed_label)).set_text(cx, &format!("{:.1}x", speed));
            self.ui.view(ids!(shader_art_canvas)).apply_over(cx, live!{
                speed: (speed)
            });
        }

        // Handle shader art2 speed slider
        if let Some(value) = self.ui.mp_slider(ids!(shader_art2_speed)).changed(&actions) {
            let speed = value.end();
            self.ui.label(ids!(shader_art2_speed_label)).set_text(cx, &format!("{:.1}x", speed));
            self.ui.view(ids!(shader_art2_canvas)).apply_over(cx, live!{
                speed: (speed)
            });
        }

        // Handle shader math speed slider
        if let Some(value) = self.ui.mp_slider(ids!(shader_math_speed)).changed(&actions) {
            let speed = value.end();
            self.ui.label(ids!(shader_math_speed_label)).set_text(cx, &format!("{:.1}x", speed));
            self.ui.view(ids!(shader_math_canvas)).apply_over(cx, live!{
                speed: (speed)
            });
        }

        // Handle input changes
        if let Some(text) = self.ui.text_input(ids!(input_interactive)).changed(&actions) {
            let display = if text.is_empty() {
                "Value: (empty)".to_string()
            } else {
                format!("Value: {}", text)
            };
            self.ui.label(ids!(input_status)).set_text(cx, &display);
        }

        // Handle badge buttons
        if self.ui.mp_button(ids!(badge_inc_btn)).clicked(&actions) {
            let current = self.ui.mp_badge(ids!(interactive_badge)).count();
            let new_count = current + 1;
            self.ui.mp_badge(ids!(interactive_badge)).set_count(cx, new_count);
            self.ui.label(ids!(badge_count_label)).set_text(cx, &format!("Count: {}", new_count));
        }
        if self.ui.mp_button(ids!(badge_dec_btn)).clicked(&actions) {
            let current = self.ui.mp_badge(ids!(interactive_badge)).count();
            let new_count = (current - 1).max(0);
            self.ui.mp_badge(ids!(interactive_badge)).set_count(cx, new_count);
            self.ui.label(ids!(badge_count_label)).set_text(cx, &format!("Count: {}", new_count));
        }

        // Handle avatar change button
        if self.ui.mp_button(ids!(avatar_change_btn)).clicked(&actions) {
            let names = ["Alice Wang", "Bob Smith", "Carol Lee", "David Kim", "Emma Chen", "Frank Zhang"];
            let idx = (cx.event_id() as usize) % names.len();
            let name = names[idx];
            self.ui.mp_avatar(ids!(dynamic_avatar)).set_initials_from_name(cx, name);
            self.ui.label(ids!(avatar_name_label)).set_text(cx, name);
        }

        // Handle clickable card clicks using as_widget_action().cast() pattern
        for action in actions {
            if let MpCardAction::Clicked = action.as_widget_action().cast() {
                self.ui.label(ids!(card_click_status)).set_text(cx, "Card clicked!");
            }
            // Handle modal close request (backdrop or X button)
            if let MpModalAction::CloseRequested = action.as_widget_action().cast() {
                self.ui.mp_modal_widget(ids!(demo_modal)).close(cx);
                self.ui.label(ids!(modal_status)).set_text(cx, "Modal closed");
            }
        }

        // Handle open modal button
        if self.ui.mp_button(ids!(open_modal_btn)).clicked(&actions) {
            self.ui.mp_modal_widget(ids!(demo_modal)).open(cx);
            self.ui.label(ids!(modal_status)).set_text(cx, "Modal opened");
        }

        // Handle modal cancel button
        if self.ui.mp_button(ids!(modal_cancel_btn)).clicked(&actions) {
            self.ui.mp_modal_widget(ids!(demo_modal)).close(cx);
            self.ui.label(ids!(modal_status)).set_text(cx, "Cancelled");
        }

        // Handle modal confirm button
        if self.ui.mp_button(ids!(modal_confirm_btn)).clicked(&actions) {
            self.ui.mp_modal_widget(ids!(demo_modal)).close(cx);
            self.ui.label(ids!(modal_status)).set_text(cx, "Confirmed!");
        }

        // Handle popover toggle button
        if self.ui.mp_button(ids!(popover_trigger_btn)).clicked(&actions) {
            self.ui.mp_popover_widget(ids!(interactive_popover)).toggle(cx);
        }

        // Handle skeleton toggle button
        if self.ui.mp_button(ids!(skeleton_toggle_btn)).clicked(&actions) {
            let skeleton = self.ui.mp_skeleton_widget(ids!(interactive_skeleton));
            let is_loading = skeleton.is_loading();
            skeleton.set_loading(cx, !is_loading);
            let status = if !is_loading { "Loading" } else { "Loaded" };
            self.ui.label(ids!(skeleton_status)).set_text(cx, &format!("Status: {}", status));
        }

        // Handle notification buttons
        if self.ui.mp_button(ids!(show_success_notif)).clicked(&actions) {
            self.ui.mp_notification_widget(ids!(demo_notification)).show_message(
                cx, "Success!", "Operation completed successfully!"
            );
        }
        if self.ui.mp_button(ids!(show_error_notif)).clicked(&actions) {
            self.ui.mp_notification_widget(ids!(demo_notification)).show_message(
                cx, "Error", "Something went wrong. Please try again."
            );
        }
        if self.ui.mp_button(ids!(show_warning_notif)).clicked(&actions) {
            self.ui.mp_notification_widget(ids!(demo_notification)).show_message(
                cx, "Warning", "Please review your input before continuing."
            );
        }
        if self.ui.mp_button(ids!(show_info_notif)).clicked(&actions) {
            self.ui.mp_notification_widget(ids!(demo_notification)).show_message(
                cx, "Info", "Here's some helpful information for you."
            );
        }

        // Handle dropdown changes
        let labels = ["Apple", "Banana", "Cherry", "Date", "Elderberry"];
        if let Some(idx) = self.ui.drop_down(ids!(dropdown_basic)).selected(&actions) {
            let label = labels.get(idx).unwrap_or(&"Unknown");
            self.ui.label(ids!(dropdown_status)).set_text(cx, &format!("Selected: {}", label));
        }

        // Handle Tab clicks - Default style
        if self.ui.mp_tab(ids!(tab_home)).clicked(&actions) {
            self.select_tab(cx, "default", 0, "Home");
        }
        if self.ui.mp_tab(ids!(tab_profile)).clicked(&actions) {
            self.select_tab(cx, "default", 1, "Profile");
        }
        if self.ui.mp_tab(ids!(tab_settings)).clicked(&actions) {
            self.select_tab(cx, "default", 2, "Settings");
        }

        // Handle Tab clicks - Underline style
        if self.ui.mp_tab(ids!(tab_u_overview)).clicked(&actions) {
            self.select_tab(cx, "underline", 0, "Overview");
        }
        if self.ui.mp_tab(ids!(tab_u_analytics)).clicked(&actions) {
            self.select_tab(cx, "underline", 1, "Analytics");
        }
        if self.ui.mp_tab(ids!(tab_u_reports)).clicked(&actions) {
            self.select_tab(cx, "underline", 2, "Reports");
        }

        // Handle Tab clicks - Pill style
        if self.ui.mp_tab(ids!(tab_p_all)).clicked(&actions) {
            self.select_tab(cx, "pill", 0, "All");
        }
        if self.ui.mp_tab(ids!(tab_p_active)).clicked(&actions) {
            self.select_tab(cx, "pill", 1, "Active");
        }
        if self.ui.mp_tab(ids!(tab_p_completed)).clicked(&actions) {
            self.select_tab(cx, "pill", 2, "Completed");
        }

        // Handle Tab clicks - Outline style
        if self.ui.mp_tab(ids!(tab_o_day)).clicked(&actions) {
            self.select_tab(cx, "outline", 0, "Day");
        }
        if self.ui.mp_tab(ids!(tab_o_week)).clicked(&actions) {
            self.select_tab(cx, "outline", 1, "Week");
        }
        if self.ui.mp_tab(ids!(tab_o_month)).clicked(&actions) {
            self.select_tab(cx, "outline", 2, "Month");
        }

        // Handle Tab clicks - Segmented style
        if self.ui.mp_tab(ids!(tab_s_list)).clicked(&actions) {
            self.select_tab(cx, "segmented", 0, "List");
        }
        if self.ui.mp_tab(ids!(tab_s_grid)).clicked(&actions) {
            self.select_tab(cx, "segmented", 1, "Grid");
        }
        if self.ui.mp_tab(ids!(tab_s_map)).clicked(&actions) {
            self.select_tab(cx, "segmented", 2, "Map");
        }
    }
}

impl App {
    fn select_category(&mut self, cx: &mut Cx, index: usize) {
        self.current_category = index;

        // Update tab selected states
        self.ui.mp_tab(ids!(cat_form)).set_selected(cx, index == 0);
        self.ui.mp_tab(ids!(cat_display)).set_selected(cx, index == 1);
        self.ui.mp_tab(ids!(cat_nav)).set_selected(cx, index == 2);
        self.ui.mp_tab(ids!(cat_feedback)).set_selected(cx, index == 3);
        self.ui.mp_tab(ids!(cat_data)).set_selected(cx, index == 4);
        self.ui.mp_tab(ids!(cat_shader)).set_selected(cx, index == 5);
        self.ui.mp_tab(ids!(cat_shader_art)).set_selected(cx, index == 6);
        self.ui.mp_tab(ids!(cat_shader_art2)).set_selected(cx, index == 7);
        self.ui.mp_tab(ids!(cat_shader_math)).set_selected(cx, index == 8);
        self.ui.mp_tab(ids!(cat_splash)).set_selected(cx, index == 9);
        self.ui.mp_tab(ids!(cat_json)).set_selected(cx, index == 10);

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
            _ => id!(page_form),
        };
        self.ui.page_flip(ids!(category_pages)).set_active_page(cx, page_id);
        self.ui.redraw(cx);
    }

    fn update_checkbox_status(&mut self, cx: &mut Cx) {
        let mut selected = Vec::new();

        if self.ui.mp_checkbox(ids!(checkbox1)).is_checked() {
            selected.push("Option 1");
        }
        if self.ui.mp_checkbox(ids!(checkbox2)).is_checked() {
            selected.push("Option 2");
        }
        if self.ui.mp_checkbox(ids!(checkbox3)).is_checked() {
            selected.push("Option 3");
        }

        let status = if selected.is_empty() {
            "Selected: None".to_string()
        } else {
            format!("Selected: {}", selected.join(", "))
        };

        self.ui.label(ids!(checkbox_status)).set_text(cx, &status);
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
            let btn = self.ui.widget(btn_id);
            if page_idx == self.current_page {
                btn.apply_over(cx, live! {
                    draw_bg: { color: (active_bg), color_hover: (active_hover), color_pressed: (active_pressed) }
                    draw_text: { color: (active_text) }
                });
            } else {
                btn.apply_over(cx, live! {
                    draw_bg: { color: (inactive_bg), color_hover: (inactive_hover), color_pressed: (inactive_pressed) }
                    draw_text: { color: (inactive_text) }
                });
            }
        }

        self.ui.redraw(cx);
    }

    fn select_tab(&mut self, cx: &mut Cx, style: &str, index: usize, label: &str) {
        match style {
            "default" => {
                self.ui.mp_tab(ids!(tab_home)).set_selected(cx, index == 0);
                self.ui.mp_tab(ids!(tab_profile)).set_selected(cx, index == 1);
                self.ui.mp_tab(ids!(tab_settings)).set_selected(cx, index == 2);
            }
            "underline" => {
                self.ui.mp_tab(ids!(tab_u_overview)).set_selected(cx, index == 0);
                self.ui.mp_tab(ids!(tab_u_analytics)).set_selected(cx, index == 1);
                self.ui.mp_tab(ids!(tab_u_reports)).set_selected(cx, index == 2);
            }
            "pill" => {
                self.ui.mp_tab(ids!(tab_p_all)).set_selected(cx, index == 0);
                self.ui.mp_tab(ids!(tab_p_active)).set_selected(cx, index == 1);
                self.ui.mp_tab(ids!(tab_p_completed)).set_selected(cx, index == 2);
            }
            "outline" => {
                self.ui.mp_tab(ids!(tab_o_day)).set_selected(cx, index == 0);
                self.ui.mp_tab(ids!(tab_o_week)).set_selected(cx, index == 1);
                self.ui.mp_tab(ids!(tab_o_month)).set_selected(cx, index == 2);
            }
            "segmented" => {
                self.ui.mp_tab(ids!(tab_s_list)).set_selected(cx, index == 0);
                self.ui.mp_tab(ids!(tab_s_grid)).set_selected(cx, index == 1);
                self.ui.mp_tab(ids!(tab_s_map)).set_selected(cx, index == 2);
            }
            _ => {}
        }

        self.ui.label(ids!(tab_status)).set_text(cx, &format!("Selected: {}", label));
        self.ui.redraw(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
