// DbPro — dynamic tab strip.
//
// TablePro-style document tabs: an arbitrary number of tabs (table views,
// query editors) with per-tab close buttons and a trailing "+" button.
// Painted manually over a Rust-side model, following the MpTable pattern:
// quads + hit-tested Areas + widget actions.
use makepad_widgets::*;

use makepad_component::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // Tab plate shader: hover / selected washes.
    set_type_default() do #(DrawTabPlate::script_shader(vm)){
        ..mod.draw.DrawQuad

        hovered: 0.0
        selected: 0.0
        base_color: #x00000000
        hover_color: ELEMENT_HOVER
        selected_color: ELEMENT_ACTIVE

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let base = mix(self.base_color, self.hover_color, self.hovered)
            let c = mix(base, self.selected_color, self.selected)
            // Rounded top corners (tab look)
            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 4.0)
            sdf.fill(c)
            return sdf.result
        }
    }

    mod.widgets.DbTabBar = #(DbTabBar::register_widget(vm)){
        width: Fill
        height: 36
        flow: Down

        draw_bg +: {
            color: #x00000000
        }

        draw_tab +: {
            base_color: #x00000000
            hover_color: ELEMENT_HOVER
            selected_color: ELEMENT_ACTIVE
        }

        // draw-time title colors (assigned in draw_walk; MUST have DSL
        // defaults or Vec4f::default() = transparent hides the text!)
        c_text_active: TEXT
        c_text_muted: TEXT_MUTED

        draw_title +: {
            text_style: theme.font_regular{font_size: 12.0}
            color: TEXT_MUTED
        }

        draw_close +: {
            text_style: theme.font_regular{font_size: 11.0}
            color: TEXT_FAINT
        }

        draw_add +: {
            text_style: theme.font_regular{font_size: 16.0}
            color: TEXT_MUTED
        }
    }
}

/// One tab in the strip.
#[derive(Clone, Debug)]
pub struct TabDef {
    pub title: String,
    pub closable: bool,
}

impl TabDef {
    pub fn new(title: &str, closable: bool) -> Self {
        Self {
            title: title.to_string(),
            closable,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum DbTabBarAction {
    Selected(usize),
    Closed(usize),
    AddClicked,
    #[default]
    None,
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawTabPlate {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    hovered: f32,
    #[live]
    selected: f32,
    #[live]
    base_color: Vec4f,
    #[live]
    hover_color: Vec4f,
    #[live]
    selected_color: Vec4f,
}

#[derive(Script, ScriptHook, Widget)]
pub struct DbTabBar {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[redraw]
    #[live]
    draw_tab: DrawTabPlate,

    #[live]
    draw_title: DrawText,

    #[live]
    c_text_active: Vec4f,

    #[live]
    c_text_muted: Vec4f,

    #[live]
    draw_close: DrawText,

    #[live]
    draw_add: DrawText,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live(36.0)]
    tab_height: f64,

    /// Max width of a single tab; shrinks when tabs overflow the bar.
    #[live(200.0)]
    max_tab_width: f64,

    #[live(120.0)]
    min_tab_width: f64,

    #[live]
    size: MpSize,

    // Model (set by the app)
    #[rust]
    tabs: Vec<TabDef>,

    #[rust]
    active: Option<usize>,

    // Interaction state
    #[rust]
    hovered_tab: Option<usize>,

    #[rust]
    tab_areas: Vec<(Area, usize)>,

    #[rust]
    close_areas: Vec<(Area, usize)>,

    #[rust]
    add_area: Area,

    #[rust]
    area: Area,
}

impl DbTabBar {
    pub fn set_tabs(&mut self, cx: &mut Cx, tabs: Vec<TabDef>, active: Option<usize>) {
        self.tabs = tabs;
        self.active = active;
        self.hovered_tab = None;
        self.redraw(cx);
    }

    pub fn set_active(&mut self, cx: &mut Cx, active: Option<usize>) {
        self.active = active;
        self.redraw(cx);
    }

    /// Selection action in this action batch.
    pub fn selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let DbTabBarAction::Selected(i) = action.cast() {
                return Some(i);
            }
        }
        None
    }

    /// Close action in this action batch.
    pub fn closed(&self, actions: &Actions) -> Option<usize> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let DbTabBarAction::Closed(i) = action.cast() {
                return Some(i);
            }
        }
        None
    }

    /// "+"-button click in this action batch.
    pub fn add_clicked(&self, actions: &Actions) -> bool {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            return matches!(action.cast::<DbTabBarAction>(), DbTabBarAction::AddClicked);
        }
        false
    }

    fn layout_widths(&self, bar_width: f64) -> Vec<f64> {
        let n = self.tabs.len();
        if n == 0 {
            return Vec::new();
        }
        let add_w = 32.0;
        let available = (bar_width - add_w).max(self.min_tab_width * n as f64);
        let mut w = self.max_tab_width;
        if w * n as f64 > available {
            w = (available / n as f64).max(self.min_tab_width);
        }
        vec![w; n]
    }
}

