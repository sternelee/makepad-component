//! The panel a menu drops: [`Item`]s as rows, with submenus hanging off the rows the [`Cursor`] holds open.
//!
//! ## The geometry is arithmetic, so it is a function
//!
//! Every row's height, every row's top, the panel's width and which row a `y` lands on are pure functions of the items and
//! the theme, and they are the whole of this module's decisions:
//!
//! - **The glyph gutter is reserved only when a row uses it.** A menu where nothing carries a glyph keeps no room for one,
//!   which is what stops a menu bar's menus from opening with an empty column down their left — the sort of thing that
//!   looks like a bug and is impossible to unsee once it is there.
//! - **A described row is two lines tall and widens the panel to a *ceiling*, not a floor.** A description is a sentence
//!   rather than a name, so the panel widens the way one icon opens the gutter; and because the sentence is kept to one
//!   line, the panel needs a width to clip against rather than growing to whatever the longest one measures — a menu whose
//!   width depends on its longest description is a menu that jumps when the data changes.
//! - **A separator is a row with a height of its own**, not a gap: it is counted in the hit test like anything else, and it
//!   is what lets [`row_at`] answer "nothing" for the space between two groups.
//!
//! ## Dismissal is a question about *all* the panels
//!
//! The obvious implementation is an "out click" on the card, and it is wrong: the card sees only its own bounds, so a click
//! inside a submenu reads as a click away and closes the menu you were using. [`within`] takes every open panel's rect and
//! answers whether a point is inside any of them — which is the question the dismissal actually is.
//!
//! ## The rows are drawn, not slotted
//!
//! Unlike the list widgets, this panel has no fixed set of DSL slots: a menu's row count is its data's, and each row has up
//! to four pieces of text at four positions. So each row is drawn at an absolute position computed by [`row_rect`], the way
//! the step indicator and the colour picker draw their elements — and the same function is what the hit test uses, so the
//! row on screen and the row a click reports cannot come from two different arithmetics.

use makepad_widgets::*;

use crate::mp::menu::{Cursor, Hit, Item};

/// The leading glyph and the trailing check, at the size the rows are set in.
///
/// bezel's number, kept: the glyph is set below the row's own text so the icons read as a column rather than as part of the
/// labels.
pub const GLYPH: f64 = 13.0;

/// How wide a panel sits when no row has a description.
pub const PANEL_MIN: f64 = 180.0;

/// How wide one holding a described row sits — **a width, not a floor**. See the module doc.
pub const PANEL_DESCRIBED: f64 = 280.0;

/// A row's horizontal padding.
pub const ROW_PAD_X: f64 = 8.0;
/// A row's vertical padding.
pub const ROW_PAD_Y: f64 = 6.0;
/// The gap between a row's pieces.
pub const ROW_GAP: f64 = 10.0;
/// The height of a separator.
pub const SEPARATOR: f64 = 7.0;

/// Whether the panel reserves a column for glyphs.
///
/// **Only when a row uses it.** See the module doc: an empty gutter is the thing a menu bar's menus would otherwise all open
/// with.
pub fn reserves_gutter(items: &[Item]) -> bool {
    items.iter().any(|item| match item {
        Item::Action { icon, .. } | Item::Submenu { icon, .. } => icon.is_some(),
        Item::Separator => false,
    })
}

/// Whether any row carries a second line.
pub fn has_descriptions(items: &[Item]) -> bool {
    items.iter().any(|item| match item {
        Item::Action { description, .. } => description.is_some(),
        _ => false,
    })
}

/// The panel's width for this menu.
pub fn panel_width(items: &[Item]) -> f64 {
    if has_descriptions(items) {
        PANEL_DESCRIBED
    } else {
        PANEL_MIN
    }
}

/// A row's height: one line, two when described, and a separator's own small height.
pub fn row_height(item: &Item, line: f64) -> f64 {
    match item {
        Item::Separator => SEPARATOR,
        Item::Action {
            description: Some(_), ..
        } => line * 2.0 + ROW_PAD_Y * 2.0,
        _ => line + ROW_PAD_Y * 2.0,
    }
}

/// The panel's height: every row and nothing else.
///
/// No padding above and below, because a menu's first and last rows are its own edges — a panel with an outer inset reads as
/// a card with a list inside it rather than as a menu.
pub fn panel_height(items: &[Item], line: f64) -> f64 {
    items.iter().map(|item| row_height(item, line)).sum()
}

