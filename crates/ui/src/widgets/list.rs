use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    // ============================================================
    // MpList - List container and item components
    // ============================================================

    // List container
    mod.widgets.MpList = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
    }

    // List with dividers (using RoundedView for proper border support)
    mod.widgets.MpListDivided = mod.widgets.RoundedView{
        width: Fill
        height: Fit
        flow: Down

        draw_bg +: {
            color: CARD
            border_radius: 8.0
            border_color: BORDER
        }
    }

    // ============================================================
    // List Items
    // ============================================================

    // Basic list item
    mod.widgets.MpListItem = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, right: 16.0, top: 12.0, bottom: 12.0}
        flow: Right
        align: Align{y: 0.5}
        spacing: 12
    }

    // List item with hover effect
    mod.widgets.MpListItemHover = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, right: 16.0, top: 12.0, bottom: 12.0}
        flow: Right
        align: Align{y: 0.5}
        spacing: 12
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x00000000)
            bg_color_hover: instance(#xf8fafc)
            hover: instance(0.0)

            pixel: fn() {
                return Pal.premul(mix(self.bg_color, self.bg_color_hover, self.hover))
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

    // List item with active state
    mod.widgets.MpListItemActive = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, right: 16.0, top: 12.0, bottom: 12.0}
        flow: Right
        align: Align{y: 0.5}
        spacing: 12
        cursor: MouseCursor.Hand

        show_bg: true
        draw_bg +: {
            bg_color: instance(#x00000000)
            bg_color_hover: instance(#xf8fafc)
            bg_color_active: instance(#xeff6ff)
            hover: instance(0.0)
            active: instance(0.0)

            pixel: fn() {
                let base = mix(self.bg_color, self.bg_color_active, self.active)
                return Pal.premul(mix(base, self.bg_color_hover, self.hover * (1.0 - self.active)))
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
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {active: 1.0}}
                }
            }
        }
    }

    // ============================================================
    // List Item Components
    // ============================================================

    // List item leading (icon/avatar area)
    mod.widgets.MpListItemLeading = mod.widgets.View{
        width: Fit
        height: Fit
        align: Align{x: 0.5, y: 0.5}
    }

    // List item content (title + description)
    mod.widgets.MpListItemContent = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 2
    }

    // List item title
    mod.widgets.MpListItemTitle = mod.widgets.Label{
        width: Fill
        height: Fit
        draw_text +: {
            text_style: theme.font_regular{font_size: 14.0}
            color: FOREGROUND
        }
    }

    // List item description/subtitle
    mod.widgets.MpListItemDescription = mod.widgets.Label{
        width: Fill
        height: Fit
        draw_text +: {
            text_style: theme.font_regular{font_size: 12.0}
            color: MUTED_FOREGROUND
        }
    }

    // List item trailing (action area)
    mod.widgets.MpListItemTrailing = mod.widgets.View{
        width: Fit
        height: Fit
        align: Align{x: 0.5, y: 0.5}
    }

    // ============================================================
    // List Divider
    // ============================================================

    mod.widgets.MpListDivider = mod.widgets.SolidView{
        width: Fill
        height: 1
        margin: Inset{left: 16.0, right: 16.0}
        draw_bg +: {
            color: BORDER
        }
    }

    mod.widgets.MpListDividerFull = mod.widgets.SolidView{
        width: Fill
        height: 1
        draw_bg +: {
            color: BORDER
        }
    }

    // ============================================================
    // List Section Header
    // ============================================================

    mod.widgets.MpListSectionHeader = mod.widgets.SolidView{
        width: Fill
        height: Fit
        padding: Inset{left: 16.0, right: 16.0, top: 8.0, bottom: 8.0}

        draw_bg +: {
            color: MUTED
        }

        Label{
            width: Fill
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 12.0}
                color: MUTED_FOREGROUND
            }
        }
    }

    // ============================================================
    // Compact List Item
    // ============================================================

    mod.widgets.MpListItemCompact = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 12.0, right: 12.0, top: 8.0, bottom: 8.0}
        flow: Right
        align: Align{y: 0.5}
        spacing: 8
    }

    // ============================================================
    // Large List Item
    // ============================================================

    mod.widgets.MpListItemLarge = mod.widgets.View{
        width: Fill
        height: Fit
        padding: Inset{left: 20.0, right: 20.0, top: 16.0, bottom: 16.0}
        flow: Right
        align: Align{y: 0.5}
        spacing: 16
    }
}
