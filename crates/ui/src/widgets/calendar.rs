use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    pub MpCalendar = {{MpCalendar}} {
        width: Fit
        height: Fit
        flow: Down

        draw_bg: {
            instance bg_color: #0000

            fn pixel(self) -> vec4 {
                return self.bg_color;
            }
        }

        draw_cell: {
            instance border_color: #5588bb66
            instance border_width: 0.5

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(
                    self.border_width,
                    self.border_width,
                    self.rect_size.x - self.border_width * 2.0,
                    self.rect_size.y - self.border_width * 2.0,
                    2.0
                );
                sdf.fill_keep(self.color);
                sdf.stroke(self.border_color, self.border_width);
                return sdf.result;
            }
        }

        draw_text: {
            text_style: <THEME_FONT_REGULAR> { font_size: 11.0, line_spacing: 1.3 }
            color: #FFFFFF
        }

        draw_header_text: {
            text_style: <THEME_FONT_BOLD> { font_size: 13.0, line_spacing: 1.3 }
            color: #FFFFFF
        }

        draw_divider: {
            color: #5588bb

            fn pixel(self) -> vec4 {
                return self.color;
            }
        }
    }
}

// ============================================================================
// Data types (no A2UI dependency)
// ============================================================================

/// Data for a single calendar cell
#[derive(Clone, Debug, Default)]
pub struct CalendarCellData {
    pub line1: String,
    pub line2: String,
    pub time: String,
    pub description: String,
    pub tips: String,
}

/// Configuration for the calendar grid structure
#[derive(Clone, Debug, Default)]
pub struct CalendarConfig {
    pub title: String,
    pub footer: String,
    pub column_headers: Vec<String>,
    pub column_subtitles: Vec<String>,
    pub row_labels: Vec<String>,
    pub row_color_hints: Vec<String>,
}

/// Actions emitted by MpCalendar
#[derive(Clone, Debug, DefaultNone)]
pub enum MpCalendarAction {
    CellClicked { row: usize, col: usize },
    None,
}

// ============================================================================
// MpCalendar Widget
// ============================================================================

#[derive(Live, LiveHook, Widget)]
pub struct MpCalendar {
    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[redraw]
    #[live]
    draw_cell: DrawColor,

    #[live]
    draw_text: DrawText,

    #[live]
    draw_header_text: DrawText,

    #[redraw]
    #[live]
    draw_divider: DrawColor,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live(900.0)]
    grid_width: f64,

    // Data (set by caller)
    #[rust]
    config: CalendarConfig,

    #[rust]
    cells: Vec<Vec<CalendarCellData>>,

    // Interaction state
    #[rust]
    cell_areas: Vec<Area>,

    #[rust]
    cell_meta: Vec<(usize, usize)>,

    #[rust]
    selected_cell: Option<(usize, usize)>,

    #[rust]
    hovered_idx: Option<usize>,

    #[rust]
    area: Area,
}

// ============================================================================
// Public API
// ============================================================================

impl MpCalendar {
    /// Set the grid structure configuration
    pub fn set_config(&mut self, config: CalendarConfig) {
        self.config = config;
    }

    /// Set data for a single cell
    pub fn set_cell(&mut self, row: usize, col: usize, data: CalendarCellData) {
        // Grow rows if needed
        while self.cells.len() <= row {
            self.cells.push(Vec::new());
        }
        // Grow cols if needed
        let num_cols = self.config.column_headers.len().max(col + 1);
        while self.cells[row].len() < num_cols {
            self.cells[row].push(CalendarCellData::default());
        }
        self.cells[row][col] = data;
    }

    /// Set all cell data at once
    pub fn set_all_cells(&mut self, cells: Vec<Vec<CalendarCellData>>) {
        self.cells = cells;
    }

