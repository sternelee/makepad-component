//! `MpDate` — the calendar, and the arithmetic under it.
//!
//! ## Why the arithmetic is the module
//!
//! A month grid is a header, seven letters and forty-two numbers. The numbers are the
//! part that can be wrong, and **a wrong calendar is wrong on a small fraction of
//! dates** — which is the worst possible failure rate, because it survives every
//! look-at-it test and then puts an appointment on the wrong day.
//!
//! Two of the ways it goes wrong are worth naming, because both are invisible in a
//! screenshot of one month:
//!
//! - **The leap rule.** A year divisible by 4 is a leap year *except* a century that
//!   is not divisible by 400. So 2000 is and 1900 and 2100 are not. A calendar that
//!   gets this wrong is correct for **three years out of four** and wrong for the one
//!   it matters in.
//! - **The weekday of a date**, which needs a real algorithm rather than a table. The
//!   one used here is Howard Hinnant's days-from-civil, and the test that it is right
//!   is not an anchor date but a **second, independent computation** — accumulate one
//!   day at a time from a known epoch and assert the two agree over thirty years. An
//!   anchor proves a date; a cross-check proves an algorithm.
//!
//! ## The grid is always six rows
//!
//! A month needs between four and six rows. The grid here is always
//! `GRID_ROWS × GRID_COLS`, with `0` in the cells that belong to a neighbouring
//! month, because **a calendar that changes height as you page through it moves its own
//! navigation arrows** — the reader's pointer lands on a different control in March
//! than it did in February. Six rows is the maximum any month needs (the worst case is
//! an offset of 6 and 31 days, which is 37 cells), so six always fits and the widget
//! never resizes.
//!
//! ## The library does not read the clock
//!
//! `today` is **set by the caller**. A widget that called `SystemTime::now()` itself
//! would be untestable, would disagree with the app's own idea of today across a
//! midnight boundary, and would make "is this cell today" impossible to check without
//! waiting a day.

use makepad_widgets::*;

use crate::mp::text;

/// Cells in the month grid: seven columns, six rows.
pub const GRID_COLS: usize = 7;
/// See [`GRID_COLS`].
pub const GRID_ROWS: usize = 6;
/// What a caller iterates: [`GRID_ROWS`] × [`GRID_COLS`].
pub const GRID_CELLS: usize = GRID_ROWS * GRID_COLS;

/// Which day a week starts on.
///
/// A closed set, because there are two and a flag would let a caller pass a third
/// meaning. Registered as `mod.mp.WeekStart`, so a DSL block writes
/// `week_start: mod.mp.WeekStart.Monday`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Script, ScriptHook)]
pub enum WeekStart {
    /// `M T W T F S S`.
    #[live]
    Monday,
    /// `S M T W T F S`. The default, because it is what the platform this library
    /// targets shows.
    #[pick]
    #[default]
    #[live]
    Sunday,
}

impl WeekStart {
    /// The offset added to a Sunday-based weekday to get this week's column.
    fn offset(self, sunday_based: u32) -> u32 {
        match self {
            WeekStart::Sunday => sunday_based,
            WeekStart::Monday => (sunday_based + 6) % 7,
        }
    }
}

/// Whether `year` has 366 days.
///
/// The full rule, not the divisible-by-four approximation: a century is a leap year
/// only when divisible by 400, so 2000 is and 1900 and 2100 are not.
pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// How many days `month` (1..=12) has in `year`.
///
/// Out-of-range months are not counted around: a month of `0` or `13` is a caller's
/// bug, and answering `31` for it would hide the bug until the grid drew a row of
/// blanks. It returns 0, which draws nothing and is obviously wrong.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Days from 1970-01-01 to `year-month-day`, which may be negative.
///
/// Howard Hinnant's `days_from_civil`, kept in its published form. The two steps that
/// are easy to get wrong and are therefore called out:
///
/// - **The year is shifted so March is the first month.** That puts the leap day at
///   the *end* of the shifted year, so February's irregularity cannot fall in the
///   middle of the arithmetic.
/// - **The era is 400 years**, and the `- 399` in the era division is what makes the
///   floor division correct for dates before the epoch. Without it every date before
///   1970 is off by a day in a way no test with a modern date would show.
pub fn days_since_epoch(year: i32, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year as i64 - 1 } else { year as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let shifted = if month > 2 { month - 3 } else { month + 9 } as i64;
    let doy = (153 * shifted + 2) / 5 + day as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// The day of the week of `year-month-day`, **0 = Sunday**.
///
/// 1970-01-01 was a Thursday, which is 4 with Sunday as 0, and the epoch is the one
/// fixed point this rests on.
pub fn weekday(year: i32, month: u32, day: u32) -> u32 {
    // `rem_euclid`, not `%`: a date before 1970 gives a negative day count and `%`
    // would return a negative index.
    ((days_since_epoch(year, month, day) + 4).rem_euclid(7)) as u32
}

/// The date `days` after 1970-01-01, which may be negative.
///
/// The inverse of [`days_since_epoch`], and it exists for a reason a library should
/// state: **this is how an app turns a clock reading into a date without the library
/// reading the clock.** `SystemTime` gives seconds since the epoch, the app divides by
/// 86400, and this turns that count back into a day the calendar can ring. Keeping it
/// here rather than in the app is what makes it testable, and the test is the
/// round trip — which is stronger than any expected value, because it only holds if
/// both directions are right.
///
/// Hinnant's `civil_from_days`; the `719468` is the same shift the forward function
/// applies at the end, and the two constants have to match or the round trip fails by
/// exactly that many days.
pub fn date_from_days_since_epoch(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11], March-based
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    // The year comes back March-based, so January and February belong to the next one.
    (if m <= 2 { y + 1 } else { y } as i32, m as u32, d as u32)
}

