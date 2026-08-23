use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpSpinner - Base rotating loading spinner component
    // ============================================================

    mod.widgets.MpSpinnerBase = #(MpSpinner::register_widget(vm))
    mod.widgets.MpSpinner = set_type_default() do mod.widgets.MpSpinnerBase{
        width: 24.0
        height: 24.0
        show_bg: true

        draw_bg +: {
            rotation: instance(0.0)
            spinner_color: instance(#x3b82f6)
            spinner_track: instance(#xe5e7eb)
            stroke_width: instance(3.0)
            arc_ratio: instance(0.25)

            pixel: fn() {
                let center = self.rect_size * 0.5
                let radius = min(center.x, center.y) - self.stroke_width * 0.5 - 1.0
                let pos = self.pos * self.rect_size - center
                let dist = length(pos)

                // Ring mask with anti-aliasing
                let inner = radius - self.stroke_width * 0.5
                let outer = radius + self.stroke_width * 0.5
                let ring = smoothstep(inner - 0.5, inner + 0.5, dist) * smoothstep(outer + 0.5, outer - 0.5, dist)

                // Angle normalized to 0..1, arc positioned relative to rotation
                let angle = atan2(pos.y, pos.x)
                let norm_angle = modf(angle / (2.0 * PI) + 0.5, 1.0)
                let arc_pos = modf(norm_angle - self.rotation + 1.0, 1.0)
                let in_arc = step(arc_pos, self.arc_ratio)

                let color = mix(self.spinner_track, self.spinner_color, in_arc)
                return vec4(color.rgb, color.a * ring)
            }
        }

        animator: Animator{
            spin: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {rotation: 0.0}}
                }
                on: AnimatorState{
                    redraw: true
                    from: {all: Loop {duration: 0.8, end: 1.0}}
                    apply: {draw_bg: {rotation: timeline(0.0, 0.0, 1.0, 1.0)}}
                }
            }
        }
    }

    // ============================================================
    // Size variants
    // ============================================================

    mod.widgets.MpSpinnerXs = mod.widgets.MpSpinner{width: 16.0, height: 16.0, draw_bg +: {stroke_width: instance(2.0)}}
    mod.widgets.MpSpinnerSm = mod.widgets.MpSpinner{width: 20.0, height: 20.0, draw_bg +: {stroke_width: instance(2.5)}}
    mod.widgets.MpSpinnerMd = mod.widgets.MpSpinner{width: 24.0, height: 24.0, draw_bg +: {stroke_width: instance(3.0)}}
    mod.widgets.MpSpinnerLg = mod.widgets.MpSpinner{width: 32.0, height: 32.0, draw_bg +: {stroke_width: instance(4.0)}}
    mod.widgets.MpSpinnerXl = mod.widgets.MpSpinner{width: 48.0, height: 48.0, draw_bg +: {stroke_width: instance(5.0)}}

    // ============================================================
    // Color variants
    // ============================================================

    mod.widgets.MpSpinnerPrimary = mod.widgets.MpSpinner{draw_bg +: {spinner_color: instance(ACCENT)}}
    mod.widgets.MpSpinnerSuccess = mod.widgets.MpSpinner{draw_bg +: {spinner_color: instance(#x22c55e), spinner_track: instance(#xdcfce7)}}
    mod.widgets.MpSpinnerWarning = mod.widgets.MpSpinner{draw_bg +: {spinner_color: instance(#xf59e0b), spinner_track: instance(#xfef3c7)}}
    mod.widgets.MpSpinnerDanger = mod.widgets.MpSpinner{draw_bg +: {spinner_color: instance(#xdc2626), spinner_track: instance(#xfee2e2)}}

    // ============================================================
    // Style variants
    // ============================================================

    mod.widgets.MpSpinnerNoTrack = mod.widgets.MpSpinner{draw_bg +: {spinner_track: instance(#x00000000)}}
    mod.widgets.MpSpinnerThin = mod.widgets.MpSpinner{draw_bg +: {stroke_width: instance(2.0)}}
    mod.widgets.MpSpinnerThick = mod.widgets.MpSpinner{draw_bg +: {stroke_width: instance(5.0)}}

    // Speed variants (full animator override with a different loop duration)
    mod.widgets.MpSpinnerFast = mod.widgets.MpSpinner{
        animator: Animator{
            spin: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {rotation: 0.0}}
                }
                on: AnimatorState{
                    redraw: true
                    from: {all: Loop {duration: 0.5, end: 1.0}}
                    apply: {draw_bg: {rotation: timeline(0.0, 0.0, 1.0, 1.0)}}
                }
            }
        }
    }

    mod.widgets.MpSpinnerSlow = mod.widgets.MpSpinner{
        animator: Animator{
            spin: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {rotation: 0.0}}
                }
                on: AnimatorState{
                    redraw: true
                    from: {all: Loop {duration: 1.5, end: 1.0}}
                    apply: {draw_bg: {rotation: timeline(0.0, 0.0, 1.0, 1.0)}}
                }
            }
        }
    }

    // ============================================================
    // Spinner with label
    // ============================================================

    mod.widgets.MpSpinnerWithLabel = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Right
        spacing: 8.0
        align: Align{y: 0.5}

        spinner := mod.widgets.MpSpinner{}
        label := mod.widgets.Label{
            draw_text +: {
                color: TEXT_MUTED
                text_style: theme.font_regular{font_size: 14.0}
            }
            text: "Loading..."
        }
    }

    mod.widgets.MpSpinnerWithLabelVertical = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Down
        spacing: 8.0
        align: Align{x: 0.5}

        spinner := mod.widgets.MpSpinner{}
        label := mod.widgets.Label{
            draw_text +: {
                color: TEXT_MUTED
                text_style: theme.font_regular{font_size: 14.0}
            }
            text: "Loading..."
        }
    }

    // ============================================================
    // Dots spinner (alternative style)
    // ============================================================

    mod.widgets.MpSpinnerDotsBase = #(MpSpinnerDots::register_widget(vm))
    mod.widgets.MpSpinnerDots = set_type_default() do mod.widgets.MpSpinnerDotsBase{
        width: 40.0
        height: 12.0
        show_bg: true

        draw_bg +: {
            phase: instance(0.0)
            spinner_color: instance(#x3b82f6)

            pixel: fn() {
                let sz = self.rect_size
                let dot_r = sz.y * 0.35
                let spacing = sz.x / 3.0
                let dot1_x = spacing * 0.5
                let dot2_x = spacing * 1.5
                let dot3_x = spacing * 2.5
                let dot_y = sz.y * 0.5
                let phase1 = fract(self.phase)
                let phase2 = fract(self.phase + 0.33)
                let phase3 = fract(self.phase + 0.66)
                let scale1 = 0.5 + 0.5 * sin(phase1 * 2.0 * PI)
                let scale2 = 0.5 + 0.5 * sin(phase2 * 2.0 * PI)
                let scale3 = 0.5 + 0.5 * sin(phase3 * 2.0 * PI)
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.circle(dot1_x, dot_y, dot_r * scale1)
                sdf.fill_keep(self.spinner_color)
                sdf.circle(dot2_x, dot_y, dot_r * scale2)
                sdf.fill_keep(self.spinner_color)
                sdf.circle(dot3_x, dot_y, dot_r * scale3)
                sdf.fill(self.spinner_color)
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
                    from: {all: Loop {duration: 1.0, end: 1.0}}
                    apply: {draw_bg: {phase: timeline(0.0, 0.0, 1.0, 1.0)}}
                }
            }
        }
    }

    // ============================================================
    // Pulse spinner (alternative style)
    // ============================================================

    mod.widgets.MpSpinnerPulseBase = #(MpSpinnerPulse::register_widget(vm))
    mod.widgets.MpSpinnerPulse = set_type_default() do mod.widgets.MpSpinnerPulseBase{
        width: 24.0
        height: 24.0
        show_bg: true

        draw_bg +: {
            scale: instance(0.0)
            spinner_color: instance(#x3b82f6)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = min(c.x, c.y) - 2.0
                let actual_scale = 0.3 + 0.7 * self.scale
                let alpha = 1.0 - self.scale * 0.7
                sdf.circle(c.x, c.y, r * actual_scale)
                sdf.fill(vec4(self.spinner_color.xyz, alpha))
                return sdf.result
            }
        }

        animator: Animator{
            spin: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {scale: 0.0}}
                }
                on: AnimatorState{
                    redraw: true
                    from: {all: Loop {duration: 1.0, end: 1.0}}
                    apply: {draw_bg: {scale: timeline(0.0, 0.0, 1.0, 1.0)}}
                }
            }
        }
    }
}

