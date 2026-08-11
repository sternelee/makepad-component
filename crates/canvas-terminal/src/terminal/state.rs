use vte::Perform;

/// ANSI color index → RGB. Standard 16-color palette (dark background scheme).
const ANSI_COLORS: [[f32; 3]; 16] = [
    // 0 black
    [0.08, 0.09, 0.11],
    // 1 red
    [0.95, 0.26, 0.21],
    // 2 green
    [0.62, 0.78, 0.34],
    // 3 yellow
    [0.95, 0.75, 0.28],
    // 4 blue
    [0.37, 0.57, 0.90],
    // 5 magenta
    [0.90, 0.39, 0.70],
    // 6 cyan
    [0.35, 0.82, 0.85],
    // 7 white
    [0.91, 0.93, 0.95],
    // 8 bright black
    [0.36, 0.39, 0.43],
    // 9 bright red
    [0.98, 0.47, 0.42],
    // 10 bright green
    [0.78, 0.90, 0.55],
    // 11 bright yellow
    [0.98, 0.86, 0.50],
    // 12 bright blue
    [0.56, 0.72, 0.98],
    // 13 bright magenta
    [0.98, 0.62, 0.82],
    // 14 bright cyan
    [0.55, 0.90, 0.93],
    // 15 bright white
    [1.0, 1.0, 1.0],
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

/// Terminal grid state updated by the vte parser (on the reader thread) and
/// read by the UI (on the main thread).
pub struct TerminalState {
    pub cols: usize,
    pub rows: usize,
    pub lines: Vec<Vec<Cell>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// Number of lines scrolled out of view (kept for scrollback).
    pub scrollback_lines: Vec<Vec<Cell>>,
    pub max_scrollback: usize,

    // transient rendering state
    cur_fg: [f32; 3],
    cur_bg: [f32; 3],
    cur_bold: bool,
    cursor_visible: bool,
}

impl TerminalState {
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut st = Self {
            cols,
            rows,
            lines: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            scrollback_lines: Vec::new(),
            max_scrollback: 2000,
            cur_fg: DEFAULT_FG,
            cur_bg: DEFAULT_BG,
            cur_bold: false,
            cursor_visible: true,
        };
        st.ensure_rows();
        st
    }

    fn ensure_rows(&mut self) {
        while self.lines.len() < self.rows {
            self.lines.push(vec![Cell::default(); self.cols]);
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        let new_cols = cols.max(1);
        let new_rows = rows.max(1);
        if new_cols == self.cols && new_rows == self.rows {
            return;
        }
        // Only drain top lines when the row count actually shrinks; a pure
        // column change must never drop history (spawn-vs-draw cols differ by
        // one cell and draining on that would throw away the prompt).
        if new_rows < self.rows && self.lines.len() > new_rows {
            let drop = self.lines.len() - new_rows;
            let mut dropped: Vec<Vec<Cell>> = self.lines.drain(..drop).collect();
            self.scrollback_lines.append(&mut dropped);
            let overflow = self
                .scrollback_lines
                .len()
                .saturating_sub(self.max_scrollback);
            if overflow > 0 {
                self.scrollback_lines.drain(..overflow);
            }
        }
        self.cols = new_cols;
        self.rows = new_rows;
        self.ensure_rows();
        for line in self.lines.iter_mut() {
            line.resize(self.cols, Cell::default());
        }
        self.cursor_row = self.cursor_row.min(self.rows - 1);
        self.cursor_col = self.cursor_col.min(self.cols - 1);
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        let mut parser = vte::Parser::new();
        parser.advance(self, bytes);
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Scroll up one line, pushing the top line into scrollback.
    fn scroll_up(&mut self) {
        if !self.lines.is_empty() {
            let line = self.lines.remove(0);
            if self.scrollback_lines.len() >= self.max_scrollback {
                self.scrollback_lines.remove(0);
            }
            self.scrollback_lines.push(line);
        }
        self.lines.push(vec![Cell::default(); self.cols]);
    }

    /// Scroll the viewport down by `n` lines (used for scrollback view).
    #[allow(dead_code)]
    pub fn scroll_viewport(&mut self, _delta: i32) {
        // Full scrollback navigation is a later phase; MVP keeps the tail.
    }
}

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        // Ignore control chars that might slip through.
        if c.is_control() && c != ' ' {
            return;
        }
        if self.cursor_row >= self.lines.len() {
            self.ensure_rows();
        }
        if self.cursor_col >= self.cols {
            // wrap to next line
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row >= self.rows {
                self.cursor_row = self.rows - 1;
                self.scroll_up();
            }
        }
        if self.cursor_col < self.cols {
            let (fg, bg, bold) = (self.cur_fg, self.cur_bg, self.cur_bold);
            let cell = &mut self.lines[self.cursor_row][self.cursor_col];
            cell.ch = c;
            cell.fg = fg;
            cell.bg = bg;
            cell.bold = bold;
            self.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                self.cursor_col = 0;
            }
            b'\n' | 0x0b | 0x0c => {
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.cursor_row = self.rows - 1;
                    self.scroll_up();
                }
            }
            0x08 => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            0x07 => {
                // bell: ignore for now
            }
            0x09 => {
                // tab: advance to next multiple of 8
                self.cursor_col = ((self.cursor_col / 8) + 1) * 8;
                if self.cursor_col >= self.cols {
                    self.cursor_col = self.cols - 1;
                }
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let mut iter = params.iter();
        let mut p0 = || iter.next().and_then(|p| p.first()).map(|v| *v as i32);
        match action {
            'A' => {
                // Cursor up
                let n = p0().unwrap_or(1).max(1);
                self.cursor_row = self.cursor_row.saturating_sub(n as usize);
            }
            'B' => {
                // Cursor down
                let n = p0().unwrap_or(1).max(1);
                self.cursor_row = (self.cursor_row + n as usize).min(self.rows - 1);
            }
            'C' => {
                // Cursor forward
                let n = p0().unwrap_or(1).max(1);
                self.cursor_col = (self.cursor_col + n as usize).min(self.cols - 1);
            }
            'D' => {
                // Cursor back
                let n = p0().unwrap_or(1).max(1);
                self.cursor_col = self.cursor_col.saturating_sub(n as usize);
            }
            'G' | '`' => {
                // Cursor horizontal position
                let n = p0().unwrap_or(1);
                self.cursor_col = ((n as usize).saturating_sub(1)).min(self.cols - 1);
            }
            'd' => {
                // Cursor vertical position
                let n = p0().unwrap_or(1);
                self.cursor_row = ((n as usize).saturating_sub(1)).min(self.rows - 1);
            }
            'H' | 'f' => {
                // Cursor position
                let row = p0().unwrap_or(1);
                let col = p0().unwrap_or(1);
                self.cursor_row = ((row as usize).saturating_sub(1)).min(self.rows - 1);
                self.cursor_col = ((col as usize).saturating_sub(1)).min(self.cols - 1);
            }
            'J' => {
                // Erase in display
                let mode = p0().unwrap_or(0);
                match mode {
                    0 => {
                        if let Some(line) = self.lines.get_mut(self.cursor_row) {
                            line[self.cursor_col..].fill(Cell::default());
                        }
                        for r in (self.cursor_row + 1)..self.rows {
                            if let Some(line) = self.lines.get_mut(r) {
                                for cell in line.iter_mut() {
                                    *cell = Cell::default();
                                }
                            }
                        }
                    }
                    1 =>
                    {
                        #[allow(clippy::needless_range_loop)]
                        for r in 0..=self.cursor_row {
                            if let Some(line) = self.lines.get_mut(r) {
                                #[allow(clippy::needless_range_loop)]
                                for c in 0..self.cols {
                                    if r == self.cursor_row && c > self.cursor_col {
                                        break;
                                    }
                                    line[c] = Cell::default();
                                }
                            }
                        }
                    }
                    2 | 3 => {
                        for line in self.lines.iter_mut() {
                            for cell in line.iter_mut() {
                                *cell = Cell::default();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in line
                let mode = p0().unwrap_or(0);
                if let Some(line) = self.lines.get_mut(self.cursor_row) {
                    match mode {
                        0 => {
                            line[self.cursor_col..].fill(Cell::default());
                        }
                        1 =>
                        {
                            #[allow(clippy::needless_range_loop)]
                            for c in 0..=self.cursor_col {
                                line[c] = Cell::default();
                            }
                        }
                        2 => {
                            for cell in line.iter_mut() {
                                *cell = Cell::default();
                            }
                        }
                        _ => {}
                    }
                }
            }
            'm' => {
                // SGR
                let values: Vec<u16> = params.iter().flat_map(|p| p.iter().copied()).collect();
                if values.is_empty() {
                    self.cur_fg = DEFAULT_FG;
                    self.cur_bg = DEFAULT_BG;
                    self.cur_bold = false;
                } else {
                    let mut i = 0;
                    while i < values.len() {
                        let v = values[i];
                        match v {
                            0 => {
                                self.cur_fg = DEFAULT_FG;
                                self.cur_bg = DEFAULT_BG;
                                self.cur_bold = false;
                            }
                            1 => self.cur_bold = true,
                            22 => self.cur_bold = false,
                            30..=37 => {
                                let idx = (v - 30) as usize;
                                self.cur_fg = ANSI_COLORS[idx];
                            }
                            38 => {
                                // 38;5;N or 38;2;r;g;b
                                if i + 1 < values.len()
                                    && values[i + 1] == 5
                                    && i + 2 < values.len()
                                {
                                    self.cur_fg = index_color(values[i + 2] as usize);
                                    i += 2;
                                } else if i + 1 < values.len()
                                    && values[i + 1] == 2
                                    && i + 4 < values.len()
                                {
                                    self.cur_fg = [
                                        values[i + 2] as f32 / 255.0,
                                        values[i + 3] as f32 / 255.0,
                                        values[i + 4] as f32 / 255.0,
                                    ];
                                    i += 4;
                                }
                            }
                            39 => self.cur_fg = DEFAULT_FG,
                            40..=47 => {
                                let idx = (v - 40) as usize;
                                self.cur_bg = ANSI_COLORS[idx];
                            }
                            48 => {
                                if i + 1 < values.len()
                                    && values[i + 1] == 5
                                    && i + 2 < values.len()
                                {
                                    self.cur_bg = index_color(values[i + 2] as usize);
                                    i += 2;
                                } else if i + 1 < values.len()
                                    && values[i + 1] == 2
                                    && i + 4 < values.len()
                                {
                                    self.cur_bg = [
                                        values[i + 2] as f32 / 255.0,
                                        values[i + 3] as f32 / 255.0,
                                        values[i + 4] as f32 / 255.0,
                                    ];
                                    i += 4;
                                }
                            }
                            49 => self.cur_bg = DEFAULT_BG,
                            90..=97 => {
                                let idx = (v - 90 + 8) as usize;
                                self.cur_fg = ANSI_COLORS[idx];
                            }
                            100..=107 => {
                                let idx = (v - 100 + 8) as usize;
                                self.cur_bg = ANSI_COLORS[idx];
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                }
            }
            'h' | 'l' => {
                // DECSET/DECRST: only support cursor visibility for now.
                let mode = p0().unwrap_or(0);
                if mode == 25 {
                    self.cursor_visible = action == 'h';
                }
            }
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
}

/// Map a 256-color index to RGB (16-color cube fallback).
fn index_color(idx: usize) -> [f32; 3] {
    if idx < 16 {
        return ANSI_COLORS[idx];
    }
    // 16-231: 6x6x6 color cube
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
    // 232-255: grayscale ramp
    let g = 8 + (idx - 232) * 10;
    [g as f32 / 255.0, g as f32 / 255.0, g as f32 / 255.0]
}
