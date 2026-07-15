use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp_theme.*

    let AVATAR_SIZE_XS = 24.0
    let AVATAR_SIZE_SM = 32.0
    let AVATAR_SIZE_MD = 40.0
    let AVATAR_SIZE_LG = 56.0
    let AVATAR_SIZE_XL = 80.0

    // ============================================================
    // MpAvatar - Avatar/profile picture component
    // ============================================================

    // Base avatar with initials (circle shape)
    mod.widgets.MpAvatarBase = #(MpAvatar::register_widget(vm))
    mod.widgets.MpAvatar = set_type_default() do mod.widgets.MpAvatarBase{
        width: AVATAR_SIZE_MD
        height: AVATAR_SIZE_MD
        align: Align{x: 0.5, y: 0.5}

        show_bg: true
        draw_bg +: {
            radius: instance(20.0)
            bg_color: instance(MUTED)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let c = self.rect_size * 0.5
                let r = min(c.x, c.y)

                sdf.circle(c.x, c.y, r)
                sdf.fill(self.bg_color)

                return sdf.result
            }
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 14.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }
    }

    // Extra small avatar
    mod.widgets.MpAvatarXSmall = mod.widgets.MpAvatar{
        width: AVATAR_SIZE_XS
        height: AVATAR_SIZE_XS

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 10.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }
    }

    // Small avatar
    mod.widgets.MpAvatarSmall = mod.widgets.MpAvatar{
        width: AVATAR_SIZE_SM
        height: AVATAR_SIZE_SM

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 12.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }
    }

    // Large avatar
    mod.widgets.MpAvatarLarge = mod.widgets.MpAvatar{
        width: AVATAR_SIZE_LG
        height: AVATAR_SIZE_LG

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 20.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }
    }

    // Extra large avatar
    mod.widgets.MpAvatarXLarge = mod.widgets.MpAvatar{
        width: AVATAR_SIZE_XL
        height: AVATAR_SIZE_XL

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 28.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }
    }

    // ============================================================
    // Square avatar variants
    // ============================================================

    mod.widgets.MpAvatarSquareBase = mod.widgets.MpAvatar{
        draw_bg +: {
            radius: instance(6.0)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)

                sdf.box(
                    0.0
                    0.0
                    self.rect_size.x
                    self.rect_size.y
                    self.radius
                )
                sdf.fill(self.bg_color)

                return sdf.result
            }
        }
    }

    mod.widgets.MpAvatarSquare = mod.widgets.MpAvatarSquareBase{}

    mod.widgets.MpAvatarSquareSmall = mod.widgets.MpAvatarSquareBase{
        width: AVATAR_SIZE_SM
        height: AVATAR_SIZE_SM

        draw_bg +: {
            radius: instance(4.0)
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 12.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }
    }

    mod.widgets.MpAvatarSquareLarge = mod.widgets.MpAvatarSquareBase{
        width: AVATAR_SIZE_LG
        height: AVATAR_SIZE_LG

        draw_bg +: {
            radius: instance(8.0)
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 20.0}
                color: MUTED_FOREGROUND
            }
            text: ""
        }
    }

    // ============================================================
    // Color variants
    // ============================================================

    mod.widgets.MpAvatarPrimary = mod.widgets.MpAvatar{
        draw_bg +: {
            bg_color: instance(PRIMARY)
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 14.0}
                color: PRIMARY_FOREGROUND
            }
            text: ""
        }
    }

    mod.widgets.MpAvatarDanger = mod.widgets.MpAvatar{
        draw_bg +: {
            bg_color: instance(DANGER)
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 14.0}
                color: DANGER_FOREGROUND
            }
            text: ""
        }
    }

    mod.widgets.MpAvatarSuccess = mod.widgets.MpAvatar{
        draw_bg +: {
            bg_color: instance(SUCCESS)
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 14.0}
                color: SUCCESS_FOREGROUND
            }
            text: ""
        }
    }

    mod.widgets.MpAvatarWarning = mod.widgets.MpAvatar{
        draw_bg +: {
            bg_color: instance(WARNING)
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 14.0}
                color: WARNING_FOREGROUND
            }
            text: ""
        }
    }

    // ============================================================
    // Avatar Group
    // ============================================================

    mod.widgets.MpAvatarGroup = mod.widgets.View{
        width: Fit
        height: Fit
        flow: Right
        spacing: -12.0  // Negative spacing for overlap
        align: Align{y: 0.5}
    }
}

/// Avatar widget for displaying user profile pictures or initials
#[derive(Script, ScriptHook, Widget)]
pub struct MpAvatar {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for MpAvatar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpAvatar {
    /// Set the avatar text (usually initials)
    pub fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.view.label(cx, ids!(label)).set_text(cx, text);
    }

    /// Get initials from a full name (e.g., "John Doe" -> "JD")
    pub fn set_initials_from_name(&mut self, cx: &mut Cx, name: &str) {
        let initials: String = name
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        self.set_text(cx, &initials);
    }
}

impl MpAvatarRef {
    /// Set the avatar text (usually initials)
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, text);
        }
    }

    /// Set initials from a full name
    pub fn set_initials_from_name(&self, cx: &mut Cx, name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_initials_from_name(cx, name);
        }
    }
}
