use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpSkeleton - Loading placeholder component with shimmer animation
    // The shimmer shaders read self.draw_pass.time (like LoadingSpinner);
    // MpSkeletonWidget keeps repainting itself via NextFrame while loading.
    // ============================================================

    // Rectangular skeleton (default)
    mod.widgets.MpSkeleton = View{
        width: Fill
        height: 20.0

        show_bg: true
        draw_bg +: {
            color_base: uniform(#xe5e7eb)
            color_shimmer: uniform(#xf3f4f6)
            shimmer_width: uniform(0.3)
            shimmer_speed: uniform(0.8)

            pixel: fn() {
                let shimmer_pos = fract(self.draw_pass.time * self.shimmer_speed) * (1.0 + self.shimmer_width * 2.0) - self.shimmer_width
                let dist = abs(self.pos.x - shimmer_pos)
                let shimmer = 1.0 - smoothstep(0.0, self.shimmer_width, dist)
                let result_color = mix(self.color_base, self.color_shimmer, shimmer)
                return result_color
            }
        }
    }

    // Rounded skeleton
    mod.widgets.MpSkeletonRounded = View{
        width: Fill
        height: 20.0

        show_bg: true
        draw_bg +: {
            radius: instance(4.0)

            color_base: uniform(#xe5e7eb)
            color_shimmer: uniform(#xf3f4f6)
            shimmer_speed: uniform(0.8)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                let shimmer_pos = fract(self.draw_pass.time * self.shimmer_speed) * 1.6 - 0.3
                let dist = abs(self.pos.x - shimmer_pos)
                let shimmer = 1.0 - smoothstep(0.0, 0.3, dist)
                let result_color = mix(self.color_base, self.color_shimmer, shimmer)

                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius)
                sdf.fill(result_color)

                return sdf.result
            }
        }
    }

    // Circle skeleton (for avatars)
    mod.widgets.MpSkeletonCircle = View{
        width: 40.0
        height: 40.0

        show_bg: true
        draw_bg +: {
            color_base: uniform(#xe5e7eb)
            color_shimmer: uniform(#xf3f4f6)
            shimmer_speed: uniform(0.8)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = min(c.x, c.y)

                let shimmer_pos = fract(self.draw_pass.time * self.shimmer_speed) * 1.6 - 0.3
                let dist = abs(self.pos.x - shimmer_pos)
                let shimmer = 1.0 - smoothstep(0.0, 0.3, dist)
                let result_color = mix(self.color_base, self.color_shimmer, shimmer)

                sdf.circle(c.x, c.y, r)
                sdf.fill(result_color)

                return sdf.result
            }
        }
    }

    // ============================================================
    // Size variants
    // ============================================================

    // Text line skeleton
    mod.widgets.MpSkeletonText = mod.widgets.MpSkeletonRounded{
        width: Fill
        height: 16.0
        draw_bg +: { radius: 2.0 }
    }

    // Title skeleton
    mod.widgets.MpSkeletonTitle = mod.widgets.MpSkeletonRounded{
        width: 200.0
        height: 24.0
        draw_bg +: { radius: 4.0 }
    }

    // Paragraph skeleton
    mod.widgets.MpSkeletonParagraph = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 8.0

        mod.widgets.MpSkeletonText{ width: Fill }
        mod.widgets.MpSkeletonText{ width: Fill }
        mod.widgets.MpSkeletonText{ width: 280.0 }
    }

    // Avatar skeleton sizes
    mod.widgets.MpSkeletonAvatarSmall = mod.widgets.MpSkeletonCircle{
        width: 32.0
        height: 32.0
    }

    mod.widgets.MpSkeletonAvatarLarge = mod.widgets.MpSkeletonCircle{
        width: 56.0
        height: 56.0
    }

    // ============================================================
    // Card skeleton
    // ============================================================

    mod.widgets.MpSkeletonCard = RoundedView{
        width: Fill
        height: Fit
        flow: Down
        padding: Inset{left: 16.0, right: 16.0, top: 16.0, bottom: 16.0}
        spacing: 12.0

        draw_bg +: {
            color: CARD
            border_radius: 8.0
            border_color: BORDER
        }

        View{
            width: Fill
            height: Fit
            flow: Right
            spacing: 12.0
            align: Align{y: 0.5}

            mod.widgets.MpSkeletonCircle{}

            View{
                width: Fill
                height: Fit
                flow: Down
                spacing: 8.0

                mod.widgets.MpSkeletonRounded{ width: 120.0, height: 16.0 }
                mod.widgets.MpSkeletonRounded{ width: 80.0, height: 12.0 }
            }
        }

        mod.widgets.MpSkeletonParagraph{}
    }

    // ============================================================
    // List item skeleton
    // ============================================================

    mod.widgets.MpSkeletonListItem = View{
        width: Fill
        height: Fit
        flow: Right
        padding: Inset{left: 12.0, right: 12.0, top: 12.0, bottom: 12.0}
        spacing: 12.0
        align: Align{y: 0.5}

        mod.widgets.MpSkeletonCircle{
            width: 40.0
            height: 40.0
        }

        View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 6.0

            mod.widgets.MpSkeletonRounded{ width: 150.0, height: 16.0 }
            mod.widgets.MpSkeletonRounded{ width: 100.0, height: 12.0 }
        }
    }

    // ============================================================
    // Interactive Skeleton Widget
    // ============================================================

    mod.widgets.MpSkeletonWidgetBase = #(MpSkeletonWidget::register_widget(vm))
    mod.widgets.MpSkeletonWidget = set_type_default() do mod.widgets.MpSkeletonWidgetBase{
        width: Fill
        height: Fit
        flow: Overlay

        skeleton := View{
            width: Fill
            height: Fit
            flow: Down
            spacing: 8.0
            visible: true

            mod.widgets.MpSkeletonRounded{ width: Fill, height: 20.0 }
        }

        content := View{
            width: Fill
            height: Fit
            visible: false
        }
    }
}

