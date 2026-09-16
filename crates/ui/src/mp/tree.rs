//! `MpTree` — a flat list that knows its own hierarchy.
//!
//! ## The model is flat, and that is the design
//!
//! A tree is stored as a **flat list where each item carries its depth**, and the
//! widget owns which items are collapsed. Not a nested structure.
//!
//! The reason is the same one bezel gives for its markdown model: every operation
//! a tree actually supports is a *list* operation on a flat list — expand, hide
//! a run of deeper items, move the selection to the next visible row — and each
//! of those is a restructure on a tree. This is also what the v2 tree arrived at
//! (`flatten_tree`), and what Makepad's own `FileTree` renders.
//!
//! Depth is a number rather than a path, which keeps the model flat while an
//! indent level still means something. An item is inside the nearest preceding
//! item with a *smaller* depth — no parent pointers, and therefore nothing to
//! keep consistent when the list is replaced.
//!
//! ## Visibility is one function
//!
//! [`MpTree::visible_rows`] answers which items are on screen and at what depth,
//! and the painter, the hover, the click and the keyboard all read it. A row that
//! is drawn but not hit-testable — or hit-testable but not drawn — is the fault
//! this crate has already fixed three times, and the rule that stops it is that
//! the answer comes from one place.

use makepad_widgets::*;

