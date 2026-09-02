use makepad_widgets::*;
use crate::widgets::MpContextMenuItemWidgetExt;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpMenuBar - horizontal menu bar with dropdown panels
    // ============================================================

    // Menu title on the strip (bezel menubar_title: px8/py3, r6, t13,
    // muted -> text on hover)
    mod.widgets.MpMenuTitle = mod.widgets.MpButtonGhost{
        height: 24
        padding: Inset{left: 8, right: 8, top: 3, bottom: 3}
        draw_bg +: {
            radius: 6.0
        }
        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT_MUTED
        }
    }

    mod.widgets.MpMenuBar = set_type_default() do #(MpMenuBar::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Down
        spacing: 0

        bar := View{
            width: Fill
            height: 32
            flow: Right
            spacing: 2
            padding: Inset{left: 4, right: 4, top: 0, bottom: 0}
            align: Align{x: 0.0, y: 0.5}

            show_bg: true
            draw_bg +: {
                bg_color: instance(SURFACE_RAISED)
                border_color: instance(BORDER)
                radius: instance(8.0)
                border_width: instance(1.0)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                    sdf.fill_keep(self.bg_color)
                    sdf.stroke(self.border_color, self.border_width)
                    return sdf.result
                }
            }

            t0 := mod.widgets.MpMenuTitle{ width: Fit, text: "File" }
            t1 := mod.widgets.MpMenuTitle{ width: Fit, text: "Edit" }
            t2 := mod.widgets.MpMenuTitle{ width: Fit, text: "View" }
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

            i0 := mod.widgets.MpContextMenuItem{}
            i1 := mod.widgets.MpContextMenuItem{}
            i2 := mod.widgets.MpContextMenuItem{}
            i3 := mod.widgets.MpContextMenuItem{}
            i4 := mod.widgets.MpContextMenuItem{}
            i5 := mod.widgets.MpContextMenuItem{}
            i6 := mod.widgets.MpContextMenuItem{}
            i7 := mod.widgets.MpContextMenuItem{}
        }
    }
}

pub const MENUBAR_SLOTS: usize = 8;
pub const MENUBAR_TRIGGERS: usize = 3;

#[derive(Clone, Debug, Default)]
pub enum MpMenuBarAction {
    Selected(usize, String),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpMenuBar {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[rust]
    open_menu: Option<usize>,
}

impl Widget for MpMenuBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpMenuBar {
    fn apply_items(&mut self, cx: &mut Cx) {
        let open = self.open_menu;
        self.view
            .view(cx, ids!(panel))
            .set_visible(cx, open.is_some());
        self.view.redraw(cx);
    }
}

impl WidgetMatchEvent for MpMenuBar {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for t in 0..MENUBAR_TRIGGERS {
            let key = [LiveId::from_str(&format!("t{}", t))];
            if self.view.button(cx, &key).clicked(actions) {
                self.open_menu = if self.open_menu == Some(t) { None } else { Some(t) };
                self.apply_items(cx);
            }
        }

        if let Some(open) = self.open_menu {
            for i in 0..MENUBAR_SLOTS {
                let key = [LiveId::from_str(&format!("i{}", i))];
                if let Some(selected) = self.view.mp_context_menu_item(cx, &key).item_selected(actions)
                {
                    self.open_menu = None;
                    self.apply_items(cx);
                    cx.widget_action(
                        self.widget_uid(),
                        MpMenuBarAction::Selected(open, selected),
                    );
                    return;
                }
            }
        }
    }
}

impl MpMenuBar {
    pub fn set_trigger_labels(&mut self, cx: &mut Cx, labels: &[String]) {
        for (i, label) in labels.iter().enumerate() {
            if i >= MENUBAR_TRIGGERS {
                break;
            }
            let key = [LiveId::from_str(&format!("t{}", i))];
            self.view.button(cx, &key).set_text(cx, label);
        }
    }

    pub fn set_items(&mut self, cx: &mut Cx, menu_index: usize, items: &[String]) {
        // Set the item labels for the given menu index.
        // This demo uses a single shared panel; the caller supplies labels.
        let mut shown = 0;
        for item in items {
            if shown >= MENUBAR_SLOTS {
                break;
            }
            let key = [LiveId::from_str(&format!("i{}", shown))];
            let it = self.view.mp_context_menu_item(cx, &key);
            it.set_label(item);
            it.set_visible(cx, true);
            shown += 1;
        }
        for i in shown..MENUBAR_SLOTS {
            let key = [LiveId::from_str(&format!("i{}", i))];
            self.view
                .mp_context_menu_item(cx, &key)
                .set_visible(cx, false);
        }
        self.apply_items(cx);
    }
}

impl MpMenuBarRef {
    pub fn set_trigger_labels(&self, cx: &mut Cx, labels: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_trigger_labels(cx, labels);
        }
    }

    pub fn set_items(&self, cx: &mut Cx, menu_index: usize, items: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, menu_index, items);
        }
    }

    pub fn selected(&self, actions: &Actions) -> Option<(usize, String)> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpMenuBarAction::Selected(menu, item) = action.cast() {
                    return Some((menu, item));
                }
            }
        }
        None
    }
}
