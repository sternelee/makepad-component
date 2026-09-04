use makepad_widgets::*;
use std::collections::HashSet;

use crate::widgets::sizing::MpSize;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mpc_theme.*

    mod.widgets.MpTreeBase = #(MpTree::register_widget(vm))

    // Register the row shader class before the widget defaults reference it
    set_type_default() do #(DrawTreeRow::script_shader(vm)){
        ..mod.draw.DrawQuad

        hovered: 0.0
        selected: 0.0
        disabled: 0.0
        base_color: #x00000000
        hover_color: ELEMENT_HOVER
        selected_color: ELEMENT_ACTIVE

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let base = mix(self.base_color, self.hover_color, self.hovered)
            let c = mix(base, self.selected_color, self.selected)
            // Disabled rows fade to half alpha (gpui grouped fade convention)
            let c = vec4(c.rgb, c.a * (1.0 - self.disabled * 0.5))
            sdf.box(2.0, 1.0, self.rect_size.x - 4.0, self.rect_size.y - 2.0, 6.0)
            sdf.fill(c)
            return sdf.result
        }
    }

    mod.widgets.MpTree = set_type_default() do mod.widgets.MpTreeBase{
        width: Fill
        height: Fill

        draw_bg +: {
            color: #x00000000
        }

        draw_label +: {
            text_style: theme.font_regular{font_size: 12.5}
            color: TEXT
        }

        // Chevron glyph (▸ / ▾ swapped by Rust)
        draw_chevron +: {
            text_style: theme.font_regular{font_size: 10.0}
            color: TEXT_FAINT
        }

        // Palette baked for Rust-side disabled label color
        c_text: TEXT
        c_text_muted: TEXT_MUTED
    }
}

/// One node in the flat tree model. `depth` drives indentation;
/// children are the items that follow with a greater depth.
///
/// The legacy flat API (`TreeItem::new(label, depth)`) feeds the widget
/// directly; the gpui-style nested builders (`with_children`,
/// `disabled`) produce nodes that must be passed through
/// [`flatten_tree`] first.
#[derive(Clone, Debug, Default)]
pub struct TreeItem {
    pub label: String,
    pub depth: usize,
    /// Disabled rows ignore hover/click and render faded.
    pub disabled: bool,
    /// Nested children (only populated by the `with_children` builder;
    /// ignored by the flat widget path until flattened).
    pub children: Vec<TreeItem>,
}

impl TreeItem {
    pub fn new(label: &str, depth: usize) -> Self {
        Self {
            label: label.to_string(),
            depth,
            disabled: false,
            children: Vec::new(),
        }
    }

    /// Build a subtree from the gpui-style nested `children` builder.
    /// The node and all descendants flatten into a list with depths
    /// assigned by nesting level.
    pub fn with_children(label: &str, children: Vec<TreeItem>) -> Self {
        let mut node = Self::new(label, 0);
        node.children = children;
        node
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Flatten a nested `TreeItem` forest into the flat depth-encoded list
/// the widget renders. Nested builders assign depths automatically.
pub fn flatten_tree(forest: Vec<TreeItem>) -> Vec<TreeItem> {
    fn push(items: &[TreeItem], depth: usize, out: &mut Vec<TreeItem>) {
        for item in items {
            out.push(TreeItem {
                label: item.label.clone(),
                depth,
                disabled: item.disabled,
                children: Vec::new(),
            });
            push(&item.children, depth + 1, out);
        }
    }
    let mut out = Vec::new();
    push(&forest, 0, &mut out);
    out
}

#[derive(Clone, Debug, Default)]
pub enum MpTreeAction {
    ItemSelected(usize),
    #[default]
    None,
}

/// Row paint shader: hover wash + selected plate with CONTROL radius.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawTreeRow {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    hovered: f32,
    #[live]
    selected: f32,
    #[live]
    disabled: f32,
    #[live]
    base_color: Vec4f,
    #[live]
    hover_color: Vec4f,
    #[live]
    selected_color: Vec4f,
}

/// Flat-model tree: Rust owns `TreeItem{label, depth}` array and the
/// expanded set; visibility = all ancestors expanded. Indent is
/// depth × INDENT px; chevron swaps ▸/▾ on toggle.
#[derive(Script, ScriptHook, Widget)]
pub struct MpTree {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[redraw]
    #[live]
    draw_row: DrawTreeRow,

    #[live]
    draw_label: DrawText,

    #[live]
    draw_chevron: DrawText,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live(30.0)]
    row_height: f64,

    /// Indentation per depth level (px)
    #[live(14.0)]
    indent_step: f64,

    /// Five-step size driving the row height and fonts. The default Medium
    /// defers to the DSL heights/fonts; other steps override.
    #[live]
    size: MpSize,

    // Data (set by caller)
    #[rust]
    items: Vec<TreeItem>,

