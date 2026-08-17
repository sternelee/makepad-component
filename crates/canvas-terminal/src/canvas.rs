use makepad_widgets::*;

use crate::camera::Camera;
use crate::command::{self, Command};
use crate::items::{CanvasItem, ItemKind, NoteShape, NoteTool};
use crate::terminal::state::{Cell, DEFAULT_BG};

/// Grid spacing in world units.
const GRID_SIZE: f64 = 24.0;
/// Terminal font metrics.
const TERM_CELL_W: f64 = 8.0;
const TERM_CELL_H: f64 = 17.3;

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
const DOCK_BOTTOM: f64 = 108.0;
const DOCK_BG: [f32; 4] = [0.10, 0.11, 0.15, 1.0];
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

/// A minimized item's stored data.
/// For Terminal, `extra` = command and `session` keeps the live PTY alive.
type MinimizedItem = (
    u64,
    ItemKind,
    Rect,
    String,
    String,
    Option<Box<crate::terminal::session::TerminalSession>>,
);

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
    /// The title-bar control button currently hovered, if any.
    #[rust]
    hovered_btn: Option<(u64, BtnKind)>,
    /// The dock chip currently hovered, if any.
    #[rust]
    hovered_chip: Option<u64>,
    /// Minimized items: (id, kind, world rect, title, extra, session).
    /// For Terminal, extra = command and session keeps the live PTY alive
    /// so restore can reuse it without re-spawning (which would collide on
    /// the create_only rmux session name). For Browser, extra = url.
    #[rust]
    minimized: Vec<MinimizedItem>,
    #[rust]
    focused_terminal: Option<u64>,
    /// Active terminal text selection: (item_id, start_row, start_col).
    #[rust]
    selecting: Option<(u64, usize, usize)>,
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
    /// Global whiteboard: active tool (cnvs-style palette).
    #[rust]
    tool: NoteTool,
    /// Global whiteboard: completed shapes in world coords.
    #[rust]
    shapes: Vec<NoteShape>,
    /// Global whiteboard: shape being drawn (mouse-down → move → up).
    #[rust]
    pending: Option<NoteShape>,
    /// Global whiteboard: drawing session start point in world coords.
    #[rust]
    note_draw: Option<makepad_widgets::Vec2d>,
    /// True while the Text tool is editing an in-progress text shape.
    #[rust]
    text_editing: bool,
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
    pub fn spawn_note(&mut self, cx: &mut Cx) {
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

    /// Spawn a terminal item running `command` (default shell) with the
    /// given `name`; `cwd` sets the PTY working directory (None = current
    /// directory).
    pub fn spawn_terminal(&mut self, cx: &mut Cx, name: &str, cwd: Option<&str>, command: &str) {
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
        match crate::terminal::TerminalSession::spawn(name, command, cwd, cols, rows) {
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

    /// Compute the minimize/close button rects for an item's screen rect.
    /// Buttons live at the top-right of the item title bar.
    fn control_button_rects(r: Rect) -> (Rect, Rect) {
        let bx = r.pos.x + r.size.x - 2.0 * BTN_W - 8.0;
        let by = r.pos.y + 4.0;
        let min_rect = Rect {
            pos: Vec2d { x: bx, y: by },
            size: Vec2d { x: BTN_W, y: BTN_H },
        };
        let close_rect = Rect {
            pos: Vec2d {
                x: bx + BTN_W + 4.0,
                y: by,
            },
            size: Vec2d { x: BTN_W, y: BTN_H },
        };
        (min_rect, close_rect)
    }

    /// Topmost item whose title-bar control button is under `screen`.
    fn control_button_under(&self, screen: Vec2d) -> Option<(u64, BtnKind)> {
        self.items.iter().rev().find_map(|i| {
            let r = self.item_screen_rect(i);
            let (min_r, close_r) = Self::control_button_rects(r);
            if min_r.contains(screen) {
                Some((i.id(), BtnKind::Minimize))
            } else if close_r.contains(screen) {
                Some((i.id(), BtnKind::Close))
            } else {
                None
            }
        })
    }

    /// The minimized-item chip under `screen` in the bottom dock (if any).
    fn dock_chip_under(&self, screen: Vec2d, viewport: Vec2d) -> Option<u64> {
        if self.minimized.is_empty() {
            return None;
        }
        let tray_y = viewport.y - DOCK_BOTTOM;
        let chip_w = 140.0;
        let chip_h = DOCK_H - 8.0;
        let x0 = 10.0;
        for (i, m) in self.minimized.iter().enumerate() {
            let rect = Rect {
                pos: Vec2d {
                    x: x0 + i as f64 * (chip_w + 8.0),
                    y: tray_y + 4.0,
                },
                size: Vec2d {
                    x: chip_w,
                    y: chip_h,
                },
            };
            if rect.contains(screen) {
                return Some(m.0);
            }
        }
        None
    }

    /// Minimize `id`: remove it from the canvas, add it to the bottom dock.
    fn minimize_item(&mut self, cx: &mut Cx, id: u64) {
        if self.minimized.iter().any(|m| m.0 == id) {
            return;
        }
        let Some(pos) = self.items.iter().position(|i| i.id() == id) else {
            return;
        };
        let item = self.items.remove(pos);
        let meta = match item {
            CanvasItem::Terminal {
                world,
                title,
                session,
                ..
            } => {
                let command = session
                    .as_ref()
                    .map(|s| s.command.clone())
                    .unwrap_or_default();
                (id, ItemKind::Terminal, world, title, command, session)
            }
            CanvasItem::Browser {
                world, title, url, ..
            } => (id, ItemKind::Browser, world, title, url, None),
            CanvasItem::Note { world, title, .. } => {
                (id, ItemKind::Note, world, title, String::new(), None)
            }
        };
        self.minimized.push(meta);
        self.selected = None;
        self.focused_terminal = None;
        self.redraw(cx);
    }

    /// Restore `id` from the dock back onto the canvas.
    fn restore_item(&mut self, cx: &mut Cx, id: u64) {
        let Some(pos) = self.minimized.iter().position(|m| m.0 == id) else {
            return;
        };
        let (id, kind, world, title, extra, session) = self.minimized.remove(pos);
        match kind {
            ItemKind::Terminal => {
                // Reuse the live session saved at minimize time; re-spawning
                // would collide on the create_only rmux session name.
                self.items.push(CanvasItem::Terminal {
                    id,
                    world,
                    title,
                    session,
                });
            }
            ItemKind::Browser => {
                self.items.push(CanvasItem::Browser {
                    id,
                    world,
                    title,
                    url: extra,
                });
            }
            ItemKind::Note => {
                self.items.push(CanvasItem::Note { id, world, title });
            }
        }
        self.selected = Some(id);
        self.redraw(cx);
    }

    /// Close `id`: fully remove it from the canvas and the dock.
    fn close_item(&mut self, cx: &mut Cx, id: u64) {
        self.minimized.retain(|m| m.0 != id);
        self.items.retain(|i| i.id() != id);
        if self.selected == Some(id) {
            self.selected = None;
        }
        if self.focused_terminal == Some(id) {
            self.focused_terminal = None;
        }
        self.redraw(cx);
    }

    /// Topmost item whose bottom-right resize handle is under `screen`.
    /// Screen→cell coordinate conversion for a terminal item's content area.
    fn screen_to_cell(
        &self,
        item: &CanvasItem,
        screen: Vec2d,
        viewport: Vec2d,
    ) -> Option<(usize, usize)> {
        if item.kind() != ItemKind::Terminal {
            return None;
        }
        let r = self.camera.world_rect_to_screen(item.world(), viewport);
        let origin = r.pos + Vec2d { x: 6.0, y: 30.0 };
        let state = item.session()?.state.clone();
        let grid = state.lock().ok()?;
        let cols = grid.cols.max(1);
        let rows = grid.rows.max(1);
        let char_w = TERM_CELL_W * self.camera.zoom as f64;
        let line_h = TERM_CELL_H * self.camera.zoom as f64;
        let dx = screen.x - origin.x;
        let dy = screen.y - origin.y;
        if dx < 0.0 || dy < 0.0 {
            return None;
        }
        let col = (dx / char_w) as usize;
        let row = (dy / line_h) as usize;
        if col >= cols || row >= rows {
            return None;
        }
        Some((row, col))
    }

    /// Geometry for the global left tool palette. The palette stays fixed
    /// in screen space and is vertically centered in the canvas viewport.
    fn tool_palette_rect(&self) -> Rect {
        const PALETTE_W: f64 = 40.0;
        const BTN: f64 = 28.0;
        const GAP: f64 = 4.0;
        const PADDING: f64 = 4.0;
        let height = 8.0 * (BTN + GAP) + PADDING * 2.0;
        Rect {
            pos: Vec2d {
                x: 2.0,
                y: ((self.viewport.y - height) * 0.5).max(2.0),
            },
            size: Vec2d {
                x: PALETTE_W,
                y: height,
            },
        }
    }

    /// Tool button index (0..7) under `screen` for the global palette, or None.
    fn tool_under(&self, screen: Vec2d) -> Option<usize> {
        const BTN: f64 = 28.0;
        const GAP: f64 = 4.0;
        let palette = self.tool_palette_rect();
        for i in 0..8 {
            let r = Rect {
                pos: palette.pos
                    + Vec2d {
                        x: (palette.size.x - BTN) * 0.5,
                        y: 4.0 + i as f64 * (BTN + GAP),
                    },
                size: Vec2d { x: BTN, y: BTN },
            };
            if r.contains(screen) {
                return Some(i);
            }
        }
        None
    }

    /// Return whether a screen point belongs to the fixed UI overlays rather
    /// than the drawable canvas. The bottom band contains the command bar and
    /// dock; an open new-item menu is also UI and must remain clickable.
    fn is_canvas_ui_hit(&self, cx: &Cx, screen: Vec2d) -> bool {
        const BOTTOM_UI_H: f64 = 112.0;
        if screen.y >= self.viewport.y - BOTTOM_UI_H {
            return true;
        }
        let menu = self.view.view(cx, ids!(new_item_menu));
        menu.visible() && menu.area().is_valid(cx) && menu.area().rect(cx).contains(screen)
    }

    /// Tool list in palette order (must match draw order).
    fn note_tools() -> [NoteTool; 8] {
        [
            NoteTool::Arrow,
            NoteTool::Pen,
            NoteTool::Rect,
            NoteTool::Circle,
            NoteTool::Line,
            NoteTool::Polyline,
            NoteTool::Text,
            NoteTool::Eraser,
        ]
    }

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
            Command::NewTerminal { name, cwd } => {
                // Default shell command for a new terminal.
                let cmd = std::env::var("SHELL").unwrap_or_else(|_| "zsh".to_string());
                self.spawn_terminal(cx, &name, cwd.as_deref(), &cmd);
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
            Command::Rename { old, new } => {
                if let Some(item) = self
                    .items
                    .iter_mut()
                    .find(|i| i.kind() == ItemKind::Terminal && i.title() == old)
                {
                    *item.title_mut() = new.clone();
                    self.redraw(cx);
                    self.status(cx, &format!("Renamed '{old}' → '{new}'"));
                } else {
                    self.status(cx, &format!("No terminal named '{old}'"));
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
                    "Commands: @name text · /new terminal NAME · /new browser URL · /focus NAME · /zoom N · /help",
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
            .set_dyn_instance(cx.cx, live_id!(color), &TITLE_TEXT);
        self.draw_title
            .draw_abs(cx, screen.pos + Vec2d { x: 10.0, y: 8.0 }, title);
    }

    /// Draw a straight stroke from `a` to `b` as overlapping unit squares
    /// (DrawColor is axis-aligned only; sampling the segment keeps any
    /// angle crisp enough for a whiteboard).
    fn draw_segment(
        &mut self,
        cx: &mut Cx2d,
        a: makepad_widgets::Vec2d,
        b: makepad_widgets::Vec2d,
        width: f64,
        color: [f32; 4],
    ) {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let step = (width * 0.5).max(1.0);
        let n = (dist / step).ceil().max(1.0) as usize;
        for i in 0..=n {
            let t = if n == 0 { 0.0 } else { i as f64 / n as f64 };
            let p = makepad_widgets::Vec2d {
                x: a.x + dx * t,
                y: a.y + dy * t,
            };
            self.draw_item_bg_rect(
                cx,
                Rect {
                    pos: makepad_widgets::Vec2d {
                        x: p.x - width * 0.5,
                        y: p.y - width * 0.5,
                    },
                    size: makepad_widgets::Vec2d { x: width, y: width },
                },
                color,
            );
        }
    }

    /// Draw one completed or in-progress global whiteboard shape in world coords.
    fn draw_note_shape(&mut self, cx: &mut Cx2d, shape: &NoteShape, zoom: f64, color: [f32; 4]) {
        let viewport = self.world_viewport();
        let pan = self.camera.pan;
        let zoom_f = self.camera.zoom as f64;
        let to_screen = move |p: makepad_widgets::Vec2d| -> makepad_widgets::Vec2d {
            (p - pan) * zoom_f + viewport * 0.5
        };
        let w = (2.0 * zoom).max(1.5);
        match shape {
            NoteShape::Arrow { a, b } => {
                self.draw_segment(cx, to_screen(*a), to_screen(*b), w, color);
                // Arrowhead: two short strokes near `b`.
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                let ux = dx / len;
                let uy = dy / len;
                let h = 8.0 * zoom;
                let tip = to_screen(*b);
                let left = to_screen(makepad_widgets::Vec2d {
                    x: b.x - ux * h + uy * h * 0.5,
                    y: b.y - uy * h - ux * h * 0.5,
                });
                let right = to_screen(makepad_widgets::Vec2d {
                    x: b.x - ux * h - uy * h * 0.5,
                    y: b.y - uy * h + ux * h * 0.5,
                });
                self.draw_segment(cx, tip, left, w, color);
                self.draw_segment(cx, tip, right, w, color);
            }
            NoteShape::Line { a, b } => {
                self.draw_segment(cx, to_screen(*a), to_screen(*b), w, color);
            }
            NoteShape::Pen { points } | NoteShape::Polyline { points } => {
                for seg in points.windows(2) {
                    self.draw_segment(cx, to_screen(seg[0]), to_screen(seg[1]), w, color);
                }
                if let Some(p) = points.last() {
                    let p = to_screen(*p);
                    self.draw_item_bg_rect(
                        cx,
                        Rect {
                            pos: makepad_widgets::Vec2d {
                                x: p.x - w * 0.5,
                                y: p.y - w * 0.5,
                            },
                            size: makepad_widgets::Vec2d { x: w, y: w },
                        },
                        color,
                    );
                }
            }
            NoteShape::Rect { a, b } => {
                let x0 = a.x.min(b.x);
                let y0 = a.y.min(b.y);
                let x1 = a.x.max(b.x);
                let y1 = a.y.max(b.y);
                let corners = [
                    makepad_widgets::Vec2d { x: x0, y: y0 },
                    makepad_widgets::Vec2d { x: x1, y: y0 },
                    makepad_widgets::Vec2d { x: x1, y: y1 },
                    makepad_widgets::Vec2d { x: x0, y: y1 },
                    makepad_widgets::Vec2d { x: x0, y: y0 },
                ];
                for seg in corners.windows(2) {
                    self.draw_segment(cx, to_screen(seg[0]), to_screen(seg[1]), w, color);
                }
            }
            NoteShape::Circle { center, r } => {
                let c = to_screen(*center);
                let rr = r * zoom;
                let n = ((rr * std::f64::consts::TAU) / (w * 0.5)).ceil().max(12.0) as usize;
                let mut prev = makepad_widgets::Vec2d {
                    x: c.x + rr,
                    y: c.y,
                };
                for i in 1..=n {
                    let ang = (i as f64 / n as f64) * std::f64::consts::TAU;
                    let cur = makepad_widgets::Vec2d {
                        x: c.x + rr * ang.cos(),
                        y: c.y + rr * ang.sin(),
                    };
                    self.draw_segment(cx, prev, cur, w, color);
                    prev = cur;
                }
            }
            NoteShape::Text { pos, text } => {
                let p = to_screen(*pos);
                self.draw_title.draw_vars.set_dyn_instance(
                    cx.cx,
                    live_id!(color),
                    &[1.0, 1.0, 1.0, 1.0],
                );
                // Show a blinking caret while the in-progress text shape is
                // being edited (empty text still shows the caret).
                if self.text_editing {
                    self.draw_title.draw_abs(cx, p, &format!("{text}▏"));
                } else {
                    self.draw_title.draw_abs(cx, p, text);
                }
            }
        }
    }

    /// Draw a note whiteboard: title bar, left vertical tool palette, and
    /// the drawing canvas with all shapes (cnvs-style).
    /// Global whiteboard palette: a fixed vertical tool bar on the LEFT edge
    /// of the canvas (screen-fixed, like the dock), always available.
    fn draw_tool_palette(&mut self, cx: &mut Cx2d) {
        const PALETTE_W: f64 = 40.0;
        const BTN: f64 = 28.0;
        const GAP: f64 = 4.0;
        let palette_rect = self.tool_palette_rect();
        self.draw_item_bg_rect(cx, palette_rect, [0.12, 0.14, 0.20, 0.94]);

        for (i, t) in Self::note_tools().iter().enumerate() {
            let r = Rect {
                pos: palette_rect.pos
                    + Vec2d {
                        x: (PALETTE_W - BTN) * 0.5,
                        y: 4.0 + i as f64 * (BTN + GAP),
                    },
                size: Vec2d { x: BTN, y: BTN },
            };
            let active = *t == self.tool;
            self.draw_item_bg_rect(
                cx,
                r,
                if active {
                    [0.30, 0.62, 0.98, 0.85]
                } else {
                    [0.17, 0.19, 0.26, 1.0]
                },
            );
            if active {
                self.draw_border_rect(cx, r, [0.45, 0.72, 1.0, 1.0]);
            }
            self.draw_title.draw_vars.set_dyn_instance(
                cx.cx,
                live_id!(color),
                &if active {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.72, 0.78, 0.90, 1.0]
                },
            );
            self.draw_title
                .draw_abs(cx, r.pos + Vec2d { x: 6.0, y: 4.0 }, t.label());
        }
    }

    /// Draw all global whiteboard shapes (world coords) plus the in-progress
    /// stroke, clipped to the whole canvas viewport.
    fn draw_canvas_shapes(
        &mut self,
        cx: &mut Cx2d,
        viewport: Vec2d,
        shapes: &[NoteShape],
        pending: Option<&NoteShape>,
    ) {
        let zoom = self.camera.zoom as f64;
        let ink: [f32; 4] = [0.92, 0.95, 1.0, 1.0];
        cx.push_clip_rect(Rect {
            pos: Vec2d { x: 0.0, y: 0.0 },
            size: viewport,
        });
        for shape in shapes {
            self.draw_note_shape(cx, shape, zoom, ink);
        }
        if let Some(p) = pending {
            self.draw_note_shape(cx, p, zoom, ink);
        }
        cx.pop_clip_rect();
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

    /// Draw the minimize/close control buttons in the item's title bar.
    fn draw_control_buttons(&mut self, cx: &mut Cx2d, id: u64, screen: Rect) {
        let (min_r, close_r) = Self::control_button_rects(screen);
        let min_hov = self.hovered_btn == Some((id, BtnKind::Minimize));
        let close_hov = self.hovered_btn == Some((id, BtnKind::Close));
        // Minimize button (—)
        self.draw_item_bg_rect(cx, min_r, if min_hov { BTN_HOVER } else { BTN_BG });
        self.draw_border_rect(cx, min_r, BTN_BORDER);
        self.draw_cursor.color = Vec4f {
            x: 0.7,
            y: 0.75,
            z: 0.85,
            w: 1.0,
        };
        let dash = Rect {
            pos: min_r.pos + Vec2d { x: 6.0, y: 8.0 },
            size: Vec2d {
                x: min_r.size.x - 12.0,
                y: 2.0,
            },
        };
        self.draw_cursor.draw_abs(cx, dash);
        // Close button (×)
        self.draw_item_bg_rect(
            cx,
            close_r,
            if close_hov { BTN_CLOSE_HOVER } else { BTN_BG },
        );
        self.draw_border_rect(cx, close_r, BTN_BORDER);
        self.draw_cursor.color = Vec4f {
            x: 0.9,
            y: 0.85,
            z: 0.85,
            w: 1.0,
        };
        let x1 = Rect {
            pos: close_r.pos + Vec2d { x: 6.0, y: 5.0 },
            size: Vec2d {
                x: close_r.size.x - 12.0,
                y: 2.0,
            },
        };
        self.draw_cursor.draw_abs(cx, x1);
        let x2 = Rect {
            pos: close_r.pos + Vec2d { x: 6.0, y: 11.0 },
            size: Vec2d {
                x: close_r.size.x - 12.0,
                y: 2.0,
            },
        };
        self.draw_cursor.draw_abs(cx, x2);
    }

    /// Draw the bottom dock tray with minimized-item chips.
    fn draw_dock(&mut self, cx: &mut Cx2d, viewport: Vec2d) {
        if self.minimized.is_empty() {
            return;
        }
        let tray_y = viewport.y - DOCK_BOTTOM;
        let tray_rect = Rect {
            pos: Vec2d { x: 0.0, y: tray_y },
            size: Vec2d {
                x: viewport.x,
                y: DOCK_H,
            },
        };
        self.draw_item_bg_rect(cx, tray_rect, DOCK_BG);
        self.draw_border_rect(cx, tray_rect, CHIP_BORDER);

        let chip_w = 140.0;
        let chip_h = DOCK_H - 8.0;
        let x0 = 10.0;
        let minimized: Vec<(u64, ItemKind, String)> = self
            .minimized
            .iter()
            .map(|m| (m.0, m.1, m.3.clone()))
            .collect();
        let hovered_chip = self.hovered_chip;
        for (i, (id, kind, title)) in minimized.iter().enumerate() {
            let rect = Rect {
                pos: Vec2d {
                    x: x0 + i as f64 * (chip_w + 8.0),
                    y: tray_y + 4.0,
                },
                size: Vec2d {
                    x: chip_w,
                    y: chip_h,
                },
            };
            let hov = hovered_chip == Some(*id);
            self.draw_item_bg_rect(cx, rect, if hov { CHIP_BG_HOVER } else { CHIP_BG });
            self.draw_border_rect(cx, rect, CHIP_BORDER);
            let kind_label = match kind {
                ItemKind::Terminal => ">_",
                ItemKind::Browser => "◎",
                ItemKind::Note => "📝",
            };
            let label = format!("{kind_label} {title}");
            self.draw_cell_text
                .draw_vars
                .set_dyn_instance(cx.cx, live_id!(color), &TITLE_TEXT);
            self.draw_cell_text
                .draw_abs(cx, rect.pos + Vec2d { x: 8.0, y: 6.0 }, &label);
        }
    }

    /// Render one terminal grid row as color runs.
    #[allow(clippy::too_many_arguments)]
    fn draw_terminal_row(
        &mut self,
        cx: &mut Cx2d,
        row: &[Cell],
        x0: f64,
        y: f64,
        char_w: f64,
        line_h: f64,
        dim: bool,
        in_sel: impl Fn(usize, usize) -> bool,
    ) {
        let mut i = 0;
        while i < row.len() {
            let cell = &row[i];
            let fg = cell.fg;
            let bg = cell.bg;
            let bold = cell.bold;
            let mut j = i;
            while j < row.len() {
                let c = &row[j];
                if c.fg != fg || c.bg != bg || c.bold != bold {
                    break;
                }
                j += 1;
            }
            let run_rect = Rect {
                pos: Vec2d {
                    x: x0 + i as f64 * char_w,
                    y,
                },
                size: Vec2d {
                    x: (j - i) as f64 * char_w,
                    y: line_h,
                },
            };
            let selected = in_sel(i, j);
            let eff_bg = if selected {
                // Selection highlight (semi-transparent blue).
                [0.20, 0.45, 0.90, 0.55]
            } else if bg != DEFAULT_BG {
                [bg[0], bg[1], bg[2], 1.0]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            if selected || bg != DEFAULT_BG {
                let a = eff_bg[3];
                let c = if dim {
                    [eff_bg[0] * 0.6, eff_bg[1] * 0.6, eff_bg[2] * 0.6, a]
                } else {
                    eff_bg
                };
                self.draw_cell_bg.color = Vec4f {
                    x: c[0],
                    y: c[1],
                    z: c[2],
                    w: c[3],
                };
                self.draw_cell_bg.draw_abs(cx, run_rect);
            }
            let mut col = [fg[0], fg[1], fg[2], 1.0];
            if dim {
                col[0] *= 0.6;
                col[1] *= 0.6;
                col[2] *= 0.6;
            }
            self.draw_cell_text.color = Vec4f {
                x: col[0],
                y: col[1],
                z: col[2],
                w: col[3],
            };
            // Per-character slot drawing: each glyph at its cell's fixed
            // x (char_w grid), so no advance-vs-cell-width drift overlaps.
            for (k, cell) in row.iter().take(j).skip(i).enumerate() {
                if cell.wide_padding {
                    continue;
                }
                if cell.ch == ' ' {
                    continue;
                }
                let cx_pos = x0 + (i + k) as f64 * char_w;
                let cs = cell.ch.to_string();
                self.draw_cell_text
                    .draw_abs(cx, Vec2d { x: cx_pos, y }, &cs);
            }
            i = j;
        }
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
            .set_dyn_instance(cx.cx, live_id!(color), &TITLE_TEXT);
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

        // Fixed cell size: characters keep a constant width/height; dragging
        // the resize handle only changes how many cols/rows fit (the PTY is
        // resized by draw_walk), never stretching glyphs.
        let cols = grid.cols.max(1);
        let rows = grid.rows.max(1);
        let char_w = TERM_CELL_W * self.camera.zoom as f64;
        let line_h = TERM_CELL_H * self.camera.zoom as f64;
        // Scale the font with zoom so glyph width matches char_w (prevents
        // horizontal overlap when zooming out; glyphs would otherwise stay at
        // fixed size and collide).
        self.draw_cell_text.font_scale = self.camera.zoom;

        // Tail view: show the LAST `rows` lines (the on-screen viewport).
        let total = grid.lines.len();
        let start = total.saturating_sub(rows);
        let draw_rows = (total - start).min(rows);

        // Scrollback: when scrolled, show the tail of `grid.scrollback`
        // above the main grid (both rendered as cell runs with colors).
        let hist_offset = grid.scroll_offset.min(grid.scrollback.len());
        let hist_start = grid.scrollback.len().saturating_sub(hist_offset);
        let hist_rows: Vec<&Vec<Cell>> = grid.scrollback.iter().skip(hist_start).collect();
        // Live rows below the history, limited by remaining space.
        let live_rows = rows.saturating_sub(hist_offset);
        let live_rows = live_rows.min(draw_rows);

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
        // History rows first (dimmed), then live grid rows.
        let sel = grid.selection;
        let mut disp_r = 0usize;
        // History rows (scrollback tail).
        for row in hist_rows.iter() {
            let y = origin.y + disp_r as f64 * line_h;
            let row_sel: Vec<Cell> = (*row).clone();
            self.draw_terminal_row(
                cx,
                &row_sel,
                origin.x,
                y,
                char_w,
                line_h,
                true,
                |c0, _c1| {
                    sel.is_some_and(|(r0, c0s, r1, c1s)| {
                        let (ra, rb) = (r0.min(r1), r0.max(r1));
                        let (ca, cb) = (c0s.min(c1s), c0s.max(c1s));
                        (ra..=rb).contains(&disp_r) && (ca..=cb).contains(&c0)
                    })
                },
            );
            disp_r += 1;
        }
        // Live grid rows (dimmed if scrolled).
        for r in 0..live_rows {
            let row = &grid.lines[start + r];
            let y = origin.y + disp_r as f64 * line_h;
            let dim = hist_offset > 0;
            let disp_row = disp_r;
            let row_sel = row.clone();
            self.draw_terminal_row(cx, &row_sel, origin.x, y, char_w, line_h, dim, |c0, _c1| {
                sel.is_some_and(|(r0, c0s, r1, c1s)| {
                    let (ra, rb) = (r0.min(r1), r0.max(r1));
                    let (ca, cb) = (c0s.min(c1s), c0s.max(c1s));
                    (ra..=rb).contains(&disp_row) && (ca..=cb).contains(&c0)
                })
            });
            disp_r += 1;
        }

        let _ = sel;

        // Cursor (block / beam / underline per PTY style)
        if grid.cursor_visible() {
            let c = grid.cursor_col.min(cols - 1);
            let r = grid
                .cursor_row
                .saturating_sub(start)
                .min(draw_rows.saturating_sub(1));
            let pos = origin
                + Vec2d {
                    x: c as f64 * char_w,
                    y: (hist_offset + r) as f64 * line_h,
                };
            self.draw_cursor.color = Vec4f {
                x: 0.30,
                y: 0.62,
                z: 0.98,
                w: 1.0,
            };
            match grid.cursor_style {
                1 => {
                    // Beam (vertical bar): thin column at the left edge.
                    let w = (char_w * 0.12).max(2.0);
                    self.draw_cursor.draw_abs(
                        cx,
                        Rect {
                            pos,
                            size: Vec2d { x: w, y: line_h },
                        },
                    );
                }
                2 => {
                    // Underline: thin bar at the bottom.
                    let h = (line_h * 0.15).max(2.0);
                    self.draw_cursor.draw_abs(
                        cx,
                        Rect {
                            pos: Vec2d {
                                x: pos.x,
                                y: pos.y + line_h - h,
                            },
                            size: Vec2d { x: char_w, y: h },
                        },
                    );
                }
                _ => {
                    // Block (default)
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
            }
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
            .set_dyn_instance(cx.cx, live_id!(color), &TITLE_TEXT);
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
        let slot_id = LiveId::from_str(&format!("browser_slot_{}", slot));
        let slot_widget = self.view.widget(cx.cx, &[slot_id]);
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

#[allow(clippy::collapsible_match)]
impl Widget for CanvasPanel {
    #[allow(clippy::collapsible_match)]
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
                // Title-bar control buttons take priority.
                if let Some((id, kind)) = self.control_button_under(me.abs) {
                    match kind {
                        BtnKind::Minimize => self.minimize_item(cx, id),
                        BtnKind::Close => self.close_item(cx, id),
                    }
                    return;
                }
                // Dock chip: restore a minimized item.
                if let Some(id) = self.dock_chip_under(me.abs, self.viewport) {
                    self.restore_item(cx, id);
                    return;
                }
                // Global whiteboard palette: clicking a tool switches it.
                if let Some(tool_idx) = self.tool_under(me.abs) {
                    self.tool = Self::note_tools()[tool_idx];
                    self.redraw(cx);
                    return;
                }
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
                        // In the terminal CONTENT area (below title bar),
                        // drag starts a text selection like alacritty/wezterm;
                        // the title bar still drags/moves the item.
                        let in_content = if is_terminal {
                            let r = self
                                .camera
                                .world_rect_to_screen(item.world(), self.world_viewport());
                            me.abs.y > r.pos.y + 26.0
                        } else {
                            false
                        };
                        if is_terminal && in_content {
                            if let Some((row, col)) =
                                self.screen_to_cell(item, me.abs, self.world_viewport())
                            {
                                self.selecting = Some((id, row, col));
                                if let Some(session) = item.session() {
                                    if let Ok(mut st) = session.state.lock() {
                                        st.selection = Some((row, col, row, col));
                                    }
                                }
                            }
                            // Clicking the content area focuses the terminal too
                            // (otherwise keyboard input stays routed to the
                            // command bar after focus left the terminal).
                            // `item` borrow ends here; focus_terminal needs &mut self.
                            self.focus_terminal(cx, Some(id));
                        } else {
                            self.drag = Some(DragState {
                                item_id: id,
                                grab_world: self
                                    .camera
                                    .screen_to_world(me.abs, self.world_viewport()),
                                item_origin_world: item.world().pos,
                                item_origin_size: item.world().size,
                                mode: DragMode::Move,
                            });
                            if is_terminal {
                                self.focus_terminal(cx, Some(id));
                            }
                        }
                    }
                } else {
                    // Empty canvas press: start a global whiteboard stroke
                    // with the active tool. Canvas panning stays on the
                    // trackpad scroll (or middle-drag) like other whiteboards.
                    self.selected = None;
                    self.focused_terminal = None;
                    self.panning = false;
                    let ui_hit = self.is_canvas_ui_hit(cx, me.abs);
                    let world = self.camera.screen_to_world(me.abs, self.world_viewport());
                    if !ui_hit {
                        if self.tool == NoteTool::Text {
                            // Text tool: click places an empty, editable text
                            // shape; typing appends until Return/Escape. No
                            // note_draw session, so MouseUp won't commit it.
                            self.text_editing = true;
                            self.pending = Some(NoteShape::Text {
                                pos: world,
                                text: String::new(),
                            });
                        } else {
                            self.note_draw = Some(world);
                            if self.tool != NoteTool::Eraser {
                                self.pending = Some(match self.tool {
                                    NoteTool::Pen => NoteShape::Pen {
                                        points: vec![world],
                                    },
                                    NoteTool::Polyline => NoteShape::Polyline {
                                        points: vec![world],
                                    },
                                    NoteTool::Arrow => NoteShape::Arrow { a: world, b: world },
                                    NoteTool::Rect => NoteShape::Rect { a: world, b: world },
                                    NoteTool::Circle => NoteShape::Circle {
                                        center: world,
                                        r: 0.0,
                                    },
                                    NoteTool::Line => NoteShape::Line { a: world, b: world },
                                    NoteTool::Text => unreachable!(),
                                    NoteTool::Eraser => unreachable!(),
                                });
                            }
                        }
                    }
                }
                self.redraw(cx);
            }
        }

        if let Event::MouseMove(me) = event {
            self.last_mouse = me.abs;
            if let Some(start) = self.note_draw {
                let local = self.camera.screen_to_world(me.abs, self.world_viewport());
                let pending = &mut self.pending;
                let tool = self.tool;
                match tool {
                    NoteTool::Pen | NoteTool::Polyline => {
                        if let Some(NoteShape::Pen { points })
                        | Some(NoteShape::Polyline { points }) = pending
                        {
                            let last = points.last().copied().unwrap_or(start);
                            if (local.x - last.x).hypot(local.y - last.y) > 2.0 {
                                points.push(local);
                            }
                        }
                    }
                    NoteTool::Arrow => {
                        if let Some(NoteShape::Arrow { a, .. }) = pending {
                            *a = start;
                        }
                        *pending = Some(NoteShape::Arrow { a: start, b: local });
                    }
                    NoteTool::Line => {
                        *pending = Some(NoteShape::Line { a: start, b: local });
                    }
                    NoteTool::Rect => {
                        *pending = Some(NoteShape::Rect { a: start, b: local });
                    }
                    NoteTool::Circle => {
                        let r = ((local.x - start.x).powi(2) + (local.y - start.y).powi(2)).sqrt();
                        *pending = Some(NoteShape::Circle { center: start, r });
                    }
                    NoteTool::Text | NoteTool::Eraser => {}
                }
                self.redraw(cx);
                return;
            }
            if let Some((sel_id, sr, sc)) = self.selecting {
                if let Some(item) = self.items.iter().find(|i| i.id() == sel_id) {
                    if let Some((er, ec)) = self.screen_to_cell(item, me.abs, self.world_viewport())
                    {
                        if let Some(session) = item.session() {
                            if let Ok(mut st) = session.state.lock() {
                                st.selection = Some((sr, sc, er, ec));
                            }
                        }
                        self.redraw(cx);
                    }
                }
            } else if let Some(drag) = &self.drag {
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
                let hov_btn = self.control_button_under(me.abs);
                let hov_chip = self.dock_chip_under(me.abs, self.viewport);
                if hov != self.hovered
                    || hov_btn != self.hovered_btn
                    || hov_chip != self.hovered_chip
                {
                    self.hovered = hov;
                    self.hovered_btn = hov_btn;
                    self.hovered_chip = hov_chip;
                    self.redraw(cx);
                }
            }
        }

        if let Event::MouseUp(me) = event {
            if me.button.contains(MouseButton::PRIMARY) {
                self.drag = None;
                self.panning = false;
                // Commit a global whiteboard drawing session (draw shape / erase).
                if let Some(_start) = self.note_draw.take() {
                    if self.tool == NoteTool::Eraser {
                        // Erase shapes under the release point.
                        let local = self.camera.screen_to_world(me.abs, self.world_viewport());
                        let tol = 8.0 / (self.camera.zoom as f64).max(0.1);
                        self.shapes.retain(|sh| !sh.hit(local, tol));
                    } else if let Some(shape) = self.pending.take() {
                        self.shapes.push(shape);
                    }
                    self.redraw(cx);
                }
                if let Some((sel_id, _, _)) = self.selecting {
                    self.selecting = None;
                    if let Some(item) = self.items.iter().find(|i| i.id() == sel_id) {
                        if let Some(session) = item.session() {
                            let text = if let Ok(st) = session.state.lock() {
                                st.selected_text()
                            } else {
                                String::new()
                            };
                            if !text.is_empty() {
                                cx.copy_to_clipboard(&text);
                                self.status(cx, &format!("Copied {} chars", text.chars().count()));
                            }
                        }
                    }
                }
            }
        }

        if let Event::Scroll(se) = event {
            // Scrollback: if the cursor is over a terminal's content area,
            // scroll the terminal history instead of panning/zooming the
            // canvas (like alacritty/wezterm).
            let term_under = self.items.iter().rev().find(|i| {
                i.kind() == ItemKind::Terminal
                    && self.item_screen_rect(i).contains(se.abs)
                    && se.abs.y > self.item_screen_rect(i).pos.y + 26.0
            });
            if let Some(item) = term_under {
                if let Some(session) = item.session() {
                    // Wheel up (negative scroll.y) shows history: offset+.
                    let delta = (se.scroll.y / 20.0).round() as i32;
                    if delta != 0 {
                        session.scroll_display(-delta);
                        self.redraw(cx);
                    }
                    return;
                }
            }
            // Not over a terminal: zoom/pan the canvas as before.
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
            // Whiteboard text editing takes priority over terminal input.
            if self.text_editing {
                let commit = |pending: &mut Option<NoteShape>,
                             shapes: &mut Vec<NoteShape>|
                -> bool {
                    match pending.take() {
                        Some(NoteShape::Text { .. }) => true,
                        other => {
                            if let Some(s) = other {
                                shapes.push(s);
                            }
                            false
                        }
                    }
                };
                match key.key_code {
                    KeyCode::Backspace => {
                        if let Some(NoteShape::Text { text, .. }) = &mut self.pending {
                            text.pop();
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ReturnKey => {
                        let was_text = commit(&mut self.pending, &mut self.shapes);
                        self.text_editing = false;
                        if was_text {
                            self.redraw(cx);
                        }
                    }
                    KeyCode::Escape => {
                        self.pending = None;
                        self.text_editing = false;
                        self.redraw(cx);
                    }
                    _ => {}
                }
            } else if let Some(id) = self.focused_terminal {
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
            if self.text_editing {
                if let Some(NoteShape::Text { text, .. }) = &mut self.pending {
                    text.push_str(&te.input);
                }
                self.redraw(cx);
            } else if let Some(id) = self.focused_terminal {
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
            self.spawn_terminal(cx, "term", None, &cmd);
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
                    self.draw_control_buttons(cx, item_id, item_screen);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
                ItemKind::Note => {
                    self.draw_border_rect(
                        cx,
                        item_screen,
                        if is_sel { SEL_BORDER } else { NOTE_BORDER },
                    );
                    self.draw_note_title(cx, &title, item_screen);
                    self.draw_control_buttons(cx, item_id, item_screen);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
                ItemKind::Browser => {
                    self.draw_browser_at(cx, item_id, item_screen, is_sel, &url);
                    self.draw_control_buttons(cx, item_id, item_screen);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
            }
        }

        // Global whiteboard shapes (world coords) draw ON TOP of items so
        // annotations/flowcharts can mark terminals and browsers.
        let shapes = self.shapes.clone();
        let pending = self.pending.clone();
        self.draw_canvas_shapes(cx, rect.size, &shapes, pending.as_ref());

        // Bottom dock (minimized items tray) draws FIRST — it sits in the
        // gap between the command bar and the bottom edge (y-108..y-74), and
        // drawing it before the command-bar pass lets the new-item menu
        // (which pops up above the input row into the dock's band) render on
        // top of the dock instead of being occluded by it.
        self.draw_dock(cx, rect.size);

        // Children (command bar, status label, popup menu) draw AFTER the
        // dock, so the menu always covers the dock tray.
        while self.view.draw_walk(cx, scope, walk).step().is_some() {}

        // Global tool palette: fixed to the left edge, always on top.
        self.draw_tool_palette(cx);

        DrawStep::done()
    }
}
