use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpCard - Card container component
    // ============================================================

    // Base card style (using RoundedView for border support)
    mod.widgets.MpCard = mod.widgets.RoundedView{
        width: Fill
        height: Fit
        flow: Down
        padding: 16
        spacing: 12

        draw_bg +: {
            color: SURFACE_CARD
            border_radius: 12.0
            border_size: 1.0
            border_color: BORDER
        }
    }

    // Card with shadow
    mod.widgets.MpCardShadow = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
        padding: 16
        spacing: 12

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            border_radius: instance(12.0)
            shadow_color: instance(#x0000001A)
            shadow_offset_x: instance(0.0)
            shadow_offset_y: instance(2.0)
            shadow_blur: instance(8.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Shadow (drawn first, behind the card)
                let shadow_x = self.shadow_offset_x
                let shadow_y = self.shadow_offset_y
                sdf.box(
                    shadow_x,
                    shadow_y,
                    self.rect_size.x - abs(shadow_x),
                    self.rect_size.y - abs(shadow_y),
                    self.border_radius
                )
                sdf.blur = self.shadow_blur
                sdf.fill(self.shadow_color)
                sdf.blur = 0.0

                // Main card
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.border_radius)
                sdf.fill(self.bg_color)

                return sdf.result
            }
        }
    }

    // Card with hover effect
    mod.widgets.MpCardHover = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
        padding: 16
        spacing: 12
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            bg_color_hover: instance(ELEMENT_HOVER)
            border_radius: instance(12.0)
            border_color: instance(BORDER)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                let bg = mix(self.bg_color, self.bg_color_hover, self.hover)

                sdf.box(
                    0.5,
                    0.5,
                    self.rect_size.x - 1.0,
                    self.rect_size.y - 1.0,
                    self.border_radius
                )
                sdf.fill_keep(bg)
                sdf.stroke(self.border_color, 1.0)

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

    // Outlined card (no background fill)
    mod.widgets.MpCardOutline = mod.widgets.RoundedView{
        width: Fill
        height: Fit
        flow: Down
        padding: 16
        spacing: 12

        draw_bg +: {
            color: TRANSPARENT
            border_radius: 8.0
            border_size: 1.0
            border_color: BORDER
        }
    }

    // Ghost card (transparent, no border)
    mod.widgets.MpCardGhost = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
        padding: 16
        spacing: 12
    }

    // ============================================================
    // Card sub-components
    // ============================================================

    // Card header section
    mod.widgets.MpCardHeader = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 4
    }

    // Card title
    mod.widgets.MpCardTitle = mod.widgets.Label{
        width: Fit
        height: Fit
        padding: 0
        draw_text +: {
            text_style: theme.font_bold{font_size: 18.0}
            color: TEXT
        }
    }

    // Card description
    mod.widgets.MpCardDescription = mod.widgets.Label{
        width: Fill
        height: Fit
        padding: 0
        draw_text +: {
            text_style: theme.font_regular{font_size: 14.0}
            color: TEXT_MUTED
        }
    }

    // Card content section
    mod.widgets.MpCardContent = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 8
    }

    // Card footer section
    mod.widgets.MpCardFooter = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 8
        align: Align{y: 0.5}
    }

    // ============================================================
    // Size Variants
    // ============================================================

    // Small card
    mod.widgets.MpCardSmall = mod.widgets.MpCard{
        padding: 12
        spacing: 8

        draw_bg +: {
            border_radius: 6.0
        }
    }

    // Large card
    mod.widgets.MpCardLarge = mod.widgets.MpCard{
        padding: 24
        spacing: 16

        draw_bg +: {
            border_radius: 12.0
        }
    }

    // ============================================================
    // Color Variants
    // ============================================================

    // Primary card
    mod.widgets.MpCardPrimary = mod.widgets.MpCard{
        draw_bg +: {
            color: ACCENT
        }
    }

    // Danger card
    mod.widgets.MpCardDanger = mod.widgets.MpCard{
        draw_bg +: {
            color: #xfef2f2
            border_size: 1.0
            border_color: #xfecaca
        }
    }

    // Success card
    mod.widgets.MpCardSuccess = mod.widgets.MpCard{
        draw_bg +: {
            color: #xf0fdf4
            border_size: 1.0
            border_color: #xbbf7d0
        }
    }

    // Warning card
    mod.widgets.MpCardWarning = mod.widgets.MpCard{
        draw_bg +: {
            color: #xfffbeb
            border_size: 1.0
            border_color: #xfde68a
        }
    }

    // Info card
    mod.widgets.MpCardInfo = mod.widgets.MpCard{
        draw_bg +: {
            color: #xecfeff
            border_size: 1.0
            border_color: #xa5f3fc
        }
    }

    // ============================================================
    // Interactive Card (with click support)
    // ============================================================

    mod.widgets.MpCardClickableBase = #(MpCardClickable::register_widget(vm))
    mod.widgets.MpCardClickable = set_type_default() do mod.widgets.MpCardClickableBase{
        width: Fill
        height: Fit
        flow: Down
        padding: 16
        spacing: 12
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(SURFACE_CARD)
            bg_color_hover: instance(ELEMENT_HOVER)
            border_radius: instance(12.0)
            border_color: instance(BORDER)
            hover: instance(0.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                let bg = mix(self.bg_color, self.bg_color_hover, self.hover)

                sdf.box(
                    0.5,
                    0.5,
                    self.rect_size.x - 1.0,
                    self.rect_size.y - 1.0,
                    self.border_radius
                )
                sdf.fill_keep(bg)
                sdf.stroke(self.border_color, 1.0)

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

/// Card action emitted when clicked
#[derive(Clone, Debug, Default)]
pub enum MpCardAction {
    Clicked,
    #[default]
    None,
}

/// Interactive card widget with hover effect and click support
#[derive(Script, ScriptHook, Widget)]
pub struct MpCardClickable {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpCardClickable {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if let Hit::FingerUp(fe) = event.hits(cx, self.view.area()) {
            if fe.is_over {
                cx.widget_action(self.widget_uid(), MpCardAction::Clicked);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpCardClickable {
    /// Check if the card was clicked
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            matches!(item.cast(), MpCardAction::Clicked)
        } else {
            false
        }
    }
}

impl MpCardClickableRef {
    /// Check if the card was clicked
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(inner) = self.borrow() {
            inner.clicked(actions)
        } else {
            false
        }
    }
}
