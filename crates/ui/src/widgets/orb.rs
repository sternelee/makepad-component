use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpOrb - bezel-style "thinking" loaders, one shader, four modes:
    // 0 = Cluster (3 breathing blobs), 1 = Ring (8-dot chase),
    // 2 = Converge (dots gather to a point), 3 = Bloom (rings out)
    // ============================================================

    mod.widgets.MpOrbBase = #(MpOrb::register_widget(vm))

    mod.widgets.MpOrb = set_type_default() do mod.widgets.MpOrbBase{
        width: 48.0
        height: 48.0

        show_bg: true

        draw_bg +: {
            phase: instance(0.0)
            mode: instance(0.0)
            orb_color: instance(ACCENT)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = min(self.rect_size.x, self.rect_size.y)
                let c = self.rect_size * 0.5
                let col = self.orb_color
                let TAU = 6.2831853
                let m = self.mode

                if (m < 0.5) {
                    // Cluster: three orbs, a third of a period apart,
                    // swinging between 14% and 62% of the box while drifting
                    // on small circles around their seats.
                    let ph0 = fract(self.phase + 0.0)
                    let w0 = 0.5 - 0.5 * cos(ph0 * TAU)
                    let p0 = c + (vec2(0.36, 0.36) - 0.5 + vec2(0.13 * cos(ph0 * TAU), 0.13 * sin(ph0 * TAU))) * sz
                    sdf.circle(p0.x, p0.y, (0.14 + 0.48 * w0) * sz * 0.5)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * w0))

                    let ph1 = fract(self.phase + 0.3333333)
                    let w1 = 0.5 - 0.5 * cos(ph1 * TAU)
                    let p1 = c + (vec2(0.64, 0.32) - 0.5 + vec2(0.13 * cos(ph1 * TAU), 0.13 * sin(ph1 * TAU))) * sz
                    sdf.circle(p1.x, p1.y, (0.14 + 0.48 * w1) * sz * 0.5)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * w1))

                    let ph2 = fract(self.phase + 0.6666667)
                    let w2 = 0.5 - 0.5 * cos(ph2 * TAU)
                    let p2 = c + (vec2(0.48, 0.66) - 0.5 + vec2(0.13 * cos(ph2 * TAU), 0.13 * sin(ph2 * TAU))) * sz
                    sdf.circle(p2.x, p2.y, (0.14 + 0.48 * w2) * sz * 0.5)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * w2))
                } else if (m < 1.5) {
                    // Ring: eight dots on a circle, brightness chasing round.
                    // Twelve o'clock first, going clockwise.
                    let r_dot = 0.16 * sz * 0.5
                    let ring_r = 0.34 * sz

                    let pha = fract(self.phase + 0.0)
                    let wa = 0.5 - 0.5 * cos(pha * TAU)
                    let pa = c + vec2(cos(-1.5707963), sin(-1.5707963)) * ring_r
                    sdf.circle(pa.x, pa.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * wa))

                    let phb = fract(self.phase + 0.125)
                    let wb = 0.5 - 0.5 * cos(phb * TAU)
                    let pb = c + vec2(cos(-0.7853982), sin(-0.7853982)) * ring_r
                    sdf.circle(pb.x, pb.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * wb))

                    let phc = fract(self.phase + 0.25)
                    let wc = 0.5 - 0.5 * cos(phc * TAU)
                    let pc = c + vec2(cos(0.0), sin(0.0)) * ring_r
                    sdf.circle(pc.x, pc.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * wc))

                    let phd = fract(self.phase + 0.375)
                    let wd = 0.5 - 0.5 * cos(phd * TAU)
                    let pd = c + vec2(cos(0.7853982), sin(0.7853982)) * ring_r
                    sdf.circle(pd.x, pd.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * wd))

                    let phe = fract(self.phase + 0.5)
                    let we = 0.5 - 0.5 * cos(phe * TAU)
                    let pe = c + vec2(cos(1.5707963), sin(1.5707963)) * ring_r
                    sdf.circle(pe.x, pe.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * we))

                    let phf = fract(self.phase + 0.625)
                    let wf = 0.5 - 0.5 * cos(phf * TAU)
                    let pf = c + vec2(cos(2.3561945), sin(2.3561945)) * ring_r
                    sdf.circle(pf.x, pf.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * wf))

                    let phg = fract(self.phase + 0.75)
                    let wg = 0.5 - 0.5 * cos(phg * TAU)
                    let pg = c + vec2(cos(3.1415927), sin(3.1415927)) * ring_r
                    sdf.circle(pg.x, pg.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * wg))

                    let phh = fract(self.phase + 0.875)
                    let wh = 0.5 - 0.5 * cos(phh * TAU)
                    let ph_pt = c + vec2(cos(3.9269908), sin(3.9269908)) * ring_r
                    sdf.circle(ph_pt.x, ph_pt.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.35 + 0.65 * wh))
                } else if (m < 2.5) {
                    // Converge: every dot shares one radius, so they gather
                    // into a single point and open back out to the ring.
                    let wave = 0.5 - 0.5 * cos(fract(self.phase) * TAU)
                    let rad = 0.34 * wave * sz
                    let r_dot = 0.16 * sz * 0.5

                    let qa = c + vec2(cos(-1.5707963), sin(-1.5707963)) * rad
                    sdf.circle(qa.x, qa.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                    let qb = c + vec2(cos(-0.7853982), sin(-0.7853982)) * rad
                    sdf.circle(qb.x, qb.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                    let qc = c + vec2(cos(0.0), sin(0.0)) * rad
                    sdf.circle(qc.x, qc.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                    let qd = c + vec2(cos(0.7853982), sin(0.7853982)) * rad
                    sdf.circle(qd.x, qd.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                    let qe = c + vec2(cos(1.5707963), sin(1.5707963)) * rad
                    sdf.circle(qe.x, qe.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                    let qf = c + vec2(cos(2.3561945), sin(2.3561945)) * rad
                    sdf.circle(qf.x, qf.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                    let qg = c + vec2(cos(3.1415927), sin(3.1415927)) * rad
                    sdf.circle(qg.x, qg.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                    let qh = c + vec2(cos(3.9269908), sin(3.9269908)) * rad
                    sdf.circle(qh.x, qh.y, r_dot)
                    sdf.fill(vec4(col.xyz, 0.9))
                } else {
                    // Bloom: rings leaving the centre and fading before the edge.
                    let bh0 = fract(self.phase + 0.0)
                    let bw0 = 0.5 - 0.5 * cos(bh0 * TAU)
                    sdf.circle(c.x, c.y, (0.16 + 0.84 * bw0) * sz * 0.5)
                    sdf.stroke(vec4(col.xyz, 1.0 - bw0), max(sz * 0.05, 1.0))

                    let bh1 = fract(self.phase + 0.3333333)
                    let bw1 = 0.5 - 0.5 * cos(bh1 * TAU)
                    sdf.circle(c.x, c.y, (0.16 + 0.84 * bw1) * sz * 0.5)
                    sdf.stroke(vec4(col.xyz, 1.0 - bw1), max(sz * 0.05, 1.0))

                    let bh2 = fract(self.phase + 0.6666667)
                    let bw2 = 0.5 - 0.5 * cos(bh2 * TAU)
                    sdf.circle(c.x, c.y, (0.16 + 0.84 * bw2) * sz * 0.5)
                    sdf.stroke(vec4(col.xyz, 1.0 - bw2), max(sz * 0.05, 1.0))
                }

                return sdf.result
            }
        }

        animator: Animator{
            spin: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {phase: 0.0}}
                }
                on: AnimatorState{
                    redraw: true
                    from: {all: Loop {duration: 2.0, end: 1.0}}
                    apply: {draw_bg: {phase: timeline(0.0, 0.0, 1.0, 1.0)}}
                }
            }
        }
    }

    mod.widgets.MpOrbCluster = mod.widgets.MpOrb{draw_bg +: {mode: instance(0.0)}}
    mod.widgets.MpOrbRing = mod.widgets.MpOrb{draw_bg +: {mode: instance(1.0)}}
    mod.widgets.MpOrbConverge = mod.widgets.MpOrb{draw_bg +: {mode: instance(2.0)}}
    mod.widgets.MpOrbBloom = mod.widgets.MpOrb{draw_bg +: {mode: instance(3.0)}}

    // ============================================================
    // MpLoadingWord - "L O A D I N G" caption under an orb
    // ============================================================

    mod.widgets.MpLoadingWord = Label{
        width: Fit
        height: Fit
        draw_text +: {
            text_style: theme.font_regular{font_size: 11.0}
            color: TEXT_FAINT
        }
        text: "L O A D I N G"
    }
}

// ============================================================
// MpOrb - animated thinking loader
// ============================================================

#[derive(Script, Widget, Animator)]
pub struct MpOrb {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
}

impl ScriptHook for MpOrb {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }

    fn on_after_reload(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }
}

impl MpOrb {
    /// Start the infinite loop unless it is already running.
    fn start_spin(&mut self, cx: &mut Cx) {
        if !self.animator.is_track_animating(live_id!(spin)) {
            self.animator_play(cx, ids!(spin.on));
        }
    }
}

impl Widget for MpOrb {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.animator_handle_event(cx, event).must_redraw();
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
