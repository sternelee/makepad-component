use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Base input style
    mod.widgets.MpInputBase = mod.widgets.TextInput{
        width: Fill
        height: Fit

        padding: Inset{left: 12.0, right: 12.0, top: 10.0, bottom: 10.0}

        empty_text: "Enter text..."

        draw_bg +: {
            border_radius: 8.0
            border_width: uniform(1.0)

            bg_color: uniform(INPUT_BG)
            bg_color_hover: uniform(INPUT_BG)
            bg_color_focus: uniform(INPUT_BG)
            bg_color_disabled: uniform(SURFACE)

            border_color: BORDER
            border_color_hover: BORDER_STRONG
            border_color_focus: CARET
            border_color_disabled: BORDER

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                // Background
                let bg = mix(
                    mix(self.bg_color, self.bg_color_hover, self.hover),
                    self.bg_color_focus,
                    self.focus
                )
                let bg_final = mix(bg, self.bg_color_disabled, self.disabled)

                // Border
                let border = mix(
                    mix(self.border_color, self.border_color_hover, self.hover),
                    self.border_color_focus,
                    self.focus
                )
                let border_final = mix(border, self.border_color_disabled, self.disabled)

                // Draw rounded rectangle
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.border_radius
                )

                sdf.fill_keep(bg_final)

                // Focus ring (thicker border when focused)
                let border_w = mix(self.border_width, 2.0, self.focus)
                sdf.stroke(border_final, border_w)

                return sdf.result
            }
        }

        draw_text +: {
            color: TEXT
            color_disabled: TEXT_FAINT
            color_empty: TEXT_FAINT

            text_style: theme.font_regular{
                font_size: 13.0
            }

            get_color: fn() {
                return mix(
                    mix(self.color, self.color_empty, self.empty),
                    self.color_disabled,
                    self.disabled
                )
            }
        }

        draw_cursor +: {
            color: CARET

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 1.0)
                sdf.fill(mix(#x0000, self.color, self.focus * (1.0 - self.blink)))
                return sdf.result
            }
        }

        draw_selection +: {
            color: SELECTION

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 2.0)
                sdf.fill(self.color)
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
            focus: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.2}}
                    apply: {draw_bg: {focus: 0.0} draw_cursor: {focus: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {focus: 1.0} draw_cursor: {focus: 1.0}}
                }
            }
            disabled: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {disabled: 0.0} draw_text: {disabled: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {disabled: 1.0} draw_text: {disabled: 1.0}}
                }
            }
        }
    }

    // Default input
    mod.widgets.MpInput = mod.widgets.MpInputBase{}

    // Small input
    mod.widgets.MpInputSmall = mod.widgets.MpInputBase{
        padding: Inset{left: 8.0, right: 8.0, top: 6.0, bottom: 6.0}

        draw_bg +: {
            border_radius: 4.0
        }

        draw_text +: {
            text_style: theme.font_regular{
                font_size: 12.0
            }
        }
    }

    // Large input
    mod.widgets.MpInputLarge = mod.widgets.MpInputBase{
        padding: Inset{left: 16.0, right: 16.0, top: 14.0, bottom: 14.0}

        draw_bg +: {
            border_radius: 8.0
        }

        draw_text +: {
            text_style: theme.font_regular{
                font_size: 16.0
            }
        }
    }

    // Password input with toggle icon
    mod.widgets.MpInputPasswordBase = #(MpInputPassword::register_widget(vm))
    mod.widgets.MpInputPassword = set_type_default() do mod.widgets.MpInputPasswordBase{
        width: Fill
        height: Fit

        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 12.0, right: 12.0, top: 10.0, bottom: 10.0}
        spacing: 8.0

        show_bg: true
        draw_bg +: {
            hover: instance(0.0)
            focus: instance(0.0)

            border_radius: uniform(6.0)
            border_width: uniform(1.0)

            bg_color: uniform(INPUT_BG)
            bg_color_hover: uniform(INPUT_BG)
            bg_color_focus: uniform(INPUT_BG)

            border_color: uniform(BORDER)
            border_color_hover: uniform(BORDER_STRONG)
            border_color_focus: uniform(CARET)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                let bg = mix(
                    mix(self.bg_color, self.bg_color_hover, self.hover),
                    self.bg_color_focus,
                    self.focus
                )

                let border = mix(
                    mix(self.border_color, self.border_color_hover, self.hover),
                    self.border_color_focus,
                    self.focus
                )

                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    self.border_radius
                )

                sdf.fill_keep(bg)
                let border_w = mix(self.border_width, 2.0, self.focus)
                sdf.stroke(border, border_w)

                return sdf.result
            }
        }

        // Password text input (borderless)
        input := TextInput{
            width: Fill
            height: Fit
            is_password: true
            empty_text: "Enter password..."

            draw_bg +: {
                pixel: fn() {
                    return #x0000
                }
            }

            draw_text +: {
                text_style: theme.font_regular{font_size: 14.0}
                color: TEXT
                color_empty: TEXT_FAINT

                get_color: fn() {
                    return mix(self.color, self.color_empty, self.empty)
                }
            }

            draw_cursor +: {
                color: uniform(CARET)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 1.0)
                    sdf.fill(mix(#x0000, self.color, self.focus * (1.0 - self.blink)))
                    return sdf.result
                }
            }

            draw_selection +: {
                selection_color: instance(SELECTION)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 2.0)
                    sdf.fill(self.selection_color)
                    return sdf.result
                }
            }
        }

        // Eye icon for toggle visibility (clickable)
        eye_icon := Button{
            width: Fit
            height: Fit
            padding: 4
            text: ""

            draw_bg +: {
                pixel: fn() {
                    return #x0000
                }
            }

            draw_icon +: {
                svg: crate_resource("self:resources/icons/eye-off.svg")
                color: #x9E9E9E
            }

            icon_walk: Walk{width: 18.0, height: 18.0}

            animator: Animator{
                hover: {
                    default: @off
                    off: AnimatorState{
                        from: {all: Forward {duration: 0.1}}
                        apply: {draw_icon: {color: #x9E9E9E}}
                    }
                    on: AnimatorState{
                        from: {all: Forward {duration: 0.1}}
                        apply: {draw_icon: {color: #x666666}}
                    }
                }
            }
        }
    }

    // Numeric input
    mod.widgets.MpInputNumeric = mod.widgets.MpInputBase{
        is_numeric_only: true
        empty_text: "Enter number..."
    }

    // Borderless input (for inline editing)
    mod.widgets.MpInputBorderless = mod.widgets.MpInputBase{
        draw_bg +: {
            bg_color: #x00000000
            bg_color_hover: ELEMENT_HOVER
            bg_color_focus: #x00000000
            border_color: #x00000000
            border_color_hover: #x00000000
            border_color_focus: CARET
        }
    }

    // Search input with icon (capsule/pill shape) + clear button
    mod.widgets.MpInputSearchBase = #(MpInputSearch::register_widget(vm))
    mod.widgets.MpInputSearch = set_type_default() do mod.widgets.MpInputSearchBase{
        width: Fill
        height: Fit

        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 16.0, right: 16.0, top: 10.0, bottom: 10.0}
        spacing: 8.0

        show_bg: true
        draw_bg +: {
            hover: instance(0.0)
            focus: instance(0.0)

            bg_color: uniform(INPUT_BG)
            bg_color_hover: uniform(ELEMENT_ACTIVE)
            bg_color_focus: uniform(INPUT_BG)

            border_color: uniform(BORDER)
            border_color_hover: uniform(BORDER)
            border_color_focus: uniform(CARET)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size
                let r = sz.y * 0.5

                let bg = mix(
                    mix(self.bg_color, self.bg_color_hover, self.hover),
                    self.bg_color_focus,
                    self.focus
                )

                let border = mix(
                    mix(self.border_color, self.border_color_hover, self.hover),
                    self.border_color_focus,
                    self.focus
                )

                // Draw capsule: left circle + rectangle + right circle
                sdf.circle(r, r, r)
                sdf.rect(r, 0.0, sz.x - sz.y, sz.y)
                sdf.circle(sz.x - r, r, r)

                sdf.fill_keep(bg)
                sdf.stroke(border, 1.0)

                return sdf.result
            }
        }

        // Search icon
        search_icon := Icon{
            icon_walk: Walk{width: 14.0, height: Fit}
            draw_icon +: {
                svg: crate_resource("self:resources/icons/search.svg")
                color: TEXT_FAINT
            }
        }

        // Text input (borderless)
        input := TextInput{
            width: Fill
            height: Fit
            empty_text: "Search..."

            draw_bg +: {
                pixel: fn() {
                    return #x0000
                }
            }

            draw_text +: {
                text_style: theme.font_regular{font_size: 13.0}
                color: TEXT
                color_empty: TEXT_FAINT

                get_color: fn() {
                    return mix(self.color, self.color_empty, self.empty)
                }
            }

            draw_cursor +: {
                color: uniform(CARET)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 1.0)
                    sdf.fill(mix(#x0000, self.color, self.focus * (1.0 - self.blink)))
                    return sdf.result
                }
            }

            draw_selection +: {
                selection_color: instance(SELECTION)

                pixel: fn() {
                    let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 2.0)
                    sdf.fill(self.selection_color)
                    return sdf.result
                }
            }
        }

        // Clear button (visible only when the input has text)
        clear_btn := mod.widgets.MpButtonGhost{
            width: 18
            height: 18
            text: "✕"
            draw_text +: {
                text_style: theme.font_regular{font_size: 11.0}
                color: TEXT_FAINT
            }
        }
    }
}