/// The top of row `index`.
pub fn row_top(items: &[Item], index: usize, line: f64) -> f64 {
    items
        .iter()
        .take(index)
        .map(|item| row_height(item, line))
        .sum()
}

/// The rect of row `index` in a panel whose top-left is `origin`.
pub fn row_rect(items: &[Item], index: usize, line: f64, origin: DVec2, width: f64) -> Rect {
    let Some(item) = items.get(index) else {
        return Rect {
            pos: origin,
            size: dvec2(0.0, 0.0),
        };
    };
    Rect {
        pos: dvec2(origin.x, origin.y + row_top(items, index, line)),
        size: dvec2(width, row_height(item, line)),
    }
}

/// Which row a `y` lands on, where `y` is relative to the panel's top.
///
/// **The separator is included in the walk**, so a point in the space between two groups answers `Some(index of the
/// separator)` rather than the row above or below it — and the caller, seeing a separator, treats it as nothing. Answering
/// `None` from inside the panel would be the same thing with less information; what matters is that a point can never be
/// rounded to a row somebody did not aim at.
///
/// `None` outside the panel, and `None` for a non-finite `y` — every comparison against `NaN` is false, so a version
/// written with `<` alone would fall through the bounds and land on a row.
pub fn row_at(items: &[Item], y: f64, line: f64) -> Option<usize> {
    if !y.is_finite() || y < 0.0 {
        return None;
    }
    let mut top = 0.0;
    for (index, item) in items.iter().enumerate() {
        let height = row_height(item, line);
        if y < top + height {
            return Some(index);
        }
        top += height;
    }
    None
}

/// Where a submenu's panel sits, given its parent panel's top-left and the row it hangs off.
///
/// Overlapped by a hair rather than butted against the parent: two panels sharing an edge show the parent's row and the
/// child's first row as one continuous strip, and any rounding at the boundary leaves a seam of background between them.
pub fn submenu_origin(
    parent: DVec2,
    parent_width: f64,
    items: &[Item],
    row: usize,
    line: f64,
) -> DVec2 {
    dvec2(
        parent.x + parent_width - SUBMENU_OVERLAP,
        parent.y + row_top(items, row, line),
    )
}

/// How far a submenu panel is pulled back over its parent.
pub const SUBMENU_OVERLAP: f64 = 4.0;

/// Whether a point is inside any of the open panels.
///
/// **The dismissal question.** An "out click" on the card sees only the card's own bounds, so a click inside a submenu reads
/// as a click away and closes the menu being used. Every panel's rect goes in and one answer comes out.
pub fn within(panels: &[Rect], point: DVec2) -> bool {
    panels.iter().any(|panel| {
        point.x >= panel.pos.x
            && point.x < panel.pos.x + panel.size.x
            && point.y >= panel.pos.y
            && point.y < panel.pos.y + panel.size.y
    })
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.DrawMpMenuPlate = #(DrawMpMenuPlate::script_shader(vm)){
        ..mod.draw.DrawQuad

        live: 0.0
        disabled: 0.0
        radius: 5.0
        plate: #x00000000
        live_color: #x00000000

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(0.0, 0.0, sz.x, sz.y, self.radius)
            // The live plate fades with `live`, which the widget sets to 0 or 1 — and a disabled row never lights, because a
            // row that lit under the pointer would be inviting a press that does nothing.
            let lit = self.live * (1.0 - self.disabled)
            sdf.fill_keep(mix(self.plate, self.live_color, lit))
            return sdf.result
        }
    }

    mod.mp.MpMenuCardBase = #(MpMenuCard::register_widget(vm))

    mod.mp.MpMenuCard = set_type_default() do mod.mp.MpMenuCardBase{
        width: Fit
        height: Fit

        // **No `draw_plate +: {...}` block here**, and that is the finding rather than an omission: a widget that
        // **self-draws** through its own `#[live] draw_x` cannot configure that shader's instance fields from its own DSL —
        // the fields belong to the shader prototype ([`DrawMpMenuPlate`] above, which carries the defaults) and the widget
        // writes them per row from Rust. A `draw_x +: {...}` here is `field draw_plate not found in type-check and has no
        // default` at runtime, with `cargo build` green. This is the same trap this port hit in `mp/step_indicator.rs` and
        // `mp/color_picker.rs`; the difference is that those never wrote the block.
        draw_label +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text}
        draw_description +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_muted}
        draw_keystroke +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_faint}
        draw_check +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text}
    }
}