#[derive(Script, Widget, Animator)]
pub struct MpSpinner {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
}

impl ScriptHook for MpSpinner {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }

    fn on_after_reload(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }
}

impl MpSpinner {
    /// Start the infinite spin loop unless it is already running.
    fn start_spin(&mut self, cx: &mut Cx) {
        if !self.animator.is_track_animating(live_id!(spin)) {
            self.animator_play(cx, ids!(spin.on));
        }
    }
}

impl Widget for MpSpinner {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Script, Widget, Animator)]
pub struct MpSpinnerDots {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
}

impl ScriptHook for MpSpinnerDots {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }

    fn on_after_reload(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }
}

impl MpSpinnerDots {
    /// Start the infinite phase loop unless it is already running.
    fn start_spin(&mut self, cx: &mut Cx) {
        if !self.animator.is_track_animating(live_id!(spin)) {
            self.animator_play(cx, ids!(spin.on));
        }
    }
}

impl Widget for MpSpinnerDots {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Script, Widget, Animator)]
pub struct MpSpinnerPulse {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
}

impl ScriptHook for MpSpinnerPulse {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }

    fn on_after_reload(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| self.start_spin(cx));
    }
}

impl MpSpinnerPulse {
    /// Start the infinite scale loop unless it is already running.
    fn start_spin(&mut self, cx: &mut Cx) {
        if !self.animator.is_track_animating(live_id!(spin)) {
            self.animator_play(cx, ids!(spin.on));
        }
    }
}

impl Widget for MpSpinnerPulse {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
