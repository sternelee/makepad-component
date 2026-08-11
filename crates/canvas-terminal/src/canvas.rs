use makepad_widgets::*;

use crate::camera::Camera;
use crate::command::{self, Command};
use crate::items::{CanvasItem, ItemKind};
use crate::terminal::state::DEFAULT_BG;

/// Grid spacing in world units.
const GRID_SIZE: f64 = 24.0;
/// Terminal font metrics.
const TERM_CELL_W: f64 = 7.6;
const TERM_CELL_H: f64 = 16.0;

const NOTE_COLOR: [f32; 4] = [0.20, 0.24, 0.34, 1.0];
const NOTE_BORDER: [f32; 4] = [0.36, 0.43, 0.60, 1.0];
const TERM_BG: [f32; 4] = [0.075, 0.082, 0.10, 1.0];
const TERM_BORDER: [f32; 4] = [0.20, 0.24, 0.32, 1.0];
const SEL_BORDER: [f32; 4] = [0.30, 0.62, 0.98, 1.0];
const TITLE_TEXT: [f32; 4] = [0.85, 0.88, 0.94, 1.0];

/// Title-bar control buttons (minimize / close).
const BTN_W: f64 = 22.0;
const BTN_H: f64 = 18.0;
const BTN_BG: [f32; 4] = [0.16, 0.19, 0.26, 1.0];
const BTN_BORDER: [f32; 4] = [0.28, 0.32, 0.42, 1.0];
const BTN_HOVER: [f32; 4] = [0.22, 0.28, 0.40, 1.0];
const BTN_CLOSE_HOVER: [f32; 4] = [0.65, 0.25, 0.25, 1.0];

/// Bottom dock (screen-fixed minimize tray).
const DOCK_H: f64 = 34.0;
const DOCK_BOTTOM: f64 = 76.0;
const DOCK_BG: [f32; 4] = [0.10, 0.11, 0.15, 0.92];
const CHIP_BG: [f32; 4] = [0.16, 0.18, 0.24, 1.0];
const CHIP_BG_HOVER: [f32; 4] = [0.22, 0.26, 0.34, 1.0];
const CHIP_BORDER: [f32; 4] = [0.28, 0.32, 0.42, 1.0];

/// Drag state while moving or resizing an item.
struct DragState {
    item_id: u64,
    /// World position of the cursor at drag start.
    grab_world: Vec2d,
    item_origin_world: Vec2d,
    /// Starting world size (for resize mode).
    item_origin_size: Vec2d,
    mode: DragMode,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DragMode {
    Move,
    Resize,
}

/// Title-bar control button kind.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BtnKind {
    Minimize,
    Close,
}

#[derive(Script, ScriptHook, Widget)]
pub struct CanvasPanel {
    #[deref]
    view: View,
    #[rust]
    area: Area,
    #[rust]
    viewport: Vec2d,
    #[rust]
    camera: Camera,
    #[rust]
    items: Vec<CanvasItem>,
    #[rust]
    next_item_id: u64,
    #[rust]
    selected: Option<u64>,
    #[rust]
    drag: Option<DragState>,
    #[rust]
    panning: bool,
    #[rust]
    last_mouse: Vec2d,
    #[rust]
    hovered: Option<u64>,
    #[rust]
    focused_terminal: Option<u64>,
    /// Set when the reader thread reported new output for a session.
    #[rust]
    redraw_pending: bool,
    #[rust]
    timer: Option<Timer>,
    #[live]
    draw_grid: DrawQuad,
    #[live]
    draw_item_bg: DrawColor,
    #[live]
    draw_title: DrawText,
    #[live]
    draw_cell_bg: DrawColor,
    #[live]
    draw_cell_text: DrawText,
    #[live]
    draw_cursor: DrawColor,
    /// Browser page area (used to anchor the system WebView window).
    #[live]
    draw_browser_page: DrawColor,
    #[rust]
    browser_spawned: Vec<u64>,
    /// item_id → browser slot index (0..=3) for CEF embedded browsers.
    #[rust]
    browser_slots: Vec<(u64, usize)>,
}

impl CanvasPanel {
    fn world_viewport(&self) -> Vec2d {
        self.viewport
    }

    fn item_screen_rect(&self, item: &CanvasItem) -> Rect {
        self.camera
            .world_rect_to_screen(item.world(), self.world_viewport())
    }

    /// Spawn a note item at the current view center (world coords).
    fn spawn_note(&mut self, cx: &mut Cx) {
        let world_pos = self
            .camera
            .screen_to_world(self.viewport * 0.5, self.world_viewport());
        let item = CanvasItem::Note {
            id: self.next_item_id,
            world: Rect {
                pos: world_pos,
                size: Vec2d { x: 240.0, y: 120.0 },
            },
            title: format!("Note {}", self.next_item_id),
        };
        self.next_item_id += 1;
        self.items.push(item);
        self.redraw(cx);
    }

