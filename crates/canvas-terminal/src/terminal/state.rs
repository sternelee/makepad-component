use rmux_sdk::{PaneColor, PaneSnapshot};

/// ANSI color index → RGB. Standard 16-color palette (dark background scheme).
const ANSI_COLORS: [[f32; 3]; 16] = [
    [0.08, 0.09, 0.11], // 0 black
    [0.95, 0.26, 0.21], // 1 red
    [0.62, 0.78, 0.34], // 2 green
    [0.95, 0.75, 0.28], // 3 yellow
    [0.37, 0.57, 0.90], // 4 blue
    [0.90, 0.39, 0.70], // 5 magenta
    [0.35, 0.82, 0.85], // 6 cyan
    [0.91, 0.93, 0.95], // 7 white
    [0.36, 0.39, 0.43], // 8 bright black
    [0.98, 0.47, 0.42], // 9 bright red
    [0.78, 0.90, 0.55], // 10 bright green
    [0.98, 0.86, 0.50], // 11 bright yellow
    [0.56, 0.72, 0.98], // 12 bright blue
    [0.98, 0.62, 0.82], // 13 bright magenta
    [0.55, 0.90, 0.93], // 14 bright cyan
    [1.0, 1.0, 1.0],    // 15 bright white
];

pub const DEFAULT_FG: [f32; 3] = [0.86, 0.89, 0.94];
pub const DEFAULT_BG: [f32; 3] = [0.10, 0.11, 0.14];

/// A single terminal grid cell.
#[derive(Clone, Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: [f32; 3],
    pub bg: [f32; 3],
    pub bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            bold: false,
        }
    }
}

/// Terminal grid state, updated from rmux `PaneSnapshot` snapshots.
pub struct TerminalState {
    pub cols: usize,
    pub rows: usize,
    pub lines: Vec<Vec<Cell>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub cursor_visible: bool,
    /// Cursor style from the PTY: 0=block, 1=beam(bar), 2=underline.
    pub cursor_style: u8,
    /// Text selection in cell coords: (start_row, start_col, end_row, end_col).
    pub selection: Option<(usize, usize, usize, usize)>,
    /// Lines scrolled above the viewport (scrollback), captured from the
    /// rmux daemon. Index 0 is the oldest visible history line.
    pub history: Vec<String>,
    /// How many history lines are shown above the current screen (0 = tail).
    pub scroll_offset: usize,
}

