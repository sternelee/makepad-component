use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // Register MpPopoverTrigger enum for script VM
    mod.widgets.MpPopoverTrigger = set_type_default() do #(MpPopoverTrigger::script_api(vm))

    // ============================================================
    // MpPopover - Popover/Dropdown panel component
    // ============================================================

    // Base popover container
    mod.widgets.MpPopoverBase = View{
        width: Fit
        height: Fit
        padding: 8

        show_bg: true
        draw_bg +: {
            bg_color: instance(CARD)
            border_radius: instance(8.0)
            border_color: instance(BORDER)
            shadow_color: instance(#x0000001A)
            shadow_offset_y: instance(4.0)
            shadow_blur: instance(12.0)
            opacity: instance(1.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Shadow
                sdf.box(
                    0.0,
                    self.shadow_offset_y,
                    self.rect_size.x,
                    self.rect_size.y,
                    self.border_radius
                )
                sdf.blur = self.shadow_blur
                sdf.fill(self.shadow_color)
                sdf.blur = 0.0

                // Main card
                sdf.box(
                    0.5,
                    0.5,
                    self.rect_size.x - 1.0,
                    self.rect_size.y - 1.0,
                    self.border_radius
                )
                sdf.fill_keep(self.bg_color)
                sdf.stroke(self.border_color, 1.0)

                return Pal.premul(vec4(sdf.result.rgb, sdf.result.a * self.opacity))
            }
        }
    }

    // ============================================================
    // Default Popover
    // ============================================================

    mod.widgets.MpPopover = mod.widgets.MpPopoverBase{
        width: 240
        height: Fit
        padding: 12
        flow: Down
        spacing: 8
    }

    // Popover with arrow pointing up
    mod.widgets.MpPopoverArrowUp = View{
        width: Fit
        height: Fit
        flow: Down
        align: Align{x: 0.5}

        arrow := View{
            width: 16
            height: 8
            margin: Inset{bottom: -1}

            show_bg: true
            draw_bg +: {
                arrow_color: instance(CARD)
                arrow_border_color: instance(BORDER)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    let w = self.rect_size.x
                    let h = self.rect_size.y

                    // Triangle pointing up
                    sdf.move_to(w * 0.5, 0.0)
                    sdf.line_to(w, h)
                    sdf.line_to(0.0, h)
                    sdf.close_path()
                    sdf.fill_keep(self.arrow_color)
                    sdf.stroke(self.arrow_border_color, 1.0)

                    return sdf.result
                }
            }
        }

        content := mod.widgets.MpPopoverBase{
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }
    }

    // Popover with arrow pointing down
    mod.widgets.MpPopoverArrowDown = View{
        width: Fit
        height: Fit
        flow: Down
        align: Align{x: 0.5}

        content := mod.widgets.MpPopoverBase{
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }

        arrow := View{
            width: 16
            height: 8
            margin: Inset{top: -1}

            show_bg: true
            draw_bg +: {
                arrow_color: instance(CARD)
                arrow_border_color: instance(BORDER)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    let w = self.rect_size.x
                    let h = self.rect_size.y

                    // Triangle pointing down
                    sdf.move_to(0.0, 0.0)
                    sdf.line_to(w, 0.0)
                    sdf.line_to(w * 0.5, h)
                    sdf.close_path()
                    sdf.fill_keep(self.arrow_color)
                    sdf.stroke(self.arrow_border_color, 1.0)

                    return sdf.result
                }
            }
        }
    }

    // Popover with arrow pointing left
    mod.widgets.MpPopoverArrowLeft = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}

        arrow := View{
            width: 8
            height: 16
            margin: Inset{right: -1}

            show_bg: true
            draw_bg +: {
                arrow_color: instance(CARD)
                arrow_border_color: instance(BORDER)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    let w = self.rect_size.x
                    let h = self.rect_size.y

                    // Triangle pointing left
                    sdf.move_to(w, 0.0)
                    sdf.line_to(w, h)
                    sdf.line_to(0.0, h * 0.5)
                    sdf.close_path()
                    sdf.fill_keep(self.arrow_color)
                    sdf.stroke(self.arrow_border_color, 1.0)

                    return sdf.result
                }
            }
        }

        content := mod.widgets.MpPopoverBase{
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }
    }

    // Popover with arrow pointing right
    mod.widgets.MpPopoverArrowRight = View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}

        content := mod.widgets.MpPopoverBase{
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }

        arrow := View{
            width: 8
            height: 16
            margin: Inset{left: -1}

            show_bg: true
            draw_bg +: {
                arrow_color: instance(CARD)
                arrow_border_color: instance(BORDER)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    let w = self.rect_size.x
                    let h = self.rect_size.y

                    // Triangle pointing right
                    sdf.move_to(0.0, 0.0)
                    sdf.line_to(w, h * 0.5)
                    sdf.line_to(0.0, h)
                    sdf.close_path()
                    sdf.fill_keep(self.arrow_color)
                    sdf.stroke(self.arrow_border_color, 1.0)

                    return sdf.result
                }
            }
        }
    }

    // ============================================================
    // Popover Menu (for dropdown menus)
    // ============================================================

    mod.widgets.MpPopoverMenu = mod.widgets.MpPopoverBase{
        width: 200
        height: Fit
        padding: 4
        flow: Down
    }

    // Menu item
    mod.widgets.MpPopoverMenuItem = View{
        width: Fill
        height: Fit
        padding: Inset{left: 12, right: 12, top: 8, bottom: 8}
        flow: Right
        align: Align{y: 0.5}
        spacing: 8
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x00000000)
            bg_color_hover: instance(#xf1f5f9)
            border_radius: instance(4.0)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let result_color = mix(self.bg_color, self.bg_color_hover, self.hover)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.border_radius)
                sdf.fill(result_color)
                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.05}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
        }

        label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 14.0}
                color: FOREGROUND
            }
            text: "Menu Item"
        }
    }

    // Danger menu item
    mod.widgets.MpPopoverMenuItemDanger = mod.widgets.MpPopoverMenuItem{
        draw_bg +: {
            bg_color_hover: instance(#xfef2f2)
        }
        label := Label{
            draw_text +: {
                color: DANGER
            }
        }
    }

    // Menu divider
    mod.widgets.MpPopoverMenuDivider = View{
        width: Fill
        height: 1
        margin: Inset{top: 4, bottom: 4}
        show_bg: true
        draw_bg +: {
            color: instance(BORDER)
            pixel: fn() {
                return self.color
            }
        }
    }

    // Menu section header
    mod.widgets.MpPopoverMenuHeader = View{
        width: Fill
        height: Fit
        padding: Inset{left: 12, right: 12, top: 8, bottom: 4}

        Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 11.0}
                color: MUTED_FOREGROUND
            }
        }
    }

    // ============================================================
    // Popover Content Variants
    // ============================================================

    // Simple text popover
    mod.widgets.MpPopoverText = mod.widgets.MpPopoverBase{
        width: 240
        height: Fit
        padding: 12

        Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: FOREGROUND
            }
            text: "Popover content"
        }
    }

    // Popover with header
    mod.widgets.MpPopoverWithHeader = mod.widgets.MpPopoverBase{
        width: 280
        height: Fit
        flow: Down

        header := View{
            width: Fill
            height: Fit
            padding: Inset{left: 12, right: 12, top: 12, bottom: 8}

            title_label := Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: theme.font_bold{font_size: 14.0}
                    color: FOREGROUND
                }
                text: "Popover Title"
            }
        }

        body := View{
            width: Fill
            height: Fit
            padding: Inset{left: 12, right: 12, top: 0, bottom: 12}

            desc_label := Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 13.0}
                    color: MUTED_FOREGROUND
                }
                text: "Popover description text."
            }
        }
    }

    // ============================================================
    // Interactive Popover Widget
    // ============================================================

    mod.widgets.MpPopoverWidgetBase = #(MpPopoverWidget::register_widget(vm))
    mod.widgets.MpPopoverWidget = set_type_default() do mod.widgets.MpPopoverWidgetBase{
        width: Fit
        height: Fit
        flow: Overlay

        animation_duration: 0.15

        content := mod.widgets.MpPopoverBase{
            visible: false
            draw_bg +: { opacity: instance(0.0) }
            width: 200
            height: Fit
            padding: 8
            flow: Down
            spacing: 4
        }
    }

    // Fast animation variant
    mod.widgets.MpPopoverWidgetFast = mod.widgets.MpPopoverWidget{
        animation_duration: 0.03
    }

    // Slow animation variant
    mod.widgets.MpPopoverWidgetSlow = mod.widgets.MpPopoverWidget{
        animation_duration: 1.2
    }

    // Instant (no animation) variant
    mod.widgets.MpPopoverWidgetInstant = mod.widgets.MpPopoverWidget{
        animation_duration: 0.0
    }

    // ============================================================
    // Placement Variants (12 positions)
    // ============================================================

    // Top placement
    mod.widgets.MpPopoverTop = mod.widgets.MpPopoverWidget{
        trigger: mod.widgets.MpPopoverTrigger.Hover
        content := {
            abs_pos: vec2(-30.0, -70.0)
            width: Fit
            height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // TopLeft placement
    mod.widgets.MpPopoverTopLeft = mod.widgets.MpPopoverTop{
        content := { abs_pos: vec2(0.0, -70.0) }
    }

    // TopRight placement
    mod.widgets.MpPopoverTopRight = mod.widgets.MpPopoverTop{
        content := { abs_pos: vec2(-60.0, -70.0) }
    }

    // Bottom placement
    mod.widgets.MpPopoverBottom = mod.widgets.MpPopoverWidget{
        trigger: mod.widgets.MpPopoverTrigger.Hover
        content := {
            abs_pos: vec2(-30.0, 40.0)
            width: Fit
            height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // BottomLeft placement
    mod.widgets.MpPopoverBottomLeft = mod.widgets.MpPopoverBottom{
        content := { abs_pos: vec2(0.0, 40.0) }
    }

    // BottomRight placement
    mod.widgets.MpPopoverBottomRight = mod.widgets.MpPopoverBottom{
        content := { abs_pos: vec2(-60.0, 40.0) }
    }

    // Left placement
    mod.widgets.MpPopoverLeft = mod.widgets.MpPopoverWidget{
        trigger: mod.widgets.MpPopoverTrigger.Hover
        content := {
            abs_pos: vec2(-105.0, -10.0)
            width: Fit
            height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // LeftTop placement
    mod.widgets.MpPopoverLeftTop = mod.widgets.MpPopoverLeft{
        content := { abs_pos: vec2(-105.0, 0.0) }
    }

    // LeftBottom placement
    mod.widgets.MpPopoverLeftBottom = mod.widgets.MpPopoverLeft{
        content := { abs_pos: vec2(-105.0, -25.0) }
    }

    // Right placement
    mod.widgets.MpPopoverRight = mod.widgets.MpPopoverWidget{
        trigger: mod.widgets.MpPopoverTrigger.Hover
        content := {
            abs_pos: vec2(90.0, -10.0)
            width: Fit
            height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // RightTop placement
    mod.widgets.MpPopoverRightTop = mod.widgets.MpPopoverRight{
        content := { abs_pos: vec2(90.0, 0.0) }
    }

    // RightBottom placement
    mod.widgets.MpPopoverRightBottom = mod.widgets.MpPopoverRight{
        content := { abs_pos: vec2(90.0, -25.0) }
    }

    // ============================================================
    // Interactive Menu Item Widget
    // ============================================================

    mod.widgets.MpPopoverMenuItemWidgetBase = #(MpPopoverMenuItemWidget::register_widget(vm))
    mod.widgets.MpPopoverMenuItemWidget = set_type_default() do mod.widgets.MpPopoverMenuItemWidgetBase{
        width: Fill
        height: Fit
        padding: Inset{left: 12, right: 12, top: 8, bottom: 8}
        flow: Right
        align: Align{y: 0.5}
        spacing: 8
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x00000000)
            bg_color_hover: instance(#xf1f5f9)
            border_radius: instance(4.0)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let result_color = mix(self.bg_color, self.bg_color_hover, self.hover)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.border_radius)
                sdf.fill(result_color)
                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.05}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
        }

        label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 14.0}
                color: FOREGROUND
            }
            text: "Menu Item"
        }
    }
}

/// Popover actions
#[derive(Clone, Debug, Default)]
pub enum MpPopoverAction {
    #[default]
    None,
    Opened,
    Closed,
}

/// Menu item actions
#[derive(Clone, Debug, Default)]
pub enum MpPopoverMenuItemAction {
    #[default]
    None,
    Clicked,
}

/// Trigger mode for popover
#[derive(Copy, Clone, Debug, Default, Script, ScriptHook, PartialEq)]
pub enum MpPopoverTrigger {
    #[default]
    Click,
    Hover,
    Focus,
}

/// Interactive popover widget with show/hide functionality
/// Uses NextFrame manual animation for opacity fade
#[derive(Script, ScriptHook, Widget)]
pub struct MpPopoverWidget {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Trigger mode: Click, Hover, or Focus
    #[live]
    trigger: MpPopoverTrigger,

    /// Animation duration in seconds
    #[live(0.15)]
    animation_duration: f64,

    #[rust]
    opened: bool,

    /// Current opacity value (0.0 to 1.0)
    #[rust]
    opacity: f64,

    /// Animation direction: true = opening (fade in), false = closing (fade out)
    #[rust]
    animating: Option<bool>,

    /// Last frame time for animation
    #[rust]
    last_time: Option<f64>,

    /// NextFrame for animation
    #[rust]
    next_frame: NextFrame,
}

impl Widget for MpPopoverWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // Handle NextFrame for manual animation
        if let Some(nf) = self.next_frame.is_event(event) {
            if self.animating.is_some() {
                let is_opening = self.animating.unwrap();
                let dt = if let Some(last_time) = self.last_time {
                    nf.time - last_time
                } else {
                    0.0
                };
                self.last_time = Some(nf.time);

                // Calculate opacity change
                let duration = self.animation_duration.max(0.001); // Avoid division by zero
                let delta = dt / duration;

                if is_opening {
                    self.opacity = (self.opacity + delta).min(1.0);
                    if self.opacity >= 1.0 {
                        self.animating = None;
                    }
                } else {
                    self.opacity = (self.opacity - delta).max(0.0);
                    if self.opacity <= 0.0 {
                        self.animating = None;
                        // Hide content after fade-out completes
                        self.view(cx, ids!(content)).set_visible(cx, false);
                    }
                }

                // Apply opacity to content's draw_bg via script_apply_eval
                let mut content_view = self.view(cx, ids!(content));
                script_apply_eval!(cx, content_view, {
                    draw_bg: { opacity: #(self.opacity as f32) }
                });

                self.redraw(cx);

                // Request next frame if still animating
                if self.animating.is_some() {
                    self.next_frame = cx.new_next_frame();
                }
            }
        }

        // Handle trigger-specific events
        match self.trigger {
            MpPopoverTrigger::Hover => match event.hits(cx, self.view.area()) {
                Hit::FingerHoverIn(_) => {
                    self.open(cx);
                }
                Hit::FingerHoverOut(_) => {
                    self.close(cx);
                }
                _ => {}
            },
            MpPopoverTrigger::Focus => {
                match event.hits(cx, self.view.area()) {
                    Hit::FingerDown(_) => {
                        self.open(cx);
                    }
                    Hit::FingerHoverOut(_) if self.opened => {
                        // Close on blur (moving away)
                        self.close(cx);
                    }
                    _ => {}
                }
            }
            MpPopoverTrigger::Click => {
                // Click handling is done externally via toggle()
            }
        }

        self.view.handle_event(cx, event, _scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpPopoverWidget {
    /// Open the popover with animation
    pub fn open(&mut self, cx: &mut Cx) {
        if self.opened {
            return;
        }
        self.opened = true;
        // Make content visible
        self.view(cx, ids!(content)).set_visible(cx, true);

        // Start fade-in animation
        if self.animation_duration > 0.0 {
            self.animating = Some(true);
            self.last_time = None; // Will be set on first NextFrame
            self.next_frame = cx.new_next_frame();
        } else {
            // Instant show
            self.opacity = 1.0;
            let mut content_view = self.view(cx, ids!(content));
            script_apply_eval!(cx, content_view, {
                draw_bg: { opacity: 1.0 }
            });
        }
        self.redraw(cx);
    }

    /// Close the popover with animation
    pub fn close(&mut self, cx: &mut Cx) {
        if !self.opened {
            return;
        }
        self.opened = false;

        // Start fade-out animation
        if self.animation_duration > 0.0 {
            self.animating = Some(false);
            self.last_time = None; // Will be set on first NextFrame
            self.next_frame = cx.new_next_frame();
        } else {
            // Instant hide
            self.opacity = 0.0;
            self.view(cx, ids!(content)).set_visible(cx, false);
        }
        self.redraw(cx);
    }

    /// Toggle popover visibility with animation
    pub fn toggle(&mut self, cx: &mut Cx) {
        if self.opened {
            self.close(cx);
        } else {
            self.open(cx);
        }
    }

    /// Check if popover is visible
    pub fn is_open(&self) -> bool {
        self.opened
    }
}

impl MpPopoverWidgetRef {
    pub fn open(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open(cx);
        }
    }

    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
        }
    }

    pub fn toggle(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.toggle(cx);
        }
    }

    pub fn is_open(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_open()
        } else {
            false
        }
    }
}

/// Interactive menu item widget with click handling
#[derive(Script, ScriptHook, Widget)]
pub struct MpPopoverMenuItemWidget {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpPopoverMenuItemWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.view.handle_event(cx, event, _scope);

        let uid = self.widget_uid();

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                self.view.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.view.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(uid, MpPopoverMenuItemAction::Clicked);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpPopoverMenuItemWidget {
    /// Set the menu item label
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.view.label(cx, ids!(label)).set_text(cx, text);
    }
}

impl MpPopoverMenuItemWidgetRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            if let Some(item) = actions.find_widget_action(inner.widget_uid()) {
                return matches!(
                    item.cast::<MpPopoverMenuItemAction>(),
                    MpPopoverMenuItemAction::Clicked
                );
            }
        }
        false
    }
}
