use makepad_widgets::{
    makepad_pdf_parse::{parse_content_stream, PdfDocument},
    *,
};
use std::collections::HashMap;

use crate::camera::Camera;
use crate::command::{self, Command};
use crate::items::{
    path_file_name, AgentStatus, CanvasItem, DrawnShape, ItemKind, MediaKind, NoteShape, NoteTool,
};
use crate::note::{BlockKind, InlineStyle};
use crate::terminal::state::{default_bg, Cell};

/// A parked tab's "favicon" colour, keyed by card kind.
fn kind_accent(kind: ItemKind) -> [f32; 4] {
    match kind {
        ItemKind::Terminal => [0.42, 0.80, 0.55, 1.0],
        ItemKind::Agent => [0.72, 0.55, 0.95, 1.0],
        ItemKind::Note => [0.95, 0.75, 0.28, 1.0],
        ItemKind::Browser => [0.30, 0.62, 0.98, 1.0],
        ItemKind::MusicPlayer => [0.90, 0.39, 0.70, 1.0],
        ItemKind::Media => [0.35, 0.78, 0.80, 1.0],
    }
}

/// The kind a saved entry rebuilds as (mirrors the restore match).
fn saved_item_kind(item: &crate::persist::SavedItem) -> ItemKind {
    match item {
        crate::persist::SavedItem::Note { .. } => ItemKind::Note,
        crate::persist::SavedItem::Terminal { .. } => ItemKind::Terminal,
        crate::persist::SavedItem::Agent { .. } => ItemKind::Agent,
        crate::persist::SavedItem::Browser { .. } => ItemKind::Browser,
        crate::persist::SavedItem::MusicPlayer { .. } => ItemKind::MusicPlayer,
        crate::persist::SavedItem::Media { .. } => ItemKind::Media,
    }
}

/// One row of the command palette: what accepting it writes into the input,
/// and how the row reads (`/new terminal   New terminal card — NAME[:CWD]`).
struct PaletteRow {
    insert: String,
    label: String,
}

/// One canvas item in its saved form. Shared by the on-canvas list and the
/// parked tab strip, so a parked card persists exactly like a placed one.
fn saved_item(item: &CanvasItem) -> Option<crate::persist::SavedItem> {
    let world = item.world();
    let rect = crate::persist::SavedRect {
        pos: crate::persist::Point {
            x: world.pos.x,
            y: world.pos.y,
        },
        size: crate::persist::Point {
            x: world.size.x,
            y: world.size.y,
        },
    };
    match item {
                CanvasItem::Note {
                    title,
                    body,
                    font_size,
                    color_idx,
                    edited_ms,
                    world: _,
                    ..
                } => Some(crate::persist::SavedItem::Note {
                    title: title.clone(),
                    body: body.clone(),
                    font_size: *font_size,
                    color_idx: *color_idx,
                    edited_ms: *edited_ms,
                    rect: rect,
                }),
                CanvasItem::Terminal {
                    title, world: _, ..
                } => Some(crate::persist::SavedItem::Terminal {
                    name: title.clone(),
                    command: String::new(),
                    rect: rect,
                }),
                CanvasItem::Agent {
                    title,
                    cwd,
                    provider,
                    world: _,
                    ..
                } => {
                    let cli_session_id = item
                        .agent_session()
                        .and_then(|s| s.chat.lock().ok())
                        .and_then(|card| card.session_id.clone());
                    Some(crate::persist::SavedItem::Agent {
                        name: title.clone(),
                        cwd: cwd.clone(),
                        provider: provider.clone(),
                        cli_session_id,
                        rect: rect,
                    })
                }
                CanvasItem::Browser {
                    title,
                    url,
                    world: _,
                    ..
                } => Some(crate::persist::SavedItem::Browser {
                    title: title.clone(),
                    url: url.clone(),
                    rect: rect,
                }),
                CanvasItem::MusicPlayer {
                    title,
                    progress,
                    world: _,
                    ..
                } => Some(crate::persist::SavedItem::MusicPlayer {
                    title: title.clone(),
                    progress: *progress,
                    rect: rect,
                }),
                CanvasItem::Media {
                    title,
                    path,
                    world: _,
                    ..
                } => Some(crate::persist::SavedItem::Media {
                    title: title.clone(),
                    path: path.clone(),
                    rect: rect,
                }),
    }
}

/// The source line of the checklist hit area containing `point` on card `id`.
///
/// Split out of the click handler so the lookup is testable without a window:
/// the areas are published by the draw pass (see `CanvasPanel::note_check_rows`).
fn note_check_line_at(rows: &[(u64, Rect, usize)], id: u64, point: Vec2d) -> Option<usize> {
    rows.iter()
        .find(|(row_id, rect, _)| *row_id == id && rect.contains(point))
        .map(|(_, _, line)| *line)
}

/// One laid-out row of a note body: a source char range plus its measured runs.
///
/// Produced by [`CanvasPanel::note_rows`] and consumed by
/// [`CanvasPanel::draw_note_row`] and the caret/checkbox hit-tests.
struct NoteRow {
    kind: BlockKind,
    /// Card this row belongs to (checkbox hit areas are published per item).
    item_id: u64,
    /// Left edge of the row's text column (the card body's x).
    x: f64,
    /// Top of the row's line box.
    y: f64,
    /// Line box height; the text is centred in it.
    height: f64,
    /// Height of the face's ascender-to-descender box at this row's scale.
    text_h: f64,
    /// Face scale (relative to the 13pt note base) this row was measured at.
    /// Drawing has to use the same value or the glyphs and the reserved
    /// advance disagree — which is what left gaps after a heading.
    scale: f32,
    /// Hanging indent: the text starts this far right of `x`.
    indent: f64,
    /// Width available to the text column.
    width: f64,
    segments: Vec<NoteSegment>,
    /// Char range of this row in the source, for caret mapping.
    char_start: usize,
    char_end: usize,
    /// Source line, for checklist toggling.
    src_line: usize,
    /// First row of its block: where bullet/box/quote-bar chrome draws.
    first: bool,
}

/// A measured run inside a note row.
struct NoteSegment {
    text: String,
    style: InlineStyle,
    /// x offset from the row's text column origin.
    x: f64,
    /// Measured advance of `text` (spaces included).
    w: f64,
    /// Char index of the run's first char in the source.
    char_start: usize,
    /// Trailing spaces trimmed from `text`; they still occupy caret space.
    trailing: usize,
}

/// Grid spacing in world units.
const GRID_SIZE: f64 = 24.0;
/// Seed mono cell metrics (logical px at zoom 1) for the frames before the
/// face has been measured, and for hit-tests that run before the first draw.
/// The live numbers come from [`CanvasPanel::refresh_cell_metrics`]: the grid
/// follows the font instead of a hand-picked guess.
const CELL_W_SEED: f64 = 8.0;
const CELL_H_SEED: f64 = 17.3;

/// Image widget slots backing dropped-image previews.
const IMAGE_SLOTS: usize = 8;
/// Video widget slots backing dropped-video previews.
const VIDEO_SLOTS: usize = 2;
/// PdfView widget slots backing dropped-PDF previews.
const PDF_SLOTS: usize = 2;

/// Dimmed text used for agent card chrome when the card is not focused.
const DIM_TEXT: [f32; 4] = [0.55, 0.58, 0.66, 1.0];
/// Agent card chrome. A violet accent keeps them distinct from terminals
/// (blue) and media cards, without leaving the card palette.
const AGENT_BG: [f32; 4] = [0.11, 0.12, 0.16, 1.0];
const AGENT_BORDER: [f32; 4] = [0.28, 0.25, 0.38, 1.0];
const AGENT_ACCENT: [f32; 4] = [0.62, 0.51, 0.96, 1.0];
/// Text colours for the transcript rows.
const AGENT_ROW_USER: [f32; 4] = [0.78, 0.83, 0.99, 1.0];
const AGENT_ROW_ASSISTANT: [f32; 4] = [0.88, 0.90, 0.95, 1.0];
const AGENT_ROW_REASONING: [f32; 4] = [0.55, 0.57, 0.66, 1.0];
const AGENT_ROW_NOTICE: [f32; 4] = [0.62, 0.66, 0.74, 1.0];
const AGENT_ROW_OK: [f32; 4] = [0.42, 0.80, 0.55, 1.0];
const AGENT_ROW_FAIL: [f32; 4] = [0.95, 0.55, 0.52, 1.0];
/// Per-row line height multiplier (composer strip chrome only; transcript
/// rows use the measured cell height).
const AGENT_LINE_H: f64 = 1.45;
/// Approval buttons at the bottom of the card.
const AGENT_BTN_W: f64 = 84.0;
const AGENT_BTN_H: f64 = 26.0;
/// Height of the always-visible composer strip at the bottom of an agent
/// card (text line plus padding), reserved out of the transcript area.
const AGENT_STRIP_H: f64 = 13.0 * AGENT_LINE_H + 8.0;

/// Soft-wrap `text` at `max_chars` characters, prefixing the first line.
/// Character-count wrapping rather than measuring, matching the note card.
fn wrap(prefix: &str, text: &str, max_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::from(prefix);
    for ch in text.chars() {
        if ch == '\n' {
            out.push(std::mem::take(&mut current));
        } else {
            if current.chars().count() >= max_chars {
                out.push(std::mem::take(&mut current));
            }
            current.push(ch);
        }
    }
    out.push(current);
    out
}

/// Last path component of a workspace, for the card subtitle.
fn path_tail(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
}

/// Which CLI hosts new agent cards, from the environment.
///
/// `AGENT_CLI` picks `pi` (default), `claude` or `codex`. The card is a chat
/// view over a PTY running that CLI in its JSON mode; auth, tools and
/// session persistence all belong to the CLI, not to this app.
fn agent_cli_from_env() -> String {
    match std::env::var("AGENT_CLI").as_deref() {
        Ok("claude") => "claude".into(),
        Ok("codex") => "codex".into(),
        _ => "pi".into(),
    }
}

/// Tool palette tooltip colors, matching the MpTooltip component's bubble
/// (#1f2937 bg / #374151 border / #f9fafb text).
///
/// `approx_constant` fires on the border's blue channel because 81/255 = 0.318
/// is close to 1/π. These are colour channels, not math constants, so the lint
/// is wrong here — and as a `correctness` lint it denies by default, which
/// would otherwise fail `cargo clippy` for the whole crate.
#[allow(clippy::approx_constant)]
const TIP_BG: [f32; 4] = [0.122, 0.161, 0.216, 0.98];
#[allow(clippy::approx_constant)]
const TIP_BORDER: [f32; 4] = [0.216, 0.255, 0.318, 1.0];
#[allow(clippy::approx_constant)]
const TIP_TEXT: [f32; 4] = [0.976, 0.980, 0.965, 1.0];
/// Hover delay before the palette tooltip shows (MpTooltip show_delay).
const TIP_SHOW_DELAY: f64 = 0.3;
/// Per-file byte cap for text media previews (reads are synchronous).
const TEXT_MAX_BYTES: usize = 256 * 1024;

/// Media card chrome subtracted from the card to get the content rect
/// (2px inset each side, 32px title bar, 2px bottom rim).
const MEDIA_CHROME_W: f64 = 4.0;
const MEDIA_CHROME_H: f64 = 34.0;
/// Media card content box in world units: content is fitted (aspect kept)
/// between these bounds, so dropped images/videos spawn at their own ratio.
const MEDIA_MIN_CONTENT: Vec2d = Vec2d { x: 200.0, y: 150.0 };
const MEDIA_MAX_CONTENT: Vec2d = Vec2d { x: 560.0, y: 400.0 };

/// Fit an intrinsic pixel size into the media-card content box, keeping the
/// aspect ratio and clamping between the min/max content bounds. Extreme
/// aspects may drift slightly from the exact ratio (the image/video widgets
/// letterbox the remainder).
fn fitted_content_size(iw: f64, ih: f64) -> Option<(f64, f64)> {
    if iw <= 0.0 || ih <= 0.0 {
        return None;
    }
    let mut scale = (MEDIA_MAX_CONTENT.x / iw).min(MEDIA_MAX_CONTENT.y / ih);
    scale = scale.max((MEDIA_MIN_CONTENT.x / iw).max(MEDIA_MIN_CONTENT.y / ih));
    Some((
        (iw * scale).clamp(MEDIA_MIN_CONTENT.x, MEDIA_MAX_CONTENT.x),
        (ih * scale).clamp(MEDIA_MIN_CONTENT.y, MEDIA_MAX_CONTENT.y),
    ))
}

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
/// Top of a card's title text, as an offset from the card's top edge. Every
/// title-bar text (card title, presence label) uses this one number so they
/// cannot drift onto different baselines.
const TITLE_TEXT_DY: f64 = 6.0;
const BTN_BG: [f32; 4] = [0.16, 0.19, 0.26, 1.0];
const BTN_BORDER: [f32; 4] = [0.28, 0.32, 0.42, 1.0];
const BTN_HOVER: [f32; 4] = [0.22, 0.28, 0.40, 1.0];
const BTN_CLOSE_HOVER: [f32; 4] = [0.65, 0.25, 0.25, 1.0];

/// Parked-card tabs in the top bar (browser style): they share the bar with the
/// workspace tabs, shrinking down to `CARD_TAB_MIN_W` before the overflow clips.
const CARD_TAB_H: f64 = 24.0;
const CARD_TAB_GAP: f64 = 3.0;
const CARD_TAB_MIN_W: f64 = 76.0;
const CARD_TAB_MAX_W: f64 = 168.0;
const CARD_TAB_CLOSE: f64 = 14.0;
/// Gap either side of the divider that separates spaces from parked cards.
const CARD_TAB_DIVIDER: f64 = 10.0;
/// An inactive tab body, one step above the bar itself.
const CARD_TAB_BG: [f32; 4] = [0.17, 0.19, 0.25, 1.0];
/// Inactive tab titles read back a step, like a browser's unfocused tabs.
const TAB_TITLE_DIM: [f32; 4] = [0.66, 0.71, 0.82, 1.0];

/// The command palette's catalogue: what a row inserts, and what it does.
/// Kept beside `command::parse`'s grammar (and rendered by
/// [`CanvasPanel::palette_rows`]), so a new command is one line here.
const PALETTE_COMMANDS: &[(&str, &str)] = &[
    ("/new terminal ", "New terminal card — NAME[:CWD]"),
    ("/new note", "New note card"),
    ("/new browser ", "New browser card — URL"),
    ("/new agent ", "New agent card — NAME, a coding CLI"),
    ("/new music ", "New music player card — TITLE"),
    ("/open ", "Open a local file as an image/video/PDF card"),
    ("@", "Send text to a card — @name message"),
    ("/focus ", "Focus a terminal by name"),
    ("/rename ", "Rename a terminal — OLD NEW"),
    ("/status ", "Set a card's status badge — NAME STATUS"),
    ("/zoom ", "Set the canvas zoom — /zoom 1.5"),
    ("/grid", "Toggle the grid overlay"),
    ("/clear", "Clear the whiteboard"),
    ("/help", "List every command"),
];

/// Rows the palette can show, matching the `suggestion_0..` labels in the DSL.
const PALETTE_ROWS: usize = 14;
/// Row stride in the palette's list. `main.rs` sets the same height on each
/// suggestion label: a press is mapped back to a row by this step, so the two
/// have to stay in step (they are also why the list carries no padding).
const PALETTE_ROW_H: f64 = 24.0;

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
    /// PDF media cards only: send the file to the system print queue.
    Print,
}

/// What part of the top bar a click landed on: a workspace tab, the new-space
/// button, or one of the minimized cards parked as tabs beside them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TopBarHit {
    Workspace(usize),
    AddWorkspace,
    /// A parked card's tab: click restores it.
    Card(u64),
    /// The ✕ on a parked card's tab: closes the card (and its session).
    CardClose(u64),
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
    minimized: Vec<CanvasItem>,
    browser_spawned: Vec<u64>,
    browser_slots: Vec<(u64, usize)>,
    media_image_slots: Vec<(u64, usize)>,
    media_video_slots: Vec<(u64, usize)>,
    media_pdf_slots: Vec<(u64, usize)>,
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
    agent_composer_id: Option<u64>,
    agent_input: String,
    agent_scroll: HashMap<u64, f64>,
    /// Per-note body scroll, in world units (so it survives camera zoom).
    note_scroll: HashMap<u64, f64>,
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
            media_image_slots: Vec::new(),
            media_video_slots: Vec::new(),
            media_pdf_slots: Vec::new(),
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
            agent_composer_id: None,
            agent_input: String::new(),
            agent_scroll: HashMap::new(),
            note_scroll: HashMap::new(),
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct CanvasPanel {
    #[deref]
    view: View,
    #[rust]
    area: Area,
    #[rust]
    viewport: Vec2d,
    /// The command palette is up (⌘K). Mirrors `command_wrap`'s visibility so
    /// the input and mouse paths can ask without a widget lookup.
    #[rust]
    command_open: bool,
    /// The rows the palette is currently showing, in display order: what a
    /// press or Tab inserts, and how the row reads.
    #[rust]
    palette_rows: Vec<PaletteRow>,
    /// Where the panel's rect starts in *draw* space. The window's title bar
    /// occupies the first 32px of that space, so the walk rect is at (0, 32) and
    /// everything drawn by hand has to be offset by it — otherwise the top bar
    /// lands under the title bar and the canvas sits a title bar too high.
    #[rust]
    viewport_pos: Vec2d,
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
    /// The top-bar tab currently hovered: a workspace tab, the new-space
    /// button, or a parked card's tab (and its ✕).
    #[rust]
    hovered_tab: Option<TopBarHit>,
    /// Minimized cards, parked whole: they are the top bar's tabs. Keeping the
    /// `CanvasItem` itself (rather than a per-kind copy) means minimizing cannot
    /// lose a note's styling, a media card's kind, or a terminal's live PTY
    /// session — restore is exact, and the card re-attaches without re-spawning
    /// (which would collide on the create_only rmux session name).
    #[rust]
    minimized: Vec<CanvasItem>,
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
    /// Note body text: the regular face. The body used to borrow `draw_title`,
    /// the bold title face, which left inline bold with nothing to stand out
    /// against.
    #[live]
    draw_note_text: DrawText,
    /// Note body italics (`*emphasis*`).
    #[live]
    draw_note_italic: DrawText,
    /// Measured mono cell (`advance`, line box) in logical px at zoom 1;
    /// `None` until the first draw pass measures `draw_cell_text`'s face.
    #[rust]
    cell_metrics: Option<(f64, f64)>,
    /// Cache key for `cell_metrics` (font size + dpi factor).
    #[rust]
    cell_metrics_key: Option<(u32, u64)>,
    /// Checkbox hit areas from the last draw pass: (item id, screen rect, source
    /// line). A click has no `Cx2d`, and measuring a row needs one, so the draw
    /// pass publishes the geometry the click handler reads back.
    #[rust]
    note_check_rows: Vec<(u64, Rect, usize)>,
    /// Scrollable overflow of each note body (world units) from the last draw
    /// pass, so a wheel event knows whether the card can scroll at all before
    /// it swallows the gesture.
    #[rust]
    note_scroll_max: HashMap<u64, f64>,
    /// Per-note body scroll offset (world units). Mirrored on `Workspace` so the
    /// offset travels with its workspace, like `agent_scroll`.
    #[rust]
    note_scroll: HashMap<u64, f64>,
    /// Shrink factor for fallback glyphs wider than their cell, keyed by
    /// `(char, cells)` — 1.0 when the glyph already fits.
    #[rust]
    glyph_fit_cache: HashMap<(char, u8), f32>,
    #[live]
    draw_cursor: DrawColor,
    /// Browser page area (used to anchor the system WebView window).
    #[live]
    draw_browser_page: DrawColor,
    /// Tool palette tooltip bubble (styled after the MpTooltip component).
    #[live]
    draw_tooltip: DrawColor,
    /// Tool palette tooltip text.
    #[live]
    draw_tooltip_text: DrawText,
    #[rust]
    browser_spawned: Vec<u64>,
    /// item_id → browser slot index (0..=3) for CEF embedded browsers.
    #[rust]
    browser_slots: Vec<(u64, usize)>,
    /// item_id → image slot index (0..=7) for dropped-image previews.
    #[rust]
    media_image_slots: Vec<(u64, usize)>,
    /// item_id → video slot index (0..=1) for native video previews.
    #[rust]
    media_video_slots: Vec<(u64, usize)>,
    /// item_id → PDF slot index (0..=1) for native PdfPageView previews.
    #[rust]
    media_pdf_slots: Vec<(u64, usize)>,
    /// Parsed first page per PDF media item (keyed by item id; global so
    /// minimize/restore and workspace switches keep their parse).
    #[rust]
    pdf_pages: HashMap<u64, CachedPage>,
    /// Read text content per text media item (keyed by item id; global like
    /// pdf_pages). Capped at TEXT_MAX_BYTES per file.
    #[rust]
    text_docs: HashMap<u64, String>,
    /// Scroll offset (in lines) per text media item.
    #[rust]
    text_scroll: HashMap<u64, f64>,
    /// Tool-palette button currently hovered (drives its tooltip).
    #[rust]
    palette_hover: Option<PaletteHit>,
    /// Delay timer before the hovered button's tooltip shows (MpTooltip's
    /// 0.3s show_delay).
    #[rust]
    palette_tip_timer: Option<Timer>,
    /// Whether the delayed palette tooltip is currently shown.
    #[rust]
    palette_tip_visible: bool,
    /// Dimensions from `Event::VideoPlaybackPrepared`, FIFO-paired with the
    /// `VideoAction::PlaybackPrepared` widget actions (same child iteration
    /// order) so a prepared video slot can snap its card to the clip's aspect.
    #[rust]
    video_prep_queue: Vec<(usize, usize)>,
    /// True while dragged files hover the canvas (draws a drop highlight).
    #[rust]
    drop_hover: bool,
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
    /// Agent card whose inline composer is open, if any.
    #[rust]
    agent_composer_id: Option<u64>,
    /// Mirrors the composer proxy's text so the canvas can draw it (with the
    /// caret) while IME composition is still in flight.
    #[rust]
    agent_input: String,
    /// Set while a click that belongs to the composing card is in flight.
    /// The hidden composer is zero-size, and makepad clears key focus when a
    /// MouseUp lands outside the focused input's rect - which for an empty
    /// rect is every MouseUp, including the one that finishes the very click
    /// that started composing. When the KeyFocusLost action then arrives
    /// (actions run after all event handlers), the click is known to belong
    /// to the card and the focus is taken right back.
    #[rust]
    composer_focus_repair: bool,
    /// Same repair for the hidden note editor, which shares the mechanism.
    #[rust]
    note_focus_repair: bool,
    /// Last-saved (session_id, usage) per agent card: when a CLI reports its
    /// session id or usage changes, the canvas is re-saved so a restart
    /// resumes the right conversation with fresh metadata.
    #[rust]
    saved_chat_sig: HashMap<u64, (Option<String>, Option<String>)>,
    /// Transcript scroll offset (lines from the tail) per agent card.
    #[rust]
    agent_scroll: HashMap<u64, f64>,
    /// Whether the per-card Stop button is hovered.
    #[rust]
    hovered_stop: Option<u64>,
    /// Inline edit buffer for the note being edited.
    #[rust]
    note_edit_buffer: String,
    /// Caret position in `note_edit_buffer` (char index).
    #[rust]
    note_edit_caret: usize,
}

impl CanvasPanel {
    /// The viewport in *draw* space, which is what the camera transforms and
    /// `me.abs` share: `(p - pan) * zoom + viewport / 2` then centres on the
    /// panel's real middle instead of a title bar above it.
    fn world_viewport(&self) -> Vec2d {
        self.viewport + self.viewport_pos * 2.0
    }

    /// The panel's centre, in draw space.
    fn view_center(&self) -> Vec2d {
        self.viewport_pos + self.viewport * 0.5
    }

