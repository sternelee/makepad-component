use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpCommand - shadcn/cmdk-style command palette
    // ============================================================

    // Command palette backdrop
    mod.widgets.MpCommandBackdrop = View{
        width: Fill
        height: Fill
        show_bg: true
        draw_bg +: {
            color: instance(#x00000000)
            opacity: instance(0.0)
            pixel: fn() { return self.color * self.opacity }
        }
    }

    // Command input field
    mod.widgets.MpCommandInputBase = #(MpCommandInput::register_widget(vm))
    mod.widgets.MpCommandInput = set_type_default() do mod.widgets.MpCommandInputBase{
        width: Fill
        height: 40
        padding: Inset{left: 12, right: 12, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}
        flow: Right
        spacing: 8

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x0000)
            radius: instance(0.0)
            border_color: instance(BORDER)
            cursor_color: instance(ACCENT)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.fill(self.bg_color)
                // Bottom border
                let y = self.rect_size.y - 1.0
                sdf.move_to(0.0, y)
                sdf.line_to(self.rect_size.x, y)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }

        search_icon := View{
            width: 16, height: 16
            show_bg: true
            draw_bg +: {
                icon_color: instance(TEXT_MUTED)
                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    let c = self.rect_size * 0.5
                    let r = 5.0
                    sdf.circle(c.x, c.y - 1.0, r)
                    sdf.stroke(self.icon_color, 1.5)
                    let a = 2.5
                    sdf.move_to(c.x + a, c.y + a)
                    sdf.line_to(c.x + r, c.y + r)
                    sdf.stroke(self.icon_color, 1.5)
                    return sdf.result
                }
            }
        }

        // placeholder stored as Rust field, not DSL property

        animator: Animator{
            focus: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.0}} apply: {} }
                on: AnimatorState{ from: {all: Forward {duration: 0.0}} apply: {} }
            }
        }
    }

    // Command group container
    mod.widgets.MpCommandGroup = View{
        width: Fill
        height: Fit
        flow: Down
    }

    // Command group heading
    mod.widgets.MpCommandGroupHeading = View{
        width: Fill
        height: 28
        padding: Inset{left: 12, right: 12, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 11.0}
                color: TEXT_MUTED
            }
            text: ""
        }
    }

    // Command item
    mod.widgets.MpCommandItemBase = #(MpCommandItem::register_widget(vm))
    mod.widgets.MpCommandItem = set_type_default() do mod.widgets.MpCommandItemBase{
        width: Fill
        height: 40
        padding: Inset{left: 12, right: 12, top: 0, bottom: 0}
        align: Align{x: 0.0, y: 0.5}
        flow: Right
        spacing: 8
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x0000)
            bg_hover: instance(ELEMENT_HOVER)
            bg_selected: instance(ACCENT)
            hover: instance(0.0)
            selected: instance(0.0)
            radius: instance(6.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let bg = mix(self.bg_color, self.bg_hover, self.hover)
                let final_bg = mix(bg, self.bg_selected, self.selected)
                sdf.rect(4.0, 2.0, self.rect_size.x - 8.0, self.rect_size.y - 4.0)
                sdf.fill(final_bg)
                return sdf.result
            }
        }

        label := Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: (TEXT)
                color_selected: (ON_ACCENT)
                selected: instance(0.0)
                get_color: fn() { return mix(self.color, self.color_selected, self.selected) }
            }
            text: ""
        }

        shortcut := Label{
            width: Fit
            height: Fit
            visible: false
            draw_text +: {
                text_style: theme.font_regular{font_size: 12.0}
                color: TEXT_MUTED
            }
            text: ""
        }


        selected: false

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.08}} apply: {draw_bg: {hover: 1.0}} }
            }
            active: {
                default: @off
                off: AnimatorState{ from: {all: Forward {duration: 0.15}} redraw: true apply: {draw_bg: {selected: 0.0} label: {selected: 0.0}} }
                on: AnimatorState{ from: {all: Forward {duration: 0.15}} redraw: true apply: {draw_bg: {selected: 1.0} label: {selected: 1.0}} }
            }
        }
    }

    // Command palette container
    mod.widgets.MpCommandPaletteBase = #(MpCommandPalette::register_widget(vm))
    mod.widgets.MpCommandPalette = set_type_default() do mod.widgets.MpCommandPaletteBase{
        width: Fill
        height: Fill
        flow: Overlay
        visible: false

        backdrop := mod.widgets.MpCommandBackdrop{}

        panel := RoundedView{
            width: 480
            height: Fit
            align: Align{x: 0.5, y: 0.3}
            flow: Down

            draw_bg +: {
                color: SURFACE_CARD
                border_radius: 12.0
                border_color: BORDER
                shadow_color: instance(#x00000044)
                shadow_offset_y: instance(12.0)
                shadow_blur: instance(32.0)
            }

            input := mod.widgets.MpCommandInput{}

            separator := SolidView{ width: Fill, height: 1, draw_bg +: { color: BORDER } }

            list := View{
                width: Fill
                height: Fit
                flow: Down
                padding: Inset{left: 4, right: 4, top: 8, bottom: 8}
            }

            empty := Label{
                width: Fill
                height: 80
                align: Align{x: 0.5, y: 0.5}
                visible: false
                draw_text +: { text_style: theme.font_regular{font_size: 14.0} color: TEXT_MUTED }
                text: "No results found."
            }
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Clone, Debug, Default)]
pub enum MpCommandAction {
    Open,
    Close,
    Selected(String),
    Search(String),
    #[default]
    None,
}

// Command input
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpCommandInput {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
}

impl Widget for MpCommandInput {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

// Command item
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpCommandItem {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,

    #[live]
    selected: bool,

    /// Five-step size driving the row height and label font (Medium = the
    /// DSL 40).
    #[live]
    size: MpSize,

    /// Last size applied to the label (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

/// Command-row height for a size step (Medium = the DSL 40).
fn command_row_height(size: MpSize) -> f64 {
    match size {
        MpSize::XSmall => 32.0,
        MpSize::Small => 36.0,
        MpSize::Medium => 40.0,
        MpSize::Large => 46.0,
        MpSize::XLarge => 52.0,
    }
}

impl Widget for MpCommandItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        let uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe)
                if fe.is_over => {
                    cx.widget_action(uid, MpCommandAction::Selected(String::new()));
                }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Metrics from the size system (Medium = the DSL 40-row look)
        let mut walk = walk;
        walk.height = Size::Fixed(command_row_height(self.size));
        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut label) = self.view.label(cx, ids!(label)).borrow_mut() {
                label.draw_text.text_style.font_size = self.size.font_size();
            }
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpCommandItem {
    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        if self.selected != selected {
            self.selected = selected;
            self.animator_toggle(
                cx,
                selected,
                Animate::Yes,
                ids!(active.on),
                ids!(active.off),
            );
            self.redraw(cx);
        }
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            // Label re-syncs on the next draw_walk.
            self.applied_size = None;
            self.redraw(cx);
        }
    }
}

impl MpCommandItemRef {
    pub fn selected(&self, actions: &Actions) -> Option<String> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpCommandAction::Selected(v) = item.cast() {
                return Some(v);
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

// Command palette
#[derive(Script, ScriptHook, Widget)]
pub struct MpCommandPalette {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live(false)]
    open: bool,

    #[rust]
    highlighted_idx: usize,
}

impl Widget for MpCommandPalette {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.open {
            return;
        }

        self.view.handle_event(cx, event, scope);

        let uid = self.widget_uid();

        // Click backdrop to close
        if let Hit::FingerUp(fe) = event.hits(cx, self.view.view(cx, ids!(backdrop)).area()) {
            if fe.is_over {
                self.close(cx);
                cx.widget_action(uid, MpCommandAction::Close);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.open {
            return DrawStep::done();
        }
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpCommandPalette {
    pub fn open(&mut self, cx: &mut Cx) {
        self.open = true;
        self.redraw(cx);
    }

    pub fn close(&mut self, cx: &mut Cx) {
        self.open = false;
        self.redraw(cx);
    }

    pub fn is_open(&self) -> bool {
        self.open
    }
}

impl MpCommandPaletteRef {
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
}