    /// Spawn a terminal item running `command` (default shell).
    pub fn spawn_terminal(&mut self, cx: &mut Cx, name: &str, command: &str) {
        let world_pos = self
            .camera
            .screen_to_world(self.viewport * 0.5, self.world_viewport());
        let id = self.next_item_id;
        self.next_item_id += 1;

        let item = CanvasItem::Terminal {
            id,
            world: Rect {
                pos: world_pos,
                size: Vec2d { x: 620.0, y: 380.0 },
            },
            title: name.to_string(),
            session: None,
        };
        let (cols, rows) = self.term_grid_size(&item);
        match crate::terminal::TerminalSession::spawn(name, command, cols, rows) {
            Ok(session) => {
                let mut item = item;
                // Store the session into the Terminal variant's slot.
                match &mut item {
                    CanvasItem::Terminal { session: slot, .. } => *slot = Some(Box::new(session)),
                    CanvasItem::Note { .. } | CanvasItem::Browser { .. } => unreachable!(),
                }
                self.items.push(item);
                self.selected = Some(id);
                self.focused_terminal = Some(id);
                self.set_canvas_focus(cx);
                self.redraw(cx);
            }
            Err(e) => {
                log!("canvas: failed to spawn terminal '{name}': {e}");
                self.status(cx, &format!("Failed to spawn {name}: {e}"));
            }
        }
    }

    /// Spawn a browser item showing `url` (via the OS WebView system browser).
    pub fn spawn_browser(&mut self, cx: &mut Cx, url: &str) {
        let world_pos = self
            .camera
            .screen_to_world(self.viewport * 0.5, self.world_viewport());
        let id = self.next_item_id;
        self.next_item_id += 1;

        let item = CanvasItem::Browser {
            id,
            world: Rect {
                pos: world_pos,
                size: Vec2d { x: 720.0, y: 480.0 },
            },
            title: "browser".to_string(),
            url: url.to_string(),
        };
        self.items.push(item);
        self.selected = Some(id);
        self.redraw(cx);
    }

    fn term_grid_size_from(&self, w: f64, h: f64) -> (usize, usize) {
        let cols = ((w - 12.0) / TERM_CELL_W).floor().max(10.0) as usize;
        let rows = ((h - 34.0) / TERM_CELL_H).floor().max(3.0) as usize;
        (cols, rows)
    }

    fn term_grid_size(&self, item: &CanvasItem) -> (usize, usize) {
        let w = item.world().size.x;
        let h = item.world().size.y;
        self.term_grid_size_from(w, h)
    }

    fn status(&mut self, cx: &mut Cx, text: &str) {
        let l = self.view.label(cx, ids!(status_label));
        l.set_text(cx, text);
    }

    fn set_canvas_focus(&mut self, cx: &mut Cx) {
        // Route keyboard to the canvas so terminal keys are captured.
        if self.area.is_valid(cx) {
            cx.set_key_focus(self.area);
        }
    }

    fn focus_terminal(&mut self, cx: &mut Cx, id: Option<u64>) {
        self.focused_terminal = id;
        self.selected = id;
        if id.is_some() {
            self.set_canvas_focus(cx);
        } else {
            // Return focus to the command bar.
            let ti = self.view.text_input(cx, ids!(command_input));
            ti.set_key_focus(cx);
        }
        self.redraw(cx);
    }

    /// Find a terminal by (case-insensitive) name.
    fn find_terminal(&self, name: &str) -> Option<&CanvasItem> {
        self.items.iter().find(|i| {
            i.kind() == ItemKind::Terminal && i.title().to_lowercase() == name.to_lowercase()
        })
    }

    fn hit_test(&self, screen: Vec2d) -> Option<u64> {
        // Topmost = last in list.
        self.items
            .iter()
            .rev()
            .find(|i| self.item_screen_rect(i).contains(screen))
            .map(|i| i.id())
    }

    /// Topmost item whose bottom-right resize handle is under `screen`.
    fn resize_handle_under(&self, screen: Vec2d) -> Option<u64> {
        const HANDLE: f64 = 18.0;
        self.items
            .iter()
            .rev()
            .find(|i| {
                let r = self.item_screen_rect(i);
                let handle = Rect {
                    pos: Vec2d {
                        x: r.pos.x + r.size.x - HANDLE,
                        y: r.pos.y + r.size.y - HANDLE,
                    },
                    size: Vec2d {
                        x: HANDLE,
                        y: HANDLE,
                    },
                };
                handle.contains(screen)
            })
            .map(|i| i.id())
    }

