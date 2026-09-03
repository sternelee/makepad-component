use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // Alert Icons - SDF drawn icons for each variant
    // ============================================================

    // Info icon (circle with i)
    mod.widgets.MpAlertIconInfo = View{
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

    // Success icon (checkmark in circle)
    mod.widgets.MpAlertIconSuccess = View{
        width: 20
        height: 20
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(SUCCESS)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = min(c.x, c.y) - 1.0

                // Circle
                sdf.circle(c.x, c.y, r)
                sdf.stroke(self.icon_color, 1.5)

                // Checkmark
                sdf.move_to(c.x - 4.0, c.y)
                sdf.line_to(c.x - 1.0, c.y + 3.0)
                sdf.line_to(c.x + 5.0, c.y - 3.0)
                sdf.stroke(self.icon_color, 1.5)

                return sdf.result
            }
        }
    }

    // Warning icon (triangle with !)
    mod.widgets.MpAlertIconWarning = View{
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

    // Error icon (X in circle)
    mod.widgets.MpAlertIconError = View{
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

    // Secondary icon (info style, neutral)
    mod.widgets.MpAlertIconSecondary = View{
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

                // Circle
                sdf.circle(c.x, c.y, r)
                sdf.stroke(self.icon_color, 1.5)

                // Letter i
                sdf.circle(c.x, c.y - 3.0, 1.5)
                sdf.fill(self.icon_color)
                sdf.rect(c.x - 1.0, c.y, 2.0, 5.0)
                sdf.fill(self.icon_color)

                return sdf.result
            }
        }
    }

    // ============================================================
    // Close button
    // ============================================================

    mod.widgets.MpAlertCloseButton = View{
        width: 20
        height: 20
        cursor: MouseCursor.Hand
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            icon_color: instance(TEXT_MUTED)
            hover: instance(0.0)
            bg_hover_color: instance(ELEMENT_HOVER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let size = 5.0
                let r = min(c.x, c.y)

                // Hover background
                sdf.circle(c.x, c.y, r)
                sdf.fill(mix(#x0000, self.bg_hover_color, self.hover))

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
    // Content subtree templates (title + message)
    // Variants override nested colors by re-declaring these
    // templates with the same ids; the entries apply in place.
    // ============================================================

    mod.widgets.MpAlertTitle = Label{
        width: Fill
        height: Fit
        draw_text +: {
            text_style: theme.font_bold{font_size: 14.0}
            color: TEXT
        }
        text: ""
    }

    mod.widgets.MpAlertTitleWrapper = View{
        width: Fill
        height: Fit
        visible: false

        title := mod.widgets.MpAlertTitle{}
    }

    mod.widgets.MpAlertMessage = Label{
        width: Fill
        height: Fit
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT_MUTED
        }
        text: ""
    }

    mod.widgets.MpAlertContent = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 4.0

        title_wrapper := mod.widgets.MpAlertTitleWrapper{}
        message := mod.widgets.MpAlertMessage{}
    }

    // ============================================================
    // Base Alert
    // ============================================================

    mod.widgets.MpAlertBase = #(MpAlert::register_widget(vm))
    mod.widgets.MpAlert = set_type_default() do mod.widgets.MpAlertBase{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, right: 16.0, top: 12.0, bottom: 12.0}
        flow: Right
        spacing: 12.0
        align: Align{y: 0.0}

        show_bg: true
        draw_bg +: {
            bg_color: instance(ELEMENT_HOVER)
            border_radius: instance(8.0)
            border_color: instance(BORDER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Main box with rounded corners
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

        icon := mod.widgets.MpAlertIconSecondary{}
        content := mod.widgets.MpAlertContent{}
        close_button := mod.widgets.MpAlertCloseButton{}
    }

    // ============================================================
    // Alert Variants
    // ============================================================

    // Info Alert
    mod.widgets.MpAlertInfo = mod.widgets.MpAlert{
        draw_bg +: {
            bg_color: instance(#x0891b210)
            border_color: instance(INFO)
        }

        icon := mod.widgets.MpAlertIconInfo{}

        content := mod.widgets.MpAlertContent{
            title_wrapper := mod.widgets.MpAlertTitleWrapper{
                title := mod.widgets.MpAlertTitle{
                    draw_text +: { color: INFO }
                }
            }
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: INFO }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(INFO)
                bg_hover_color: instance(INFO)
            }
        }
    }

    // Success Alert
    mod.widgets.MpAlertSuccess = mod.widgets.MpAlert{
        draw_bg +: {
            bg_color: instance(#x16a34a10)
            border_color: instance(SUCCESS)
        }

        icon := mod.widgets.MpAlertIconSuccess{}

        content := mod.widgets.MpAlertContent{
            title_wrapper := mod.widgets.MpAlertTitleWrapper{
                title := mod.widgets.MpAlertTitle{
                    draw_text +: { color: SUCCESS }
                }
            }
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: SUCCESS }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(SUCCESS)
                bg_hover_color: instance(SUCCESS)
            }
        }
    }

    // Warning Alert
    mod.widgets.MpAlertWarning = mod.widgets.MpAlert{
        draw_bg +: {
            bg_color: instance(#xd9770610)
            border_color: instance(WARNING)
        }

        icon := mod.widgets.MpAlertIconWarning{}

        content := mod.widgets.MpAlertContent{
            title_wrapper := mod.widgets.MpAlertTitleWrapper{
                title := mod.widgets.MpAlertTitle{
                    draw_text +: { color: WARNING }
                }
            }
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: WARNING }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(WARNING)
                bg_hover_color: instance(WARNING)
            }
        }
    }

    // Error/Danger Alert
    mod.widgets.MpAlertError = mod.widgets.MpAlert{
        draw_bg +: {
            bg_color: instance(#xdc26261a)
            border_color: instance(DANGER)
        }

        icon := mod.widgets.MpAlertIconError{}

        content := mod.widgets.MpAlertContent{
            title_wrapper := mod.widgets.MpAlertTitleWrapper{
                title := mod.widgets.MpAlertTitle{
                    draw_text +: { color: DANGER }
                }
            }
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: DANGER }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(DANGER)
                bg_hover_color: instance(DANGER)
            }
        }
    }

    // ============================================================
    // Banner Variants (full width, no border radius)
    // ============================================================

    mod.widgets.MpAlertBannerBase = mod.widgets.MpAlert{
        align: Align{y: 0.5}

        draw_bg +: {
            border_radius: instance(0.0)
        }
    }

    mod.widgets.MpAlertBanner = mod.widgets.MpAlertBannerBase{}

    mod.widgets.MpAlertBannerInfo = mod.widgets.MpAlertBannerBase{
        draw_bg +: {
            bg_color: instance(#x0891b210)
            border_color: instance(INFO)
        }

        icon := mod.widgets.MpAlertIconInfo{}

        content := mod.widgets.MpAlertContent{
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: INFO }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(INFO)
                bg_hover_color: instance(INFO)
            }
        }
    }

    mod.widgets.MpAlertBannerSuccess = mod.widgets.MpAlertBannerBase{
        draw_bg +: {
            bg_color: instance(#x16a34a10)
            border_color: instance(SUCCESS)
        }

        icon := mod.widgets.MpAlertIconSuccess{}

        content := mod.widgets.MpAlertContent{
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: SUCCESS }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(SUCCESS)
                bg_hover_color: instance(SUCCESS)
            }
        }
    }

    mod.widgets.MpAlertBannerWarning = mod.widgets.MpAlertBannerBase{
        draw_bg +: {
            bg_color: instance(#xd9770610)
            border_color: instance(WARNING)
        }

        icon := mod.widgets.MpAlertIconWarning{}

        content := mod.widgets.MpAlertContent{
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: WARNING }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(WARNING)
                bg_hover_color: instance(WARNING)
            }
        }
    }

    mod.widgets.MpAlertBannerError = mod.widgets.MpAlertBannerBase{
        draw_bg +: {
            bg_color: instance(#xdc26261a)
            border_color: instance(DANGER)
        }

        icon := mod.widgets.MpAlertIconError{}

        content := mod.widgets.MpAlertContent{
            message := mod.widgets.MpAlertMessage{
                draw_text +: { color: DANGER }
            }
        }

        close_button := mod.widgets.MpAlertCloseButton{
            draw_bg +: {
                icon_color: instance(DANGER)
                bg_hover_color: instance(DANGER)
            }
        }
    }
}