impl Widget for DbTabBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let mut needs_redraw = false;

        // Close buttons take priority over tab selection.
        let mut close_hit: Option<usize> = None;
        for (area, idx) in self.close_areas.iter() {
            if let Hit::FingerUp(fe) = event.hits(cx, *area) {
                if fe.is_over && fe.was_tap() {
                    close_hit = Some(*idx);
                }
            }
        }
        if let Some(idx) = close_hit {
            cx.widget_action(self.widget_uid(), DbTabBarAction::Closed(idx));
            needs_redraw = true;
            // Fall through: redraw, and don't treat this as a selection.
        }

        if close_hit.is_none() {
            for (area, idx) in self.tab_areas.iter() {
                match event.hits(cx, *area) {
                    Hit::FingerHoverIn(_) => {
                        if self.hovered_tab != Some(*idx) {
                            self.hovered_tab = Some(*idx);
                            cx.set_cursor(MouseCursor::Hand);
                            needs_redraw = true;
                        }
                    }
                    Hit::FingerHoverOut(_) => {
                        if self.hovered_tab == Some(*idx) {
                            self.hovered_tab = None;
                            cx.set_cursor(MouseCursor::Default);
                            needs_redraw = true;
                        }
                    }
                    Hit::FingerUp(fe)
                        if fe.is_over && fe.was_tap() && self.active != Some(*idx) =>
                    {
                        self.active = Some(*idx);
                        cx.widget_action(self.widget_uid(), DbTabBarAction::Selected(*idx));
                        needs_redraw = true;
                    }
                    _ => {}
                }
            }
        }

        if let Hit::FingerUp(fe) = event.hits(cx, self.add_area) {
            if fe.is_over && fe.was_tap() {
                cx.widget_action(self.widget_uid(), DbTabBarAction::AddClicked);
                needs_redraw = true;
            }
        }

        if needs_redraw {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.size != MpSize::Medium {
            self.tab_height = match self.size {
                MpSize::XSmall => 28.0,
                MpSize::Small => 32.0,
                MpSize::Large => 40.0,
                MpSize::XLarge => 44.0,
                MpSize::Medium => 36.0,
            };
        }

        self.tab_areas.clear();
        self.close_areas.clear();

        // Width from the previous frame (first frame falls back to min
        // widths and self-corrects on the next repaint).
        let bar_width = self.area.rect(cx).size.x;
        let widths = self.layout_widths(bar_width);

        self.draw_bg.begin(cx, walk, self.layout);
        self.area = self.draw_bg.area();

        // Tab row
        cx.begin_turtle(
            Walk::new(Size::fill(), Size::Fixed(self.tab_height)),
            Layout {
                flow: Flow::right(),
                spacing: 2.0,
                align: Align { x: 0.0, y: 0.5 },
                ..Layout::default()
            },
        );

        for (idx, tab) in self.tabs.iter().enumerate() {
            let w = widths.get(idx).copied().unwrap_or(self.max_tab_width);
            let selected = self.active == Some(idx);
            let hovered = self.hovered_tab == Some(idx);

            self.draw_tab.selected = if selected { 1.0 } else { 0.0 };
            self.draw_tab.hovered = if hovered && !selected { 1.0 } else { 0.0 };

            cx.begin_turtle(
                Walk::new(Size::Fixed(w), Size::Fixed(self.tab_height)),
                Layout {
                    flow: Flow::right(),
                    spacing: 4.0,
                    align: Align { x: 0.5, y: 0.5 },
                    padding: Inset {
                        left: 12.0,
                        right: 4.0,
                        ..Default::default()
                    },
                    ..Layout::default()
                },
            );

            // plate turtle: centers the title+close group in the tab
            self.draw_tab.begin(
                cx,
                Walk::new(Size::fill(), Size::Fixed(self.tab_height)),
                Layout {
                    flow: Flow::right(),
                    spacing: 4.0,
                    align: Align { x: 0.5, y: 0.5 },
                    ..Default::default()
                },
            );

            let mut label = tab.title.clone();
            let max_chars = ((w - 36.0) / 7.2).max(4.0) as usize;
            if label.chars().count() > max_chars {
                label = format!("{}…", label.chars().take(max_chars).collect::<String>());
            }
            // selected tab gets the bright title color
            self.draw_title.color = if selected {
                self.c_text_active
            } else {
                self.c_text_muted
            };
            self.draw_title
                .draw_walk(cx, Walk::fit(), Align::default(), &label);
            self.draw_close.draw_walk(
                cx,
                Walk::new(Size::Fixed(18.0), Size::Fixed(18.0)),
                Align::default(),
                if tab.closable { "×" } else { "" },
            );
            if tab.closable {
                self.close_areas.push((self.draw_close.area(), idx));
            }

            self.draw_tab.end(cx);
            self.tab_areas.push((self.draw_tab.area(), idx));

            cx.end_turtle(); // tab
        }

        // "+" (new query) button
        self.draw_add.draw_walk(
            cx,
            Walk::new(Size::Fixed(32.0), Size::Fixed(self.tab_height)),
            Align::default(),
            "+",
        );
        self.add_area = self.draw_add.area();

        cx.end_turtle(); // row
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl DbTabBarRef {
    pub fn set_tabs(&self, cx: &mut Cx, tabs: Vec<TabDef>, active: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_tabs(cx, tabs, active);
        }
    }

    pub fn set_active(&self, cx: &mut Cx, active: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_active(cx, active);
        }
    }

    pub fn selected(&self, actions: &Actions) -> Option<usize> {
        self.borrow()
            .map(|inner| inner.selected(actions))
            .unwrap_or(None)
    }

    pub fn closed(&self, actions: &Actions) -> Option<usize> {
        self.borrow()
            .map(|inner| inner.closed(actions))
            .unwrap_or(None)
    }

    pub fn add_clicked(&self, actions: &Actions) -> bool {
        self.borrow()
            .map(|inner| inner.add_clicked(actions))
            .unwrap_or(false)
    }
}