    /// The panel's bottom-right corner, in draw space.
    fn view_bottom_right(&self) -> Vec2d {
        self.viewport_pos + self.viewport
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
            media_image_slots: std::mem::take(&mut self.media_image_slots),
            media_video_slots: std::mem::take(&mut self.media_video_slots),
            media_pdf_slots: std::mem::take(&mut self.media_pdf_slots),
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
            agent_composer_id: self.agent_composer_id,
            agent_input: std::mem::take(&mut self.agent_input),
            agent_scroll: std::mem::take(&mut self.agent_scroll),
            note_scroll: std::mem::take(&mut self.note_scroll),
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
        self.media_image_slots = std::mem::take(&mut ws.media_image_slots);
        self.media_video_slots = std::mem::take(&mut ws.media_video_slots);
        self.media_pdf_slots = std::mem::take(&mut ws.media_pdf_slots);
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
        self.agent_composer_id = ws.agent_composer_id;
        self.agent_input = ws.agent_input;
        self.agent_scroll = ws.agent_scroll;
        self.note_scroll = ws.note_scroll;
        self.current_workspace = idx;
        // Clear transient cross-workspace interaction state.
        self.drag = None;
        self.panning = false;
        self.selecting = None;
        self.hovered = None;
        self.hovered_btn = None;
        self.hovered_tab = None;
        self.note_draw = None;
        self.text_editing = false;
        self.note_edit_id = None;
        self.note_edit_buffer.clear();
        self.note_edit_caret = 0;
        self.agent_composer_id = None;
        self.agent_input.clear();
        self.composer_focus_repair = false;
        self.note_focus_repair = false;
        self.saved_chat_sig.clear();
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
        // Every undoable mutation (shape commit, erase, clear) funnels
        // through here, so the saved canvas tracks the whiteboard too.
        self.save_canvas();
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

    /// Source line of the checklist row under `point` on card `id`, if any.
    /// Reads the hit areas the last draw pass published.
    fn note_check_hit(&self, id: u64, point: Vec2d) -> Option<usize> {
        note_check_line_at(&self.note_check_rows, id, point)
    }

    /// Replace a note's body from a canvas interaction (a checklist click, or an
    /// inline edit committing) and keep the hidden editor, the edit stamp and
    /// the saved canvas in step.
    fn set_note_body(&mut self, cx: &mut Cx, id: u64, body: String) {
        let mut changed = false;
        if let Some(item) = self.items.iter_mut().find(|i| i.id() == id) {
            if let Some(current) = item.body_mut() {
                if *current != body {
                    *current = body.clone();
                    item.set_edited_now();
                    changed = true;
                }
            }
        }
        if !changed {
            return;
        }
        if self.note_edit_id == Some(id) {
            // The hidden editor owns IME and caret state while a note is open,
            // so it has to see the same text.
            let caret = self.note_edit_caret.min(body.chars().count());
            let byte_index = body
                .char_indices()
                .nth(caret)
                .map(|(i, _)| i)
                .unwrap_or(body.len());
            self.note_edit_buffer = body.clone();
            self.note_edit_caret = caret;
            let editor = self.view.text_input(cx, ids!(note_editor));
            editor.set_text(cx, &body);
            editor.set_cursor(
                cx,
                makepad_widgets::makepad_draw::text::selection::Cursor {
                    index: byte_index,
                    prefer_next_row: false,
                },
                false,
            );
        }
        self.save_canvas();
        self.redraw(cx);
    }

    /// Enter inline-edit mode for the note `id`.
    fn start_note_edit(&mut self, cx: &mut Cx, id: u64) {
        if let Some(item) = self.items.iter().find(|i| i.id() == id) {
            if let Some(body) = item.body() {
                self.note_edit_id = Some(id);
                // Same MouseUp-clears-focus repair as the agent composer.
                self.note_focus_repair = true;
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
        self.note_focus_repair = false;
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
            // Write through `set_note_body` so the commit is stamped and saved:
            // this used to save *before* writing the buffer, so the last edit
            // only reached disk if something else saved later.
            self.set_note_body(cx, id, buffer);
            // Release the hidden editor and hand the keyboard back to the
            // canvas without changing the current selection.
            self.view.text_input(cx, ids!(note_editor)).set_text(cx, "");
            self.set_canvas_focus(cx);
            self.save_canvas();
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
            .screen_to_world(self.view_center(), self.world_viewport());
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
            edited_ms: crate::items::now_ms(),
        };
        self.next_item_id += 1;
        self.items.push(item);
        self.redraw(cx);
    }

    /// Spawn a music player card at the current view center.
    pub fn spawn_music_player(&mut self, cx: &mut Cx, title: &str) {
        let world_pos = self
            .camera
            .screen_to_world(self.view_center(), self.world_viewport());
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
        self.save_canvas();
    }

    /// Create an agent card: a terminal session whose CLI launches straight
    /// into JSON mode, rendered as a chat. The daemon types the launch line
    /// into the fresh PTY exactly as the user would.
    pub fn spawn_agent_card(&mut self, cx: &mut Cx, name: &str) {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let cli = agent_cli_from_env();
        let (world, cols, rows) = self.new_terminal_geometry();
        match crate::terminal::TerminalSession::spawn(name, "zsh", Some(&cwd), cols, rows) {
            Ok(session) => {
                session.chat_switch(&cli, true, crate::ipc::SwitchScript::FreshLaunch);
                let id = self.next_item_id;
                self.next_item_id += 1;
                self.items.push(CanvasItem::Agent {
                    id,
                    world,
                    title: name.to_owned(),
                    cwd,
                    provider: cli,
                    session: Some(Box::new(session)),
                });
                self.selected = Some(id);
                self.redraw(cx);
                self.save_canvas();
                self.status(cx, &format!("agent '{name}' ready — type to prompt it"));
            }
            Err(message) => {
                log!("canvas: failed to spawn agent '{name}': {message}");
                self.status(cx, &format!("Failed to create agent: {message}"));
            }
        }
    }

    /// Open the inline composer on agent card `id` (double-click, like notes).
    ///
    /// A zero-size TextInput proxy owns focus and IME composition while the
    /// canvas draws the line itself — the same arrangement as note editing.
    fn start_agent_composer(&mut self, cx: &mut Cx, id: u64) {
        let Some(item) = self.items.iter().find(|i| i.id() == id) else {
            return;
        };
        if item.kind() != ItemKind::Agent {
            return;
        }
        self.agent_composer_id = Some(id);
        // The MouseUp completing this click will clear the fresh focus (the
        // hidden input's rect is empty); re-take it from the action loop.
        self.composer_focus_repair = true;
        self.agent_input.clear();
        self.selected = Some(id);
        // Keys must reach the composer, not a terminal or the note editor.
        self.focused_terminal = None;
        self.note_edit_id = None;
        let composer = self.view.text_input(cx, ids!(agent_composer));
        composer.set_text(cx, "");
        // Focus is (re)taken for real at MouseUp: makepad's MouseUp rule
        // clears whatever this sets, and set_key_focus is applied lazily.
        composer.set_key_focus(cx);
        self.redraw(cx);
    }

    /// Submit the composer: a prompt when idle, a steer mid-turn.
    fn finish_agent_composer(&mut self, cx: &mut Cx) {
        let Some(id) = self.agent_composer_id.take() else {
            return;
        };
        self.composer_focus_repair = false;
        // Prefer the proxy's text so a pending IME composition is captured.
        let text = {
            let composer = self.view.text_input(cx, ids!(agent_composer));
            let text = composer.text();
            composer.set_text(cx, "");
            text
        };
        let text = if text.is_empty() {
            std::mem::take(&mut self.agent_input)
        } else {
            text
        };
        self.agent_input.clear();
        let text = text.trim().to_owned();
        if text.is_empty() {
            self.redraw(cx);
            return;
        }
        if let Some(session) = self
            .items
            .iter()
            .find(|i| i.id() == id)
            .and_then(|i| i.agent_session())
        {
            // The adapter decides the shape: a JSONL user message for
            // persistent CLIs (a mid-turn message steers), or a relaunch
            // command for per-turn ones.
            session.chat_send(&text);
            self.status(cx, "sent");
            // A sent prompt should be read at the tail again.
            self.agent_scroll.remove(&id);
        }
        // Focus returns to the canvas, matching note editing.
        self.set_canvas_focus(cx);
        self.redraw(cx);
    }

    /// Close the composer without sending (Escape).
    fn cancel_agent_composer(&mut self, cx: &mut Cx) {
        self.composer_focus_repair = false;
        if self.agent_composer_id.take().is_some() {
            self.agent_input.clear();
            self.view
                .text_input(cx, ids!(agent_composer))
                .set_text(cx, "");
            self.redraw(cx);
        }
    }

    /// The agent card's Stop button rect, pure in the card rect so drawing and
    /// hit-testing agree.
    fn agent_stop_rect(screen: Rect) -> Rect {
        Rect {
            pos: Vec2d {
                x: screen.pos.x + screen.size.x - 8.0 - 52.0,
                y: screen.pos.y + screen.size.y - 8.0 - 22.0,
            },
            size: Vec2d { x: 52.0, y: 22.0 },
        }
    }

    /// Default geometry for a new agent card.
    fn new_agent_geometry(&self) -> Rect {
        let mut world_pos = self
            .camera
            .screen_to_world(self.view_center(), self.world_viewport());
        // Cascade by the number of existing agents, like the attach path.
        let n = self
            .items
            .iter()
            .filter(|i| i.kind() == ItemKind::Agent)
            .count() as f64;
        world_pos.x += n * 32.0;
        world_pos.y += n * 26.0;
        Rect {
            pos: world_pos - Vec2d { x: 240.0, y: 200.0 },
            size: Vec2d { x: 480.0, y: 400.0 },
        }
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
    // ── canvas persistence ─────────────────────────────────────────────
    //
    // The daemon owns sessions; this file owns the *layout*. An agent card
    // is saved as identity (name + CLI + the CLI's own session id), so the
    // restore relaunches the CLI against its persisted conversation instead
    // of keeping a second copy of the transcript here.

    /// Where the canvas state is persisted.
    fn canvas_path() -> std::path::PathBuf {
        crate::persist::default_path()
    }

    /// Snapshot the current workspace into the plain-data canvas model.
    pub fn snapshot(&self) -> crate::persist::SavedCanvas {
        let items: Vec<_> = self.items.iter().filter_map(saved_item).collect();
        let minimized: Vec<_> = self.minimized.iter().filter_map(saved_item).collect();
        let shapes = self
            .shapes
            .iter()
            .map(crate::persist::SavedShape::from_shape)
            .collect();
        let mut canvas = crate::persist::SavedCanvas::empty();
        canvas.workspaces[0] = crate::persist::SavedWorkspace {
            name: format!("workspace {}", self.current_workspace),
            camera_pan: crate::persist::Point {
                x: self.camera.pan.x,
                y: self.camera.pan.y,
            },
            camera_zoom: self.camera.zoom,
            items,
            minimized,
            shapes,
        };
        canvas
    }

    /// Persist the canvas to its default path. Best-effort: a failed write
    /// is logged, never surfaced as UI noise.
    pub fn save_canvas(&self) {
        let canvas = self.snapshot();
        let path = Self::canvas_path();
        if let Err(e) = canvas.save(&path) {
            log!("canvas: save failed: {e}");
        }
    }

    /// Restore the canvas from disk: re-attach live sessions (terminal or
    /// chat view), relaunch agent CLIs against their persisted conversations
    /// when the session is gone, and rebuild plain cards. Falls back to the
    /// old behavior (fresh default terminal) when there is nothing saved.
    pub fn restore_canvas(&mut self, cx: &mut Cx) {
        let Some(saved) = crate::persist::SavedCanvas::load(&Self::canvas_path()) else {
            // No canvas on disk (first run): keep the historical default.
            match crate::terminal::TerminalSession::list_sessions() {
                Ok(infos) if infos.iter().any(|s| s.alive) => {
                    for info in infos.iter().filter(|s| s.alive) {
                        self.attach_terminal(cx, &info.name);
                    }
                }
                _ => self.spawn_terminal(cx, "claude", None, "zsh"),
            }
            return;
        };
        let ws = &saved.workspaces[0];
        let live = crate::terminal::TerminalSession::list_sessions()
            .map(|infos| {
                infos
                    .into_iter()
                    .filter(|s| s.alive)
                    .map(|s| s.name)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let to_world = |rect: &crate::persist::SavedRect| Rect {
            pos: Vec2d {
                x: rect.pos.x,
                y: rect.pos.y,
            },
            size: Vec2d {
                x: rect.size.x,
                y: rect.size.y,
            },
        };
        // Parked cards are rebuilt through exactly the same paths as placed ones
        // (session attach/relaunch included) and then moved to the tab strip, so
        // the session logic lives in one place.
        let to_restore: Vec<(&crate::persist::SavedItem, bool)> = ws
            .items
            .iter()
            .map(|item| (item, false))
            .chain(ws.minimized.iter().map(|item| (item, true)))
            .collect();
        for (item, parked) in to_restore {
            let parked_title = item.title().to_string();
            let parked_kind = saved_item_kind(item);
            match item {
                crate::persist::SavedItem::Terminal { name, rect, .. } => {
                    if live.iter().any(|n| n == name) {
                        self.attach_terminal_in(cx, name, to_world(rect));
                    }
                }
                crate::persist::SavedItem::Agent {
                    name,
                    cwd,
                    provider,
                    cli_session_id,
                    rect,
                } => {
                    let world = to_world(rect);
                    if live.iter().any(|n| n == name) {
                        // Daemon still holds it: attach and flip the parser
                        // back on (the ring replays the transcript).
                        if let Ok(session) = crate::terminal::TerminalSession::attach(name, 100, 30)
                        {
                            session.chat_switch(provider, true, crate::ipc::SwitchScript::None);
                            session.chat_sync();
                            let id = self.next_item_id;
                            self.next_item_id += 1;
                            self.items.push(CanvasItem::Agent {
                                id,
                                world,
                                title: name.clone(),
                                cwd: cwd.clone(),
                                provider: provider.clone(),
                                session: Some(Box::new(session)),
                            });
                        }
                    } else {
                        // Session gone (daemon restarted): relaunch the CLI
                        // against its persisted conversation.
                        if let Ok(session) =
                            crate::terminal::TerminalSession::spawn(name, "zsh", Some(cwd), 100, 30)
                        {
                            session.chat_switch(
                                provider,
                                true,
                                crate::ipc::SwitchScript::FreshLaunch,
                            );
                            if let Some(sid) = cli_session_id {
                                // Re-launch with the exact id so pi/claude
                                // resume the stored conversation.
                                session.write_line(
                                    &(crate::chat::CliAdapter::from_comm(provider)
                                        .launch(crate::chat::ChatMode::Chat, Some(sid))
                                        + "\n"),
                                );
                            }
                            let id = self.next_item_id;
                            self.next_item_id += 1;
                            self.items.push(CanvasItem::Agent {
                                id,
                                world,
                                title: name.clone(),
                                cwd: cwd.clone(),
                                provider: provider.clone(),
                                session: Some(Box::new(session)),
                            });
                        }
                    }
                }
                crate::persist::SavedItem::Note {
                    title,
                    body,
                    font_size,
                    color_idx,
                    edited_ms,
                    rect,
                } => {
                    let world = to_world(rect);
                    let id = self.next_item_id;
                    self.next_item_id += 1;
                    self.items.push(CanvasItem::Note {
                        id,
                        world,
                        title: title.clone(),
                        body: body.clone(),
                        font_size: *font_size,
                        color_idx: *color_idx,
                        edited_ms: *edited_ms,
                    });
                }
                crate::persist::SavedItem::Browser { title, url, rect } => {
                    let world = to_world(rect);
                    let id = self.next_item_id;
                    self.next_item_id += 1;
                    self.items.push(CanvasItem::Browser {
                        id,
                        world,
                        title: title.clone(),
                        url: url.clone(),
                    });
                }
                crate::persist::SavedItem::MusicPlayer {
                    title,
                    progress,
                    rect,
                } => {
                    let world = to_world(rect);
                    let id = self.next_item_id;
                    self.next_item_id += 1;
                    self.items.push(CanvasItem::MusicPlayer {
                        id,
                        world,
                        title: title.clone(),
                        progress: *progress,
                        playing: false,
                    });
                }
                crate::persist::SavedItem::Media { .. } => {
                    // Media cards need their file re-dropped; a stale path
                    // restoring as a dead card is worse than no card.
                }
            }
            if parked {
                self.park_restored(&parked_title, parked_kind);
            }
        }
        self.camera = crate::camera::Camera {
            pan: Vec2d {
                x: ws.camera_pan.x,
                y: ws.camera_pan.y,
            },
            zoom: ws.camera_zoom,
        };
        self.shapes = ws
            .shapes
            .iter()
            .filter_map(crate::persist::SavedShape::to_shape)
            .collect();
        self.redraw(cx);
    }

    /// Attach to a live session at a specific saved rect (no cascade).
    pub fn attach_terminal_in(&mut self, cx: &mut Cx, name: &str, world: Rect) {
        let (cols, rows) = self.term_grid_size_from(world.size.x, world.size.y);
        match crate::terminal::TerminalSession::attach(name, cols, rows) {
            Ok(session) => self.place_terminal(cx, world, session),
            Err(e) => log!("canvas: failed to attach terminal '{name}': {e}"),
        }
    }

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
            .screen_to_world(self.view_center(), self.world_viewport());
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
            .screen_to_world(self.view_center(), self.world_viewport());
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
        self.save_canvas();
    }

    /// Spawn a media card for `path` at the canvas center (the `/open`
    /// command path — drag & drop uses [`Self::drop_files`] instead).
    pub fn spawn_media(&mut self, cx: &mut Cx, path: &str, kind: MediaKind) {
        let center = self.view_center();
        self.spawn_media_at(cx, path, kind, center, 0);
    }

    /// Handle dropped file paths, creating a media card per supported file
    /// centered at the drop point (`screen`), cascading multiple files so
    /// they don't stack exactly. Unsupported extensions are reported via
    /// the status bar.
    pub fn drop_files(&mut self, cx: &mut Cx, paths: &[String], screen: Vec2d) {
        let mut opened = 0usize;
        let mut rejected: Vec<String> = Vec::new();
        for path in paths {
            match MediaKind::from_path(path) {
                Some(kind) => {
                    self.spawn_media_at(cx, path, kind, screen, opened);
                    opened += 1;
                }
                None => rejected.push(path_file_name(path).to_string()),
            }
        }
        let mut msg = match opened {
            0 => "Dropped: no previewable file".to_string(),
            1 => format!("Opened {}", path_file_name(&paths[0])),
            n => format!("Opened {n} files"),
        };
        if !rejected.is_empty() {
            let names: Vec<String> = rejected.iter().take(3).cloned().collect();
            msg.push_str(&format!(" — skipped {} (unsupported)", names.join(", ")));
        }
        self.status(cx, &msg);
    }

    /// Create a Media item centered at `screen` (window coords), offset by
    /// `nth * 24px` so multiple simultaneous drops cascade.
    fn spawn_media_at(
        &mut self,
        cx: &mut Cx,
        path: &str,
        kind: MediaKind,
        screen: Vec2d,
        nth: usize,
    ) {
        let mut world_pos = self.camera.screen_to_world(screen, self.world_viewport());
        world_pos.x += nth as f64 * 24.0;
        world_pos.y += nth as f64 * 24.0;
        let (w, h) = match kind {
            MediaKind::Image => std::fs::read(path)
                .ok()
                .and_then(|data| {
                    let p = std::path::Path::new(path);
                    image_size_by_data(&data, p)
                        .ok()
                        .and_then(|(iw, ih)| fitted_content_size(iw as f64, ih as f64))
                })
                .map(|(cw, ch)| (cw + MEDIA_CHROME_W, MEDIA_CHROME_H + ch))
                .unwrap_or_else(|| kind.default_size()),
            _ => kind.default_size(),
        };
        let id = self.next_item_id;
        let item = CanvasItem::Media {
            id,
            world: Rect {
                pos: world_pos,
                size: Vec2d { x: w, y: h },
            },
            title: path_file_name(path).to_string(),
            path: path.to_string(),
            kind,
        };
        self.next_item_id += 1;
        self.items.push(item);
        self.selected = Some(id);
        self.redraw(cx);
    }

    /// Snap a media card's size to a content aspect ratio (fitted between
    /// [`MEDIA_MIN_CONTENT`] and [`MEDIA_MAX_CONTENT`]); position is kept.
    fn resize_media_to_aspect(&mut self, cx: &mut Cx, id: u64, iw: f64, ih: f64) {
        let Some((cw, ch)) = fitted_content_size(iw, ih) else {
            return;
        };
        if let Some(item) = self.items.iter_mut().find(|i| i.id() == id) {
            item.world_mut().size = Vec2d {
                x: cw + MEDIA_CHROME_W,
                y: MEDIA_CHROME_H + ch,
            };
        }
        self.redraw(cx);
    }

    /// Screen rect of a tool-palette button (mirrors draw_tool_palette's
    /// layout; used to anchor its tooltip).
    fn palette_button_rect(&self, hit: PaletteHit) -> Option<Rect> {
        let (palette_rect, tools_y, colors_y, widths_y) = self.palette_layout();
        let in_bounds = |i: usize, n: usize| i < n;
        match hit {
            PaletteHit::Tool(i) => {
                if !in_bounds(i, Self::note_tools().len()) {
                    return None;
                }
                Some(Rect {
                    pos: Vec2d {
                        x: palette_rect.pos.x + (Self::PAL_W - Self::PAL_BTN) * 0.5,
                        y: tools_y + i as f64 * (Self::PAL_BTN + Self::PAL_GAP),
                    },
                    size: Vec2d {
                        x: Self::PAL_BTN,
                        y: Self::PAL_BTN,
                    },
                })
            }
            PaletteHit::Color(i) => {
                if !in_bounds(i, INK_COLORS.len()) {
                    return None;
                }
                let col_w = Self::PAL_SWATCH + Self::PAL_SWATCH_GAP;
                let grid_x = palette_rect.pos.x
                    + (palette_rect.size.x - 2.0 * col_w + Self::PAL_SWATCH_GAP) * 0.5;
                Some(Rect {
                    pos: Vec2d {
                        x: grid_x + (i % 2) as f64 * col_w,
                        y: colors_y + (i / 2) as f64 * (Self::PAL_SWATCH + Self::PAL_SWATCH_GAP),
                    },
                    size: Vec2d {
                        x: Self::PAL_SWATCH,
                        y: Self::PAL_SWATCH,
                    },
                })
            }
            PaletteHit::Width(i) => {
                if !in_bounds(i, INK_WIDTHS.len()) {
                    return None;
                }
                Some(Rect {
                    pos: Vec2d {
                        x: palette_rect.pos.x + (palette_rect.size.x - Self::PAL_BTN) * 0.5,
                        y: widths_y + i as f64 * (Self::PAL_WIDTH_BTN_H + Self::PAL_GAP),
                    },
                    size: Vec2d {
                        x: Self::PAL_BTN,
                        y: Self::PAL_WIDTH_BTN_H,
                    },
                })
            }
        }
    }

    /// Tooltip copy for a tool-palette button.
    fn palette_tip_text(hit: PaletteHit) -> String {
        match hit {
            PaletteHit::Tool(i) => Self::note_tools()
                .get(i)
                .map(|t| t.label().to_string())
                .unwrap_or_default(),
            PaletteHit::Color(i) => ["White", "Blue", "Green", "Yellow", "Red", "Magenta"]
                .get(i)
                .map(|n| format!("Ink: {n}"))
                .unwrap_or_default(),
            PaletteHit::Width(i) => INK_WIDTHS
                .get(i)
                .map(|w| format!("Stroke: {w:.1} px"))
                .unwrap_or_default(),
        }
    }

    /// Draw the hovered palette button's tooltip: a MpTooltip-style dark
    /// bubble to the right of the button, with a left-pointing arrow and a
    /// hand-drawn wobble to match the palette's sketch aesthetic.
    fn draw_palette_tooltip(&mut self, cx: &mut Cx2d) {
        let Some(hit) = self.palette_hover else {
            return;
        };
        if !self.palette_tip_visible {
            return;
        }
        let Some(anchor) = self.palette_button_rect(hit) else {
            return;
        };
        let text = Self::palette_tip_text(hit);
        if text.is_empty() {
            return;
        }

        // Bubble size: ~6.6px per char at 12px regular + MpTooltip padding.
        const PAD_X: f64 = 8.0;
        const TIP_H: f64 = 26.0;
        const GAP: f64 = 8.0; // MpTooltip gap + arrow depth
        let text_w = text.chars().count() as f64 * 6.6;
        let tip_w = (text_w + PAD_X * 2.0).max(40.0);
        let mut pos = Vec2d {
            x: anchor.pos.x + anchor.size.x + GAP,
            y: anchor.pos.y + (anchor.size.y - TIP_H) * 0.5,
        };
        // Clamp inside the viewport (MpTooltip's edge behavior, simplified —
        // the palette hugs the left edge so only vertical clamping bites).
        let bottom_right = self.view_bottom_right();
        pos.x = pos.x.min((bottom_right.x - tip_w - 2.0).max(2.0));
        pos.y = pos.y.max(2.0).min((bottom_right.y - TIP_H - 2.0).max(2.0));

        let tip_rect = Rect {
            pos,
            size: Vec2d { x: tip_w, y: TIP_H },
        };
        self.draw_tooltip.color = vec4f(TIP_BG);
        self.draw_tooltip.draw_abs(cx, tip_rect);

        // Left-pointing arrow from the bubble towards the button.
        let cy = (anchor.pos.y + anchor.size.y * 0.5).clamp(pos.y + 7.0, pos.y + TIP_H - 7.0);
        let ax = pos.x + 1.0;
        self.draw_sketch_polyline(
            cx,
            &[
                Vec2d {
                    x: ax + 5.0,
                    y: cy - 4.5,
                },
                Vec2d { x: ax, y: cy },
                Vec2d {
                    x: ax + 5.0,
                    y: cy + 4.5,
                },
            ],
            false,
            1.2,
            TIP_BORDER,
            900,
            0.3,
        );

        self.draw_tooltip_text.color = vec4f(TIP_TEXT);
        self.draw_tooltip_text.draw_abs(
            cx,
            pos + Vec2d {
                x: PAD_X,
                y: (TIP_H - 14.0) * 0.5,
            },
            &text,
        );
    }

    /// World-space grid for a card of `w`×`h` world units. World space is
    /// zoom-independent, so this uses the base (zoom 1) cell metrics.
    fn term_grid_size_from(&self, w: f64, h: f64) -> (usize, usize) {
        let (cell_w, cell_h) = self.cell_metrics();
        let cols = ((w - 12.0) / cell_w).floor().max(10.0) as usize;
        let rows = ((h - 34.0) / cell_h).floor().max(3.0) as usize;
        (cols, rows)
    }

    /// Monospace cell (`advance`, line box) in logical px at zoom 1, measured
    /// from the face by [`Self::refresh_cell_metrics`]; falls back to the seed
    /// until the first draw pass.
    fn cell_metrics(&self) -> (f64, f64) {
        self.cell_metrics.unwrap_or((CELL_W_SEED, CELL_H_SEED))
    }

    /// Cell width in screen px at the current camera zoom.
    fn cell_w(&self) -> f64 {
        self.cell_metrics().0 * self.camera.zoom as f64
    }

    /// Cell height in screen px at the current camera zoom.
    fn cell_h(&self) -> f64 {
        self.cell_metrics().1 * self.camera.zoom as f64
    }

    /// Measure the mono cell from the face in `draw_cell_text`, so the grid
    /// follows the font instead of a hand-picked guess — makepad's own terminal
    /// does exactly this (`apps/terminal/src/widget.rs`, `refresh_metrics`).
    ///
    /// The distinction is load-bearing: makepad sizes text in POINTS
    /// (`font_size_in_lpxs = font_size_in_pts * 96 / 72`, see
    /// `draw/src/text/layouter.rs`), while this canvas lays cards out in
    /// pixels. Reading `font_size: 12.5` as "12.5px" is what put a ~10px glyph
    /// into an 8px cell and squeezed every column into its neighbour.
    ///
    /// Measured with `font_scale = 1`, so the result is cached at zoom 1 and
    /// scaled by [`Self::cell_w`] / [`Self::cell_h`] at the call sites.
    fn refresh_cell_metrics(&mut self, cx: &mut Cx2d) {
        let font_size = self.draw_cell_text.text_style.font_size;
        let line_spacing = self.draw_cell_text.text_style.line_spacing as f64;
        let key = (font_size.to_bits(), cx.current_dpi_factor().to_bits());
        if self.cell_metrics_key == Some(key) {
            return;
        }
        let saved_scale = self.draw_cell_text.font_scale;
        self.draw_cell_text.font_scale = 1.0;
        let measured = self.draw_cell_text.prepare_single_line_run(cx, "M");
        self.draw_cell_text.font_scale = saved_scale;
        let Some(run) = measured else {
            return;
        };
        let Some(advance) = run.glyphs.first().map(|g| g.advance_in_lpxs as f64) else {
            return;
        };
        let line_box = (run.ascender_in_lpxs - run.descender_in_lpxs) as f64 * line_spacing;
        if advance <= 0.0 || line_box <= 0.0 {
            return;
        }
        if self.cell_metrics != Some((advance, line_box)) {
            log!(
                "canvas: terminal cell {advance:.2}x{line_box:.2}px (font_size {font_size:.1}pt) — grid follows the face"
            );
        }
        self.cell_metrics = Some((advance, line_box));
        self.cell_metrics_key = Some(key);
    }

    /// How much one glyph must shrink to stay inside its cell(s); 1.0 when it
    /// already fits. Only fallback faces need this (emoji and rare symbols are
    /// proportional) — a real monospace Latin face advances exactly one cell.
    /// Measured once per `(char, cells)` and cached, like makepad's terminal
    /// `cached_glyph`.
    fn glyph_fit(&mut self, cx: &mut Cx2d, ch: char, cells: u8) -> f32 {
        if let Some(fit) = self.glyph_fit_cache.get(&(ch, cells)) {
            return *fit;
        }
        let available = self.cell_metrics().0 * cells as f64;
        let saved_scale = self.draw_cell_text.font_scale;
        self.draw_cell_text.font_scale = 1.0;
        let mut buf = [0u8; 4];
        let advance = self
            .draw_cell_text
            .prepare_single_line_run(cx, ch.encode_utf8(&mut buf))
            .map(|run| run.width_in_lpxs as f64)
            .unwrap_or(0.0);
        self.draw_cell_text.font_scale = saved_scale;
        // Small rounding differences between the grid advance and the glyph's
        // own advance must not shrink normal text.
        let fit = if advance > available * 1.05 {
            (available / advance) as f32
        } else {
            1.0
        };
        self.glyph_fit_cache.insert((ch, cells), fit);
        fit
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
        // Either way the keyboard belongs to the canvas: with a card focused
        // it is forwarded to that card's shell, without one the canvas takes
        // the shortcuts.
        self.set_canvas_focus(cx);
        self.redraw(cx);
    }

    /// Find a terminal by (case-insensitive) name.
    fn find_terminal(&self, name: &str) -> Option<&CanvasItem> {
        self.items.iter().find(|i| {
            i.kind() == ItemKind::Terminal && i.title().to_lowercase() == name.to_lowercase()
        })
    }

    /// True when `screen` is inside a video/PDF card's content area (below
    /// its title bar). Clicks there belong to the embedded viewer widget
    /// (playback controls / page scrolling), not to card dragging.
    fn is_media_content(&self, id: u64, screen: Vec2d) -> bool {
        match self.items.iter().find(|i| i.id() == id) {
            Some(item)
                if matches!(
                    item.media_kind(),
                    Some(MediaKind::Video) | Some(MediaKind::Pdf)
                ) =>
            {
                let r = self
                    .camera
                    .world_rect_to_screen(item.world(), self.world_viewport());
                screen.y > r.pos.y + 26.0
            }
            _ => false,
        }
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
    /// Title-bar control button rects, right-aligned: close, minimize, and
    /// (for PDF media cards) print leftmost.
    fn control_button_rects(r: Rect, with_print: bool) -> (Option<Rect>, Rect, Rect) {
        let by = r.pos.y + 4.0;
        let close_rect = Rect {
            pos: Vec2d {
                x: r.pos.x + r.size.x - BTN_W - 4.0,
                y: by,
            },
            size: Vec2d { x: BTN_W, y: BTN_H },
        };
        let min_rect = Rect {
            pos: Vec2d {
                x: close_rect.pos.x - BTN_W - 4.0,
                y: by,
            },
            size: Vec2d { x: BTN_W, y: BTN_H },
        };
        let print_rect = if with_print {
            Some(Rect {
                pos: Vec2d {
                    x: min_rect.pos.x - BTN_W - 4.0,
                    y: by,
                },
                size: Vec2d { x: BTN_W, y: BTN_H },
            })
        } else {
            None
        };
        (print_rect, min_rect, close_rect)
    }

    /// Whether an item's title bar carries the extra print button.
    fn has_print_button(item: &CanvasItem) -> bool {
        item.media_kind() == Some(MediaKind::Pdf)
    }

    /// Topmost item whose title-bar control button is under `screen`.
    fn control_button_under(&self, screen: Vec2d) -> Option<(u64, BtnKind)> {
        self.items.iter().rev().find_map(|i| {
            let r = self.item_screen_rect(i);
            let (print_r, min_r, close_r) =
                Self::control_button_rects(r, Self::has_print_button(i));
            if let Some(pr) = print_r {
                if pr.contains(screen) {
                    return Some((i.id(), BtnKind::Print));
                }
            }
            if min_r.contains(screen) {
                Some((i.id(), BtnKind::Minimize))
            } else if close_r.contains(screen) {
                Some((i.id(), BtnKind::Close))
            } else {
                None
            }
        })
    }

    /// Flip a card between the terminal grid view and the agent chat view.
    ///
    /// The PTY session moves between the item variants untouched — the grid
    /// kept feeding the whole time, and the daemon's event ring survives a
    /// pause — so neither direction loses history. The daemon types the
    /// exit + relaunch script into the PTY (with continue flags when the
    /// CLI's own session id is unknown), exactly as a user switching by
    /// hand would.
    fn toggle_chat_view(&mut self, cx: &mut Cx, id: u64, to_chat: bool) {
        let Some(index) = self.items.iter().position(|i| i.id() == id) else {
            return;
        };
        let item = self.items.remove(index);
        let (world, title, mut session, cwd) = match item {
            CanvasItem::Terminal {
                world,
                title,
                session,
                ..
            } => (world, title, session, None),
            CanvasItem::Agent {
                world,
                title,
                session,
                cwd,
                ..
            } => (world, title, session, Some(cwd)),
            other => {
                self.items.insert(index, other);
                return;
            }
        };
        let Some(mut session) = session else {
            return;
        };
        if to_chat {
            // "auto" lets the daemon detect what the user launched here;
            // it falls back to AGENT_CLI (default pi) when nothing matches.
            session.chat_switch("auto", true, crate::ipc::SwitchScript::FromTui);
            let cli = std::env::var("AGENT_CLI").unwrap_or_else(|_| "pi".into());
            self.items.insert(
                index,
                CanvasItem::Agent {
                    id,
                    world,
                    title,
                    cwd: cwd.unwrap_or_default(),
                    provider: cli,
                    session: Some(session),
                },
            );
            self.status(cx, "switched to chat view");
        } else {
            session.chat_switch("auto", false, crate::ipc::SwitchScript::FromChat);
            self.items.insert(
                index,
                CanvasItem::Terminal {
                    id,
                    world,
                    title,
                    status: crate::items::AgentStatus::Online,
                    session: Some(session),
                },
            );
            self.status(cx, "switched to terminal view");
        }
        // Close any composer that pointed at the old variant.
        if self.agent_composer_id == Some(id) {
            self.agent_composer_id = None;
            self.agent_input.clear();
        }
        self.selected = Some(id);
        self.redraw(cx);
        self.save_canvas();
    }

    /// Draw the view-switch button: a chat bubble on terminal cards (click to
    /// read the transcript) and a shell prompt on chat cards (click to go back
    /// to the grid).
    ///
    /// Drawn as geometry, not as a glyph: the icons used to come out of
    /// `draw_cell_text`, so they inherited the *grid's* font, size and scale —
    /// and `▮` is covered by no shipped face, so it rendered as a missing-glyph
    /// box twice the size of the slot it sat in.
    fn draw_chat_switch(&mut self, cx: &mut Cx2d, screen: Rect, to_chat: bool) {
        let rect = Self::chat_switch_rect(screen);
        self.draw_item_bg_rect(cx, rect, BTN_BG);
        self.draw_border_rect(cx, rect, BTN_BORDER);
        let ink = [0.70, 0.75, 0.85, 1.0];
        // A 12×10 icon centred in the 22×18 slot.
        let icon = Vec2d {
            x: rect.pos.x + (rect.size.x - 12.0) * 0.5,
            y: rect.pos.y + (rect.size.y - 10.0) * 0.5,
        };
        if to_chat {
            // Speech bubble: outlined body with a pixel-art tail.
            self.draw_border_rect(
                cx,
                Rect {
                    pos: icon,
                    size: Vec2d { x: 12.0, y: 7.0 },
                },
                ink,
            );
            self.draw_item_bg_rect(
                cx,
                Rect {
                    pos: Vec2d {
                        x: icon.x + 2.0,
                        y: icon.y + 7.0,
                    },
                    size: Vec2d { x: 3.0, y: 3.0 },
                },
                ink,
            );
        } else {
            // Shell prompt: a `>` chevron with a trailing underscore.
            self.draw_segment(cx, icon, Vec2d { x: icon.x + 4.5, y: icon.y + 3.0 }, 1.6, ink);
            self.draw_segment(
                cx,
                Vec2d { x: icon.x + 4.5, y: icon.y + 3.0 },
                Vec2d { x: icon.x, y: icon.y + 6.0 },
                1.6,
                ink,
            );
            self.draw_segment(
                cx,
                Vec2d { x: icon.x + 7.0, y: icon.y + 8.0 },
                Vec2d { x: icon.x + 11.5, y: icon.y + 8.0 },
                1.6,
                ink,
            );
        }
    }

    /// Draw a card's presence indicator (dot + label) in the title bar, clear of
    /// every title-bar slot. Shared by terminal and chat cards so the two cannot
    /// drift apart.
    ///
    /// Pinning `draw_title.font_scale` here is load-bearing: `draw_title` is
    /// shared and its scale is set all over this file (empty-hint 1.2/1.5, note
    /// titles `font_size / 13.0`, …), so the label used to inherit whatever the
    /// previous drawer left — several times too large, on top of the buttons.
    fn draw_presence(
        &mut self,
        cx: &mut Cx2d,
        screen: Rect,
        label: &str,
        dot_color: [f32; 4],
        label_color: [f32; 4],
    ) {
        self.draw_title.font_scale = 1.0;
        // Clear the *leftmost* slot: the view-switch button sits left of
        // minimize, so reserving only the two control buttons still let the
        // label slide under it.
        let (_, min_r, _) = Self::control_button_rects(screen, true);
        let switch_r = Self::chat_switch_rect(screen);
        let slots_left = min_r.pos.x.min(switch_r.pos.x) - screen.pos.x;
        // Measure the label: `len() * 8.0` assumed an average advance the bold
        // title face does not have.
        let run = self.draw_title.prepare_single_line_run(cx, label);
        let (label_w, line_h) = match &run {
            Some(run) => (
                run.width_in_lpxs as f64,
                (run.ascender_in_lpxs - run.descender_in_lpxs) as f64,
            ),
            None => (0.0, 0.0),
        };
        let dot = 8.0;
        let gap = 5.0;
        // The dot and the gap are part of the group and were previously left
        // out of the reservation.
        let status_x = (slots_left - 14.0 - label_w - dot - gap).max(60.0);
        // The label shares the card title's text top, so both sit on one
        // baseline (they are drawn with the same `draw_title` style).
        let text_y = screen.pos.y + TITLE_TEXT_DY;
        // Centre the dot on the label's *line box* — the box runs from the text
        // top to `top + ascender - descender`, which is where the eye puts the
        // text line. The old code centred it on the control-button line
        // instead, a few pixels above the label.
        let dot_cy = text_y + line_h * 0.5;
        self.draw_filled_disc(
            cx,
            Vec2d {
                x: screen.pos.x + status_x + dot * 0.5,
                y: dot_cy,
            },
            dot * 0.5,
            dot_color,
        );
        self.draw_title.color = vec4f(label_color);
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: screen.pos.x + status_x + dot + gap,
                y: text_y,
            },
            label,
        );
    }

    /// The view-switch button rect: sits left of the control buttons, same
    /// slot geometry so the draw pass and hit-test agree.
    fn chat_switch_rect(screen: Rect) -> Rect {
        let (_, min_r, _) = Self::control_button_rects(screen, true);
        Rect {
            pos: Vec2d {
                x: min_r.pos.x - BTN_W - 4.0,
                y: min_r.pos.y,
            },
            size: Vec2d { x: BTN_W, y: BTN_H },
        }
    }

    /// The terminal/agent card whose view-switch button is under `screen`,
    /// with the view it wants next (`true` = chat).
    fn chat_switch_under(&self, screen: Vec2d) -> Option<(u64, bool)> {
        self.items.iter().rev().find_map(|item| {
            let wants_chat = match item.kind() {
                ItemKind::Terminal => true,
                ItemKind::Agent => false,
                _ => return None,
            };
            // Both variants need a live session to switch.
            let has_session = match item {
                CanvasItem::Terminal { session, .. } => session.is_some(),
                CanvasItem::Agent { session, .. } => session.is_some(),
                _ => false,
            };
            if !has_session {
                return None;
            }
            let rect = Self::chat_switch_rect(self.item_screen_rect(item));
            rect.contains(screen).then_some((item.id(), wants_chat))
        })
    }

    /// The agent Stop button under `screen`, if the agent is mid-turn.
    /// The composer strip rect, derived from the card rect with the same
    /// math as the draw pass, so clicking exactly where the input line is
    /// drawn starts typing.
    fn agent_composer_rect(screen: Rect) -> Rect {
        let body = Rect {
            pos: screen.pos + Vec2d { x: 8.0, y: 32.0 },
            size: Vec2d {
                x: (screen.size.x - 16.0).max(1.0),
                y: (screen.size.y - 40.0).max(1.0),
            },
        };
        let text_h = (body.size.y - AGENT_STRIP_H).max(1.0);
        let line_y = body.pos.y + text_h;
        Rect {
            pos: Vec2d {
                x: body.pos.x - 4.0,
                y: line_y + 2.0,
            },
            size: Vec2d {
                x: body.size.x + 8.0,
                y: AGENT_STRIP_H - 4.0,
            },
        }
    }

    fn agent_stop_under(&self, screen: Vec2d) -> Option<u64> {
        self.items.iter().rev().find_map(|item| {
            if item.kind() != ItemKind::Agent {
                return None;
            }
            let busy = item
                .agent_session()
                .and_then(|session| session.chat.lock().ok())
                .map(|card| card.busy)
                .unwrap_or(false);
            if !busy {
                return None;
            }
            let rect = Self::agent_stop_rect(self.item_screen_rect(item));
            rect.contains(screen).then_some(item.id())
        })
    }

    /// Minimize `id`: the card leaves the canvas and becomes a tab in the top
    /// bar. Nothing is copied, so restoring brings back exactly what left.
    fn minimize_item(&mut self, cx: &mut Cx, id: u64) {
        if self.minimized.iter().any(|i| i.id() == id) {
            return;
        }
        let Some(pos) = self.items.iter().position(|i| i.id() == id) else {
            return;
        };
        let item = self.items.remove(pos);
        self.minimized.push(item);
        self.selected = None;
        self.focused_terminal = None;
        self.save_canvas();
        self.redraw(cx);
    }

    /// Move a card a restore just rebuilt into the tab strip, when the canvas
    /// was saved with it parked. Matched by title and kind: terminal and agent
    /// cards are titled after their (unique) daemon session.
    fn park_restored(&mut self, title: &str, kind: ItemKind) {
        if let Some(pos) = self
            .items
            .iter()
            .rposition(|i| i.title() == title && i.kind() == kind)
        {
            let item = self.items.remove(pos);
            if self.selected == Some(item.id()) {
                self.selected = None;
            }
            if self.focused_terminal == Some(item.id()) {
                self.focused_terminal = None;
            }
            self.minimized.push(item);
        }
    }

    /// Restore `id` from the tab strip back onto the canvas.
    fn restore_item(&mut self, cx: &mut Cx, id: u64) {
        let Some(pos) = self.minimized.iter().position(|i| i.id() == id) else {
            return;
        };
        let item = self.minimized.remove(pos);
        let is_terminal = item.kind() == ItemKind::Terminal;
        self.items.push(item);
        self.selected = Some(id);
        // Restoring a non-terminal item should not leave a stale terminal focus.
        if !is_terminal {
            self.focused_terminal = None;
        }
        self.save_canvas();
        self.redraw(cx);
    }

    /// Close `id`: fully remove it from the canvas and the tab strip. Terminal
    /// sessions are killed so a closed terminal doesn't keep running in the
    /// daemon and resurrect on the next launch.
    fn close_item(&mut self, cx: &mut Cx, id: u64) {
        if let Some(item) = self.items.iter().find(|i| i.id() == id) {
            if let Some(session) = item.session() {
                session.kill();
            }
            // An agent card's session is a terminal session: `session()`
            // above found nothing (it only matches Terminal items), so kill
            // through the agent arm instead.
            if let Some(agent) = item.agent_session() {
                agent.kill();
            }
        }
        // A parked card's session is just as live as a canvas one's.
        if let Some(item) = self.minimized.iter().find(|i| i.id() == id) {
            if let Some(session) = item.session() {
                session.kill();
            }
            if let Some(agent) = item.agent_session() {
                agent.kill();
            }
        }
        self.minimized.retain(|i| i.id() != id);
        self.items.retain(|i| i.id() != id);
        self.note_scroll.remove(&id);
        self.save_canvas();
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
        // Release any preview slots the item held so later items can reuse
        // them (also fixes the same leak for CEF browser slots).
        self.browser_spawned.retain(|&i| i != id);
        self.browser_slots.retain(|(i, _)| *i != id);
        self.media_video_slots.retain(|(i, _)| *i != id);
        self.media_pdf_slots.retain(|(i, _)| *i != id);
        self.media_image_slots.retain(|(i, _)| *i != id);
        self.pdf_pages.remove(&id);
        self.text_docs.remove(&id);
        self.text_scroll.remove(&id);
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
        let char_w = self.cell_w();
        let line_h = self.cell_h();
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
        const SECT: f64 = 8.0; // extra gap between groups
        let tools_h = 10.0 * (Self::PAL_BTN + Self::PAL_GAP);
        let colors_h = 3.0 * (Self::PAL_SWATCH + Self::PAL_SWATCH_GAP);
        let widths_h = 3.0 * (Self::PAL_WIDTH_BTN_H + Self::PAL_GAP);
        let total_h = Self::PAL_PAD + tools_h + SECT + colors_h + SECT + widths_h + Self::PAL_PAD;
        let palette_rect = Rect {
            pos: Vec2d {
                x: self.viewport_pos.x + 2.0,
                y: self.viewport_pos.y + ((self.viewport.y - total_h) * 0.5).max(2.0),
            },
            size: Vec2d {
                x: Self::PAL_W,
                y: total_h,
            },
        };
        let tools_y = palette_rect.pos.y + Self::PAL_PAD;
        let colors_y = tools_y + tools_h + 8.0;
        let widths_y = colors_y + colors_h + 8.0;
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
    /// than the drawable canvas. The command palette is modal and owns every
    /// press while it is up; the right-side properties panel is UI and must
    /// remain clickable.
    fn is_canvas_ui_hit(&self, cx: &Cx, screen: Vec2d) -> bool {
        if screen.y <= TAB_BAR_H {
            return true;
        }
        // The palette is modal: while it is up, nothing behind it is a hit.
        if self.command_open {
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
            let edge = self.view_bottom_right();
            if screen.x >= edge.x - RIGHT_PANEL_W - RIGHT_PANEL_RIGHT_PAD
                && screen.x <= edge.x - RIGHT_PANEL_RIGHT_PAD
                && screen.y >= RIGHT_PANEL_TOP_PAD
                && screen.y <= edge.y - RIGHT_PANEL_BOTTOM_PAD
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
                // An agent card with that name takes the message as a prompt.
                let agent = self
                    .items
                    .iter()
                    .find(|i| {
                        i.kind() == ItemKind::Agent
                            && i.title().to_lowercase() == target.to_lowercase()
                    })
                    .map(|i| i.id());
                if let Some(id) = agent {
                    if let Some(session) = self
                        .items
                        .iter()
                        .find(|i| i.id() == id)
                        .and_then(|i| i.agent_session())
                    {
                        if text.is_empty() {
                            self.status(cx, "Say something: @name message");
                        } else {
                            session.chat_send(&text);
                            self.status(cx, &format!("sent to agent '{target}'"));
                        }
                    }
                    return;
                }
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
            Command::NewAgent { name } => {
                self.spawn_agent_card(cx, &name);
            }
            Command::OpenPath { path } => match MediaKind::from_path(&path) {
                Some(kind) => {
                    self.spawn_media(cx, &path, kind);
                    self.status(
                        cx,
                        &format!("Opened {} ({})", path_file_name(&path), kind.label()),
                    );
                }
                None => {
                    self.status(
                        cx,
                        "Unsupported file type — try images, mp4/mov/webm, pdf, or text (txt/md/code)",
                    );
                }
            },
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
                    .zoom_at(factor, self.view_center(), self.world_viewport());
                self.redraw(cx);
                self.status(cx, &format!("Zoom: {:.0}%", self.camera.zoom * 100.0));
            }
            Command::Help => {
                self.status(
                    cx,
                    "Commands: @agent text · /new agent NAME [pi|claude|codex] · /new terminal NAME · /new browser URL · /new music TITLE · /new note · /open FILE · /focus NAME · /zoom N · /grid · /clear · /help",
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
                // A selected agent takes the prompt instead of a terminal.
                let selected_agent = self
                    .items
                    .iter()
                    .find(|i| Some(i.id()) == self.selected && i.kind() == ItemKind::Agent)
                    .and_then(|i| i.agent_session().map(|c| (i.id(), c)));
                if let Some((id, session)) = selected_agent {
                    session.chat_send(&text);
                    self.status(cx, "sent");
                    let _ = id;
                    return;
                }
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
                    "Nothing focused — select an agent card, or use @name text",
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

    /// The palette's rows for the current query, in display order.
    ///
    /// An empty query lists the whole catalogue — that is the point of the
    /// palette: the commands are visible instead of remembered. A query
    /// matches a row by its template prefix or anywhere in its description, so
    /// "terminal" finds `/new terminal`; recent lines follow the catalogue.
    fn palette_rows(&self, text: &str) -> Vec<PaletteRow> {
        let query = text.trim();
        let needle = query.to_lowercase();
        let mut out: Vec<PaletteRow> = Vec::new();
        for (insert, hint) in PALETTE_COMMANDS {
            let matches = query.is_empty()
                || insert.starts_with(query)
                || hint.to_lowercase().contains(&needle);
            if matches {
                out.push(PaletteRow {
                    insert: (*insert).to_string(),
                    label: format!("{}   {hint}", insert.trim_end()),
                });
            }
        }
        if !query.is_empty() {
            for h in self.command_history.iter().rev() {
                if h.starts_with(query) && !out.iter().any(|r| r.insert == *h) {
                    out.push(PaletteRow {
                        insert: h.clone(),
                        label: format!("{h}   recent"),
                    });
                }
            }
        }
        out.truncate(PALETTE_ROWS);
        out
    }

    /// Recompute the palette's rows and repaint its labels. Called whenever the
    /// query changes, the selection moves, or the palette opens.
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
        }
        let rows = self.palette_rows(&text);
        let index = self.suggestion_index.min(rows.len().saturating_sub(1));
        self.suggestion_index = index;
        let list = self
            .view
            .view(cx, ids!(command_wrap.command_bar.suggestion_list));
        list.set_visible(cx, self.command_open && !rows.is_empty());
        for i in 0..PALETTE_ROWS {
            let id = LiveId::from_str(&format!("suggestion_{i}"));
            let label = list.label(cx, &[id]);
            match rows.get(i) {
                Some(row) => {
                    let marker = if i == index { "› " } else { "  " };
                    label.set_visible(cx, true);
                    label.set_text(cx, &format!("{marker}{}", row.label));
                }
                None => {
                    label.set_visible(cx, false);
                    label.set_text(cx, "");
                }
            }
        }
        self.palette_rows = rows;
    }

    /// Write the row at `index` into the input, leaving the palette open so
    /// arguments can follow (`/new terminal my-term`).
    fn accept_suggestion_at(&mut self, cx: &mut Cx, index: usize) {
        let Some(insert) = self.palette_rows.get(index).map(|r| r.insert.clone()) else {
            return;
        };
        let input_id = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        let ti = self.view.text_input(cx, input_id);
        ti.set_text(cx, &insert);
        ti.set_cursor(
            cx,
            makepad_widgets::makepad_draw::text::selection::Cursor {
                index: insert.len(),
                prefer_next_row: false,
            },
            true,
        );
        ti.take_key_focus(cx);
        self.last_input_text = insert;
        self.suggestion_index = index;
        self.update_suggestions(cx);
        self.redraw(cx);
    }

    /// Summon the palette (⌘K): centred over the canvas, focused, and showing
    /// the whole catalogue until a query narrows it.
    fn open_command_palette(&mut self, cx: &mut Cx) {
        if self.command_open {
            return;
        }
        self.command_open = true;
        self.view
            .view(cx, ids!(command_wrap))
            .set_visible(cx, true);
        let input_id = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        let ti = self.view.text_input(cx, input_id);
        ti.set_text(cx, "");
        ti.take_key_focus(cx);
        self.last_input_text.clear();
        self.history_index = None;
        self.suggestion_index = 0;
        self.update_suggestions(cx);
        self.redraw(cx);
    }

    /// Dismiss the palette and hand the keyboard back to the canvas (and on to
    /// whatever card is focused).
    fn close_command_palette(&mut self, cx: &mut Cx) {
        if !self.command_open {
            return;
        }
        self.command_open = false;
        self.view
            .view(cx, ids!(command_wrap))
            .set_visible(cx, false);
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
            .set_text(cx, "");
        self.last_input_text.clear();
        self.palette_rows.clear();
        // Hand the keyboard back to whoever was using it: an inline note edit
        // that is still open keeps it (with the same MouseUp re-take the edit
        // started with), otherwise it goes to the canvas.
        if self.note_edit_id.is_some() {
            self.view
                .text_input(cx, ids!(note_editor))
                .set_key_focus(cx);
            self.note_focus_repair = true;
        } else {
            self.set_canvas_focus(cx);
        }
        self.redraw(cx);
    }

    /// Put the caret in the palette's input. A press inside the panel lands
    /// here: the palette is modal, so that press never reaches the widget
    /// dispatch, and the input is already the only thing that can own it.
    ///
    /// `take_key_focus` rather than `set_key_focus`: a TextInput lives on its
    /// `draw_bg` area, and only this call focuses that one *and* lights the
    /// caret (see its own docs — the plain `Widget::set_key_focus` leaves the
    /// field typable-looking but deaf).
    fn focus_command_input(&mut self, cx: &mut Cx) {
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
            .take_key_focus(cx);
    }

    /// The palette panel's rect, in draw space (as last laid out).
    fn palette_rect(&mut self, cx: &mut Cx) -> Option<Rect> {
        let rect = self
            .view
            .view(cx, ids!(command_wrap.command_bar))
            .area()
            .rect(cx);
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return None;
        }
        Some(rect)
    }

    /// The row a press landed on, mapped back through `PALETTE_ROW_H` (the
    /// stride the DSL lays the rows out with).
    fn palette_row_under(&mut self, cx: &mut Cx, screen: Vec2d) -> Option<usize> {
        let list = self
            .view
            .view(cx, ids!(command_wrap.command_bar.suggestion_list))
            .area()
            .rect(cx);
        if list.size.x <= 0.0 || screen.x < list.pos.x || screen.x > list.pos.x + list.size.x {
            return None;
        }
        let row = ((screen.y - list.pos.y) / PALETTE_ROW_H).floor();
        if row < 0.0 {
            return None;
        }
        let index = row as usize;
        (index < self.palette_rows.len()).then_some(index)
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

    /// True if any text widget (the palette, a note editor, a properties
    /// field) currently holds key focus. While one does, keystrokes belong to
    /// it and must NOT be forwarded to a focused terminal.
    fn any_text_focused(&self, cx: &Cx) -> bool {
        let command_input = ids!(
            command_wrap
                .command_bar
                .input_row
                .input_capsule
                .command_input
        );
        (self.command_open && self.view.text_input(cx, command_input).key_focus(cx))
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
            if let Some(agent) = item.agent_session() {
                if agent.poll() {
                    changed = true;
                }
            }
        }
        changed
    }

    // ── drawing helpers ──────────────────────────────────────────────────

    /// Soft onboarding hint shown in the middle of an empty canvas (no items
    /// and no whiteboard shapes yet). Fades out automatically once the user
    /// adds anything. Centering is approximate (average glyph advance ~0.6×
    /// font size); good enough for a subtle watermark.
    fn draw_empty_hint(&mut self, cx: &mut Cx2d, rect: Rect) {
        // Hide once there's any content: on-canvas items, whiteboard shapes,
        // or cards parked as top-bar tabs.
        if !self.items.is_empty() || !self.shapes.is_empty() || !self.minimized.is_empty() {
            return;
        }
        // Center within the free area above the status strip (~114px)
        // and left of the tool palette (~44px), so the hint isn't crowded
        // against the chrome.
        let cx_pos = rect.pos.x + (rect.size.x - 44.0) * 0.5 + 44.0;
        let cy = rect.pos.y + (rect.size.y - 114.0) * 0.42;
        let title = "No items yet";
        let sub = "⌘K  or  /new terminal  /new note  /new browser";
        // Approximate glyph advance for the bold font at each scale.
        let avg = |s: f64| s * 0.60;
        // Small accent “＋” badge above the title, echoing the menu button.
        let badge_c = Vec2d {
            x: cx_pos,
            y: cy - 44.0,
        };
        self.draw_sketch_ellipse(cx, badge_c, 13.0, 13.0, 1.3, PAL_ACCENT, 777, 0.4);
        self.draw_title.font_scale = 1.2;
        self.draw_title.color = vec4f([0.98, 0.72, 0.28, 0.9]);
        self.draw_title
            .draw_abs(cx, badge_c - Vec2d { x: 4.0, y: -8.0 }, "＋");
        self.draw_title.font_scale = 1.5;
        let w1 = title.len() as f64 * avg(13.0) * 1.5;
        self.draw_title.color = vec4f([0.94, 0.96, 1.0, 0.6]);
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: cx_pos - w1 * 0.5,
                y: cy - 10.0,
            },
            title,
        );
        // Sub-hint (smaller, dimmer), below the title.
        self.draw_title.font_scale = 1.0;
        let w2 = sub.len() as f64 * avg(13.0);
        self.draw_title.color = vec4f([0.62, 0.68, 0.80, 0.5]);
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: cx_pos - w2 * 0.5,
                y: cy + 12.0,
            },
            sub,
        );
        self.draw_title.font_scale = 1.0;
    }

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
    /// DrawQuad-pixel-shader text corruption issue. The shadow is offset
    /// slightly down-right and fades out smoothly (not too harsh up close).
    fn draw_shadow_rect(&mut self, cx: &mut Cx2d, rect: Rect) {
        let offsets = [2.0, 4.0, 7.0, 11.0, 16.0, 22.0];
        let alphas = [0.10, 0.07, 0.05, 0.03, 0.02, 0.012];
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
            (16.0, 0.02),
            (11.0, 0.04),
            (7.0, 0.07),
            (3.5, 0.12),
            (1.5, 0.45),
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
        const SIZE: f64 = 20.0;
        let center = pos
            + Vec2d {
                x: SIZE * 0.5,
                y: SIZE * 0.5,
            };
        // Circular avatar chip with a soft ring.
        self.draw_filled_disc(cx, center, SIZE * 0.5, bg);
        self.draw_sketch_ellipse(
            cx,
            center,
            SIZE * 0.5 - 0.5,
            SIZE * 0.5 - 0.5,
            1.0,
            [1.0, 1.0, 1.0, 0.22],
            0,
            0.35,
        );
        let initial: String = name
            .chars()
            .filter(|c| c.is_alphabetic())
            .take(1)
            .collect::<String>()
            .to_uppercase();
        if !initial.is_empty() {
            // Fixed-size chrome (SIZE above): never scale with the camera.
            self.draw_title.font_scale = 1.0;
            self.draw_title.color = vec4f([1.0, 1.0, 1.0, 0.92]);
            self.draw_title
                .draw_abs(cx, pos + Vec2d { x: 6.0, y: 3.0 }, &initial);
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
        edited_ms: i64,
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

        // Title in dark handwriting ink. Card chrome is fixed-size, so pin the
        // scale rather than inheriting the last drawer's.
        self.draw_title.font_scale = 1.0;
        self.draw_title.color = vec4f(NOTE_TITLE_INK);
        self.draw_title
            .draw_abs(cx, screen.pos + Vec2d { x: 12.0, y: 7.0 }, title);

        // Note body, clipped to the card content area. The bottom strip is
        // reserved for the "edited …" stamp.
        let body_rect = Rect {
            pos: screen.pos + Vec2d { x: 10.0, y: 32.0 },
            size: Vec2d {
                x: (screen.size.x - 20.0).max(1.0),
                y: (screen.size.y - 52.0).max(1.0),
            },
        };
        cx.push_clip_rect(body_rect);
        let color = NOTE_TEXT_COLORS[color_idx.min(NOTE_TEXT_COLORS.len() - 1)];
        let ink_dim = [color[0] * 0.6, color[1] * 0.6, color[2] * 0.6];
        // Owned: `note_rows` needs `&mut self` for the measuring calls, so the
        // edited buffer cannot stay borrowed across them.
        let display_body: String = if editing {
            self.note_edit_buffer.clone()
        } else {
            body.to_string()
        };
        let mut rows = self.note_rows(cx, id, &display_body, body_rect, font_size);
        // In-card scrolling: a note body can be taller than its card. The offset
        // lives in world units so it survives camera zoom; the clamp comes from
        // this frame's measured content, so typing or a card resize re-clamps it.
        let zoom = (self.camera.zoom as f64).max(0.1);
        let content_h = rows
            .last()
            .map(|row| (row.y + row.height - body_rect.pos.y).max(0.0))
            .unwrap_or(0.0);
        let max_scroll = ((content_h - body_rect.size.y).max(0.0)) / zoom;
        let mut scroll = self
            .note_scroll
            .get(&id)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, max_scroll);
        // While editing, keep the caret's row on screen — by the minimum the
        // caret needs, so it never fights a deliberate scroll.
        if editing {
            if let Some(row) = rows
                .iter()
                .find(|row| caret >= row.char_start && caret <= row.char_end)
            {
                let top = row.y - body_rect.pos.y;
                let view_top = scroll * zoom;
                let view_bottom = view_top + body_rect.size.y;
                if top < view_top {
                    scroll = (top / zoom).max(0.0);
                } else if top + row.height > view_bottom {
                    scroll = (top + row.height - body_rect.size.y) / zoom;
                }
                scroll = scroll.clamp(0.0, max_scroll);
            }
        }
        if max_scroll > f64::EPSILON {
            self.note_scroll.insert(id, scroll);
            self.note_scroll_max.insert(id, max_scroll);
        } else if self.note_scroll.remove(&id).is_some() {
            // Content shrank (or the card grew): forget the offset.
        }
        let shift = scroll * zoom;
        for row in rows.iter_mut() {
            row.y -= shift;
        }

        for row in rows.iter() {
            self.draw_note_row(cx, row, color, font_size);
        }
        // Scroll indicator: a thin thumb on the body's right edge, only when
        // there is something to scroll.
        if max_scroll > f64::EPSILON {
            let track = body_rect.size.y;
            let thumb = (track * (track / content_h).clamp(0.08, 1.0)).max(18.0);
            let t = (scroll / max_scroll).clamp(0.0, 1.0);
            self.draw_item_bg_rect(
                cx,
                Rect {
                    pos: Vec2d {
                        x: body_rect.pos.x + body_rect.size.x - 3.0,
                        y: body_rect.pos.y + (track - thumb) * t,
                    },
                    size: Vec2d { x: 2.5, y: thumb },
                },
                [ink_dim[0], ink_dim[1], ink_dim[2], 0.35],
            );
        }
        // Blinking caret, placed by the same measured layout the rows came from.
        if editing {
            if let Some((caret_x, caret_y, caret_h)) =
                self.note_caret_pos(cx, &rows, caret)
            {
                let blink = (cx.cx.time() * 2.0) as i32 % 2 == 0;
                if blink {
                    self.draw_cursor.color = Vec4f {
                        x: color[0],
                        y: color[1],
                        z: color[2],
                        w: 0.9,
                    };
                    self.draw_cursor.draw_abs(
                        cx,
                        Rect {
                            pos: Vec2d {
                                x: caret_x,
                                y: caret_y,
                            },
                            size: Vec2d {
                                x: 2.0,
                                y: caret_h,
                            },
                        },
                    );
                }
            }
        }
        cx.pop_clip_rect();

        // "edited 14:32" in the footer, dim: notes' list rows carry the same
        // stamp under each preview.
        let stamp = crate::note::format_edited(edited_ms, crate::items::now_ms());
        if std::env::var_os("CANVAS_TRACE_NOTE").is_some() {
            log!(
                "note stamp {:?} edited_ms={edited_ms} card=({:.1},{:.1} {:.1}x{:.1})",
                stamp,
                screen.pos.x,
                screen.pos.y,
                screen.size.x,
                screen.size.y
            );
        }
        if !stamp.is_empty() {
            self.draw_note_text.font_scale = font_size / 13.0 * 0.82;
            self.draw_note_text.color = vec4f([
                NOTE_TITLE_INK[0] * 0.85,
                NOTE_TITLE_INK[1] * 0.85,
                NOTE_TITLE_INK[2] * 0.85,
                0.55,
            ]);
            let text = format!("edited {stamp}");
            // Left-aligned under the body, like notes' row metadata, which also
            // keeps it clear of the resize handle in the bottom-right corner.
            self.draw_note_text.draw_abs(
                cx,
                Vec2d {
                    x: screen.pos.x + 12.0,
                    y: screen.pos.y + screen.size.y - 20.0,
                },
                &text,
            );
            self.draw_note_text.font_scale = 1.0;
        }
    }

