use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpToggleGroup - segmented single-select control
    // ============================================================

    mod.widgets.MpToggleGroupItem = set_type_default() do #(MpToggleGroupItem::register_widget(vm)){
        width: Fit
        height: Fit
        padding: Inset{left: 10.0, right: 10.0, top: 4.0, bottom: 4.0}

        draw_bg +: {
            bg_active: instance(SOLID)
            bg_inactive: instance(#x0000)
            hover_wash: instance(ELEMENT_HOVER)
            active: instance(0.0)
            hovered: instance(0.0)
            radius: instance(6.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                let quiet = mix(self.bg_inactive, self.hover_wash, self.hovered)
                sdf.fill(mix(quiet, self.bg_active, self.active))
                return sdf.result
            }
        }

        draw_text +: {
            text_style: theme.font_regular{font_size: 12.5}
            color: TEXT_MUTED
            color_active: instance(ON_SOLID)
            active: instance(0.0)
            get_color: fn() {
                return mix(self.color, self.color_active, self.active)
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hovered: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hovered: 1.0}} }
            }
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {active: 0.0} draw_text: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {active: 1.0} draw_text: {active: 1.0}}
                }
            }
        }
    }

    mod.widgets.MpToggleGroup = set_type_default() do #(MpToggleGroup::register_widget(vm)){
        width: Fit
        height: Fit
        flow: Right
        spacing: 4.0
        padding: Inset{left: 4.0, right: 4.0, top: 4.0, bottom: 4.0}

        draw_bg +: {
            bg_color: instance(INPUT_BG)
            border_color: instance(BORDER)
            border_width: instance(0.0)
            radius: instance(8.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.radius
                )
                sdf.fill(self.bg_color)
                if (self.border_width > 0.0) {
                    sdf.stroke(self.border_color, self.border_width)
                }
                return sdf.result
            }
        }

        i0 := mod.widgets.MpToggleGroupItem{text: "One"}
        i1 := mod.widgets.MpToggleGroupItem{text: "Two"}
        i2 := mod.widgets.MpToggleGroupItem{text: "Three"}
        i3 := mod.widgets.MpToggleGroupItem{visible: false}
        i4 := mod.widgets.MpToggleGroupItem{visible: false}
        i5 := mod.widgets.MpToggleGroupItem{visible: false}
        i6 := mod.widgets.MpToggleGroupItem{visible: false}
        i7 := mod.widgets.MpToggleGroupItem{visible: false}
    }
}

pub const TOGGLE_GROUP_SLOTS: usize = 8;

#[derive(Clone, Debug, Default)]
pub enum MpToggleGroupAction {
    Changed(usize),
    #[default]
    None,
}

#[derive(Clone, Debug, Default)]
pub enum MpToggleGroupItemAction {
    Clicked(usize),
    #[default]
    None,
}

#[derive(Script, Widget, Animator)]
pub struct MpToggleGroupItem {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[live]
    draw_text: DrawText,
    #[apply_default]
    animator: Animator,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    text: ArcStringMut,
    #[live(true)]
    visible: bool,

    /// Five-step size metrics (padding + font) for this segmented item.
    #[live]
    size: MpSize,

    #[rust]
    area: Area,
    #[rust]
    index: usize,
}

impl ScriptHook for MpToggleGroupItem {}

impl Widget for MpToggleGroupItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
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
            Hit::FingerUp(fe) if fe.is_over && fe.was_tap() => {
                cx.widget_action(self.uid, MpToggleGroupItemAction::Clicked(self.index));
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        // Metrics from the size system (Medium ≈ the original DSL look)
        self.layout.padding = Inset {
            left: self.size.padding_h(),
            right: self.size.padding_h(),
            top: self.size.padding_v(),
            bottom: self.size.padding_v(),
        };
        self.draw_text.text_style.font_size = self.size.font_size();
        self.draw_bg.begin(cx, walk, self.layout);
        let label = self.text.as_ref().to_string();
        self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), &label);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpToggleGroupItem {
    pub fn set_active(&mut self, cx: &mut Cx, active: bool) {
        if active {
            self.animator_play(cx, ids!(active.on));
        } else {
            self.animator_play(cx, ids!(active.off));
        }
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpToggleGroup {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[rust]
    selected: Option<usize>,

    /// Five-step size system propagated to the item slots.
    #[live]
    size: MpSize,

    /// Last size applied to the items (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

impl Widget for MpToggleGroup {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Propagate the size to the item slots once per size change
        // (guarded: item.set_size requests redraws, so don't do it per frame).
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            for i in 0..TOGGLE_GROUP_SLOTS {
                let key = [LiveId::from_str(&format!("i{}", i))];
                if let Some(mut item) = self.view.mp_toggle_group_item(cx, &key).borrow_mut() {
                    item.set_size(cx, self.size);
                }
            }
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MpToggleGroup {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for i in 0..TOGGLE_GROUP_SLOTS {
            let key = [LiveId::from_str(&format!("i{}", i))];
            let clicked = self
                .view
                .mp_toggle_group_item(cx, &key)
                .borrow()
                .map(|inner| inner.widget_uid())
                .and_then(|uid| {
                    actions.find_widget_action(uid).map(|action| {
                        matches!(
                            action.cast::<MpToggleGroupItemAction>(),
                            MpToggleGroupItemAction::Clicked(_)
                        )
                    })
                })
                .unwrap_or(false);

            if clicked {
                self.set_selected_internal(cx, Some(i));
                cx.widget_action(self.widget_uid(), MpToggleGroupAction::Changed(i));
            }
        }
    }
}

impl MpToggleGroup {
    pub fn set_selected_internal(&mut self, cx: &mut Cx, sel: Option<usize>) {
        self.selected = sel;
        for i in 0..TOGGLE_GROUP_SLOTS {
            let key = [LiveId::from_str(&format!("i{}", i))];
            let active = self.selected == Some(i);
            if let Some(mut inner) = self.view.mp_toggle_group_item(cx, &key).borrow_mut() {
                inner.set_active(cx, active);
            }
        }
        self.view.redraw(cx);
    }

    pub fn set_items(&mut self, cx: &mut Cx, items: &[String]) {
        for i in 0..TOGGLE_GROUP_SLOTS {
            let key = [LiveId::from_str(&format!("i{}", i))];
            if let Some(mut inner) = self.view.mp_toggle_group_item(cx, &key).borrow_mut() {
                if i < items.len() {
                    inner.visible = true;
                    inner.text.as_mut_empty().push_str(&items[i]);
                    inner.set_index(i);
                    inner.set_active(cx, self.selected == Some(i));
                } else {
                    inner.visible = false;
                }
                inner.redraw(cx);
            }
        }
        self.view.redraw(cx);
    }

    pub fn set_selected(&mut self, cx: &mut Cx, index: usize) {
        self.set_selected_internal(cx, Some(index));
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Items re-sync on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpToggleGroupRef {
    pub fn set_items(&self, cx: &mut Cx, items: &[String]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn set_selected(&self, cx: &mut Cx, index: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, index);
        }
    }

    pub fn selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(inner) = self.borrow() {
            let uid = inner.widget_uid();
            if let Some(action) = actions.find_widget_action(uid) {
                if let MpToggleGroupAction::Changed(idx) = action.cast() {
                    return Some(idx);
                }
            }
            return inner.selected();
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