/// Alert action emitted when the close button is clicked
#[derive(Clone, Debug, Default)]
pub enum MpAlertAction {
    #[default]
    None,
    Close,
}

/// Alert widget for displaying important messages to users
#[derive(Script, Widget)]
pub struct MpAlert {
    #[source]
    source: ScriptObjectRef,

    #[deref]
    view: View,

    /// Whether the alert is visible
    #[live(true)]
    #[visible]
    visible: bool,

    /// Whether to show the close button
    #[live(false)]
    closable: bool,

    /// Five-step size driving padding and title/message font.
    #[live]
    size: MpSize,

    /// Last size applied to the labels (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl ScriptHook for MpAlert {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        // Re-assert the close button visibility after every structural apply,
        // mirroring the old `LiveHook::after_apply` behavior. Animator-driven
        // and eval applies don't change the child set, so skip those.
        if apply.is_eval() || apply.is_animate() || apply.as_default().is_some() {
            return;
        }
        vm.with_cx_mut(|cx| self.sync_visibility(cx));
    }
}

impl Widget for MpAlert {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        let uid = self.widget_uid();

        // Handle close button hover and click
        let close_button = self.view.view(cx, ids!(close_button));
        match event.hits(cx, close_button.area()) {
            Hit::FingerHoverIn(_) => {
                close_button.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                close_button.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerDown(_) => {
                cx.widget_action(uid, MpAlertAction::Close);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }

        // Metrics from the size system (Medium = the original 16/12 look).
        // Alerts are roomy display containers: padding scaled up from the
        // control metrics.
        self.view.layout.padding = Inset {
            left: self.size.padding_h() + 4.0,
            right: self.size.padding_h() + 4.0,
            top: self.size.padding_v() + 6.0,
            bottom: self.size.padding_v() + 6.0,
        };

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut title) = self
                .view
                .label(cx, ids!(content.title_wrapper.title))
                .borrow_mut()
            {
                title.draw_text.text_style.font_size = self.size.font_size() + 1.0;
            }
            if let Some(mut message) = self.view.label(cx, ids!(content.message)).borrow_mut() {
                message.draw_text.text_style.font_size = self.size.font_size();
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpAlert {
    /// Sync visibility of close button
    fn sync_visibility(&mut self, cx: &mut Cx) {
        let closable = self.closable;
        self.view
            .view(cx, ids!(close_button))
            .set_visible(cx, closable);
    }

    /// Set the alert title
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        let title_label = self.view.label(cx, ids!(content.title_wrapper.title));
        title_label.set_text(cx, title);
        self.view
            .view(cx, ids!(content.title_wrapper))
            .set_visible(cx, !title.is_empty());
        self.redraw(cx);
    }

    /// Set the alert message
    pub fn set_message(&mut self, cx: &mut Cx, message: &str) {
        self.view
            .label(cx, ids!(content.message))
            .set_text(cx, message);
        self.redraw(cx);
    }

    /// Set whether the alert is visible
    pub fn set_visible(&mut self, cx: &mut Cx, visible: bool) {
        self.visible = visible;
        self.redraw(cx);
    }

    /// Get whether the alert is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set whether the close button is shown
    pub fn set_closable(&mut self, cx: &mut Cx, closable: bool) {
        self.closable = closable;
        self.sync_visibility(cx);
        self.redraw(cx);
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

    /// Close the alert (set visible to false)
    pub fn close(&mut self, cx: &mut Cx) {
        self.set_visible(cx, false);
    }
}

impl MpAlertRef {
    /// Set the alert title
    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    /// Set the alert message
    pub fn set_message(&self, cx: &mut Cx, message: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_message(cx, message);
        }
    }

    /// Set whether the alert is visible
    pub fn set_visible(&self, cx: &mut Cx, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_visible(cx, visible);
        }
    }

    /// Get whether the alert is visible
    pub fn is_visible(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_visible()
        } else {
            false
        }
    }

    /// Set whether the close button is shown
    pub fn set_closable(&self, cx: &mut Cx, closable: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_closable(cx, closable);
        }
    }

    /// Close the alert
    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close(cx);
        }
    }

    /// Check if the close action was triggered
    pub fn closed(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast::<MpAlertAction>(), MpAlertAction::Close)
        } else {
            false
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