    /// Width of `text` in one of the note's inline faces, at `scale`.
    ///
    /// `scale` is relative to the note body base (13pt). `code` spans are drawn
    /// in the cell face, whose base is 12.5pt, so they are rescaled to land on
    /// the same point size as the prose.
    fn note_text_width(&mut self, cx: &mut Cx2d, style: InlineStyle, text: &str, scale: f32) -> f64 {
        let scale = if style == InlineStyle::Code {
            scale * 13.0 / 12.5
        } else {
            scale
        };
        let draw = match style {
            InlineStyle::Plain => &mut self.draw_note_text,
            InlineStyle::Bold => &mut self.draw_title,
            InlineStyle::Italic => &mut self.draw_note_italic,
            InlineStyle::Code => &mut self.draw_cell_text,
        };
        let saved = draw.font_scale;
        draw.font_scale = scale;
        let width = draw
            .prepare_single_line_run(cx, text)
            .map(|run| run.width_in_lpxs as f64)
            .unwrap_or(0.0);
        draw.font_scale = saved;
        width
    }

    /// Ascender-to-descender height of the prose face at `scale` (the line box a
    /// row's text is centred in).
    fn note_line_box(&mut self, cx: &mut Cx2d, scale: f32) -> f64 {
        let saved = self.draw_note_text.font_scale;
        self.draw_note_text.font_scale = scale;
        let measured = self.draw_note_text.prepare_single_line_run(cx, "Mg");
        self.draw_note_text.font_scale = saved;
        measured
            .map(|run| (run.ascender_in_lpxs - run.descender_in_lpxs) as f64)
            .unwrap_or(0.0)
    }

