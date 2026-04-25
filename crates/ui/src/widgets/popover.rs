use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::theme::colors::*;

    // ============================================================
    // MpPopover - Popover/Dropdown panel component
    // ============================================================

    // Base popover container
    pub MpPopoverBase = <View> {
        width: Fit
        height: Fit
        padding: 8

        show_bg: true
        draw_bg: {
            instance bg_color: (CARD)
            instance border_radius: 8.0
            instance border_color: (BORDER)
            instance shadow_color: #0000001A
            instance shadow_offset_y: 4.0
            instance shadow_blur: 12.0
            instance opacity: 1.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);

                // Shadow
                sdf.box(
                    0.0,
                    self.shadow_offset_y,
                    self.rect_size.x,
                    self.rect_size.y,
                    self.border_radius
                );
                sdf.blur = self.shadow_blur;
                sdf.fill(self.shadow_color);
                sdf.blur = 0.0;

                // Main card
                sdf.box(
                    0.5,
                    0.5,
                    self.rect_size.x - 1.0,
                    self.rect_size.y - 1.0,
                    self.border_radius
                );
                sdf.fill_keep(self.bg_color);
                sdf.stroke(self.border_color, 1.0);

                return Pal::premul(vec4(sdf.result.rgb, sdf.result.a * self.opacity));
            }
        }
    }

    // ============================================================
    // Default Popover
    // ============================================================

    pub MpPopover = <MpPopoverBase> {
        width: 240
        height: Fit
        padding: 12
        flow: Down
        spacing: 8
    }

    // Popover with arrow pointing up
    pub MpPopoverArrowUp = <View> {
        width: Fit
        height: Fit
        flow: Down
        align: { x: 0.5 }

        arrow = <View> {
            width: 16
            height: 8
            margin: { bottom: -1 }

            show_bg: true
            draw_bg: {
                instance arrow_color: (CARD)
                instance arrow_border_color: (BORDER)

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let w = self.rect_size.x;
                    let h = self.rect_size.y;

                    // Triangle pointing up
                    sdf.move_to(w * 0.5, 0.0);
                    sdf.line_to(w, h);
                    sdf.line_to(0.0, h);
                    sdf.close_path();
                    sdf.fill_keep(self.arrow_color);
                    sdf.stroke(self.arrow_border_color, 1.0);

                    return sdf.result;
                }
            }
        }

        content = <MpPopoverBase> {
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }
    }

    // Popover with arrow pointing down (static display)
    pub MpPopoverArrowDown = <View> {
        width: Fit
        height: Fit
        flow: Down
        align: { x: 0.5 }

        content = <MpPopoverBase> {
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }

        arrow = <View> {
            width: 16
            height: 8
            margin: { top: -1 }

            show_bg: true
            draw_bg: {
                instance arrow_color: (CARD)
                instance arrow_border_color: (BORDER)

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let w = self.rect_size.x;
                    let h = self.rect_size.y;

                    // Triangle pointing down
                    sdf.move_to(0.0, 0.0);
                    sdf.line_to(w, 0.0);
                    sdf.line_to(w * 0.5, h);
                    sdf.close_path();
                    sdf.fill_keep(self.arrow_color);
                    sdf.stroke(self.arrow_border_color, 1.0);

                    return sdf.result;
                }
            }
        }
    }

    // Popover with arrow pointing left (for right placement)
    pub MpPopoverArrowLeft = <View> {
        width: Fit
        height: Fit
        flow: Right
        align: { y: 0.5 }

        arrow = <View> {
            width: 8
            height: 16
            margin: { right: -1 }

            show_bg: true
            draw_bg: {
                instance arrow_color: (CARD)
                instance arrow_border_color: (BORDER)

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let w = self.rect_size.x;
                    let h = self.rect_size.y;

                    // Triangle pointing left
                    sdf.move_to(w, 0.0);
                    sdf.line_to(w, h);
                    sdf.line_to(0.0, h * 0.5);
                    sdf.close_path();
                    sdf.fill_keep(self.arrow_color);
                    sdf.stroke(self.arrow_border_color, 1.0);

                    return sdf.result;
                }
            }
        }

        content = <MpPopoverBase> {
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }
    }

    // Popover with arrow pointing right (for left placement)
    pub MpPopoverArrowRight = <View> {
        width: Fit
        height: Fit
        flow: Right
        align: { y: 0.5 }

        content = <MpPopoverBase> {
            width: 240
            height: Fit
            padding: 12
            flow: Down
            spacing: 8
        }

        arrow = <View> {
            width: 8
            height: 16
            margin: { left: -1 }

            show_bg: true
            draw_bg: {
                instance arrow_color: (CARD)
                instance arrow_border_color: (BORDER)

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let w = self.rect_size.x;
                    let h = self.rect_size.y;

                    // Triangle pointing right
                    sdf.move_to(0.0, 0.0);
                    sdf.line_to(w, h * 0.5);
                    sdf.line_to(0.0, h);
                    sdf.close_path();
                    sdf.fill_keep(self.arrow_color);
                    sdf.stroke(self.arrow_border_color, 1.0);

                    return sdf.result;
                }
            }
        }
    }

    // ============================================================
    // Popover Menu (for dropdown menus)
    // ============================================================

    pub MpPopoverMenu = <MpPopoverBase> {
        width: 200
        height: Fit
        padding: 4
        flow: Down
    }

    // Menu item
    pub MpPopoverMenuItem = <View> {
        width: Fill
        height: Fit
        padding: { left: 12, right: 12, top: 8, bottom: 8 }
        flow: Right
        align: { y: 0.5 }
        spacing: 8
        cursor: Hand

        show_bg: true
        draw_bg: {
            instance bg_color: #00000000
            instance bg_color_hover: #f1f5f9
            instance border_radius: 4.0
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let result_color = mix(self.bg_color, self.bg_color_hover, self.hover);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.border_radius);
                sdf.fill(result_color);
                return sdf.result;
            }
        }

        animator: {
            hover = {
                default: off
                off = {
                    from: { all: Forward { duration: 0.1 } }
                    apply: { draw_bg: { hover: 0.0 } }
                }
                on = {
                    from: { all: Forward { duration: 0.05 } }
                    apply: { draw_bg: { hover: 1.0 } }
                }
            }
        }

        label = <Label> {
            width: Fill
            height: Fit
            draw_text: {
                text_style: <THEME_FONT_REGULAR> { font_size: 14.0 }
                color: (FOREGROUND)
            }
            text: "Menu Item"
        }
    }

    // Danger menu item
    pub MpPopoverMenuItemDanger = <MpPopoverMenuItem> {
        draw_bg: {
            bg_color_hover: #fef2f2
        }

        label = <Label> {
            draw_text: {
                color: (DANGER)
            }
        }
    }

    // Menu divider
    pub MpPopoverMenuDivider = <View> {
        width: Fill
        height: 1
        margin: { top: 4, bottom: 4 }
        show_bg: true
        draw_bg: {
            color: (BORDER)
        }
    }

    // Menu section header
    pub MpPopoverMenuHeader = <View> {
        width: Fill
        height: Fit
        padding: { left: 12, right: 12, top: 8, bottom: 4 }

        <Label> {
            width: Fill
            height: Fit
            draw_text: {
                text_style: <THEME_FONT_BOLD> { font_size: 11.0 }
                color: (MUTED_FOREGROUND)
            }
        }
    }

    // ============================================================
    // Popover Content Variants
    // ============================================================

    // Simple text popover (for tooltips with more content)
    pub MpPopoverText = <MpPopoverBase> {
        width: 240
        height: Fit
        padding: 12

        <Label> {
            width: Fill
            height: Fit
            draw_text: {
                text_style: <THEME_FONT_REGULAR> { font_size: 13.0 }
                color: (FOREGROUND)
            }
            text: "Popover content"
        }
    }

    // Popover with header
    pub MpPopoverWithHeader = <MpPopoverBase> {
        width: 280
        height: Fit
        flow: Down

        header = <View> {
            width: Fill
            height: Fit
            padding: { left: 12, right: 12, top: 12, bottom: 8 }

            title_label = <Label> {
                width: Fill
                height: Fit
                draw_text: {
                    text_style: <THEME_FONT_BOLD> { font_size: 14.0 }
                    color: (FOREGROUND)
                }
                text: "Popover Title"
            }
        }

        body = <View> {
            width: Fill
            height: Fit
            padding: { left: 12, right: 12, top: 0, bottom: 12 }

            desc_label = <Label> {
                width: Fill
                height: Fit
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> { font_size: 13.0 }
                    color: (MUTED_FOREGROUND)
                }
                text: "Popover description text."
            }
        }
    }

    // ============================================================
    // Interactive Popover Widget
    // ============================================================

    pub MpPopoverWidget = {{MpPopoverWidget}} {
        width: Fit
        height: Fit
        flow: Overlay

        // Animation duration in seconds (can be overridden)
        animation_duration: 0.15

        // Content with opacity-enabled background (hidden by default)
        content = <MpPopoverBase> {
            visible: false
            draw_bg: { opacity: 0.0 }
            width: 200
            height: Fit
            padding: 8
            flow: Down
            spacing: 4
        }
    }

    // Fast animation variant
    pub MpPopoverWidgetFast = <MpPopoverWidget> {
        animation_duration: 0.03
    }

    // Slow animation variant
    pub MpPopoverWidgetSlow = <MpPopoverWidget> {
        animation_duration: 1.2
    }

    // Instant (no animation) variant
    pub MpPopoverWidgetInstant = <MpPopoverWidget> {
        animation_duration: 0.0
    }

    // ============================================================
    // Placement Variants (12 positions like Ant Design)
    // ============================================================

    // --- Top placements (popover above trigger, arrow points down) ---

    // Top placement (centered) - popover appears above trigger
    pub MpPopoverTop = <MpPopoverWidget> {
        trigger: Hover
        content = {
            abs_pos: vec2(-30.0, -70.0)
            width: Fit, height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // TopLeft placement
    pub MpPopoverTopLeft = <MpPopoverTop> {
        content = { abs_pos: vec2(0.0, -70.0) }
    }

    // TopRight placement
    pub MpPopoverTopRight = <MpPopoverTop> {
        content = { abs_pos: vec2(-60.0, -70.0) }
    }

    // --- Bottom placements (popover below trigger, arrow points up) ---

    // Bottom placement (centered) - popover appears below trigger
    pub MpPopoverBottom = <MpPopoverWidget> {
        trigger: Hover
        content = {
            abs_pos: vec2(-30.0, 40.0)
            width: Fit, height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // BottomLeft placement
    pub MpPopoverBottomLeft = <MpPopoverBottom> {
        content = { abs_pos: vec2(0.0, 40.0) }
    }

    // BottomRight placement
    pub MpPopoverBottomRight = <MpPopoverBottom> {
        content = { abs_pos: vec2(-60.0, 40.0) }
    }

    // --- Left placements (popover to the left, arrow points right) ---

    // Left placement (centered) - popover appears to the left
    pub MpPopoverLeft = <MpPopoverWidget> {
        trigger: Hover
        content = {
            abs_pos: vec2(-105.0, -10.0)
            width: Fit, height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // LeftTop placement
    pub MpPopoverLeftTop = <MpPopoverLeft> {
        content = { abs_pos: vec2(-105.0, 0.0) }
    }

    // LeftBottom placement
    pub MpPopoverLeftBottom = <MpPopoverLeft> {
        content = { abs_pos: vec2(-105.0, -25.0) }
    }

    // --- Right placements (popover to the right, arrow points left) ---

    // Right placement (centered) - popover appears to the right
    pub MpPopoverRight = <MpPopoverWidget> {
        trigger: Hover
        content = {
            abs_pos: vec2(90.0, -10.0)
            width: Fit, height: Fit
            padding: 12
            flow: Down
            spacing: 4
        }
    }

    // RightTop placement
    pub MpPopoverRightTop = <MpPopoverRight> {
        content = { abs_pos: vec2(90.0, 0.0) }
    }

    // RightBottom placement
    pub MpPopoverRightBottom = <MpPopoverRight> {
        content = { abs_pos: vec2(90.0, -25.0) }
    }

    // ============================================================
    // Interactive Menu Item Widget
    // ============================================================

    pub MpPopoverMenuItemWidget = {{MpPopoverMenuItemWidget}} {
        width: Fill
        height: Fit
        padding: { left: 12, right: 12, top: 8, bottom: 8 }
        flow: Right
        align: { y: 0.5 }
        spacing: 8
        cursor: Hand

        show_bg: true
        draw_bg: {
            instance bg_color: #00000000
            instance bg_color_hover: #f1f5f9
            instance border_radius: 4.0
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let result_color = mix(self.bg_color, self.bg_color_hover, self.hover);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.border_radius);
                sdf.fill(result_color);
                return sdf.result;
            }
        }

        label = <Label> {
            width: Fill
            height: Fit
            draw_text: {
                text_style: <THEME_FONT_REGULAR> { font_size: 14.0 }
                color: (FOREGROUND)
            }
            text: "Menu Item"
        }
    }
}

/// Popover actions
#[derive(Clone, Debug, DefaultNone)]
pub enum MpPopoverAction {
    None,
    Opened,
    Closed,
}

/// Menu item actions
#[derive(Clone, Debug, DefaultNone)]
pub enum MpPopoverMenuItemAction {
    None,
    Clicked,
}

/// Trigger mode for popover
#[derive(Copy, Clone, Debug, Live, LiveHook)]
#[live_ignore]
pub enum MpPopoverTrigger {
    #[pick]
    Click,
    Hover,
    Focus,
}

/// Interactive popover widget with show/hide functionality
/// Uses NextFrame manual animation for opacity fade (animator cannot animate child components)
#[derive(Live, LiveHook, Widget)]
pub struct MpPopoverWidget {
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
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
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
                        self.view.view(ids!(content)).set_visible(cx, false);
                    }
                }

                // Apply opacity to content's draw_bg
                self.view.view(ids!(content)).apply_over(
                    cx,
                    live! {
                        draw_bg: { opacity: (self.opacity) }
                    },
                );

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

        self.view.handle_event(cx, event, scope);
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
        self.view.view(ids!(content)).set_visible(cx, true);

        // Start fade-in animation
        if self.animation_duration > 0.0 {
            self.animating = Some(true);
            self.last_time = None; // Will be set on first NextFrame
            self.next_frame = cx.new_next_frame();
        } else {
            // Instant show
            self.opacity = 1.0;
            self.view.view(ids!(content)).apply_over(
                cx,
                live! {
                    draw_bg: { opacity: 1.0 }
                },
            );
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
            self.view.view(ids!(content)).set_visible(cx, false);
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
#[derive(Live, LiveHook, Widget)]
pub struct MpPopoverMenuItemWidget {
    #[deref]
    view: View,
}

impl Widget for MpPopoverMenuItemWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                self.view.apply_over(cx, live! { draw_bg: { hover: 1.0 } });
                self.redraw(cx);
            }
            Hit::FingerHoverOut(_) => {
                self.view.apply_over(cx, live! { draw_bg: { hover: 0.0 } });
                self.redraw(cx);
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(
                        self.widget_uid(),
                        &scope.path,
                        MpPopoverMenuItemAction::Clicked,
                    );
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
        self.view.label(ids!(label)).set_text(cx, text);
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