    /// Check if a cell was clicked in the given actions
    pub fn cell_clicked(&self, actions: &Actions) -> Option<(usize, usize)> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let MpCalendarAction::CellClicked { row, col } = action.cast() {
                return Some((row, col));
            }
        }
        None
    }

    /// Get the currently selected cell
    pub fn selected_cell(&self) -> Option<(usize, usize)> {
        self.selected_cell
    }

    /// Set the selected cell programmatically
    pub fn set_selected_cell(&mut self, cell: Option<(usize, usize)>) {
        self.selected_cell = cell;
    }
}

// ============================================================================
// Widget trait implementation
// ============================================================================

impl Widget for MpCalendar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let mut needs_redraw = false;

        for (idx, area) in self.cell_areas.iter().enumerate() {
            match event.hits(cx, *area) {
                Hit::FingerHoverIn(_) => {
                    if self.hovered_idx != Some(idx) {
                        self.hovered_idx = Some(idx);
                        cx.set_cursor(MouseCursor::Hand);
                        needs_redraw = true;
                    }
                }
                Hit::FingerHoverOut(_) => {
                    if self.hovered_idx == Some(idx) {
                        self.hovered_idx = None;
                        cx.set_cursor(MouseCursor::Default);
                        needs_redraw = true;
                    }
                }
                Hit::FingerDown(_) => {
                    if let Some(&(row, col)) = self.cell_meta.get(idx) {
                        self.selected_cell = Some((row, col));
                        cx.widget_action(
                            self.widget_uid(),
                            &scope.path,
                            MpCalendarAction::CellClicked { row, col },
                        );
                        needs_redraw = true;
                    }
                }
                _ => {}
            }
        }

        if needs_redraw {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let num_cols = self.config.column_headers.len();
        let num_rows = self.config.row_labels.len();

        // Clear per-frame hit testing data
        self.cell_areas.clear();
        self.cell_meta.clear();

        self.draw_bg.begin(cx, walk, self.layout);

        if num_cols == 0 || num_rows == 0 {
            self.draw_bg.end(cx);
            self.area = self.draw_bg.area();
            return DrawStep::done();
        }

        let total_width = self.grid_width;
        let col_width = total_width / num_cols as f64;

        let selected_cell = self.selected_cell;
        let hovered_idx = self.hovered_idx;

        // Outer container
        cx.begin_turtle(
            Walk::new(Size::Fixed(total_width), Size::fit()),
            Layout {
                flow: Flow::Down,
                ..Layout::default()
            },
        );

        // === Title row ===
        if !self.config.title.is_empty() {
            let title_color = vec4(0.08, 0.10, 0.18, 1.0);
            self.draw_cell.color = title_color;
            self.draw_cell.begin(
                cx,
                Walk::new(Size::Fixed(total_width), Size::Fixed(45.0)),
                Layout {
                    align: Align { x: 0.5, y: 0.5 },
                    ..Layout::default()
                },
            );
            self.draw_header_text.text_style.font_size = 16.0;
            self.draw_header_text
                .draw_walk(cx, Walk::fit(), Align::default(), &self.config.title);
            self.draw_cell.end(cx);
        }

        // === Data rows ===
        let mut cell_counter = 0usize;

        for row_idx in 0..num_rows {
            let color_hint = self
                .config
                .row_color_hints
                .get(row_idx)
                .map(|s| s.as_str())
                .unwrap_or("");

            let base_row_color = Self::row_color(color_hint);
            let row_height = match color_hint {
                "header" => 55.0,
                "budget" => 40.0,
                _ => 70.0,
            };

            let row_label = self
                .config
                .row_labels
                .get(row_idx)
                .cloned()
                .unwrap_or_default();

            // Row container
            cx.begin_turtle(
                Walk::new(Size::Fixed(total_width), Size::Fixed(row_height)),
                Layout {
                    flow: Flow::right(),
                    ..Layout::default()
                },
            );

            for col_idx in 0..num_cols {
                let is_selected = selected_cell == Some((row_idx, col_idx));
                let is_hovered = hovered_idx == Some(cell_counter);

                let cell_color = if is_selected {
                    vec4(0.231, 0.510, 0.965, 1.0) // #3B82F6
                } else if is_hovered {
                    Self::brighten_color(base_row_color, 0.15)
                } else {
                    base_row_color
                };

                self.draw_cell.color = cell_color;
                self.draw_cell.begin(
                    cx,
                    Walk::new(Size::Fixed(col_width), Size::Fixed(row_height)),
                    Layout {
                        flow: Flow::Down,
                        padding: Padding {
                            left: 4.0,
                            right: 4.0,
                            top: 3.0,
                            bottom: 3.0,
                        },
                        ..Layout::default()
                    },
                );

                if color_hint == "header" {
                    // Header row: show column headers + subtitles
                    let header = self
                        .config
                        .column_headers
                        .get(col_idx)
                        .cloned()
                        .unwrap_or_default();
                    let subtitle = self
                        .config
                        .column_subtitles
                        .get(col_idx)
                        .cloned()
                        .unwrap_or_default();

                    self.draw_header_text.text_style.font_size = 13.0;
                    self.draw_header_text
                        .draw_walk(cx, Walk::fit(), Align::default(), &header);

                    if !subtitle.is_empty() {
                        self.draw_text.text_style.font_size = 10.0;
                        self.draw_text.color = vec4(0.7, 0.8, 0.9, 1.0);
                        self.draw_text
                            .draw_walk(cx, Walk::fit(), Align::default(), &subtitle);
                    }
                } else {
                    // Data cell: show row_label + cell data
                    let cell_data = self.cells.get(row_idx).and_then(|row| row.get(col_idx));

                    let line1 = cell_data.map(|c| c.line1.as_str()).unwrap_or("");
                    let line2 = cell_data.map(|c| c.line2.as_str()).unwrap_or("");

                    if !row_label.is_empty() {
                        self.draw_text.text_style.font_size = 9.0;
                        self.draw_text.color = vec4(0.6, 0.7, 0.8, 0.8);
                        self.draw_text
                            .draw_walk(cx, Walk::fit(), Align::default(), &row_label);
                    }

                    if !line1.is_empty() {
                        self.draw_header_text.text_style.font_size = 11.0;
                        self.draw_header_text
                            .draw_walk(cx, Walk::fit(), Align::default(), line1);
                    }

                    if !line2.is_empty() {
                        self.draw_text.text_style.font_size = 9.0;
                        self.draw_text.color = vec4(0.7, 0.8, 0.9, 0.9);
                        self.draw_text
                            .draw_walk(cx, Walk::fit(), Align::default(), line2);
                    }
                }

                self.draw_cell.end(cx);

                // Store cell area for hit testing
                let cell_area = self.draw_cell.area();
                self.cell_areas.push(cell_area);
                self.cell_meta.push((row_idx, col_idx));
                cell_counter += 1;
            }

            cx.end_turtle();
        }

        // === Footer row ===
        if !self.config.footer.is_empty() {
            let footer_color = vec4(0.08, 0.10, 0.18, 1.0);
            self.draw_cell.color = footer_color;
            self.draw_cell.begin(
                cx,
                Walk::new(Size::Fixed(total_width), Size::Fixed(40.0)),
                Layout {
                    align: Align { x: 0.5, y: 0.5 },
                    ..Layout::default()
                },
            );
            self.draw_header_text.text_style.font_size = 14.0;
            self.draw_header_text
                .draw_walk(cx, Walk::fit(), Align::default(), &self.config.footer);
            self.draw_cell.end(cx);
        }

        // === Detail panel for selected cell ===
        if let Some((sel_row, sel_col)) = selected_cell {
            self.render_detail(cx, sel_row, sel_col, total_width);
        }

        cx.end_turtle();

        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();

        DrawStep::done()
    }
}