    /// Lay out a note body into rows, measuring every word in the face it will
    /// be drawn in.
    ///
    /// The blocks and inline runs come from [`crate::note`] (makepad's
    /// `apps/notes` block split); the widths come from the text stack. The old
    /// path wrapped on a character count against a hardcoded `CHAR_W = 7.5`,
    /// which is the same guess that squeezed the terminal grid.
    fn note_rows(
        &mut self,
        cx: &mut Cx2d,
        item_id: u64,
        source: &str,
        body: Rect,
        font_size: f32,
    ) -> Vec<NoteRow> {
        let blocks = crate::note::parse_blocks(source);
        let base = font_size / 13.0;
        let prose_lh = font_size as f64 * 1.36;
        let mut rows: Vec<NoteRow> = Vec::new();
        let mut y = body.pos.y;
        for block in blocks.iter() {
            // (face scale factor, hanging indent, space before, space after, line height)
            let (factor, indent, before, after, line_h) = match block.kind {
                BlockKind::Heading(1) => (1.5, 0.0, 5.0, 5.0, font_size as f64 * 1.5 * 1.18),
                BlockKind::Heading(2) => (1.28, 0.0, 4.0, 4.0, font_size as f64 * 1.28 * 1.2),
                BlockKind::Heading(_) => (1.12, 0.0, 3.0, 3.0, font_size as f64 * 1.12 * 1.25),
                BlockKind::Bullet => (1.0, 16.0, 1.5, 1.5, prose_lh),
                BlockKind::Numbered(_) => (1.0, 20.0, 1.5, 1.5, prose_lh),
                BlockKind::Check { .. } => (1.0, 22.0, 2.5, 2.5, prose_lh),
                BlockKind::Quote => (1.0, 14.0, 2.5, 2.5, prose_lh),
                BlockKind::Rule => (1.0, 0.0, 6.0, 6.0, 6.0),
                BlockKind::Code => (1.0, 10.0, 0.0, 0.0, font_size as f64 * 1.32),
                BlockKind::Paragraph => (1.0, 0.0, if rows.is_empty() { 0.0 } else { 3.0 }, 0.0, prose_lh),
            };
            y += before;
            let text_h = self.note_line_box(cx, base * factor);
            if matches!(block.kind, BlockKind::Rule) {
                rows.push(NoteRow {
                    kind: block.kind,
                    item_id,
                    x: body.pos.x,
                    y,
                    height: line_h,
                    indent,
                    text_h,
                    scale: base * factor,
                    segments: Vec::new(),
                    char_start: block.char_start,
                    char_end: block.char_start,
                    src_line: block.src_line,
                    first: true,
                    width: body.size.x - indent,
                });
                y += line_h + after;
                continue;
            }
            // Words keep their trailing spaces, so no prefix ever has to be
            // re-measured while wrapping.
            let mut words: Vec<(String, InlineStyle, usize)> = Vec::new();
            let mut char_idx = block.char_start;
            let mut word = String::new();
            let mut word_start = char_idx;
            for span in block.spans.iter() {
                for ch in span.text.chars() {
                    if word.is_empty() {
                        word_start = char_idx;
                    }
                    word.push(ch);
                    char_idx += 1;
                    if ch == ' ' {
                        words.push((std::mem::take(&mut word), span.style, word_start));
                    }
                }
            }
            if !word.is_empty() {
                words.push((word, InlineStyle::Plain, word_start));
            }
            let avail = (body.size.x - indent).max(24.0);
            let mut segments: Vec<NoteSegment> = Vec::new();
            let mut used = 0.0f64;
            let mut row_start: Option<usize> = None;
            let mut row_end = block.char_start;
            let mut first_row = true;
            for (text, style, start) in words.iter() {
                let width = self.note_text_width(cx, *style, text, base * factor);
                let len = text.chars().count();
                if row_start.is_some() && used + width > avail {
                    let segments = std::mem::take(&mut segments);
                    rows.push(NoteRow {
                        kind: block.kind,
                        item_id,
                        x: body.pos.x,
                        y,
                        height: line_h,
                        indent,
                        text_h,
                        scale: base * factor,
                        segments,
                        char_start: row_start.unwrap_or(*start),
                        char_end: row_end,
                        src_line: block.src_line,
                        first: first_row,
                        width: avail,
                    });
                    y += line_h;
                    first_row = false;
                    used = 0.0;
                    row_start = None;
                }
                if row_start.is_none() {
                    row_start = Some(*start);
                }
                let x = used;
                used += width;
                row_end = *start + len;
                // Trailing spaces do not push the caret past the word.
                segments.push(NoteSegment {
                    text: text.trim_end_matches(' ').to_string(),
                    style: *style,
                    x,
                    w: width,
                    char_start: *start,
                    trailing: len - text.trim_end_matches(' ').chars().count(),
                });
            }
            rows.push(NoteRow {
                kind: block.kind,
                item_id,
                x: body.pos.x,
                y,
                height: line_h,
                indent,
                text_h,
                scale: base * factor,
                segments,
                char_start: row_start.unwrap_or(block.char_start),
                char_end: row_end,
                src_line: block.src_line,
                first: first_row,
                width: avail,
            });
            y += line_h + after;
        }
        if std::env::var_os("CANVAS_TRACE_NOTE").is_some() {
            // Debug aid, same shape as MAKEPAD_TRACE_FONT_LOAD: dump the measured
            // rows so a wrapping bug can be read off instead of eyeballed.
            for row in rows.iter() {
                let spans: Vec<String> = row
                    .segments
                    .iter()
                    .map(|s| format!("{:?}@{:.1} {:.1}px {:?}", s.style, s.x, s.w, s.text))
                    .collect();
                log!(
                    "note row {:?} y={:.1} h={:.1} indent={:.1} scale={:.2} chars={}..{} first={} | {}",
                    row.kind,
                    row.y,
                    row.height,
                    row.indent,
                    row.scale,
                    row.char_start,
                    row.char_end,
                    row.first,
                    spans.join("  ")
                );
            }
        }
        rows
    }

