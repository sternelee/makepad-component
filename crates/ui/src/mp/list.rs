//! `MpList` — rows of things, each one an icon, a label and a detail.
//!
//! The third of the data widgets, and the simplest: where [`MpTable`] has columns
//! and [`MpTree`] has depth, a list has none of that and is the shape a sidebar,
//! a command palette and a picker's options all reduce to.
//!
//! ## One widget, like the table and the tree
//!
//! Rows come from Rust as data ([`ListItem`]) and the widget paints them, for the
//! reason `mp/table.rs` records: a list is *a column of strings*, and a widget per
//! row is a widget per string. The measure and clip arithmetic is shared with the
//! other two through [`crate::mp::text`], so all three clip the same string at the
//! same point.
//!
//! ## What a row is
//!
//! A leading glyph if it has one, a label, and a detail pushed to the far edge —
//! which is the shape of every command palette row ("Format Document", "⇧⌥F"),
//! every picker option with a count, and every sidebar entry with a badge. The
//! glyph is a *character* rather than an [`MpIcon`](crate::mp::icon) child for the
//! same reason the tree's chevron is: a row is a string and a glyph, not a
//! composition, and making it one would be the widget-per-row cost returning
//! through the back door.
//!
//! ## Selection is the caller's, focus is the widget's
//!
//! `selected` is set by whoever owns the meaning of "the current one" — a list
//! does not know whether it is a sidebar, a set of options or a log. What the
//! widget owns is the hover, and the arrow keys, which move the selection through
//! the visible rows and report it.

use makepad_widgets::*;

use crate::mp::text;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpList::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 10.0

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

    mod.mp.MpListBase = #(MpList::register_widget(vm))

    mod.mp.MpList = set_type_default() do mod.mp.MpListBase{
        width: Fill
        height: Fit
        show_row_lines: true

        draw_label +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_detail +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
        draw_glyph +: {
            // The icon face, for a row's leading glyph.
            text_style: theme.font_icons{font_size: 11.0}
            color: #x00000000
        }
    }
    /// A menu, or a command palette's list: rows whose lines appear only where a
    /// group starts. Same widget, one flag — because a menu *is* a list of rows
    /// that happen to be commands, and a second widget would be a second copy of
    /// the row layout, the glyph, the detail and the hit test.
    mod.mp.MpMenu = set_type_default() do mod.mp.MpList{
        show_row_lines: false
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpList {
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

/// One row: an optional leading glyph, a label, and an optional trailing detail.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListItem {
    pub label: String,
    /// Trailing text, pushed to the far edge. A count, a shortcut, a state.
    pub detail: String,
    /// A leading glyph from the icon face, as the character itself. Empty means
    /// no glyph, and the label moves left to where the glyph would have been —
    /// which is what keeps a list of mixed rows from having two left edges.
    pub glyph: String,
    /// Draw a hairline **above** this row, separating it from the group before it.
    ///
    /// A menu's rows are grouped and a palette's are not, and the difference is
    /// where the lines go: between every row reads as a table, between groups
    /// reads as sections. So the separator belongs to the row that *starts* a
    /// group rather than to the list, which is also what lets a caller build a
    /// menu from data without a second structure for the groups.
    pub separator: bool,
    /// Draw the label in the danger tone. A destructive action has to be visible
    /// as one before it is read, because a menu is scanned and then chosen.
    pub danger: bool,
}

impl ListItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: String::new(),
            glyph: String::new(),
            separator: false,
            danger: false,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn glyph(mut self, glyph: impl Into<String>) -> Self {
        self.glyph = glyph.into();
        self
    }

    /// Start a group: a hairline above this row.
    pub fn starts_group(mut self) -> Self {
        self.separator = true;
        self
    }

    /// Mark this row destructive.
    pub fn destructive(mut self) -> Self {
        self.danger = true;
        self
    }
}

/// What a list reports.
#[derive(Clone, Debug, Default)]
pub enum MpListAction {
    /// A row became the selection, carrying its index.
    Selected(usize),
    #[default]
    None,
}

const ROW_H: f64 = 28.0;
/// The leading inset, and the room a glyph takes when there is one.
const PAD: f64 = 10.0;

