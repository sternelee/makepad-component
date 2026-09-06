use crate::widgets::icon::MpIconWidgetExt;
use crate::widgets::sizing::MpSize;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpSidebar - app navigation sidebar (gpui Sidebar port).
    // Header/body/footer slots; collapses to an icon rail (Icon mode)
    // or out of layout entirely (Offcanvas).
    // ============================================================

    mod.widgets.MpSidebarCollapsible = #(MpSidebarCollapsible::script_api(vm))

    // Navigation item: icon + label with hover/active states.
    mod.widgets.MpSidebarItemBase = #(MpSidebarItem::register_widget(vm))
    mod.widgets.MpSidebarItem = set_type_default() do mod.widgets.MpSidebarItemBase{
        width: Fill
        height: 32
        padding: Inset{left: 8, right: 8, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}
        flow: Right
        spacing: 8
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x0000)
            bg_hover: instance(ELEMENT_HOVER)
            bg_active: instance(ELEMENT_ACTIVE)
            hover: instance(0.0)
            active: instance(0.0)
            disabled: instance(0.0)
            radius: instance(6.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                let final_bg = mix(bg, self.bg_active, self.active)
                sdf.box(2.0, 1.0, self.rect_size.x - 4.0, self.rect_size.y - 2.0, self.radius)
                sdf.fill(final_bg * mix(1.0, 0.45, self.disabled))
                return sdf.result
            }
        }

        icon := mod.widgets.MpIcon{ name: "circle" }

        label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT
                color_disabled: instance(TEXT_MUTED)
                disabled: instance(0.0)

                get_color: fn() { return mix(self.color, self.color_disabled, self.disabled) }
            }
            text: ""
        }

        disabled: false

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 1.0}} }
            }
            disabled_t: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.1}} redraw: true apply: {draw_bg: {disabled: 0.0} label: {disabled: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.1}} redraw: true apply: {draw_bg: {disabled: 1.0} label: {disabled: 1.0}} }
            }
            active: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.12}} redraw: true apply: {draw_bg: {active: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.12}} redraw: true apply: {draw_bg: {active: 1.0}} }
            }
        }
    }

    // Group: muted section title + item stack (pure DSL, no Rust state).
    mod.widgets.MpSidebarGroup = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 2

        group_label := Label{
            width: Fill, height: Fit
            padding: Inset{left: 10, right: 8, top: 6, bottom: 2}
            draw_text +: {
                text_style: theme.font_regular{font_size: 11.0}
                color: TEXT_MUTED
            }
            text: ""
        }

        content := View{
            width: Fill, height: Fit
            flow: Down
            spacing: 2
        }
    }

    // Sidebar container: header / body / footer slots.
    mod.widgets.MpSidebarBase = #(MpSidebar::register_widget(vm))
    mod.widgets.MpSidebar = set_type_default() do mod.widgets.MpSidebarBase{
        width: 240
        height: Fill
        flow: Down
        spacing: 8
        padding: Inset{left: 8, right: 8, top: 10, bottom: 10}

        show_bg: true
        draw_bg +: {
            color: SURFACE_CARD
            border_color: BORDER
            border_width: instance(1.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y)
                sdf.fill(self.color)
                // Right hairline separator
                sdf.rect(self.rect_size.x - self.border_width, 0.0, self.border_width, self.rect_size.y)
                sdf.fill(self.border_color)
                return sdf.result
            }
        }

        header := View{
            width: Fill, height: Fit
            flow: Down
        }

        body := View{
            width: Fill, height: Fill
            flow: Down
            spacing: 2
        }

        footer := View{
            width: Fill, height: Fit
            flow: Down
        }
    }
}

/// Sidebar item actions
#[derive(Clone, Debug, Default)]
pub enum MpSidebarItemAction {
    /// The item was clicked.
    Clicked,
    #[default]
    None,
}

/// Sidebar container actions
#[derive(Clone, Debug, Default)]
pub enum MpSidebarAction {
    /// Collapse state changed after `toggle`/`set_collapsed`.
    CollapsedChanged(bool),
    #[default]
    None,
}

/// Collapse behavior of the sidebar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum MpSidebarCollapsible {
    /// Collapse to an icon rail (labels hidden, square items).
    #[default]
    Icon,
    /// Collapse completely out of the layout (width 0, content hidden).
    Offcanvas,
    /// Collapse disabled; `collapsed` is ignored.
    None,
}

fn sidebar_item_height(size: MpSize) -> f64 {
    match size {
        MpSize::XSmall => 26.0,
        MpSize::Small => 30.0,
        MpSize::Medium => 32.0,
        MpSize::Large => 38.0,
        MpSize::XLarge => 44.0,
    }
}

/// Width of the collapsed icon rail.
const SIDEBAR_RAIL_WIDTH: f64 = 56.0;

/// Navigation item: icon + label, hover/active states, optional disabled.
#[derive(Script, ScriptHook, Widget)]
pub struct MpSidebarItem {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,

    /// Active (current page) state.
    #[live]
    active: bool,

    #[live]
    disabled: bool,

    /// Compact (icon-rail) mode: square item, label hidden.
    #[live]
    compact: bool,

    /// Five-step size driving the row height and label font.
    #[live]
    size: MpSize,

    #[rust]
    applied_size: Option<MpSize>,
    #[rust]
    applied_compact: Option<bool>,
}

