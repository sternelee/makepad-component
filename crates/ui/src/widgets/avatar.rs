use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

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
            bg_color: instance(SURFACE)

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
                color: TEXT_MUTED
            }
            text: ""
        }
    }

    // Extra small avatar
    mod.widgets.MpAvatarXSmall = mod.widgets.MpAvatar{
        size: MpSize.XSmall
    }

    // Small avatar
    mod.widgets.MpAvatarSmall = mod.widgets.MpAvatar{
        size: MpSize.Small
    }

    // Large avatar
    mod.widgets.MpAvatarLarge = mod.widgets.MpAvatar{
        size: MpSize.Large
    }

    // Extra large avatar
    mod.widgets.MpAvatarXLarge = mod.widgets.MpAvatar{
        size: MpSize.XLarge
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
        size: MpSize.Small

        draw_bg +: {
            radius: instance(4.0)
        }
    }

    mod.widgets.MpAvatarSquareLarge = mod.widgets.MpAvatarSquareBase{
        size: MpSize.Large

        draw_bg +: {
            radius: instance(8.0)
        }
    }

    // ============================================================
    // Color variants
    // ============================================================

    mod.widgets.MpAvatarPrimary = mod.widgets.MpAvatar{
        draw_bg +: {
            bg_color: instance(ACCENT)
        }

        label := Label{
            width: Fit
            height: Fit
            draw_text +: {
                text_style: theme.font_bold{font_size: 14.0}
                color: ON_ACCENT
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
                color: BG
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
                color: BG
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
                color: BG
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

    /// Five-step size driving the avatar diameter and label font.
    #[live]
    size: MpSize,

    /// Last size applied to the label (avoids re-applying every draw).
    #[rust]
    applied_size: Option<MpSize>,
}

/// Avatar diameter for a size step (MpSize-aware metric table).
fn avatar_diameter(size: MpSize) -> f64 {
    match size {
        MpSize::XSmall => 24.0,
        MpSize::Small => 32.0,
        MpSize::Medium => 40.0,
        MpSize::Large => 56.0,
        MpSize::XLarge => 80.0,
    }
}

/// Initials font size for a size step.
fn avatar_font(size: MpSize) -> f32 {
    match size {
        MpSize::XSmall => 10.0,
        MpSize::Small => 12.0,
        MpSize::Medium => 14.0,
        MpSize::Large => 20.0,
        MpSize::XLarge => 28.0,
    }
}

impl Widget for MpAvatar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // The avatar extent scales with the size system (square).
        let diameter = avatar_diameter(self.size);
        let mut walk = walk;
        walk.width = Size::Fixed(diameter);
        walk.height = Size::Fixed(diameter);

        if self.applied_size != Some(self.size) {
            self.applied_size = Some(self.size);
            if let Some(mut label) = self.view.label(cx, ids!(label)).borrow_mut() {
                label.draw_text.text_style.font_size = avatar_font(self.size);
            }
        }

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

    /// Set the avatar's left walk margin (used for overlap stacks; negative
    /// values pull later siblings over earlier ones).
    pub fn set_overlap_margin(&self, cx: &mut Cx, margin: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.walk.margin.left = margin;
            inner.redraw(cx);
        }
    }
}