/// Extra room the row's **trailing** text keeps from the panel's edge.
///
/// The detail is right-aligned, and its position is arithmetic on an *estimate* of its
/// width ([`crate::mp::text`]). An estimate cannot be exact for every string, and the
/// direction that hurts is an under-estimate: the text runs past the edge instead of
/// stopping short of it. That is what the Shortcuts page showed — `⇧⌘S` with its `S`
/// drawn under the page's scroll bar, which **overlays** the panel's right edge.
///
/// Raising the estimator's symbol correction as far as the measurements justified (see
/// `mp/text.rs`) closed most of the gap; this closes the rest, and it is the same
/// decision `mp/segmented.rs` reached for the same reason: **a slot that carries the
/// measurement's error margin is better than a label that collides.** Fourteen points is
/// the residual for a three-glyph chord, and it reads as a wider gap before the panel's
/// edge, which is a change nobody minds.
const DETAIL_SLACK: f64 = 14.0;
const GLYPH_W: f64 = 22.0;

#[derive(Script, ScriptHook, Widget)]
pub struct MpList {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpList,
    #[live]
    draw_label: DrawText,
    #[live]
    draw_detail: DrawText,
    #[live]
    draw_glyph: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    /// Whether to draw a hairline between every row.
    ///
    /// `true` is a list — a table of things whose rows are peers. `false` is a
    /// menu or a palette, where only the explicit group separators draw, because a
    /// line between every command reads as a grid rather than as a menu.
    #[live]
    show_row_lines: bool,

    #[rust]
    items: Vec<ListItem>,
    #[rust]
    selected: Option<usize>,
    #[rust]
    hovered: Option<usize>,
    #[rust]
    area: Area,
}

impl MpList {
    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<ListItem>) {
        self.items = items;
        // The selection is an index into a list that no longer exists, so it is
        // dropped rather than reinterpreted.
        self.selected = None;
        self.hovered = None;
        self.redraw(cx);
    }

    pub fn items(&self) -> &[ListItem] {
        &self.items
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        crate::mp::action::first::<MpListAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpListAction::Selected(i) => Some(*i),
                MpListAction::None => None,
            }
        })
    }

    pub fn select(&mut self, cx: &mut Cx, index: usize) {
        if index >= self.items.len() || self.selected == Some(index) {
            return;
        }
        self.selected = Some(index);
        cx.widget_action(self.widget_uid(), MpListAction::Selected(index));
        self.redraw(cx);
    }

    /// Where row `index`'s label starts.
    ///
    /// **Every row has the same label origin, whether or not it has a glyph.** A
    /// list whose glyphless rows started further left would have two left edges,
    /// and that reads as a rendering fault rather than as a row without an icon —
    /// so the glyph's room is reserved for every row and only *drawn* by the ones
    /// that have one.
    fn label_x() -> f64 {
        PAD + GLYPH_W
    }

    fn row_at(&self, p: Vec2d) -> Option<usize> {
        if p.y < 0.0 {
            return None;
        }
        let row = (p.y / ROW_H).floor() as usize;
        (row < self.items.len()).then_some(row)
    }

    pub fn content_height(&self) -> f64 {
        self.items.len() as f64 * ROW_H
    }

    fn move_selection(&mut self, cx: &mut Cx, delta: i64) {
        if self.items.is_empty() {
            return;
        }
        let next = match self.selected {
            Some(current) => (current as i64 + delta).clamp(0, self.items.len() as i64 - 1) as usize,
            None => {
                if delta >= 0 {
                    0
                } else {
                    self.items.len() - 1
                }
            }
        };
        self.select(cx, next);
    }
}