/// The date at `seconds` after the epoch, **in a zone `offset_seconds` ahead of UTC**.
///
/// This is the function an app calls, and the offset is a parameter because a library
/// cannot know it. **Passing zero means UTC, and UTC is not what a person means by
/// today** — it is the same day as the reader's for only part of every day.
///
/// The bug this exists to prevent was in this port's own gallery: "today" was computed
/// as `epoch_seconds / 86400`, which is the UTC day. On a machine at **UTC+8**, that is
/// the previous day for the first eight hours of every local day — the page printed
/// `today=2026-09-16` while the system clock said `2026-09-17`, and a calendar that
/// rings the wrong day is wrong in exactly the way this module's doc calls the worst
/// possible failure rate.
///
/// The offset is added to the *seconds* before the division, not to the resulting day,
/// because a day boundary is a property of the zone: `div_euclid` then makes a negative
/// pre-1970 timestamp floor correctly rather than truncating towards zero.
pub fn date_from_epoch_seconds(seconds: i64, offset_seconds: i64) -> (i32, u32, u32) {
    date_from_days_since_epoch((seconds + offset_seconds).div_euclid(86_400))
}

/// The month's days laid into a six-row grid, **0 meaning a cell outside the month**.
///
/// Six rows always, so the grid never changes height; see the module doc. A `0` is
/// never a valid day number, so "outside" needs no separate flag and the grid stays
/// `Copy`.
pub fn month_grid(year: i32, month: u32, week_start: WeekStart) -> [u32; GRID_CELLS] {
    let mut cells = [0u32; GRID_CELLS];
    let days = days_in_month(year, month);
    if days == 0 {
        return cells;
    }
    let offset = week_start.offset(weekday(year, month, 1)) as usize;
    for day in 1..=days {
        let index = offset + day as usize - 1;
        if index < GRID_CELLS {
            cells[index] = day;
        }
    }
    cells
}

/// The month `delta` months after `year-month`, with the year carried.
///
/// Month arithmetic on `(year, month)` pairs is where "December plus one" becomes
/// month 13, so this is the only place a month is stepped.
pub fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    // Months since year 0, which makes the carry a single division. `month` is
    // 1-based and becomes 0-based here, which is the whole reason for the `- 1`.
    let total = year as i64 * 12 + (month as i64 - 1) + delta as i64;
    let y = total.div_euclid(12) as i32;
    let m = (total.rem_euclid(12) + 1) as u32;
    (y, m)
}

/// `September`, for a header.
pub fn month_name(month: u32) -> &'static str {
    const NAMES: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    NAMES.get(month.wrapping_sub(1) as usize).copied().unwrap_or("")
}

/// The seven column headings, rotated to match `week_start`.
///
/// One letter each, because seven one-letter headings are what a calendar has room
/// for and the letters are unambiguous except for the two `S`s and two `T`s — which
/// is exactly why they are *ordered* rather than labelled: the reader reads position,
/// not letter.
pub fn weekday_initials(week_start: WeekStart) -> [&'static str; 7] {
    const SUNDAY_FIRST: [&str; 7] = ["S", "M", "T", "W", "T", "F", "S"];
    match week_start {
        WeekStart::Sunday => SUNDAY_FIRST,
        // The same week, started one later — not a different array, because a second
        // array is a second thing to get wrong when a weekday is renamed.
        WeekStart::Monday => ["M", "T", "W", "T", "F", "S", "S"],
    }
}