    /// Indices of expanded branch nodes
    #[rust]
    expanded: HashSet<usize>,

    // Interaction state
    #[rust]
    selected_item: Option<usize>,

    #[rust]
    hovered_item: Option<usize>,

    /// Visible rows this frame: (item_idx, area, chevron_area_opt)
    #[rust]
    row_areas: Vec<(Area, usize)>,

    #[rust]
    chevron_areas: Vec<(Area, usize)>,

    #[rust]
    area: Area,

    // Palette baked from the theme (disabled label color)
    #[live]
    c_text: Vec4f,
    #[live]
    c_text_muted: Vec4f,
}

impl MpTree {
    pub fn size(&self) -> MpSize {
        self.size
    }

    pub fn set_size(&mut self, cx: &mut Cx, size: MpSize) {
        if self.size != size {
            self.size = size;
            self.redraw(cx);
        }
    }

    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<TreeItem>) {
        self.items = items;
        self.expanded.clear();
        self.selected_item = None;
        // Auto-expand depth-0 branches so the tree opens showing content
        for (i, item) in self.items.iter().enumerate() {
            if item.depth == 0 && self.has_children(i) {
                self.expanded.insert(i);
            }
        }
        self.redraw(cx);
    }

    fn has_children(&self, idx: usize) -> bool {
        let Some(depth) = self.items.get(idx).map(|i| i.depth) else {
            return false;
        };
        self.items
            .get(idx + 1)
            .map(|next| next.depth > depth)
            .unwrap_or(false)
    }

    fn is_ancestor_expanded(&self, idx: usize) -> bool {
        let Some(depth) = self.items.get(idx).map(|i| i.depth) else {
            return false;
        };
        if depth == 0 {
            return true;
        }
        // Every ancestor (depths depth-1 .. 0) must be in the expanded set
        let mut need = depth - 1;
        for i in (0..idx).rev() {
            let d = self.items[i].depth;
            if d == need {
                if !self.expanded.contains(&i) {
                    return false;
                }
                if need == 0 {
                    return true;
                }
                need -= 1;
            }
        }
        true
    }

    /// Visible item indices in display order.
    fn visible_items(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(i, _)| self.is_ancestor_expanded(*i))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn item_selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpTreeAction::ItemSelected(i) = action.cast() {
                return Some(i);
            }
        }
        None
    }

    pub fn set_expanded(&mut self, cx: &mut Cx, idx: usize, open: bool) {
        if open {
            self.expanded.insert(idx);
        } else {
            self.expanded.remove(&idx);
        }
        self.redraw(cx);
    }
}

impl Widget for MpTree {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let mut needs_redraw = false;