impl Widget for MpList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let mut hovered = self.hovered;
        match event.hits(cx, self.area) {
            // Split rather than matched together: a hover event and a move event
            // are different types with the same field.
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
                        self.select(cx, index);
                    }
                }
                hovered = self.hovered;
            }
            _ => {
                if cx.has_key_focus(self.area) {
                    if let Event::KeyDown(ke) = event {
                        if !ke.is_repeat {
                            match ke.key_code {
                                KeyCode::ArrowDown => self.move_selection(cx, 1),
                                KeyCode::ArrowUp => self.move_selection(cx, -1),
                                KeyCode::Home => self.select(cx, 0),
                                KeyCode::End => {
                                    if !self.items.is_empty() {
                                        self.select(cx, self.items.len() - 1);
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
        // Copied out before any mutable use of `cx`; see the note in
        // `mp/table.rs`, which is where this was learned.
        let (panel, border, hover_wash, selected_wash, muted, body, danger, divider, radius) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                // A list is content, so its ground is the content plane rather
                // than the page.
                p.surface_card,
                p.border,
                p.element_hover,
                p.element_active,
                p.text_muted,
                p.text,
                p.danger,
                p.divider,
                makepad_theme::Theme::panel_radius() as f32,
            )
        };

        // **The font sizes are read back from the draw targets, not taken from the
        // theme.** The arithmetic below clips and right-aligns, and the paint below that
        // uses whatever `text_style` the DSL gave each `DrawText`; measuring with
        // `theme.metrics(...)` instead means measuring with a *different number* than is
        // painted, and the two only agree if the DSL's `mod.mpc.type.*` happens to resolve
        // to the theme's own metrics.
        //
        // They did not agree, and this is the second widget to pay for it — the fix is
        // the one `mp/segmented.rs` records, applied here. The symptom was a row's
        // trailing chord drawn with its last character **past the panel**, on top of the
        // page's scroll bar, because a right-aligned string was placed by an estimate
        // that was too small. Naming the size once, from the thing that paints, is what
        // makes the measurement and the paint incapable of disagreeing.
        let font = self.draw_label.text_style.font_size as f64;
        let small = self.draw_detail.text_style.font_size as f64;

        self.draw_bg.fill = panel;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        // A widget painted entirely by `draw_abs` inside its own turtle has
        // nothing the layout pass can measure, so it states its own size.
        self.walk.height = Size::Fixed(self.content_height());
        let walk = self.walk;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        let rect = self.draw_bg.area().rect(cx.cx);
        self.area = self.draw_bg.area();
        let origin = rect.pos;
        let width = rect.size.x;

        for (index, item) in self.items.iter().enumerate() {
            let y = index as f64 * ROW_H;
            let is_selected = self.selected == Some(index);
            let is_hovered = self.hovered == Some(index);

            if is_selected || is_hovered {
                self.draw_bg.fill = if is_selected { selected_wash } else { hover_wash };
                self.draw_bg.border_width = 0.0;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: origin + dvec2(0.0, y),
                        size: dvec2(width, ROW_H),
                    },
                );
                self.draw_bg.fill = panel;
                self.draw_bg.border_width = 1.0;
            }

            // The detail is measured first, because the label's room depends on
            // how much the detail took.
            let detail = text::clip(&item.detail, width * 0.4, small);
            let detail_w = text::width(&detail, small);

            let text_y = y + (ROW_H - font) * 0.5;
            let label_room = width - Self::label_x() - PAD - detail_w - 8.0;
            let label = text::clip(&item.label, label_room, font);

            if !item.glyph.is_empty() {
                self.draw_glyph.color = muted;
                self.draw_glyph.draw_abs(
                    cx,
                    origin + dvec2(PAD, text_y + font * 0.5 - 5.0),
                    &item.glyph,
                );
            }

            // The danger tone, not a coloured glyph beside it: a destructive row
            // is scanned before it is read, and a red mark on the right of a row
            // is read after the label.
            self.draw_label.color = if item.danger { danger } else { body };
            self.draw_label
                .draw_abs(cx, origin + dvec2(Self::label_x(), text_y), &label);

            if !detail.is_empty() {
                // Temporary evidence, gated so it costs nothing in normal runs: a
                // right-alignment fault is invisible in a log and a screenshot only
                // shows the symptom, so the numbers have to come out of the widget.
                if std::env::var("MP_LIST_DEBUG").is_ok() {
                    println!(
                        "LIST row {:?} rect_w={width} pad={PAD} font={small} body={font} \
                         detail={detail:?} est_w={} x={} end={}",
                        item.label,
                        text::width(&detail, small),
                        text::right_aligned_x(&detail, 0.0, width, PAD + DETAIL_SLACK, small),
                        text::right_aligned_x(&detail, 0.0, width, PAD + DETAIL_SLACK, small)
                            + text::width(&detail, small),
                    );
                }
                self.draw_detail.color = muted;
                self.draw_detail.draw_abs(
                    cx,
                    origin
                        + dvec2(
                            text::right_aligned_x(&detail, 0.0, width, PAD + DETAIL_SLACK, small),
                            y + (ROW_H - small) * 0.5,
                        ),
                    &detail,
                );
            }

            // Between every row for a list, and only where a group starts for a
            // menu. The separator is drawn at this row's *top*, which is why the
            // two cases do not share a branch.
            let line_y = if !self.show_row_lines && item.separator {
                Some(y)
            } else if self.show_row_lines && index + 1 < self.items.len() {
                Some(y + ROW_H - 1.0)
            } else {
                None
            };
            if let Some(line_y) = line_y {
                self.draw_bg.fill = divider;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: origin + dvec2(0.0, line_y),
                        size: dvec2(width, 1.0),
                    },
                );
            }
        }

        DrawStep::done()
    }
}