use crate::mp::text;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpTree::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 8.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, self.rect_size.y - bw * 2.0, max(1.0, self.radius))
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }
            return sdf.result
        }
    }

    mod.mp.MpTreeBase = #(MpTree::register_widget(vm))

    mod.mp.MpTree = set_type_default() do mod.mp.MpTreeBase{
        width: Fill
        height: Fit

        draw_bg +: {}
        draw_label +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_chevron +: {
            // The icon face, for the disclosure triangle. Same font `MpIcon`
            // uses, drawn here rather than as a child widget because a row is a
            // string and a glyph rather than a composition.
            text_style: theme.font_icons{font_size: 9.0}
            color: #x00000000
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpTree {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    fill: Vec4f,
    #[live]
    border: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
}

/// One row: what it says, and how deep it sits.
///
/// `depth` is the number of ancestors, so a root item is `0`. Nothing checks that
/// a list's depths are *well formed* — an item whose depth jumps by three is
/// drawn three levels in, because that is what it asked for, and a tree that
/// silently corrected it would be hiding a caller's bug.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeItem {
    pub label: String,
    pub depth: usize,
}

impl TreeItem {
    pub fn new(label: impl Into<String>, depth: usize) -> Self {
        Self {
            label: label.into(),
            depth,
        }
    }
}

/// What a tree reports.
#[derive(Clone, Debug, Default)]
pub enum MpTreeAction {
    /// A row became the selection, carrying its index in the *item* list.
    Selected(usize),
    /// A row was expanded or collapsed, carrying its index and its new state.
    Toggled(usize, bool),
    #[default]
    None,
}

/// The collapsed set, as a named type.
///
/// A type alias rather than the path inline, because `#[derive(Script)]`'s field
/// parser is token-based and cannot read a fully-qualified generic type — the
/// same gotcha `AGENTS.md` records from dbpro, whose note is "commas inside
/// generic field types break parsing; use type aliases". `HashSet<usize>` has no
/// comma and still fails, so the restriction is broader than that note says: any
/// non-trivial generic path is safer as an alias.
type Collapsed = std::collections::HashSet<usize>;

/// The row height, the indent per level, and the room a chevron takes.
const ROW_H: f64 = 26.0;
const INDENT: f64 = 16.0;
const CHEVRON_W: f64 = 16.0;
const PAD: f64 = 8.0;

#[derive(Script, ScriptHook, Widget)]
pub struct MpTree {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpTree,
    #[live]
    draw_label: DrawText,
    #[live]
    draw_chevron: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[rust]
    items: Vec<TreeItem>,
    // Which items are collapsed, by index into `items`.
    #[rust]
    collapsed: Collapsed,
    #[rust]
    selected: Option<usize>,
    #[rust]
    hovered: Option<usize>,
    #[rust]
    area: Area,
}

impl MpTree {
    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<TreeItem>) {
        self.items = items;
        // A collapsed set from the *previous* list is indices into a list that no
        // longer exists, so it is dropped rather than reinterpreted — an item
        // that happened to share an index would silently inherit a state it never
        // had.
        self.collapsed.clear();
        self.hovered = None;
        self.selected = None;
        self.redraw(cx);
    }

    pub fn items(&self) -> &[TreeItem] {
        &self.items
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        crate::mp::action::first::<MpTreeAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpTreeAction::Selected(i) => Some(*i),
                _ => None,
            }
        })
    }

    pub fn row_toggled(&self, actions: &Actions) -> Option<(usize, bool)> {
        crate::mp::action::first::<MpTreeAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpTreeAction::Toggled(i, open) => Some((*i, *open)),
                _ => None,
            }
        })
    }

    /// Whether item `index` has anything under it.
    ///
    /// Derived rather than stored: an item has children exactly when the next
    /// item is deeper. A `has_children` field would be a second copy of the same
    /// fact, and the two would disagree the first time a list was edited.
    pub fn has_children(&self, index: usize) -> bool {
        self.items
            .get(index + 1)
            .is_some_and(|next| next.depth > self.items[index].depth)
    }

    pub fn is_collapsed(&self, index: usize) -> bool {
        self.collapsed.contains(&index)
    }

    /// The last item belonging to `index`'s subtree, whatever its depth.
    ///
    /// An item's subtree is the maximal run of following items that are deeper.
    fn subtree_end(&self, index: usize) -> usize {
        let depth = self.items[index].depth;
        let mut end = index + 1;
        while end < self.items.len() && self.items[end].depth > depth {
            end += 1;
        }
        end
    }

    /// Which items are on screen, as `(item index, depth)`.
    ///
    /// An item is visible when no ancestor is collapsed. Because depth is a
    /// number rather than a path, "no ancestor is collapsed" is a walk that
    /// remembers the shallowest collapsed depth it has passed: anything deeper
    /// than that is hidden.
    pub fn visible_rows(&self) -> Vec<(usize, usize)> {
        let mut rows = Vec::with_capacity(self.items.len());
        // The depth of the innermost collapsed ancestor, if any. `None` means
        // nothing above is collapsed.
        let mut hiding: Option<usize> = None;
        for (index, item) in self.items.iter().enumerate() {
            if let Some(depth) = hiding {
                if item.depth > depth {
                    continue;
                }
                // Back out to a depth at or above the collapsed one: that
                // subtree's run has ended.
                hiding = None;
            }
            rows.push((index, item.depth));
            if self.collapsed.contains(&index) && self.has_children(index) {
                hiding = Some(item.depth);
            }
        }
        rows
    }

    pub fn toggle(&mut self, cx: &mut Cx, index: usize) {
        if !self.has_children(index) {
            // A leaf has nothing to disclose, so this is a no-op rather than a
            // state change: a chevron that turned on a leaf would be a control
            // that does nothing, and callers cannot be asked to check first.
            return;
        }
        let open = if self.collapsed.remove(&index) {
            true
        } else {
            self.collapsed.insert(index);
            false
        };
        cx.widget_action(self.widget_uid(), MpTreeAction::Toggled(index, open));
        self.redraw(cx);
    }

    pub fn set_collapsed(&mut self, cx: &mut Cx, index: usize, collapsed: bool) {
        let changed = if collapsed {
            self.collapsed.insert(index)
        } else {
            self.collapsed.remove(&index)
        };
        if changed {
            self.redraw(cx);
        }
    }

    pub fn select(&mut self, cx: &mut Cx, index: usize) {
        if self.selected == Some(index) || index >= self.items.len() {
            return;
        }
        self.selected = Some(index);
        cx.widget_action(self.widget_uid(), MpTreeAction::Selected(index));
        self.redraw(cx);
    }

    /// The drawn row a point is over, as an item index.
    fn row_at(&self, p: Vec2d) -> Option<usize> {
        if p.y < 0.0 {
            return None;
        }
        let row = (p.y / ROW_H).floor() as usize;
        self.visible_rows().get(row).map(|(index, _)| *index)
    }

    /// Where a row's label starts, given its depth.
    ///
    /// One function for the same reason the table has one: a label indented by
    /// one number and hit-tested by another puts the clickable region somewhere
    /// the text is not.
    fn label_x(depth: usize) -> f64 {
        PAD + depth as f64 * INDENT + CHEVRON_W
    }

    pub fn content_height(&self) -> f64 {
        self.visible_rows().len() as f64 * ROW_H
    }

    /// Move the selection by `delta` rows through the *visible* list.
    ///
    /// Through the visible list rather than the item list, because a collapsed
    /// subtree is not on screen and an arrow key that stepped into it would move
    /// a selection nobody can see.
    fn move_selection(&mut self, cx: &mut Cx, delta: i64) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return;
        }
        let current = self
            .selected
            .and_then(|sel| rows.iter().position(|(i, _)| *i == sel));
        let next = match current {
            Some(pos) => (pos as i64 + delta).clamp(0, rows.len() as i64 - 1) as usize,
            // Nothing selected: an arrow key starts at the first row, or the last
            // for an upward one — the behaviour every list has.
            None => {
                if delta >= 0 {
                    0
                } else {
                    rows.len() - 1
                }
            }
        };
        let index = rows[next].0;
        if self.selected != Some(index) {
            self.selected = Some(index);
            cx.widget_action(self.widget_uid(), MpTreeAction::Selected(index));
            self.redraw(cx);
        }
    }
}

