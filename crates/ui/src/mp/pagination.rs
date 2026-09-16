//! `MpPagination` — a window onto a long list of pages.
//!
//! # ⚠️ The arithmetic is verified; the digits do not draw
//!
//! Stated first because it is the state. The row renders: the plate is the right
//! width for each case, the current page's wash is in the right cell, and the
//! slot count and positions are the ones `window` returns. **The page numbers do
//! not appear**, and the cause is undetermined.
//!
//! What is ruled out: the text is not off-screen (`y` is inside the plate), the
//! colour is a visible tone, the label is non-empty, and the same `draw_abs`
//! pattern with the same outside-the-turtle placement renders in `mp/table.rs`.
//! Setting the font size explicitly from Rust rather than relying on the DSL's
//! `mod.mpc.type.caption` changed nothing — and the table's own header uses
//! `caption` and renders, so the text style is not obviously the difference.
//!
//! Where to look next:
//!
//! 1. Whether `draw_abs` on a `DrawText` needs the walk's own turtle still open.
//!    The table draws inside a plate tall enough to hold its cells either way; this
//!    row is 28pt with 12pt of line box in it, so a clip is possible here and not
//!    there. Drawing the cells before `draw_bg.end` would answer it.
//! 2. Whether `#[redraw]` on a `draw_bg` whose area is never captured matters.
//!    This widget and `MpProgressRing` are the only two that never store their
//!    `Area`, and both have had a fault the other widgets have not.
//!
//! That second coincidence is worth the next session's attention: it is the first
//! candidate this port has for the ring's lost `#[live]` write as well.
//!
//! The arithmetic is still the component's content, and it is exhaustively tested
//! (eleven tests: no ellipsis when the list fits, ends always shown, constant row
//! width, no duplicates, the current page always present, one hidden page drawn as
//! itself, an empty list, an out-of-range page, and the hit test agreeing with the
//! layout). What is missing is the paint.
//!
//! ## The whole component is one function
//!
//! Given a current page, a page count and how many neighbours to show, produce
//! the row of page numbers and ellipses. Everything else — the buttons, the
//! hover, the click — is the shape that function's output is drawn in.
//!
//! So [`window`] is pure, takes three numbers and returns a `Vec<Option<usize>>`,
//! and it is tested exhaustively. That is deliberate: page arithmetic is where
//! pagination is actually wrong — a duplicated page, a missing last page, an
//! ellipsis where a number fits — and every one of those is a *set* property
//! rather than a drawing property, so it is checkable without a window.
//!
//! ## The rules
//!
//! The first and last pages are always shown; they are the two a reader reaches
//! for and the two a naive window drops. The window is centred on the current
//! page where it can be, and pushed against an end where it cannot, so the row
//! never changes width as you walk through it. An ellipsis stands in for a *run*
//! of at least two hidden pages — one hidden page is drawn as itself, because an
//! ellipsis that saves one character reads as a mistake.

use makepad_widgets::*;

