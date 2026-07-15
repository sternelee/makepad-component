use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpModal - Modal/Dialog component
    // ============================================================

    // Modal backdrop (overlay)
    mod.widgets.MpModalBackdrop = View{
        width: Fill
        height: Fill

        show_bg: true
        draw_bg +: {
            color: instance(#x00000080)

            pixel: fn() {
                return self.color
            }
        }
    }

    // Modal container (centers the dialog)
    mod.widgets.MpModalContainer = View{
        width: Fill
        height: Fill
        flow: Overlay
        align: Align{x: 0.5, y: 0.5}

        backdrop := mod.widgets.MpModalBackdrop{}
    }

    // ============================================================
    // Modal Dialog
    // ============================================================

    // Base modal dialog
    mod.widgets.MpModal = View{
        width: 400
        height: Fit
        flow: Down

        show_bg: true
        draw_bg +: {
            bg_color: instance(CARD)
            border_radius: instance(12.0)
            border_color: instance(BORDER)
            shadow_color: instance(#x00000033)
            shadow_offset_y: instance(8.0)
            shadow_blur: instance(24.0)

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

        header := View{
            width: Fill
            height: Fit
            padding: Inset{left: 24, right: 24, top: 20, bottom: 16}
            flow: Right
            align: Align{y: 0.5}

            title := Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: theme.font_bold{font_size: 18.0}
                    color: FOREGROUND
                }
                text: "Modal Title"
            }

            close := View{
                width: 24
                height: 24
                cursor: MouseCursor.Hand
                align: Align{x: 0.5, y: 0.5}

                show_bg: true
                draw_bg +: {
                    icon_color: instance(#x94a3b8)
                    hover: instance(0.0)

                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        let c = self.rect_size * 0.5
                        let size = 6.0

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
        }

        body := View{
            width: Fill
            height: Fit
            padding: Inset{left: 24, right: 24, top: 0, bottom: 16}
            flow: Down
            spacing: 8

            Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: MUTED_FOREGROUND
                }
                text: "Modal content goes here."
            }
        }

        footer := View{
            width: Fill
            height: Fit
            padding: Inset{left: 24, right: 24, top: 16, bottom: 20}
            flow: Right
            spacing: 8
            align: Align{x: 1.0, y: 0.5}
        }
    }

    // ============================================================
    // Modal Size Variants
    // ============================================================

    mod.widgets.MpModalSmall = mod.widgets.MpModal{
        width: 320
    }

    mod.widgets.MpModalLarge = mod.widgets.MpModal{
        width: 560
    }

    mod.widgets.MpModalFullWidth = mod.widgets.MpModal{
        width: Fill
        margin: 24
    }

    // ============================================================
    // Alert Dialog (simple confirmation)
    // ============================================================

    mod.widgets.MpAlertDialog = mod.widgets.MpModal{
        width: 360

        header := View{
            width: Fill
            height: Fit
            padding: Inset{left: 24, right: 24, top: 24, bottom: 12}
            align: Align{x: 0.5}

            title := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: theme.font_bold{font_size: 18.0}
                    color: FOREGROUND
                }
                text: "Are you sure?"
            }
        }

        body := View{
            width: Fill
            height: Fit
            padding: Inset{left: 24, right: 24, top: 0, bottom: 20}
            align: Align{x: 0.5}

            Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 14.0}
                    color: MUTED_FOREGROUND
                }
                text: "This action cannot be undone."
            }
        }

        footer := View{
            width: Fill
            height: Fit
            padding: Inset{left: 24, right: 24, top: 0, bottom: 24}
            flow: Right
            spacing: 12
            align: Align{x: 0.5, y: 0.5}
        }
    }

    // ============================================================
    // Danger Alert Dialog
    // ============================================================

    mod.widgets.MpAlertDialogDanger = mod.widgets.MpAlertDialog{
        header := View{
            width: Fill
            height: Fit
            padding: Inset{left: 24, right: 24, top: 24, bottom: 12}
            flow: Down
            spacing: 12
            align: Align{x: 0.5}

            icon := View{
                width: 48
                height: 48
                align: Align{x: 0.5, y: 0.5}

                show_bg: true
                draw_bg +: {
                    bg_color: instance(#xfef2f2)
                    icon_color: instance(DANGER)

                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        let c = self.rect_size * 0.5
                        let r = min(c.x, c.y)

                        // Circle background
                        sdf.circle(c.x, c.y, r)
                        sdf.fill(self.bg_color)

                        // Warning icon (triangle with !)
                        let size = 12.0
                        sdf.move_to(c.x, c.y - size + 4.0)
                        sdf.line_to(c.x + size - 2.0, c.y + size - 6.0)
                        sdf.line_to(c.x - size + 2.0, c.y + size - 6.0)
                        sdf.close_path()
                        sdf.stroke(self.icon_color, 2.0)

                        // Exclamation mark
                        sdf.rect(c.x - 1.0, c.y - 4.0, 2.0, 6.0)
                        sdf.fill(self.icon_color)
                        sdf.circle(c.x, c.y + 6.0, 1.5)
                        sdf.fill(self.icon_color)

                        return sdf.result
                    }
                }
            }

            title := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: theme.font_bold{font_size: 18.0}
                    color: FOREGROUND
                }
                text: "Delete item?"
            }
        }
    }

    // ============================================================
    // Modal header/footer components
    // ============================================================

    mod.widgets.MpModalHeader = View{
        width: Fill
        height: Fit
        padding: Inset{left: 24, right: 24, top: 20, bottom: 16}
        flow: Right
        align: Align{y: 0.5}
    }

    mod.widgets.MpModalBody = View{
        width: Fill
        height: Fit
        padding: Inset{left: 24, right: 24, top: 0, bottom: 16}
        flow: Down
        spacing: 8
    }

    mod.widgets.MpModalFooter = View{
        width: Fill
        height: Fit
        padding: Inset{left: 24, right: 24, top: 16, bottom: 20}
        flow: Right
        spacing: 8
        align: Align{x: 1.0, y: 0.5}
    }

    // Divider for modal sections
    mod.widgets.MpModalDivider = View{
        width: Fill
        height: 1
        show_bg: true
        draw_bg +: {
            color: instance(BORDER)
            pixel: fn() {
                return self.color
            }
        }
    }

    // ============================================================
    // Interactive Modal Widget
    // ============================================================

    mod.widgets.MpModalWidgetBase = #(MpModalWidget::register_widget(vm))
    mod.widgets.MpModalWidget = set_type_default() do mod.widgets.MpModalWidgetBase{
        width: Fill
        height: Fill
        flow: Overlay
        visible: false

        backdrop := View{
            width: Fill
            height: Fill
            show_bg: true
            draw_bg +: { color: instance(#x00000080) }
        }

        content := View{
            width: Fill
            height: Fill
            align: Align{x: 0.5, y: 0.5}

            dialog := mod.widgets.MpModal{}
        }
    }
}

/// Modal actions
#[derive(Clone, Debug, Default)]
pub enum MpModalAction {
    #[default]
    None,
    Opened,
    Closed,
    CloseRequested,
}

/// Interactive modal widget with open/close functionality
#[derive(Script, ScriptHook, Widget)]
pub struct MpModalWidget {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live(false)]
    #[visible]
    visible: bool,
}