/// Which of the 42 grid cells a position falls in, relative to the grid's top-left.
///
/// `None` outside the grid or for a position that is not a number. Both guards are
/// load-bearing for the reason `mp/segmented.rs` records at length: a `NaN` passes
/// every `<` and `>=` comparison, so a guard that only compared bounds would let one
/// through and `NaN.floor() as usize` saturates to 0.
pub fn cell_at(x: f64, y: f64, cell_w: f64, cell_h: f64) -> Option<usize> {
    if !x.is_finite() || !y.is_finite() || !cell_w.is_finite() || !cell_h.is_finite() {
        return None;
    }
    if cell_w <= 0.0 || cell_h <= 0.0 || x < 0.0 || y < 0.0 {
        return None;
    }
    let col = (x / cell_w).floor();
    let row = (y / cell_h).floor();
    if col < 0.0 || row < 0.0 || col >= GRID_COLS as f64 || row >= GRID_ROWS as f64 {
        return None;
    }
    let index = row as usize * GRID_COLS + col as usize;
    if index < GRID_CELLS {
        Some(index)
    } else {
        None
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.mp.WeekStart = set_type_default() do #(WeekStart::script_api(vm))

    /// The calendar's plate: the panel, a selected cell, a hovered cell.
    set_type_default() do #(DrawMpDate::script_shader(vm)){
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

    mod.mp.MpDateBase = #(MpDate::register_widget(vm))

    mod.mp.MpDate = set_type_default() do mod.mp.MpDateBase{
        width: Fit
        height: Fit

        week_start: mod.mp.WeekStart.Sunday

        draw_title +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_weekday +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
        draw_day +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpDate {
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

/// What a calendar reports.
#[derive(Clone, Debug, Default)]
pub enum MpDateAction {
    /// A day was chosen, as `(year, month, day)`.
    Selected {
        year: i32,
        month: u32,
        day: u32,
    },
    /// The view moved to another month without a day being chosen.
    ViewChanged {
        year: i32,
        month: u32,
    },
    #[default]
    None,
}

/// The month/year a calendar is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YearMonth {
    pub year: i32,
    pub month: u32,
}

impl YearMonth {
    pub fn new(year: i32, month: u32) -> Self {
        Self { year, month }
    }

    pub fn shift(self, delta: i32) -> Self {
        let (year, month) = shift_month(self.year, self.month, delta);
        Self { year, month }
    }

    /// `September 2026`, for a header.
    pub fn title(self) -> String {
        format!("{} {}", month_name(self.month), self.year)
    }
}

#[derive(Script, Widget)]
pub struct MpDate {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpDate,
    #[live]
    draw_title: DrawText,
    #[live]
    draw_weekday: DrawText,
    #[live]
    draw_day: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    week_start: WeekStart,

    /// The month being shown. In `#[rust]`, and written only through `show`.
    #[rust]
    view: Option<YearMonth>,
    /// The chosen day, or `None`.
    #[rust]
    selected: Option<(i32, u32, u32)>,
    /// What the caller says today is. **Never read from the clock**; see the module
    /// doc.
    #[rust]
    today: Option<(i32, u32, u32)>,
    /// The grid cell under the pointer.
    #[rust]
    hovered: Option<usize>,
    /// Which header control is under the pointer: 0 is the previous arrow, 1 the
    /// next.
    #[rust]
    hovered_nav: Option<usize>,
    #[rust]
    area: Area,
}

/// The header row's height: a month name and two arrows on a `control_radius`
/// target. Chosen against [`crate::mp::control`]'s regular height, which it matches,
/// so the arrows are the size every other small control in the library is.
const HEADER_H: f64 = 24.0;
/// The weekday row: seven single letters, which need less than a line.
const WEEKDAY_H: f64 = 20.0;
/// One day cell. Square, because a day is a number and a number in a wide box leaves
/// two gaps where the reader expects one edge.
const CELL: f64 = 28.0;
/// How wide the arrows' targets are, on each side of the title.
const NAV_W: f64 = 26.0;
/// The previous-month arrow. A guillemet, for the reason `draw_walk` records where it
/// is drawn: the icon face's chevrons are not in this port's subset.
const PREV: &str = "\u{2039}";
/// The next-month arrow.
const NEXT: &str = "\u{203A}";

impl MpDate {
    pub fn view(&self) -> YearMonth {
        self.view.unwrap_or(YearMonth::new(1970, 1))
    }

    pub fn selected(&self) -> Option<(i32, u32, u32)> {
        self.selected
    }

    pub fn view_changed(&self, actions: &Actions) -> Option<YearMonth> {
        self.actions(actions).and_then(|a| match a {
            MpDateAction::ViewChanged { year, month } => Some(YearMonth::new(*year, *month)),
            _ => None,
        })
    }

    pub fn day_selected(&self, actions: &Actions) -> Option<(i32, u32, u32)> {
        self.actions(actions).and_then(|a| match a {
            MpDateAction::Selected { year, month, day } => Some((*year, *month, *day)),
            _ => None,
        })
    }

    fn actions<'a>(&self, actions: &'a Actions) -> Option<&'a MpDateAction> {
        crate::mp::action::first::<MpDateAction>(self.widget_uid(), actions)
    }

    /// Show a month, without reporting.
    pub fn show(&mut self, cx: &mut Cx, view: YearMonth) {
        if self.view == Some(view) {
            return;
        }
        self.view = Some(view);
        self.hovered = None;
        self.redraw(cx);
    }

    /// Move the view, reporting it.
    pub fn shift(&mut self, cx: &mut Cx, delta: i32) {
        let moved = self.view().shift(delta);
        self.view = Some(moved);
        self.hovered = None;
        cx.widget_action(
            self.widget_uid(),
            MpDateAction::ViewChanged {
                year: moved.year,
                month: moved.month,
            },
        );
        self.redraw(cx);
    }

    /// Set the chosen day, without reporting.
    pub fn set_selected(&mut self, cx: &mut Cx, day: Option<(i32, u32, u32)>) {
        if self.selected == day {
            return;
        }
        self.selected = day;
        self.redraw(cx);
    }

    /// Tell the calendar what today is.
    pub fn set_today(&mut self, cx: &mut Cx, today: Option<(i32, u32, u32)>) {
        if self.today == today {
            return;
        }
        self.today = today;
        self.redraw(cx);
    }

    /// Choose a cell as the user would, reporting it — and following the view if the
    /// cell belongs to the month on screen.
    pub fn select_cell(&mut self, cx: &mut Cx, index: usize) {
        let view = self.view();
        let cells = month_grid(view.year, view.month, self.week_start);
        let Some(day) = cells.get(index).copied().filter(|d| *d != 0) else {
            // A padding cell is a day of a neighbouring month, and this widget draws
            // it as an empty cell rather than as a dimmed day — so there is nothing
            // to choose. Recorded here rather than left implicit: a calendar that
            // reported a neighbouring month's day from a blank cell would be
            // choosing something the reader cannot see.
            return;
        };
        self.selected = Some((view.year, view.month, day));
        cx.widget_action(
            self.widget_uid(),
            MpDateAction::Selected {
                year: view.year,
                month: view.month,
                day,
            },
        );
        self.redraw(cx);
    }

    /// The width the widget needs: seven cells.
    fn content_width(&self) -> f64 {
        CELL * GRID_COLS as f64
    }

    /// The height: header, weekday row, six cell rows.
    fn content_height(&self) -> f64 {
        HEADER_H + WEEKDAY_H + CELL * GRID_ROWS as f64
    }

    /// Where the grid starts, in the widget's own coordinates.
    fn grid_origin(&self) -> DVec2 {
        dvec2(0.0, HEADER_H + WEEKDAY_H)
    }

    /// The header control at a position in the widget's own coordinates.
    fn nav_at(&self, x: f64, y: f64, width: f64) -> Option<usize> {
        if !x.is_finite() || !y.is_finite() || y < 0.0 || y >= HEADER_H {
            return None;
        }
        // The arrows are the **outer** targets, not adjacent to the title: the title
        // is centred and the arrows are at the edges, so a reader aiming at the far
        // left of the header gets the previous arrow without having to find its
        // rectangle.
        if x >= 0.0 && x < NAV_W {
            Some(0)
        } else if x >= width - NAV_W && x < width {
            Some(1)
        } else {
            None
        }
    }
}

/// Empty for the same reason `MpSegmented`'s is: there is no animator to seat and no
/// initial value to place, so a hook with a body here would be doing something it
/// should not.
impl ScriptHook for MpDate {
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {}
}

impl Widget for MpDate {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(fe) => {
                let local = fe.abs - self.area.rect(cx).pos;
                let width = self.area.rect(cx).size.x;
                self.hovered_nav = self.nav_at(local.x, local.y, width);
                self.hovered = if self.hovered_nav.is_some() {
                    None
                } else {
                    let origin = self.grid_origin();
                    cell_at(local.x - origin.x, local.y - origin.y, CELL, CELL)
                };
                cx.set_cursor(if self.hovered_nav.is_some() {
                    MouseCursor::Hand
                } else {
                    MouseCursor::Default
                });
                self.redraw(cx);
            }
            Hit::FingerMove(fe) => {
                let local = fe.abs - self.area.rect(cx).pos;
                let width = self.area.rect(cx).size.x;
                let hovered_nav = self.nav_at(local.x, local.y, width);
                let origin = self.grid_origin();
                let hovered = if hovered_nav.is_some() {
                    None
                } else {
                    cell_at(local.x - origin.x, local.y - origin.y, CELL, CELL)
                };
                if hovered != self.hovered || hovered_nav != self.hovered_nav {
                    self.hovered = hovered;
                    self.hovered_nav = hovered_nav;
                    self.redraw(cx);
                }
            }
            Hit::FingerHoverOut(_) => {
                if self.hovered.is_some() || self.hovered_nav.is_some() {
                    self.hovered = None;
                    self.hovered_nav = None;
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(fe) => {
                if !fe.is_over {
                    return;
                }
                let local = fe.abs - self.area.rect(cx).pos;
                let width = self.area.rect(cx).size.x;
                if let Some(nav) = self.nav_at(local.x, local.y, width) {
                    self.shift(cx, if nav == 0 { -1 } else { 1 });
                } else {
                    let origin = self.grid_origin();
                    if let Some(index) =
                        cell_at(local.x - origin.x, local.y - origin.y, CELL, CELL)
                    {
                        self.select_cell(cx, index);
                    }
                }
            }
            _ => {
                if cx.has_key_focus(self.area) {
                    if let Event::KeyDown(ke) = event {
                        if !ke.is_repeat {
                            match ke.key_code {
                                KeyCode::ArrowRight => self.shift(cx, 1),
                                KeyCode::ArrowLeft => self.shift(cx, -1),
                                KeyCode::ArrowDown => self.shift(cx, 12),
                                KeyCode::ArrowUp => self.shift(cx, -12),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Copied out before any mutable use of `cx`; see the note in `mp/table.rs`.
        let (panel, border, selected_plate, hover_plate, strong, muted, faint, accent, today_ring, radius) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.surface_card,
                p.border,
                p.solid,
                p.element_hover,
                p.text,
                p.text_muted,
                p.text_faint,
                p.accent,
                p.border_strong,
                makepad_theme::Theme::control_radius() as f32,
            )
        };

        // The view is seeded from the selection's month, or from `today`, or from the
        // epoch — in that order, because a calendar that opens on January 1970 because
        // nobody called `show` is a worse first frame than one that opens on the month
        // the app just told it about.
        if self.view.is_none() {
            let seed = self
                .selected
                .map(|(y, m, _)| YearMonth::new(y, m))
                .or_else(|| self.today.map(|(y, m, _)| YearMonth::new(y, m)))
                .unwrap_or(YearMonth::new(1970, 1));
            self.view = Some(seed);
        }
        let view = self.view();

        // States its own size: every part is drawn by `draw_abs` inside its own
        // turtle and nothing else can measure it. See `mp/table.rs`.
        self.walk.width = Size::Fixed(self.content_width());
        self.walk.height = Size::Fixed(self.content_height());
        let walk = self.walk;

        self.draw_bg.fill = panel;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);

        let rect = self.draw_bg.area().rect(cx.cx);
        self.area = self.draw_bg.area();

        // The title, centred in the header.
        self.draw_title.color = strong;
        let title = view.title();
        let title_w = text::width(&title, self.draw_title.text_style.font_size as f64);
        let title_h = self.draw_title.text_style.font_size as f64 * 1.2;
        self.draw_title.draw_abs(
            cx,
            dvec2(
                rect.pos.x + (rect.size.x - title_w) * 0.5,
                rect.pos.y + (HEADER_H - title_h) * 0.5,
            ),
            &title,
        );

        // The arrows are **guillemets from the text face, not chevrons from the icon
        // face**, and that is a finding rather than a preference: `\u{f053}` and
        // `\u{f054}` — FontAwesome's chevron-left and chevron-right — are not in this
        // port's icon subset, so the first render drew two tofu boxes either side of the
        // month. Nothing errored. The glyphs that *are* in the subset are the ones
        // already used elsewhere (`f002`, `f067`, `f00c`), and a single glyph that is
        // missing looks exactly like a glyph that is present until you look at it.
        self.draw_weekday.color = if self.hovered_nav == Some(0) {
            strong
        } else {
            muted
        };
        let nav_h = self.draw_weekday.text_style.font_size as f64 * 1.2;
        let nav_y = rect.pos.y + (HEADER_H - nav_h) * 0.5;
        let nav_w = text::width(PREV, self.draw_weekday.text_style.font_size as f64);
        self.draw_weekday.draw_abs(
            cx,
            dvec2(rect.pos.x + (NAV_W - nav_w) * 0.5, nav_y),
            PREV,
        );
        self.draw_weekday.color = if self.hovered_nav == Some(1) {
            strong
        } else {
            muted
        };
        self.draw_weekday.draw_abs(
            cx,
            dvec2(
                rect.pos.x + rect.size.x - NAV_W + (NAV_W - nav_w) * 0.5,
                nav_y,
            ),
            NEXT,
        );

        // The weekday row.
        self.draw_weekday.color = faint;
        let initials = weekday_initials(self.week_start);
        for (col, letter) in initials.iter().enumerate() {
            let letter_w = text::width(letter, self.draw_weekday.text_style.font_size as f64);
            self.draw_weekday.draw_abs(
                cx,
                dvec2(
                    rect.pos.x + col as f64 * CELL + (CELL - letter_w) * 0.5,
                    rect.pos.y + HEADER_H + (WEEKDAY_H - nav_h) * 0.5,
                ),
                letter,
            );
        }

        // The grid.
        let origin = self.grid_origin();
        let cells = month_grid(view.year, view.month, self.week_start);
        let day_h = self.draw_day.text_style.font_size as f64 * 1.2;
        for (index, day) in cells.iter().enumerate() {
            if *day == 0 {
                // A padding cell draws nothing. Not a dimmed neighbouring day, which
                // would need a rule for what happens when the reader clicks it and a
                // second month's arithmetic to fill it.
                continue;
            }
            let col = (index % GRID_COLS) as f64;
            let row = (index / GRID_COLS) as f64;
            let cell_pos = rect.pos + origin + dvec2(col * CELL, row * CELL);
            let is_selected = self.selected == Some((view.year, view.month, *day));
            let is_today = self.today == Some((view.year, view.month, *day));
            let is_hovered = self.hovered == Some(index);

            if is_selected {
                // The chosen day is a filled plate with the ink that reads on it, the
                // same pair the rest of the library uses for a chosen thing.
                self.draw_bg.fill = selected_plate;
                self.draw_bg.border_width = 0.0;
                self.draw_bg.radius = CELL as f32 * 0.5;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: cell_pos,
                        size: dvec2(CELL, CELL),
                    },
                );
                self.draw_bg.border_width = 1.0;
                self.draw_bg.fill = panel;
            } else if is_hovered {
                self.draw_bg.fill = hover_plate;
                self.draw_bg.border_width = 0.0;
                self.draw_bg.radius = CELL as f32 * 0.5;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: cell_pos,
                        size: dvec2(CELL, CELL),
                    },
                );
                self.draw_bg.border_width = 1.0;
                self.draw_bg.fill = panel;
            } else if is_today {
                // Today is an **outline**, not a fill: it is a fact about the date,
                // not a choice, and a filled plate would make it read as one.
                self.draw_bg.fill = Vec4f::default();
                self.draw_bg.border = today_ring;
                self.draw_bg.border_width = 1.0;
                self.draw_bg.radius = CELL as f32 * 0.5;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: cell_pos,
                        size: dvec2(CELL, CELL),
                    },
                );
                self.draw_bg.border = border;
            }

            let label = day.to_string();
            let label_w = text::width(&label, self.draw_day.text_style.font_size as f64);
            // The selected day's number is painted in the ink that reads on the plate
            // it sits on, and a day in the future is not receded — a calendar is read
            // both ways, so there is no "past is muted" rule here.
            self.draw_day.color = if is_selected { accent } else { strong };
            self.draw_day.draw_abs(
                cx,
                dvec2(
                    cell_pos.x + (CELL - label_w) * 0.5,
                    cell_pos.y + (CELL - day_h) * 0.5,
                ),
                &label,
            );
        }

        DrawStep::done()
    }
}

