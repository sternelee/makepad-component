use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Badge indicator constants
    let BADGE_FONT_SIZE = 10.0
    let BADGE_PADDING_H = 7.0
    let BADGE_PADDING_V = 2.0

    let DOT_SIZE = 8.0

    // Badge indicator (count badge)
    // Uses SDF capsule: two circles + middle rect for perfect rounded ends
    mod.widgets.MpBadgeIndicator = mod.widgets.View{
        width: Fit
        height: Fit
        padding: Inset{left: BADGE_PADDING_H, right: BADGE_PADDING_H, top: BADGE_PADDING_V, bottom: BADGE_PADDING_V}
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            bg_color: instance(DANGER)

            // True capsule: two semicircles + middle rectangle
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let r = self.rect_size.y * 0.5
                let w = self.rect_size.x
                // Left semicircle
                sdf.circle(r, r, r)
                // Middle rectangle
                sdf.rect(r, 0.0, w - 2.0 * r, 2.0 * r)
                // Right semicircle
                sdf.circle(w - r, r, r)
                sdf.fill(self.bg_color)
                return sdf.result
            }
        }

        label := Label{
            width: Fit
            height: Fit
            // Override makepad Label's default `padding: theme.mspace_1` (3px
            // all sides). The indicator is Fit-height (pill wraps the text), so
            // any extra label padding would offset the number from the capsule's
            // vertical center.
            padding: 0

            draw_text +: {
                text_style: theme.font_bold{font_size: BADGE_FONT_SIZE}
                color: ON_SOLID
            }
            text: ""
        }
    }

    // Dot indicator (small circle) - also uses SDF for consistency
    mod.widgets.MpBadgeDotIndicator = mod.widgets.View{
        width: DOT_SIZE
        height: DOT_SIZE

        show_bg: true
        draw_bg +: {
            bg_color: instance(DANGER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let r = min(self.rect_size.x, self.rect_size.y) * 0.5
                sdf.circle(r, r, r)
                sdf.fill(self.bg_color)
                return sdf.result
            }
        }
    }

    // Base badge wrapper
    // - Only content padding for badge space (no container padding)
    // - badge_wrapper Fill + align for positioning
    // - badge_offset for fine-tuning
    mod.widgets.MpBadgeBase = #(MpBadge::register_widget(vm))

    // Default badge (red)
    mod.widgets.MpBadge = set_type_default() do mod.widgets.MpBadgeBase{
        width: Fit
        height: Fit

        flow: Overlay

        // Content with padding to reserve space for badge
        content := View{
            width: Fit
            height: Fit
            padding: Inset{top: 9.0, right: 9.0}
        }

        // Badge wrapper: Fill enables align to work
        badge_wrapper := View{
            width: Fill
            height: Fill
            align: Align{x: 1.0, y: 0.0}

            indicator := mod.widgets.MpBadgeIndicator{}
        }
    }

    // Success badge (green)
    mod.widgets.MpBadgeSuccess = mod.widgets.MpBadge{
        badge_wrapper +: {
            indicator +: {
                draw_bg +: { bg_color: instance(SUCCESS) }
            }
        }
    }

    // Warning badge (yellow/orange)
    mod.widgets.MpBadgeWarning = mod.widgets.MpBadge{
        badge_wrapper +: {
            indicator +: {
                draw_bg +: { bg_color: instance(WARNING) }
            }
        }
    }

    // Info badge (blue)
    mod.widgets.MpBadgeInfo = mod.widgets.MpBadge{
        badge_wrapper +: {
            indicator +: {
                draw_bg +: { bg_color: instance(INFO) }
            }
        }
    }

    // Secondary badge (gray)
    mod.widgets.MpBadgeSecondary = mod.widgets.MpBadge{
        badge_wrapper +: {
            indicator +: {
                draw_bg +: { bg_color: instance(TEXT_MUTED) }
            }
        }
    }

    // Dot badge (small indicator) - shares same layout logic
    // Derives from MpBadgeBase (like the old <MpBadgeBase>) so the count
    // indicator child is never instantiated in dot mode.
    mod.widgets.MpBadgeDot = mod.widgets.MpBadgeBase{
        width: Fit
        height: Fit

        flow: Overlay
        dot_mode: true

        content := View{
            width: Fit
            height: Fit
            padding: Inset{top: 4.0, right: 4.0}
        }

        badge_wrapper := View{
            width: Fill
            height: Fill
            align: Align{x: 1.0, y: 0.0}

            indicator := mod.widgets.MpBadgeDotIndicator{}
        }
    }

    // Dot badge variants
    mod.widgets.MpBadgeDotSuccess = mod.widgets.MpBadgeDot{
        badge_wrapper +: {
            indicator +: {
                draw_bg +: { bg_color: instance(SUCCESS) }
            }
        }
    }

    mod.widgets.MpBadgeDotWarning = mod.widgets.MpBadgeDot{
        badge_wrapper +: {
            indicator +: {
                draw_bg +: { bg_color: instance(WARNING) }
            }
        }
    }

    // Standalone badge (inline, not positioned)
    mod.widgets.MpBadgeStandalone = mod.widgets.MpBadgeIndicator{}

    mod.widgets.MpBadgeStandaloneSuccess = mod.widgets.MpBadgeIndicator{
        draw_bg +: { bg_color: instance(SUCCESS) }
    }

    mod.widgets.MpBadgeStandaloneWarning = mod.widgets.MpBadgeIndicator{
        draw_bg +: { bg_color: instance(WARNING) }
    }

    mod.widgets.MpBadgeStandaloneInfo = mod.widgets.MpBadgeIndicator{
        draw_bg +: { bg_color: instance(INFO) }
    }
}

