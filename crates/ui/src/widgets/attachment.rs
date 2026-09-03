use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpAttachment - file attachment chip (gpui Attachment, simplified)
    // ============================================================

    mod.widgets.MpAttachment = set_type_default() do #(MpAttachment::register_widget(vm)){
        width: Fit
        height: Fit
        flow: Right
        spacing: 8.0
        align: Align{y: 0.5}
        padding: Inset{left: 10.0, right: 10.0, top: 8.0, bottom: 8.0}

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_RAISED)
            border_color: instance(BORDER)
            border_width: instance(1.0)
            radius: instance(8.0)
            hover: instance(0.0)
            c_hover: instance(ELEMENT_HOVER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                let bg = mix(self.bg_color, self.c_hover, self.hover)
                sdf.fill_keep(bg)
                sdf.stroke(self.border_color, self.border_width)
                return sdf.result
            }
        }

        ext_badge := View{
            width: 34.0
            height: 34.0
            align: Align{x: 0.5, y: 0.5}

            show_bg: true
            draw_bg +: {
                bg_color: instance(ACCENT_MUTED)
                radius: instance(6.0)
            }

            ext_label := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: theme.font_bold{font_size: 9.0}
                    color: ON_ACCENT
                }
                text: "TXT"
            }
        }

        file_info := View{
            width: Fit
            height: Fit
            flow: Down
            spacing: 2.0

            file_name := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 13.0}
                    color: TEXT
                }
                text: ""
            }

            file_meta := Label{
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: theme.font_regular{font_size: 11.0}
                    color: TEXT_MUTED
                }
                text: ""
            }
        }

        remove := mod.widgets.MpButtonGhost{
            width: 20
            height: 20
            text: "×"
            draw_text +: { text_style: theme.font_regular{font_size: 11.0} }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpAttachmentAction {
    Removed(String),
    #[default]
    None,
}

/// Map a filename to its upper-cased extension badge text (max 4 chars).
fn ext_badge_text(filename: &str) -> String {
    filename
        .rsplit('.')
        .next()
        .filter(|ext| *ext != filename)
        .map(|ext| ext.to_uppercase())
        .map(|ext| ext.chars().take(4).collect())
        .unwrap_or_else(|| "FILE".to_string())
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpAttachment {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// The attachment file name (drives the extension badge).
    #[live]
    filename: ArcStringMut,

    /// Optional human-readable meta line (size, date...).
    #[live]
    meta: ArcStringMut,

    /// Five-step size driving fonts and padding.
    #[live]
    size: MpSize,

    /// Last size applied (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpAttachment {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Metrics from the size system (Medium = the DSL 10/8 look)
        self.view.layout.padding = Inset {
            left: self.size.padding_h() * 0.83,
            right: self.size.padding_h() * 0.83,
            top: self.size.padding_v() * 1.33,
            bottom: self.size.padding_v() * 1.33,
        };

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut badge) = self.view.view(cx, ids!(ext_badge)).borrow_mut() {
                let badge_size = self.size.icon_size() + 18.0;
                badge.walk.width = Size::Fixed(badge_size);
                badge.walk.height = Size::Fixed(badge_size);
            }
            if let Some(mut ext) = self.view.label(cx, ids!(ext_badge.ext_label)).borrow_mut() {
                ext.draw_text.text_style.font_size = self.size.font_size() - 4.0;
            }
            if let Some(mut name) = self.view.label(cx, ids!(file_info.file_name)).borrow_mut() {
                name.draw_text.text_style.font_size = self.size.font_size();
            }
            if let Some(mut meta) = self.view.label(cx, ids!(file_info.file_meta)).borrow_mut() {
                meta.draw_text.text_style.font_size = self.size.font_size() - 2.0;
            }
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MpAttachment {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.view.button(cx, ids!(remove)).clicked(actions) {
            let name = self.filename.as_ref().to_string();
            cx.widget_action(self.widget_uid(), MpAttachmentAction::Removed(name));
        }
    }
}

impl MpAttachment {
    /// Set the file name and refresh the extension badge.
    pub fn set_filename(&mut self, cx: &mut Cx, filename: &str) {
        self.filename.as_mut_empty().push_str(filename);
        self.view
            .label(cx, ids!(file_info.file_name))
            .set_text(cx, filename);
        self.view
            .label(cx, ids!(ext_badge.ext_label))
            .set_text(cx, &ext_badge_text(filename));
        self.redraw(cx);
    }

    /// Set the meta line (size, date...).
    pub fn set_meta(&mut self, cx: &mut Cx, meta: &str) {
        self.meta.as_mut_empty().push_str(meta);
        self.view
            .label(cx, ids!(file_info.file_meta))
            .set_text(cx, meta);
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
}

impl MpAttachmentRef {
    pub fn set_filename(&self, cx: &mut Cx, filename: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_filename(cx, filename);
        }
    }

    pub fn set_meta(&self, cx: &mut Cx, meta: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_meta(cx, meta);
        }
    }

    pub fn removed(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpAttachmentAction::Removed(s) = action.cast() {
                    return Some(s);
                }
            }
        }
        None
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