    /// Draw one laid-out note row: block chrome (bullet, box, quote bar, code
    /// tint, rule) then its measured text runs.
    fn draw_note_row(&mut self, cx: &mut Cx2d, row: &NoteRow, ink: [f32; 4], font_size: f32) {
        let text_x = row.x + row.indent;
        // Rows are centred in their line box so mixed sizes share a baseline grid.
        let text_y = row.y + (row.height - row.text_h) * 0.5;
        let lead = [ink[0] * 0.35, ink[1] * 0.35, ink[2] * 0.35, 1.0];
        let mark = [ink[0] * 0.8, ink[1] * 0.8, ink[2] * 0.8, 1.0];
        match row.kind {
            BlockKind::Bullet if row.first => {
                self.draw_filled_disc(
                    cx,
                    Vec2d {
                        x: text_x - 9.0,
                        y: row.y + row.height * 0.5,
                    },
                    2.2,
                    mark,
                );
            }
            BlockKind::Numbered(number) if row.first => {
                let label = format!("{number}.");
                let scale = row.scale;
                let w = self.note_text_width(cx, InlineStyle::Plain, &label, scale);
                self.draw_note_text.font_scale = scale;
                self.draw_note_text.color = vec4f(mark);
                self.draw_note_text.draw_abs(
                    cx,
                    Vec2d {
                        x: text_x - 6.0 - w,
                        y: text_y,
                    },
                    &label,
                );
                self.draw_note_text.font_scale = 1.0;
            }
            BlockKind::Check { checked } => {
                let size = font_size as f64 * 0.86;
                let box_rect = Rect {
                    pos: Vec2d {
                        x: text_x - size - 8.0,
                        y: row.y + (row.height - size) * 0.5,
                    },
                    size: Vec2d { x: size, y: size },
                };
                // Publish the click area: the box plus the label, and every row
                // of a wrapped item — notes toggles the whole row, not just the
                // box. A click has no `Cx2d`, so it reads this back.
                self.note_check_rows.push((
                    row.item_id,
                    Rect {
                        pos: Vec2d {
                            x: box_rect.pos.x,
                            y: row.y,
                        },
                        size: Vec2d {
                            x: (text_x + row.width - box_rect.pos.x).max(size),
                            y: row.height,
                        },
                    },
                    row.src_line,
                ));
                if std::env::var_os("CANVAS_TRACE_NOTE").is_some() {
                    let (_, hit, line) = self.note_check_rows.last().unwrap();
                    log!(
                        "note check row line={line} hit=({:.1},{:.1} {:.1}x{:.1})",
                        hit.pos.x,
                        hit.pos.y,
                        hit.size.x,
                        hit.size.y
                    );
                }
                if row.first {
                    self.draw_item_bg_rect(
                        cx,
                        box_rect,
                        if checked {
                            [mark[0], mark[1], mark[2], 0.18]
                        } else {
                            [0.0, 0.0, 0.0, 0.0]
                        },
                    );
                    self.draw_border_rect(cx, box_rect, mark);
                    if checked {
                        // Tick: two strokes, sized to the box.
                        let a = box_rect.pos + Vec2d { x: size * 0.22, y: size * 0.52 };
                        let b = box_rect.pos + Vec2d { x: size * 0.42, y: size * 0.74 };
                        let c = box_rect.pos + Vec2d { x: size * 0.8, y: size * 0.26 };
                        self.draw_segment(cx, a, b, 1.6, mark);
                        self.draw_segment(cx, b, c, 1.6, mark);
                    }
                }
            }
            BlockKind::Quote => {
                self.draw_item_bg_rect(
                    cx,
                    Rect {
                        pos: Vec2d {
                            x: text_x - 8.0,
                            y: row.y,
                        },
                        size: Vec2d {
                            x: 2.5,
                            y: row.height,
                        },
                    },
                    mark,
                );
            }
            BlockKind::Code => {
                self.draw_item_bg_rect(
                    cx,
                    Rect {
                        pos: Vec2d {
                            x: text_x - 5.0,
                            y: row.y,
                        },
                        size: Vec2d {
                            x: row.width + 10.0,
                            y: row.height,
                        },
                    },
                    [lead[0] * 0.25, lead[1] * 0.25, lead[2] * 0.3, 0.35],
                );
            }
            BlockKind::Rule => {
                self.draw_item_bg_rect(
                    cx,
                    Rect {
                        pos: Vec2d {
                            x: text_x,
                            y: row.y + row.height * 0.5,
                        },
                        size: Vec2d {
                            x: row.width,
                            y: 1.0,
                        },
                    },
                    mark,
                );
                return;
            }
            _ => {}
        }
        let muted = matches!(row.kind, BlockKind::Check { checked: true });
        for segment in row.segments.iter() {
            if segment.text.is_empty() {
                continue;
            }
            let scale = row.scale;
            let color = match segment.style {
                _ if muted => [ink[0] * 0.55, ink[1] * 0.55, ink[2] * 0.55, 0.85],
                InlineStyle::Code => [ink[0] * 0.75, ink[1] * 0.75, ink[2] * 0.8, 1.0],
                _ => ink,
            };
            match segment.style {
                InlineStyle::Plain => {
                    self.draw_note_text.font_scale = scale;
                    self.draw_note_text.color = vec4f(color);
                    self.draw_note_text.draw_abs(
                        cx,
                        Vec2d {
                            x: text_x + segment.x,
                            y: text_y,
                        },
                        &segment.text,
                    );
                    self.draw_note_text.font_scale = 1.0;
                }
                InlineStyle::Bold => {
                    self.draw_title.font_scale = scale;
                    self.draw_title.color = vec4f(color);
                    self.draw_title.draw_abs(
                        cx,
                        Vec2d {
                            x: text_x + segment.x,
                            y: text_y,
                        },
                        &segment.text,
                    );
                    self.draw_title.font_scale = 1.0;
                }
                InlineStyle::Italic => {
                    self.draw_note_italic.font_scale = scale;
                    self.draw_note_italic.color = vec4f(color);
                    self.draw_note_italic.draw_abs(
                        cx,
                        Vec2d {
                            x: text_x + segment.x,
                            y: text_y,
                        },
                        &segment.text,
                    );
                    self.draw_note_italic.font_scale = 1.0;
                }
                InlineStyle::Code => {
                    let scale = scale * 13.0 / 12.5;
                    self.draw_cell_text.font_scale = scale;
                    self.draw_cell_text.color = vec4f(color);
                    self.draw_cell_text.draw_abs(
                        cx,
                        Vec2d {
                            x: text_x + segment.x,
                            y: text_y,
                        },
                        &segment.text,
                    );
                    self.draw_cell_text.font_scale = 1.0;
                }
            }
        }
        // A checked item is struck through, the way notes strikes done rows.
        if muted {
            let last = row.segments.last();
            let width = last
                .map(|s| s.x + self.note_text_width(cx, s.style, &s.text, row.scale))
                .unwrap_or(0.0);
            self.draw_item_bg_rect(
                cx,
                Rect {
                    pos: Vec2d {
                        x: text_x,
                        y: text_y + row.text_h * 0.5,
                    },
                    size: Vec2d {
                        x: width.max(8.0),
                        y: 1.0,
                    },
                },
                [ink[0] * 0.6, ink[1] * 0.6, ink[2] * 0.6, 0.7],
            );
        }
    }

