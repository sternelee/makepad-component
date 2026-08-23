use makepad_widgets::*;

use crate::camera::Camera;
use crate::command::{self, Command};
use crate::items::{AgentStatus, CanvasItem, DrawnShape, ItemKind, NoteShape, NoteTool};
use crate::terminal::state::{Cell, DEFAULT_BG};

/// Grid spacing in world units.
const GRID_SIZE: f64 = 24.0;
/// Terminal font metrics.
const TERM_CELL_W: f64 = 8.0;
const TERM_CELL_H: f64 = 17.3;

/// Sticky-note (hand-drawn) palette: cream paper, pencil-brown wobbly edge,
/// translucent washi tape strip, dark ink text.
const NOTE_PAPER: [f32; 4] = [0.97, 0.93, 0.80, 1.0];
const NOTE_PAPER_EDGE: [f32; 4] = [0.45, 0.37, 0.26, 1.0];
const NOTE_TAPE: [f32; 4] = [0.55, 0.78, 0.92, 0.45];
const NOTE_TAPE_EDGE: [f32; 4] = [1.0, 1.0, 1.0, 0.35];
const NOTE_TITLE_INK: [f32; 4] = [0.28, 0.22, 0.14, 1.0];
const TERM_BG: [f32; 4] = [0.075, 0.082, 0.10, 1.0];
const TERM_BORDER: [f32; 4] = [0.20, 0.24, 0.32, 1.0];
const SEL_BORDER: [f32; 4] = [0.30, 0.62, 0.98, 1.0];
const TITLE_TEXT: [f32; 4] = [0.85, 0.88, 0.94, 1.0];

/// Music player card colors.
const MUSIC_BG: [f32; 4] = [0.10, 0.11, 0.16, 1.0];
const MUSIC_BORDER: [f32; 4] = [0.28, 0.32, 0.44, 1.0];
const MUSIC_ACCENT: [f32; 4] = [0.95, 0.48, 0.32, 1.0];
const MUSIC_TEXT: [f32; 4] = [0.90, 0.92, 0.96, 1.0];
const MUSIC_SECONDARY: [f32; 4] = [0.55, 0.60, 0.72, 1.0];
const MUSIC_PROGRESS_BG: [f32; 4] = [0.18, 0.20, 0.28, 1.0];
const MUSIC_BAR: [f32; 4] = [0.30, 0.62, 0.98, 1.0];

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

/// Top workspace tab bar.
const TAB_BAR_H: f64 = 34.0;
const TAB_BG: [f32; 4] = [0.10, 0.11, 0.15, 1.0];
const TAB_ACTIVE_BG: [f32; 4] = [0.20, 0.24, 0.34, 1.0];
const TAB_INACTIVE_BG: [f32; 4] = [0.14, 0.16, 0.22, 1.0];
const TAB_BORDER: [f32; 4] = [0.28, 0.32, 0.42, 1.0];
const TAB_PLUS_W: f64 = 32.0;

/// Whiteboard ink color presets (RGBA) shown in the tool palette.
const INK_COLORS: [[f32; 4]; 6] = [
    [0.92, 0.95, 1.00, 1.0], // white
    [0.30, 0.62, 0.98, 1.0], // blue
    [0.62, 0.78, 0.34, 1.0], // green
    [0.95, 0.75, 0.28, 1.0], // yellow
    [0.95, 0.26, 0.21, 1.0], // red
    [0.90, 0.39, 0.70, 1.0], // magenta
];
/// Whiteboard stroke width presets (world units) shown in the tool palette.
const INK_WIDTHS: [f64; 3] = [1.5, 3.0, 6.0];

/// Note body ink color presets (RGBA), dark enough to read on cream paper.
const NOTE_TEXT_COLORS: [[f32; 4]; 6] = [
    [0.25, 0.21, 0.16, 1.0], // ink black
    [0.16, 0.32, 0.72, 1.0], // indigo
    [0.25, 0.48, 0.24, 1.0], // forest green
    [0.72, 0.42, 0.10, 1.0], // ochre
    [0.78, 0.22, 0.18, 1.0], // brick red
    [0.55, 0.26, 0.62, 1.0], // plum
];

/// Warm amber accent for hand-drawn UI markers (active tool ring, selection).
const PAL_ACCENT: [f32; 4] = [0.98, 0.72, 0.28, 1.0];

/// Avatar color presets (RGBA) for terminal/agent cards.
const AVATAR_COLORS: [[f32; 4]; 8] = [
    [0.30, 0.62, 0.98, 1.0], // blue
    [0.95, 0.48, 0.32, 1.0], // orange
    [0.62, 0.78, 0.34, 1.0], // green
    [0.90, 0.39, 0.70, 1.0], // magenta
    [0.95, 0.75, 0.28, 1.0], // yellow
    [0.55, 0.90, 0.93, 1.0], // cyan
    [0.98, 0.55, 0.55, 1.0], // red
    [0.75, 0.55, 0.95, 1.0], // purple
];

/// Pick a stable avatar color from a name string.
fn name_color(name: &str) -> [f32; 4] {
    let mut h = 0u32;
    for b in name.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    AVATAR_COLORS[h as usize % AVATAR_COLORS.len()]
}

/// Deterministic pseudo-random in [-1, 1] from (seed, index). Stable across
/// frames so hand-drawn wobble never flickers.
fn sketch_rand(seed: u32, i: u32) -> f64 {
    let mut h = seed.wrapping_mul(0x9E3779B1) ^ i.wrapping_mul(0x85EBCA77);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2AE3D);
    h ^= h >> 16;
    (h % 2000) as f64 / 1000.0 - 1.0
}

/// Convenience: RGBA array → Vec4f for direct DrawText/DrawColor assignment
/// (draw_vars.set_dyn_instance is a no-op for these shaders; the direct
/// `.color` field is the reliable path, as used by the terminal cell renderer).
fn vec4f(c: [f32; 4]) -> Vec4f {
    Vec4f {
        x: c[0],
        y: c[1],
        z: c[2],
        w: c[3],
    }
}

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

/// What part of the workspace tab bar a click landed on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WorkspaceTabHit {
    Tab(usize),
    Add,
}

/// What part of the global tool palette a click landed on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PaletteHit {
    Tool(usize),
    Color(usize),
    Width(usize),
}

/// Active Move-tool drag: which shape is being dragged and the grab point.
struct MoveDrag {
    shape_idx: usize,
    grab_world: Vec2d,
    /// Snapshot of shapes before the move, for undo (committed only if the
    /// shape actually moved).
    snapshot: Vec<DrawnShape>,
}

/// Per-workspace canvas state. The panel swaps the active workspace in and out
/// so each space keeps its own terminals, camera, whiteboard, etc.
struct Workspace {
    camera: Camera,
    items: Vec<CanvasItem>,
    next_item_id: u64,
    selected: Option<u64>,
    focused_terminal: Option<u64>,
    minimized: Vec<MinimizedItem>,
    browser_spawned: Vec<u64>,
    browser_slots: Vec<(u64, usize)>,
    tool: NoteTool,
    shapes: Vec<DrawnShape>,
    pending: Option<NoteShape>,
    /// Seed of the in-progress shape's hand-drawn wobble.
    pending_seed: u32,
    note_draw: Option<Vec2d>,
    text_editing: bool,
    undo_stack: Vec<Vec<DrawnShape>>,
    redo_stack: Vec<Vec<DrawnShape>>,
    ink_color_idx: usize,
    ink_width_idx: usize,
    erase_snapshot: Option<Vec<DrawnShape>>,
    last_click_time: f64,
    last_click_pos: Vec2d,
    move_drag: Option<MoveDrag>,
    hovered_shape: Option<usize>,
    grid_enabled: bool,
    prop_selected_id: Option<u64>,
    prop_synced: bool,
    ime_active: bool,
}

impl Workspace {
    fn empty() -> Self {
        Self {
            camera: Camera::default(),
            items: Vec::new(),
            next_item_id: 1,
            selected: None,
            focused_terminal: None,
            minimized: Vec::new(),
            browser_spawned: Vec::new(),
            browser_slots: Vec::new(),
            tool: NoteTool::default(),
            shapes: Vec::new(),
            pending: None,
            pending_seed: 0,
            note_draw: None,
            text_editing: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            ink_color_idx: 0,
            ink_width_idx: 1,
            erase_snapshot: None,
            last_click_time: 0.0,
            last_click_pos: Vec2d::default(),
            move_drag: None,
            hovered_shape: None,
            grid_enabled: false,
            prop_selected_id: None,
            prop_synced: false,
            ime_active: false,
        }
    }
}

/// A minimized item's stored data.
/// For Terminal, `extra` = command, `status` = agent presence, and `session`
/// keeps the live PTY alive.
type MinimizedItem = (
    u64,
    ItemKind,
    Rect,
    String,
    String,
    Option<crate::items::AgentStatus>,
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
    shapes: Vec<DrawnShape>,
    /// Global whiteboard: shape being drawn (mouse-down → move → up).
    #[rust]
    pending: Option<NoteShape>,
    /// Hand-drawn wobble seed for the `pending` shape.
    #[rust]
    pending_seed: u32,
    /// Monotonic counter used to mint fresh wobble seeds.
    #[rust]
    next_seed: u32,
    /// Global whiteboard: drawing session start point in world coords.
    #[rust]
    note_draw: Option<makepad_widgets::Vec2d>,
    /// True while the Text tool is editing an in-progress text shape.
    #[rust]
    text_editing: bool,
    /// Undo/redo history for the whiteboard shapes list.
    #[rust]
    undo_stack: Vec<Vec<DrawnShape>>,
    #[rust]
    redo_stack: Vec<Vec<DrawnShape>>,
    /// Selected ink color index into `INK_COLORS`.
    #[rust]
    ink_color_idx: usize,
    /// Selected ink width index into `INK_WIDTHS`.
    #[rust]
    ink_width_idx: usize,
    /// Snapshot of `shapes` taken when an eraser drag starts; committed to
    /// the undo stack only if the eraser actually removed something.
    #[rust]
    erase_snapshot: Option<Vec<DrawnShape>>,
    /// Last mouse-down time/pos, for Polyline double-click to finish.
    #[rust]
    last_click_time: f64,
    #[rust]
    last_click_pos: Vec2d,
    /// True while the canvas is keeping the platform text IME active for
    /// whiteboard text editing. Toggled in draw_walk so hide_text_ime
    /// fires once when editing ends.
    #[rust]
    ime_active: bool,
    /// Active Move-tool drag (shape index + grab point + undo snapshot).
    #[rust]
    move_drag: Option<MoveDrag>,
    /// Shape index hovered by the Move tool (for highlight), if any.
    #[rust]
    hovered_shape: Option<usize>,
    /// Whether to draw the grid overlay on top of the CNVS background.
    #[rust]
    grid_enabled: bool,
    /// Recently executed commands (oldest first), capped to 50.
    #[rust]
    command_history: Vec<String>,
    /// Index into `command_history` when browsing with Ctrl+Up/Down.
    /// None means not browsing history.
    #[rust]
    history_index: Option<usize>,
    /// Last text seen in the command input, to detect changes.
    #[rust]
    last_input_text: String,
    /// Current suggestion selection index.
    #[rust]
    suggestion_index: usize,
    /// Which item id is currently shown in the properties panel.
    #[rust]
    prop_selected_id: Option<u64>,
    /// Whether the properties panel widgets were just synced from the item.
    #[rust]
    prop_synced: bool,
    /// All workspaces; only the active one is unpacked into the panel fields.
    #[rust]
    workspaces: Vec<Workspace>,
    /// Index of the currently active workspace.
    #[rust]
    current_workspace: usize,
    /// Note currently being edited inline, if any.
    #[rust]
    note_edit_id: Option<u64>,
    /// Inline edit buffer for the note being edited.
    #[rust]
    note_edit_buffer: String,
    /// Caret position in `note_edit_buffer` (char index).
    #[rust]
    note_edit_caret: usize,
}