impl Widget for MpModalWidget {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if !self.visible {
            return;
        }

        self.view.handle_event(cx, event, _scope);

        let uid = self.widget_uid();

        // Handle backdrop click to close
        let backdrop = self.view(cx, ids!(backdrop));
        if let Hit::FingerUp(fe) = event.hits(cx, backdrop.area()) {
            if fe.is_over {
                cx.widget_action(uid, MpModalAction::CloseRequested);
            }
        }

        // Handle close button click with animator
        let close_btn = self.view(cx, ids!(content.dialog.header.close));
        match event.hits(cx, close_btn.area()) {
            Hit::FingerHoverIn(_) => {
                close_btn.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                close_btn.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    cx.widget_action(uid, MpModalAction::CloseRequested);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpModalWidget {
    /// Open the modal
    pub fn open(&mut self, cx: &mut Cx) {
        self.visible = true;
        self.redraw(cx);
    }

    /// Close the modal
    pub fn close(&mut self, cx: &mut Cx) {
        self.visible = false;
        self.redraw(cx);
    }

    /// Check if modal is visible
    pub fn is_open(&self) -> bool {
        self.visible
    }

    /// Set the modal title
    pub fn set_title(&mut self, cx: &mut Cx, title: &str) {
        self.view
            .label(cx, ids!(content.dialog.header.title))
            .set_text(cx, title);
    }
}

impl MpModalWidgetRef {
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

    pub fn is_open(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_open()
        } else {
            false
        }
    }

    pub fn set_title(&self, cx: &mut Cx, title: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_title(cx, title);
        }
    }

    pub fn close_requested(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            if let Some(item) = actions.find_widget_action(inner.widget_uid()) {
                return matches!(item.cast::<MpModalAction>(), MpModalAction::CloseRequested);
            }
        }
        false
    }
}