impl TerminalState {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            lines: vec![vec![Cell::default(); cols]; rows],
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            cursor_style: 0,
            selection: None,
            history: Vec::new(),
            scroll_offset: 0,
        }
    }

    /// Replace the entire grid from a rmux `PaneSnapshot`. This is the
    /// authoritative source of truth — the snapshot reflects the actual
    /// rendered screen (after all vte processing by the PTY), so glyphs,
    /// colors, cursor, and dimensions are always correct, including after
    /// resize/zoom.
    pub fn apply_snapshot(&mut self, snap: &PaneSnapshot) {
        let cols = snap.cols as usize;
        let rows = snap.rows as usize;
        self.cols = cols.max(1);
        self.rows = rows.max(1);

        // Rebuild the grid to match the snapshot dimensions.
        if self.lines.len() != self.rows {
            self.lines
                .resize(self.rows, vec![Cell::default(); self.cols]);
        }
        for line in self.lines.iter_mut() {
            if line.len() != self.cols {
                line.resize(self.cols, Cell::default());
            }
        }

        // Fill cells from the snapshot (row-major: index = row * cols + col).
        for r in 0..self.rows {
            for c in 0..self.cols {
                let idx = r * self.cols + c;
                if let Some(pcell) = snap.cells.get(idx) {
                    let cell = &mut self.lines[r][c];
                    // Use the first character of the glyph text, or space.
                    cell.ch = pcell.glyph.text.chars().next().unwrap_or(' ');
                    // Skip padding cells (trailing column of wide glyphs).
                    if pcell.glyph.padding {
                        cell.ch = ' ';
                    }
                    cell.fg = pane_color_to_rgb(&pcell.foreground, true);
                    cell.bg = pane_color_to_rgb(&pcell.background, false);
                    cell.bold = pcell.attributes.bits & rmux_sdk::PaneAttributes::BOLD.bits != 0;
                }
            }
        }

        self.cursor_row = snap.cursor.row as usize;
        self.cursor_col = snap.cursor.col as usize;
        self.cursor_visible = snap.cursor.visible;
        // style: rmux PaneCursor.style raw value (0 block, 1 bar, 2 underline).
        self.cursor_style = (snap.cursor.style & 0xff) as u8;
        // Selection persists across snapshots (user-driven).
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Whether the cell at (row, col) is inside the current selection.
    pub fn in_selection(&self, row: usize, col: usize) -> bool {
        let Some((r0, c0, r1, c1)) = self.selection else {
            return false;
        };
        let (ra, rb) = (r0.min(r1), r0.max(r1));
        let (ca, cb) = (c0.min(c1), c0.max(c1));
        if row < ra || row > rb {
            return false;
        }
        if row == ra && row == rb {
            return col >= ca && col <= cb;
        }
        if row == ra {
            return col >= ca;
        }
        if row == rb {
            return col <= cb;
        }
        true
    }

    /// Extract the selected text as a plain string.
    pub fn selected_text(&self) -> String {
        let Some((r0, c0, r1, c1)) = self.selection else {
            return String::new();
        };
        let (ra, rb) = (r0.min(r1), r0.max(r1));
        let (ca, cb) = (c0.min(c1), c0.max(c1));
        let mut out = String::new();
        for r in ra..=rb.min(self.rows - 1) {
            if r > ra {
                out.push('\n');
            }
            let line = &self.lines[r];
            let start = if r == ra { ca } else { 0 };
            let end = if r == rb { cb } else { line.len().saturating_sub(1) };
            let end = end.min(line.len().saturating_sub(1));
            for cell in line.iter().take(end + 1).skip(start) {
                if cell.ch != '\0' {
                    out.push(cell.ch);
                }
            }
            // Trim trailing spaces per line (terminal-style copy).
            while out.ends_with(' ') {
                out.pop();
            }
        }
        out
    }
}

/// Convert a rmux `PaneColor` to an RGB triple.
fn pane_color_to_rgb(color: &PaneColor, is_fg: bool) -> [f32; 3] {
    match color {
        PaneColor::Default | PaneColor::Terminal => {
            if is_fg {
                DEFAULT_FG
            } else {
                DEFAULT_BG
            }
        }
        PaneColor::None => DEFAULT_BG,
        PaneColor::Ansi { index } => ANSI_COLORS[(*index as usize).min(15)],
        PaneColor::BrightAnsi { index } => ANSI_COLORS[((*index as i32 - 90 + 8).clamp(0, 15)) as usize],
        PaneColor::Indexed { index } => index_color(*index as usize),
        PaneColor::Rgb { red, green, blue } => [
            *red as f32 / 255.0,
            *green as f32 / 255.0,
            *blue as f32 / 255.0,
        ],
        PaneColor::Encoded { .. } => DEFAULT_BG,
        _ => DEFAULT_BG,
    }
}

/// Map a 256-color index to RGB.
fn index_color(idx: usize) -> [f32; 3] {
    if idx < 16 {
        return ANSI_COLORS[idx];
    }
    if idx < 232 {
        let n = idx - 16;
        let r = n / 36;
        let g = (n % 36) / 6;
        let b = n % 6;
        let v = |c: usize| [0, 95, 135, 175, 215, 255][c];
        return [
            v(r) as f32 / 255.0,
            v(g) as f32 / 255.0,
            v(b) as f32 / 255.0,
        ];
    }
    let g = 8 + (idx - 232) * 10;
    [g as f32 / 255.0, g as f32 / 255.0, g as f32 / 255.0]
}