    /// Where the caret sits for `caret` (a char index into the body), using the
    /// same measured rows the body was drawn from: `(x, y, height)`.
    fn note_caret_pos(
        &mut self,
        cx: &mut Cx2d,
        rows: &[NoteRow],
        caret: usize,
    ) -> Option<(f64, f64, f64)> {
        let row = rows
            .iter()
            .find(|r| caret >= r.char_start && caret <= r.char_end && !r.segments.is_empty())
            .or_else(|| rows.iter().find(|r| !r.segments.is_empty()))
            .or_else(|| rows.first())?;
        let text_x = row.x + row.indent;
        let text_y = row.y + (row.height - row.text_h) * 0.5;
        let mut x = text_x;
        for segment in row.segments.iter() {
            let len = segment.text.chars().count();
            let prefix = if caret <= segment.char_start {
                0
            } else if caret >= segment.char_start + len + segment.trailing {
                len + segment.trailing
            } else {
                (caret - segment.char_start).min(len + segment.trailing)
            };
            let sliced: String = segment.text.chars().take(prefix).collect();
            let width = self.note_text_width(cx, segment.style, &segment.text, row.scale);
            if prefix < len {
                let partial = self.note_text_width(cx, segment.style, &sliced, row.scale);
                return Some((text_x + segment.x + partial, text_y, row.text_h));
            }
            x = text_x + segment.x + width;
            if caret >= segment.char_start + len {
                continue;
            }
            return Some((x, text_y, row.text_h));
        }
        Some((x, text_y, row.text_h))
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
                // Pointer/select cursor (matches the reference's default tool).
                let tip = Vec2d { x: x0, y: y0 };
                let pts = [
                    tip,
                    Vec2d { x: x0, y: y1 - 3.0 },
                    Vec2d {
                        x: x0 + 3.0,
                        y: y1 - 3.0,
                    },
                    Vec2d {
                        x: x0 + 4.5,
                        y: y0 + 5.5,
                    },
                    Vec2d {
                        x: x0 + 10.0,
                        y: y1 - 1.0,
                    },
                    Vec2d {
                        x: x0 + 7.2,
                        y: y0 + 3.4,
                    },
                ];
                self.draw_clean_polyline(cx, &pts, true, w, color);
            }
            NoteTool::Arrow => {
                let tip = Vec2d { x: x1, y: y0 };
                self.draw_segment(cx, Vec2d { x: x0, y: y1 }, tip, w, color);
                let len = (x1 - x0).hypot(y0 - y1);
                self.draw_icon_arrowhead(cx, tip, (x1 - x0) / len, (y0 - y1) / len, 4.5, w, color);
            }
            NoteTool::Pen => {
                // Freehand squiggle (reference style).
                let pts = [
                    Vec2d {
                        x: x0 + 1.0,
                        y: y1 - 1.0,
                    },
                    Vec2d {
                        x: x0 + 3.5,
                        y: y0 + 3.0,
                    },
                    Vec2d {
                        x: x0 + 6.0,
                        y: y1 - 3.0,
                    },
                    Vec2d {
                        x: x0 + 8.5,
                        y: y0 + 2.0,
                    },
                    Vec2d {
                        x: x0 + 11.0,
                        y: y1 - 1.0,
                    },
                ];
                self.draw_clean_polyline(cx, &pts, false, 2.0, color);
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
                // "A" glyph flanked by a text I-beam (reference style).
                self.draw_title.font_scale = 1.0;
                self.draw_title.color = vec4f(color);
                self.draw_title
                    .draw_abs(cx, r.pos + Vec2d { x: 8.0, y: 3.0 }, "A");
                // I-beam: top and bottom serifs with a vertical stem.
                let bx = x1 - 2.0;
                self.draw_segment(
                    cx,
                    Vec2d { x: bx - 2.0, y: y0 },
                    Vec2d { x: bx + 2.0, y: y0 },
                    w,
                    color,
                );
                self.draw_segment(
                    cx,
                    Vec2d { x: bx - 2.0, y: y1 },
                    Vec2d { x: bx + 2.0, y: y1 },
                    w,
                    color,
                );
                self.draw_segment(cx, Vec2d { x: bx, y: y0 }, Vec2d { x: bx, y: y1 }, w, color);
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
                // World-space content: scales with the camera like the shapes
                // around it.
                self.draw_title.font_scale = self.camera.zoom;
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
    /// of the canvas (screen-fixed, always available).
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
                // Filled rounded pill behind the active tool, plus a clean
                // accent border (reads clearly at a glance, like the
                // reference's highlighted default tool).
                let pill = Rect {
                    pos: r.pos + Vec2d { x: 1.0, y: 1.0 },
                    size: Vec2d {
                        x: r.size.x - 2.0,
                        y: r.size.y - 2.0,
                    },
                };
                self.draw_item_bg_rect(cx, pill, [0.30, 0.24, 0.12, 0.95]);
                self.draw_border_rect(cx, pill, PAL_ACCENT);
                // Soft accent glow ring just outside the pill.
                self.draw_glow_border(cx, pill, PAL_ACCENT);
            }
            let icon_color = if active {
                PAL_ACCENT
            } else {
                [0.68, 0.72, 0.82, 1.0]
            };
            self.draw_tool_icon(cx, i, *t, r, icon_color);
        }

        // ── Group dividers (subtle, matches the reference's grouping) ──
        let div_color = [0.42, 0.48, 0.62, 0.5];
        let div_x0 = palette_rect.pos.x + 6.0;
        let div_x1 = palette_rect.pos.x + palette_rect.size.x - 6.0;
        let div_y1 = colors_y - 4.0;
        self.draw_segment(
            cx,
            Vec2d {
                x: div_x0,
                y: div_y1,
            },
            Vec2d {
                x: div_x1,
                y: div_y1,
            },
            1.0,
            div_color,
        );
        let div_y2 = widths_y - 4.0;
        self.draw_segment(
            cx,
            Vec2d {
                x: div_x0,
                y: div_y2,
            },
            Vec2d {
                x: div_x1,
                y: div_y2,
            },
            1.0,
            div_color,
        );

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
                // Active swatch: solid accent ring + soft glow.
                self.draw_sketch_ellipse(
                    cx,
                    center,
                    Self::PAL_SWATCH * 0.5 + 2.0,
                    Self::PAL_SWATCH * 0.5 + 2.0,
                    1.4,
                    PAL_ACCENT,
                    400 + i as u32,
                    0.35,
                );
                self.draw_glow_border(
                    cx,
                    Rect {
                        pos: Vec2d {
                            x: center.x - Self::PAL_SWATCH * 0.5,
                            y: center.y - Self::PAL_SWATCH * 0.5,
                        },
                        size: Vec2d {
                            x: Self::PAL_SWATCH,
                            y: Self::PAL_SWATCH,
                        },
                    },
                    PAL_ACCENT,
                );
            } else {
                // Inactive swatch: readable neutral ring.
                self.draw_sketch_ellipse(
                    cx,
                    center,
                    Self::PAL_SWATCH * 0.5,
                    Self::PAL_SWATCH * 0.5,
                    0.8,
                    [0.34, 0.38, 0.50, 0.75],
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
            let active = i == self.ink_width_idx;
            if active {
                // Filled pill + accent frame marks the active width, matching
                // the active-tool treatment for consistency.
                self.draw_item_bg_rect(cx, r, [0.30, 0.24, 0.12, 0.95]);
                self.draw_sketch_rect_outline(cx, r, 1.2, PAL_ACCENT, 600 + i as u32, 0.4);
            } else {
                // Subtle neutral pill so the row reads as a tappable preset.
                self.draw_item_bg_rect(cx, r, [0.17, 0.20, 0.28, 0.7]);
                self.draw_sketch_rect_outline(
                    cx,
                    r,
                    0.8,
                    [0.30, 0.34, 0.44, 0.5],
                    600 + i as u32,
                    0.4,
                );
            }
            // Sample stroke across the button, clamped to the button height,
            // drawn with the same sketchy wobble as real strokes.
            let sw = wd.min(Self::PAL_WIDTH_BTN_H - 6.0);
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

        // Hovered button's tooltip (MpTooltip-style bubble).
        self.draw_palette_tooltip(cx);
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
            pos: self.viewport_pos,
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
        let with_print = self
            .items
            .iter()
            .find(|i| i.id() == id)
            .is_some_and(Self::has_print_button);
        let (print_r, min_r, close_r) = Self::control_button_rects(screen, with_print);
        let min_hov = self.hovered_btn == Some((id, BtnKind::Minimize));
        let close_hov = self.hovered_btn == Some((id, BtnKind::Close));
        if let Some(pr) = print_r {
            let print_hov = self.hovered_btn == Some((id, BtnKind::Print));
            self.draw_item_bg_rect(cx, pr, if print_hov { BTN_HOVER } else { BTN_BG });
            self.draw_border_rect(cx, pr, BTN_BORDER);
            // Printer glyph: paper sheet over a body tray.
            let glyph = [0.70, 0.75, 0.85, 1.0];
            let body = Rect {
                pos: pr.pos + Vec2d { x: 4.0, y: 8.0 },
                size: Vec2d {
                    x: pr.size.x - 8.0,
                    y: pr.size.y - 11.0,
                },
            };
            self.draw_border_rect(cx, body, glyph);
            let paper = Rect {
                pos: pr.pos + Vec2d { x: 6.5, y: 4.0 },
                size: Vec2d {
                    x: pr.size.x - 13.0,
                    y: 6.0,
                },
            };
            self.draw_item_bg_rect(cx, paper, BTN_BG);
            self.draw_border_rect(cx, paper, glyph);
        }
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
        // Proper “×”: two diagonal strokes corner-to-corner.
        let close_color = [0.90, 0.55, 0.52, 1.0];
        let in_x = close_r.pos.x + 6.0;
        let in_y = close_r.pos.y + 5.0;
        let x_max = close_r.pos.x + close_r.size.x - 6.0;
        let y_max = close_r.pos.y + close_r.size.y - 5.0;
        self.draw_segment(
            cx,
            Vec2d { x: in_x, y: in_y },
            Vec2d { x: x_max, y: y_max },
            1.8,
            close_color,
        );
        self.draw_segment(
            cx,
            Vec2d { x: x_max, y: in_y },
            Vec2d { x: in_x, y: y_max },
            1.8,
            close_color,
        );
    }

    /// Width and x of every parked card's tab, in bar order.
    ///
    /// Tabs shrink to share the bar (browser style) down to a floor, so a
    /// handful of parked cards always stay reachable; past the floor the rest
    /// are clipped rather than scrolled.
    fn card_tab_rects(&self, start_x: f64) -> Vec<(u64, Rect, Rect)> {
        let right = self.view_bottom_right().x;
        let avail = (right - 8.0 - start_x).max(0.0);
        let n = self.minimized.len();
        if n == 0 || avail < CARD_TAB_MIN_W {
            return Vec::new();
        }
        let tab_h = CARD_TAB_H;
        let tab_y = self.viewport_pos.y + (TAB_BAR_H - tab_h) * 0.5;
        // One gap between tabs, plus a slot for the ✕ inside each.
        let per = ((avail - CARD_TAB_GAP * (n.saturating_sub(1)) as f64) / n as f64)
            .clamp(CARD_TAB_MIN_W, CARD_TAB_MAX_W);
        let mut rects = Vec::with_capacity(n);
        let mut x = start_x;
        for item in self.minimized.iter() {
            if x + per > right - 8.0 {
                break;
            }
            let rect = Rect {
                pos: Vec2d { x, y: tab_y },
                size: Vec2d {
                    x: per,
                    y: tab_h,
                },
            };
            let close = Rect {
                pos: Vec2d {
                    x: rect.pos.x + rect.size.x - CARD_TAB_CLOSE - 4.0,
                    y: rect.pos.y + (rect.size.y - CARD_TAB_CLOSE) * 0.5,
                },
                size: Vec2d {
                    x: CARD_TAB_CLOSE,
                    y: CARD_TAB_CLOSE,
                },
            };
            rects.push((item.id(), rect, close));
            x += per + CARD_TAB_GAP;
        }
        rects
    }

    /// The workspace tab rects and new-space button, plus the x where the
    /// parked-card strip begins. One layout, so the draw pass and the hit-test
    /// cannot disagree.
    fn workspace_tab_layout(&self) -> (Vec<(TopBarHit, Rect)>, f64) {
        let mut out = Vec::new();
        let mut x = self.viewport_pos.x + 8.0;
        let tab_h = TAB_BAR_H - 8.0;
        let tab_y = self.viewport_pos.y + 4.0;
        let n = self.workspaces.len().max(1);
        for i in 0..n {
            let tab_w = (Self::space_tab_w(i) + 28.0).max(70.0);
            out.push((
                TopBarHit::Workspace(i),
                Rect {
                    pos: Vec2d { x, y: tab_y },
                    size: Vec2d { x: tab_w, y: tab_h },
                },
            ));
            x += tab_w + 6.0;
        }
        out.push((
            TopBarHit::AddWorkspace,
            Rect {
                pos: Vec2d { x, y: tab_y },
                size: Vec2d {
                    x: TAB_PLUS_W,
                    y: tab_h,
                },
            },
        ));
        (out, x + TAB_PLUS_W + CARD_TAB_DIVIDER)
    }

    /// Width of a `Space N` tab's label, in the bar face.
    fn space_tab_w(index: usize) -> f64 {
        format!("Space {index}").chars().count() as f64 * 7.5
    }

    /// Which workspace tab, new-space button or parked-card tab is under
    /// `screen`.
    fn top_bar_hit(&self, screen: Vec2d) -> Option<TopBarHit> {
        if screen.y < self.viewport_pos.y || screen.y > self.viewport_pos.y + TAB_BAR_H {
            return None;
        }
        let (spaces, strip_x) = self.workspace_tab_layout();
        if let Some(hit) = spaces
            .iter()
            .find(|(_, rect)| rect.contains(screen))
            .map(|(hit, _)| *hit)
        {
            return Some(hit);
        }
        self.card_tab_rects(strip_x)
            .into_iter()
            .find_map(|(id, rect, close)| {
                if close.contains(screen) {
                    Some(TopBarHit::CardClose(id))
                } else if rect.contains(screen) {
                    Some(TopBarHit::Card(id))
                } else {
                    None
                }
            })
    }

    /// Draw the top bar: workspace tabs, the new-space button, then the parked
    /// cards as browser-style tabs (favicon dot, title, ✕).
    fn draw_top_bar(&mut self, cx: &mut Cx2d) {
        let bar_rect = Rect {
            pos: self.viewport_pos,
            size: Vec2d {
                x: self.viewport.x,
                y: TAB_BAR_H,
            },
        };
        self.draw_item_bg_rect(cx, bar_rect, TAB_BG);

        self.draw_border_rect(cx, bar_rect, TAB_BORDER);
        // Fixed-size chrome: pin the text scale so camera zoom (left behind by
        // the last card drawn) cannot leak into the tab bar.
        self.draw_title.font_scale = 1.0;

        let (spaces, strip_x) = self.workspace_tab_layout();
        for (hit, tab_rect) in spaces.iter() {
            let TopBarHit::Workspace(i) = hit else {
                continue;
            };
            let active = *i == self.current_workspace;
            let label = format!("Space {}", i + 1);
            self.draw_item_bg_rect(
                cx,
                *tab_rect,
                if active {
                    TAB_ACTIVE_BG
                } else {
                    TAB_INACTIVE_BG
                },
            );
            self.draw_border_rect(cx, *tab_rect, TAB_BORDER);
            if active {
                // Accent underline marks the active workspace clearly.
                self.draw_item_bg_rect(
                    cx,
                    Rect {
                        pos: Vec2d {
                            x: tab_rect.pos.x,
                            y: tab_rect.pos.y + tab_rect.size.y - 2.5,
                        },
                        size: Vec2d {
                            x: tab_rect.size.x,
                            y: 2.5,
                        },
                    },
                    PAL_ACCENT,
                );
            }
            self.draw_title.color = vec4f(if active { PAL_ACCENT } else { TITLE_TEXT });
            self.draw_title
                .draw_abs(cx, tab_rect.pos + Vec2d { x: 12.0, y: 5.0 }, &label);
        }

        // Add-workspace button.
        if let Some((_, plus_rect)) = spaces
            .iter()
            .find(|(hit, _)| matches!(hit, TopBarHit::AddWorkspace))
        {
            let plus_rect = *plus_rect;
            self.draw_item_bg_rect(cx, plus_rect, TAB_INACTIVE_BG);
            self.draw_border_rect(cx, plus_rect, TAB_BORDER);
            self.draw_title.color = vec4f(TITLE_TEXT);
            self.draw_title
                .draw_abs(cx, plus_rect.pos + Vec2d { x: 10.0, y: 5.0 }, "+");
        }

        // Divider, then the parked cards.
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: strip_x - CARD_TAB_DIVIDER * 0.5,
                    y: self.viewport_pos.y + 8.0,
                },
                size: Vec2d {
                    x: 1.0,
                    y: TAB_BAR_H - 16.0,
                },
            },
            TAB_BORDER,
        );
        let hovered = self.hovered_tab;
        for (id, rect, _close) in self.card_tab_rects(strip_x) {
            let tab_hovered = hovered == Some(TopBarHit::Card(id));
            let close_hovered = hovered == Some(TopBarHit::CardClose(id));
            self.draw_card_tab(cx, id, rect, tab_hovered, close_hovered);
        }
    }
    /// One parked card's tab: a raised body with clipped top corners (the tab
    /// silhouette), a kind-coloured "favicon" dot, the title, and a ✕.
    fn draw_card_tab(
        &mut self,
        cx: &mut Cx2d,
        id: u64,
        rect: Rect,
        hovered: bool,
        close_hovered: bool,
    ) {
        let Some(item) = self.items.iter().find(|i| i.id() == id).or_else(|| {
            self.minimized.iter().find(|i| i.id() == id)
        }) else {
            return;
        };
        let kind = item.kind();
        let title = item.title().to_string();
        let body = if hovered { TAB_ACTIVE_BG } else { CARD_TAB_BG };
        self.draw_item_bg_rect(cx, rect, body);
        self.draw_border_rect(cx, rect, TAB_BORDER);
        // Pixel-art rounded top corners: a 2px and a 1px step off each shoulder,
        // painted in the bar's own colour (the bar is flat, so this is exact).
        for (step, w) in [(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)] {
            for side in [rect.pos.x, rect.pos.x + rect.size.x - w] {
                self.draw_item_bg_rect(
                    cx,
                    Rect {
                        pos: Vec2d {
                            x: side,
                            y: rect.pos.y + step,
                        },
                        size: Vec2d { x: w, y: 1.0 },
                    },
                    TAB_BG,
                );
            }
        }
        // Kind dot, the tab's "favicon".
        let dot = 7.0;
        self.draw_filled_disc(
            cx,
            Vec2d {
                x: rect.pos.x + 10.0,
                y: rect.pos.y + rect.size.y * 0.5,
            },
            dot * 0.5,
            kind_accent(kind),
        );
        // Title, elided to the space between the dot and the ✕.
        let text_x = rect.pos.x + 10.0 + dot + 5.0;
        let close_x = rect.pos.x + rect.size.x - CARD_TAB_CLOSE - 4.0;
        let label = self.elide_title(cx, &title, (close_x - 6.0 - text_x).max(8.0));
        self.draw_title.color = vec4f(if hovered { TITLE_TEXT } else { TAB_TITLE_DIM });
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: text_x,
                y: rect.pos.y + (rect.size.y - 16.0) * 0.5,
            },
            &label,
        );
        // ✕ close affordance.
        let close = Rect {
            pos: Vec2d {
                x: close_x,
                y: rect.pos.y + (rect.size.y - CARD_TAB_CLOSE) * 0.5,
            },
            size: Vec2d {
                x: CARD_TAB_CLOSE,
                y: CARD_TAB_CLOSE,
            },
        };
        let ink = if close_hovered {
            BTN_CLOSE_HOVER
        } else if hovered {
            TITLE_TEXT
        } else {
            TAB_TITLE_DIM
        };
        let a = close.pos + Vec2d { x: 4.0, y: 4.0 };
        let b = close.pos + Vec2d {
            x: CARD_TAB_CLOSE - 4.0,
            y: CARD_TAB_CLOSE - 4.0,
        };
        self.draw_segment(cx, a, b, 1.4, ink);
        self.draw_segment(
            cx,
            Vec2d { x: b.x, y: a.y },
            Vec2d { x: a.x, y: b.y },
            1.4,
            ink,
        );
    }

    /// `s` elided with an ellipsis to `max_w` (measured, not guessed).
    fn elide_title(&mut self, cx: &mut Cx2d, s: &str, max_w: f64) -> String {
        self.draw_title.font_scale = 1.0;
        let width = self
            .draw_title
            .prepare_single_line_run(cx, s)
            .map(|run| run.width_in_lpxs as f64)
            .unwrap_or(0.0);
        if width <= max_w {
            return s.to_string();
        }
        let mut out = String::new();
        for ch in s.chars() {
            let mut probe = out.clone();
            probe.push(ch);
            probe.push('…');
            let w = self
                .draw_title
                .prepare_single_line_run(cx, &probe)
                .map(|run| run.width_in_lpxs as f64)
                .unwrap_or(0.0);
            if w > max_w {
                break;
            }
            out.push(ch);
        }
        out.push('…');
        out
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
            } else if bg != default_bg() {
                [bg[0], bg[1], bg[2], 1.0]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            if selected || bg != default_bg() {
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
                // A fallback face can hand back a glyph wider than its cell
                // (emoji, rare symbols). Shrink it into its slot instead of
                // letting it run over its neighbour — the "fit" makepad's
                // terminal applies in `cached_glyph`. ASCII skips the check:
                // the mono face advances exactly one cell for it.
                let fit = if cell.ch.is_ascii() {
                    1.0
                } else {
                    // A wide glyph owns its trailing padding cell too.
                    let cells = if row.get(i + k + 1).is_some_and(|c| c.wide_padding) {
                        2u8
                    } else {
                        1u8
                    };
                    self.glyph_fit(cx, cell.ch, cells)
                };
                if fit < 1.0 {
                    let scale = self.draw_cell_text.font_scale;
                    self.draw_cell_text.font_scale = scale * fit;
                    self.draw_cell_text.draw_abs(cx, Vec2d { x: cx_pos, y }, &cs);
                    self.draw_cell_text.font_scale = scale;
                } else {
                    self.draw_cell_text.draw_abs(cx, Vec2d { x: cx_pos, y }, &cs);
                }
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
        // Avatar chip + title (dimmed slightly when the card isn't focused,
        // consistent with the dimmed grid + hidden cursor).
        let avatar_color = name_color(title);
        self.draw_avatar(
            cx,
            screen.pos + Vec2d { x: 8.0, y: 5.0 },
            title,
            avatar_color,
        );
        // Title text (DrawText) - drawn after bg/border but before content
        self.draw_title.color = vec4f(if is_sel {
            TITLE_TEXT
        } else {
            [
                TITLE_TEXT[0] * 0.7,
                TITLE_TEXT[1] * 0.7,
                TITLE_TEXT[2] * 0.7,
                1.0,
            ]
        });
        self.draw_title.draw_abs(
            cx,
            screen.pos + Vec2d {
                x: 32.0,
                y: TITLE_TEXT_DY,
            },
            &format!("{} — {}", title, command),
        );

        // Agent status dot + label on the right side of the title bar.
        self.draw_presence(
            cx,
            screen,
            status.label(),
            status.color(),
            if is_sel {
                MUSIC_SECONDARY
            } else {
                [
                    MUSIC_SECONDARY[0] * 0.7,
                    MUSIC_SECONDARY[1] * 0.7,
                    MUSIC_SECONDARY[2] * 0.7,
                    1.0,
                ]
            },
        );

        let grid = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        // Subtle divider between the title bar and the terminal grid.
        let div_y = screen.pos.y + 28.0;
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: screen.pos.x + 6.0,
                    y: div_y,
                },
                size: Vec2d {
                    x: screen.size.x - 12.0,
                    y: 1.0,
                },
            },
            [0.20, 0.24, 0.32, 0.8],
        );

        // Content area (below the title bar, inside the item border).
        let origin = screen.pos + Vec2d { x: 6.0, y: 30.0 };
        let content_w = (screen.size.x - 12.0).max(1.0);
        let content_h = (screen.size.y - 34.0).max(1.0);

        // Fixed cell size: characters keep a constant width/height; dragging
        // the resize handle only changes how many cols/rows fit (the PTY is
        // resized by draw_walk), never stretching glyphs.
        let cols = grid.cols.max(1);
        let rows = grid.rows.max(1);
        let char_w = self.cell_w();
        let line_h = self.cell_h();
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
        // Live grid rows (dimmed if scrolled, or if the card isn't the
        // focused terminal so an inactive card reads as background).
        for r in 0..live_rows {
            let row = &grid.lines[start + r];
            let y = origin.y + disp_r as f64 * line_h;
            let dim = hist_offset > 0 || !is_sel;
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

        // Cursor (block / beam / underline per PTY style). Only the focused
        // terminal shows its cursor; a non-selected card reads as background.
        if is_sel && grid.cursor_visible() {
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

    /// Draw an agent card: title bar with presence, then the transcript.
    ///
    /// The transcript is rendered from the card state's rows, soft-wrapped at a
    /// fixed character width like the note card — good enough to read, and
    /// consistent with everything else on the canvas.
    // Eleven parameters matches the sibling draw_*_at functions (see
    // `draw_terminal_at`, `draw_music_player`); the card state travels as an
    // Arc so drawing never touches the client.
    #[allow(clippy::too_many_arguments)]
    fn draw_agent_at(
        &mut self,
        cx: &mut Cx2d,
        item_id: u64,
        screen: Rect,
        title: &str,
        cwd: &str,
        provider: &str,
        is_sel: bool,
        card: &std::sync::Arc<std::sync::Mutex<crate::chat::ChatCardState>>,
        composer: Option<&str>,
        stop_hover: bool,
    ) {
        let card = card.lock();
        let Ok(card) = card else { return };

        self.draw_shadow_rect(cx, screen);
        if is_sel {
            self.draw_glow_border(cx, screen, SEL_BORDER);
        }
        self.draw_item_bg_rect(cx, screen, AGENT_BG);
        self.draw_border_rect(cx, screen, if is_sel { SEL_BORDER } else { AGENT_BORDER });

        // Title bar: avatar, name + provider, presence.
        let avatar_color = name_color(title);
        self.draw_avatar(
            cx,
            screen.pos + Vec2d { x: 8.0, y: 5.0 },
            title,
            avatar_color,
        );
        self.draw_title.color = vec4f(if is_sel { TITLE_TEXT } else { DIM_TEXT });
        // Card chrome is fixed-size (title bar offsets below are in pixels), so
        // pin the scale rather than inheriting the last drawer's.
        self.draw_title.font_scale = 1.0;
        let sub = if provider.is_empty() {
            cwd.to_string()
        } else {
            format!("{provider} · {}", path_tail(cwd))
        };
        self.draw_title.draw_abs(
            cx,
            screen.pos + Vec2d {
                x: 32.0,
                y: TITLE_TEXT_DY,
            },
            &format!("{title} — {sub}"),
        );

        // Presence: working / ready, on the right of the title bar.
        let (status_label, status_color) = if card.busy {
            ("working", crate::items::AgentStatus::Busy.color())
        } else {
            ("ready", crate::items::AgentStatus::Online.color())
        };
        self.draw_presence(
            cx,
            screen,
            status_label,
            status_color,
            if is_sel { MUSIC_SECONDARY } else { DIM_TEXT },
        );

        // Divider between the title bar and the transcript.
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: screen.pos.x + 6.0,
                    y: screen.pos.y + 28.0,
                },
                size: Vec2d {
                    x: screen.size.x - 12.0,
                    y: 1.0,
                },
            },
            [0.24, 0.22, 0.32, 0.8],
        );

        // Transcript, clipped to the content area.
        let body_rect = Rect {
            pos: screen.pos + Vec2d { x: 8.0, y: 32.0 },
            size: Vec2d {
                x: (screen.size.x - 16.0).max(1.0),
                y: (screen.size.y - 40.0).max(1.0),
            },
        };
        cx.push_clip_rect(body_rect);
        // Transcript content scales with the camera, like the terminal grid.
        self.draw_cell_text.font_scale = self.camera.zoom;
        let char_w = self.cell_w();
        let line_h = self.cell_h();
        let max_chars = ((body_rect.size.x / char_w).floor().max(8.0)) as usize;
        let max_lines = (body_rect.size.y / line_h).floor().max(1.0) as usize;

        // The transcript must never run under the composer strip.
        let text_h = (body_rect.size.y - AGENT_STRIP_H).max(1.0);
        let budget = (text_h / line_h).floor() as usize;

        // No conversation yet: say so, or the card reads as broken.
        if card.rows().is_empty() {
            self.draw_cell_text.color = vec4f([0.42, 0.46, 0.56, 1.0]);
            self.draw_cell_text.draw_abs(
                cx,
                Vec2d {
                    x: body_rect.pos.x + 4.0,
                    y: body_rect.pos.y + line_h,
                },
                "No conversation yet.",
            );
            self.draw_cell_text.color = vec4f([0.34, 0.37, 0.45, 1.0]);
            self.draw_cell_text.draw_abs(
                cx,
                Vec2d {
                    x: body_rect.pos.x + 4.0,
                    y: body_rect.pos.y + line_h * 3.0,
                },
                "Click the input line below, or double-click the card.",
            );
            self.draw_cell_text.draw_abs(
                cx,
                Vec2d {
                    x: body_rect.pos.x + 4.0,
                    y: body_rect.pos.y + line_h * 4.0,
                },
                "@name text from the palette.",
            );
        }

        // Show the tail of the conversation: an agent's transcript grows, and
        // the latest turns are what the card is for.
        let mut rendered_lines: Vec<([f32; 4], String)> = Vec::new();
        for row in card.rows().iter().rev() {
            match row {
                crate::chat::Row::User { text } => {
                    for line in wrap("> ", text, max_chars) {
                        rendered_lines.push((AGENT_ROW_USER, line));
                    }
                    rendered_lines.push((AGENT_ROW_USER, String::new()));
                }
                crate::chat::Row::Assistant { text, streaming } => {
                    let marker = if *streaming { "▍" } else { "" };
                    for line in wrap("", text, max_chars) {
                        rendered_lines.push((AGENT_ROW_ASSISTANT, line));
                    }
                    if *streaming && !text.is_empty() {
                        if let Some(last) = rendered_lines.last_mut() {
                            last.1.push_str(marker);
                        }
                    }
                    rendered_lines.push((AGENT_ROW_ASSISTANT, String::new()));
                }
                crate::chat::Row::Reasoning { text } => {
                    for line in wrap("· ", text, max_chars) {
                        rendered_lines.push((AGENT_ROW_REASONING, line));
                    }
                    rendered_lines.push((AGENT_ROW_REASONING, String::new()));
                }
                crate::chat::Row::Tool { name, ok, summary } => {
                    let (icon, color) = match ok {
                        Some(true) => ("✓", AGENT_ROW_OK),
                        Some(false) => ("✗", AGENT_ROW_FAIL),
                        None => ("…", AGENT_ROW_NOTICE),
                    };
                    let mut first = format!("{icon} {name}");
                    if !summary.is_empty() {
                        first.push_str(&format!(" — {summary}"));
                    }
                    for line in wrap("", &first, max_chars) {
                        rendered_lines.push((color, line));
                    }
                    rendered_lines.push((color, String::new()));
                }
                crate::chat::Row::Notice { text } => {
                    for line in wrap("", text, max_chars) {
                        rendered_lines.push((AGENT_ROW_NOTICE, line));
                    }
                    rendered_lines.push((AGENT_ROW_NOTICE, String::new()));
                }
            }
        }
        rendered_lines.reverse();

        // Draw the tail, honouring a per-card scroll offset into the history.
        // Scrolling is clamped so the view can never leave the transcript.
        let scroll = self.agent_scroll.get(&item_id).copied().unwrap_or(0.0);
        let scroll = scroll.clamp(0.0, (rendered_lines.len() as f64).max(1.0) - 1.0);
        let tail_len = rendered_lines.len().saturating_sub(scroll as usize);
        let gap_at_tail = rendered_lines.len() - tail_len > 0 && scroll > 0.0;

        // Draw only as much as fits, from the bottom up.
        let skip = tail_len.saturating_sub(budget);
        let view: Vec<([f32; 4], String)> = rendered_lines
            .iter()
            .skip(skip)
            .take(budget)
            .cloned()
            .collect();
        for (index, (color, line)) in view.iter().enumerate() {
            if index >= max_lines {
                break;
            }
            self.draw_cell_text.color = vec4f(*color);
            self.draw_cell_text.draw_abs(
                cx,
                Vec2d {
                    x: body_rect.pos.x,
                    y: body_rect.pos.y + index as f64 * line_h,
                },
                line,
            );
        }

        // Composer line: where the user types into this card. Always drawn,
        // so the card is visibly interactive; an empty line shows a
        // placeholder instead of a bare caret.
        if let Some(text) = composer {
            let line_y = body_rect.pos.y + text_h;
            self.draw_item_bg_rect(
                cx,
                Rect {
                    pos: Vec2d {
                        x: body_rect.pos.x - 4.0,
                        y: line_y + 2.0,
                    },
                    size: Vec2d {
                        x: body_rect.size.x + 8.0,
                        y: AGENT_STRIP_H - 4.0,
                    },
                },
                [0.16, 0.17, 0.22, 1.0],
            );
            if text.is_empty() {
                self.draw_cell_text.color = vec4f([0.40, 0.44, 0.54, 1.0]);
                self.draw_cell_text.draw_abs(
                    cx,
                    Vec2d {
                        x: body_rect.pos.x,
                        y: line_y + 4.0,
                    },
                    "> type a prompt; ⏎ sends",
                );
            } else {
                self.draw_cell_text.color = vec4f(AGENT_ROW_USER);
                self.draw_cell_text.draw_abs(
                    cx,
                    Vec2d {
                        x: body_rect.pos.x,
                        y: line_y + 4.0,
                    },
                    &format!("> {text}▍"),
                );
            }
            let _ = gap_at_tail;
        }
        cx.pop_clip_rect();

        // Usage in the bottom-left corner, when the CLI reports it.
        if let Some(usage) = &card.usage {
            let text = usage.clone();
            self.draw_title.color = vec4f(DIM_TEXT);
            self.draw_title.draw_abs(
                cx,
                Vec2d {
                    x: screen.pos.x + 10.0,
                    y: screen.pos.y + screen.size.y - 17.0 - AGENT_STRIP_H,
                },
                &text,
            );
        }

        // Stop button while the CLI is mid-turn.
        if card.busy {
            let rect = Self::agent_stop_rect(screen);
            let fill = if stop_hover {
                [0.72, 0.30, 0.34, 1.0]
            } else {
                [0.45, 0.22, 0.26, 1.0]
            };
            self.draw_item_bg_rect(cx, rect, fill);
            self.draw_cell_text.color = vec4f([1.0, 1.0, 1.0, 1.0]);
            self.draw_cell_text
                .draw_abs(cx, rect.pos + Vec2d { x: 10.0, y: 4.0 }, "■ stop");
        }
    }

    /// Approval button rects, derived purely from the card rect so the draw
    /// pass and the hit-test can never disagree (same discipline as
    /// [`Self::control_button_rects`]).
    fn approval_button_rects(screen: Rect) -> (Rect, Rect) {
        let bottom = screen.pos.y + screen.size.y - AGENT_BTN_H - 8.0;
        let allow = Rect {
            pos: Vec2d {
                x: screen.pos.x + 12.0,
                y: bottom,
            },
            size: Vec2d {
                x: AGENT_BTN_W,
                y: AGENT_BTN_H,
            },
        };
        let deny = Rect {
            pos: Vec2d {
                x: allow.pos.x + AGENT_BTN_W + 8.0,
                y: bottom,
            },
            size: Vec2d {
                x: AGENT_BTN_W,
                y: AGENT_BTN_H,
            },
        };
        (allow, deny)
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
        // Fixed-size browser chrome: pin the text scale to the card's own.
        self.draw_title.font_scale = 1.0;
        self.draw_title.color = vec4f(TITLE_TEXT);
        self.draw_title
            .draw_abs(cx, bar_rect.pos + Vec2d { x: 32.0, y: 7.0 }, url);

        // Page area below the title bar.
        let page_rect = Rect {
            pos: screen.pos + Vec2d { x: 2.0, y: 32.0 },
            size: screen.size - Vec2d { x: 4.0, y: 34.0 },
        };
        // Subtle divider between the browser chrome and the page area.
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: screen.pos.x + 6.0,
                    y: screen.pos.y + 30.0,
                },
                size: Vec2d {
                    x: screen.size.x - 12.0,
                    y: 1.0,
                },
            },
            [0.20, 0.24, 0.32, 0.8],
        );
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

    /// Allocate a free slot in `slots` (a pool of `count` widget instances).
    /// Returns `(slot_index, fresh)` — `fresh` is true when the item was
    /// mapped for the first time, which is the "load the source once" guard.
    /// None means every slot is taken by live items; the caller draws a
    /// placeholder instead of stealing (stealing would ping-pong sources).
    fn alloc_slot(slots: &mut Vec<(u64, usize)>, id: u64, count: usize) -> Option<(usize, bool)> {
        if let Some((_, s)) = slots.iter().find(|(i, _)| *i == id) {
            return Some((*s, false));
        }
        let free = (0..count).find(|s| !slots.iter().any(|(_, used)| used == s))?;
        slots.push((id, free));
        Some((free, true))
    }

    /// Draw a dropped-media card (image / video / PDF preview) and return
    /// `Some((id, content_rect, path))` for video items, which the caller
    /// must draw after the child pass (see `draw_deferred_videos`).
    ///
    /// Image items render through one of the hidden `Image` widget slots and
    /// PDF items through the CEF `Browser` slots (Chromium's built-in PDF
    /// viewer loading a `file://` URL) — both follow the browser-slot
    /// pattern: make visible, draw at the item rect with an abs_pos walk,
    /// hide again. Video items are deferred because the Video widget has no
    /// `visible` flag: its 0×0 flow-pass draw would clobber the widget area
    /// (and thus the controls' hit testing).
    fn draw_media_at(
        &mut self,
        cx: &mut Cx2d,
        id: u64,
        screen: Rect,
        is_sel: bool,
        kind: MediaKind,
        path: &str,
    ) -> Option<(u64, Rect, MediaKind, String)> {
        // Card chrome: shadow, background, border.
        self.draw_shadow_rect(cx, screen);
        if is_sel {
            self.draw_glow_border(cx, screen, SEL_BORDER);
        }
        self.draw_item_bg_rect(cx, screen, TERM_BG);
        self.draw_border_rect(cx, screen, if is_sel { SEL_BORDER } else { TERM_BORDER });

        let accent = kind.accent();
        let bar_rect = Rect {
            pos: screen.pos,
            size: Vec2d {
                x: screen.size.x,
                y: 30.0,
            },
        };
        self.draw_item_bg_rect(cx, bar_rect, [0.14, 0.16, 0.22, 1.0]);
        self.draw_avatar(
            cx,
            screen.pos + Vec2d { x: 8.0, y: 5.0 },
            kind.avatar(),
            accent,
        );
        // Fixed-size media chrome: pin the text scale to the card's own.
        self.draw_title.font_scale = 1.0;
        self.draw_title.color = vec4f(TITLE_TEXT);
        self.draw_title.draw_abs(
            cx,
            bar_rect.pos + Vec2d { x: 32.0, y: 7.0 },
            path_file_name(path),
        );

        // Content area below the title bar, with a chrome divider like the
        // browser card.
        let content = Rect {
            pos: screen.pos + Vec2d { x: 2.0, y: 32.0 },
            size: screen.size - Vec2d { x: 4.0, y: 34.0 },
        };
        self.draw_item_bg_rect(
            cx,
            Rect {
                pos: Vec2d {
                    x: screen.pos.x + 6.0,
                    y: screen.pos.y + 30.0,
                },
                size: Vec2d {
                    x: screen.size.x - 12.0,
                    y: 1.0,
                },
            },
            [0.20, 0.24, 0.32, 0.8],
        );
        if content.size.x <= 4.0 || content.size.y <= 4.0 {
            return None;
        }

        match kind {
            MediaKind::Image => {
                self.draw_image_slot(cx, id, content, path);
                None
            }
            MediaKind::Pdf => {
                self.draw_pdf_slot(cx, id, content, path);
                None
            }
            MediaKind::Text => {
                self.draw_text_slot(cx, id, content, path);
                None
            }
            MediaKind::Web => {
                self.draw_web_slot(cx, id, content, path);
                None
            }
            MediaKind::Video => Some((id, content, kind, path.to_string())),
        }
    }

    /// Draw an image item's content rect through an `Image` widget slot,
    /// loading the file into that slot the first time the item is drawn.
    fn draw_image_slot(&mut self, cx: &mut Cx2d, id: u64, rect: Rect, path: &str) {
        let Some((slot, fresh)) = Self::alloc_slot(&mut self.media_image_slots, id, IMAGE_SLOTS)
        else {
            self.draw_slot_busy(cx, rect);
            return;
        };
        let slot_id = LiveId::from_str(&format!("image_slot_{}", slot));
        let widget = self.view.widget(cx.cx, &[slot_id]);
        let image = widget.as_image();
        if fresh {
            if let Err(e) = image.load_image_file_by_path(cx.cx, std::path::Path::new(path)) {
                log!("canvas: failed to load image '{path}': {e:?}");
            }
        }
        image.set_visible(cx.cx, true);
        let walk = Walk {
            abs_pos: Some(rect.pos),
            width: Size::Fixed(rect.size.x),
            height: Size::Fixed(rect.size.y),
            ..Default::default()
        };
        let _ = widget.draw_walk(cx, &mut Scope::empty(), walk);
        image.set_visible(cx.cx, false);
    }

    /// Draw a text item's content rect: read the file once (UTF-8 lossy,
    /// capped at [`TEXT_MAX_BYTES`]) and render it with the terminal's
    /// monospace font, clipped to the card and offset by the per-item scroll.
    fn draw_text_slot(&mut self, cx: &mut Cx2d, id: u64, rect: Rect, path: &str) {
        if !self.text_docs.contains_key(&id) {
            let content = std::fs::read(path)
                .map(|bytes| {
                    let truncated = bytes.len() > TEXT_MAX_BYTES;
                    let mut text =
                        String::from_utf8_lossy(&bytes[..bytes.len().min(TEXT_MAX_BYTES)])
                            .into_owned();
                    if truncated {
                        text.push_str("\n… (truncated)");
                    }
                    text
                })
                .unwrap_or_else(|e| format!("(failed to read {path}: {e})"));
            self.text_docs.insert(id, content);
        }
        let content = self.text_docs.get(&id).cloned().unwrap_or_default();
        let scroll = self.text_scroll.get(&id).copied().unwrap_or(0.0);

        // Monospace grid shared with terminal rendering; measured from the
        // face like the terminal grid (the old 8×17.3 constants were the
        // terminal's guess and never matched the font).
        let cell_w = self.cell_w();
        let cell_h = self.cell_h();
        let rows = ((rect.size.y / cell_h).floor() as usize).max(1);
        let lines: Vec<&str> = content.lines().collect();
        let max_scroll = lines.len().saturating_sub(rows) as f64;
        let scroll = scroll.clamp(0.0, max_scroll);
        let first = scroll as usize;

        cx.push_clip_rect(rect);
        // Keep the card's dark paper under the text (the card bg is already
        // dark; draw text a row at a time).
        self.draw_cell_text.color = vec4f([0.86, 0.89, 0.94, 1.0]);
        // Scale the face with the camera, like the terminal grid does.
        self.draw_cell_text.font_scale = self.camera.zoom;
        let baseline = rect.pos + Vec2d { x: 8.0, y: 4.0 };
        for (row, line) in lines.iter().skip(first).take(rows + 1).enumerate() {
            // Trim trailing \r from CRLF files.
            let line = line.strip_suffix('\r').unwrap_or(line);
            // Rough horizontal clip: skip drawing rows that fall below the rect.
            let y = baseline.y + row as f64 * cell_h;
            if y > rect.pos.y + rect.size.y {
                break;
            }
            // Horizontal cull: drop leading chars that scroll out of view.
            let max_chars = ((rect.size.x - 16.0) / cell_w).floor().max(1.0) as usize;
            let line = if line.chars().count() > max_chars {
                let cropped: String = line.chars().take(max_chars).collect();
                cropped
            } else {
                line.to_string()
            };
            self.draw_cell_text.draw_abs(
                cx,
                baseline
                    + Vec2d {
                        x: 0.0,
                        y: row as f64 * cell_h,
                    },
                &line,
            );
        }
        cx.pop_clip_rect();
    }

    /// Draw a local HTML item's content rect through a CEF `Browser` slot
    /// pointing at the file's `file://` URL (shares the browser slot pool;
    /// Chromium renders it like any web page).
    fn draw_web_slot(&mut self, cx: &mut Cx2d, id: u64, rect: Rect, path: &str) {
        let slot = match self.browser_slots.iter().find(|(i, _)| *i == id) {
            Some((_, s)) => *s,
            None => {
                let free = (0..4)
                    .find(|s| !self.browser_slots.iter().any(|(_, used)| used == s))
                    .unwrap_or(0);
                self.browser_slots.push((id, free));
                free
            }
        };
        let slot_id = LiveId::from_str(&format!("browser_slot_{}", slot));
        let slot_widget = self.view.widget(cx.cx, &[slot_id]);
        let slot_browser = slot_widget.as_browser();
        if !self.browser_spawned.contains(&id) {
            self.browser_spawned.push(id);
            slot_browser.set_url(cx.cx, &file_url(path));
        }
        slot_browser.set_visible(cx.cx, true);
        let walk = Walk {
            abs_pos: Some(rect.pos),
            width: Size::Fixed(rect.size.x),
            height: Size::Fixed(rect.size.y),
            ..Default::default()
        };
        let _ = slot_widget.draw_walk(cx, &mut Scope::empty(), walk);
        slot_browser.set_visible(cx.cx, false);
    }

    /// Print a PDF media item's file via the system print queue (`lp` on
    /// Unix). Feedback lands in the status bar.
    fn print_pdf(&mut self, cx: &mut Cx, id: u64) {
        let Some(path) = self
            .items
            .iter()
            .find(|i| i.id() == id)
            .and_then(|i| i.path())
            .map(|p| p.to_string())
        else {
            return;
        };
        self.status(cx, &format!("Printing {}…", path_file_name(&path)));
        let output = std::process::Command::new("lp").arg(&path).output();
        match output {
            Ok(out) if out.status.success() => {
                self.status(
                    cx,
                    &format!("Sent {} to the default printer", path_file_name(&path)),
                );
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                self.status(cx, &format!("Print failed: {}", err.trim()));
            }
            Err(e) => {
                self.status(cx, &format!("Print failed: {e} (no 'lp' command?)"));
            }
        }
        self.redraw(cx);
    }

    /// Draw deferred video slots (after the child pass, so the Video
    /// widget's recorded area — its controls' hit rect — is the item rect,
    /// not its 0×0 flow-pass slot).
    fn draw_deferred_media(&mut self, cx: &mut Cx2d, deferred: &[(u64, Rect, MediaKind, String)]) {
        for (id, rect, kind, path) in deferred {
            match kind {
                MediaKind::Video => self.draw_video_slot(cx, *id, *rect, path),
                _ => {}
            }
        }
    }

    /// Draw a video item's content rect through a `Video` widget slot,
    /// handing the file path to the slot the first time the item is drawn.
    /// Playback is started explicitly (no DSL autoplay): autoplay would
    /// prepare the slot's empty startup source and crash the platform
    /// player on a nil URL.
    fn draw_video_slot(&mut self, cx: &mut Cx2d, id: u64, rect: Rect, path: &str) {
        let Some((slot, fresh)) = Self::alloc_slot(&mut self.media_video_slots, id, VIDEO_SLOTS)
        else {
            self.draw_slot_busy(cx, rect);
            return;
        };
        let slot_id = LiveId::from_str(&format!("video_slot_{}", slot));
        let widget = self.view.widget(cx.cx, &[slot_id]);
        let video = widget.as_video();
        if fresh {
            if !video.is_unprepared() {
                // Slot reuse: release the previous item's player first.
                video.stop_and_cleanup_resources(cx.cx);
            }
            video.set_source(VideoDataSource::Filesystem {
                path: path.to_string(),
            });
            video.begin_playback(cx.cx);
        }
        let walk = Walk {
            abs_pos: Some(rect.pos),
            width: Size::Fixed(rect.size.x),
            height: Size::Fixed(rect.size.y),
            ..Default::default()
        };
        let _ = widget.draw_walk(cx, &mut Scope::empty(), walk);
    }

    /// Draw a PDF item's content rect through a native `PdfPageView` slot
    /// (makepad's own PDF renderer, `pdf` feature). The file is parsed once
    /// into [`Self::pdf_pages`]; the first page is then letterboxed into the
    /// content rect. PdfPageView is a simple draw-call widget, so it is
    /// drawn off-flow like the video slots, in two phases:
    /// draw_walk (page paper) → render_page (ops) → draw_walk (finish).
    fn draw_pdf_slot(&mut self, cx: &mut Cx2d, id: u64, rect: Rect, path: &str) {
        if !self.pdf_pages.contains_key(&id) {
            let parsed = std::fs::read(path).ok().and_then(|data| {
                let mut doc = PdfDocument::parse(&data).ok()?;
                let page = doc.page(0).ok()?;
                let ops = parse_content_stream(&page.content_data).unwrap_or_default();
                Some(CachedPage::new(page, ops))
            });
            let Some(cached) = parsed else {
                self.draw_slot_busy(cx, rect);
                return;
            };
            self.pdf_pages.insert(id, cached);
        }
        let Some((slot, _fresh)) = Self::alloc_slot(&mut self.media_pdf_slots, id, PDF_SLOTS)
        else {
            self.draw_slot_busy(cx, rect);
            return;
        };
        // Letterbox the page inside the content rect.
        let page_size = self
            .pdf_pages
            .get(&id)
            .map(|c| c.size())
            .unwrap_or_default();
        if page_size.x <= 0.0 || page_size.y <= 0.0 {
            self.draw_slot_busy(cx, rect);
            return;
        }
        let zoom = (rect.size.x / page_size.x)
            .min(rect.size.y / page_size.y)
            .min(6.0);
        let draw_w = (page_size.x * zoom).max(1.0);
        let draw_h = (page_size.y * zoom).max(1.0);
        let page_rect = Rect {
            pos: rect.pos
                + Vec2d {
                    x: (rect.size.x - draw_w) * 0.5,
                    y: (rect.size.y - draw_h) * 0.5,
                },
            size: Vec2d {
                x: draw_w,
                y: draw_h,
            },
        };
        let slot_id = LiveId::from_str(&format!("pdf_slot_{}", slot));
        let widget = self.view.widget(cx.cx, &[slot_id]);
        let walk = Walk {
            abs_pos: Some(page_rect.pos),
            width: Size::Fixed(draw_w),
            height: Size::Fixed(draw_h),
            ..Default::default()
        };
        // Two-phase draw with the page ops rendered in between.
        let _ = widget.draw_walk(cx, &mut Scope::empty(), walk);
        if let Some(cached) = self.pdf_pages.get(&id) {
            if let Some(mut pv) = widget.borrow_mut::<PdfPageView>() {
                pv.render_page(cx, cached, zoom);
            }
        }
        let _ = widget.draw_walk(cx, &mut Scope::empty(), walk);
    }

    /// Placeholder for a media card whose widget slot pool is exhausted.
    fn draw_slot_busy(&mut self, cx: &mut Cx2d, rect: Rect) {
        self.draw_item_bg_rect(cx, rect, [0.09, 0.10, 0.13, 1.0]);
        self.draw_cell_text.color = vec4f([0.55, 0.60, 0.72, 1.0]);
        self.draw_cell_text.font_scale = self.camera.zoom;
        self.draw_cell_text.draw_abs(
            cx,
            rect.pos
                + Vec2d {
                    x: 12.0,
                    y: (rect.size.y * 0.5 - 8.0).max(8.0),
                },
            "All preview slots are busy — close another media card",
        );
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
        // Fixed-size card chrome: pin the text scale like the other cards.
        self.draw_title.font_scale = 1.0;
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

        // Play / pause button (circular, matching the round avatar chip).
        let btn_size = 36.0;
        let btn_c = Vec2d {
            x: inner_x + btn_size * 0.5,
            y: body_y + body_h * 0.5,
        };
        self.draw_filled_disc(
            cx,
            btn_c,
            btn_size * 0.5,
            if playing {
                MUSIC_ACCENT
            } else {
                MUSIC_PROGRESS_BG
            },
        );
        self.draw_sketch_ellipse(
            cx,
            btn_c,
            btn_size * 0.5 - 0.5,
            btn_size * 0.5 - 0.5,
            1.0,
            MUSIC_BORDER,
            id as u32,
            0.35,
        );
        self.draw_title.color = vec4f(MUSIC_TEXT);
        let icon = if playing { "❚❚" } else { "▶" };
        self.draw_title.draw_abs(
            cx,
            Vec2d {
                x: btn_c.x - 8.0,
                y: btn_c.y - 9.0,
            },
            icon,
        );

        // Progress bar to the right of the button (rounded ends).
        let bar_x = btn_c.x + btn_size * 0.5 + 14.0;
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
            // Rounded fill: a filled bar plus a disc at the fill's right end.
            let fill_rect = Rect {
                pos: bar_bg.pos,
                size: Vec2d {
                    x: fill_w,
                    y: bar_h,
                },
            };
            self.draw_item_bg_rect(cx, fill_rect, MUSIC_ACCENT);
            let cap = Vec2d {
                x: bar_x + fill_w,
                y: bar_y + bar_h * 0.5,
            };
            self.draw_filled_disc(cx, cap, bar_h * 0.5, MUSIC_ACCENT);
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

        // The palette sees its keys first: ⌘K toggles it, and while it is up
        // the arrows/Tab/Escape belong to its list, not to the shell behind.
        if let Event::KeyDown(key) = event {
            // ⌘K (Ctrl+K off Apple) summons the palette from anywhere — the
            // platform's `is_primary` is what keeps Ctrl+K free for a focused
            // shell's own binding.
            if key.modifiers.is_primary() && key.key_code == KeyCode::KeyK {
                if self.command_open {
                    self.close_command_palette(cx);
                } else {
                    self.open_command_palette(cx);
                }
                return;
            }
            if self.command_open {
                match key.key_code {
                    KeyCode::ArrowDown => {
                        self.suggestion_index += 1;
                        self.update_suggestions(cx);
                        return;
                    }
                    KeyCode::ArrowUp => {
                        if self.suggestion_index > 0 {
                            self.suggestion_index -= 1;
                        }
                        self.update_suggestions(cx);
                        return;
                    }
                    KeyCode::Tab => {
                        self.accept_suggestion_at(cx, self.suggestion_index);
                        return;
                    }
                    KeyCode::Escape => {
                        self.close_command_palette(cx);
                        return;
                    }
                    KeyCode::ReturnKey => {
                        // An empty query picks the highlighted command rather
                        // than sending a bare newline to whatever is behind.
                        if self.view.text_input(cx, input_id).text().trim().is_empty() {
                            self.accept_suggestion_at(cx, self.suggestion_index);
                            return;
                        }
                    }
                    _ => {}
                }
            }
            // Ctrl+Up/Down browses command history when the command input is focused.
            let ctrl = key.modifiers.control;
            let input_focused = self.command_open && self.view.text_input(cx, input_id).key_focus(cx);
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

        // Pointer left the window: drop any pending palette tooltip.
        if let Event::MouseLeave(_) = event {
            if self.palette_hover.is_some() || self.palette_tip_visible {
                self.palette_hover = None;
                self.palette_tip_visible = false;
                self.palette_tip_timer = None;
                self.redraw(cx);
            }
        }

        // File drag & drop: while dragged files hover the canvas, highlight
        // it; on drop, spawn a media card per supported file at the drop
        // point (image / video / PDF, classified by extension).
        match event.drag_hits(cx, self.area) {
            DragHit::Drag(f) => {
                let has_files = f
                    .items
                    .iter()
                    .any(|i| matches!(i, DragItem::FilePath { .. }));
                let hovering = has_files
                    && matches!(
                        f.state,
                        makepad_widgets::DragState::In | makepad_widgets::DragState::Over
                    );
                // Claim the drag: dragging_updated returns NSDragOperation::None
                // unless a widget writes a DragResponse while handling the Drag
                // event, and with None macOS never fires performDragOperation
                // (i.e. Event::Drop is never delivered).
                if hovering {
                    *f.response.lock().unwrap() = DragResponse::Copy;
                }
                if hovering != self.drop_hover {
                    self.drop_hover = hovering;
                    if hovering {
                        self.status(
                            cx,
                            "Drop to preview on the canvas (image · video · PDF · text)",
                        );
                    }
                    self.redraw(cx);
                }
            }
            DragHit::Drop(f) => {
                self.drop_hover = false;
                let paths: Vec<String> = f
                    .items
                    .iter()
                    .filter_map(|i| match i {
                        DragItem::FilePath { path, .. } => Some(path.clone()),
                        _ => None,
                    })
                    .collect();
                if !paths.is_empty() {
                    self.drop_files(cx, &paths, f.abs);
                }
            }
            DragHit::DragEnd => {
                if self.drop_hover {
                    self.drop_hover = false;
                    self.redraw(cx);
                }
            }
            _ => {}
        }

        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Re-take focus the hidden inputs just lost to the empty-rect
        // MouseUp rule, when the click belonged to their hosting card.
        // Actions arrive after every event handler, so a re-assert here is
        // final for this event round - no later handler can clear it again.
        if self.composer_focus_repair || self.note_focus_repair {
            let composer_uid = self.view.text_input(cx, ids!(agent_composer)).widget_uid();
            let note_uid = self.view.text_input(cx, ids!(note_editor)).widget_uid();
            for item in actions
                .iter()
                .filter_map(|a| a.downcast_ref::<WidgetAction>())
            {
                if !matches!(
                    item.action.downcast_ref::<TextInputAction>(),
                    Some(TextInputAction::KeyFocusLost)
                ) {
                    continue;
                }
                if self.composer_focus_repair && item.widget_uid == composer_uid {
                    self.view
                        .text_input(cx, ids!(agent_composer))
                        .take_key_focus(cx);
                    self.composer_focus_repair = false;
                    self.redraw(cx);
                } else if self.note_focus_repair && item.widget_uid == note_uid {
                    self.view
                        .text_input(cx, ids!(note_editor))
                        .take_key_focus(cx);
                    self.note_focus_repair = false;
                    self.redraw(cx);
                }
            }
        }

        // Snap video cards to their clip's aspect once playback prepares.
        for item in actions
            .iter()
            .filter_map(|a| a.downcast_ref::<WidgetAction>())
        {
            if !matches!(
                item.action.downcast_ref::<VideoAction>(),
                Some(VideoAction::PlaybackPrepared)
            ) {
                continue;
            }
            let Some((vw, vh)) = self.video_prep_queue.first().copied() else {
                continue;
            };
            self.video_prep_queue.remove(0);
            let slot = (0..VIDEO_SLOTS).find(|&s| {
                let slot_id = LiveId::from_str(&format!("video_slot_{}", s));
                self.view.widget(cx, &[slot_id]).widget_uid() == item.widget_uid
            });
            let Some(slot) = slot else { continue };
            let Some(item_id) = self
                .media_video_slots
                .iter()
                .find(|(_, s)| *s == slot)
                .map(|(i, _)| *i)
            else {
                continue;
            };
            self.resize_media_to_aspect(cx, item_id, vw as f64, vh as f64);
        }
        // Drop stale entries (a prepare whose action never fired) so the
        // FIFO pairing can't drift.
        self.video_prep_queue.clear();

        // Sync the hidden note editor with the canvas-drawn buffer.
        self.sync_note_editor(cx);

        // Refresh suggestions whenever the command input text may have changed.
        self.update_suggestions(cx);

        // Palette tooltip show delay elapsed while still hovering.
        if let Some(t) = self.palette_tip_timer {
            if t.is_event(event).is_some() {
                if self.palette_hover.is_some() && !self.palette_tip_visible {
                    self.palette_tip_visible = true;
                    self.redraw(cx);
                }
            }
        }

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

        if let Event::MouseMove(me) = event {
            // Track the agent Stop button so it can show a pressed-ready tint.
            let stop = self.agent_stop_under(me.abs);
            if stop != self.hovered_stop {
                self.hovered_stop = stop;
                self.redraw(cx);
            }
        }

        if let Event::MouseDown(me) = event {
            if me.button.contains(MouseButton::PRIMARY) {
                // The palette is modal. A press on a row accepts it, a press
                // inside the panel refocuses the input, and a press anywhere
                // else dismisses it — in every case the canvas and the cards
                // underneath stay untouched, and the widget dispatch at the
                // bottom of this handler is skipped.
                if self.command_open {
                    if let Some(index) = self.palette_row_under(cx, me.abs) {
                        self.accept_suggestion_at(cx, index);
                    } else if self
                        .palette_rect(cx)
                        .is_some_and(|rect| rect.contains(me.abs))
                    {
                        self.focus_command_input(cx);
                    } else {
                        self.close_command_palette(cx);
                    }
                    return;
                }
                if std::env::var_os("CANVAS_TRACE_INPUT").is_some() {
                    // Same shape as MAKEPAD_TRACE_FONT_LOAD: what the press hit,
                    // in the draw space the app and the pointer share.
                    log!(
                        "down abs=({:.1},{:.1}) top_bar={:?} ctrl={:?} chat={:?} item={:?}",
                        me.abs.x,
                        me.abs.y,
                        self.top_bar_hit(me.abs),
                        self.control_button_under(me.abs),
                        self.chat_switch_under(me.abs),
                        self.hit_test(me.abs),
                    );
                }
                self.last_mouse = me.abs;
                // A click inside the card/note being edited keeps its hidden
                // input focused: mark this click for the focus repair, since
                // the MouseUp would otherwise clear it (empty-rect rule).
                if self.agent_composer_id.is_some() || self.note_edit_id.is_some() {
                    let hosting = self
                        .items
                        .iter()
                        .find(|i| Some(i.id()) == self.agent_composer_id.or(self.note_edit_id));
                    if let Some(item) = hosting {
                        if self.item_screen_rect(item).contains(me.abs) {
                            self.composer_focus_repair = self.agent_composer_id.is_some();
                            self.note_focus_repair = self.note_edit_id.is_some();
                        }
                    }
                }
                // Any press hides the palette tooltip (the click may switch
                // the tool, and a stale bubble shouldn't linger).
                if self.palette_tip_visible {
                    self.palette_tip_visible = false;
                    self.redraw(cx);
                }
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
                // Top bar: workspace tabs, new-space, and the parked cards
                // (a tab restores its card, its ✕ closes the card).
                if let Some(hit) = self.top_bar_hit(me.abs) {
                    match hit {
                        TopBarHit::Workspace(i) => self.switch_workspace(cx, i),
                        TopBarHit::AddWorkspace => self.add_workspace(cx),
                        TopBarHit::Card(id) => self.restore_item(cx, id),
                        TopBarHit::CardClose(id) => self.close_item(cx, id),
                    }
                    return;
                }
                // An open agent composer commits when clicking outside its
                // card, exactly like the note editor does.
                if let Some(compose_id) = self.agent_composer_id {
                    match self.hit_test(me.abs) {
                        Some(id) if id == compose_id => {}
                        _ => {
                            self.finish_agent_composer(cx);
                        }
                    }
                }
                // View switch (◎ / ▮): flip terminal <-> chat card.
                if let Some((id, to_chat)) = self.chat_switch_under(me.abs) {
                    self.toggle_chat_view(cx, id, to_chat);
                    return;
                }
                // Agent Stop button: abort the running turn.
                // The Stop button sits above the card body, so it wins over
                // selecting the card.
                if let Some(id) = self.agent_stop_under(me.abs) {
                    if let Some(session) = self
                        .items
                        .iter()
                        .find(|i| i.id() == id)
                        .and_then(|i| i.agent_session())
                    {
                        // Ctrl-C on the PTY: how a user interrupts the CLI.
                        session.write_bytes(&[3]);
                        self.status(cx, "cancelled");
                        self.hovered_stop = None;
                    }
                    return;
                }
                // Agent Stop button hover is derived in the Move handler; a
                // press lands directly below.
                if let Some((id, kind)) = self.control_button_under(me.abs) {
                    match kind {
                        BtnKind::Minimize => self.minimize_item(cx, id),
                        BtnKind::Close => self.close_item(cx, id),
                        BtnKind::Print => self.print_pdf(cx, id),
                    }
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
                            self.status(cx, &format!("Tool: {}", self.tool.label()));
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
                    // A click on a note's checklist box toggles that item in
                    // place — notes' `toggle_checkbox`: the markdown source *is*
                    // the document, so the edit happens there and the card
                    // re-renders from it. Published by the last draw pass, since
                    // measuring a row needs the `Cx2d` only drawing has.
                    if let Some(src_line) = self.note_check_hit(id, me.abs) {
                        if let Some(next) = self
                            .items
                            .iter()
                            .find(|i| i.id() == id)
                            .and_then(|i| i.body())
                            .and_then(|body| crate::note::toggle_checkbox(body, src_line))
                        {
                            self.set_note_body(cx, id, next);
                            self.selected = Some(id);
                            return;
                        }
                    }
                    self.selected = Some(id);
                    // Selecting a non-terminal item must clear any stale
                    // terminal focus so the property panel keys don't leak
                    // into the shell.
                    if let Some(item) = self.items.iter().find(|i| i.id() == id) {
                        if item.kind() != ItemKind::Terminal {
                            self.focused_terminal = None;
                        }
                        // Clicking an agent card's input line focuses it for
                        // typing (double-click anywhere also opens it). No-op
                        // when that composer is already open, so re-clicking
                        // the line never wipes a half-typed draft.
                        if item.kind() == ItemKind::Agent && self.agent_composer_id != Some(id) {
                            if Self::agent_composer_rect(self.item_screen_rect(item))
                                .contains(me.abs)
                            {
                                self.start_agent_composer(cx, id);
                            }
                        }
                    }
                    // Double-click a note to edit it inline, or an agent card
                    // to open its composer.
                    if is_double {
                        let kind = self.items.iter().find(|i| i.id() == id).map(|i| i.kind());
                        match kind {
                            Some(ItemKind::Note) => {
                                self.start_note_edit(cx, id);
                                return;
                            }
                            Some(ItemKind::Agent) => {
                                self.start_agent_composer(cx, id);
                                return;
                            }
                            _ => {}
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
                    }
                    // Video media card: clicks in the content area belong to
                    // the embedded Video widget (play/pause/seek/volume);
                    // starting a card drag here would fight its controls. The
                    // title bar still drags the card.
                    else if self.is_media_content(id, me.abs) {
                        // Selection already set above; nothing else to do.
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
                            // Clicking the content area focuses the terminal too,
                            // so keys follow the click instead of staying with
                            // whatever held the keyboard.
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
                    // UI overlays (the right panel, the palette) stay
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
                let hov_tab = self.top_bar_hit(me.abs);
                // Move tool: track which shape is under the cursor for
                // highlight feedback.
                let hov_shape = if self.tool == NoteTool::Move {
                    self.shape_under(me.abs)
                } else {
                    None
                };
                let hov_pal = self.palette_hit(me.abs);
                if hov != self.hovered
                    || hov_btn != self.hovered_btn
                    || hov_tab != self.hovered_tab
                    || hov_shape != self.hovered_shape
                    || hov_pal != self.palette_hover
                {
                    self.hovered = hov;
                    self.hovered_btn = hov_btn;
                    self.hovered_tab = hov_tab;
                    self.hovered_shape = hov_shape;
                    if hov_pal != self.palette_hover {
                        self.palette_hover = hov_pal;
                        // Moving between buttons restarts the show delay
                        // (same as MpTooltip's hover restart).
                        self.palette_tip_visible = false;
                        self.palette_tip_timer = if hov_pal.is_some() {
                            Some(cx.start_timeout(TIP_SHOW_DELAY))
                        } else {
                            None
                        };
                    }
                    self.redraw(cx);
                }
            }
        }

        if let Event::MouseUp(me) = event {
            if me.button.contains(MouseButton::PRIMARY) {
                self.drag = None;
                self.panning = false;
                // The hidden composer/note inputs are zero-size, and makepad
                // clears key focus when a MouseUp lands outside the focused
                // input's rect - which for an empty rect is every MouseUp,
                // including the one ending the click that opened them. This
                // handler runs after the widget dispatch (which already did
                // the clearing) and nothing later in this event can clear
                // again, so re-taking here is final - and it happens before
                // the user can type, so no keystroke is lost to the repair.
                if self.agent_composer_id.is_some() {
                    self.view
                        .text_input(cx, ids!(agent_composer))
                        .take_key_focus(cx);
                    self.redraw(cx);
                }
                if self.note_edit_id.is_some() {
                    self.view
                        .text_input(cx, ids!(note_editor))
                        .take_key_focus(cx);
                    self.redraw(cx);
                }
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

        // Video dimensions arrive on the platform prepared event; the
        // VideoAction::PlaybackPrepared for the same widget is captured from
        // the children below (same iteration order) and applies them.
        if let Event::VideoPlaybackPrepared(ev) = event {
            self.video_prep_queue
                .push((ev.video_width as usize, ev.video_height as usize));
        }

        if let Event::Scroll(se) = event {
            if std::env::var_os("CANVAS_TRACE_SCROLL").is_some() {
                log!(
                    "scroll abs=({:.1},{:.1}) dy={:.1} is_mouse={} over_note={}",
                    se.abs.x,
                    se.abs.y,
                    se.scroll.y,
                    se.is_mouse,
                    self.items.iter().rev().any(|i| {
                        i.kind() == ItemKind::Note && self.item_screen_rect(i).contains(se.abs)
                    })
                );
            }
            // The topmost card under the cursor owns the wheel. Without this the
            // per-kind handlers below were tried in a fixed order, so a note card
            // lying on top of a terminal scrolled the terminal underneath it.
            let topmost = self
                .items
                .iter()
                .rev()
                .find(|i| self.item_screen_rect(i).contains(se.abs))
                .map(|i| i.id());
            // Transcript: over an agent card, wheel scrolls the conversation
            // history instead of panning the canvas.
            let agent_under = self.items.iter().rev().find(|i| {
                i.kind() == ItemKind::Agent
                    && Some(i.id()) == topmost
                    && self.item_screen_rect(i).contains(se.abs)
            });
            if let Some(item) = agent_under {
                let id = item.id();
                let rows: usize = item
                    .agent_session()
                    .and_then(|c| c.chat.lock().ok())
                    .map(|card| card.rows().len().max(1))
                    .unwrap_or(1);
                let delta = (se.scroll.y / 20.0).round();
                let current = self.agent_scroll.get(&id).copied().unwrap_or(0.0);
                // Wheel up (negative y) moves into history: offset grows.
                let next = (current - delta).clamp(0.0, rows as f64);
                if (next - current).abs() > f64::EPSILON {
                    if next <= f64::EPSILON {
                        self.agent_scroll.remove(&id);
                    } else {
                        self.agent_scroll.insert(id, next);
                    }
                    self.redraw(cx);
                }
                return;
            }
            // Scrollback: if the cursor is over a terminal's content area,
            // scroll the terminal history instead of panning/zooming the
            // canvas (like alacritty/wezterm).
            let term_under = self.items.iter().rev().find(|i| {
                i.kind() == ItemKind::Terminal
                    && Some(i.id()) == topmost
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
            // Over a PDF card's content area: the embedded PdfView scrolls
            // its pages; don't pan/zoom the canvas underneath.
            let pdf_under = self.items.iter().rev().find(|i| {
                i.media_kind() == Some(MediaKind::Pdf)
                    && Some(i.id()) == topmost
                    && self.item_screen_rect(i).contains(se.abs)
                    && se.abs.y > self.item_screen_rect(i).pos.y + 26.0
            });
            if pdf_under.is_some() {
                return;
            }
            // Over a text card's content area: scroll the text preview
            // (offset clamped at draw time to the line count).
            let text_under = self.items.iter().rev().find(|i| {
                i.media_kind() == Some(MediaKind::Text)
                    && Some(i.id()) == topmost
                    && self.item_screen_rect(i).contains(se.abs)
                    && se.abs.y > self.item_screen_rect(i).pos.y + 26.0
            });
            if let Some(item) = text_under {
                let delta = (se.scroll.y / 20.0).round();
                if delta != 0.0 {
                    let entry = self.text_scroll.entry(item.id()).or_insert(0.0);
                    *entry = (*entry + delta).max(0.0);
                    self.redraw(cx);
                }
                return;
            }
            // Over a note card's body: scroll the note (a note can be taller
            // than its card). Only claims the gesture when the body actually
            // overflows, so the wheel still zooms the canvas over short notes.
            let note_under = self.items.iter().rev().find(|i| {
                i.kind() == ItemKind::Note
                    && Some(i.id()) == topmost
                    && self.item_screen_rect(i).contains(se.abs)
                    && se.abs.y > self.item_screen_rect(i).pos.y + 28.0
            });
            if let Some(item) = note_under {
                let id = item.id();
                if std::env::var_os("CANVAS_TRACE_SCROLL").is_some() {
                    log!(
                        "scroll note id={id} max={:?} offset={:?}",
                        self.note_scroll_max.get(&id),
                        self.note_scroll.get(&id)
                    );
                }
                if self.note_scroll_max.get(&id).copied().unwrap_or(0.0) > 0.0 {
                    let delta = (se.scroll.y / 20.0).round();
                    if delta != 0.0 {
                        // One prose line per notch, in world units.
                        let step = item.note_font_size().unwrap_or(13.0) as f64 * 1.36;
                        let entry = self.note_scroll.entry(id).or_insert(0.0);
                        *entry = (*entry + delta * step).max(0.0);
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
            // The agent composer takes priority over terminal input, mirroring
            // the note editor: Return submits, Escape cancels.
            if self.agent_composer_id.is_some() {
                let composer = self.view.text_input(cx, ids!(agent_composer));
                if !composer.key_focus(cx) {
                    // Safety net behind the MouseUp re-take: a key while
                    // composing means the focus belongs to the composer.
                    composer.take_key_focus(cx);
                }
                match key.key_code {
                    KeyCode::ReturnKey => {
                        self.finish_agent_composer(cx);
                    }
                    KeyCode::Escape => {
                        self.cancel_agent_composer(cx);
                    }
                    _ => {}
                }
                // Typing reaches the hidden proxy through the text-input path;
                // mirror its text for the canvas to draw.
                if let Some(text) = {
                    let composer = self.view.text_input(cx, ids!(agent_composer));
                    let text = composer.text();
                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                } {
                    self.agent_input = text;
                }
                self.redraw(cx);
            } else if self.text_editing {
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

        // Unified command input: the palette runs a command and puts itself
        // away, like every other palette — ⌘K brings it back.
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
            self.close_command_palette(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Canvas background
        let rect = cx.walk_turtle_with_area(&mut self.area, walk);
        self.viewport = rect.size;
        self.viewport_pos = rect.pos;
        if rect.size.x <= 1.0 || rect.size.y <= 1.0 {
            return DrawStep::done();
        }

        if self.timer.is_none() {
            self.timer = Some(cx.cx.start_interval(0.05));
            // Nothing holds the keyboard on a fresh window, and makepad only
            // delivers key events to the focused area: give it to the canvas,
            // so a restored card sees typing without a click first.
            self.set_canvas_focus(cx);
        }

        self.ensure_workspace();
        // The palette owns the keyboard while it is up. A field that was laid
        // out while the wrap was hidden cannot keep the focus taken on open —
        // makepad drops it — so re-take it whenever the keyboard ends up
        // unowned. Idempotent: the moment it holds, this stops firing.
        if self.command_open && cx.key_focus().is_empty() {
            let input_id = ids!(
                command_wrap
                    .command_bar
                    .input_row
                    .input_capsule
                    .command_input
            );
            self.view.text_input(cx, input_id).set_key_focus(cx);
        }
        // Measure the mono cell from the face before anything lays out text:
        // the grid, the PTY resize and the hit-tests all read it.
        self.refresh_cell_metrics(cx);
        self.draw_grid(cx, rect);
        self.draw_empty_hint(cx, rect);
        if self.drop_hover {
            // Inset glow frame signals the canvas accepts the pending drop.
            let inset = Rect {
                pos: rect.pos + Vec2d { x: 14.0, y: 14.0 },
                size: rect.size - Vec2d { x: 28.0, y: 28.0 },
            };
            self.draw_glow_border(cx, inset, [0.30, 0.62, 0.98, 1.0]);
        }

        // Draw items (world → screen) by index; extract item data first to
        // avoid borrowing self.items while mutating self.
        type DrawItem = (
            u64,
            ItemKind,
            Rect,
            bool,
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
            Option<(MediaKind, String)>,
            Option<std::sync::Arc<std::sync::Mutex<crate::chat::ChatCardState>>>,
            String,
            String,
            i64,
        );
        let mut chat_sigs: HashMap<u64, (Option<String>, Option<String>)> = HashMap::new();
        let n_items = self.items.len();
        let mut draw_queue: Vec<DrawItem> = Vec::new();
        // Fresh every frame: the note draws below publish their checkbox hit
        // areas and overflow for this frame's input handling.
        self.note_check_rows.clear();
        self.note_scroll_max.clear();
        for idx in 0..n_items {
            let item = &self.items[idx];
            let screen = self.item_screen_rect(item);
            if !screen.intersects(rect) {
                continue;
            }
            let is_sel = self.selected == Some(item.id());
            let is_hovered = self.hovered == Some(item.id());
            let command = item
                .session()
                .map(|t| t.command.clone())
                .unwrap_or_default();
            let url = item.url().unwrap_or("").to_string();
            let state = item.session().map(|t| t.state.clone());
            let agent_state = item.agent_session().map(|a| a.chat.clone());
            if let Some(session) = item.agent_session() {
                if let Ok(card) = session.chat.lock() {
                    chat_sigs.insert(item.id(), (card.session_id.clone(), card.usage.clone()));
                }
            }
            let agent_cwd = item.agent_cwd().unwrap_or("").to_string();
            let agent_provider = item.agent_provider().unwrap_or("").to_string();
            let (body, font_size, color_idx, edited_ms) = match item {
                CanvasItem::Note {
                    body,
                    font_size,
                    color_idx,
                    edited_ms,
                    ..
                } => (body.clone(), *font_size, *color_idx, *edited_ms),
                _ => (String::new(), 13.0, 0, 0),
            };
            let (progress, playing) = match item {
                CanvasItem::MusicPlayer {
                    progress, playing, ..
                } => (*progress, *playing),
                _ => (0.0f32, false),
            };
            let media = match item {
                CanvasItem::Media { path, kind, .. } => Some((*kind, path.clone())),
                _ => None,
            };
            let status = item.agent_status().unwrap_or_default();
            draw_queue.push((
                item.id(),
                item.kind(),
                screen,
                is_sel,
                is_hovered,
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
                media,
                agent_state,
                agent_cwd,
                agent_provider,
                edited_ms,
            ));
        }

        // Video/PDF cards draw after the child pass (see draw_deferred_media).
        let mut deferred_media: Vec<(u64, Rect, MediaKind, String)> = Vec::new();
        for (
            item_id,
            kind,
            item_screen,
            is_sel,
            is_hovered,
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
            media,
            agent_state,
            agent_cwd,
            agent_provider,
            edited_ms,
        ) in draw_queue
        {
            // Subtle hover backlight so the card under the mouse is clear,
            // without competing with the selected-card glow.
            if is_hovered && !is_sel {
                self.draw_glow_border(cx, item_screen, [0.62, 0.72, 0.92, 1.0]);
            }
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
                        let cols = (content_w / self.cell_w()).floor().max(10.0) as usize;
                        let rows = (content_h / self.cell_h()).floor().max(3.0) as usize;
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
                        // View switch: turn this terminal into a chat card.
                        self.draw_chat_switch(cx, item_screen, true);
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
                        edited_ms,
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
                ItemKind::Agent => {
                    if let Some(card) = agent_state {
                        // Track CLI-reported metadata for the autosave.
                        if let Ok(card) = card.lock() {
                            chat_sigs
                                .insert(item_id, (card.session_id.clone(), card.usage.clone()));
                        }
                        // A gap in the sequence stream means the card is missing
                        // transcript; repair it from the daemon's journal
                        // before drawing, so the user never sees a hole.
                        let needs_reload = card
                            .lock()
                            .map(|card| card.gap().is_some())
                            .unwrap_or(false);
                        if needs_reload {
                            if let Some(session) = self
                                .items
                                .iter()
                                .find(|i| i.id() == item_id)
                                .and_then(|i| i.agent_session())
                            {
                                session.chat_sync();
                            }
                        }
                        // The composer line is drawn only for the selected
                        // card, and the Stop button reads its hover state.
                        let composing = self.agent_composer_id == Some(item_id);
                        let composer_text = if composing {
                            let proxy = self.view.text_input(cx, ids!(agent_composer));
                            let typed = proxy.text();
                            if typed.is_empty() {
                                self.agent_input.clone()
                            } else {
                                typed
                            }
                        } else {
                            String::new()
                        };
                        let stop_hover = self.hovered_stop == Some(item_id);
                        self.draw_agent_at(
                            cx,
                            item_id,
                            item_screen,
                            &title,
                            &agent_cwd,
                            &agent_provider,
                            is_sel,
                            &card,
                            Some(composer_text.as_str()),
                            stop_hover,
                        );
                        // View switch: turn this chat card back into a grid.
                        self.draw_chat_switch(cx, item_screen, false);
                    }
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
                ItemKind::Media => {
                    if let Some((mkind, mpath)) = &media {
                        if let Some(deferred) =
                            self.draw_media_at(cx, item_id, item_screen, is_sel, *mkind, mpath)
                        {
                            deferred_media.push(deferred);
                        }
                    }
                    self.draw_control_buttons(cx, item_id, item_screen);
                    if is_sel {
                        self.draw_resize_handle(cx, item_screen);
                    }
                }
            }
        }

        // Right-side properties panel reflects the current selection.
        self.sync_properties_panel(cx);

        // A chat card learned its CLI session id or usage since the last
        // save: re-save so a restart resumes with fresh metadata.
        if chat_sigs != self.saved_chat_sig {
            self.saved_chat_sig = chat_sigs;
            self.save_canvas();
        }

        // Global whiteboard shapes (world coords) draw ON TOP of items so
        // annotations/flowcharts can mark terminals and browsers.
        let shapes = self.shapes.clone();
        let pending = self.pending.clone();
        self.draw_canvas_shapes(cx, rect.size, &shapes, pending.as_ref());

        // Children (the palette, the status label, the popup menu).
        while self.view.draw_walk(cx, scope, walk).step().is_some() {}

        // Video/PDF preview slots draw after the child pass so each widget's
        // recorded area (its controls'/scroll view's hit rect) points at the
        // item rect, not at its 0×0 flow-pass slot.
        if !deferred_media.is_empty() {
            let deferred = std::mem::take(&mut deferred_media);
            self.draw_deferred_media(cx, &deferred);
        }

        // The top bar (workspace tabs + the parked cards' tabs) sits on top of
        // the canvas items and the palette.
        self.draw_top_bar(cx);

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

/// Percent-encode a filesystem path into a `file://` URL for the embedded
/// browser (HTML preview). Unreserved characters and `/` pass through; all
/// other bytes are %XX-escaped so spaces/`#`/`?` in filenames survive.
fn file_url(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 8);
    out.push_str("file://");
    for b in path.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_url_encodes_special_characters() {
        assert_eq!(file_url("/tmp/a.html"), "file:///tmp/a.html");
        assert_eq!(
            file_url("/Users/me/My Page.html"),
            "file:///Users/me/My%20Page.html"
        );
        assert_eq!(file_url("/tmp/a#b.html"), "file:///tmp/a%23b.html");
    }

    #[test]
    fn fitted_content_size_keeps_aspect_within_bounds() {
        // 16:9 landscape fills the width bound.
        let (w, h) = fitted_content_size(1280.0, 720.0).unwrap();
        assert!((w - 560.0).abs() < 0.01 && (h - 315.0).abs() < 0.01);
        // Small 4:3 images upscale to a usable card.
        let (w, h) = fitted_content_size(64.0, 48.0).unwrap();
        assert!((w - 533.33).abs() < 0.01 && (h - 400.0).abs() < 0.01);
        // Tall images clamp at the height bound (slight aspect drift is ok,
        // the widget letterboxes the remainder).
        let (w, h) = fitted_content_size(400.0, 1200.0).unwrap();
        assert_eq!((w, h), (200.0, 400.0));
        // Degenerate input: no size.
        assert_eq!(fitted_content_size(0.0, 0.0), None);
    }

    #[test]
    fn checklist_click_lookup_matches_row_and_card() {
        let rows = vec![
            (
                7u64,
                Rect {
                    pos: Vec2d { x: 10.0, y: 20.0 },
                    size: Vec2d { x: 100.0, y: 18.0 },
                },
                3usize,
            ),
            (
                8u64,
                Rect {
                    pos: Vec2d { x: 10.0, y: 40.0 },
                    size: Vec2d { x: 100.0, y: 18.0 },
                },
                5usize,
            ),
        ];
        // Inside the first row's band, on that card.
        assert_eq!(
            note_check_line_at(&rows, 7, Vec2d { x: 50.0, y: 28.0 }),
            Some(3)
        );
        // Same point, other card: that card's own row answers.
        assert_eq!(
            note_check_line_at(&rows, 8, Vec2d { x: 50.0, y: 48.0 }),
            Some(5)
        );
        // Between rows (row 1 ends at y=38, row 2 starts at 40), outside the
        // card, and a card with no check rows.
        assert_eq!(note_check_line_at(&rows, 7, Vec2d { x: 50.0, y: 39.0 }), None);
        assert_eq!(note_check_line_at(&rows, 7, Vec2d { x: 200.0, y: 28.0 }), None);
        assert_eq!(note_check_line_at(&rows, 9, Vec2d { x: 50.0, y: 28.0 }), None);
    }

    #[test]
    fn alloc_slot_maps_items_and_reports_freshness() {
        let mut slots = Vec::new();
        assert_eq!(CanvasPanel::alloc_slot(&mut slots, 1, 2), Some((0, true)));
        assert_eq!(CanvasPanel::alloc_slot(&mut slots, 2, 2), Some((1, true)));
        // Existing mapping returns the same slot, not fresh.
        assert_eq!(CanvasPanel::alloc_slot(&mut slots, 1, 2), Some((0, false)));
        // Full pool: no silent eviction.
        assert_eq!(CanvasPanel::alloc_slot(&mut slots, 3, 2), None);
        // Closing an item frees its slot for reuse.
        slots.retain(|(i, _)| *i != 1);
        assert_eq!(CanvasPanel::alloc_slot(&mut slots, 3, 2), Some((0, true)));
    }
}
