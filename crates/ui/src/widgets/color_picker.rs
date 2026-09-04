use makepad_widgets::*;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    // ============================================================
    // MpColorPicker - palette swatch grid color picker
    // ============================================================

    mod.widgets.MpColorPicker = set_type_default() do #(MpColorPicker::register_widget(vm)){
        width: Fit
        height: Fit

        // Swatch shader: rounded color cell with hover/selected rings,
        // instances written from Rust per swatch in draw_walk
        set_type_default() do #(DrawMpSwatch::script_shader(vm)){
            ..mod.draw.DrawQuad

            bg: #xffffffff
            selected: 0.0
            hovered: 0.0
            radius: 5.0
            border_color: #x0000
            ring_color: #xffffffff

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let sz = self.rect_size

                sdf.box(0.5, 0.5, sz.x - 1.0, sz.y - 1.0, self.radius)
                sdf.fill_keep(self.bg)
                // Quiet hairline; brightens on hover
                sdf.stroke(mix(self.border_color, self.ring_color, self.hovered * 0.6), 1.0)
                // Inner ring marks the selection
                if (self.selected > 0.5) {
                    sdf.box(2.5, 2.5, sz.x - 5.0, sz.y - 5.0, max(self.radius - 2.0, 1.0))
                    sdf.stroke(self.ring_color, 2.0)
                }
                return sdf.result
            }
        }

        // Palette baked for Rust-side ring/border resolution
        c_ring: TEXT
        c_border: BORDER
    }
}

#[derive(Clone, Debug, Default)]
pub enum MpColorPickerAction {
    /// A swatch was picked; carries the selected color.
    Picked(Vec4f),
    #[default]
    None,
}

/// Swatch shader: one draw instance reused for the whole grid (table row
/// pattern).
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpSwatch {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    bg: Vec4f,
    #[live]
    selected: f32,
    #[live]
    hovered: f32,
    #[live]
    radius: f32,
    #[live]
    border_color: Vec4f,
    #[live]
    ring_color: Vec4f,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MpColorPicker {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_swatch: DrawMpSwatch,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// Palette swatches (row-major grid, wrapped at `columns` per row).
    #[rust]
    colors: Vec<Vec4f>,
    /// Swatches per row (default 8).
    #[rust]
    columns: usize,
    /// Index of the currently selected swatch.
    #[rust]
    selected: Option<usize>,
    /// Index of the swatch under the pointer.
    #[rust]
    hovered: Option<usize>,
    /// Hit areas captured during the last draw pass.
    #[rust]
    swatch_areas: Vec<(Area, usize)>,

    /// Five-step size driving the swatch cell metrics.
    #[live]
    size: MpSize,

    // Palette baked from the theme
    #[live]
    c_ring: Vec4f,
    #[live]
    c_border: Vec4f,
}

impl MpColorPicker {
    fn cell_size(&self) -> f64 {
        match self.size {
            MpSize::XSmall => 14.0,
            MpSize::Small => 18.0,
            MpSize::Medium => 22.0,
            MpSize::Large => 26.0,
            MpSize::XLarge => 32.0,
        }
    }

    fn columns_clamped(&self) -> usize {
        if self.columns == 0 {
            8
        } else {
            self.columns
        }
    }
}

impl Widget for MpColorPicker {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let areas = self.swatch_areas.clone();
        for (area, idx) in areas.iter() {
            match event.hits(cx, *area) {
                Hit::FingerHoverIn(_) => {
                    if self.hovered != Some(*idx) {
                        self.hovered = Some(*idx);
                        cx.set_cursor(MouseCursor::Hand);
                        self.redraw(cx);
                    }
                }
                Hit::FingerHoverOut(_) => {
                    if self.hovered == Some(*idx) {
                        self.hovered = None;
                        cx.set_cursor(MouseCursor::Default);
                        self.redraw(cx);
                    }
                }
                Hit::FingerDown(_) => {
                    self.selected = Some(*idx);
                    let color = self
                        .colors
                        .get(*idx)
                        .copied()
                        .unwrap_or(Vec4f::default());
                    cx.widget_action(
                        self.widget_uid(),
                        MpColorPickerAction::Picked(color),
                    );
                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, _walk: Walk) -> DrawStep {
        let cell = self.cell_size();
        let gap = (cell * 0.22).round();
        let cols = self.columns_clamped();

        // Fixed grid width so RightWrap breaks rows exactly at `columns`
        let total_w = (cols as f64) * cell + (cols.saturating_sub(1) as f64) * gap;

        self.draw_swatch.radius = (cell * 0.22) as f32;
        self.draw_swatch.border_color = self.c_border;
        self.draw_swatch.ring_color = self.c_ring;

        self.swatch_areas.clear();
        cx.begin_turtle(
            Walk::new(Size::Fixed(total_w), Size::fit()),
            Layout {
                flow: Flow::right_wrap(),
                spacing: gap,
                ..Layout::default()
            },
        );

        for (idx, color) in self.colors.iter().enumerate() {
            self.draw_swatch.bg = *color;
            self.draw_swatch.selected = if self.selected == Some(idx) {
                1.0
            } else {
                0.0
            };
            self.draw_swatch.hovered = if self.hovered == Some(idx) {
                1.0
            } else {
                0.0
            };
            self.draw_swatch
                .begin(cx, Walk::fixed(cell, cell), Layout::default());
            self.draw_swatch.end(cx);
            self.swatch_areas.push((self.draw_swatch.area(), idx));
        }

        cx.end_turtle();
        DrawStep::done()
    }
}

impl MpColorPicker {
    pub fn set_colors(&mut self, cx: &mut Cx, colors: Vec<Vec4f>) {
        self.colors = colors;
        self.selected = None;
        self.redraw(cx);
    }

    pub fn set_columns(&mut self, cx: &mut Cx, columns: usize) {
        self.columns = columns;
        self.redraw(cx);
    }

    pub fn set_selected(&mut self, cx: &mut Cx, index: Option<usize>) {
        self.selected = index;
        self.redraw(cx);
    }

    pub fn picked_color(&self) -> Option<Vec4f> {
        self.selected.and_then(|i| self.colors.get(i).copied())
    }

    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }
}

impl MpColorPickerRef {
    pub fn set_colors(&self, cx: &mut Cx, colors: Vec<Vec4f>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_colors(cx, colors);
        }
    }

    pub fn set_columns(&self, cx: &mut Cx, columns: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_columns(cx, columns);
        }
    }

    pub fn set_selected(&self, cx: &mut Cx, index: Option<usize>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, index);
        }
    }

    pub fn picked_color(&self) -> Option<Vec4f> {
        if let Some(inner) = self.borrow() {
            inner.picked_color()
        } else {
            None
        }
    }

    /// Color picked since the last action snapshot.
    pub fn picked(&self, actions: &Actions) -> Option<Vec4f> {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            if let MpColorPickerAction::Picked(color) = item.cast() {
                return Some(color);
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
