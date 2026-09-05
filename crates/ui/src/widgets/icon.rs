use crate::widgets::sizing::MpSize;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpIcon - named SVG icon from the embedded lucide set
    // (ISC licensed, https://lucide.dev). Size integrates with the
    // MpSize system; `color` tints via the SVG currentColor channel.
    // ============================================================

    mod.widgets.MpIconBase = #(MpIcon::register_widget(vm))
    mod.widgets.MpIcon = set_type_default() do mod.widgets.MpIconBase{
        width: Fit
        height: Fit

        draw_bg +: {
            color: #x0000
        }
    }
}

/// Icon actions
#[derive(Clone, Debug, Default)]
pub enum MpIconAction {
    #[default]
    None,
}

/// Embedded lucide icon catalog (name → SVG source). Icons are 24x24
/// stroke paths using `stroke="currentColor"`, so `MpIcon::color` tints
/// them per theme.
pub const ICON_SVGS: &[(&str, &str)] = &[
    ("arrow-left", include_str!("resources/icons/arrow-left.svg")),
    ("arrow-right", include_str!("resources/icons/arrow-right.svg")),
    ("bell", include_str!("resources/icons/bell.svg")),
    ("calendar", include_str!("resources/icons/calendar.svg")),
    ("check", include_str!("resources/icons/check.svg")),
    ("chevron-down", include_str!("resources/icons/chevron-down.svg")),
    ("chevron-left", include_str!("resources/icons/chevron-left.svg")),
    ("chevron-right", include_str!("resources/icons/chevron-right.svg")),
    ("chevron-up", include_str!("resources/icons/chevron-up.svg")),
    ("circle-alert", include_str!("resources/icons/circle-alert.svg")),
    ("circle-check", include_str!("resources/icons/circle-check.svg")),
    ("circle-x", include_str!("resources/icons/circle-x.svg")),
    ("clock", include_str!("resources/icons/clock.svg")),
    ("copy", include_str!("resources/icons/copy.svg")),
    ("download", include_str!("resources/icons/download.svg")),
    ("ellipsis", include_str!("resources/icons/ellipsis.svg")),
    ("file", include_str!("resources/icons/file.svg")),
    ("folder", include_str!("resources/icons/folder.svg")),
    ("grip-vertical", include_str!("resources/icons/grip-vertical.svg")),
    ("heart", include_str!("resources/icons/heart.svg")),
    // lucide renamed `home` to `house`; keep the old name as an alias
    ("home", include_str!("resources/icons/house.svg")),
    ("house", include_str!("resources/icons/house.svg")),
    ("info", include_str!("resources/icons/info.svg")),
    ("loader", include_str!("resources/icons/loader.svg")),
    ("log-out", include_str!("resources/icons/log-out.svg")),
    ("mail", include_str!("resources/icons/mail.svg")),
    ("menu", include_str!("resources/icons/menu.svg")),
    ("minus", include_str!("resources/icons/minus.svg")),
    ("moon", include_str!("resources/icons/moon.svg")),
    ("pause", include_str!("resources/icons/pause.svg")),
    ("pencil", include_str!("resources/icons/pencil.svg")),
    ("play", include_str!("resources/icons/play.svg")),
    ("plus", include_str!("resources/icons/plus.svg")),
    ("rotate-cw", include_str!("resources/icons/rotate-cw.svg")),
    ("search", include_str!("resources/icons/search.svg")),
    ("send", include_str!("resources/icons/send.svg")),
    ("settings", include_str!("resources/icons/settings.svg")),
    ("star", include_str!("resources/icons/star.svg")),
    ("sun", include_str!("resources/icons/sun.svg")),
    ("trash", include_str!("resources/icons/trash.svg")),
    ("upload", include_str!("resources/icons/upload.svg")),
    ("user", include_str!("resources/icons/user.svg")),
    ("x", include_str!("resources/icons/x.svg")),
];

/// Look up an icon's SVG source by name.
pub fn icon_svg(name: &str) -> Option<&'static str> {
    ICON_SVGS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, svg)| *svg)
}

/// Named SVG icon widget.
#[derive(Script, ScriptHook, Widget)]
pub struct MpIcon {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    #[redraw]
    #[live]
    draw_icon: DrawSvg,
    #[rust]
    icon_walk: Walk,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// Icon name from the embedded lucide catalog (e.g. "chevron-down").
    #[live]
    name: ArcStringMut,

    /// Five-step icon size (XSmall 12 .. XLarge 20).
    #[live]
    size: MpSize,

    /// Icon tint applied through the SVG currentColor channel; an alpha
    /// of 0 keeps the SVG's own colors (lucide ships black strokes).
    #[live]
    color: Vec4f,

    #[rust]
    applied_name: Option<String>,
    #[rust]
    applied_size: Option<MpSize>,
    #[rust]
    applied_color: Option<Vec4f>,
    #[rust]
    area: Area,
}

impl Widget for MpIcon {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let _ = (cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Load a newly named icon before drawing so the same frame uses it.
        let name = self.name.as_ref().to_string();
        if self.applied_name.as_deref() != Some(name.as_str()) {
            match icon_svg(&name) {
                Some(svg) => {
                    self.draw_icon.load_from_str(svg);
                    self.applied_name = Some(name);
                }
                None => {
                    log!("MpIcon: unknown icon name {name:?} (see ICON_SVGS)");
                }
            }
        }

        if self.applied_size != Some(self.size) {
            let px = self.size.icon_size();
            let mut iw = Walk::default();
            iw.width = Size::Fixed(px);
            iw.height = Size::Fixed(px);
            self.icon_walk = iw;
            self.applied_size = Some(self.size);
        }

        if self.applied_color != Some(self.color) {
            if self.color.w > 0.0 {
                self.draw_icon
                    .set_color(self.color.x, self.color.y, self.color.z, self.color.w);
            }
            self.applied_color = Some(self.color);
        }

        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_icon.draw_walk(cx, self.icon_walk);
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpIcon {
    /// Change the displayed icon.
    pub fn set_name(&mut self, cx: &mut Cx, name: &str) {
        if self.name.as_ref() != name {
            self.name.as_mut_empty().clear();
            self.name.as_mut_empty().push_str(name);
            self.applied_name = None;
            self.redraw(cx);
        }
    }

    /// Tint the icon (alpha 0 keeps the SVG's own colors).
    pub fn set_color(&mut self, cx: &mut Cx, color: Vec4f) {
        if self.color != color {
            self.color = color;
            self.applied_color = None;
            self.redraw(cx);
        }
    }

    pub fn area(&self) -> Area {
        self.area
    }
}

impl MpIconRef {
    pub fn set_name(&self, cx: &mut Cx, name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_name(cx, name);
        }
    }

    pub fn set_color(&self, cx: &mut Cx, color: Vec4f) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_color(cx, color);
        }
    }
}