        for (area, item_idx) in self.row_areas.iter() {
            // Disabled rows ignore hover, cursor and clicks (gpui parity)
            let disabled = self.items.get(*item_idx).map(|i| i.disabled).unwrap_or(false);
            match event.hits(cx, *area) {
                Hit::FingerHoverIn(_) => {
                    if !disabled && self.hovered_item != Some(*item_idx) {
                        self.hovered_item = Some(*item_idx);
                        cx.set_cursor(MouseCursor::Hand);
                        needs_redraw = true;
                    }
                }
                Hit::FingerHoverOut(_) => {
                    if self.hovered_item == Some(*item_idx) {
                        self.hovered_item = None;
                        cx.set_cursor(MouseCursor::Default);
                        needs_redraw = true;
                    }
                }
                Hit::FingerDown(_) => {
                    if disabled {
                        continue;
                    }
                    self.selected_item = Some(*item_idx);
                    cx.widget_action(self.widget_uid(), MpTreeAction::ItemSelected(*item_idx));
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        // Chevron hit areas toggle expansion without changing selection
        for (area, item_idx) in self.chevron_areas.iter() {
            if let Hit::FingerUp(fe) = event.hits(cx, *area) {
                if fe.is_over && fe.was_tap() {
                    if self.expanded.contains(item_idx) {
                        self.expanded.remove(item_idx);
                    } else {
                        self.expanded.insert(*item_idx);
                    }
                    needs_redraw = true;
                }
            }
        }

        if needs_redraw {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Size system: Medium defers to the DSL heights/fonts; other steps
        // override.
        if self.size != MpSize::Medium {
            self.row_height = match self.size {
                MpSize::XSmall => 24.0,
                MpSize::Small => 27.0,
                MpSize::Large => 36.0,
                MpSize::XLarge => 42.0,
                MpSize::Medium => 30.0,
            };
            let font = match self.size {
                MpSize::XSmall => 10.5,
                MpSize::Small => 11.5,
                MpSize::Large => 13.5,
                MpSize::XLarge => 15.5,
                MpSize::Medium => 12.5,
            };
            self.draw_label.text_style.font_size = font;
            self.draw_chevron.text_style.font_size = font - 2.0;
        }

        let visible = self.visible_items();

        self.row_areas.clear();
        self.chevron_areas.clear();

        self.draw_bg.begin(cx, walk, self.layout);

        // Outer container
        cx.begin_turtle(
            Walk::new(Size::fill(), Size::fit()),
            Layout {
                flow: Flow::Down,
                spacing: 1.0,
                ..Layout::default()
            },
        );

        for item_idx in visible.iter() {
            let item = &self.items[*item_idx];
            let is_hovered = self.hovered_item == Some(*item_idx);
            let is_selected = self.selected_item == Some(*item_idx);
            let is_disabled = item.disabled;
            let has_children = self.has_children(*item_idx);
            let is_open = self.expanded.contains(item_idx);

            self.draw_row.hovered = if is_hovered { 1.0 } else { 0.0 };
            self.draw_row.selected = if is_selected { 1.0 } else { 0.0 };
            self.draw_row.disabled = if is_disabled { 1.0 } else { 0.0 };
            // Disabled rows never show hover/selected washes
            self.draw_row.hovered = self.draw_row.hovered * (1.0 - self.draw_row.disabled);
            self.draw_row.selected = self.draw_row.selected * (1.0 - self.draw_row.disabled);
            // Disabled label color resolves to the muted token
            let label_color = if is_disabled {
                self.c_text_muted
            } else {
                self.c_text
            };
            self.draw_label.color = label_color;

            self.draw_row.begin(
                cx,
                Walk::new(Size::fill(), Size::Fixed(self.row_height)),
                Layout {
                    flow: Flow::right(),
                    align: Align { x: 0.0, y: 0.5 },
                    padding: Inset {
                        left: 8.0 + item.depth as f64 * self.indent_step,
                        right: 8.0,
                        ..Default::default()
                    },
                    ..Layout::default()
                },
            );

            // Chevron gutter for branch nodes
            if has_children {
                let glyph = if is_open { "▾" } else { "▸" };
                self.draw_chevron.draw_walk(
                    cx,
                    Walk {
                        width: Size::Fixed(self.indent_step.min(14.0)),
                        height: Size::fit(),
                        ..Walk::default()
                    },
                    Align::default(),
                    glyph,
                );
                self.chevron_areas.push((self.draw_chevron.area(), *item_idx));
            } else {
                // Leaf: reserve the same gutter so labels align across depths
                self.draw_label.draw_walk(
                    cx,
                    Walk {
                        width: Size::Fixed(self.indent_step.min(14.0)),
                        height: Size::fit(),
                        ..Walk::default()
                    },
                    Align::default(),
                    "",
                );
            }

            self.draw_label
                .draw_walk(cx, Walk::fit(), Align::default(), &item.label);

            self.draw_row.end(cx);
            self.row_areas.push((self.draw_row.area(), *item_idx));
        }

        cx.end_turtle();
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

impl MpTreeRef {
    pub fn set_items(&self, cx: &mut Cx, items: Vec<TreeItem>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn set_expanded(&self, cx: &mut Cx, idx: usize, open: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_expanded(cx, idx, open);
        }
    }

    pub fn item_selected(&self, actions: &Actions) -> Option<usize> {
        if let Some(inner) = self.borrow() {
            inner.item_selected(actions)
        } else {
            None
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_assigns_depths_by_nesting() {
        let flat = flatten_tree(vec![
            TreeItem::with_children(
                "src",
                vec![
                    TreeItem::with_children(
                        "widgets",
                        vec![
                            TreeItem::new("button.rs", 0),
                            TreeItem::new("tree.rs", 0).disabled(true),
                        ],
                    ),
                    TreeItem::new("main.rs", 0),
                ],
            ),
            TreeItem::new("Cargo.toml", 0),
        ]);
        let labels: Vec<(&str, usize, bool)> = flat
            .iter()
            .map(|i| (i.label.as_str(), i.depth, i.disabled))
            .collect();
        assert_eq!(
            labels,
            vec![
                ("src", 0, false),
                ("widgets", 1, false),
                ("button.rs", 2, false),
                ("tree.rs", 2, true),
                ("main.rs", 1, false),
                ("Cargo.toml", 0, false),
            ]
        );
        // Flattened rows carry no children
        assert!(flat.iter().all(|i| i.children.is_empty()));
    }

    #[test]
    fn disabled_rows_reported_on_item() {
        let mut item = TreeItem::new("wip.rs", 0);
        assert!(!item.disabled);
        item = item.disabled(true);
        assert!(item.disabled);
        // gpui-style builder keeps disabled flag through flatten
        let flat = flatten_tree(vec![TreeItem::with_children(
            "branch",
            vec![TreeItem::new("a", 0).disabled(true)],
        )]);
        assert!(flat[1].disabled);
        assert!(!flat[0].disabled);
    }
}