pub use makepad_widgets::text_input::TextInputAction as MpInputAction;

// Password input widget with toggle visibility
#[derive(Script, ScriptHook, Widget)]
pub struct MpInputPassword {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[rust]
    password_visible: bool,
}

impl Widget for MpInputPassword {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MpInputPassword {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Handle eye icon button click
        if self.view.button(cx, ids!(eye_icon)).clicked(actions) {
            self.password_visible = !self.password_visible;

            // Toggle password visibility on the input
            let input = self.view.text_input(cx, ids!(input));
            input.set_is_password(cx, !self.password_visible);

            self.view.redraw(cx);
        }
    }
}

impl MpInputPasswordRef {
    /// Get the current password text
    pub fn text(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.view.child(id!(input)).as_text_input().text()
        } else {
            String::new()
        }
    }

    /// Set the password text
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(inner) = self.borrow() {
            inner.view.text_input(cx, ids!(input)).set_text(cx, text);
        }
    }
}

// Search input widget: capsule with search icon, text input and a clear
// button that is only visible while the field has text.
#[derive(Script, ScriptHook, Widget)]
pub struct MpInputSearch {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpInputSearch {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for MpInputSearch {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Clear button click -> empty the field.
        if self.view.button(cx, ids!(clear_btn)).clicked(actions) {
            let input = self.view.text_input(cx, ids!(input));
            input.set_text(cx, "");
            self.sync_clear_visible(cx);
            self.view.redraw(cx);
        }

        // Keep the clear button in sync with the text contents.
        if self
            .view
            .text_input(cx, ids!(input))
            .changed(actions)
            .is_some()
        {
            self.sync_clear_visible(cx);
        }
    }
}

impl MpInputSearch {
    fn sync_clear_visible(&mut self, cx: &mut Cx) {
        let non_empty = !self.view.text_input(cx, ids!(input)).text().is_empty();
        self.view.button(cx, ids!(clear_btn)).set_visible(cx, non_empty);
    }
}

impl MpInputSearchRef {
    /// Get the current search text.
    pub fn text(&self) -> String {
        if let Some(inner) = self.borrow() {
            inner.view.child(id!(input)).as_text_input().text()
        } else {
            String::new()
        }
    }

    /// Set the search text.
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.text_input(cx, ids!(input)).set_text(cx, text);
            inner.sync_clear_visible(cx);
            inner.view.redraw(cx);
        }
    }
}