    pub fn exec_command(&mut self, cx: &mut Cx, line: &str) {
        match command::parse(line) {
            Command::Send { target, text } => {
                let target_id = self
                    .items
                    .iter()
                    .find(|i| {
                        i.kind() == ItemKind::Terminal
                            && i.title().to_lowercase() == target.to_lowercase()
                    })
                    .map(|i| i.id());
                if let Some(id) = target_id {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id() == id) {
                        if let Some(session) = item.session() {
                            if text.is_empty() {
                                session.write_bytes(b"\r");
                            } else {
                                session.write_line(&text);
                            }
                        }
                    }
                    self.focus_terminal(cx, Some(id));
                } else {
                    self.status(cx, &format!("No terminal named '@{target}'"));
                }
            }
            Command::NewNote => {
                self.spawn_note(cx);
                self.status(cx, "Created note");
            }
            Command::NewTerminal { name } => {
                // Default shell command for a new terminal.
                let cmd = std::env::var("SHELL").unwrap_or_else(|_| "zsh".to_string());
                self.spawn_terminal(cx, &name, &cmd);
            }
            Command::NewBrowser { url } => {
                self.spawn_browser(cx, &url);
                self.status(cx, &format!("Browser: {url}"));
            }
            Command::Focus { name } => {
                if let Some(item) = self.find_terminal(&name) {
                    self.focus_terminal(cx, Some(item.id()));
                } else {
                    self.status(cx, &format!("No terminal named '{name}'"));
                }
            }
            Command::Zoom { factor } => {
                self.camera
                    .zoom_at(factor, self.viewport * 0.5, self.world_viewport());
                self.redraw(cx);
            }
            Command::Help => {
                self.status(
                    cx,
                    "Commands: @name text · /new note · /new terminal NAME · /focus NAME · /zoom N · /help",
                );
            }
            Command::Forward { text } => {
                if let Some(id) = self.focused_terminal {
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        if let Some(session) = item.session() {
                            session.write_line(&text);
                            return;
                        }
                    }
                }
                self.status(
                    cx,
                    "No focused terminal — use @name text or click a terminal",
                );
            }
        }
    }

    /// Translate a KeyEvent to PTY bytes.
    ///
    /// NOTE: `KeyCode` variants are ordered by QWERTY layout position, NOT
    /// alphabetically, so subtracting discriminants is wrong. Map each key
    /// to its character explicitly.
    fn key_to_bytes(&self, key: &KeyEvent) -> Option<Vec<u8>> {
        use KeyCode::*;
        let mut out = Vec::new();
        let ctrl = key.modifiers.control;
        let alt = key.modifiers.alt;
        let shift = key.modifiers.shift;
        match key.key_code {
            ReturnKey => out.extend_from_slice(b"\r"),
            Tab => out.extend_from_slice(if shift { b"\x1b[Z" } else { b"\t" }),
            Backspace => out.extend_from_slice(if ctrl { b"\x08" } else { b"\x7f" }),
            Delete => out.extend_from_slice(b"\x1b[3~"),
            ArrowUp => out.extend_from_slice(if ctrl { b"\x1b[1;5A" } else { b"\x1b[A" }),
            ArrowDown => out.extend_from_slice(if ctrl { b"\x1b[1;5B" } else { b"\x1b[B" }),
            ArrowRight => out.extend_from_slice(if ctrl { b"\x1b[1;5C" } else { b"\x1b[C" }),
            ArrowLeft => out.extend_from_slice(if ctrl { b"\x1b[1;5D" } else { b"\x1b[D" }),
            Home => out.extend_from_slice(b"\x1b[H"),
            End => out.extend_from_slice(b"\x1b[F"),
            PageUp => out.extend_from_slice(b"\x1b[5~"),
            PageDown => out.extend_from_slice(b"\x1b[6~"),
            Insert => out.extend_from_slice(b"\x1b[2~"),
            Escape => out.extend_from_slice(b"\x1b"),
            Space => out.extend_from_slice(b" "),
            Key0 | Key1 | Key2 | Key3 | Key4 | Key5 | Key6 | Key7 | Key8 | Key9 => {
                let n = key.key_code as u8 - KeyCode::Key0 as u8;
                push_modified_char(&mut out, (b'0' + n) as char, ctrl, alt, shift);
            }
            KeyA => push_modified_char(&mut out, 'a', ctrl, alt, shift),
            KeyB => push_modified_char(&mut out, 'b', ctrl, alt, shift),
            KeyC => push_modified_char(&mut out, 'c', ctrl, alt, shift),
            KeyD => push_modified_char(&mut out, 'd', ctrl, alt, shift),
            KeyE => push_modified_char(&mut out, 'e', ctrl, alt, shift),
            KeyF => push_modified_char(&mut out, 'f', ctrl, alt, shift),
            KeyG => push_modified_char(&mut out, 'g', ctrl, alt, shift),
            KeyH => push_modified_char(&mut out, 'h', ctrl, alt, shift),
            KeyI => push_modified_char(&mut out, 'i', ctrl, alt, shift),
            KeyJ => push_modified_char(&mut out, 'j', ctrl, alt, shift),
            KeyK => push_modified_char(&mut out, 'k', ctrl, alt, shift),
            KeyL => push_modified_char(&mut out, 'l', ctrl, alt, shift),
            KeyM => push_modified_char(&mut out, 'm', ctrl, alt, shift),
            KeyN => push_modified_char(&mut out, 'n', ctrl, alt, shift),
            KeyO => push_modified_char(&mut out, 'o', ctrl, alt, shift),
            KeyP => push_modified_char(&mut out, 'p', ctrl, alt, shift),
            KeyQ => push_modified_char(&mut out, 'q', ctrl, alt, shift),
            KeyR => push_modified_char(&mut out, 'r', ctrl, alt, shift),
            KeyS => push_modified_char(&mut out, 's', ctrl, alt, shift),
            KeyT => push_modified_char(&mut out, 't', ctrl, alt, shift),
            KeyU => push_modified_char(&mut out, 'u', ctrl, alt, shift),
            KeyV => push_modified_char(&mut out, 'v', ctrl, alt, shift),
            KeyW => push_modified_char(&mut out, 'w', ctrl, alt, shift),
            KeyX => push_modified_char(&mut out, 'x', ctrl, alt, shift),
            KeyY => push_modified_char(&mut out, 'y', ctrl, alt, shift),
            KeyZ => push_modified_char(&mut out, 'z', ctrl, alt, shift),
            Minus => push_modified_char(&mut out, '-', ctrl, alt, false),
            Equals => push_modified_char(&mut out, '=', ctrl, alt, false),
            LBracket => push_modified_char(&mut out, '[', ctrl, alt, false),
            RBracket => push_modified_char(&mut out, ']', ctrl, alt, false),
            Semicolon => push_modified_char(&mut out, ';', ctrl, alt, shift),
            Quote => push_modified_char(&mut out, '\'', ctrl, alt, shift),
            Backslash => push_modified_char(&mut out, '\\', ctrl, alt, shift),
            Comma => push_modified_char(&mut out, ',', ctrl, alt, shift),
            Period => push_modified_char(&mut out, '.', ctrl, alt, shift),
            Slash => push_modified_char(&mut out, '/', ctrl, alt, shift),
            Backtick => push_modified_char(&mut out, '`', ctrl, alt, shift),
            _ => return None,
        }
        Some(out)
    }

    /// Poll all terminal sessions; returns true if anything changed.
    fn poll_sessions(&mut self) -> bool {
        let mut changed = false;
        for item in self.items.iter() {
            if let Some(session) = item.session() {
                if session.poll() {
                    changed = true;
                }
            }
        }
        changed
    }

    // ── drawing helpers ──────────────────────────────────────────────────

    fn draw_grid(&mut self, cx: &mut Cx2d, rect: Rect) {
        self.draw_grid.draw_vars.set_uniform(
            cx.cx,
            live_id!(cam_pan),
            &[self.camera.pan.x as f32, self.camera.pan.y as f32],
        );
        self.draw_grid
            .draw_vars
            .set_uniform(cx.cx, live_id!(cam_zoom), &[self.camera.zoom]);
        self.draw_grid
            .draw_vars
            .set_uniform(cx.cx, live_id!(grid_size), &[GRID_SIZE as f32]);
        self.draw_grid.draw_abs(cx, rect);
    }

    fn draw_item_bg_rect(&mut self, cx: &mut Cx2d, rect: Rect, color: [f32; 4]) {
        self.draw_item_bg.color = Vec4f {
            x: color[0],
            y: color[1],
            z: color[2],
            w: color[3],
        };
        self.draw_item_bg.draw_abs(cx, rect);
    }

    /// Draw a 4-edge border using plain DrawColor quads (avoids the custom
    /// DrawQuad pixel-fn shader, whose draw corrupts subsequent DrawText
    /// rendering in the same frame).
    fn draw_border_rect(&mut self, cx: &mut Cx2d, rect: Rect, color: [f32; 4]) {
        const T: f64 = 1.5;
        // top
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: rect.pos,
                size: Vec2d {
                    x: rect.size.x,
                    y: T,
                },
            },
            color,
        );
        // bottom
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: rect.pos.x,
                    y: rect.pos.y + rect.size.y - T,
                },
                size: Vec2d {
                    x: rect.size.x,
                    y: T,
                },
            },
            color,
        );
        // left
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: rect.pos,
                size: Vec2d {
                    x: T,
                    y: rect.size.y,
                },
            },
            color,
        );
        // right
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: rect.pos.x + rect.size.x - T,
                    y: rect.pos.y,
                },
                size: Vec2d {
                    x: T,
                    y: rect.size.y,
                },
            },
            color,
        );
    }

    fn draw_note_title(&mut self, cx: &mut Cx2d, title: &str, screen: Rect) {
        self.draw_item_bg_rect(cx, screen, NOTE_COLOR);
        self.draw_title
            .draw_vars
            .set_uniform(cx.cx, live_id!(color), &TITLE_TEXT);
        self.draw_title
            .draw_abs(cx, screen.pos + Vec2d { x: 10.0, y: 8.0 }, title);
    }

    /// Draw the bottom-right resize handle of a selected item.
    fn draw_resize_handle(&mut self, cx: &mut Cx2d, screen: Rect) {
        const HANDLE: f64 = 18.0;
        let handle_rect = Rect {
            pos: Vec2d {
                x: screen.pos.x + screen.size.x - HANDLE,
                y: screen.pos.y + screen.size.y - HANDLE,
            },
            size: Vec2d {
                x: HANDLE,
                y: HANDLE,
            },
        };
        // Handle background.
        self.draw_item_bg_rect(cx, handle_rect, [0.16, 0.20, 0.30, 1.0]);
        // Handle border.
        self.draw_border_rect(cx, handle_rect, SEL_BORDER);
        // Diagonal grip line.
        self.draw_cursor.color = Vec4f {
            x: 0.62,
            y: 0.78,
            z: 0.98,
            w: 1.0,
        };
        let grip = Rect {
            pos: handle_rect.pos + Vec2d { x: 5.0, y: 5.0 },
            size: Vec2d { x: 8.0, y: 2.0 },
        };
        self.draw_cursor.draw_abs(cx, grip);
        let grip2 = Rect {
            pos: handle_rect.pos + Vec2d { x: 9.0, y: 11.0 },
            size: Vec2d { x: 4.0, y: 2.0 },
        };
        self.draw_cursor.draw_abs(cx, grip2);
    }

    fn draw_terminal_at(
        &mut self,
        cx: &mut Cx2d,
        screen: Rect,
        title: &str,
        command: &str,
        state: &std::sync::Arc<std::sync::Mutex<crate::terminal::state::TerminalState>>,
    ) {
        // Background first (DrawColor)
        self.draw_item_bg_rect(cx, screen, TERM_BG);
        // Border (DrawColor)
        self.draw_border_rect(cx, screen, TERM_BORDER);
        // Title text (DrawText) - drawn after bg/border but before content
        self.draw_title
            .draw_vars
            .set_uniform(cx.cx, live_id!(color), &TITLE_TEXT);
        self.draw_title.draw_abs(
            cx,
            screen.pos + Vec2d { x: 8.0, y: 6.0 },
            &format!("{} — {}", title, command),
        );

        let grid = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        // Content area (below the title bar, inside the item border).
        let origin = screen.pos + Vec2d { x: 6.0, y: 30.0 };
        let content_w = (screen.size.x - 12.0).max(1.0);
        let content_h = (screen.size.y - 34.0).max(1.0);

        // Cell size tiles the content rect exactly (grid dims include zoom).
        let cols = grid.cols.max(1);
        let rows = grid.rows.max(1);
        let char_w = content_w / cols as f64;
        let line_h = content_h / rows as f64;

        // Tail view: show the LAST `rows` lines (the on-screen viewport).
        let total = grid.lines.len();
        let start = total.saturating_sub(rows);
        let draw_rows = (total - start).min(rows);

        // Clip glyph/background drawing to the content rect so no text can
        // overflow outside the terminal item (e.g. long unwrapped lines).
        let content_rect = Rect {
            pos: origin,
            size: Vec2d {
                x: content_w,
                y: content_h,
            },
        };
        cx.push_clip_rect(content_rect);

        // Draw rows: background cells then text runs.
        for r in 0..draw_rows {
            let row = &grid.lines[start + r];
            let y = origin.y + r as f64 * line_h;

            let mut i = 0;
            while i < row.len() && i < cols {
                let cell = &row[i];
                let fg = cell.fg;
                let bg = cell.bg;
                let bold = cell.bold;
                let mut j = i;
                while j < row.len() && j < cols {
                    let c = &row[j];
                    if c.fg != fg || c.bg != bg || c.bold != bold {
                        break;
                    }
                    j += 1;
                }
                let mut text = String::new();
                for cell in row.iter().take(j).skip(i) {
                    text.push(cell.ch);
                }
                let x = origin.x + i as f64 * char_w;
                let run_rect = Rect {
                    pos: Vec2d { x, y },
                    size: Vec2d {
                        x: (j - i) as f64 * char_w,
                        y: line_h,
                    },
                };
                if bg != DEFAULT_BG {
                    self.draw_cell_bg.color = Vec4f {
                        x: bg[0],
                        y: bg[1],
                        z: bg[2],
                        w: 1.0,
                    };
                    self.draw_cell_bg.draw_abs(cx, run_rect);
                }
                self.draw_cell_text.draw_vars.set_uniform(
                    cx.cx,
                    live_id!(color),
                    &[fg[0], fg[1], fg[2], 1.0],
                );
                self.draw_cell_text.draw_abs(cx, Vec2d { x, y }, &text);
                i = j;
            }
        }

        // Cursor
        if grid.cursor_visible() {
            let c = grid.cursor_col.min(cols - 1);
            let r = grid
                .cursor_row
                .saturating_sub(start)
                .min(draw_rows.saturating_sub(1));
            let pos = origin
                + Vec2d {
                    x: c as f64 * char_w,
                    y: r as f64 * line_h,
                };
            self.draw_cursor.color = Vec4f {
                x: 0.30,
                y: 0.62,
                z: 0.98,
                w: 1.0,
            };
            self.draw_cursor.draw_abs(
                cx,
                Rect {
                    pos,
                    size: Vec2d {
                        x: char_w,
                        y: line_h,
                    },
                },
            );
        }

        cx.pop_clip_rect();
    }

    /// Draw a browser card and drive the embedded CEF browser slot.
    ///
    /// The browser item is rendered by one of four CEF `Browser` slots
    /// (`browser_slot_0..3`) declared in the DSL. Each slot is hidden from
    /// the overlay flow (`visible: false`); we toggle it visible, draw it
    /// with an abs_pos walk at the item's page rect, then hide it again so
    /// the regular `view.draw_walk` pass never lays it out.
    fn draw_browser_at(&mut self, cx: &mut Cx2d, id: u64, screen: Rect, is_sel: bool, url: &str) {
        // Card background + border + title chrome.
        self.draw_item_bg_rect(cx, screen, TERM_BG);
        self.draw_border_rect(cx, screen, if is_sel { SEL_BORDER } else { TERM_BORDER });

        let bar_rect = Rect {
            pos: screen.pos,
            size: Vec2d {
                x: screen.size.x,
                y: 30.0,
            },
        };
        self.draw_item_bg_rect(cx, bar_rect, [0.14, 0.16, 0.22, 1.0]);
        self.draw_title
            .draw_vars
            .set_uniform(cx.cx, live_id!(color), &TITLE_TEXT);
        self.draw_title.draw_abs(
            cx,
            bar_rect.pos + Vec2d { x: 10.0, y: 7.0 },
            &format!("🌐 {url}"),
        );

        // Page area below the title bar.
        let page_rect = Rect {
            pos: screen.pos + Vec2d { x: 2.0, y: 32.0 },
            size: screen.size - Vec2d { x: 4.0, y: 34.0 },
        };
        if page_rect.size.x <= 4.0 || page_rect.size.y <= 4.0 {
            return;
        }

        // Map this item to a CEF slot (reuse the first free one).
        let slot = match self.browser_slots.iter().find(|(iid, _)| *iid == id) {
            Some((_, s)) => *s,
            None => {
                let free = (0..4)
                    .find(|s| !self.browser_slots.iter().any(|(_, used)| used == s))
                    .unwrap_or(0);
                self.browser_slots.push((id, free));
                free
            }
        };

        // Drive the slot: set url once, make it visible, draw it at the
        // item rect, then hide it again for the overlay flow pass.
        let slot_path = LiveId::from_str_num("browser_slot_", slot as u64);
        let slot_widget = self.view.widget(cx.cx, &[slot_path]);
        let slot_browser = slot_widget.as_browser();
        if !self.browser_spawned.contains(&id) {
            self.browser_spawned.push(id);
            slot_browser.set_url(cx.cx, url);
        }
        slot_browser.set_visible(cx.cx, true);
        let walk = Walk {
            abs_pos: Some(page_rect.pos),
            width: Size::Fixed(page_rect.size.x),
            height: Size::Fixed(page_rect.size.y),
            ..Default::default()
        };
        let _ = slot_widget.draw_walk(cx, &mut Scope::empty(), walk);
        slot_browser.set_visible(cx.cx, false);
    }
}