// ============================================================================
// Private rendering helpers
// ============================================================================

impl MpCalendar {
    /// Render the detail panel below the calendar showing the full day schedule
    fn render_detail(&mut self, cx: &mut Cx2d, sel_row: usize, sel_col: usize, total_width: f64) {
        let num_rows = self.config.row_labels.len();
        let content_width = total_width - 32.0;

        // Day header
        let day_name = self
            .config
            .column_headers
            .get(sel_col)
            .cloned()
            .unwrap_or_else(|| format!("Day {}", sel_col + 1));
        let day_subtitle = self
            .config
            .column_subtitles
            .get(sel_col)
            .cloned()
            .unwrap_or_default();

        // Detail card container
        self.draw_cell.color = vec4(0.10, 0.12, 0.22, 1.0);
        self.draw_cell.begin(
            cx,
            Walk {
                width: Size::Fixed(total_width),
                height: Size::fit(),
                margin: Margin {
                    left: 0.0,
                    right: 0.0,
                    top: 12.0,
                    bottom: 0.0,
                },
                ..Walk::default()
            },
            Layout {
                flow: Flow::Down,
                padding: Padding {
                    left: 16.0,
                    right: 16.0,
                    top: 14.0,
                    bottom: 14.0,
                },
                spacing: 4.0,
                ..Layout::default()
            },
        );

        // Title
        let title = if day_subtitle.is_empty() {
            day_name.clone()
        } else {
            format!("{} | {}", day_name, day_subtitle)
        };
        self.draw_header_text.text_style.font_size = 18.0;
        self.draw_header_text.color = vec4(1.0, 1.0, 1.0, 1.0);
        self.draw_header_text
            .draw_walk(cx, Walk::fit(), Align::default(), &title);

        // Subtitle
        self.draw_text.text_style.font_size = 11.0;
        self.draw_text.color = vec4(0.5, 0.6, 0.7, 0.8);
        self.draw_text
            .draw_walk(cx, Walk::fit(), Align::default(), "Full Day Itinerary");

        // Divider
        self.draw_divider.draw_walk(
            cx,
            Walk {
                width: Size::Fixed(content_width),
                height: Size::Fixed(1.0),
                margin: Margin {
                    top: 4.0,
                    bottom: 8.0,
                    left: 0.0,
                    right: 0.0,
                },
                ..Walk::default()
            },
        );

        // Readable time labels and emoji for each slot
        let readable_labels = ["", "Morning", "Afternoon", "Evening", "Budget Summary"];
        let slot_emoji = ["", "  ", "  ", "  ", "  "];

        // Each time slot for this day (skip header row 0)
        for row_idx in 1..num_rows {
            let color_hint = self
                .config
                .row_color_hints
                .get(row_idx)
                .map(|s| s.as_str())
                .unwrap_or("");

            let cell_data = self.cells.get(row_idx).and_then(|row| row.get(sel_col));
            let line1 = cell_data.map(|c| c.line1.as_str()).unwrap_or("");
            let line2 = cell_data.map(|c| c.line2.as_str()).unwrap_or("");
            let time_range = cell_data.map(|c| c.time.as_str()).unwrap_or("");
            let description = cell_data.map(|c| c.description.as_str()).unwrap_or("");
            let tips = cell_data.map(|c| c.tips.as_str()).unwrap_or("");

            if line1.is_empty() && line2.is_empty() && description.is_empty() {
                continue;
            }

            let is_selected_slot = row_idx == sel_row;
            let time_label = readable_labels.get(row_idx).copied().unwrap_or("Other");
            let emoji = slot_emoji.get(row_idx).copied().unwrap_or("");

            // Slot section container
            cx.begin_turtle(
                Walk {
                    width: Size::Fixed(content_width),
                    height: Size::fit(),
                    margin: Margin {
                        top: 2.0,
                        bottom: 6.0,
                        left: 0.0,
                        right: 0.0,
                    },
                    ..Walk::default()
                },
                Layout {
                    flow: Flow::Down,
                    padding: Padding {
                        left: 12.0,
                        right: 12.0,
                        top: 8.0,
                        bottom: 8.0,
                    },
                    spacing: 3.0,
                    ..Layout::default()
                },
            );

            // Slot header: emoji + time label + time range
            let header_text = if time_range.is_empty() {
                format!("{}{}", emoji, time_label)
            } else {
                format!("{}{}    {}", emoji, time_label, time_range)
            };
            self.draw_header_text.text_style.font_size = 15.0;
            self.draw_header_text.color = if is_selected_slot {
                match color_hint {
                    "morning" => vec4(1.0, 0.75, 0.3, 1.0),
                    "afternoon" => vec4(1.0, 0.85, 0.4, 1.0),
                    "evening" => vec4(0.4, 0.75, 1.0, 1.0),
                    "budget" => vec4(0.4, 0.9, 0.5, 1.0),
                    _ => vec4(0.4, 0.75, 1.0, 1.0),
                }
            } else {
                match color_hint {
                    "morning" => vec4(0.8, 0.6, 0.35, 1.0),
                    "afternoon" => vec4(0.85, 0.7, 0.4, 1.0),
                    "evening" => vec4(0.5, 0.65, 0.85, 1.0),
                    "budget" => vec4(0.5, 0.75, 0.5, 1.0),
                    _ => vec4(0.6, 0.7, 0.85, 1.0),
                }
            };
            self.draw_header_text
                .draw_walk(cx, Walk::fit(), Align::default(), &header_text);

            // Location name (bold)
            if !line1.is_empty() {
                let location = if line2.is_empty() {
                    line1.to_string()
                } else {
                    format!("{} - {}", line1, line2)
                };
                self.draw_header_text.text_style.font_size = 14.0;
                self.draw_header_text.color = if is_selected_slot {
                    vec4(1.0, 1.0, 1.0, 1.0)
                } else {
                    vec4(0.85, 0.9, 0.95, 1.0)
                };
                self.draw_header_text
                    .draw_walk(cx, Walk::fit(), Align::default(), &location);
            }

            // Description
            if !description.is_empty() {
                self.draw_text.text_style.font_size = 12.0;
                self.draw_text.color = if is_selected_slot {
                    vec4(0.85, 0.9, 0.95, 0.95)
                } else {
                    vec4(0.65, 0.72, 0.8, 0.9)
                };
                self.draw_text
                    .draw_walk(cx, Walk::fit(), Align::default(), description);
            }

            // Tips
            if !tips.is_empty() {
                let tips_text = format!("Tip: {}", tips);
                self.draw_text.text_style.font_size = 11.0;
                self.draw_text.color = if is_selected_slot {
                    vec4(0.6, 0.85, 0.6, 0.9)
                } else {
                    vec4(0.45, 0.65, 0.45, 0.8)
                };
                self.draw_text
                    .draw_walk(cx, Walk::fit(), Align::default(), &tips_text);
            }

            cx.end_turtle();

            // Thin divider between slots
            if row_idx < num_rows - 1 {
                self.draw_divider.draw_walk(
                    cx,
                    Walk {
                        width: Size::Fixed(content_width - 24.0),
                        height: Size::Fixed(1.0),
                        margin: Margin {
                            top: 0.0,
                            bottom: 0.0,
                            left: 12.0,
                            right: 0.0,
                        },
                        ..Walk::default()
                    },
                );
            }
        }

        self.draw_cell.end(cx);
    }

    /// Get the background color for a calendar row based on its color hint
    fn row_color(hint: &str) -> Vec4 {
        match hint {
            "header" => vec4(0.102, 0.165, 0.290, 1.0),
            "morning" => vec4(0.239, 0.169, 0.122, 1.0),
            "afternoon" => vec4(0.290, 0.208, 0.125, 1.0),
            "evening" => vec4(0.102, 0.165, 0.227, 1.0),
            "budget" => vec4(0.165, 0.165, 0.102, 1.0),
            _ => vec4(0.133, 0.133, 0.200, 1.0),
        }
    }

    /// Brighten a color by a factor (for hover effect)
    fn brighten_color(color: Vec4, amount: f32) -> Vec4 {
        vec4(
            (color.x + amount).min(1.0),
            (color.y + amount).min(1.0),
            (color.z + amount).min(1.0),
            color.w,
        )
    }
}
