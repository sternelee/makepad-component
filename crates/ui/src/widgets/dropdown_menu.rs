use makepad_widgets::*;
use crate::widgets::MpContextMenuItemWidgetExt;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpDropdownMenu - trigger button + item panel
    // (items reuse mod.widgets.MpContextMenuItem)
    // ============================================================

    mod.widgets.MpDropdownMenu = set_type_default() do #(MpDropdownMenu::register_widget(vm)){
        width: Fit
        height: Fit
        flow: Down

        trigger := mod.widgets.MpButtonGhost{
            text: "Menu"
        }

        panel := View{
            width: 180
            height: Fit
            flow: Down
            spacing: 0
            padding: Inset{top: 4, right: 4, bottom: 4, left: 4}
            visible: false

            show_bg: true
            draw_bg +: {
                bg_color: instance(SURFACE_OVERLAY)
                border_color: instance(BORDER)
                radius: instance(12.0)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                    sdf.fill_keep(self.bg_color)
                    sdf.stroke(self.border_color, 1.0)
                    return sdf.result
                }
            }

            item0 := mod.widgets.MpContextMenuItem{}
            item1 := mod.widgets.MpContextMenuItem{}
            item2 := mod.widgets.MpContextMenuItem{}
            item3 := mod.widgets.MpContextMenuItem{}
            item4 := mod.widgets.MpContextMenuItem{}
            item5 := mod.widgets.MpContextMenuItem{}
            item6 := mod.widgets.MpContextMenuItem{}
            item7 := mod.widgets.MpContextMenuItem{}
        }
    }
}

pub const DROPDOWN_MENU_SLOTS: usize = 8;

#[derive(Clone, Debug, Default)]
pub enum MpDropdownMenuAction {
    Selected(String),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpDropdownMenu {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[rust]
    open: bool,
}

impl Widget for MpDropdownMenu {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            Hit::FingerDown(_) => {
                self.open = !self.open;
                self.apply_items(cx);
                self.view.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpDropdownMenu {
    fn apply_items(&mut self, cx: &mut Cx) {
        for i in 0..DROPDOWN_MENU_SLOTS {
            let key = [LiveId::from_str(&format!("item{}", i))];
            self.view
                .mp_context_menu_item(cx, &key)
                .set_visible(cx, self.open);
        }
        self.view
            .view(cx, ids!(panel))
            .set_visible(cx, self.open);
    }

    pub fn set_items(&mut self, cx: &mut Cx, items: &[String]) {
        for (i, label) in items.iter().enumerate().take(DROPDOWN_MENU_SLOTS) {
            let key = [LiveId::from_str(&format!("item{}", i))];
            self.view.mp_context_menu_item(cx, &key).set_label(label);
        }
        self.apply_items(cx);
    }

    pub fn set_trigger_label(&mut self, cx: &mut Cx, text: &str) {
        self.view.button(cx, ids!(trigger)).set_text(cx, text);
    }
}

impl WidgetMatchEvent for MpDropdownMenu {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for i in 0..DROPDOWN_MENU_SLOTS {
            let key = [LiveId::from_str(&format!("item{}", i))];
            if let Some(selected) = self
                .view
                .mp_context_menu_item(cx, &key)
                .item_selected(actions)
            {
                self.open = false;
                self.apply_items(cx);
                self.view.redraw(cx);
                cx.widget_action(self.widget_uid(), MpDropdownMenuAction::Selected(selected));
                return;
            }
        }
    }
}

impl MpDropdownMenuRef {
    pub fn set_items(&self, cx: &mut Cx, items: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn set_trigger_label(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_trigger_label(cx, text);
        }
    }

    pub fn item_selected(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpDropdownMenuAction::Selected(s) = action.cast() {
                    return Some(s);
                }
            }
        }
        None
    }
}
