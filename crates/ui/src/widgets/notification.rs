use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpNotification - Toast/notification component
    // ============================================================

    // Close button (X mark with hover animator)
    mod.widgets.MpNotificationCloseButton = View{
        width: 20
        height: 20
        cursor: MouseCursor.Hand
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(TEXT_MUTED)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let size = 5.0

                let final_color = mix(self.icon_color, #x64748b, self.hover)

                // X mark
                sdf.move_to(c.x - size, c.y - size)
                sdf.line_to(c.x + size, c.y + size)
                sdf.stroke(final_color, 1.5)

                sdf.move_to(c.x + size, c.y - size)
                sdf.line_to(c.x - size, c.y + size)
                sdf.stroke(final_color, 1.5)

                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
        }
    }

    // ============================================================
    // Icon templates (SDF drawn, one per notification type)
    // ============================================================

    // Default icon (info glyph, muted color)
    mod.widgets.MpNotificationIcon = View{
        width: 20
        height: 20
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(TEXT_MUTED)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = min(c.x, c.y) - 1.0

                // Info icon (circle with i)
                sdf.circle(c.x, c.y, r)
                sdf.stroke(self.icon_color, 1.5)

                // Letter i - dot
                sdf.circle(c.x, c.y - 3.0, 1.5)
                sdf.fill(self.icon_color)
                // Letter i - stem
                sdf.rect(c.x - 1.0, c.y, 2.0, 5.0)
                sdf.fill(self.icon_color)

                return sdf.result
            }
        }
    }

    // Success icon (checkmark)
    mod.widgets.MpNotificationIconSuccess = View{
        width: 20
        height: 20
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(SUCCESS)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5

                // Checkmark
                sdf.move_to(c.x - 5.0, c.y)
                sdf.line_to(c.x - 1.0, c.y + 4.0)
                sdf.line_to(c.x + 6.0, c.y - 4.0)
                sdf.stroke(self.icon_color, 2.0)

                return sdf.result
            }
        }
    }

    // Error icon (X in circle)
    mod.widgets.MpNotificationIconError = View{
        width: 20
        height: 20
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(DANGER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = min(c.x, c.y) - 1.0

                // Circle
                sdf.circle(c.x, c.y, r)
                sdf.stroke(self.icon_color, 1.5)

                // X mark
                let size = 4.0
                sdf.move_to(c.x - size, c.y - size)
                sdf.line_to(c.x + size, c.y + size)
                sdf.stroke(self.icon_color, 1.5)

                sdf.move_to(c.x + size, c.y - size)
                sdf.line_to(c.x - size, c.y + size)
                sdf.stroke(self.icon_color, 1.5)

                return sdf.result
            }
        }
    }

    // Warning icon (triangle with !)
    mod.widgets.MpNotificationIconWarning = View{
        width: 20
        height: 20
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(WARNING)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5

                // Triangle
                sdf.move_to(c.x, 2.0)
                sdf.line_to(self.rect_size.x - 2.0, self.rect_size.y - 2.0)
                sdf.line_to(2.0, self.rect_size.y - 2.0)
                sdf.close_path()
                sdf.stroke(self.icon_color, 1.5)

                // Exclamation mark - stem
                sdf.rect(c.x - 1.0, 7.0, 2.0, 5.0)
                sdf.fill(self.icon_color)
                // Exclamation mark - dot
                sdf.circle(c.x, 15.0, 1.5)
                sdf.fill(self.icon_color)

                return sdf.result
            }
        }
    }

    // Info icon (circle with i, info color)
    mod.widgets.MpNotificationIconInfo = View{
        width: 20
        height: 20
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(INFO)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = min(c.x, c.y) - 1.0

                // Circle
                sdf.circle(c.x, c.y, r)
                sdf.stroke(self.icon_color, 1.5)

                // Letter i - dot
                sdf.circle(c.x, c.y - 3.0, 1.5)
                sdf.fill(self.icon_color)
                // Letter i - stem
                sdf.rect(c.x - 1.0, c.y, 2.0, 5.0)
                sdf.fill(self.icon_color)

                return sdf.result
            }
        }
    }

    // ============================================================
    // Content subtree templates (title + message)
    // Variants override nested colors/text by re-declaring these
    // templates with the same ids; the entries apply in place.
    // ============================================================

    mod.widgets.MpNotificationTitle = Label{
        width: Fill
        height: Fit
        draw_text +: {
            text_style: theme.font_bold{font_size: 14.0}
            color: TEXT
        }
        text: "Notification"
    }

    mod.widgets.MpNotificationMessage = Label{
        width: Fill
        height: Fit
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT_MUTED
        }
        text: ""
    }

    mod.widgets.MpNotificationContent = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 4

        title := mod.widgets.MpNotificationTitle{}
        message := mod.widgets.MpNotificationMessage{}
    }

    // ============================================================
    // Base notification
    // ============================================================

    mod.widgets.MpNotificationBase = View{
        width: 320
        height: Fit
        padding: 16
        flow: Right
        spacing: 12
        align: Align{y: 0.0}

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_radius: instance(8.0)
            border_color: instance(BORDER)
            shadow_color: instance(#x0000001A)
            shadow_offset_y: instance(4.0)
            shadow_blur: instance(12.0)

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

                return sdf.result
            }
        }
    }

    // ============================================================
    // Default Notification
    // ============================================================

    mod.widgets.MpNotification = mod.widgets.MpNotificationBase{
        icon := mod.widgets.MpNotificationIcon{}
        content := mod.widgets.MpNotificationContent{}
        close := mod.widgets.MpNotificationCloseButton{}
    }

    // ============================================================
    // Success Notification
    // ============================================================

    mod.widgets.MpNotificationSuccess = mod.widgets.MpNotificationBase{
        draw_bg +: {
            border_color: instance(SUCCESS)
        }

        icon := mod.widgets.MpNotificationIconSuccess{}

        content := mod.widgets.MpNotificationContent{
            title := mod.widgets.MpNotificationTitle{
                draw_text +: { color: SUCCESS }
                text: "Success"
            }
        }

        close := View{
            width: 20
            height: 20
            cursor: MouseCursor.Hand
        }
    }

    // ============================================================
    // Error Notification
    // ============================================================

    mod.widgets.MpNotificationError = mod.widgets.MpNotificationBase{
        draw_bg +: {
            border_color: instance(DANGER)
        }

        icon := mod.widgets.MpNotificationIconError{}

        content := mod.widgets.MpNotificationContent{
            title := mod.widgets.MpNotificationTitle{
                draw_text +: { color: DANGER }
                text: "Error"
            }
        }

        close := View{
            width: 20
            height: 20
            cursor: MouseCursor.Hand
        }
    }

    // ============================================================
    // Warning Notification
    // ============================================================

    mod.widgets.MpNotificationWarning = mod.widgets.MpNotificationBase{
        draw_bg +: {
            border_color: instance(WARNING)
        }

        icon := mod.widgets.MpNotificationIconWarning{}

        content := mod.widgets.MpNotificationContent{
            title := mod.widgets.MpNotificationTitle{
                draw_text +: { color: WARNING }
                text: "Warning"
            }
        }

        close := View{
            width: 20
            height: 20
            cursor: MouseCursor.Hand
        }
    }

    // ============================================================
    // Info Notification
    // ============================================================

    mod.widgets.MpNotificationInfo = mod.widgets.MpNotificationBase{
        draw_bg +: {
            border_color: instance(INFO)
        }

        icon := mod.widgets.MpNotificationIconInfo{}

        content := mod.widgets.MpNotificationContent{
            title := mod.widgets.MpNotificationTitle{
                draw_text +: { color: INFO }
                text: "Info"
            }
        }

        close := View{
            width: 20
            height: 20
            cursor: MouseCursor.Hand
        }
    }

    // ============================================================
    // Notification Container (for positioning)
    // ============================================================

    mod.widgets.MpNotificationContainer = View{
        width: Fill
        height: Fill
        flow: Overlay

        // Top-right corner positioning
        align: Align{x: 1.0, y: 0.0}
        padding: 16

        notifications := View{
            width: Fit
            height: Fit
            flow: Down
            spacing: 8
        }
    }

    // ============================================================
    // Interactive Notification Widget
    // ============================================================

    mod.widgets.MpNotificationWidgetBase = #(MpNotificationWidget::register_widget(vm))
    mod.widgets.MpNotificationWidget = set_type_default() do mod.widgets.MpNotificationWidgetBase{
        width: 320
        height: Fit
        padding: 16
        flow: Right
        spacing: 12
        align: Align{y: 0.0}
        visible: false

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_radius: instance(8.0)
            border_color: instance(BORDER)
            shadow_color: instance(#x0000001A)
            shadow_offset_y: instance(4.0)
            shadow_blur: instance(12.0)

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

                return sdf.result
            }
        }

        content := mod.widgets.MpNotificationContent{}
        close := mod.widgets.MpNotificationCloseButton{}
    }
}