impl CanvasPanel {
    fn world_viewport(&self) -> Vec2d {
        self.viewport
    }

    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid_enabled = enabled;
    }

    /// Ensure at least one workspace exists (called lazily from draw_walk).
    fn ensure_workspace(&mut self) {
        if self.workspaces.is_empty() {
            self.workspaces.push(Workspace::empty());
            self.current_workspace = 0;
        }
    }

    /// Pack the current panel fields back into the active workspace slot.
    fn save_current_workspace(&mut self) {
        self.ensure_workspace();
        let ws = Workspace {
            camera: self.camera,
            items: std::mem::take(&mut self.items),
            next_item_id: self.next_item_id,
            selected: self.selected,
            focused_terminal: self.focused_terminal,
            minimized: std::mem::take(&mut self.minimized),
            browser_spawned: std::mem::take(&mut self.browser_spawned),
            browser_slots: std::mem::take(&mut self.browser_slots),
            tool: self.tool,
            shapes: std::mem::take(&mut self.shapes),
            pending: self.pending.take(),
            pending_seed: self.pending_seed,
            note_draw: self.note_draw,
            text_editing: self.text_editing,
            undo_stack: std::mem::take(&mut self.undo_stack),
            redo_stack: std::mem::take(&mut self.redo_stack),
            ink_color_idx: self.ink_color_idx,
            ink_width_idx: self.ink_width_idx,
            erase_snapshot: self.erase_snapshot.take(),
            last_click_time: self.last_click_time,
            last_click_pos: self.last_click_pos,
            move_drag: self.move_drag.take(),
            hovered_shape: self.hovered_shape,
            grid_enabled: self.grid_enabled,
            prop_selected_id: self.prop_selected_id,
            prop_synced: self.prop_synced,
            ime_active: self.ime_active,
        };
        if self.current_workspace < self.workspaces.len() {
            self.workspaces[self.current_workspace] = ws;
        } else {
            self.workspaces.push(ws);
            self.current_workspace = self.workspaces.len() - 1;
        }
    }

    /// Switch to workspace `idx`, swapping the stored state into the panel.
    fn switch_workspace(&mut self, cx: &mut Cx, idx: usize) {
        if idx >= self.workspaces.len() || idx == self.current_workspace {
            return;
        }
        self.save_current_workspace();
        let mut ws = std::mem::replace(&mut self.workspaces[idx], Workspace::empty());
        self.camera = ws.camera;
        self.items = std::mem::take(&mut ws.items);
        self.next_item_id = ws.next_item_id;
        self.selected = ws.selected;
        self.focused_terminal = ws.focused_terminal;
        self.minimized = std::mem::take(&mut ws.minimized);
        self.browser_spawned = std::mem::take(&mut ws.browser_spawned);
        self.browser_slots = std::mem::take(&mut ws.browser_slots);
        self.tool = ws.tool;
        self.shapes = std::mem::take(&mut ws.shapes);
        self.pending = ws.pending.take();
        self.pending_seed = ws.pending_seed;
        self.note_draw = ws.note_draw;
        self.text_editing = ws.text_editing;
        self.undo_stack = std::mem::take(&mut ws.undo_stack);
        self.redo_stack = std::mem::take(&mut ws.redo_stack);
        self.ink_color_idx = ws.ink_color_idx;
        self.ink_width_idx = ws.ink_width_idx;
        self.erase_snapshot = ws.erase_snapshot.take();
        self.last_click_time = ws.last_click_time;
        self.last_click_pos = ws.last_click_pos;
        self.move_drag = ws.move_drag.take();
        self.hovered_shape = ws.hovered_shape;
        self.grid_enabled = ws.grid_enabled;
        self.prop_selected_id = ws.prop_selected_id;
        self.prop_synced = ws.prop_synced;
        self.ime_active = ws.ime_active;
        self.current_workspace = idx;
        // Clear transient cross-workspace interaction state.
        self.drag = None;
        self.panning = false;
        self.selecting = None;
        self.hovered = None;
        self.hovered_btn = None;
        self.hovered_chip = None;
        self.note_draw = None;
        self.text_editing = false;
        self.note_edit_id = None;
        self.note_edit_buffer.clear();
        self.note_edit_caret = 0;
        self.redraw(cx);
    }

    /// Add a new empty workspace and switch to it.
    fn add_workspace(&mut self, cx: &mut Cx) {
        self.save_current_workspace();
        self.workspaces.push(Workspace::empty());
        self.switch_workspace(cx, self.workspaces.len() - 1);
    }

    fn item_screen_rect(&self, item: &CanvasItem) -> Rect {
        self.camera
            .world_rect_to_screen(item.world(), self.world_viewport())
    }

    /// Current ink color (RGBA) from the palette selection.
    fn ink_color(&self) -> [f32; 4] {
        INK_COLORS[self.ink_color_idx.min(INK_COLORS.len() - 1)]
    }

    /// Current stroke width (world units) from the palette selection.
    fn ink_width(&self) -> f64 {
        INK_WIDTHS[self.ink_width_idx.min(INK_WIDTHS.len() - 1)]
    }

    /// Snapshot the current shapes list onto the undo stack and clear redo.
    /// Call before any mutation to `self.shapes`.
    fn push_undo(&mut self) {
        self.undo_stack.push(self.shapes.clone());
        // Cap history to avoid unbounded growth.
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo the last whiteboard change. Restores shapes from the undo stack
    /// and pushes the current state onto the redo stack.
    fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack
                .push(std::mem::replace(&mut self.shapes, prev));
            true
        } else {
            false
        }
    }

    /// Redo the last undone whiteboard change.
    fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack
                .push(std::mem::replace(&mut self.shapes, next));
            true
        } else {
            false
        }
    }

    /// Commit the in-progress `pending` shape to `shapes` with the current
    /// ink color/width, recording an undo entry. Returns true if a shape was
    /// committed.
    fn commit_pending(&mut self) -> bool {
        if let Some(shape) = self.pending.take() {
            self.push_undo();
            self.shapes.push(DrawnShape {
                shape,
                color: self.ink_color(),
                width: self.ink_width(),
                seed: self.pending_seed,
            });
            true
        } else {
            false
        }
    }

    /// Mint a fresh hand-drawn wobble seed for a new pending shape.
    fn fresh_seed(&mut self) -> u32 {
        self.next_seed = self.next_seed.wrapping_add(1);
        self.next_seed.wrapping_mul(0x9E3779B1) | 1
    }

    /// Screen-space bounding rect of a whiteboard text shape. Text is drawn
    /// at a fixed screen font size (not scaled by zoom), so bounds live in
    /// screen space.
    fn text_screen_bounds(&self, pos: Vec2d, text: &str) -> Rect {
        let p = self.camera.world_to_screen(pos, self.world_viewport());
        const CHAR_W: f64 = 7.5;
        const LINE_H: f64 = 16.0;
        let mut max_w = 0.0;
        let mut lines = 0;
        for line in text.split('\n') {
            let w = line.chars().count() as f64 * CHAR_W;
            if w > max_w {
                max_w = w;
            }
            lines += 1;
        }
        if lines == 0 {
            lines = 1;
        }
        Rect {
            pos: p,
            size: Vec2d {
                x: max_w.max(CHAR_W),
                y: lines as f64 * LINE_H,
            },
        }
    }

    /// Index of an existing Text shape whose screen bounds contain `screen`.
    fn text_shape_under(&self, screen: Vec2d) -> Option<usize> {
        for (i, ds) in self.shapes.iter().enumerate().rev() {
            if let NoteShape::Text { pos, text } = &ds.shape {
                if self.text_screen_bounds(*pos, text).contains(screen) {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Index of the topmost whiteboard shape under `screen` (for the Move
    /// tool). Text shapes are hit-tested in screen space (fixed font size);
    /// all others are hit-tested in world space with a zoom-aware tolerance.
    fn shape_under(&self, screen: Vec2d) -> Option<usize> {
        let world = self.camera.screen_to_world(screen, self.world_viewport());
        let tol = 8.0 / (self.camera.zoom as f64).max(0.1);
        for (i, ds) in self.shapes.iter().enumerate().rev() {
            if let NoteShape::Text { pos, text } = &ds.shape {
                if self.text_screen_bounds(*pos, text).contains(screen) {
                    return Some(i);
                }
            } else if ds.shape.hit(world, tol) {
                return Some(i);
            }
        }
        None
    }

    /// Enter inline-edit mode for the note `id`.
    fn start_note_edit(&mut self, cx: &mut Cx, id: u64) {
        if let Some(item) = self.items.iter().find(|i| i.id() == id) {
            if let Some(body) = item.body() {
                self.note_edit_id = Some(id);
                self.note_edit_buffer = body.to_string();
                self.note_edit_caret = self.note_edit_buffer.chars().count();
                self.selected = Some(id);
                // A focused terminal must not keep receiving keys while we edit
                // a note; otherwise typed text leaks into the shell.
                self.focused_terminal = None;
                // Hand the focus over to the hidden note editor so that Makepad's
                // TextInput handles IME composition, backspace, arrows, etc.
                // The canvas draws the buffer itself in draw_note_title.
                let editor = self.view.text_input(cx, ids!(note_editor));
                editor.set_text(cx, body);
                editor.set_cursor(
                    cx,
                    makepad_widgets::makepad_draw::text::selection::Cursor {
                        index: body.len(),
                        prefer_next_row: false,
                    },
                    false,
                );
                editor.set_key_focus(cx);
                self.redraw(cx);
            }
        }
    }

    /// Commit the inline edit buffer back to the note and exit edit mode.
    fn finish_note_edit(&mut self, cx: &mut Cx) {
        if let Some(id) = self.note_edit_id.take() {
            // Prefer the hidden editor's text so the final IME composition is
            // captured; fall back to the cached buffer if the editor is gone.
            let buffer = {
                let editor = self.view.text_input(cx, ids!(note_editor));
                let text = editor.text();
                if text.is_empty() {
                    std::mem::take(&mut self.note_edit_buffer)
                } else {
                    text
                }
            };
            self.note_edit_buffer.clear();
            self.note_edit_caret = 0;
            if let Some(item) = self.items.iter_mut().find(|i| i.id() == id) {
                if let Some(body) = item.body_mut() {
                    *body = buffer;
                }
            }
            // Release the hidden editor and return focus to the command bar
            // without changing the current selection.
            self.view.text_input(cx, ids!(note_editor)).set_text(cx, "");
            self.view
                .text_input(
                    cx,
                    ids!(
                        command_wrap
                            .command_bar
                            .input_row
                            .input_capsule
                            .command_input
                    ),
                )
                .set_key_focus(cx);
            self.redraw(cx);
        }
    }

    /// Pull the latest text/cursor from the hidden note editor so the canvas
    /// can draw the body and caret in sync with IME composition.
    fn sync_note_editor(&mut self, cx: &mut Cx) {
        if self.note_edit_id.is_none() {
            return;
        }
        let editor = self.view.text_input(cx, ids!(note_editor));
        let text = editor.text();
        if text != self.note_edit_buffer {
            self.note_edit_buffer = text;
            self.redraw(cx);
        }
        let cursor = editor.cursor();
        let index = cursor.index.min(self.note_edit_buffer.len());
        let new_caret = self.note_edit_buffer[..index].chars().count();
        if new_caret != self.note_edit_caret {
            self.note_edit_caret = new_caret;
            self.redraw(cx);
        }
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
                size: Vec2d { x: 260.0, y: 160.0 },
            },
            title: format!("Note {}", self.next_item_id),
            body: "Double-click to edit this note.\nUse the right panel to style it.".to_string(),
            font_size: 13.0,
            color_idx: 0,
        };
        self.next_item_id += 1;
        self.items.push(item);
        self.redraw(cx);
    }

    /// Spawn a music player card at the current view center.
    pub fn spawn_music_player(&mut self, cx: &mut Cx, title: &str) {
        let world_pos = self
            .camera
            .screen_to_world(self.viewport * 0.5, self.world_viewport());
        let item = CanvasItem::MusicPlayer {
            id: self.next_item_id,
            world: Rect {
                pos: world_pos,
                size: Vec2d { x: 320.0, y: 140.0 },
            },
            title: title.to_string(),
            progress: 0.0,
            playing: true,
        };
        self.next_item_id += 1;
        let id = item.id();
        self.items.push(item);
        self.selected = Some(id);
        self.redraw(cx);
    }

    /// Spawn a terminal item running `command` (default shell) with the
    /// given `name`; `cwd` sets the PTY working directory (None = current
    /// directory).
    pub fn spawn_terminal(&mut self, cx: &mut Cx, name: &str, cwd: Option<&str>, command: &str) {
        let (world, cols, rows) = self.new_terminal_geometry();
        match crate::terminal::TerminalSession::spawn(name, command, cwd, cols, rows) {
            Ok(session) => self.place_terminal(cx, world, session),
            Err(e) => {
                log!("canvas: failed to spawn terminal '{name}': {e}");
                self.status(cx, &format!("Failed to spawn {name}: {e}"));
            }
        }
    }

    /// Re-attach a terminal card to a persistent daemon session by name
    /// (GUI restart path), replaying its scrollback into the local grid.
    /// Failures are logged silently — startup may race a session dying
    /// right as we attach.
    pub fn attach_terminal(&mut self, cx: &mut Cx, name: &str) {
        let (mut world, cols, rows) = self.new_terminal_geometry();
        // Cascade each restored card by the number of terminals already on
        // the canvas, so several re-attached sessions don't stack exactly.
        let n = self
            .items
            .iter()
            .filter(|i| i.kind() == ItemKind::Terminal)
            .count() as f64;
        world.pos.x += n * 26.0;
        world.pos.y += n * 22.0;
        match crate::terminal::TerminalSession::attach(name, cols, rows) {
            Ok(session) => self.place_terminal(cx, world, session),
            Err(e) => log!("canvas: failed to attach terminal '{name}': {e}"),
        }
    }

    /// Default terminal card geometry: 620×380 world units at canvas center.
    fn new_terminal_geometry(&self) -> (Rect, usize, usize) {
        let world_pos = self
            .camera
            .screen_to_world(self.viewport * 0.5, self.world_viewport());
        let world = Rect {
            pos: world_pos,
            size: Vec2d { x: 620.0, y: 380.0 },
        };
        let (cols, rows) = self.term_grid_size_from(world.size.x, world.size.y);
        (world, cols, rows)
    }

    /// Push a terminal card at `world` with a live `session`; the card title
    /// is the session name. Selects and focuses it.
    fn place_terminal(
        &mut self,
        cx: &mut Cx,
        world: Rect,
        session: crate::terminal::TerminalSession,
    ) {
        let id = self.next_item_id;
        self.next_item_id += 1;
        let name = session.name.clone();
        self.items.push(CanvasItem::Terminal {
            id,
            world,
            title: name,
            status: crate::items::AgentStatus::default(),
            session: Some(Box::new(session)),
        });
        self.selected = Some(id);
        self.focused_terminal = Some(id);
        self.set_canvas_focus(cx);
        self.redraw(cx);
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
            let ti = self.view.text_input(
                cx,
                ids!(
                    command_wrap
                        .command_bar
                        .input_row
                        .input_capsule
                        .command_input
                ),
            );
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
                status,
                session,
                ..
            } => {
                let command = session
                    .as_ref()
                    .map(|s| s.command.clone())
                    .unwrap_or_default();
                (
                    id,
                    ItemKind::Terminal,
                    world,
                    title,
                    command,
                    Some(status),
                    session,
                )
            }
            CanvasItem::Browser {
                world, title, url, ..
            } => (id, ItemKind::Browser, world, title, url, None, None),
            CanvasItem::Note {
                world, title, body, ..
            } => (id, ItemKind::Note, world, title, body, None, None),
            CanvasItem::MusicPlayer {
                world,
                title,
                progress,
                ..
            } => (
                id,
                ItemKind::MusicPlayer,
                world,
                title,
                progress.to_string(),
                None,
                None,
            ),
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
        let (id, kind, world, title, extra, status, session) = self.minimized.remove(pos);
        match kind {
            ItemKind::Terminal => {
                // Reuse the live session saved at minimize time; re-spawning
                // would collide on the create_only rmux session name.
                self.items.push(CanvasItem::Terminal {
                    id,
                    world,
                    title,
                    status: status.unwrap_or_default(),
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
                self.items.push(CanvasItem::Note {
                    id,
                    world,
                    title,
                    body: extra,
                    font_size: 13.0,
                    color_idx: 0,
                });
            }
            ItemKind::MusicPlayer => {
                let progress = extra.parse::<f32>().unwrap_or(0.0);
                self.items.push(CanvasItem::MusicPlayer {
                    id,
                    world,
                    title,
                    progress,
                    playing: false,
                });
            }
        }
        self.selected = Some(id);
        // Restoring a non-terminal item should not leave a stale terminal focus.
        if !matches!(kind, ItemKind::Terminal) {
            self.focused_terminal = None;
        }
        self.redraw(cx);
    }

    /// Close `id`: fully remove it from the canvas and the dock. Terminal
    /// sessions are killed so a closed terminal doesn't keep running in the
    /// daemon and resurrect on the next launch.
    fn close_item(&mut self, cx: &mut Cx, id: u64) {
        if let Some(item) = self.items.iter().find(|i| i.id() == id) {
            if let Some(session) = item.session() {
                session.kill();
            }
        }
        self.minimized.retain(|m| m.0 != id);
        self.items.retain(|i| i.id() != id);
        if self.selected == Some(id) {
            self.selected = None;
        }
        if self.focused_terminal == Some(id) {
            self.focused_terminal = None;
        }
        if self.note_edit_id == Some(id) {
            self.note_edit_id = None;
            self.note_edit_buffer.clear();
            self.note_edit_caret = 0;
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
    /// Palette layout constants shared by drawing and hit-testing.
    const PAL_W: f64 = 40.0;
    const PAL_BTN: f64 = 28.0;
    const PAL_GAP: f64 = 4.0;
    const PAL_PAD: f64 = 4.0;
    const PAL_SWATCH: f64 = 16.0;
    const PAL_SWATCH_GAP: f64 = 2.0;
    const PAL_WIDTH_BTN_H: f64 = 16.0;

    /// Absolute palette rect plus the Y origin of each section (tools,
    /// colors, widths), computed once so draw and hit-test agree.
    fn palette_layout(&self) -> (Rect, f64, f64, f64) {
        let tools_h = 10.0 * (Self::PAL_BTN + Self::PAL_GAP);
        let colors_h = 3.0 * (Self::PAL_SWATCH + Self::PAL_SWATCH_GAP);
        let widths_h = 3.0 * (Self::PAL_WIDTH_BTN_H + Self::PAL_GAP);
        let total_h = Self::PAL_PAD
            + tools_h
            + Self::PAL_GAP
            + colors_h
            + Self::PAL_GAP
            + widths_h
            + Self::PAL_PAD;
        let palette_rect = Rect {
            pos: Vec2d {
                x: 2.0,
                y: ((self.viewport.y - total_h) * 0.5).max(2.0),
            },
            size: Vec2d {
                x: Self::PAL_W,
                y: total_h,
            },
        };
        let tools_y = palette_rect.pos.y + Self::PAL_PAD;
        let colors_y = tools_y + tools_h + Self::PAL_GAP;
        let widths_y = colors_y + colors_h + Self::PAL_GAP;
        (palette_rect, tools_y, colors_y, widths_y)
    }

    /// What part of the global tool palette a screen point hits, if any.
    fn palette_hit(&self, screen: Vec2d) -> Option<PaletteHit> {
        let (palette, tools_y, colors_y, widths_y) = self.palette_layout();
        // Tools: 8 buttons, centered horizontally.
        for i in 0..10 {
            let r = Rect {
                pos: palette.pos
                    + Vec2d {
                        x: (palette.size.x - Self::PAL_BTN) * 0.5,
                        y: tools_y - palette.pos.y + i as f64 * (Self::PAL_BTN + Self::PAL_GAP),
                    },
                size: Vec2d {
                    x: Self::PAL_BTN,
                    y: Self::PAL_BTN,
                },
            };
            if r.contains(screen) {
                return Some(PaletteHit::Tool(i));
            }
        }
        // Colors: 6 swatches in a 2-column grid.
        let col_w = Self::PAL_SWATCH + Self::PAL_SWATCH_GAP;
        let grid_x = palette.pos.x + (palette.size.x - 2.0 * col_w + Self::PAL_SWATCH_GAP) * 0.5;
        for i in 0..6 {
            let col = i % 2;
            let row = i / 2;
            let r = Rect {
                pos: Vec2d {
                    x: grid_x + col as f64 * col_w,
                    y: colors_y + row as f64 * (Self::PAL_SWATCH + Self::PAL_SWATCH_GAP),
                },
                size: Vec2d {
                    x: Self::PAL_SWATCH,
                    y: Self::PAL_SWATCH,
                },
            };
            if r.contains(screen) {
                return Some(PaletteHit::Color(i));
            }
        }
        // Widths: 3 buttons (full palette width), each shows a sample line.
        for i in 0..3 {
            let r = Rect {
                pos: Vec2d {
                    x: palette.pos.x + (palette.size.x - Self::PAL_BTN) * 0.5,
                    y: widths_y + i as f64 * (Self::PAL_WIDTH_BTN_H + Self::PAL_GAP),
                },
                size: Vec2d {
                    x: Self::PAL_BTN,
                    y: Self::PAL_WIDTH_BTN_H,
                },
            };
            if r.contains(screen) {
                return Some(PaletteHit::Width(i));
            }
        }
        None
    }

    /// Return whether a screen point belongs to the fixed UI overlays rather
    /// than the drawable canvas. The bottom band contains the command bar and
    /// dock; the right-side properties panel and an open new-item menu are also
    /// UI and must remain clickable.
    fn is_canvas_ui_hit(&self, cx: &Cx, screen: Vec2d) -> bool {
        if screen.y <= TAB_BAR_H {
            return true;
        }
        const BOTTOM_UI_H: f64 = 112.0;
        if screen.y >= self.viewport.y - BOTTOM_UI_H {
            return true;
        }
        let menu = self.view.view(cx, ids!(new_item_menu));
        if menu.visible() && menu.area().is_valid(cx) && menu.area().rect(cx).contains(screen) {
            return true;
        }
        // Right-side properties panel (known geometry fallback in case the
        // panel's own area rect does not yet reflect its right-aligned layout).
        const RIGHT_PANEL_W: f64 = 220.0;
        const RIGHT_PANEL_RIGHT_PAD: f64 = 12.0;
        const RIGHT_PANEL_TOP_PAD: f64 = 46.0;
        const RIGHT_PANEL_BOTTOM_PAD: f64 = 120.0;
        let panel = self
            .view
            .view(cx, ids!(right_panel_container.properties_panel));
        if panel.visible() {
            if panel.area().is_valid(cx) && panel.area().rect(cx).contains(screen) {
                return true;
            }
            if screen.x >= self.viewport.x - RIGHT_PANEL_W - RIGHT_PANEL_RIGHT_PAD
                && screen.x <= self.viewport.x - RIGHT_PANEL_RIGHT_PAD
                && screen.y >= RIGHT_PANEL_TOP_PAD
                && screen.y <= self.viewport.y - RIGHT_PANEL_BOTTOM_PAD
            {
                return true;
            }
        }
        false
    }

    /// Tool list in palette order (must match draw order).
    fn note_tools() -> [NoteTool; 10] {
        [
            NoteTool::Move,
            NoteTool::Arrow,
            NoteTool::Pen,
            NoteTool::Line,
            NoteTool::Rect,
            NoteTool::Circle,
            NoteTool::Ellipse,
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
            Command::NewMusicPlayer { title } => {
                self.spawn_music_player(cx, &title);
                self.status(cx, &format!("Music player: {title}"));
            }
            Command::SetStatus { name, status } => {
                let new_status = match status.to_lowercase().as_str() {
                    "busy" => AgentStatus::Busy,
                    "idle" => AgentStatus::Idle,
                    _ => AgentStatus::Online,
                };
                if let Some(item) = self
                    .items
                    .iter_mut()
                    .find(|i| i.kind() == ItemKind::Terminal && i.title() == name)
                {
                    if let Some(st) = item.agent_status_mut() {
                        *st = new_status;
                        self.redraw(cx);
                        self.status(cx, &format!("{name} is now {}", new_status.label()));
                    }
                } else {
                    self.status(cx, &format!("No terminal named '{name}'"));
                }
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
                    // Update the daemon session name too, so a renamed
                    // terminal keeps its name across a GUI restart
                    // (sessions are re-attached by name).
                    if let Some(session) = item.session() {
                        session.rename(&new);
                    }
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
                    "Commands: @name text · /new terminal NAME · /new browser URL · /new music TITLE · /new note · /status NAME online|busy|idle · /focus NAME · /zoom N · /grid · /clear · /help",
                );
            }
            Command::Clear => {
                if self.shapes.is_empty() {
                    self.status(cx, "Whiteboard already empty");
                } else {
                    self.push_undo();
                    self.shapes.clear();
                    self.redraw(cx);
                    self.status(cx, "Whiteboard cleared");
                }
            }
            Command::Grid => {
                self.grid_enabled = !self.grid_enabled;
                self.redraw(cx);
                self.status(
                    cx,
                    if self.grid_enabled {
                        "Grid overlay on"
                    } else {
                        "Grid overlay off"
                    },
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

    /// Push an executed command onto the history ring, avoiding duplicates at
    /// the tail and capping at 50 entries.
    fn push_command_history(&mut self, text: String) {
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        if self.command_history.last() == Some(&text) {
            return;
        }
        self.command_history.push(text);
        if self.command_history.len() > 50 {
            self.command_history.remove(0);
        }
    }

    /// All static command templates offered as suggestions.
    fn command_templates() -> &'static [&'static str] {
        &[
            "@",
            "/new terminal ",
            "/new browser ",
            "/new note",
            "/new music ",
            "/status ",
            "/focus ",
            "/rename ",
            "/zoom ",
            "/grid",
            "/clear",
            "/help",
        ]
    }

    /// Suggestions for the current command input text.
    fn suggestions_for(&self, text: &str) -> Vec<String> {
        let text = text.trim_start();
        let mut out: Vec<String> = Vec::new();
        // Static templates that start with the typed prefix.
        for t in Self::command_templates() {
            if t.starts_with(text) && !out.contains(&t.to_string()) {
                out.push(t.to_string());
            }
        }
        // History items that start with the typed prefix.
        for h in self.command_history.iter().rev() {
            if h.starts_with(text) && !out.contains(h) {
                out.push(h.clone());
            }
        }
        out.into_iter().take(6).collect()
    }

    /// Update the suggestion dropdown labels and visibility.
    fn update_suggestions(&mut self, cx: &mut Cx) {
        let input_id = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        let text = self.view.text_input(cx, input_id).text();
        if text != self.last_input_text {
            self.last_input_text = text.clone();
            self.suggestion_index = 0;
            let suggestions = self.suggestions_for(&text);
            let list = self
                .view
                .view(cx, ids!(command_wrap.command_bar.suggestion_list));
            let has_suggestions = !suggestions.is_empty() && !text.is_empty();
            list.set_visible(cx, has_suggestions);
            for i in 0..6 {
                let id = LiveId::from_str(&format!("suggestion_{i}"));
                let label = list.label(cx, &[id]);
                if let Some(s) = suggestions.get(i) {
                    label.set_visible(cx, true);
                    let prefix = if i == self.suggestion_index {
                        "▸ "
                    } else {
                        "  "
                    };
                    label.set_text(cx, &format!("{prefix}{s}"));
                } else {
                    label.set_visible(cx, false);
                    label.set_text(cx, "");
                }
            }
        }
    }

    /// Accept the currently selected suggestion into the command input.
    fn accept_suggestion(&mut self, cx: &mut Cx) {
        let input_id = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        let text = self.view.text_input(cx, input_id).text();
        let suggestions = self.suggestions_for(&text);
        if let Some(s) = suggestions.get(self.suggestion_index) {
            let ti = self.view.text_input(cx, input_id);
            ti.set_text(cx, s);
            ti.set_key_focus(cx);
            self.last_input_text = s.clone();
        }
        self.view
            .view(cx, ids!(command_wrap.command_bar.suggestion_list))
            .set_visible(cx, false);
    }

    /// Sync the right-side properties panel with the currently selected item.
    /// Only updates the panel when the selection changes to avoid overwriting
    /// user edits while typing.
    fn sync_properties_panel(&mut self, cx: &mut Cx) {
        let panel = self
            .view
            .view(cx, ids!(right_panel_container.properties_panel));
        if self.selected == self.prop_selected_id && self.prop_synced {
            return;
        }
        self.prop_selected_id = self.selected;
        self.prop_synced = true;
        if let Some(id) = self.selected {
            if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                if let CanvasItem::Note {
                    title,
                    body,
                    font_size,
                    color_idx,
                    ..
                } = item
                {
                    panel.set_visible(cx, true);
                    self.view
                        .text_input(cx, ids!(right_panel_container.properties_panel.prop_title))
                        .set_text(cx, title);
                    self.view
                        .text_input(cx, ids!(right_panel_container.properties_panel.prop_body))
                        .set_text(cx, body);
                    // Highlight selected color/font buttons by toggling text.
                    for i in 0..6 {
                        let id = LiveId::from_str(&format!("prop_color_{i}"));
                        let btn = self.view.button(cx, &[id]);
                        let label = if i == *color_idx { "◉" } else { "●" };
                        btn.set_text(cx, label);
                    }
                    let font_labels = ["11", "13", "16", "20"];
                    let font_values = [11.0f32, 13.0, 16.0, 20.0];
                    for i in 0..4 {
                        let id = LiveId::from_str(&format!("prop_font_{i}"));
                        let btn = self.view.button(cx, &[id]);
                        let label = if (font_values[i] - *font_size).abs() < 0.5 {
                            format!("[{}]", font_labels[i])
                        } else {
                            font_labels[i].to_string()
                        };
                        btn.set_text(cx, &label);
                    }
                    return;
                }
            }
        }
        panel.set_visible(cx, false);
    }

    /// Apply property panel edits to the selected Note item.
    fn apply_properties_panel(&mut self, cx: &mut Cx) {
        let Some(id) = self.selected else {
            return;
        };
        let title = self
            .view
            .text_input(cx, ids!(right_panel_container.properties_panel.prop_title))
            .text();
        let body = self
            .view
            .text_input(cx, ids!(right_panel_container.properties_panel.prop_body))
            .text();
        if let Some(item) = self.items.iter_mut().find(|i| i.id() == id) {
            if let CanvasItem::Note {
                title: t, body: b, ..
            } = item
            {
                *t = title;
                *b = body;
            }
        }
    }

    /// True if any text widget (command bar, note editor, properties panel)
    /// currently holds key focus. While one does, keystrokes belong to it and
    /// must NOT be forwarded to a focused terminal.
    fn any_text_focused(&self, cx: &Cx) -> bool {
        let command_input = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        self.view.text_input(cx, command_input).key_focus(cx)
            || self.view.text_input(cx, ids!(note_editor)).key_focus(cx)
            || self
                .view
                .text_input(cx, ids!(right_panel_container.properties_panel.prop_title))
                .key_focus(cx)
            || self
                .view
                .text_input(cx, ids!(right_panel_container.properties_panel.prop_body))
                .key_focus(cx)
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
        self.draw_grid
            .draw_vars
            .set_uniform(cx.cx, live_id!(time), &[cx.cx.time() as f32]);
        self.draw_grid.draw_vars.set_uniform(
            cx.cx,
            live_id!(grid_enabled),
            &[if self.grid_enabled { 1.0f32 } else { 0.0f32 }],
        );
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

    /// Draw a soft drop shadow behind `rect` using a few offset translucent
    /// quads. Kept simple (no blur) to stay within DrawColor and avoid the
    /// DrawQuad-pixel-shader text corruption issue.
    fn draw_shadow_rect(&mut self, cx: &mut Cx2d, rect: Rect) {
        let offsets = [2.0, 5.0, 9.0, 14.0];
        let alphas = [0.18, 0.10, 0.05, 0.02];
        for (i, off) in offsets.iter().enumerate() {
            let a = alphas[i];
            let r = Rect {
                pos: rect.pos + Vec2d { x: *off, y: *off },
                size: rect.size,
            };
            self.draw_item_bg_rect(cx, r, [0.0, 0.0, 0.0, a]);
        }
    }

    /// Draw a glow border around `rect` for selected items. Uses concentric
    /// translucent quads to fake a soft outer glow.
    fn draw_glow_border(&mut self, cx: &mut Cx2d, rect: Rect, color: [f32; 4]) {
        let layers = [
            (14.0, 0.03),
            (10.0, 0.06),
            (6.0, 0.10),
            (3.0, 0.18),
            (1.5, 0.55),
        ];
        for (pad, alpha) in layers {
            let r = Rect {
                pos: rect.pos - Vec2d { x: pad, y: pad },
                size: rect.size
                    + Vec2d {
                        x: pad * 2.0,
                        y: pad * 2.0,
                    },
            };
            let c = [color[0], color[1], color[2], alpha];
            self.draw_item_bg_rect(cx, r, c);
        }
    }

    /// Draw a small avatar chip (colored square + initials) for agent/terminal
    /// cards. We use DrawColor instead of SDF to avoid pixel-shader side effects.
    fn draw_avatar(
        &mut self,
        cx: &mut Cx2d,
        pos: makepad_widgets::Vec2d,
        name: &str,
        bg: [f32; 4],
    ) {
        const SIZE: f64 = 18.0;
        let rect = Rect {
            pos,
            size: Vec2d { x: SIZE, y: SIZE },
        };
        self.draw_item_bg_rect(cx, rect, bg);
        // Slight border to define the chip.
        self.draw_border_rect(cx, rect, [1.0, 1.0, 1.0, 0.2]);
        let initial: String = name
            .chars()
            .filter(|c| c.is_alphabetic())
            .take(1)
            .collect::<String>()
            .to_uppercase();
        if !initial.is_empty() {
            self.draw_title.color = vec4f([1.0, 1.0, 1.0, 0.9]);
            self.draw_title
                .draw_abs(cx, pos + Vec2d { x: 5.0, y: 2.0 }, &initial);
        }
    }

    fn draw_note_title(
        &mut self,
        cx: &mut Cx2d,
        id: u64,
        title: &str,
        body: &str,
        font_size: f32,
        color_idx: usize,
        screen: Rect,
        is_sel: bool,
        editing: bool,
        caret: usize,
    ) {
        // Per-note seed keeps the hand-drawn wobble stable for this card.
        let seed = (id as u32).wrapping_mul(0x9E3779B1) | 1;
        self.draw_shadow_rect(cx, screen);
        // Cream sticky-note paper.
        self.draw_item_bg_rect(cx, screen, NOTE_PAPER);
        // Wobbly pencil edge instead of a hard rect border.
        self.draw_sketch_rect_outline(
            cx,
            screen,
            1.6,
            if is_sel { SEL_BORDER } else { NOTE_PAPER_EDGE },
            seed,
            0.6,
        );
        if is_sel {
            // Second, looser outline as the selection marker.
            let outer = Rect {
                pos: screen.pos - Vec2d { x: 3.0, y: 3.0 },
                size: screen.size + Vec2d { x: 6.0, y: 6.0 },
            };
            self.draw_sketch_rect_outline(
                cx,
                outer,
                1.1,
                [SEL_BORDER[0], SEL_BORDER[1], SEL_BORDER[2], 0.6],
                seed ^ 0x5A5A,
                0.6,
            );
        }
        // Washi tape strip holding the note at the top.
        let tape_w = (screen.size.x * 0.34).clamp(30.0, 86.0);
        let tape = Rect {
            pos: Vec2d {
                x: screen.pos.x + (screen.size.x - tape_w) * 0.5,
                y: screen.pos.y - 6.0,
            },
            size: Vec2d { x: tape_w, y: 12.0 },
        };
        self.draw_item_bg_rect(cx, tape, NOTE_TAPE);
        self.draw_sketch_rect_outline(cx, tape, 0.9, NOTE_TAPE_EDGE, seed ^ 0x7E57, 0.5);

        // Title in dark handwriting ink.
        self.draw_title.color = vec4f(NOTE_TITLE_INK);
        self.draw_title
            .draw_abs(cx, screen.pos + Vec2d { x: 12.0, y: 7.0 }, title);

        // Note body text, clipped to the card content area.
        let body_rect = Rect {
            pos: screen.pos + Vec2d { x: 10.0, y: 32.0 },
            size: Vec2d {
                x: (screen.size.x - 20.0).max(1.0),
                y: (screen.size.y - 40.0).max(1.0),
            },
        };
        cx.push_clip_rect(body_rect);
        let color = NOTE_TEXT_COLORS[color_idx.min(NOTE_TEXT_COLORS.len() - 1)];
        self.draw_title.color = vec4f(color);
        // Use a smaller font size for body; scale line height accordingly.
        let font_scale = font_size / 13.0;
        self.draw_title.font_scale = font_scale;
        let line_h = font_size as f64 * 1.4;
        let display_body = if editing {
            &self.note_edit_buffer
        } else {
            body
        };
        const CHAR_W: f64 = 7.5;
        let max_chars = ((body_rect.size.x / (CHAR_W * font_scale as f64))
            .floor()
            .max(1.0) as usize)
            .max(1);
        let max_lines = (body_rect.size.y / line_h).max(1.0) as usize;

        // Soft-wrap the body to the card width.
        let mut wrapped: Vec<String> = Vec::new();
        let mut current = String::new();
        for ch in display_body.chars() {
            if ch == '\n' {
                wrapped.push(std::mem::take(&mut current));
            } else {
                if current.chars().count() >= max_chars {
                    wrapped.push(std::mem::take(&mut current));
                }
                current.push(ch);
            }
        }
        if !current.is_empty() || wrapped.is_empty() {
            wrapped.push(current);
        }
        for (i, line) in wrapped.iter().take(max_lines).enumerate() {
            let y = body_rect.pos.y + i as f64 * line_h;
            self.draw_title.draw_abs(
                cx,
                Vec2d {
                    x: body_rect.pos.x,
                    y,
                },
                line,
            );
        }

        // Blinking caret when the note is being edited inline.
        if editing {
            let mut line_idx = 0usize;
            let mut col = 0usize;
            let mut chars_on_line = 0usize;
            for (i, c) in display_body.chars().enumerate() {
                if i >= caret {
                    break;
                }
                if c == '\n' {
                    line_idx += 1;
                    col = 0;
                    chars_on_line = 0;
                } else if chars_on_line >= max_chars {
                    line_idx += 1;
                    col = 1;
                    chars_on_line = 1;
                } else {
                    col += 1;
                    chars_on_line += 1;
                }
            }
            let caret_x = body_rect.pos.x + col as f64 * CHAR_W * font_scale as f64;
            let caret_y = body_rect.pos.y + line_idx as f64 * line_h;
            let blink = (cx.cx.time() * 2.0) as i32 % 2 == 0;
            if blink {
                self.draw_cursor.color = Vec4f {
                    x: color[0],
                    y: color[1],
                    z: color[2],
                    w: 0.9,
                };
                let caret_rect = Rect {
                    pos: Vec2d {
                        x: caret_x,
                        y: caret_y,
                    },
                    size: Vec2d { x: 2.0, y: line_h },
                };
                self.draw_cursor.draw_abs(cx, caret_rect);
            }
        }
        self.draw_title.font_scale = 1.0;
        cx.pop_clip_rect();
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

    /// Clean (non-hand-drawn) polyline outline through `pts` (screen coords).
    fn draw_clean_polyline(
        &mut self,
        cx: &mut Cx2d,
        pts: &[makepad_widgets::Vec2d],
        closed: bool,
        width: f64,
        color: [f32; 4],
    ) {
        if pts.len() < 2 {
            if let Some(p) = pts.first() {
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
            return;
        }
        let seg_count = if closed { pts.len() } else { pts.len() - 1 };
        for i in 0..seg_count {
            self.draw_segment(cx, pts[i], pts[(i + 1) % pts.len()], width, color);
        }
    }

    /// Clean rectangle outline (screen coords), no wobble.
    fn draw_rect_outline(&mut self, cx: &mut Cx2d, rect: Rect, width: f64, color: [f32; 4]) {
        let x0 = rect.pos.x;
        let y0 = rect.pos.y;
        let x1 = rect.pos.x + rect.size.x;
        let y1 = rect.pos.y + rect.size.y;
        self.draw_segment(
            cx,
            Vec2d { x: x0, y: y0 },
            Vec2d { x: x1, y: y0 },
            width,
            color,
        );
        self.draw_segment(
            cx,
            Vec2d { x: x1, y: y0 },
            Vec2d { x: x1, y: y1 },
            width,
            color,
        );
        self.draw_segment(
            cx,
            Vec2d { x: x1, y: y1 },
            Vec2d { x: x0, y: y1 },
            width,
            color,
        );
        self.draw_segment(
            cx,
            Vec2d { x: x0, y: y1 },
            Vec2d { x: x0, y: y0 },
            width,
            color,
        );
    }

    /// Clean ellipse outline (screen coords), no wobble.
    fn draw_ellipse_outline(
        &mut self,
        cx: &mut Cx2d,
        center: makepad_widgets::Vec2d,
        rx: f64,
        ry: f64,
        width: f64,
        color: [f32; 4],
    ) {
        let rx = rx.max(0.5);
        let ry = ry.max(0.5);
        let n = ((rx.max(ry) * std::f64::consts::TAU) / 8.0)
            .ceil()
            .clamp(16.0, 96.0) as usize;
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let ang = (i as f64 / n as f64) * std::f64::consts::TAU;
            pts.push(makepad_widgets::Vec2d {
                x: center.x + rx * ang.cos(),
                y: center.y + ry * ang.sin(),
            });
        }
        self.draw_clean_polyline(cx, &pts, true, width, color);
    }

    /// Hand-drawn polyline through `pts` (screen coords): interior points get
    /// a perpendicular wobble and the path is stroked twice (rough.js-style
    /// double stroke). Jitter derives from `seed`, so it is frame-stable.
    #[allow(clippy::too_many_arguments)]
    fn draw_sketch_polyline(
        &mut self,
        cx: &mut Cx2d,
        pts: &[makepad_widgets::Vec2d],
        closed: bool,
        width: f64,
        color: [f32; 4],
        seed: u32,
        // Wobble scale: 1.0 = freehand canvas strokes, ~0.35 = neat UI icons.
        sloppy: f64,
    ) {
        if pts.len() < 2 {
            if let Some(p) = pts.first() {
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
            return;
        }
        let jitter = (width * 0.9).clamp(0.6, 2.5) * sloppy;
        let n = pts.len();
        for pass in 0..2u32 {
            let pseed = seed.wrapping_add(pass.wrapping_mul(0x9E3779B1));
            let (pw, alpha) = if pass == 0 {
                (width, color[3])
            } else {
                (
                    width * 0.7,
                    color[3] * (0.55 * sloppy.clamp(0.4, 1.0)) as f32,
                )
            };
            let mut jpts: Vec<makepad_widgets::Vec2d> = Vec::with_capacity(n);
            for (i, p) in pts.iter().enumerate() {
                // Keep open endpoints fixed on the main pass so strokes land
                // exactly where the user drew them.
                let fixed = !closed && pass == 0 && (i == 0 || i == n - 1);
                if fixed {
                    jpts.push(*p);
                } else {
                    jpts.push(makepad_widgets::Vec2d {
                        x: p.x + sketch_rand(pseed, i as u32 * 2) * jitter,
                        y: p.y + sketch_rand(pseed, i as u32 * 2 + 1) * jitter,
                    });
                }
            }
            let c = [color[0], color[1], color[2], alpha];
            let seg_count = if closed { n } else { n - 1 };
            for i in 0..seg_count {
                let a = jpts[i];
                let b = jpts[(i + 1) % n];
                self.draw_segment(cx, a, b, pw, c);
            }
        }
    }

    /// Hand-drawn straight stroke from `a` to `b` (screen coords):
    /// subdivided so the wobble reads as a sketchy line, not a straight edge.
    #[allow(clippy::too_many_arguments)]
    fn draw_sketch_segment(
        &mut self,
        cx: &mut Cx2d,
        a: makepad_widgets::Vec2d,
        b: makepad_widgets::Vec2d,
        width: f64,
        color: [f32; 4],
        seed: u32,
        // Wobble scale: 1.0 = freehand canvas strokes, ~0.35 = neat UI icons.
        sloppy: f64,
    ) {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let dist = dx.hypot(dy);
        if dist < 1.0 {
            self.draw_sketch_polyline(cx, &[a], false, width, color, seed, sloppy);
            return;
        }
        let n = ((dist / 14.0).ceil() as usize).clamp(2, 24);
        let mut pts = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let t = i as f64 / n as f64;
            pts.push(makepad_widgets::Vec2d {
                x: a.x + dx * t,
                y: a.y + dy * t,
            });
        }
        self.draw_sketch_polyline(cx, &pts, false, width, color, seed, sloppy);
    }

    /// Hand-drawn wobbly rectangle outline (screen coords), edge by edge.
    fn draw_sketch_rect_outline(
        &mut self,
        cx: &mut Cx2d,
        rect: Rect,
        width: f64,
        color: [f32; 4],
        seed: u32,
        // Wobble scale: 1.0 = freehand canvas strokes, ~0.35 = neat UI icons.
        sloppy: f64,
    ) {
        let x0 = rect.pos.x;
        let y0 = rect.pos.y;
        let x1 = rect.pos.x + rect.size.x;
        let y1 = rect.pos.y + rect.size.y;
        let corners = [
            makepad_widgets::Vec2d { x: x0, y: y0 },
            makepad_widgets::Vec2d { x: x1, y: y0 },
            makepad_widgets::Vec2d { x: x1, y: y1 },
            makepad_widgets::Vec2d { x: x0, y: y1 },
        ];
        for i in 0..4u32 {
            let a = corners[i as usize];
            let b = corners[((i + 1) % 4) as usize];
            self.draw_sketch_segment(cx, a, b, width, color, seed.wrapping_add(i * 7919), sloppy);
        }
    }

    /// Hand-drawn wobbly ellipse outline (screen coords) with radial wobble.
    #[allow(clippy::too_many_arguments)]
    fn draw_sketch_ellipse(
        &mut self,
        cx: &mut Cx2d,
        center: makepad_widgets::Vec2d,
        rx: f64,
        ry: f64,
        width: f64,
        color: [f32; 4],
        seed: u32,
        // Wobble scale: 1.0 = freehand canvas strokes, ~0.35 = neat UI icons.
        sloppy: f64,
    ) {
        let rx = rx.max(0.5);
        let ry = ry.max(0.5);
        let n = ((rx.max(ry) * std::f64::consts::TAU) / 10.0)
            .ceil()
            .clamp(16.0, 96.0) as usize;
        let wob_frac = ((width * 0.9).clamp(0.6, 2.5) / rx.max(ry)).min(0.2) * 1.2 * sloppy;
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let ang = (i as f64 / n as f64) * std::f64::consts::TAU;
            let rj = 1.0 + sketch_rand(seed, i as u32) * wob_frac;
            pts.push(makepad_widgets::Vec2d {
                x: center.x + rx * rj * ang.cos(),
                y: center.y + ry * rj * ang.sin(),
            });
        }
        self.draw_sketch_polyline(cx, &pts, true, width, color, seed, sloppy);
    }

    /// Short arrowhead (two strokes) at `tip`, pointing along unit (dx, dy).
    #[allow(clippy::too_many_arguments)]
    fn draw_icon_arrowhead(
        &mut self,
        cx: &mut Cx2d,
        tip: Vec2d,
        dx: f64,
        dy: f64,
        size: f64,
        w: f64,
        color: [f32; 4],
    ) {
        let px = -dy;
        let py = dx;
        let a = Vec2d {
            x: tip.x - dx * size + px * size * 0.5,
            y: tip.y - dy * size + py * size * 0.5,
        };
        let b = Vec2d {
            x: tip.x - dx * size - px * size * 0.5,
            y: tip.y - dy * size - py * size * 0.5,
        };
        self.draw_segment(cx, tip, a, w, color);
        self.draw_segment(cx, tip, b, w, color);
    }

    /// Vector tool icon drawn with hand-drawn strokes (no font glyphs —
    /// several palette glyphs rendered as tofu in the bundled font).
    fn draw_tool_icon(
        &mut self,
        cx: &mut Cx2d,
        idx: usize,
        tool: NoteTool,
        r: Rect,
        color: [f32; 4],
    ) {
        let x0 = r.pos.x + 7.0;
        let y0 = r.pos.y + 7.0;
        let x1 = r.pos.x + r.size.x - 7.0;
        let y1 = r.pos.y + r.size.y - 7.0;
        let mx = (x0 + x1) * 0.5;
        let my = (y0 + y1) * 0.5;
        let _ = idx;
        let w = 1.6;
        // Clean vector icons, no hand-drawn wobble.
        match tool {
            NoteTool::Move => {
                self.draw_segment(
                    cx,
                    Vec2d { x: mx, y: y0 + 2.0 },
                    Vec2d { x: mx, y: y1 - 2.0 },
                    w,
                    color,
                );
                self.draw_segment(
                    cx,
                    Vec2d { x: x0 + 2.0, y: my },
                    Vec2d { x: x1 - 2.0, y: my },
                    w,
                    color,
                );
                self.draw_icon_arrowhead(
                    cx,
                    Vec2d { x: mx, y: y0 + 1.0 },
                    0.0,
                    -1.0,
                    3.4,
                    w,
                    color,
                );
                self.draw_icon_arrowhead(cx, Vec2d { x: mx, y: y1 - 1.0 }, 0.0, 1.0, 3.4, w, color);
                self.draw_icon_arrowhead(
                    cx,
                    Vec2d { x: x0 + 1.0, y: my },
                    -1.0,
                    0.0,
                    3.4,
                    w,
                    color,
                );
                self.draw_icon_arrowhead(cx, Vec2d { x: x1 - 1.0, y: my }, 1.0, 0.0, 3.4, w, color);
            }
            NoteTool::Arrow => {
                let tip = Vec2d { x: x1, y: y0 };
                self.draw_segment(cx, Vec2d { x: x0, y: y1 }, tip, w, color);
                let len = (x1 - x0).hypot(y0 - y1);
                self.draw_icon_arrowhead(cx, tip, (x1 - x0) / len, (y0 - y1) / len, 4.5, w, color);
            }
            NoteTool::Pen => {
                self.draw_segment(
                    cx,
                    Vec2d {
                        x: x0 + 1.0,
                        y: y1 - 1.0,
                    },
                    Vec2d {
                        x: x1 - 2.0,
                        y: y0 + 2.0,
                    },
                    2.4,
                    color,
                );
                self.draw_segment(
                    cx,
                    Vec2d {
                        x: x1 - 6.0,
                        y: y0 + 1.0,
                    },
                    Vec2d {
                        x: x1 - 1.0,
                        y: y0 + 6.0,
                    },
                    1.2,
                    color,
                );
            }
            NoteTool::Line => {
                self.draw_segment(cx, Vec2d { x: x0, y: y1 }, Vec2d { x: x1, y: y0 }, w, color);
            }
            NoteTool::Rect => {
                self.draw_rect_outline(
                    cx,
                    Rect {
                        pos: Vec2d { x: x0, y: y0 + 1.0 },
                        size: Vec2d {
                            x: x1 - x0,
                            y: y1 - y0 - 2.0,
                        },
                    },
                    1.2,
                    color,
                );
            }
            NoteTool::Circle => {
                let rr = (x1 - x0).min(y1 - y0) * 0.5;
                self.draw_ellipse_outline(cx, Vec2d { x: mx, y: my }, rr, rr, 1.2, color);
            }
            NoteTool::Ellipse => {
                self.draw_ellipse_outline(
                    cx,
                    Vec2d { x: mx, y: my },
                    (x1 - x0) * 0.5,
                    (y1 - y0) * 0.32,
                    1.2,
                    color,
                );
            }
            NoteTool::Polyline => {
                let pts = [
                    Vec2d { x: x0, y: my + 4.0 },
                    Vec2d { x: mx - 2.0, y: y0 },
                    Vec2d { x: mx + 2.0, y: y1 },
                    Vec2d { x: x1, y: my - 4.0 },
                ];
                self.draw_clean_polyline(cx, &pts, false, w, color);
            }
            NoteTool::Text => {
                self.draw_title.color = vec4f(color);
                self.draw_title
                    .draw_abs(cx, r.pos + Vec2d { x: 9.0, y: 4.0 }, "T");
            }
            NoteTool::Eraser => {
                let ang = -0.6f64;
                let (hw, hh) = (6.0, 4.0);
                let rot = |px: f64, py: f64| Vec2d {
                    x: mx + px * ang.cos() - py * ang.sin(),
                    y: my + px * ang.sin() + py * ang.cos(),
                };
                let pts = [rot(-hw, -hh), rot(hw, -hh), rot(hw, hh), rot(-hw, hh)];
                self.draw_clean_polyline(cx, &pts, true, 1.3, color);
            }
        }
    }
    fn draw_filled_disc(
        &mut self,
        cx: &mut Cx2d,
        center: makepad_widgets::Vec2d,
        r: f64,
        color: [f32; 4],
    ) {
        let ri = r.ceil() as i32;
        for dy in -ri..=ri {
            let half = (r * r - (dy as f64) * (dy as f64)).max(0.0).sqrt();
            self.draw_item_bg_rect(
                cx,
                Rect {
                    pos: makepad_widgets::Vec2d {
                        x: center.x - half,
                        y: center.y + dy as f64,
                    },
                    size: makepad_widgets::Vec2d {
                        x: half * 2.0,
                        y: 1.0,
                    },
                },
                color,
            );
        }
    }

    /// Draw one completed or in-progress global whiteboard shape in world coords.
    fn draw_note_shape(
        &mut self,
        cx: &mut Cx2d,
        shape: &NoteShape,
        zoom: f64,
        color: [f32; 4],
        width: f64,
        seed: u32,
    ) {
        let viewport = self.world_viewport();
        let pan = self.camera.pan;
        let zoom_f = self.camera.zoom as f64;
        let to_screen = move |p: makepad_widgets::Vec2d| -> makepad_widgets::Vec2d {
            (p - pan) * zoom_f + viewport * 0.5
        };
        let w = (width * zoom).max(1.5);
        match shape {
            NoteShape::Arrow { a, b } => {
                self.draw_sketch_segment(cx, to_screen(*a), to_screen(*b), w, color, seed, 1.0);
                // Arrowhead: two short sketchy strokes near `b`.
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let len = (dx * dx + dy * dy).sqrt().max(1e-6);
                let ux = dx / len;
                let uy = dy / len;
                let h = 9.0 * zoom;
                let tip = to_screen(*b);
                let left = to_screen(makepad_widgets::Vec2d {
                    x: b.x - ux * h + uy * h * 0.5,
                    y: b.y - uy * h - ux * h * 0.5,
                });
                let right = to_screen(makepad_widgets::Vec2d {
                    x: b.x - ux * h - uy * h * 0.5,
                    y: b.y - uy * h + ux * h * 0.5,
                });
                self.draw_sketch_segment(cx, tip, left, w, color, seed.wrapping_add(101), 1.0);
                self.draw_sketch_segment(cx, tip, right, w, color, seed.wrapping_add(102), 1.0);
            }
            NoteShape::Line { a, b } => {
                self.draw_sketch_segment(
                    cx,
                    to_screen(*a),
                    to_screen(*b),
                    w,
                    color,
                    seed.wrapping_add(1),
                    1.0,
                );
            }
            NoteShape::Pen { points } | NoteShape::Polyline { points } => {
                // Freehand strokes are already organic: a single jittered
                // double-pass over the sampled points keeps the ink feel.
                let pts: Vec<makepad_widgets::Vec2d> =
                    points.iter().map(|p| to_screen(*p)).collect();
                self.draw_sketch_polyline(cx, &pts, false, w, color, seed, 1.0);
            }
            NoteShape::Rect { a, b } => {
                let sa = to_screen(*a);
                let sb = to_screen(*b);
                let rect = Rect {
                    pos: makepad_widgets::Vec2d {
                        x: sa.x.min(sb.x),
                        y: sa.y.min(sb.y),
                    },
                    size: makepad_widgets::Vec2d {
                        x: (sb.x - sa.x).abs(),
                        y: (sb.y - sa.y).abs(),
                    },
                };
                self.draw_sketch_rect_outline(cx, rect, w, color, seed, 1.0);
            }
            NoteShape::Circle { center, r } => {
                let c = to_screen(*center);
                let rr = (r * zoom).max(0.5);
                self.draw_sketch_ellipse(cx, c, rr, rr, w, color, seed, 1.0);
            }
            NoteShape::Ellipse { a, b } => {
                // Ellipse inscribed in the bounding box [a, b], drawn in
                // screen space (corners → screen, then wobbly perimeter).
                let sa = to_screen(*a);
                let sb = to_screen(*b);
                let mx = (sa.x + sb.x) * 0.5;
                let my = (sa.y + sb.y) * 0.5;
                let rx = ((sb.x - sa.x).abs() * 0.5).max(0.5);
                let ry = ((sb.y - sa.y).abs() * 0.5).max(0.5);
                self.draw_sketch_ellipse(
                    cx,
                    makepad_widgets::Vec2d { x: mx, y: my },
                    rx,
                    ry,
                    w,
                    color,
                    seed,
                    1.0,
                );
            }
            NoteShape::Text { pos, text } => {
                let p = to_screen(*pos);
                self.draw_title.color = vec4f(color);
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
        let (palette_rect, tools_y, colors_y, widths_y) = self.palette_layout();
        self.draw_item_bg_rect(cx, palette_rect, [0.13, 0.15, 0.21, 0.92]);
        // Hand-drawn wobbly frame around the palette panel.
        self.draw_sketch_rect_outline(cx, palette_rect, 1.2, [0.44, 0.50, 0.64, 0.8], 42, 0.5);

        // ── Tools ──
        for (i, t) in Self::note_tools().iter().enumerate() {
            let r = Rect {
                pos: palette_rect.pos
                    + Vec2d {
                        x: (Self::PAL_W - Self::PAL_BTN) * 0.5,
                        y: tools_y - palette_rect.pos.y
                            + i as f64 * (Self::PAL_BTN + Self::PAL_GAP),
                    },
                size: Vec2d {
                    x: Self::PAL_BTN,
                    y: Self::PAL_BTN,
                },
            };
            let active = *t == self.tool;
            if active {
                // Hand-drawn ring marks the active tool (no hard fill).
                let center = Vec2d {
                    x: r.pos.x + r.size.x * 0.5,
                    y: r.pos.y + r.size.y * 0.5,
                };
                self.draw_sketch_ellipse(
                    cx,
                    center,
                    r.size.x * 0.52,
                    r.size.y * 0.52,
                    1.4,
                    PAL_ACCENT,
                    300 + i as u32,
                    0.35,
                );
            }
            let icon_color = if active {
                PAL_ACCENT
            } else {
                [0.68, 0.72, 0.82, 1.0]
            };
            self.draw_tool_icon(cx, i, *t, r, icon_color);
        }

        // ── Color swatches ──
        let col_w = Self::PAL_SWATCH + Self::PAL_SWATCH_GAP;
        let grid_x =
            palette_rect.pos.x + (palette_rect.size.x - 2.0 * col_w + Self::PAL_SWATCH_GAP) * 0.5;
        for (i, c) in INK_COLORS.iter().enumerate() {
            let col = i % 2;
            let row = i / 2;
            let r = Rect {
                pos: Vec2d {
                    x: grid_x + col as f64 * col_w,
                    y: colors_y + row as f64 * (Self::PAL_SWATCH + Self::PAL_SWATCH_GAP),
                },
                size: Vec2d {
                    x: Self::PAL_SWATCH,
                    y: Self::PAL_SWATCH,
                },
            };
            let center = Vec2d {
                x: r.pos.x + Self::PAL_SWATCH * 0.5,
                y: r.pos.y + Self::PAL_SWATCH * 0.5,
            };
            self.draw_filled_disc(cx, center, Self::PAL_SWATCH * 0.5 - 1.0, *c);
            if i == self.ink_color_idx {
                self.draw_sketch_ellipse(
                    cx,
                    center,
                    Self::PAL_SWATCH * 0.5 + 2.0,
                    Self::PAL_SWATCH * 0.5 + 2.0,
                    1.3,
                    PAL_ACCENT,
                    400 + i as u32,
                    0.35,
                );
            } else {
                self.draw_sketch_ellipse(
                    cx,
                    center,
                    Self::PAL_SWATCH * 0.5,
                    Self::PAL_SWATCH * 0.5,
                    0.8,
                    [0.30, 0.34, 0.44, 0.6],
                    500 + i as u32,
                    0.35,
                );
            }
        }

        // ── Width buttons (sample stroke at each preset width) ──
        for (i, &wd) in INK_WIDTHS.iter().enumerate() {
            let r = Rect {
                pos: Vec2d {
                    x: palette_rect.pos.x + (palette_rect.size.x - Self::PAL_BTN) * 0.5,
                    y: widths_y + i as f64 * (Self::PAL_WIDTH_BTN_H + Self::PAL_GAP),
                },
                size: Vec2d {
                    x: Self::PAL_BTN,
                    y: Self::PAL_WIDTH_BTN_H,
                },
            };
            if i == self.ink_width_idx {
                // Hand-drawn frame marks the active width.
                self.draw_sketch_rect_outline(cx, r, 1.2, PAL_ACCENT, 600 + i as u32, 0.4);
            }
            // Sample stroke across the button, clamped to the button height,
            // drawn with the same sketchy wobble as real strokes.
            let sw = wd.min(Self::PAL_WIDTH_BTN_H - 4.0);
            let mid = r.pos.y + r.size.y * 0.5;
            self.draw_sketch_segment(
                cx,
                Vec2d {
                    x: r.pos.x + 4.0,
                    y: mid,
                },
                Vec2d {
                    x: r.pos.x + r.size.x - 4.0,
                    y: mid,
                },
                sw,
                self.ink_color(),
                700 + i as u32,
                0.9,
            );
        }
    }

    /// Draw all global whiteboard shapes (world coords) plus the in-progress
    /// stroke, clipped to the whole canvas viewport.
    fn draw_canvas_shapes(
        &mut self,
        cx: &mut Cx2d,
        viewport: Vec2d,
        shapes: &[DrawnShape],
        pending: Option<&NoteShape>,
    ) {
        let zoom = self.camera.zoom as f64;
        let ink_color = self.ink_color();
        let ink_width = self.ink_width();
        cx.push_clip_rect(Rect {
            pos: Vec2d { x: 0.0, y: 0.0 },
            size: viewport,
        });
        for ds in shapes {
            self.draw_note_shape(cx, &ds.shape, zoom, ds.color, ds.width, ds.seed);
        }
        // Move tool: highlight the shape under the cursor.
        if let Some(idx) = self.hovered_shape {
            if let Some(ds) = shapes.get(idx) {
                let hw = ds.width + 4.0 / zoom;
                self.draw_note_shape(cx, &ds.shape, zoom, [0.40, 0.70, 1.0, 0.45], hw, ds.seed);
            }
        }
        if let Some(p) = pending {
            self.draw_note_shape(cx, p, zoom, ink_color, ink_width, self.pending_seed);
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
                ItemKind::MusicPlayer => "♫",
            };
            let label = format!("{kind_label} {title}");
            self.draw_cell_text.color = vec4f(TITLE_TEXT);
            self.draw_cell_text
                .draw_abs(cx, rect.pos + Vec2d { x: 8.0, y: 6.0 }, &label);
        }
    }

    /// Draw the top workspace tab bar. Each workspace gets a tab; the active
    /// one is highlighted and a "+" button at the right creates a new space.
    fn draw_workspace_tabs(&mut self, cx: &mut Cx2d, viewport: Vec2d) {
        let bar_rect = Rect {
            pos: Vec2d { x: 0.0, y: 0.0 },
            size: Vec2d {
                x: viewport.x,
                y: TAB_BAR_H,
            },
        };
        self.draw_item_bg_rect(cx, bar_rect, TAB_BG);
        self.draw_border_rect(cx, bar_rect, TAB_BORDER);

        let mut x = 8.0;
        let tab_h = TAB_BAR_H - 8.0;
        let tab_y = 4.0;
        let n = self.workspaces.len().max(1);
        for i in 0..n {
            let label = format!("Space {}", i + 1);
            let text_w = label.chars().count() as f64 * 7.5;
            let tab_w = (text_w + 28.0).max(70.0);
            let tab_rect = Rect {
                pos: Vec2d { x, y: tab_y },
                size: Vec2d { x: tab_w, y: tab_h },
            };
            let active = i == self.current_workspace;
            self.draw_item_bg_rect(
                cx,
                tab_rect,
                if active {
                    TAB_ACTIVE_BG
                } else {
                    TAB_INACTIVE_BG
                },
            );
            self.draw_border_rect(cx, tab_rect, TAB_BORDER);
            self.draw_title.color = vec4f(TITLE_TEXT);
            self.draw_title
                .draw_abs(cx, tab_rect.pos + Vec2d { x: 12.0, y: 5.0 }, &label);
            x += tab_w + 6.0;
        }

        // Add-workspace button.
        let plus_rect = Rect {
            pos: Vec2d { x, y: tab_y },
            size: Vec2d {
                x: TAB_PLUS_W,
                y: tab_h,
            },
        };
        self.draw_item_bg_rect(cx, plus_rect, TAB_INACTIVE_BG);
        self.draw_border_rect(cx, plus_rect, TAB_BORDER);
        self.draw_title.color = vec4f(TITLE_TEXT);
        self.draw_title
            .draw_abs(cx, plus_rect.pos + Vec2d { x: 10.0, y: 5.0 }, "+");
    }

    /// Which workspace tab (or the add button) is under `screen`, if any.
    fn workspace_tab_hit(&self, screen: Vec2d, _viewport: Vec2d) -> Option<WorkspaceTabHit> {
        if screen.y < 0.0 || screen.y > TAB_BAR_H {
            return None;
        }
        let mut x = 8.0;
        let tab_h = TAB_BAR_H - 8.0;
        let tab_y = 4.0;
        let n = self.workspaces.len().max(1);
        for i in 0..n {
            let label = format!("Space {}", i + 1);
            let text_w = label.chars().count() as f64 * 7.5;
            let tab_w = (text_w + 28.0).max(70.0);
            let tab_rect = Rect {
                pos: Vec2d { x, y: tab_y },
                size: Vec2d { x: tab_w, y: tab_h },
            };
            if tab_rect.contains(screen) {
                return Some(WorkspaceTabHit::Tab(i));
            }
            x += tab_w + 6.0;
        }
        let plus_rect = Rect {
            pos: Vec2d { x, y: tab_y },
            size: Vec2d {
                x: TAB_PLUS_W,
                y: tab_h,
            },
        };
        if plus_rect.contains(screen) {
            Some(WorkspaceTabHit::Add)
        } else {
            None
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
        status: AgentStatus,
        is_sel: bool,
        state: &std::sync::Arc<std::sync::Mutex<crate::terminal::state::TerminalState>>,
    ) {
        // Drop shadow behind the card.
        self.draw_shadow_rect(cx, screen);
        // Outer glow for selected items.
        if is_sel {
            self.draw_glow_border(cx, screen, SEL_BORDER);
        }
        // Background first (DrawColor)
        self.draw_item_bg_rect(cx, screen, TERM_BG);
        // Border (DrawColor)
        self.draw_border_rect(cx, screen, if is_sel { SEL_BORDER } else { TERM_BORDER });
        // Avatar chip + title.
        let avatar_color = name_color(title);
        self.draw_avatar(
            cx,
            screen.pos + Vec2d { x: 8.0, y: 5.0 },
            title,
            avatar_color,
        );
        // Title text (DrawText) - drawn after bg/border but before content
        self.draw_title.color = vec4f(TITLE_TEXT);
        self.draw_title.draw_abs(
            cx,
            screen.pos + Vec2d { x: 32.0, y: 6.0 },
            &format!("{} — {}", title, command),
        );

        // Agent status dot + label on the right side of the title bar.
        let status_label = status.label();
        let status_color = status.color();
        let dot_size = 8.0;
        let status_x = screen.pos.x + screen.size.x - 80.0;
        let status_y = screen.pos.y + 9.0;
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: status_x,
                    y: status_y,
                },
                size: Vec2d {
                    x: dot_size,
                    y: dot_size,
                },
            },
            status_color,
        );
        self.draw_title.color = vec4f(MUSIC_SECONDARY);
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: status_x + dot_size + 5.0,
                y: status_y - 2.0,
            },
            status_label,
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
        self.draw_shadow_rect(cx, screen);
        if is_sel {
            self.draw_glow_border(cx, screen, SEL_BORDER);
        }
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
        // Browser icon avatar.
        self.draw_avatar(
            cx,
            screen.pos + Vec2d { x: 8.0, y: 5.0 },
            "W",
            [0.30, 0.62, 0.98, 1.0],
        );
        self.draw_title.color = vec4f(TITLE_TEXT);
        self.draw_title
            .draw_abs(cx, bar_rect.pos + Vec2d { x: 32.0, y: 7.0 }, url);

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

    /// Draw a CNVS-style music player card.
    ///
    /// The card has a title bar, a play/pause toggle, a progress bar, and a
    /// tiny simulated frequency visualizer (no real audio stream). Clicking
    /// the body toggles playback; the title bar can still be used to drag the
    /// card around.
    fn draw_music_player(
        &mut self,
        cx: &mut Cx2d,
        id: u64,
        screen: Rect,
        is_sel: bool,
        title: &str,
        progress: f32,
        playing: bool,
    ) {
        self.draw_shadow_rect(cx, screen);
        if is_sel {
            self.draw_glow_border(cx, screen, SEL_BORDER);
        }
        self.draw_item_bg_rect(cx, screen, MUSIC_BG);
        self.draw_border_rect(cx, screen, if is_sel { SEL_BORDER } else { MUSIC_BORDER });

        // Avatar + title.
        self.draw_avatar(cx, screen.pos + Vec2d { x: 8.0, y: 5.0 }, "♫", MUSIC_ACCENT);
        self.draw_title.color = vec4f(MUSIC_TEXT);
        self.draw_title
            .draw_abs(cx, screen.pos + Vec2d { x: 32.0, y: 6.0 }, title);

        // Body starts below the title bar.
        let body_y = screen.pos.y + 34.0;
        let body_h = (screen.size.y - 42.0).max(1.0);
        let pad = 12.0;
        let inner_x = screen.pos.x + pad;
        let inner_w = (screen.size.x - pad * 2.0).max(1.0);

        // Play / pause button.
        let btn_size = 36.0;
        let btn_rect = Rect {
            pos: Vec2d {
                x: inner_x,
                y: body_y + body_h * 0.5 - btn_size * 0.5,
            },
            size: Vec2d {
                x: btn_size,
                y: btn_size,
            },
        };
        self.draw_item_bg_rect(
            cx,
            btn_rect,
            if playing {
                MUSIC_ACCENT
            } else {
                MUSIC_PROGRESS_BG
            },
        );
        self.draw_border_rect(cx, btn_rect, MUSIC_BORDER);
        self.draw_title.color = vec4f(MUSIC_TEXT);
        let icon = if playing { "❚❚" } else { "▶" };
        self.draw_title
            .draw_abs(cx, btn_rect.pos + Vec2d { x: 10.0, y: 8.0 }, icon);

        // Progress bar to the right of the button.
        let bar_x = btn_rect.pos.x + btn_rect.size.x + 14.0;
        let bar_w = (inner_x + inner_w - bar_x - 8.0).max(1.0);
        let bar_h = 6.0;
        let bar_y = body_y + body_h * 0.5 - bar_h * 0.5 - 10.0;
        let bar_bg = Rect {
            pos: Vec2d { x: bar_x, y: bar_y },
            size: Vec2d { x: bar_w, y: bar_h },
        };
        self.draw_item_bg_rect(cx, bar_bg, MUSIC_PROGRESS_BG);
        let fill_w = (bar_w * progress as f64).max(0.0).min(bar_w);
        if fill_w > 0.0 {
            let bar_fill = Rect {
                pos: bar_bg.pos,
                size: Vec2d {
                    x: fill_w,
                    y: bar_h,
                },
            };
            self.draw_item_bg_rect(cx, bar_fill, MUSIC_ACCENT);
        }

        // Time labels.
        let total_s = 180u32;
        let cur_s = (total_s as f32 * progress) as u32;
        let fmt = |s: u32| format!("{:02}:{:02}", s / 60, s % 60);
        self.draw_title.color = vec4f(MUSIC_SECONDARY);
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: bar_x,
                y: bar_y + bar_h + 6.0,
            },
            &fmt(cur_s),
        );
        let total_label = fmt(total_s);
        let total_w = total_label.chars().count() as f64 * 6.0;
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: bar_x + bar_w - total_w,
                y: bar_y + bar_h + 6.0,
            },
            &total_label,
        );

        // Simulated frequency visualizer (small bars under the progress bar).
        let n_bars = 16usize;
        let vis_y = bar_y + bar_h + 22.0;
        let vis_h = 22.0;
        let gap = 3.0;
        let bar_w2 = (bar_w - gap * (n_bars as f64 - 1.0)) / n_bars as f64;
        let time = cx.cx.time();
        for i in 0..n_bars {
            let x = bar_x + i as f64 * (bar_w2 + gap);
            let level = if playing {
                let wave = (time * 3.0 + i as f64 * 1.1 + id as f64 * 0.13).sin();
                let wave2 = (time * 5.5 + i as f64 * 2.7 + id as f64 * 0.07).sin();
                0.25 + 0.45 * wave.abs() + 0.25 * wave2.abs()
            } else {
                0.12
            };
            let h = (vis_h * level).max(2.0);
            let r = Rect {
                pos: Vec2d {
                    x,
                    y: vis_y + vis_h - h,
                },
                size: Vec2d {
                    x: bar_w2.max(1.0),
                    y: h,
                },
            };
            self.draw_item_bg_rect(cx, r, MUSIC_BAR);
        }
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
        let input_id = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        let list_id = ids!(command_wrap.command_bar.suggestion_list);

        // Intercept command-bar navigation keys before TextInput consumes them.
        if let Event::KeyDown(key) = event {
            let suggestions_visible = self.view.view(cx, list_id).visible();
            if suggestions_visible {
                match key.key_code {
                    KeyCode::ArrowDown => {
                        self.suggestion_index += 1;
                        self.last_input_text.clear();
                        self.update_suggestions(cx);
                        return;
                    }
                    KeyCode::ArrowUp => {
                        if self.suggestion_index > 0 {
                            self.suggestion_index -= 1;
                        }
                        self.last_input_text.clear();
                        self.update_suggestions(cx);
                        return;
                    }
                    KeyCode::Tab => {
                        self.accept_suggestion(cx);
                        return;
                    }
                    KeyCode::Escape => {
                        self.view.view(cx, list_id).set_visible(cx, false);
                        return;
                    }
                    _ => {}
                }
            }
            // Ctrl+Up/Down browses command history when the command input is focused.
            let ctrl = key.modifiers.control;
            let input_focused = self.view.text_input(cx, input_id).key_focus(cx);
            if input_focused && ctrl {
                match key.key_code {
                    KeyCode::ArrowUp => {
                        if self.history_index.is_none() && !self.command_history.is_empty() {
                            self.history_index = Some(self.command_history.len() - 1);
                        } else if let Some(idx) = self.history_index {
                            if idx > 0 {
                                self.history_index = Some(idx - 1);
                            }
                        }
                        if let Some(idx) = self.history_index {
                            if let Some(h) = self.command_history.get(idx) {
                                let ti = self.view.text_input(cx, input_id);
                                ti.set_text(cx, h);
                            }
                        }
                    }
                    KeyCode::ArrowDown => {
                        if let Some(idx) = self.history_index {
                            if idx + 1 < self.command_history.len() {
                                self.history_index = Some(idx + 1);
                                if let Some(h) = self.command_history.get(idx + 1) {
                                    let ti = self.view.text_input(cx, input_id);
                                    ti.set_text(cx, h);
                                }
                            } else {
                                self.history_index = None;
                                let ti = self.view.text_input(cx, input_id);
                                ti.set_text(cx, "");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Sync the hidden note editor with the canvas-drawn buffer.
        self.sync_note_editor(cx);

        // Refresh suggestions whenever the command input text may have changed.
        self.update_suggestions(cx);

        if let Event::Timer(te) = event {
            let is_our_timer = self
                .timer
                .map(|t| t.is_timer(te).is_some())
                .unwrap_or(false);
            if is_our_timer {
                // Advance any playing music players.
                let mut changed = false;
                for item in self.items.iter_mut() {
                    if let CanvasItem::MusicPlayer {
                        progress, playing, ..
                    } = item
                    {
                        if *playing {
                            *progress += 0.001;
                            if *progress >= 1.0 {
                                *progress = 0.0;
                            }
                            changed = true;
                        }
                    }
                }
                if changed || self.poll_sessions() {
                    self.redraw(cx);
                }
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
                // If a note is being edited inline, clicking outside it commits
                // the edit; clicking on the same note keeps the editor active.
                if let Some(edit_id) = self.note_edit_id {
                    if let Some(id) = self.hit_test(me.abs) {
                        if id != edit_id {
                            self.finish_note_edit(cx);
                        } else {
                            self.selected = Some(id);
                            self.redraw(cx);
                            return;
                        }
                    } else {
                        self.finish_note_edit(cx);
                    }
                }
                // Shared double-click detection (notes, polyline, etc.).
                let now = me.time;
                let is_double = (now - self.last_click_time) < 0.4
                    && (me.abs.x - self.last_click_pos.x).hypot(me.abs.y - self.last_click_pos.y)
                        < 6.0;
                self.last_click_time = now;
                self.last_click_pos = me.abs;
                // Top workspace tab bar.
                if let Some(hit) = self.workspace_tab_hit(me.abs, self.viewport) {
                    match hit {
                        WorkspaceTabHit::Tab(i) => self.switch_workspace(cx, i),
                        WorkspaceTabHit::Add => self.add_workspace(cx),
                    }
                    return;
                }
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
                // Global whiteboard palette: clicking a tool/color/width
                // switches the active selection.
                if let Some(hit) = self.palette_hit(me.abs) {
                    match hit {
                        PaletteHit::Tool(idx) => {
                            // Switching tool commits any in-progress work.
                            if self.text_editing || self.pending.is_some() {
                                self.commit_pending();
                                self.text_editing = false;
                            }
                            self.hovered_shape = None;
                            self.tool = Self::note_tools()[idx];
                        }
                        PaletteHit::Color(idx) => {
                            self.ink_color_idx = idx;
                        }
                        PaletteHit::Width(idx) => {
                            self.ink_width_idx = idx;
                        }
                    }
                    self.redraw(cx);
                    return;
                }
                // Move tool: grab a whiteboard shape BEFORE item hit-testing,
                // since shapes are drawn on top of items and should be
                // grabbable even when overlapping a terminal or browser.
                if self.tool == NoteTool::Move {
                    if let Some(idx) = self.shape_under(me.abs) {
                        let world = self.camera.screen_to_world(me.abs, self.world_viewport());
                        self.move_drag = Some(MoveDrag {
                            shape_idx: idx,
                            grab_world: world,
                            snapshot: self.shapes.clone(),
                        });
                        self.redraw(cx);
                        return;
                    }
                }
                // Resize handle hit-test first: bottom-right corner of the
                // topmost item under the cursor.
                let resize_target = self.resize_handle_under(me.abs);
                if let Some(id) = resize_target {
                    self.selected = Some(id);
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        // Resizing a non-terminal item should not leave a
                        // previously focused terminal consuming keystrokes.
                        if item.kind() != ItemKind::Terminal {
                            self.focused_terminal = None;
                        }
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
                    // Selecting a non-terminal item must clear any stale
                    // terminal focus so the property panel keys don't leak
                    // into the shell.
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        if item.kind() != ItemKind::Terminal {
                            self.focused_terminal = None;
                        }
                    }
                    // Double-click a note to edit it inline.
                    if is_double {
                        if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                            if item.kind() == ItemKind::Note {
                                self.start_note_edit(cx, id);
                                return;
                            }
                        }
                    }
                    // Music player: clicking the body toggles play/pause; the
                    // title bar can still be used to drag the card.
                    let is_music_content =
                        if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                            if item.kind() == ItemKind::MusicPlayer {
                                let r = self
                                    .camera
                                    .world_rect_to_screen(item.world(), self.world_viewport());
                                me.abs.y > r.pos.y + 26.0
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                    if is_music_content {
                        if let Some(item) = self.items.iter_mut().find(|i| i.id() == id) {
                            if let CanvasItem::MusicPlayer { playing, .. } = item {
                                *playing = !*playing;
                                self.redraw(cx);
                            }
                        }
                    } else if let Some(item) = self.items.iter().find(|i| i.id() == id) {
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
                    // UI overlays (command bar, right panel, etc.) should stay
                    // interactive and must not clear the current selection.
                    let ui_hit = self.is_canvas_ui_hit(cx, me.abs);
                    if !ui_hit {
                        self.selected = None;
                        self.focused_terminal = None;
                        self.panning = false;
                        let world = self.camera.screen_to_world(me.abs, self.world_viewport());
                        // If a Polyline was in progress and the user switched
                        // tools (or picks Text/Eraser), commit it first.
                        if matches!(self.pending, Some(NoteShape::Polyline { .. }))
                            && self.tool != NoteTool::Polyline
                        {
                            self.commit_pending();
                        }
                        // Commit any in-progress text edit before starting a
                        // new action (clicking elsewhere finishes the text).
                        if self.text_editing {
                            self.commit_pending();
                            self.text_editing = false;
                        }

                        match self.tool {
                            NoteTool::Text => {
                                // Click on an existing text shape → edit it
                                // (pull it back into `pending`).
                                if let Some(idx) = self.text_shape_under(me.abs) {
                                    if let Some(DrawnShape { shape, seed, .. }) =
                                        self.shapes.get(idx).cloned()
                                    {
                                        self.push_undo();
                                        self.shapes.remove(idx);
                                        if let NoteShape::Text { .. } = &shape {
                                            self.pending = Some(shape);
                                            self.pending_seed = seed;
                                            self.text_editing = true;
                                        }
                                    }
                                } else {
                                    self.text_editing = true;
                                    self.pending_seed = self.fresh_seed();
                                    self.pending = Some(NoteShape::Text {
                                        pos: world,
                                        text: String::new(),
                                    });
                                }
                                // Route keyboard to the canvas so terminal
                                // keys and text editing are captured here.
                                // IME is activated from draw_walk below.
                                self.set_canvas_focus(cx);
                            }
                            NoteTool::Polyline => {
                                // Click-to-vertex: each click adds a vertex.
                                // Double-click (or Enter) finishes the polyline.
                                let now = me.time;
                                let double = (now - self.last_click_time) < 0.4
                                    && (me.abs.x - self.last_click_pos.x)
                                        .hypot(me.abs.y - self.last_click_pos.y)
                                        < 6.0;
                                self.last_click_time = now;
                                self.last_click_pos = me.abs;
                                match &mut self.pending {
                                    Some(NoteShape::Polyline { points }) => {
                                        if double {
                                            // A double-click on the last
                                            // vertex finishes the polyline.
                                            self.commit_pending();
                                        } else {
                                            points.push(world);
                                        }
                                    }
                                    _ => {
                                        self.pending_seed = self.fresh_seed();
                                        self.pending = Some(NoteShape::Polyline {
                                            points: vec![world],
                                        });
                                    }
                                }
                            }
                            NoteTool::Eraser => {
                                // Drag eraser: snapshot for undo, erase along
                                // the drag path in MouseMove.
                                self.erase_snapshot = Some(self.shapes.clone());
                                self.note_draw = Some(world);
                                let tol = 8.0 / (self.camera.zoom as f64).max(0.1);
                                let before = self.shapes.len();
                                self.shapes.retain(|sh| !sh.shape.hit(world, tol));
                                if self.shapes.len() != before {
                                    // Erased on the initial click: consume the
                                    // snapshot so MouseUp won't re-push it.
                                    if let Some(snap) = self.erase_snapshot.take() {
                                        self.undo_stack.push(snap);
                                        self.redo_stack.clear();
                                    }
                                }
                            }
                            NoteTool::Move => {
                                // Move tool is handled before item hit-testing
                                // (above); reaching here means no shape was
                                // under the cursor — nothing to do.
                            }
                            NoteTool::Pen
                            | NoteTool::Arrow
                            | NoteTool::Rect
                            | NoteTool::Circle
                            | NoteTool::Ellipse
                            | NoteTool::Line => {
                                self.note_draw = Some(world);
                                self.pending_seed = self.fresh_seed();
                                self.pending = Some(match self.tool {
                                    NoteTool::Pen => NoteShape::Pen {
                                        points: vec![world],
                                    },
                                    NoteTool::Arrow => NoteShape::Arrow { a: world, b: world },
                                    NoteTool::Rect => NoteShape::Rect { a: world, b: world },
                                    NoteTool::Circle => NoteShape::Circle {
                                        center: world,
                                        r: 0.0,
                                    },
                                    NoteTool::Ellipse => NoteShape::Ellipse { a: world, b: world },
                                    NoteTool::Line => NoteShape::Line { a: world, b: world },
                                    NoteTool::Move
                                    | NoteTool::Polyline
                                    | NoteTool::Text
                                    | NoteTool::Eraser => unreachable!(),
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
                    NoteTool::Pen => {
                        if let Some(NoteShape::Pen { points }) = pending {
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
                    NoteTool::Ellipse => {
                        *pending = Some(NoteShape::Ellipse { a: start, b: local });
                    }
                    NoteTool::Eraser => {
                        // Drag eraser: remove shapes hit along the path.
                        let tol = 8.0 / (self.camera.zoom as f64).max(0.1);
                        let before = self.shapes.len();
                        self.shapes.retain(|sh| !sh.shape.hit(local, tol));
                        if self.shapes.len() != before {
                            // First erase of this drag: commit the snapshot
                            // taken at MouseDown to the undo stack.
                            if let Some(snap) = self.erase_snapshot.take() {
                                self.undo_stack.push(snap);
                                self.redo_stack.clear();
                            }
                        }
                    }
                    NoteTool::Text | NoteTool::Polyline | NoteTool::Move => {}
                }
                self.redraw(cx);
                return;
            }
            // Move tool: translate the grabbed shape by the incremental
            // world delta. Take move_drag out of self to avoid split borrows.
            if let Some(mut md) = self.move_drag.take() {
                let local = self.camera.screen_to_world(me.abs, self.world_viewport());
                let delta = local - md.grab_world;
                if delta.x != 0.0 || delta.y != 0.0 {
                    if let Some(ds) = self.shapes.get_mut(md.shape_idx) {
                        ds.shape.translate(delta);
                    }
                    md.grab_world = local;
                    self.redraw(cx);
                }
                self.move_drag = Some(md);
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
                // Move tool: track which shape is under the cursor for
                // highlight feedback.
                let hov_shape = if self.tool == NoteTool::Move {
                    self.shape_under(me.abs)
                } else {
                    None
                };
                if hov != self.hovered
                    || hov_btn != self.hovered_btn
                    || hov_chip != self.hovered_chip
                    || hov_shape != self.hovered_shape
                {
                    self.hovered = hov;
                    self.hovered_btn = hov_btn;
                    self.hovered_chip = hov_chip;
                    self.hovered_shape = hov_shape;
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
                        // Final erase at the release point.
                        let local = self.camera.screen_to_world(me.abs, self.world_viewport());
                        let tol = 8.0 / (self.camera.zoom as f64).max(0.1);
                        let before = self.shapes.len();
                        self.shapes.retain(|sh| !sh.shape.hit(local, tol));
                        if self.shapes.len() != before {
                            if let Some(snap) = self.erase_snapshot.take() {
                                self.undo_stack.push(snap);
                                self.redo_stack.clear();
                            }
                        }
                        // Discard any unused snapshot (nothing erased this drag).
                        self.erase_snapshot = None;
                    } else {
                        self.commit_pending();
                    }
                    self.redraw(cx);
                }
                // Finalize a Move-tool drag: commit the pre-move snapshot
                // to the undo stack only if the shape actually moved.
                if let Some(md) = self.move_drag.take() {
                    let moved = self
                        .shapes
                        .get(md.shape_idx)
                        .zip(md.snapshot.get(md.shape_idx))
                        .map(|(cur, orig)| cur != orig)
                        .unwrap_or(false);
                    if moved {
                        self.undo_stack.push(md.snapshot);
                        self.redo_stack.clear();
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
            let ctrl = key.modifiers.control;
            let shift = key.modifiers.shift;
            // Inline note editing: only Escape commits; everything else
            // (typing, backspace, arrows, newlines) is handled by the focused
            // hidden TextInput widget, which we sync after view.handle_event.
            if self.note_edit_id.is_some() && key.key_code == KeyCode::Escape {
                self.finish_note_edit(cx);
                return;
            }
            // Whiteboard undo/redo — canvas-level only (no terminal focused,
            // not editing text) so Ctrl+Z still reaches a focused shell.
            if !self.text_editing && self.focused_terminal.is_none() && self.note_edit_id.is_none()
            {
                if ctrl && key.key_code == KeyCode::KeyZ {
                    if shift {
                        if self.redo() {
                            self.redraw(cx);
                        }
                    } else if self.undo() {
                        self.redraw(cx);
                    }
                    return;
                }
                if ctrl && key.key_code == KeyCode::KeyY {
                    if self.redo() {
                        self.redraw(cx);
                    }
                    return;
                }
                // Polyline click-to-vertex: Enter finishes, Escape cancels.
                if matches!(self.pending, Some(NoteShape::Polyline { .. })) {
                    match key.key_code {
                        KeyCode::ReturnKey => {
                            self.commit_pending();
                            self.redraw(cx);
                            return;
                        }
                        KeyCode::Escape => {
                            self.pending = None;
                            self.redraw(cx);
                            return;
                        }
                        _ => {}
                    }
                }
            }
            // Whiteboard text editing takes priority over terminal input.
            if self.text_editing {
                match key.key_code {
                    KeyCode::Backspace => {
                        if let Some(NoteShape::Text { text, .. }) = &mut self.pending {
                            text.pop();
                        }
                        self.redraw(cx);
                    }
                    KeyCode::ReturnKey => {
                        if shift {
                            // Shift+Return inserts a newline (multi-line text).
                            if let Some(NoteShape::Text { text, .. }) = &mut self.pending {
                                text.push('\n');
                            }
                            self.redraw(cx);
                        } else {
                            // Return commits the text shape.
                            self.commit_pending();
                            self.text_editing = false;
                            self.redraw(cx);
                        }
                    }
                    KeyCode::Escape => {
                        // Escape discards the in-progress text.
                        self.pending = None;
                        self.text_editing = false;
                        self.redraw(cx);
                    }
                    _ => {}
                }
            } else if !self.any_text_focused(cx) {
                if let Some(id) = self.focused_terminal {
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        if let Some(session) = item.session() {
                            if let Some(bytes) = self.key_to_bytes(key) {
                                session.write_bytes(&bytes);
                            }
                        }
                    }
                } else {
                    // Canvas-level keys (no terminal focused, no text widget).
                    if key.key_code == KeyCode::Space {
                        self.focus_terminal(cx, None);
                        self.redraw(cx);
                    }
                }
            }
        }

        if let Event::TextInput(te) = event {
            if self.text_editing {
                if let Some(NoteShape::Text { text, .. }) = &mut self.pending {
                    text.push_str(&te.input);
                }
                self.redraw(cx);
            } else if !self.any_text_focused(cx) {
                if let Some(id) = self.focused_terminal {
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        if let Some(session) = item.session() {
                            session.write_bytes(te.input.as_bytes());
                        }
                    }
                }
            }
        }

        // ── Properties panel ──
        // Live-apply title/body edits as the user types, not just on Return.
        let prop_title = ids!(right_panel_container.properties_panel.prop_title);
        let prop_body = ids!(right_panel_container.properties_panel.prop_body);
        if self
            .view
            .text_input(cx, prop_title)
            .changed(&actions)
            .is_some()
            || self
                .view
                .text_input(cx, prop_body)
                .changed(&actions)
                .is_some()
        {
            self.apply_properties_panel(cx);
            self.redraw(cx);
        }
        // Color swatches.
        let color_ids = [
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_color_row
                    .prop_color_0
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_color_row
                    .prop_color_1
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_color_row
                    .prop_color_2
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_color_row
                    .prop_color_3
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_color_row
                    .prop_color_4
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_color_row
                    .prop_color_5
            ),
        ];
        for (i, id) in color_ids.iter().enumerate() {
            if self.view.button(cx, *id).clicked(&actions) {
                if let Some(item_id) = self.selected {
                    if let Some(item) = self.items.iter_mut().find(|it| it.id() == item_id) {
                        if let CanvasItem::Note { color_idx, .. } = item {
                            *color_idx = i;
                            self.prop_synced = false;
                            self.redraw(cx);
                        }
                    }
                }
            }
        }
        // Font size buttons.
        let font_values = [11.0f32, 13.0, 16.0, 20.0];
        let font_ids = [
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_font_row
                    .prop_font_0
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_font_row
                    .prop_font_1
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_font_row
                    .prop_font_2
            ),
            ids!(
                right_panel_container
                    .properties_panel
                    .prop_font_row
                    .prop_font_3
            ),
        ];
        for i in 0..4 {
            if self.view.button(cx, font_ids[i]).clicked(&actions) {
                if let Some(item_id) = self.selected {
                    if let Some(item) = self.items.iter_mut().find(|it| it.id() == item_id) {
                        if let CanvasItem::Note { font_size, .. } = item {
                            *font_size = font_values[i];
                            self.prop_synced = false;
                            self.redraw(cx);
                        }
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
        if self.view.button(cx, ids!(menu_new_note)).clicked(&actions) {
            self.spawn_note(cx);
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
            let ti = self.view.text_input(
                cx,
                ids!(
                    command_wrap
                        .command_bar
                        .input_row
                        .input_capsule
                        .command_input
                ),
            );
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
        let input_id = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        if let Some((text, _mods)) = self.view.text_input(cx, input_id).returned(&actions) {
            self.push_command_history(text.clone());
            self.exec_command(cx, &text);
            let ti = self.view.text_input(cx, input_id);
            ti.set_text(cx, "");
            self.view
                .view(cx, ids!(command_wrap.command_bar.suggestion_list))
                .set_visible(cx, false);
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

        self.ensure_workspace();
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
            String,
            f32,
            usize,
            f32,
            bool,
            AgentStatus,
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
            let (body, font_size, color_idx) = match item {
                CanvasItem::Note {
                    body,
                    font_size,
                    color_idx,
                    ..
                } => (body.clone(), *font_size, *color_idx),
                _ => (String::new(), 13.0, 0),
            };
            let (progress, playing) = match item {
                CanvasItem::MusicPlayer {
                    progress, playing, ..
                } => (*progress, *playing),
                _ => (0.0f32, false),
            };
            let status = item.agent_status().unwrap_or_default();
            draw_queue.push((
                item.id(),
                item.kind(),
                screen,
                is_sel,
                item.title().to_string(),
                command,
                url,
                body,
                font_size,
                color_idx,
                progress,
                playing,
                status,
                state,
            ));
        }

        for (
            item_id,
            kind,
            item_screen,
            is_sel,
            title,
            command,
            url,
            body,
            font_size,
            color_idx,
            progress,
            playing,
            status,
            state,
        ) in draw_queue
        {
            match kind {
                ItemKind::Terminal => {
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
                        self.draw_terminal_at(
                            cx,
                            item_screen,
                            &title,
                            &command,
                            status,
                            is_sel,
                            &state,
                        );
                    }
                    self.draw_control_buttons(cx, item_id, item_screen);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
                ItemKind::Note => {
                    let editing = self.note_edit_id == Some(item_id);
                    self.draw_note_title(
                        cx,
                        item_id,
                        &title,
                        &body,
                        font_size,
                        color_idx,
                        item_screen,
                        is_sel,
                        editing,
                        self.note_edit_caret,
                    );
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
                ItemKind::MusicPlayer => {
                    self.draw_music_player(
                        cx,
                        item_id,
                        item_screen,
                        is_sel,
                        &title,
                        progress,
                        playing,
                    );
                    self.draw_control_buttons(cx, item_id, item_screen);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
            }
        }

        // Right-side properties panel reflects the current selection.
        self.sync_properties_panel(cx);

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

        // Workspace tab bar sits on top of the canvas items and dock.
        self.draw_workspace_tabs(cx, rect.size);

        // Global tool palette: fixed to the left edge, always on top.
        self.draw_tool_palette(cx);

        // Keep the platform text IME active while editing a whiteboard text
        // shape so the OS generates Event::TextInput for the canvas (which
        // holds key focus via set_canvas_focus). Inline note editing uses the
        // hidden TextInput widget, which manages its own IME.
        if self.text_editing {
            if let Some(NoteShape::Text { pos, .. }) = &self.pending {
                let screen_pos = self.camera.world_to_screen(*pos, self.world_viewport());
                cx.show_text_ime(self.area, screen_pos);
            }
            self.ime_active = true;
        } else if self.ime_active {
            cx.hide_text_ime();
            self.ime_active = false;
        }

        DrawStep::done()
    }
}