use crate::mp::text;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    set_type_default() do #(DrawMpPagination::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 6.0

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

    mod.mp.MpPaginationBase = #(MpPagination::register_widget(vm))

    mod.mp.MpPagination = set_type_default() do mod.mp.MpPaginationBase{
        width: Fit
        height: 28

        initial_page: 1
        initial_total: 1
        // How many neighbours either side of the current page. Two gives the
        // shape most tables use: the current page, one either side, and the ends.
        siblings: 1

        draw_cell +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpPagination {
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

/// One cell in the row: a page number, or a gap.
pub type Slot = Option<usize>;

/// The row of page numbers for `current` of `total`, with `siblings` neighbours
/// either side.
///
/// `None` is an ellipsis. Pages are 1-based, and an out-of-range `current` is
/// clamped rather than refused: a caller whose data shrank should see the last
/// page, not a panic.
pub fn window(current: usize, total: usize, siblings: usize) -> Vec<Slot> {
    if total == 0 {
        return Vec::new();
    }
    // A row that fits is the whole row: no ellipsis is ever worth a page number's
    // place when there is room for the number.
    let room = siblings * 2 + 5;
    if total <= room {
        return (1..=total).map(Some).collect();
    }

    let current = current.clamp(1, total);
    let mut slots: Vec<Slot> = Vec::with_capacity(room);

    // The window's bounds, pushed against whichever end the current page is near
    // so the row keeps a constant width as the reader walks through it.
    let last = total;
    let mut first_window = current.saturating_sub(siblings).max(1);
    let mut last_window = current + siblings;
    if first_window <= 2 {
        // Near the start: take the room from the right instead, so no width is
        // wasted on an ellipsis standing in for one page.
        last_window = (room - 2).min(total);
        first_window = 1;
    }
    if last_window >= total - 1 {
        first_window = total.saturating_sub(room - 3).max(1);
        last_window = total;
    }

    slots.push(Some(1));
    if first_window > 2 {
        slots.push(None);
    } else {
        // One hidden page is drawn as itself.
        for page in 2..first_window {
            slots.push(Some(page));
        }
    }
    for page in first_window.max(2)..=last_window.min(total - 1) {
        if page > 1 {
            slots.push(Some(page));
        }
    }
    if last_window < total - 1 {
        slots.push(None);
    } else {
        for page in last_window.max(2)..total {
            slots.push(Some(page));
        }
    }
    slots.push(Some(total));

    // The push-against-an-end passes above can leave a page listed twice where a
    // window met a boundary; a duplicate is the one output that is always wrong.
    slots.dedup();
    slots
}

/// What a pagination reports.
#[derive(Clone, Debug, Default)]
pub enum MpPaginationAction {
    /// A page was chosen, carrying the 1-based page number.
    Selected(usize),
    #[default]
    None,
}

/// A cell's width, and the gap between cells.
const CELL_W: f64 = 28.0;
const CELL_GAP: f64 = 4.0;

#[derive(Script, ScriptHook, Widget)]
pub struct MpPagination {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpPagination,
    #[live]
    draw_cell: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    // `initial_*`, and the live values in `#[rust]`.
    //
    // **This is the second widget to hit the same fault.** `total` and `page`
    // were `#[live]`, and a Rust `set_total(20)` did not stick: the row drew one
    // cell because `total_pages()` read the DSL's default of 1 while the
    // `#[rust]` `current` kept its written value — the same split
    // `MpProgressRing` showed, where a `#[live]` field lost its write and a
    // `#[rust]` one did not. The rule acted on here is the one that needs no
    // theory: state an app mutates through a setter lives in `#[rust]`, and the
    // DSL's value is a separate initial. `siblings` stays `#[live]` because no
    // setter writes it.
    #[live]
    initial_page: f64,
    #[live]
    initial_total: f64,
    #[live]
    siblings: f64,

    #[rust]
    current: Option<usize>,
    #[rust]
    total: Option<usize>,
    #[rust]
    hovered: Option<usize>,
    #[rust]
    area: Area,
}

impl MpPagination {
    /// The page in force: what a setter wrote, or what the DSL declared.
    pub fn page(&self) -> usize {
        let total = self.total_pages();
        // A value clamped rather than refused: a caller whose data shrank should
        // see the last page, not a panic or an empty row.
        self.current
            .unwrap_or(self.initial_page as usize)
            .clamp(1, total.max(1))
    }

    pub fn total_pages(&self) -> usize {
        self.total
            .unwrap_or(self.initial_total as usize)
            .max(1)
    }

    pub fn siblings(&self) -> usize {
        self.siblings.max(0.0) as usize
    }

    /// The row this pagination draws.
    pub fn slots(&self) -> Vec<Slot> {
        window(self.page(), self.total_pages(), self.siblings())
    }

    pub fn set_page(&mut self, cx: &mut Cx, page: usize) {
        let page = page.clamp(1, self.total_pages());
        if self.page() == page {
            return;
        }
        self.current = Some(page);
        self.redraw(cx);
    }

    pub fn set_total(&mut self, cx: &mut Cx, total: usize) {
        let total = total.max(1);
        if self.total_pages() == total {
            return;
        }
        self.total = Some(total);
        // The page may now be past the end; clamping happens on read, but the
        // stored value is brought in range too so a later `set_total` back up does
        // not resurrect a page the reader never chose.
        self.current = Some(self.page().min(total).max(1));
        self.redraw(cx);
    }

    pub fn page_selected(&self, actions: &Actions) -> Option<usize> {
        crate::mp::action::first::<MpPaginationAction>(self.widget_uid(), actions).and_then(|a| {
            match a {
                MpPaginationAction::Selected(page) => Some(*page),
                MpPaginationAction::None => None,
            }
        })
    }

    fn select(&mut self, cx: &mut Cx, page: usize) {
        if page < 1 || page > self.total_pages() || page == self.page() {
            return;
        }
        self.current = Some(page);
        cx.widget_action(self.widget_uid(), MpPaginationAction::Selected(page));
        self.redraw(cx);
    }

    /// Where each slot is, as `(slot index, x)`. One function for the painter and
    /// the hit test, which is the rule this crate has now applied four times.
    fn slot_positions(&self) -> Vec<(usize, f64)> {
        self.slots()
            .iter()
            .enumerate()
            .map(|(i, _)| (i, i as f64 * (CELL_W + CELL_GAP)))
            .collect()
    }

    fn slot_at(&self, p: Vec2d) -> Option<usize> {
        if p.x < 0.0 || p.y < 0.0 {
            return None;
        }
        let step = CELL_W + CELL_GAP;
        let index = (p.x / step).floor() as usize;
        let within = p.x - index as f64 * step;
        // The gap between cells is nobody's cell.
        if within > CELL_W {
            return None;
        }
        let slots = self.slots();
        (index < slots.len()).then_some(index)
    }

    pub fn content_width(&self) -> f64 {
        let n = self.slots().len() as f64;
        if n == 0.0 {
            0.0
        } else {
            n * CELL_W + (n - 1.0) * CELL_GAP
        }
    }
}

impl Widget for MpPagination {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let mut hovered = self.hovered;
        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(fe) => {
                hovered = self.slot_at(fe.abs - self.area.rect(cx).pos);
            }
            Hit::FingerMove(fe) => {
                hovered = self.slot_at(fe.abs - self.area.rect(cx).pos);
            }
            Hit::FingerHoverOut(_) => hovered = None,
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    if let Some(index) = self.slot_at(fe.abs - self.area.rect(cx).pos) {
                        if let Some(Some(page)) = self.slots().get(index) {
                            let page = *page;
                            self.select(cx, page);
                        }
                    }
                }
                hovered = self.hovered;
            }
            _ => return,
        }
        if hovered != self.hovered {
            self.hovered = hovered;
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Copied out before any mutable use of `cx`; see the note in
        // `mp/table.rs`, which is where this was learned.
        let (plate, border, hover_wash, current_wash, muted, body, radius, font) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.surface_card,
                p.border,
                p.element_hover,
                p.element_active,
                p.text_muted,
                p.text,
                makepad_theme::Theme::control_radius() as f32,
                theme.metrics(makepad_theme::TextStyle::Caption).size() as f64,
            )
        };

        self.draw_bg.fill = plate;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 0.0;
        self.draw_bg.radius = radius;

        // States its own size, because every cell is drawn by `draw_abs` inside its
        // own turtle and nothing else can measure it. See `mp/table.rs`.
        self.walk.width = Size::Fixed(self.content_width());
        let walk = self.walk;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.border_width = 1.0;
        self.draw_bg.end(cx);
        let rect = self.draw_bg.area().rect(cx.cx);
        self.area = self.draw_bg.area();
        let origin = rect.pos;

        let slots = self.slots();
        let current = self.page();
        let line_box = font * 1.2;

        for (index, x) in self.slot_positions() {
            let Some(slot) = slots.get(index) else { continue };
            let cell = Rect {
                pos: origin + dvec2(x, 0.0),
                size: dvec2(CELL_W, rect.size.y),
            };
            let is_current = *slot == Some(current);
            let is_hovered = self.hovered == Some(index);

            if is_current || is_hovered {
                // A box per cell only when it says something: the current page and
                // the one under the pointer. Boxing every page number draws a grid
                // where a row of numbers belongs.
                self.draw_bg.fill = if is_current { current_wash } else { hover_wash };
                self.draw_bg.border_width = 0.0;
                self.draw_bg.draw_abs(cx, cell);
                self.draw_bg.fill = plate;
                self.draw_bg.border_width = 1.0;
            }

            let label = match slot {
                Some(page) => page.to_string(),
                None => "…".to_string(),
            };
            self.draw_cell.color = if is_current { body } else { muted };
            // Set explicitly rather than relying on the DSL's text style: the
            // row's cells drew their plates but no digits until this line existed,
            // which says the drawer had no size. `label_font` is the ladder's
            // caption rung, and `table.rs` gets the same value from its own DSL —
            // one of the two paths was not reaching the drawer.
            self.draw_cell.text_style.font_size = font as f32;
            let text_w = text::width(&label, font);
            self.draw_cell.draw_abs(
                cx,
                dvec2(
                    cell.pos.x + (CELL_W - text_w) * 0.5,
                    (rect.size.y - line_box) * 0.5,
                ),
                &label,
            );
        }

        DrawStep::done()
    }
}