/// Skeleton widget actions
#[derive(Clone, Debug, Default)]
pub enum MpSkeletonAction {
    #[default]
    None,
    LoadingStarted,
    LoadingFinished,
}

/// Interactive skeleton widget with loading state control
#[derive(Script, ScriptHook, Widget)]
pub struct MpSkeletonWidget {
    #[source]
    source: ScriptObjectRef,

    #[deref]
    view: View,

    #[live]
    loading: bool,

    #[rust]
    next_frame: NextFrame,
}

impl Widget for MpSkeletonWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Keep the shimmer animation running while loading: the shaders read
        // self.draw_pass.time, which only advances when the pass repaints.
        if self.loading && self.next_frame.is_event(event).is_some() {
            self.next_frame = cx.new_next_frame();
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // (Re)arm the animation driver while loading
        if self.loading {
            self.next_frame = cx.new_next_frame();
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpSkeletonWidget {
    /// Set loading state - shows skeleton when true, content when false
    pub fn set_loading(&mut self, cx: &mut Cx, loading: bool) {
        self.loading = loading;
        self.view
            .widget(cx, ids!(skeleton))
            .set_visible(cx, loading);
        self.view
            .widget(cx, ids!(content))
            .set_visible(cx, !loading);
        if loading {
            // Kick off the animation driver
            self.next_frame = cx.new_next_frame();
        }
        self.redraw(cx);
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Start loading (show skeleton)
    pub fn start_loading(&mut self, cx: &mut Cx) {
        self.set_loading(cx, true);
    }

    /// Finish loading (hide skeleton, show content)
    pub fn finish_loading(&mut self, cx: &mut Cx) {
        self.set_loading(cx, false);
    }
}

impl MpSkeletonWidgetRef {
    pub fn set_loading(&self, cx: &mut Cx, loading: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_loading(cx, loading);
        }
    }

    pub fn is_loading(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_loading()
        } else {
            false
        }
    }

    pub fn start_loading(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.start_loading(cx);
        }
    }

    pub fn finish_loading(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.finish_loading(cx);
        }
    }
}
