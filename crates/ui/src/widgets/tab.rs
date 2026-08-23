use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpTab - Individual tab item (clickable)
    // ============================================================

    // Base tab item - used inside TabBar
    mod.widgets.MpTabBase = #(MpTab::register_widget(vm))
    mod.widgets.MpTab = set_type_default() do mod.widgets.MpTabBase{
        width: Fit
        height: Fit
        align: Align{x: 0.5, y: 0.5}
        padding: Inset{left: 16.0, right: 16.0, top: 8.0, bottom: 8.0}

        text: ""

        draw_bg +: {
            hover: instance(0.0)
            selected: instance(0.0)

            border_radius: uniform(6.0)
            border_width: uniform(1.0)

            color: uniform(TRANSPARENT)
            color_hover: uniform(ELEMENT_HOVER)
            color_selected: uniform(SURFACE_CARD)

            border_color: uniform(TRANSPARENT)
            border_color_selected: uniform(BORDER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Apply selected first, then hover on top
                let bg = mix(self.color, self.color_selected, self.selected)
                let bg_final = mix(bg, self.color_hover, self.hover * (1.0 - self.selected))

                // Calculate box dimensions
                let bw = self.border_width
                let box_w = self.rect_size.x - bw * 2.0
                let box_h = self.rect_size.y - bw * 2.0

                // Clamp radius to half of box height
                let max_r = box_h * 0.5
                let r = min(self.border_radius, max_r)

                sdf.box(bw, bw, box_w, box_h, r)
                sdf.fill_keep(bg_final)

                if (bw > 0.0) {
                    let border = mix(self.border_color, self.border_color_selected, self.selected)
                    sdf.stroke(border, bw)
                }

                return sdf.result
            }
        }

        draw_text +: {
            hover: instance(0.0)
            selected: instance(0.0)

            color: TEXT_FAINT
            color_hover: uniform(TEXT)
            color_selected: uniform(TEXT)

            text_style: theme.font_regular{font_size: 14.0}

            get_color: fn() {
                // Apply selected first, then hover on top (only if not selected)
                let c = mix(self.color, self.color_selected, self.selected)
                return mix(c, self.color_hover, self.hover * (1.0 - self.selected))
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {
                        draw_bg: {hover: 0.0}
                        draw_text: {hover: 0.0}
                    }
                }
                on: AnimatorState{
                    cursor: MouseCursor.Hand
                    from: {all: Forward {duration: 0.1}}
                    apply: {
                        draw_bg: {hover: 1.0}
                        draw_text: {hover: 1.0}
                    }
                }
            }
            selected: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.15}}
                    apply: {
                        draw_bg: {selected: 0.0}
                        draw_text: {selected: 0.0}
                    }
                }
                on: AnimatorState{
                    from: {all: Snap}
                    apply: {
                        draw_bg: {selected: 1.0}
                        draw_text: {selected: 1.0}
                    }
                }
            }
        }
    }

    // ============================================================
    // Tab Variants
    // ============================================================

    // Underline Tab - minimal with bottom indicator
    mod.widgets.MpTabUnderline = mod.widgets.MpTab{
        padding: Inset{left: 12.0, right: 12.0, top: 8.0, bottom: 8.0}

        draw_bg +: {
            indicator_height: uniform(2.0)
            indicator_color: uniform(ACCENT)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Draw underline indicator when selected
                if (self.selected > 0.5) {
                    sdf.rect(
                        0.0,
                        self.rect_size.y - self.indicator_height,
                        self.rect_size.x,
                        self.indicator_height
                    )
                    sdf.fill(self.indicator_color)
                }

                return sdf.result
            }
        }

        draw_text +: {
            color_selected: ACCENT
        }
    }

    // Outline Tab - bordered outline with rounded corners
    mod.widgets.MpTabOutline = mod.widgets.MpTab{
        padding: Inset{left: 16.0, right: 16.0, top: 6.0, bottom: 6.0}

        draw_bg +: {
            color_hover: ELEMENT_HOVER
            color_selected: TRANSPARENT

            border_color: BORDER
            border_color_selected: ACCENT
        }

        draw_text +: {
            color_selected: ACCENT
        }
    }

    // Pill Tab - rounded rect, transparent by default, blue bg + white text when selected
    mod.widgets.MpTabPill = mod.widgets.MpTab{
        padding: Inset{left: 16.0, right: 16.0, top: 8.0, bottom: 8.0}

        draw_bg +: {
            border_radius: 6.0
            border_width: 0.0

            color: TRANSPARENT
            color_hover: ELEMENT_HOVER
            color_selected: ACCENT

            border_color: TRANSPARENT
            border_color_selected: TRANSPARENT
        }

        draw_text +: {
            color: TEXT_FAINT
            color_hover: ON_ACCENT
            color_selected: ON_ACCENT
        }
    }

    // Segmented Tab - iOS style segmented control item
    mod.widgets.MpTabSegmented = mod.widgets.MpTab{
        padding: Inset{left: 16.0, right: 16.0, top: 6.0, bottom: 6.0}

        draw_bg +: {
            border_radius: 4.0
            border_width: 0.0

            color: TRANSPARENT
            color_hover: ELEMENT_HOVER
            color_selected: ON_ACCENT

            border_color: TRANSPARENT
            border_color_selected: TRANSPARENT
        }
    }

    // ============================================================
    // TabBar Containers
    // ============================================================

    // Base TabBar container
    mod.widgets.MpTabBarBase = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Right
        align: Align{y: 0.5}
    }

    // Default TabBar
    mod.widgets.MpTabBar = mod.widgets.MpTabBarBase{
        padding: 4
        spacing: 4
        show_bg: true
        draw_bg +: {
            color: instance(SURFACE_RAISED)
            radius: instance(8.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius)
                sdf.fill(self.color)
                return sdf.result
            }
        }
    }

    // Underline TabBar - with bottom border
    mod.widgets.MpTabBarUnderline = mod.widgets.MpTabBarBase{
        spacing: 0
        show_bg: true
        draw_bg +: {
            border_color: uniform(BORDER)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Bottom border line
                sdf.rect(0.0, self.rect_size.y - 1.0, self.rect_size.x, 1.0)
                sdf.fill(self.border_color)

                return sdf.result
            }
        }
    }

    // Pill TabBar - transparent background
    mod.widgets.MpTabBarPill = mod.widgets.MpTabBarBase{
        spacing: 4
    }

    // Outline TabBar - transparent background
    mod.widgets.MpTabBarOutline = mod.widgets.MpTabBarBase{
        spacing: 8
    }

    // Segmented TabBar - with background container
    mod.widgets.MpTabBarSegmented = mod.widgets.MpTabBarBase{
        padding: 4
        spacing: 2
        show_bg: true
        draw_bg +: {
            color: instance(SURFACE_RAISED)
            radius: instance(8.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius)
                sdf.fill(self.color)
                return sdf.result
            }
        }
    }

    // ============================================================
    // Size Variants
    // ============================================================

    mod.widgets.MpTabSmall = mod.widgets.MpTab{
        padding: Inset{left: 12.0, right: 12.0, top: 4.0, bottom: 4.0}
        draw_text +: {
            text_style: theme.font_regular{font_size: 12.0}
        }
    }

    mod.widgets.MpTabLarge = mod.widgets.MpTab{
        padding: Inset{left: 20.0, right: 20.0, top: 10.0, bottom: 10.0}
        draw_text +: {
            text_style: theme.font_regular{font_size: 16.0}
        }
    }
}