/// A row's plate.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpMenuPlate {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    live: f32,
    #[live]
    disabled: f32,
    #[live]
    radius: f32,
    #[live]
    plate: Vec4f,
    #[live]
    live_color: Vec4f,
}

/// The panel a menu drops.
#[derive(Script, ScriptHook, Widget)]
pub struct MpMenuCard {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_plate: DrawMpMenuPlate,
    /// The row's label.
    #[live]
    draw_label: DrawText,
    /// The second line under a described row.
    #[live]
    draw_description: DrawText,
    /// The accelerator, printed and never dispatched — see `mp/menu.rs`.
    #[live]
    draw_keystroke: DrawText,
    /// The trailing check on the row the menu is currently on.
    #[live]
    draw_check: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// The rows, which are the caller's.
    #[rust]
    items: Vec<Item>,
    /// Where the caller is among them. **The caller's, not this widget's** — see `mp/menu.rs` on why the open state is the
    /// application's.
    #[rust]
    cursor: Cursor,
    /// The panel's top-left, captured during the last paint so a pointer position can be made relative to it.
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

/// The height of a row's text line, from the theme's own row height.
///
/// **A one-line row is exactly `Theme::layout.row_height` tall**, which is what makes a menu row the same height as a list
/// row and a combobox row without any of them knowing about the others. The padding is subtracted rather than added: the
/// theme's number is the row, and the padding is what the row's contents sit inside.
pub fn line_height(cx: &mut Cx) -> f64 {
    let row = makepad_theme::Theme::of(cx).layout.row_height;
    // `Theme::layout.row_height` is an `f32` and this module's metrics are `f64`, like every other geometry here.
    (row as f64 - ROW_PAD_Y * 2.0).max(1.0)
}

impl MpMenuCard {
    /// The rows.
    pub fn set_items(&mut self, cx: &mut Cx, items: Vec<Item>) {
        self.items = items;
        // A cursor naming rows that are gone is cleared rather than trusted: the panel decides what is lit, and a stale
        // chain would hang a submenu off a row that is no longer a submenu. `mp/menu.rs` has the truncating version for a
        // caller that wants to keep what still resolves; this is the blunt one, because a rebuilt menu has no rows in
        // common with the old one often enough that keeping a path would be a lie more often than a kindness.
        self.cursor.clear();
        self.redraw(cx);
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn set_cursor(&mut self, cx: &mut Cx, cursor: Cursor) {
        self.cursor = cursor;
        self.redraw(cx);
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// The panel's width for the current rows.
    pub fn width(&self) -> f64 {
        panel_width(&self.items)
    }

    /// Which row a panel-relative `y` lands on.
    pub fn row_at(&self, y: f64, line: f64) -> Option<usize> {
        row_at(&self.items, y, line)
    }
}

impl Widget for MpMenuCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let hit = event.hits(cx, self.area);
        // **The pointer arrives as a `Hit` and the caller acts** — this widget moves nothing on its own. `Point` is a path
        // for `Cursor::point_at`, which is the same call the keyboard's `step` moves, so the two cannot disagree about
        // which row an open submenu hangs off.
        let local = match event {
            Event::MouseMove(me) => Some(me.abs),
            Event::MouseDown(me) => Some(me.abs),
            _ => None,
        };
        let Some(abs) = local else {
            return;
        };
        let line = line_height(cx);
        let y = abs.y - self.origin.y;
        let Some(index) = row_at(&self.items, y, line) else {
            return;
        };
        // A separator and a disabled row are not rows: the first is a line, the second a row that cannot be chosen, and
        // reporting either as a `Point` would light something the pointer is not on.
        let Some(item) = self.items.get(index) else {
            return;
        };
        if !item.selectable() {
            return;
        }
        let inside_x = abs.x >= self.origin.x && abs.x < self.origin.x + self.width();
        if !inside_x {
            return;
        }
        match (event, item) {
            (Event::MouseDown(_), Item::Action { enabled: true, .. }) => {
                cx.widget_action(self.uid, Hit::Choose(vec![index]));
            }
            (Event::MouseMove(_), _) => {
                cx.widget_action(self.uid, Hit::Point(vec![index]));
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (line, plate, live_color, label_ink, muted_ink, faint_ink) = {
            // **The line height is computed first and the theme read after**, because `Theme::of` borrows the context and
            // `line_height` needs it mutably — the pair cannot be read in one tuple expression.
            let line = line_height(cx.cx);
            let theme = makepad_theme::Theme::of(cx.cx);
            (
                line,
                // A transparent plate at rest and the theme's element-hover colour when live — the same pair a list
                // row and a combobox row use, so a menu row lights like every other row in the library.
                Vec4f::default(),
                theme.paint.element_hover,
                theme.paint.text,
                theme.paint.text_muted,
                theme.paint.text_faint,
            )
        };
        let width = panel_width(&self.items);
        let height = panel_height(&self.items, line);
        let placed = cx.walk_turtle(Walk {
            width: Size::Fixed(width),
            height: Size::Fixed(height),
            ..walk
        });
        self.origin = placed.pos;
        self.area = self.draw_plate.area();

        let gutter = reserves_gutter(&self.items);
        let check_x = width - ROW_PAD_X - GLYPH;
        // The label starts after the padding and, when one exists, the gutter. **`reserves_gutter` is asked once for the
        // whole panel**, so every label starts at the same x whether or not its own row has a glyph — a menu whose labels
        // shifted row by row would be unreadable.
        let label_x = ROW_PAD_X + if gutter { GLYPH + ROW_GAP } else { 0.0 };

        for (index, item) in self.items.iter().enumerate() {
            if matches!(item, Item::Separator) {
                continue;
            }
            let rect = row_rect(&self.items, index, line, self.origin, width);
            let live = self.cursor.lit(0) == Some(index);
            let enabled = item.selectable();
            self.draw_plate.live = if live { 1.0 } else { 0.0 };
            self.draw_plate.disabled = if enabled { 0.0 } else { 1.0 };
            self.draw_plate.plate = plate;
            self.draw_plate.live_color = live_color;
            self.draw_plate.radius = 5.0;
            self.draw_plate.draw_abs(cx, rect);

            let ink = if enabled { label_ink } else { faint_ink };
            self.draw_label.color = ink;
            self.draw_label.draw_abs(
                cx,
                dvec2(rect.pos.x + label_x, rect.pos.y + ROW_PAD_Y),
                item.label().unwrap_or_default(),
            );
            if let Item::Action {
                description: Some(description),
                ..
            } = item
            {
                self.draw_description.color = muted_ink;
                // The second line sits under the first, at the same x — so a described row reads as one block rather than
                // as a label with an annotation beside it.
                self.draw_description.draw_abs(
                    cx,
                    dvec2(rect.pos.x + label_x, rect.pos.y + ROW_PAD_Y + line),
                    description,
                );
            }
            if let Item::Action {
                keystroke: Some(keystroke),
                ..
            } = item
            {
                self.draw_keystroke.color = faint_ink;
                self.draw_keystroke.draw_abs(
                    cx,
                    dvec2(rect.pos.x + label_x, rect.pos.y + ROW_PAD_Y),
                    keystroke,
                );
            }
            if let Item::Action { checked: true, .. } = item {
                self.draw_check.color = if enabled { label_ink } else { faint_ink };
                self.draw_check.draw_abs(
                    cx,
                    dvec2(rect.pos.x + check_x, rect.pos.y + ROW_PAD_Y),
                    "\u{2713}",
                );
            }
            // A submenu row says so with a caret where the check would be, so the two never appear on the same row.
            if item.opens().is_some() {
                self.draw_check.color = faint_ink;
                self.draw_check.draw_abs(
                    cx,
                    dvec2(rect.pos.x + check_x, rect.pos.y + ROW_PAD_Y),
                    "\u{203A}",
                );
            }
        }
        DrawStep::done()
    }
}

impl MpMenuCardRef {
    pub fn set_items(&self, cx: &mut Cx, items: Vec<Item>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn set_cursor(&self, cx: &mut Cx, cursor: Cursor) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_cursor(cx, cursor);
        }
    }

    pub fn width(&self) -> f64 {
        self.borrow().map(|inner| inner.width()).unwrap_or(0.0)
    }

    pub fn items(&self) -> Vec<Item> {
        self.borrow().map(|inner| inner.items.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<Item> {
        vec![
            Item::action("New"),
            Item::action("Open"),
            Item::Separator,
            Item::submenu("Share", vec![Item::action("Copy link")]),
        ]
    }

    #[test]
    fn test_the_gutter_is_reserved_only_when_a_row_uses_it() {
        // **The rule that stops a menu bar's menus from opening with an empty column down their left.** One icon in the
        // panel opens the gutter for every row; none keeps the labels against the padding.
        assert!(!reserves_gutter(&items()));
        let mut with_icon = items();
        with_icon[1] = Item::action("Open").with_icon("folder");
        assert!(reserves_gutter(&with_icon), "one icon opens the gutter for the panel");
        // A separator is not a row with an icon, and an empty menu reserves nothing.
        assert!(!reserves_gutter(&[Item::Separator]));
        assert!(!reserves_gutter(&[]));
        // ...but a submenu row with an icon does open it, which is the case a `matches!` on `Action` alone would miss.
        assert!(reserves_gutter(&[Item::submenu("Share", vec![]).with_icon("share")]));
    }

    #[test]
    fn test_a_described_row_widens_the_panel_to_a_ceiling_not_a_floor() {
        // A description is a sentence rather than a name, so the panel widens the way one icon opens the gutter — and
        // because the sentence is kept to one line, the panel takes a **width** to clip against rather than growing to
        // whatever the longest description measures. A width that varied with the data would make the menu jump.
        assert_eq!(panel_width(&items()), PANEL_MIN);
        let described = vec![Item::action("Open").with_description("an existing file")];
        assert_eq!(panel_width(&described), PANEL_DESCRIBED);
        assert!(panel_width(&described) > panel_width(&items()));
        // A submenu's own rows never widen the panel: a description lives on an action row, and the panel that holds a
        // submenu row is the parent's.
        let nested = vec![Item::submenu(
            "Share",
            vec![Item::action("Copy").with_description("to the clipboard")],
        )];
        assert!(!has_descriptions(&nested), "a child's description is not the parent's problem");
    }

    #[test]
    fn test_a_one_line_row_is_exactly_the_theme_row_height_and_a_described_one_is_two_lines() {
        // The provenance of the height: `line` comes from the theme's row height minus the padding, so a one-line row is
        // **exactly** `Theme::layout.row_height` — the same height as a list row and a combobox row, with none of them
        // knowing about the others.
        let line = 20.0;
        assert_eq!(row_height(&Item::action("x"), line), line + ROW_PAD_Y * 2.0);
        let described = Item::action("x").with_description("why");
        assert_eq!(row_height(&described, line), line * 2.0 + ROW_PAD_Y * 2.0);
        assert_eq!(
            row_height(&described, line) - row_height(&Item::action("x"), line),
            line,
            "a described row is exactly one line taller"
        );
        // A separator has its own height, small enough to read as a line rather than as an empty row.
        assert_eq!(row_height(&Item::Separator, line), SEPARATOR);
        assert!(SEPARATOR < line, "a separator taller than a row is an empty row");
    }

    #[test]
    fn test_the_panel_height_is_its_rows_and_nothing_else() {
        // No outer padding: a menu's first and last rows are its own edges, and a panel with an inset reads as a card with a
        // list inside it rather than as a menu.
        let line = 20.0;
        let rows = items();
        let sum: f64 = rows.iter().map(|item| row_height(item, line)).sum();
        assert_eq!(panel_height(&rows, line), sum);
        assert_eq!(panel_height(&[], line), 0.0);
        assert_eq!(
            panel_height(&rows, line),
            panel_height(&rows, line) + panel_height(&[], line),
            "an empty panel adds nothing"
        );
    }

    #[test]
    fn test_the_rows_stack_without_gaps_and_tile_the_panel_exactly() {
        // **The property that keeps the drawing and the hit test from disagreeing**: the rows tile the panel with no gaps and
        // no overlaps, so the last row ends exactly where the panel does. If `row_top` and `panel_height` ever disagreed,
        // a click on the last row would land outside the panel or on its neighbour.
        let line = 20.0;
        let rows = items();
        for index in 0..rows.len() {
            let top = row_top(&rows, index, line);
            let height = row_height(&rows[index], line);
            assert_eq!(top + height, row_top(&rows, index + 1, line), "row {index} does not meet the next");
        }
        assert_eq!(
            row_top(&rows, rows.len(), line),
            panel_height(&rows, line),
            "the rows do not reach the panel's edge"
        );
    }

    #[test]
    fn test_a_point_lands_on_the_row_it_is_in_and_a_separator_count_is_none_of_the_caller_s_business() {
        // The separator is **included in the walk** so a point between two groups answers with the separator's own index
        // rather than the row above or below — and the caller, seeing a separator, treats it as nothing. Rounding such a
        // point to a neighbour would pick a row nobody aimed at.
        let line = 20.0;
        let rows = items();
        assert_eq!(row_at(&rows, 0.0, line), Some(0));
        assert_eq!(row_at(&rows, 31.9, line), Some(0), "still row 0");
        assert_eq!(row_at(&rows, 32.0, line), Some(1), "the first point of row 1");
        let separator_top = row_top(&rows, 2, line);
        assert_eq!(row_at(&rows, separator_top + 1.0, line), Some(2), "the separator's own index");
        // And past the end, nothing.
        assert_eq!(row_at(&rows, panel_height(&rows, line), line), None);
        assert_eq!(row_at(&rows, panel_height(&rows, line) + 50.0, line), None);
        assert_eq!(row_at(&rows, -1.0, line), None, "above a panel is not row 0");
        assert_eq!(row_at(&[], 0.0, line), None);
    }

    #[test]
    fn test_a_non_finite_point_lands_on_nothing() {
        // Every comparison against `NaN` is false, so a version written with `<` alone would fall through the bounds and
        // land on a row. This port has paid for a propagating `NaN` four times now.
        let line = 20.0;
        assert_eq!(row_at(&items(), f64::NAN, line), None);
        assert_eq!(row_at(&items(), f64::INFINITY, line), None);
        assert_eq!(row_at(&items(), f64::NEG_INFINITY, line), None);
    }

    #[test]
    fn test_a_submenu_hangs_beside_its_row_overlapped_by_a_hair() {
        // Overlapped rather than butted: two panels sharing an edge show the parent's row and the child's first row as one
        // continuous strip, and any rounding at the boundary leaves a seam of background between them.
        let line = 20.0;
        let rows = items();
        let parent = dvec2(100.0, 200.0);
        let width = panel_width(&rows);
        let origin = submenu_origin(parent, width, &rows, 3, line);
        // It starts at the parent's right edge, pulled back by the overlap...
        assert_eq!(origin.x, 100.0 + width - SUBMENU_OVERLAP);
        assert!(origin.x < 100.0 + width, "the child does not overlap its parent");
        assert!(SUBMENU_OVERLAP < width, "the overlap is a hair, not half the panel");
        // ...and at the top of the row it hangs off, **not at the top of the panel**.
        assert_eq!(origin.y, 200.0 + row_top(&rows, 3, line));
        assert_ne!(origin.y, parent.y, "a submenu at the panel's top is not beside its row");
    }

    #[test]
    fn test_dismissal_is_a_question_about_all_the_panels() {
        // **The subtle one.** An out-click on the card sees only the card's own bounds, so a click inside a submenu reads as
        // a click away and closes the menu being used.
        let parent = Rect {
            pos: dvec2(0.0, 0.0),
            size: dvec2(180.0, 100.0),
        };
        let child = Rect {
            pos: dvec2(176.0, 40.0),
            size: dvec2(180.0, 80.0),
        };
        let panels = [parent, child];
        // Inside the parent.
        assert!(within(&panels, dvec2(10.0, 10.0)));
        // **Inside the child but outside the parent** — the case an out-click on the card gets wrong, and the case that
        // needs the child to be in the list.
        assert!(within(&panels, dvec2(300.0, 100.0)), "a click inside a submenu is not a click away");
        assert!(!within(&[parent], dvec2(300.0, 100.0)), "and with only the parent it reads as away");
        // Outside every panel.
        assert!(!within(&panels, dvec2(500.0, 500.0)));
        assert!(!within(&panels, dvec2(-1.0, 10.0)));
        // A point exactly on the far edge is outside: a rect's size is its extent, so `x + w` is the first point past it.
        assert!(within(&panels, dvec2(179.9, 10.0)));
        assert!(!within(&panels, dvec2(180.0, 10.0)));
        assert!(!within(&[], dvec2(0.0, 0.0)), "nothing open, nothing to be inside");
    }

    #[test]
    fn test_a_submenu_row_shows_a_caret_where_a_checked_row_shows_a_check() {
        // The two marks live in the same place, and a row that was both would print one over the other. This is the rule as
        // a statement about the data: `opens` and `checked` are never both true on one row, because only an action can be
        // checked and only a submenu opens.
        let checked = Item::action("Word wrap").checked(true);
        let opens = Item::submenu("Share", vec![Item::action("Copy")]);
        assert!(matches!(checked, Item::Action { checked: true, .. }));
        assert!(opens.opens().is_some());
        assert!(!matches!(checked, Item::Submenu { .. }), "an action cannot open");
        match &checked {
            Item::Action { .. } => {}
            other => panic!("wrong variant: {other:?}"),
        }
    }
}