/// Notification actions
#[derive(Clone, Debug, Default)]
pub enum MpNotificationAction {
    #[default]
    None,
    Closed,
}

/// Interactive notification widget
#[derive(Script, ScriptHook, Widget)]
pub struct MpNotificationWidget {
    #[source]
    source: ScriptObjectRef,

    #[deref]
    view: View,

    /// Whether the notification is visible
    #[live(false)]
    #[visible]
    visible: bool,

    /// Five-step size driving padding and title/message font.
    #[live]
    size: MpSize,

    /// Last size applied to the labels (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpNotificationWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.visible {
            return;
        }

        self.view.handle_event(cx, event, scope);

        let uid = self.widget_uid();

        // Handle close button
        let close_btn = self.view.view(cx, ids!(close));
        match event.hits(cx, close_btn.area()) {
            Hit::FingerHoverIn(_) => {
                close_btn.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                close_btn.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe)
                if fe.is_over => {
                    self.visible = false;
                    self.redraw(cx);
                    cx.widget_action(uid, MpNotificationAction::Closed);
                }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }

        // Metrics from the size system (Medium = the original 16 uniform look).
        let pad = self.size.padding_h() + 4.0;
        self.view.layout.padding = Inset {
            left: pad,
            right: pad,
            top: pad,
            bottom: pad,
        };

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut title) = self.view.label(cx, ids!(content.title)).borrow_mut() {
                title.draw_text.text_style.font_size = self.size.font_size() + 1.0;
            }
            if let Some(mut message) = self.view.label(cx, ids!(content.message)).borrow_mut() {
                message.draw_text.text_style.font_size = self.size.font_size();
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpNotificationWidget {
    /// Show the notification
    pub fn show(&mut self, cx: &mut Cx) {
        self.visible = true;
        self.redraw(cx);
    }

    /// Hide the notification
    pub fn hide(&mut self, cx: &mut Cx) {
        self.visible = false;
        self.redraw(cx);
    }

    /// Set the notification title
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view.label(cx, ids!(content.title)).set_text(cx, title);
    }

    /// Set the notification message
    pub fn set_message(&mut self, cx: &mut Cx, message: &str) {
        self.view
            .label(cx, ids!(content.message))
            .set_text(cx, message);
    }

    /// Show with title and message
    pub fn show_message(&mut self, cx: &mut Cx, title: &str, message: &str) {
        self.set_title(cx, title);
        self.set_message(cx, message);
        self.show(cx);
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Labels re-sync on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpNotificationWidgetRef {
    pub fn show(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx);
        }
    }

    pub fn hide(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide(cx);
        }
    }

    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn set_message(&self, cx: &mut Cx, message: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_message(cx, message);
        }
    }

    pub fn show_message(&self, cx: &mut Cx, title: &str, message: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_message(cx, title, message);
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