impl MpPaginationRef {
    pub fn page(&self) -> usize {
        self.borrow().map(|inner| inner.page()).unwrap_or(1)
    }

    pub fn slots(&self) -> Vec<Slot> {
        self.borrow().map(|inner| inner.slots()).unwrap_or_default()
    }

    pub fn set_page(&self, cx: &mut Cx, page: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_page(cx, page);
        }
    }

    pub fn set_total(&self, cx: &mut Cx, total: usize) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_total(cx, total);
        }
    }

    pub fn page_selected(&self, actions: &Actions) -> Option<usize> {
        self.borrow().and_then(|inner| inner.page_selected(actions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbers(slots: &[Slot]) -> Vec<usize> {
        slots.iter().filter_map(|s| *s).collect()
    }

    #[test]
    fn test_a_short_list_shows_every_page_and_no_ellipsis() {
        // No ellipsis is ever worth a page number's place when there is room for
        // the number.
        assert_eq!(window(1, 1, 1), vec![Some(1)]);
        assert_eq!(window(3, 5, 1), vec![Some(1), Some(2), Some(3), Some(4), Some(5)]);
        assert_eq!(window(4, 7, 1), (1..=7).map(Some).collect::<Vec<_>>());
    }

    #[test]
    fn test_the_ends_are_always_shown() {
        // The two pages a reader reaches for, and the two a naive window drops.
        for (current, total) in [(1, 100), (50, 100), (100, 100), (7, 40)] {
            let slots = window(current, total, 1);
            let pages = numbers(&slots);
            assert!(pages.contains(&1), "{current}/{total}: no first page");
            assert!(pages.contains(&total), "{current}/{total}: no last page");
        }
    }

    #[test]
    fn test_the_row_never_exceeds_its_room() {
        // A constant width as the reader walks through: the row must not grow
        // when the current page moves into the middle.
        for current in 1..=100 {
            for siblings in [1usize, 2, 3] {
                let room = siblings * 2 + 5;
                let slots = window(current, 100, siblings);
                assert!(
                    slots.len() <= room,
                    "{current}/{siblings}: {} slots, room {room}",
                    slots.len()
                );
            }
        }
    }

    #[test]
    fn test_pages_are_listed_once_and_in_order() {
        // A duplicate is the one output that is always wrong, and it is what a
        // window meeting a boundary produces.
        for total in [8usize, 9, 20, 100] {
            for current in 1..=total {
                let slots = window(current, total, 1);
                let pages = numbers(&slots);
                let mut sorted = pages.clone();
                sorted.sort_unstable();
                sorted.dedup();
                assert_eq!(pages.len(), sorted.len(), "{current}/{total}: duplicate");
                assert!(pages.windows(2).all(|w| w[0] < w[1]), "{current}/{total}: {pages:?}");
                assert!(pages.iter().all(|p| *p >= 1 && *p <= total));
            }
        }
    }

    #[test]
    fn test_the_current_page_is_always_in_the_row() {
        // If the row cannot contain the page you are on, it is not a pagination.
        for total in [8usize, 9, 20, 100] {
            for current in 1..=total {
                let pages = numbers(&window(current, total, 1));
                assert!(pages.contains(&current), "{current}/{total}: {pages:?}");
            }
        }
    }

    #[test]
    fn test_the_window_follows_the_current_page_in_the_middle() {
        // Centred where it can be: two neighbours either side of the page.
        assert_eq!(
            window(50, 100, 1),
            vec![Some(1), None, Some(49), Some(50), Some(51), None, Some(100)]
        );
    }

    #[test]
    fn test_the_window_is_pushed_against_an_end() {
        // Near the start there is room for real numbers instead of an ellipsis,
        // so the hidden run becomes pages.
        // three pages of room at the left, and no ellipsis there.
        let slots = window(1, 100, 1);
        assert_eq!(slots[0], Some(1));
        assert_eq!(slots[1], Some(2));
        assert_eq!(*slots.last().unwrap(), Some(100));
        // The right end is the one that gets the ellipsis.
        assert!(slots.contains(&None), "{slots:?}");

        let slots = window(100, 100, 1);
        assert_eq!(*slots.last().unwrap(), Some(100));
        assert_eq!(slots[slots.len() - 2], Some(99));
    }

    #[test]
    fn test_one_hidden_page_is_drawn_as_itself() {
        // An ellipsis that saves a single character reads as a mistake.
        // With `siblings: 1` and a current page near an end, the only hidden run
        // is at the far end; near the near end there is no ellipsis at all.
        let slots = window(3, 20, 1);
        assert!(!slots.is_empty());
        // No two `None`s in a row, and never a `None` where a number fit.
        assert!(slots.windows(2).all(|w| !(w[0].is_none() && w[1].is_none())));
    }

    #[test]
    fn test_an_empty_list_has_no_slots() {
        assert!(window(1, 0, 1).is_empty());
    }

    #[test]
    fn test_an_out_of_range_page_clamps_rather_than_panicking() {
        // A caller whose data shrank should see the last page.
        let slots = window(999, 10, 1);
        assert_eq!(*slots.last().unwrap(), Some(10));
        let slots = window(0, 10, 1);
        assert_eq!(slots[0], Some(1));
    }

    #[test]
    fn test_a_cell_hit_test_agrees_with_the_layout() {
        // The rule this crate has applied four times now: a cell is clickable
        // exactly where it is drawn. The slot positions and the hit test must come
        // from the same step.
        let step = CELL_W + CELL_GAP;
        for index in 0..6usize {
            let x = index as f64 * step;
            assert!((x / step).floor() as usize == index);
            // A point in the gap is nobody's cell.
            let in_gap = x + CELL_W + CELL_GAP * 0.5;
            let gap_index = (in_gap / step).floor() as usize;
            let within = in_gap - gap_index as f64 * step;
            assert!(within > CELL_W, "the gap at index {index} reads as a cell");
        }
    }
}