fn push_modified_char(out: &mut Vec<u8>, ch: char, ctrl: bool, alt: bool, shift: bool) {
    if ctrl && !alt {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_lowercase() {
            out.push((c as u8) - b'a' + 1);
        } else {
            out.push(ch as u8);
        }
    } else if alt && !ctrl {
        out.push(0x1b);
        out.push(if shift { ch.to_ascii_uppercase() } else { ch } as u8);
    } else {
        out.push(if shift { ch.to_ascii_uppercase() } else { ch } as u8);
    }
}

impl Widget for CanvasPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        if let Event::Timer(te) = event {
            if self
                .timer
                .map(|t| t.is_timer(te).is_some())
                .unwrap_or(false)
                && self.poll_sessions()
            {
                self.redraw(cx);
            }
        }

        // Poll sessions on any event as well (output can arrive at any time).
        if self.poll_sessions() {
            self.redraw_pending = true;
            self.redraw(cx);
        }

        if let Event::MouseDown(me) = event {
            if me.button.contains(MouseButton::PRIMARY) {
                self.last_mouse = me.abs;
                // Resize handle hit-test first: bottom-right corner of the
                // topmost item under the cursor.
                let resize_target = self.resize_handle_under(me.abs);
                if let Some(id) = resize_target {
                    self.selected = Some(id);
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        self.drag = Some(DragState {
                            item_id: id,
                            grab_world: self.camera.screen_to_world(me.abs, self.world_viewport()),
                            item_origin_world: item.world().pos,
                            item_origin_size: item.world().size,
                            mode: DragMode::Resize,
                        });
                    }
                    self.redraw(cx);
                } else if let Some(id) = self.hit_test(me.abs) {
                    self.selected = Some(id);
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        let is_terminal = item.kind() == ItemKind::Terminal;
                        self.drag = Some(DragState {
                            item_id: id,
                            grab_world: self.camera.screen_to_world(me.abs, self.world_viewport()),
                            item_origin_world: item.world().pos,
                            item_origin_size: item.world().size,
                            mode: DragMode::Move,
                        });
                        if is_terminal {
                            self.focus_terminal(cx, Some(id));
                        }
                    }
                } else {
                    self.selected = None;
                    self.focused_terminal = None;
                    self.panning = true;
                }
                self.redraw(cx);
            }
        }

        if let Event::MouseMove(me) = event {
            self.last_mouse = me.abs;
            if let Some(drag) = &self.drag {
                let world = self.camera.screen_to_world(me.abs, self.world_viewport());
                let delta = world - drag.grab_world;
                let item_id = drag.item_id;
                match drag.mode {
                    DragMode::Move => {
                        let item_origin_world = drag.item_origin_world;
                        if let Some(item) = self.items.iter_mut().find(|i| i.id() == item_id) {
                            item.world_mut().pos = item_origin_world + delta;
                        }
                    }
                    DragMode::Resize => {
                        let item_origin_size = drag.item_origin_size;
                        if let Some(item) = self.items.iter_mut().find(|i| i.id() == item_id) {
                            // Clamp to a sensible minimum world size.
                            let new_w = (item_origin_size.x + delta.x).max(200.0);
                            let new_h = (item_origin_size.y + delta.y).max(140.0);
                            item.world_mut().size = Vec2d { x: new_w, y: new_h };
                        }
                    }
                }
                self.redraw(cx);
            } else if self.panning {
                self.camera
                    .pan_by(me.abs - self.last_mouse, self.world_viewport());
                self.redraw(cx);
            } else {
                let hov = self.hit_test(me.abs);
                if hov != self.hovered {
                    self.hovered = hov;
                    self.redraw(cx);
                }
            }
        }

        if let Event::MouseUp(me) = event {
            if me.button.contains(MouseButton::PRIMARY) {
                self.drag = None;
                self.panning = false;
            }
        }

        if let Event::Scroll(se) = event {
            // Zoom with pinch-like scroll; two-finger scroll on trackpads zooms
            // when the pointer is over empty canvas.
            if se.is_mouse {
                // Wheel: zoom out/in based on delta.y
                let factor = (-se.scroll.y as f32 * 0.001 + 1.0).clamp(0.5, 2.0);
                self.camera.zoom_at(factor, se.abs, self.world_viewport());
                self.redraw(cx);
            } else {
                // Trackpad: pan on scroll by default, zoom with primary modifier.
                if se.modifiers.is_primary() {
                    let factor = (-se.scroll.y as f32 * 0.01 + 1.0).clamp(0.5, 2.0);
                    self.camera.zoom_at(factor, se.abs, self.world_viewport());
                } else {
                    self.camera.pan_by(se.scroll, self.world_viewport());
                }
                self.redraw(cx);
            }
        }

        if let Event::KeyDown(key) = event {
            if let Some(id) = self.focused_terminal {
                if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                    if let Some(session) = item.session() {
                        if let Some(bytes) = self.key_to_bytes(key) {
                            session.write_bytes(&bytes);
                        }
                    }
                }
            } else {
                // Canvas-level keys.
                if key.key_code == KeyCode::Space {
                    self.focus_terminal(cx, None);
                    self.redraw(cx);
                }
            }
        }

        if let Event::TextInput(te) = event {
            if let Some(id) = self.focused_terminal {
                if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                    if let Some(session) = item.session() {
                        session.write_bytes(te.input.as_bytes());
                    }
                }
            }
        }

        // ── New-item menu ──
        let menu_btn = self.view.button(cx, ids!(menu_button));
        if menu_btn.clicked(&actions) {
            log!("canvas: menu_button clicked");
            let menu = self.view.view(cx, ids!(new_item_menu));
            let menu_visible = menu.visible();
            log!("canvas: menu visible was {menu_visible}");
            menu.set_visible(cx, !menu_visible);
            self.redraw(cx);
        }
        if self
            .view
            .button(cx, ids!(menu_new_terminal))
            .clicked(&actions)
        {
            let cmd = std::env::var("SHELL").unwrap_or_else(|_| "zsh".to_string());
            self.spawn_terminal(cx, "term", &cmd);
            self.view
                .view(cx, ids!(new_item_menu))
                .set_visible(cx, false);
            self.redraw(cx);
        }
        if self
            .view
            .button(cx, ids!(menu_new_browser))
            .clicked(&actions)
        {
            self.view
                .view(cx, ids!(new_item_menu))
                .set_visible(cx, false);
            let ti = self.view.text_input(cx, ids!(command_input));
            let prefix = "/new browser ";
            ti.set_text(cx, prefix);
            ti.set_key_focus(cx);
            ti.set_cursor(
                cx,
                makepad_widgets::makepad_draw::text::selection::Cursor {
                    index: prefix.len(),
                    prefer_next_row: false,
                },
                true,
            );
            self.redraw(cx);
        }
        if self.view.button(cx, ids!(menu_new_note)).clicked(&actions) {
            self.spawn_note(cx);
            self.view
                .view(cx, ids!(new_item_menu))
                .set_visible(cx, false);
            self.redraw(cx);
        }

        // Unified command input.
        if let Some((text, _mods)) = self
            .view
            .text_input(cx, ids!(command_input))
            .returned(&actions)
        {
            self.exec_command(cx, &text);
            let ti = self.view.text_input(cx, ids!(command_input));
            ti.set_text(cx, "");
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Canvas background
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);
        self.viewport = rect.size;
        if rect.size.x <= 1.0 || rect.size.y <= 1.0 {
            return DrawStep::done();
        }

        if self.timer.is_none() {
            self.timer = Some(cx.cx.start_interval(0.05));
        }

        self.draw_grid(cx, rect);

        // Draw items (world → screen) by index; extract item data first to
        // avoid borrowing self.items while mutating self.
        type DrawItem = (
            u64,
            ItemKind,
            Rect,
            bool,
            String,
            String,
            String,
            Option<std::sync::Arc<std::sync::Mutex<crate::terminal::state::TerminalState>>>,
        );
        let n_items = self.items.len();
        let mut draw_queue: Vec<DrawItem> = Vec::new();
        for idx in 0..n_items {
            let item = &self.items[idx];
            let screen = self.item_screen_rect(item);
            if !screen.intersects(rect) {
                continue;
            }
            let is_sel = self.selected == Some(item.id());
            let command = item
                .session()
                .map(|t| t.command.clone())
                .unwrap_or_default();
            let url = item.url().unwrap_or("").to_string();
            let state = item.session().map(|t| t.state.clone());
            draw_queue.push((
                item.id(),
                item.kind(),
                screen,
                is_sel,
                item.title().to_string(),
                command,
                url,
                state,
            ));
        }

        for (item_id, kind, item_screen, is_sel, title, command, url, state) in draw_queue {
            match kind {
                ItemKind::Terminal => {
                    self.draw_border_rect(
                        cx,
                        item_screen,
                        if is_sel { SEL_BORDER } else { TERM_BORDER },
                    );
                    // Keep the PTY grid in sync with the on-screen content
                    // area every frame (handles zoom and item resizes).
                    if let Some(session) = self
                        .items
                        .iter()
                        .find(|i| i.id() == item_id)
                        .and_then(|i| i.session())
                    {
                        let content_w = (item_screen.size.x - 12.0).max(1.0);
                        let content_h = (item_screen.size.y - 34.0).max(1.0);
                        let cols = (content_w / (TERM_CELL_W * self.camera.zoom as f64))
                            .floor()
                            .max(10.0) as usize;
                        let rows = (content_h / (TERM_CELL_H * self.camera.zoom as f64))
                            .floor()
                            .max(3.0) as usize;
                        session.resize(cols, rows);
                    }
                    if let Some(state) = state {
                        self.draw_terminal_at(cx, item_screen, &title, &command, &state);
                    }
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
                ItemKind::Note => {
                    self.draw_item_bg_rect(
                        cx,
                        item_screen,
                        if is_sel { SEL_BORDER } else { NOTE_BORDER },
                    );
                    self.draw_note_title(cx, &title, item_screen);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
                ItemKind::Browser => {
                    self.draw_browser_at(cx, item_id, item_screen, is_sel, &url);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
            }
        }

        // Children (command bar, status label) draw on top.
        while self.view.draw_walk(cx, scope, walk).step().is_some() {}

        DrawStep::done()
    }
}