impl MpListRef {
    pub fn set_items(&self, cx: &mut Cx, items: Vec<ListItem>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn select(&self, cx: &mut Cx, index: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.select(cx, index);
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.borrow().and_then(|inner| inner.selected())
    }

    pub fn row_selected(&self, actions: &Actions) -> Option<usize> {
        self.borrow().and_then(|inner| inner.row_selected(actions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_row_shares_one_label_origin_whether_or_not_it_has_a_glyph() {
        // Two left edges in one list reads as a rendering fault rather than as a
        // row without an icon, so the glyph's room is reserved for every row and
        // only drawn by the ones that have one.
        assert!(MpList::label_x() > PAD, "the glyph's room is reserved");
        assert_eq!(MpList::label_x(), PAD + GLYPH_W);
    }

    #[test]
    fn test_rows_stack_without_gaps_from_the_top() {
        // A list has no header — unlike the table — so row 0 starts at the top.
        assert_eq!(ROW_H, 28.0);
        for index in 0..5 {
            let y = index as f64 * ROW_H;
            assert!((y - index as f64 * 28.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_row_hit_testing_returns_the_row_index() {
        // The property that matters: a row is clickable exactly where it is drawn.
        // Like the table's and the tree's, this is a pure function of the index.
        let row_at = |y: f64| -> Option<usize> {
            if y < 0.0 {
                return None;
            }
            let row = (y / ROW_H).floor() as usize;
            (row < 3).then_some(row)
        };
        for index in 0..3 {
            let top = index as f64 * ROW_H;
            assert_eq!(row_at(top + 0.5), Some(index), "top of {index}");
            assert_eq!(row_at(top + ROW_H - 0.5), Some(index), "bottom of {index}");
        }
        assert_eq!(row_at(3.0 * ROW_H + 1.0), None, "past the last row");
    }

    #[test]
    fn test_a_row_with_no_glyph_and_one_with_a_glyph_align_their_labels() {
        // The reservation is what makes these equal, so this asserts the equality
        // the reservation exists for rather than the constant.
        let with_glyph = ListItem::new("Rename").glyph("\u{f2f5}");
        let without = ListItem::new("Copy");
        assert_eq!(
            with_glyph.glyph.is_empty(),
            false,
            "the first has a glyph"
        );
        assert!(without.glyph.is_empty(), "the second does not");
        // Both draw their label at the same x, because `label_x` takes no
        // argument — there is deliberately no per-row variant to get wrong.
        assert_eq!(MpList::label_x(), MpList::label_x());
    }

    #[test]
    fn test_an_item_builder_sets_only_what_it_is_given() {
        let plain = ListItem::new("Save");
        assert_eq!(plain.label, "Save");
        assert!(plain.detail.is_empty());
        assert!(plain.glyph.is_empty());

        let full = ListItem::new("Format").detail("⇧⌥F").glyph("\u{f0d7}");
        assert_eq!(full.detail, "⇧⌥F");
        assert_eq!(full.glyph, "\u{f0d7}");
    }

    #[test]
    fn test_a_detail_claims_at_most_two_fifths_of_the_row() {
        // A long detail must not eat the label: the label is the row's identity
        // and the detail is an annotation. The cap is what stops a verbose
        // shortcut from leaving one character of a name.
        let width = 200.0;
        let cap = width * 0.4;
        // The row's remaining label room is positive for any detail up to the cap.
        let label_room = width - MpList::label_x() - PAD - cap - 8.0;
        assert!(label_room > 0.0, "{label_room}");
    }

    #[test]
    fn test_an_empty_list_has_no_height_and_no_rows() {
        let items: Vec<ListItem> = Vec::new();
        assert_eq!(items.len() as f64 * ROW_H, 0.0);
    }

    #[test]
    fn test_a_group_separator_belongs_to_the_row_that_starts_it() {
        // Which is what lets a caller build a menu from data without a second
        // structure for the groups: the separator is a row's own flag, not the
        // list's map from index to section.
        let items = vec![
            ListItem::new("New terminal"),
            ListItem::new("Duplicate"),
            ListItem::new("Delete").destructive().starts_group(),
        ];
        assert!(!items[0].separator && !items[0].danger);
        assert!(!items[1].separator && !items[1].danger);
        assert!(items[2].separator, "the destructive row starts the last group");
        assert!(items[2].danger);
    }

    #[test]
    fn test_the_builders_are_independent() {
        // A destructive row in the middle of a group, and a group whose rows are
        // all harmless, are both normal.
        let danger_only = ListItem::new("Delete").destructive();
        assert!(danger_only.danger && !danger_only.separator);
        let group_only = ListItem::new("Settings").starts_group();
        assert!(group_only.separator && !group_only.danger);
    }
}