impl Widget for MpTree {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let mut hovered = self.hovered;
        match event.hits(cx, self.area) {
            // Split rather than matched together: a hover event and a move event
            // are different types with the same field, and Rust cannot bind one
            // name to both.
            Hit::FingerHoverIn(fe) => {
                hovered = self.row_at(fe.abs - self.area.rect(cx).pos);
            }
            Hit::FingerMove(fe) => {
                hovered = self.row_at(fe.abs - self.area.rect(cx).pos);
            }
            Hit::FingerHoverOut(_) => hovered = None,
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    if let Some(index) = self.row_at(fe.abs - self.area.rect(cx).pos) {
                        // A press on the chevron discloses; a press anywhere else
                        // on the row selects. That split is what makes a tree
                        // navigable without a separate control per level.
                        let x = fe.abs.x - self.area.rect(cx).pos.x;
                        let depth = self.items[index].depth;
                        let chevron_end = PAD + depth as f64 * INDENT + CHEVRON_W;
                        if self.has_children(index) && x < chevron_end {
                            self.toggle(cx, index);
                        } else {
                            self.select(cx, index);
                        }
                    }
                }
                hovered = self.hovered;
            }
            _ => {
                // Keyboard, and only while focused. Read from the raw event
                // rather than from a `Hit`, because a key is not a hit on an
                // area.
                if cx.has_key_focus(self.area) {
                    if let Event::KeyDown(ke) = event {
                        if !ke.is_repeat {
                            match ke.key_code {
                                KeyCode::ArrowDown => self.move_selection(cx, 1),
                                KeyCode::ArrowUp => self.move_selection(cx, -1),
                                KeyCode::ArrowRight => {
                                    if let Some(index) = self.selected {
                                        self.set_collapsed(cx, index, false);
                                    }
                                }
                                KeyCode::ArrowLeft => {
                                    if let Some(index) = self.selected {
                                        if self.has_children(index) && !self.is_collapsed(index) {
                                            self.set_collapsed(cx, index, true);
                                        } else if let Some(parent) =
                                            self.parent_of(index)
                                        {
                                            // Already collapsed (or a leaf): go up
                                            // to the parent, which is what every
                                            // outline view does.
                                            self.select(cx, parent);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                return;
            }
        }
        if hovered != self.hovered {
            self.hovered = hovered;
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Copied out before any mutable use of `cx`: `Theme::of` borrows it, and
        // holding that borrow across a draw call does not compile. See the note
        // in `mp/table.rs`, which is where this was learned.
        let (card, border, hover_wash, band, muted, body, selected_wash, radius) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.surface_card,
                p.border,
                p.element_hover,
                p.band,
                p.text_muted,
                p.text,
                // The selection wash. `element_active` is the same rung every
                // selected row in the library paints, so a tree row and a rail
                // row read as the same state.
                p.element_active,
                makepad_theme::Theme::panel_radius() as f32,
            )
        };

        self.draw_bg.fill = card;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        // A widget painted entirely by `draw_abs` inside its own turtle has
        // nothing the layout pass can measure, so `height: Fit` measures zero and
        // every row lands in a zero-height box — which looks exactly like a tree
        // that does not draw. It states its own size.
        self.walk.height = Size::Fixed(self.content_height());
        let walk = self.walk;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        let rect = self.draw_bg.area().rect(cx.cx);
        self.area = self.draw_bg.area();
        let origin = rect.pos;
        let width = rect.size.x;

        let line_box = {
            let theme = makepad_theme::Theme::of(cx.cx);
            theme.metrics(makepad_theme::TextStyle::Body).line_height() as f64
        };

        self.draw_label.color = body;
        self.draw_chevron.color = muted;

        for (row, (index, depth)) in self.visible_rows().into_iter().enumerate() {
            let y = row as f64 * ROW_H;
            let is_selected = self.selected == Some(index);
            let is_hovered = self.hovered == Some(index);

            if is_selected || is_hovered {
                // The band under the whole row, so a deep row's selection still
                // reads as a row rather than as a length of text.
                self.draw_bg.fill = if is_selected { selected_wash } else { hover_wash };
                self.draw_bg.border_width = 0.0;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: origin + dvec2(0.0, y),
                        size: dvec2(width, ROW_H),
                    },
                );
                self.draw_bg.fill = card;
                self.draw_bg.border_width = 1.0;
            }

            let text_y = y + (ROW_H - line_box) * 0.5;
            let label = self.items[index].label.clone();

            if self.has_children(index) {
                // A filled triangle when open, a right-pointing one when shut —
                // the disclosure convention every outline view shares.
                let glyph = if self.is_collapsed(index) {
                    "\u{f0da}"
                } else {
                    "\u{f0d7}"
                };
                self.draw_chevron.draw_abs(
                    cx,
                    origin
                        + dvec2(
                            PAD + depth as f64 * INDENT,
                            text_y + line_box * 0.5 - 4.5,
                        ),
                    glyph,
                );
            }

            // The label clipped to the row, so a deep name does not run past the
            // plate's edge.
            let avail = width - Self::label_x(depth) - PAD;
            let label = clip_to_width(&label, avail, self.draw_label.text_style.font_size as f64);
            self.draw_label
                .draw_abs(cx, origin + dvec2(Self::label_x(depth), text_y), &label);
        }

        DrawStep::done()
    }
}

impl MpTree {
    /// Item `index`'s parent, as the nearest preceding item with a smaller depth.
    fn parent_of(&self, index: usize) -> Option<usize> {
        let depth = self.items.get(index)?.depth;
        if depth == 0 {
            return None;
        }
        (0..index).rev().find(|i| self.items[*i].depth < depth)
    }
}

/// `text` cut to `width` points at `font_size`.
///
/// Delegates to [`crate::mp::text`], which is where the arithmetic lives for all
/// three row-painting widgets.
fn clip_to_width(value: &str, width: f64, font_size: f64) -> String {
    text::clip(value, width, font_size)
}

impl MpTreeRef {
    pub fn set_items(&self, cx: &mut Cx, items: Vec<TreeItem>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn set_collapsed(&self, cx: &mut Cx, index: usize, collapsed: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_collapsed(cx, index, collapsed);
        }
    }

    pub fn select(&self, cx: &mut Cx, index: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.select(cx, index);
        }
    }

    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        self.borrow().and_then(|inner| inner.row_selected(actions))
    }

    pub fn row_toggled(&self, actions: &Actions) -> Option<(usize, bool)> {
        self.borrow().and_then(|inner| inner.row_toggled(actions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pure model, exercised without a widget: building an `MpTree` needs a
    /// script VM for its drawers, and none of what is worth testing involves one.
    #[derive(Debug, Default)]
    struct Model {
        items: Vec<TreeItem>,
        collapsed: Collapsed,
    }

    impl Model {
        fn new(items: Vec<TreeItem>) -> Self {
            Self {
                items,
                collapsed: Default::default(),
            }
        }

        fn has_children(&self, index: usize) -> bool {
            self.items
                .get(index + 1)
                .is_some_and(|next| next.depth > self.items[index].depth)
        }

        fn visible_rows(&self) -> Vec<(usize, usize)> {
            let mut rows = Vec::new();
            let mut hiding: Option<usize> = None;
            for (index, item) in self.items.iter().enumerate() {
                if let Some(depth) = hiding {
                    if item.depth > depth {
                        continue;
                    }
                    hiding = None;
                }
                rows.push((index, item.depth));
                if self.collapsed.contains(&index) && self.has_children(index) {
                    hiding = Some(item.depth);
                }
            }
            rows
        }

        fn row_at(&self, y: f64) -> Option<usize> {
            if y < 0.0 {
                return None;
            }
            let row = (y / ROW_H).floor() as usize;
            self.visible_rows().get(row).map(|(index, _)| *index)
        }
    }

    /// src/
    ///   tree.rs
    ///   table.rs
    /// tests/
    fn sample() -> Model {
        Model::new(vec![
            TreeItem::new("src/", 0),
            TreeItem::new("tree.rs", 1),
            TreeItem::new("table.rs", 1),
            TreeItem::new("tests/", 0),
            TreeItem::new("tree.rs", 1),
        ])
    }

    #[test]
    fn test_everything_is_visible_until_something_is_collapsed() {
        let m = sample();
        let rows = m.visible_rows();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0], (0, 0));
        assert_eq!(rows[2], (2, 1));
    }

    #[test]
    fn test_collapsing_hides_exactly_its_subtree() {
        let mut m = sample();
        m.collapsed.insert(0);
        let rows = m.visible_rows();
        // `src/` stays; its two children go; `tests/` and its child stay.
        assert_eq!(rows, vec![(0, 0), (3, 0), (4, 1)]);
    }

    #[test]
    fn test_a_collapsed_sibling_does_not_hide_the_next_subtree() {
        // The fault a naive implementation has: hiding until the list ends, or
        // until a shallower item appears *without* clearing the flag, so the
        // second subtree disappears too.
        let mut m = sample();
        m.collapsed.insert(0);
        m.collapsed.insert(3);
        let rows = m.visible_rows();
        assert_eq!(rows, vec![(0, 0), (3, 0)]);
        // And clearing one brings its own children back without touching the
        // other's.
        m.collapsed.remove(&0);
        let rows = m.visible_rows();
        assert_eq!(rows, vec![(0, 0), (1, 1), (2, 1), (3, 0)]);
    }

    #[test]
    fn test_an_item_has_children_exactly_when_the_next_one_is_deeper() {
        // Derived rather than stored: a `has_children` field would be a second
        // copy of the same fact and would disagree the first time the list was
        // replaced.
        let m = sample();
        assert!(m.has_children(0), "src/ has two files");
        assert!(!m.has_children(1), "tree.rs is a leaf");
        assert!(m.has_children(3), "tests/ has a file");
        assert!(!m.has_children(4), "the last item is a leaf");
    }

    #[test]
    fn test_an_empty_list_has_no_rows_and_no_height() {
        let m = Model::new(Vec::new());
        assert!(m.visible_rows().is_empty());
        assert_eq!(m.row_at(0.0), None);
    }

    #[test]
    fn test_row_hit_testing_agrees_with_visible_rows() {
        // A row is clickable exactly where it is drawn — the fault this crate has
        // fixed three times. The index a point maps to must be the *item* index,
        // not the row's position, or a collapsed row above shifts every
        // selection.
        let mut m = sample();
        m.collapsed.insert(0);
        let rows = m.visible_rows();
        for (position, (index, _)) in rows.iter().enumerate() {
            let y = position as f64 * ROW_H + 0.5;
            assert_eq!(m.row_at(y), Some(*index), "row {position}");
        }
        // Past the last drawn row.
        assert_eq!(m.row_at(rows.len() as f64 * ROW_H + 1.0), None);
    }

    #[test]
    fn test_labels_indent_by_their_own_depth_only() {
        // The label's x and the chevron's x must both come from the depth, or a
        // deep row's text starts where its chevron is.
        assert!(MpTree::label_x(0) < MpTree::label_x(1));
        assert!(MpTree::label_x(1) < MpTree::label_x(2));
        assert!((MpTree::label_x(2) - MpTree::label_x(1) - INDENT).abs() < 1e-9);
        // The chevron sits before the label at every depth.
        for depth in 0..4 {
            let chevron = PAD + depth as f64 * INDENT;
            assert!(chevron + 8.0 <= MpTree::label_x(depth), "depth {depth}");
        }
    }

    #[test]
    fn test_a_label_wider_than_its_row_is_cut_with_an_ellipsis() {
        let long = "a-very-long-file-name-that-cannot-fit-in-its-row.rs";
        let cut = clip_to_width(long, 80.0, 13.0);
        assert!(cut.ends_with('…'), "{cut}");
        assert!(cut.chars().count() < long.chars().count());
        // A label that fits is untouched.
        assert_eq!(clip_to_width("tree.rs", 400.0, 13.0), "tree.rs");
        // No room at all is an empty string rather than a bare ellipsis.
        assert_eq!(clip_to_width(long, 0.0, 13.0), "");
    }

    #[test]
    fn test_a_shallow_tree_ignores_a_collapse_on_a_leaf() {
        // Collapsing a leaf is a no-op: nothing about the visible rows changes,
        // which is what stops a chevron that does nothing from also hiding a
        // sibling.
        let mut m = sample();
        m.collapsed.insert(1);
        assert_eq!(m.visible_rows().len(), 5);
    }

    #[test]
    fn test_a_depth_jump_is_drawn_as_asked_rather_than_corrected() {
        // Well-formedness is the caller's: an item that asks to be three levels
        // in is drawn three levels in, and a tree that silently corrected it
        // would be hiding a caller's bug.
        let m = Model::new(vec![TreeItem::new("a", 0), TreeItem::new("b", 3)]);
        let rows = m.visible_rows();
        assert_eq!(rows[1], (1, 3));
        assert_eq!(MpTree::label_x(3), PAD + 3.0 * INDENT + CHEVRON_W);
    }
}