// ============================================================
// Rust Implementation
// ============================================================

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpTab {
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
    text: ArcStringMut,

    #[rust]
    selected: bool,

    #[rust]
    area: Area,
}

impl Widget for MpTab {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                // Only play hover animation if not selected
                if !self.selected {
                    self.animator_play(cx, ids!(hover.on));
                }
            }
            Hit::FingerHoverOut(_) => {
                if !self.selected {
                    self.animator_play(cx, ids!(hover.off));
                }
            }
            Hit::FingerDown(_) => {
                // Reset hover state directly to prevent interference
                self.animator_toggle(cx, false, Animate::No, ids!(hover.on), ids!(hover.off));
                cx.widget_action(uid, MpTabAction::Clicked);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }

    fn text(&self) -> String {
        self.text.as_ref().to_string()
    }

    fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.redraw(cx);
    }
}

impl MpTab {
    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        if self.selected != selected {
            self.selected = selected;
            // Snap the selected animator state (the 2.0 replacement for the
            // old direct `apply_over` writes to the shader instances)
            self.animator_toggle(
                cx,
                selected,
                Animate::No,
                ids!(selected.on),
                ids!(selected.off),
            );
            self.redraw(cx);
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

impl MpTabRef {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpTabAction::Clicked = item.cast() {
                return true;
            }
        }
        false
    }

    pub fn set_selected(&self, cx: &mut Cx, selected: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, selected);
        }
    }

    pub fn is_selected(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.is_selected()
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpTabAction {
    Clicked,
    #[default]
    None,
}