impl Widget for MpSidebarItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        let uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        if self.disabled {
            return;
        }

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                cx.set_cursor(MouseCursor::Default);
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe)
                if fe.is_over => {
                    cx.widget_action(uid, MpSidebarItemAction::Clicked);
                }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let mut walk = walk;
        walk.height = Size::Fixed(sidebar_item_height(self.size));

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut label) = self.view.label(cx, ids!(label)).borrow_mut() {
                label.draw_text.text_style.font_size = self.size.font_size();
            }
        }

        if self.applied_compact != Some(self.compact) {
            self.applied_compact = Some(self.compact);
            self.view.widget(cx, ids!(label)).set_visible(cx, !self.compact);
            self.view
                .widget(cx, ids!(icon))
                .set_visible(cx, true);
        }

        self.view.draw_walk(cx, _scope, walk)
    }
}

impl MpSidebarItem {
    /// Set the item label.
    pub fn set_label(&mut self, cx: &mut Cx, label: &str) {
        self.view.label(cx, ids!(label)).set_text(cx, label);
    }

    /// Set the icon by catalog name (see `MpIcon`). The icon can also be
    /// set in DSL via `icon +: { name: "search" }`.
    pub fn set_icon(&mut self, cx: &mut Cx, name: &str) {
        self.view.mp_icon(cx, ids!(icon)).set_name(cx, name);
    }

    pub fn set_active(&mut self, cx: &mut Cx, active: bool) {
        if self.active != active {
            self.active = active;
            self.animator_toggle(
                cx,
                active,
                Animate::Yes,
                ids!(active.on),
                ids!(active.off),
            );
            self.redraw(cx);
        }
    }

    pub fn set_disabled(&mut self, cx: &mut Cx, disabled: bool) {
        if self.disabled != disabled {
            self.disabled = disabled;
            self.animator_toggle(
                cx,
                disabled,
                Animate::Yes,
                ids!(disabled_t.on),
                ids!(disabled_t.off),
            );
            self.redraw(cx);
        }
    }

    /// Compact (icon-rail) mode: square item, label hidden, icon centered.
    pub fn set_compact(&mut self, cx: &mut Cx, compact: bool) {
        if self.compact != compact {
            self.compact = compact;
            self.applied_compact = None;
            self.redraw(cx);
        }
    }
}

impl MpSidebarItemRef {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            return actions.iter().any(|a| {
                a.downcast_ref::<WidgetAction>().is_some_and(|item| {
                    item.widget_uid == inner.view.widget_uid()
                        && matches!(
                            item.action.downcast_ref::<MpSidebarItemAction>(),
                            Some(MpSidebarItemAction::Clicked)
                        )
                })
            });
        }
        false
    }

    pub fn set_label(&self, cx: &mut Cx, label: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_label(cx, label);
        }
    }

    pub fn set_icon(&self, cx: &mut Cx, name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_icon(cx, name);
        }
    }

    pub fn set_active(&self, cx: &mut Cx, active: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_active(cx, active);
        }
    }
}

/// Sidebar container with header/body/footer slots and collapse modes.
#[derive(Script, ScriptHook, Widget)]
pub struct MpSidebar {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    /// Collapse behavior (icon rail / offcanvas / none).
    #[live]
    collapsible: MpSidebarCollapsible,

    /// Whether the sidebar is currently collapsed.
    #[live]
    collapsed: bool,

    /// Width when expanded.
    #[live(240.0)]
    expanded_width: f64,

    #[rust]
    applied_layout: Option<(bool, MpSidebarCollapsible, f64)>,
}

impl Widget for MpSidebar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let effective = self.collapsed && self.collapsible != MpSidebarCollapsible::None;
        let key = (effective, self.collapsible, self.expanded_width);
        if self.applied_layout != Some(key) {
            self.applied_layout = Some(key);
            self.apply_layout(cx, effective);
        }

        let mut walk = walk;
        walk.width = match self.collapsible {
            MpSidebarCollapsible::None => walk.width,
            _ => {
                if effective {
                    match self.collapsible {
                        MpSidebarCollapsible::Offcanvas => Size::Fixed(0.0),
                        _ => Size::Fixed(SIDEBAR_RAIL_WIDTH),
                    }
                } else {
                    Size::Fixed(self.expanded_width)
                }
            }
        };
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpSidebar {
    fn apply_layout(&mut self, cx: &mut Cx, effective: bool) {
        // Icon-rail mode hides text throughout; offcanvas hides everything.
        let hide_all = effective && self.collapsible == MpSidebarCollapsible::Offcanvas;
        self.view.set_visible(cx, !hide_all);
        if effective && self.collapsible == MpSidebarCollapsible::Icon {
            Self::set_items_compact(&self.view.widget(cx, ids!(body)), cx, true);
        } else {
            Self::set_items_compact(&self.view.widget(cx, ids!(body)), cx, false);
        }
    }

    /// Recursively set compact mode on nested sidebar items (groups wrap
    /// their items in sub-views).
    fn set_items_compact(widget: &WidgetRef, cx: &mut Cx, compact: bool) {
        widget.children(&mut |_id, child| {
            if let Some(mut item) = child.borrow_mut::<MpSidebarItem>() {
                item.set_compact(cx, compact);
            } else {
                Self::set_items_compact(&child, cx, compact);
            }
        });
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn set_collapsed(&mut self, cx: &mut Cx, collapsed: bool) {
        if self.collapsed != collapsed {
            self.collapsed = collapsed;
            self.applied_layout = None;
            self.redraw(cx);
            cx.widget_action(
                self.widget_uid(),
                MpSidebarAction::CollapsedChanged(collapsed),
            );
        }
    }

    pub fn toggle(&mut self, cx: &mut Cx) {
        self.set_collapsed(cx, !self.collapsed);
    }
}

impl MpSidebarRef {
    pub fn is_collapsed(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_collapsed()
        } else {
            false
        }
    }

    pub fn set_collapsed(&self, cx: &mut Cx, collapsed: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_collapsed(cx, collapsed);
        }
    }

    pub fn toggle(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.toggle(cx);
        }
    }
}