impl MpDateRef {
    pub fn view(&self) -> Option<YearMonth> {
        self.borrow().map(|inner| inner.view())
    }

    pub fn selected(&self) -> Option<(i32, u32, u32)> {
        self.borrow().and_then(|inner| inner.selected())
    }

    pub fn day_selected(&self, actions: &Actions) -> Option<(i32, u32, u32)> {
        self.borrow().and_then(|inner| inner.day_selected(actions))
    }

    pub fn view_changed(&self, actions: &Actions) -> Option<YearMonth> {
        self.borrow().and_then(|inner| inner.view_changed(actions))
    }

    pub fn show(&self, cx: &mut Cx, view: YearMonth) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx, view);
        }
    }

    pub fn set_selected(&self, cx: &mut Cx, day: Option<(i32, u32, u32)>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, day);
        }
    }

    pub fn set_today(&self, cx: &mut Cx, today: Option<(i32, u32, u32)>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_today(cx, today);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0 = Sunday, so the names a test reads are the names a reader knows.
    fn weekday_name(index: u32) -> &'static str {
        ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
            [index as usize % 7]
    }

    #[test]
    fn test_the_three_leap_rules() {
        // Divisible by four: leap.
        assert!(is_leap(2024));
        assert!(is_leap(1996));
        // A century not divisible by 400: NOT leap. This is the rule an
        // approximation gets wrong for one year in a hundred.
        assert!(!is_leap(1900));
        assert!(!is_leap(2100));
        assert!(!is_leap(1800));
        // A century divisible by 400: leap.
        assert!(is_leap(2000));
        assert!(is_leap(1600));
        assert!(is_leap(2400));
        // Not divisible by four: not leap.
        assert!(!is_leap(2023));
        assert!(!is_leap(1999));
    }

    #[test]
    fn test_february_is_the_only_month_the_leap_rule_touches() {
        for year in [1900, 2000, 2023, 2024, 2100] {
            let leap = is_leap(year);
            for month in 1..=12u32 {
                let expected = match month {
                    2 => {
                        if leap {
                            29
                        } else {
                            28
                        }
                    }
                    4 | 6 | 9 | 11 => 30,
                    _ => 31,
                };
                assert_eq!(
                    days_in_month(year, month),
                    expected,
                    "{year}-{month} should have {expected} days"
                );
            }
        }
    }

    #[test]
    fn test_an_out_of_range_month_has_no_days() {
        // A caller's bug, answered with "nothing" rather than with 31 — which would
        // draw a row of blanks and hide it.
        assert_eq!(days_in_month(2024, 0), 0);
        assert_eq!(days_in_month(2024, 13), 0);
    }

    #[test]
    fn test_known_weekdays() {
        // The two anchors this rests on, plus dates whose weekday is well known.
        assert_eq!(weekday_name(weekday(1970, 1, 1)), "Thursday");
        assert_eq!(weekday_name(weekday(2000, 1, 1)), "Saturday");
        // Christmas 2024 was a Wednesday, and New Year's Day 2024 a Monday.
        assert_eq!(weekday_name(weekday(2024, 12, 25)), "Wednesday");
        assert_eq!(weekday_name(weekday(2024, 1, 1)), "Monday");
    }

    #[test]
    fn test_the_weekday_agrees_with_accumulating_one_day_at_a_time() {
        // The test that proves an **algorithm** rather than a date. A second,
        // independent computation: walk forward a day at a time from the epoch,
        // advancing the weekday by one and rolling over a month when the day count
        // runs out. It shares no arithmetic with `days_from_civil` — it never divides
        // by 400 or shifts March to the front — so an error in that function has to
        // be an error in the leap rule or the month lengths to agree with this.
        let (mut y, mut m, mut d) = (1970i32, 1u32, 1u32);
        let mut expected = 4u32; // 1970-01-01 was a Thursday, and Sunday is 0.
        for _ in 0..(366 * 31) {
            assert_eq!(
                weekday(y, m, d),
                expected,
                "{y}-{m}-{d} should be {}",
                weekday_name(expected)
            );
            d += 1;
            expected = (expected + 1) % 7;
            if d > days_in_month(y, m) {
                d = 1;
                m += 1;
                if m > 12 {
                    m = 1;
                    y += 1;
                }
            }
        }
    }

    #[test]
    fn test_the_calendar_repeats_exactly_every_four_hundred_years() {
        // The Gregorian cycle, and the reason a year divisible by 400 is a leap year:
        // 400 years is exactly 146097 days, which is a whole number of weeks. So every
        // date's weekday is the same 400 years later — a property that holds only if
        // the century rules are all correct, which makes it a stronger test than any
        // single anchor.
        for (month, day) in [(1u32, 1u32), (2, 29), (3, 1), (12, 31), (6, 15)] {
            assert_eq!(
                weekday(2000, month, day),
                weekday(2400, month, day),
                "2000-{month}-{day} and 2400-{month}-{day} should share a weekday"
            );
        }
    }

    #[test]
    fn test_a_date_before_the_epoch_is_not_off_by_a_day() {
        // `days_from_civil`'s era division carries a `- 399` so that floor division is
        // correct for years before 1970. Without it every date before the epoch is
        // wrong, and no test using a modern date would show it.
        //
        // Checked by the same independent walk, backwards this time.
        let (mut y, mut m, mut d) = (1970i32, 1u32, 1u32);
        let mut expected = 4u32;
        for _ in 0..(366 * 8) {
            // Step back one day.
            if d == 1 {
                if m == 1 {
                    y -= 1;
                    m = 12;
                } else {
                    m -= 1;
                }
                d = days_in_month(y, m);
            } else {
                d -= 1;
            }
            expected = (expected + 6) % 7; // one day back is -1, and 6 ≡ -1 (mod 7)
            assert_eq!(
                weekday(y, m, d),
                expected,
                "{y}-{m}-{d} should be {}",
                weekday_name(expected)
            );
        }
    }

    #[test]
    fn test_days_since_epoch_and_the_date_round_trip() {
        // The test that proves **both** directions at once: an expected value on one
        // side would only check the other. Walked from a century before the epoch to a
        // century after, so the pre-1970 branch of the era division is covered and the
        // two `719468` constants have to agree.
        for days in (-36525..=36525).step_by(7) {
            let (year, month, day) = date_from_days_since_epoch(days);
            assert_eq!(
                days_since_epoch(year, month, day),
                days,
                "day {days} came back as {year}-{month}-{day}"
            );
            assert!((1..=12).contains(&month), "month {month} from day {days}");
            assert!(
                (1..=days_in_month(year, month)).contains(&day),
                "{year}-{month}-{day} is not a date"
            );
        }
    }

    #[test]
    fn test_a_timezone_east_of_utc_can_be_a_day_ahead() {
        // The exact failure this port hit, as a test. 2026-09-16 18:00 UTC is
        // 2026-09-17 02:00 at UTC+8 — the same instant, a different *date*, and the
        // date a person in that zone writes on a form.
        let at_18_utc = days_since_epoch(2026, 9, 16) * 86_400 + 18 * 3600;
        assert_eq!(
            date_from_epoch_seconds(at_18_utc, 0),
            (2026, 9, 16),
            "UTC says the 16th"
        );
        assert_eq!(
            date_from_epoch_seconds(at_18_utc, 8 * 3600),
            (2026, 9, 17),
            "UTC+8 says the 17th, and UTC+8 is the reader's answer"
        );
    }

    #[test]
    fn test_a_timezone_west_of_utc_can_be_a_day_behind() {
        // The mirror case, so the fix is not a `+ 1` that happens to work eastwards.
        // 2026-09-17 03:00 UTC is 2026-09-16 22:00 at UTC-5.
        let at_03_utc = days_since_epoch(2026, 9, 17) * 86_400 + 3 * 3600;
        assert_eq!(date_from_epoch_seconds(at_03_utc, 0), (2026, 9, 17));
        assert_eq!(
            date_from_epoch_seconds(at_03_utc, -5 * 3600),
            (2026, 9, 16),
            "UTC-5 is still on the 16th"
        );
    }

    #[test]
    fn test_a_negative_timestamp_in_a_zone_floors_rather_than_truncates() {
        // `div_euclid`, not `/`: for a timestamp before the epoch the two disagree, and
        // truncation towards zero lands a day late. Checked across a day boundary at
        // midnight UTC, one hour before the epoch.
        let one_hour_before = -3600;
        assert_eq!(
            date_from_epoch_seconds(one_hour_before, 0),
            (1969, 12, 31),
            "one hour before the epoch is the last day of 1969"
        );
        assert_eq!(date_from_epoch_seconds(-86_400, 0), (1969, 12, 31));
        assert_eq!(date_from_epoch_seconds(-86_401, 0), (1969, 12, 30));
    }

    #[test]
    fn test_the_epoch_itself_and_its_neighbours() {
        assert_eq!(date_from_days_since_epoch(0), (1970, 1, 1));
        assert_eq!(date_from_days_since_epoch(1), (1970, 1, 2));
        assert_eq!(date_from_days_since_epoch(-1), (1969, 12, 31));
        assert_eq!(date_from_days_since_epoch(11016), (2000, 2, 29), "the leap day");
        assert_eq!(days_since_epoch(2000, 2, 29), 11016);
    }

    #[test]
    fn test_a_month_starts_in_the_column_its_first_weekday_names() {
        // February 2026 starts on a Sunday, and the grid is Sunday-first, so its first
        // day is in column 0 of the first row.
        let cells = month_grid(2026, 2, WeekStart::Sunday);
        assert_eq!(cells[0], 1);
        // March 2026 starts on a Sunday too, so its first cell is also column 0.
        let cells = month_grid(2026, 3, WeekStart::Sunday);
        assert_eq!(cells[0], 1);
        // May 2026 starts on a Friday, which is column 5.
        let cells = month_grid(2026, 5, WeekStart::Sunday);
        assert_eq!(cells[5], 1, "May 2026 starts on a Friday");
        assert!(cells[..5].iter().all(|d| *d == 0));
    }

    #[test]
    fn test_the_grid_holds_every_day_of_the_month_in_order() {
        // The invariant that matters more than any single cell: no day is missing and
        // none is out of order. Run across a whole Gregorian cycle, so every month
        // length, every offset and every leap year is covered.
        for year in 1970..2370 {
            for month in 1..=12u32 {
                let days = days_in_month(year, month);
                for week_start in [WeekStart::Sunday, WeekStart::Monday] {
                    let cells = month_grid(year, month, week_start);
                    let present: Vec<u32> = cells.iter().copied().filter(|d| *d != 0).collect();
                    assert_eq!(
                        present.len(),
                        days as usize,
                        "{year}-{month} ({week_start:?}) should hold {days} days"
                    );
                    assert_eq!(
                        present,
                        (1..=days).collect::<Vec<u32>>(),
                        "{year}-{month} ({week_start:?}) should hold its days in order"
                    );
                    // And the real days sit in consecutive cells: a gap would mean a
                    // day drawn in the wrong column.
                    let first = cells.iter().position(|d| *d == 1).unwrap();
                    let last = cells.iter().position(|d| *d == days).unwrap();
                    assert_eq!(last - first + 1, days as usize);
                }
            }
        }
    }

    #[test]
    fn test_six_rows_always_fit() {
        // The property that lets the widget state a fixed height. The worst case is
        // an offset of 6 and a 31-day month, which is 37 cells; the grid is 42.
        for month in 1..=12u32 {
            for week_start in [WeekStart::Sunday, WeekStart::Monday] {
                let offset = week_start.offset(weekday(2026, month, 1)) as usize;
                assert!(
                    offset + 31 <= GRID_CELLS,
                    "month {month} with {week_start:?} needs {} cells",
                    offset + 31
                );
            }
        }
    }

    #[test]
    fn test_a_monday_first_grid_is_the_sunday_grid_shifted_by_one_column() {
        // The rotation, stated as a property rather than as a second expected array.
        for month in 1..=12u32 {
            let sunday = month_grid(2026, month, WeekStart::Sunday);
            let monday = month_grid(2026, month, WeekStart::Monday);
            for (index, day) in monday.iter().enumerate() {
                if *day == 0 {
                    continue;
                }
                // The same day, one column left — and the Sunday-first grid's day 1 is
                // at the column that names its weekday.
                let sunday_index_of_day = sunday.iter().position(|d| d == day).unwrap();
                let col_monday = index % GRID_COLS;
                let col_sunday = sunday_index_of_day % GRID_COLS;
                assert_eq!(
                    (col_sunday + 6) % 7,
                    col_monday,
                    "month {month} day {day} should sit one column left under Monday"
                );
            }
        }
    }

    #[test]
    fn test_the_weekday_initials_are_rotated_and_never_duplicated_wrongly() {
        assert_eq!(weekday_initials(WeekStart::Sunday)[0], "S");
        assert_eq!(weekday_initials(WeekStart::Monday)[0], "M");
        // Sunday is the *last* column when the week starts on Monday, and it is the
        // same letter — so a rotation that dropped a day would still look plausible
        // and be wrong. Counting is the check.
        assert_eq!(weekday_initials(WeekStart::Monday)[6], "S");
        for start in [WeekStart::Sunday, WeekStart::Monday] {
            let initials = weekday_initials(start);
            assert_eq!(initials.len(), 7);
            assert_eq!(initials.iter().filter(|s| **s == "S").count(), 2);
            assert_eq!(initials.iter().filter(|s| **s == "T").count(), 2);
        }
    }

    #[test]
    fn test_shifting_a_month_carries_the_year() {
        assert_eq!(shift_month(2026, 1, 1), (2026, 2));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1), "December plus one");
        assert_eq!(shift_month(2026, 1, -1), (2025, 12), "January minus one");
        assert_eq!(shift_month(2026, 6, -6), (2025, 12));
        assert_eq!(shift_month(2026, 6, 6), (2026, 12));
        assert_eq!(shift_month(2026, 6, 0), (2026, 6));
        // Multi-year, in both directions, and an even multiple of twelve.
        assert_eq!(shift_month(2026, 3, 25), (2028, 4));
        assert_eq!(shift_month(2026, 3, -25), (2024, 2));
        assert_eq!(shift_month(2026, 7, 12), (2027, 7));
        assert_eq!(shift_month(2026, 7, -12), (2025, 7));
        // And it stays in range for any delta: no month 0 or 13 ever comes out.
        for delta in -500..=500 {
            let (_, month) = shift_month(2026, 6, delta);
            assert!((1..=12).contains(&month), "delta {delta} gave month {month}");
        }
    }

    #[test]
    fn test_shifting_twelve_months_lands_on_the_same_month_a_year_away() {
        // The property a calendar's arrows rely on: paging a year moves the month
        // name back to where it started.
        for month in 1..=12u32 {
            let (y, m) = shift_month(2026, month, 12);
            assert_eq!((y, m), (2027, month));
            let (y, m) = shift_month(2026, month, -12);
            assert_eq!((y, m), (2025, month));
        }
    }

    #[test]
    fn test_the_cell_hit_test_finds_the_cell_a_point_is_in() {
        let (w, h) = (CELL, CELL);
        assert_eq!(cell_at(0.0, 0.0, w, h), Some(0));
        assert_eq!(cell_at(CELL - 0.1, CELL - 0.1, w, h), Some(0));
        assert_eq!(cell_at(CELL, 0.0, w, h), Some(1), "the next column");
        assert_eq!(cell_at(0.0, CELL, w, h), Some(GRID_COLS), "the next row");
        assert_eq!(
            cell_at(CELL * 6.0, CELL * 5.0, w, h),
            Some(GRID_CELLS - 1),
            "the last cell"
        );
    }

    #[test]
    fn test_the_cell_hit_test_refuses_a_point_outside_the_grid() {
        let (w, h) = (CELL, CELL);
        assert_eq!(cell_at(-0.1, 0.0, w, h), None);
        assert_eq!(cell_at(0.0, -0.1, w, h), None);
        assert_eq!(cell_at(CELL * 7.0, 0.0, w, h), None, "one column past the last");
        assert_eq!(cell_at(0.0, CELL * 6.0, w, h), None, "one row past the last");
        assert_eq!(cell_at(1000.0, 1000.0, w, h), None);
    }

    #[test]
    fn test_the_cell_hit_test_refuses_a_position_that_is_not_a_number() {
        // The guard `mp/segmented.rs` learned the hard way: a NaN passes every `<` and
        // `>=`, so a bounds check alone lets it through and `NaN.floor() as usize`
        // saturates to 0 — a hit on the first cell for any pointer at all.
        let (w, h) = (CELL, CELL);
        assert_eq!(cell_at(f64::NAN, 0.0, w, h), None);
        assert_eq!(cell_at(0.0, f64::NAN, w, h), None);
        assert_eq!(cell_at(0.0, 0.0, f64::NAN, h), None);
        assert_eq!(cell_at(0.0, 0.0, w, f64::NAN), None);
        assert_eq!(cell_at(f64::INFINITY, 0.0, w, h), None);
    }

    #[test]
    fn test_the_hit_test_and_the_drawn_grid_agree() {
        // The hit test is arithmetic and the grid is a loop over the same cell size, so
        // they can only disagree if one of them changes. This walks every cell and
        // checks the position the widget would *draw* that cell at maps back to it.
        for index in 0..GRID_CELLS {
            let col = (index % GRID_COLS) as f64;
            let row = (index / GRID_COLS) as f64;
            let drawn_at = dvec2(col * CELL, row * CELL);
            assert_eq!(
                cell_at(drawn_at.x, drawn_at.y, CELL, CELL),
                Some(index),
                "cell {index} at {drawn_at:?}"
            );
            // And its centre, which is where a click really lands.
            let centre = drawn_at + dvec2(CELL * 0.5, CELL * 0.5);
            assert_eq!(cell_at(centre.x, centre.y, CELL, CELL), Some(index));
        }
    }

    #[test]
    fn test_a_year_month_titles_itself() {
        assert_eq!(YearMonth::new(2026, 9).title(), "September 2026");
        assert_eq!(YearMonth::new(2026, 1).title(), "January 2026");
        assert_eq!(YearMonth::new(2026, 12).title(), "December 2026");
        // An out-of-range month titles as an empty name rather than panicking, for the
        // same reason `days_in_month` returns 0: a caller's bug should be visible.
        assert_eq!(YearMonth::new(2026, 13).title(), " 2026");
    }

    #[test]
    fn test_shifting_a_year_month_round_trips() {
        // What a pair of arrows does: there and back is where you started.
        for month in 1..=12u32 {
            for delta in [1, 2, 11, 12, 25] {
                let start = YearMonth::new(2026, month);
                assert_eq!(start.shift(delta).shift(-delta), start);
            }
        }
    }
}
