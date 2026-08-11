use vte::Perform;

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
    /// Cell is the trailing padding of a wide (CJK/emoji) glyph.
    pub wide_padding: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            bold: false,
            wide_padding: false,
        }
    }
}

/// Local terminal state maintained by the vte parser. This is the
/// authoritative grid (like alacritty's): PTY bytes → vte → grid +
/// scrollback + colors + cursor. makepad renders this grid.
pub struct TerminalState {
    pub cols: usize,
    pub rows: usize,
    pub lines: Vec<Vec<Cell>>,
    /// Scrollback history (lines scrolled out of the viewport).
    pub scrollback: Vec<Vec<Cell>>,
    pub max_scrollback: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub cursor_visible: bool,
    pub cursor_style: u8,
    /// Text selection in cell coords (relative to display offset).
    pub selection: Option<(usize, usize, usize, usize)>,
    /// How many scrollback lines are displayed above the viewport.
    pub scroll_offset: usize,

    // SGR state
    cur_fg: [f32; 3],
    cur_bg: [f32; 3],
    cur_bold: bool,
}

impl TerminalState {
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut st = Self {
            cols,
            rows,
            lines: Vec::new(),
            scrollback: Vec::new(),
            max_scrollback: 2000,
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            cursor_style: 0,
            selection: None,
            scroll_offset: 0,
            cur_fg: DEFAULT_FG,
            cur_bg: DEFAULT_BG,
            cur_bold: false,
        };
        st.ensure_rows();
        st
    }

    fn ensure_rows(&mut self) {
        while self.lines.len() < self.rows {
            self.lines.push(vec![Cell::default(); self.cols]);
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        let mut parser = vte::Parser::new();
        parser.advance(self, bytes);
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Scroll the viewport by `delta` scrollback lines (positive = up).
    pub fn scroll_display(&mut self, delta: i32) {
        let max = self.scrollback.len() as i32;
        let new = (self.scroll_offset as i32 + delta).clamp(0, max);
        self.scroll_offset = new as usize;
    }

    #[allow(dead_code)]
    pub fn reset_display(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll up one line, pushing the top line into scrollback.
    fn scroll_up(&mut self) {
        if !self.lines.is_empty() {
            let line = self.lines.remove(0);
            if self.scrollback.len() >= self.max_scrollback {
                self.scrollback.remove(0);
            }
            self.scrollback.push(line);
        }
        self.lines.push(vec![Cell::default(); self.cols]);
    }

    /// Resize the grid, preserving content where possible.
    #[allow(dead_code)]
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let new_cols = cols.max(1);
        let new_rows = rows.max(1);
        if new_cols == self.cols && new_rows == self.rows {
            return;
        }
        // Shrink rows: push dropped top lines into scrollback.
        if new_rows < self.rows && self.lines.len() > new_rows {
            let drop = self.lines.len() - new_rows;
            let mut dropped: Vec<Vec<Cell>> = self.lines.drain(..drop).collect();
            self.scrollback.append(&mut dropped);
            let overflow = self.scrollback.len().saturating_sub(self.max_scrollback);
            if overflow > 0 {
                self.scrollback.drain(..overflow);
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
        // Clamp scroll offset to available history.
        self.scroll_offset = self.scroll_offset.min(self.scrollback.len());
    }

    /// Extract selected text (display coordinates), trimming trailing spaces
    /// per line and joining rows with newlines.
    pub fn selected_text(&self) -> String {
        let Some((r0, c0, r1, c1)) = self.selection else {
            return String::new();
        };
        let (ra, rb) = (r0.min(r1), r0.max(r1));
        let (ca, cb) = (c0.min(c1), c0.max(c1));
        let mut out = String::new();
        for r in ra..=rb {
            if r > ra {
                out.push('\n');
            }
            let display_row = r + self.scroll_offset;
            let Some(line) = self.lines.get(display_row) else {
                continue;
            };
            let start = if r == ra { ca } else { 0 };
            let end = if r == rb { cb } else { line.len().saturating_sub(1) };
            for cell in line.iter().take(end + 1).skip(start) {
                if cell.ch != '\0' {
                    out.push(cell.ch);
                }
            }
            while out.ends_with(' ') {
                out.pop();
            }
        }
        out
    }
}

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        if c.is_control() && c != ' ' {
            return;
        }
        if self.cursor_row >= self.lines.len() {
            self.ensure_rows();
        }
        // Wide character handling: CJK/emoji occupy two cells.
        let width = unicode_width(c);
        if width == 2 {
            // Ensure room for two cells.
            if self.cursor_col + 1 >= self.cols {
                self.cursor_col = 0;
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.cursor_row = self.rows - 1;
                    self.scroll_up();
                }
            }
            if self.cursor_col + 1 < self.cols {
                let (fg, bg, bold) = (self.cur_fg, self.cur_bg, self.cur_bold);
                {
                    let cell = &mut self.lines[self.cursor_row][self.cursor_col];
                    cell.ch = c;
                    cell.fg = fg;
                    cell.bg = bg;
                    cell.bold = bold;
                    cell.wide_padding = false;
                }
                {
                    let cell = &mut self.lines[self.cursor_row][self.cursor_col + 1];
                    cell.ch = ' ';
                    cell.fg = fg;
                    cell.bg = bg;
                    cell.bold = bold;
                    cell.wide_padding = true;
                }
                self.cursor_col += 2;
            }
            return;
        }
        if self.cursor_col >= self.cols {
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
            cell.wide_padding = false;
            self.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => self.cursor_col = 0,
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
            0x09 => {
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
                let n = p0().unwrap_or(1).max(1);
                self.cursor_row = self.cursor_row.saturating_sub(n as usize);
            }
            'B' => {
                let n = p0().unwrap_or(1).max(1);
                self.cursor_row = (self.cursor_row + n as usize).min(self.rows - 1);
            }
            'C' => {
                let n = p0().unwrap_or(1).max(1);
                self.cursor_col = (self.cursor_col + n as usize).min(self.cols - 1);
            }
            'D' => {
                let n = p0().unwrap_or(1).max(1);
                self.cursor_col = self.cursor_col.saturating_sub(n as usize);
            }
            'G' | '`' => {
                let n = p0().unwrap_or(1);
                self.cursor_col = ((n as usize).saturating_sub(1)).min(self.cols - 1);
            }
            'd' => {
                let n = p0().unwrap_or(1);
                self.cursor_row = ((n as usize).saturating_sub(1)).min(self.rows - 1);
            }
            'H' | 'f' => {
                let row = p0().unwrap_or(1);
                let col = p0().unwrap_or(1);
                self.cursor_row = ((row as usize).saturating_sub(1)).min(self.rows - 1);
                self.cursor_col = ((col as usize).saturating_sub(1)).min(self.cols - 1);
            }
            'J' => {
                let mode = p0().unwrap_or(0);
                match mode {
                    0 => {
                        if let Some(line) = self.lines.get_mut(self.cursor_row) {
                            line[self.cursor_col..].fill(Cell::default());
                        }
                        for r in (self.cursor_row + 1)..self.rows {
                            if let Some(line) = self.lines.get_mut(r) {
                                line.fill(Cell::default());
                            }
                        }
                    }
                    1 => {
                        for r in 0..=self.cursor_row {
                            if let Some(line) = self.lines.get_mut(r) {
                                let end = if r == self.cursor_row {
                                    self.cursor_col + 1
                                } else {
                                    self.cols
                                };
                                line[..end.min(self.cols)].fill(Cell::default());
                            }
                        }
                    }
                    2 | 3 => {
                        for line in self.lines.iter_mut() {
                            line.fill(Cell::default());
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                let mode = p0().unwrap_or(0);
                if let Some(line) = self.lines.get_mut(self.cursor_row) {
                    match mode {
                        0 => line[self.cursor_col..].fill(Cell::default()),
                        1 => line[..=self.cursor_col.min(self.cols - 1)].fill(Cell::default()),
                        2 => line.fill(Cell::default()),
                        _ => {}
                    }
                }
            }
            'm' => {
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
                                self.cur_fg = ANSI_COLORS[(v - 30) as usize];
                            }
                            38 => {
                                if i + 1 < values.len() && values[i + 1] == 5 && i + 2 < values.len()
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
                                self.cur_bg = ANSI_COLORS[(v - 40) as usize];
                            }
                            48 => {
                                if i + 1 < values.len() && values[i + 1] == 5 && i + 2 < values.len()
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
                                self.cur_fg = ANSI_COLORS[(v - 90 + 8) as usize];
                            }
                            100..=107 => {
                                self.cur_bg = ANSI_COLORS[(v - 100 + 8) as usize];
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                }
            }
            'h' | 'l' => {
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

/// Unicode display width (1 or 2). CJK/emoji are wide.
fn unicode_width(c: char) -> u8 {
    if c as u32 >= 0x1100
        && (c as u32 <= 0x115f
            || c as u32 == 0x2329
            || c as u32 == 0x232a
            || (c as u32 >= 0x2e80 && c as u32 <= 0xa4cf && c as u32 != 0x303f)
            || (c as u32 >= 0xac00 && c as u32 <= 0xd7a3)
            || (c as u32 >= 0xf900 && c as u32 <= 0xfaff)
            || (c as u32 >= 0xfe10 && c as u32 <= 0xfe19)
            || (c as u32 >= 0xfe30 && c as u32 <= 0xfe6f)
            || (c as u32 >= 0xff00 && c as u32 <= 0xff60)
            || (c as u32 >= 0xffe0 && c as u32 <= 0xffe6)
            || (c as u32 >= 0x1f300 && c as u32 <= 0x1faff)
            || (c as u32 >= 0x20000 && c as u32 <= 0x2fffd))
    {
        2
    } else {
        1
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
