use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpContextMenu - right-click menu on a target area
    // ============================================================

    // Menu item row
    mod.widgets.MpContextMenuItemBase = #(MpContextMenuItem::register_widget(vm))
    mod.widgets.MpContextMenuItem = set_type_default() do mod.widgets.MpContextMenuItemBase{
        width: Fill
        height: 28
        visible: true
        padding: Inset{left: 8, right: 8, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}

        draw_bg +: {
            bg_color: instance(#x0000)
            bg_hover: instance(ELEMENT_HOVER)
            bg_danger: instance(DANGER_MUTED)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                sdf.fill(bg)
                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: TEXT
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 1.0}} }
            }
        }
    }

    // Context menu container: content area + anchored popup
    mod.widgets.MpContextMenu = set_type_default() do #(MpContextMenu::register_widget(vm)){
        width: Fill
        height: 120
        flow: Overlay

        content := View{
            width: Fill
            height: Fill
            flow: Down
            align: Align{x: 0.5, y: 0.5}

            show_bg: true
            draw_bg +: {
                color: instance(SURFACE_RAISED)
                border_color: instance(BORDER)
                radius: instance(8.0)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                    sdf.fill_keep(self.color)
                    sdf.stroke(self.border_color, 1.0)
                    return sdf.result
                }
            }

            Label{
                draw_text +: {
                    text_style: theme.font_regular{font_size: 13.0}
                    color: TEXT_MUTED
                }
                text: "Right-click here"
            }
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

pub const CONTEXT_MENU_SLOTS: usize = 8;

#[derive(Clone, Debug, Default)]
pub enum MpContextMenuAction {
    Selected(String),
    #[default]
    None,
}

// ============================================================
// Menu item row
// ============================================================

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpContextMenuItem {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[apply_default]
    animator: Animator,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[live]
    draw_text: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    label: ArcStringMut,
    #[live(true)]
    visible: bool,

    #[rust]
    area: Area,
}

impl Widget for MpContextMenuItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if !self.visible {
            return;
        }
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe) if fe.is_over => {
                cx.widget_action(
                    self.uid,
                    MpContextMenuAction::Selected(self.label.as_ref().to_string()),
                );
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.label.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpContextMenuItem {
    pub fn set_label(&mut self, text: &str) {
        self.label.as_mut_empty().push_str(text);
    }

    pub fn set_visible(&mut self, cx: &mut Cx, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.redraw(cx);
        }
    }
}

impl MpContextMenuItemRef {
    pub fn item_selected(&self, actions: &Actions) -> Option<String> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpContextMenuAction::Selected(v) = item.cast() {
                return Some(v);
            }
        }
        None
    }

    pub fn set_label(&self, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_label(text);
        }
    }

    pub fn set_visible(&self, cx: &mut Cx, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_visible(cx, visible);
        }
    }
}

// ============================================================
// Container
// ============================================================

#[derive(Script, ScriptHook, Widget)]
pub struct MpContextMenu {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[rust]
    items: Vec<String>,

    #[rust]
    open: bool,
}

impl Widget for MpContextMenu {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        match event.hits(cx, self.view.area()) {
            // Any button down toggles the menu (right-click preferred,
            // left-click fallback — makepad finger hits may be primary-only)
            Hit::FingerDown(fe) => {
                let over_item = self.open && self.item_hit(cx, fe.abs);
                if !over_item {
                    self.open = !self.open;
                    self.apply_items(cx);
                    self.view.redraw(cx);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpContextMenu {
    fn item_hit(&self, cx: &mut Cx, pos: DVec2) -> bool {
        for i in 0..CONTEXT_MENU_SLOTS {
            let key = [LiveId::from_str(&format!("item{}", i))];
            if let Some(inner) = self.view.mp_context_menu_item(cx, &key).borrow() {
                if inner.visible && inner.area.rect(cx).contains(pos) {
                    return true;
                }
            }
        }
        false
    }

    fn apply_items(&mut self, cx: &mut Cx) {
        let shown: Vec<&String> = if self.open {
            self.items.iter().take(CONTEXT_MENU_SLOTS).collect()
        } else {
            Vec::new()
        };

        for i in 0..CONTEXT_MENU_SLOTS {
            let key = [LiveId::from_str(&format!("item{}", i))];
            let item = self.view.mp_context_menu_item(cx, &key);
            if let Some(label) = shown.get(i) {
                item.set_visible(cx, true);
                item.set_label(label);
            } else {
                item.set_visible(cx, false);
            }
        }

        self.view
            .view(cx, ids!(panel))
            .set_visible(cx, !shown.is_empty());
    }
}

impl WidgetMatchEvent for MpContextMenu {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Item click -> close + re-emit under our own uid
        for i in 0..CONTEXT_MENU_SLOTS {
            let key = [LiveId::from_str(&format!("item{}", i))];
            if let Some(selected) = self
                .view
                .mp_context_menu_item(cx, &key)
                .item_selected(actions)
            {
                self.open = false;
                self.apply_items(cx);
                cx.widget_action(self.widget_uid(), MpContextMenuAction::Selected(selected));
                return;
            }
        }
    }
}

impl MpContextMenu {
    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<String>) {
        self.items = items;
        self.apply_items(cx);
    }
}

impl MpContextMenuRef {
    pub fn set_items(&self, cx: &mut Cx, items: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn item_selected(&self, actions: &Actions) -> Option<String> {
        if let Some(inner) = self.borrow() {
            if let Some(action) = actions.find_widget_action(inner.widget_uid()) {
                if let MpContextMenuAction::Selected(s) = action.cast() {
                    return Some(s);
                }
            }
        }
        None
    }
}