/// Badge widget for displaying counts, dots, or status indicators
#[derive(Script, Widget)]
pub struct MpBadge {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live(99.0)]
    max_count: f64,
    #[live(0.0)]
    count: f64,
    #[live(false)]
    show_zero: bool,
    #[live(false)]
    dot_mode: bool,

    /// Offset for fine-tuning badge position (shared by Dot and Count)
    #[live]
    badge_offset: DVec2,

    /// Five-step size driving the indicator font, capsule padding and dot
    /// diameter. Badges are tiny indicators: the font is scaled down from
    /// the control metrics (Medium = the original 10px look).
    #[live]
    size: MpSize,

    /// Track if display needs update
    #[rust]
    display_dirty: bool,

    /// Last size applied to the indicator (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl ScriptHook for MpBadge {
    fn on_after_apply(
        &mut self,
        _vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        // Mark display as dirty after any script apply. The offset and text are
        // synced lazily in draw_walk, where the children are guaranteed to exist
        // (they may not be reachable yet while the apply is still running).
        self.display_dirty = true;
    }
}

impl Widget for MpBadge {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Only update display if dirty (avoid layout changes during render)
        if self.display_dirty {
            self.apply_badge_offset(cx);
            self.sync_badge_display(cx);
            self.display_dirty = false;
        }

        // Metrics from the size system (Medium = the original 10px look).
        let font = (self.size.font_size() - 3.0) as f64;
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            let indicator = self.view.view(cx, ids!(badge_wrapper.indicator));
            if let Some(mut inner) = indicator.borrow_mut() {
                inner.layout.padding = Inset {
                    left: font * 0.7,
                    right: font * 0.7,
                    top: font * 0.2,
                    bottom: font * 0.2,
                };
                if self.dot_mode {
                    // Dot badge: scale the circle diameter instead.
                    inner.walk.width = Size::Fixed(font * 0.8);
                    inner.walk.height = Size::Fixed(font * 0.8);
                }
            }
            if let Some(mut label) = self
                .view
                .label(cx, ids!(badge_wrapper.indicator.label))
                .borrow_mut()
            {
                label.draw_text.text_style.font_size = font as f32;
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpBadge {
    /// Apply badge_offset to indicator margin
    fn apply_badge_offset(&mut self, cx: &mut Cx) {
        let top = self.badge_offset.y;
        let right = -self.badge_offset.x;
        let mut indicator = self.view(cx, ids!(badge_wrapper.indicator));
        script_apply_eval!(cx, indicator, {
            margin: mod.turtle.Inset{top: #(top), right: #(right)}
        });
    }

    /// Sync badge visibility and text (called when dirty)
    fn sync_badge_display(&mut self, cx: &mut Cx) {
        // Determine visibility
        let visible = if self.dot_mode {
            true
        } else if self.count == 0.0 {
            self.show_zero
        } else {
            true
        };

        // Update badge wrapper visibility
        self.view(cx, ids!(badge_wrapper)).set_visible(cx, visible);

        // Update badge text (for non-dot mode)
        if !self.dot_mode && visible {
            let text = if self.count > self.max_count {
                format!("{}+", self.max_count)
            } else {
                self.count.to_string()
            };
            self.label(cx, ids!(badge_wrapper.indicator.label))
                .set_text(cx, &text);
        }
    }

    /// Set the count value
    pub fn set_count(&mut self, cx: &mut Cx, count: i64) {
        let count = count as f64;
        if self.count != count {
            self.count = count;
            self.display_dirty = true;
            self.redraw(cx);
        }
    }

    /// Get the current count
    pub fn count(&self) -> i64 {
        self.count as i64
    }

    /// Set whether to show the badge when count is zero
    pub fn set_show_zero(&mut self, cx: &mut Cx, show: bool) {
        if self.show_zero != show {
            self.show_zero = show;
            self.display_dirty = true;
            self.redraw(cx);
        }
    }

    /// Set dot mode
    pub fn set_dot_mode(&mut self, cx: &mut Cx, dot: bool) {
        if self.dot_mode != dot {
            self.dot_mode = dot;
            self.display_dirty = true;
            self.redraw(cx);
        }
    }

    /// Set badge offset for fine-tuning position
    pub fn set_badge_offset(&mut self, cx: &mut Cx, offset: DVec2) {
        self.badge_offset = offset;
        self.display_dirty = true;
        self.redraw(cx);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Indicator re-syncs on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpBadgeRef {
    /// Set the count value
    pub fn set_count(&self, cx: &mut Cx, count: i64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_count(cx, count);
        }
    }

    /// Get the current count
    pub fn count(&self) -> i64 {
        if let Some(inner) = self.borrow() {
            inner.count()
        } else {
            0
        }
    }

    /// Set whether to show the badge when count is zero
    pub fn set_show_zero(&self, cx: &mut Cx, show: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_show_zero(cx, show);
        }
    }

    /// Set dot mode
    pub fn set_dot_mode(&self, cx: &mut Cx, dot: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_dot_mode(cx, dot);
        }
    }

    /// Set badge offset for fine-tuning position
    pub fn set_badge_offset(&self, cx: &mut Cx, offset: DVec2) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_badge_offset(cx, offset);
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
