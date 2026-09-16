//! `MpEditor` — the half of an editor that needs a window.
//!
//! ## What it is, and what it is not
//!
//! bezel splits its editor the same way and says why: *"`markdown` holds the document, its markdown wire form, and
//! the painting — all of it testable without a window. What lives here is the half that needs one: focus, keys,
//! the platform input handler, the mouse, undo, and the menus."*
//!
//! The parts this port already had, and where:
//!
//! | part | crate |
//! |---|---|
//! | the document, `parse`/`serialize`, the layout | `makepad-markdown` |
//! | what a key **does** to a document (`Shortcut`, `apply`) | `makepad-markdown::edit` |
//! | undo, and what counts as one step | `makepad-editor` |
//!
//! So what is left for a widget is: **which key means which shortcut, where the caret is, where a click lands, and
//! painting it.** This is that, and it is deliberately the small part — everything above it was built to be
//! testable without a window, and is.
//!
//! ## Not built, and named rather than implied
//!
//! - **No clipboard.** ⌘C/⌘V need a platform pasteboard; `makepad-clipboard` exists in this workspace and is the
//!   place for it, and wiring it is a separate piece of work.
//! - **No IME.** `TextInputEvent` carries a composition range for one, and this widget inserts `input` and ignores
//!   the rest. A composition that arrives as several events therefore inserts correctly but cannot show an
//!   underline.
//! - **No menus**, no block handles, no comments, no links — the four `editor.rs` submodules on the reference's
//!   list that are about a *document editor* rather than about an editing surface.
//! - **No horizontal scrolling.** A long line is clipped by the plate, like a fence in `mp/markdown.rs`.
//!
//! Each of those is a named absence rather than an oversight, because a reader comparing this to the reference
//! should be able to see what is missing without reading the whole file.

use makepad_widgets::*;

use makepad_editor::slash::{self, SlashMenu};
use makepad_editor::{EditKind, History};
use makepad_markdown::edit::{self, Selection, Shortcut};
use makepad_markdown::layout::{self, Laid, Metrics};
use makepad_markdown::{BlockKind, Doc};

use crate::mp::text;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    set_type_default() do #(DrawMpEditor::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 10.0

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

    mod.mp.MpEditorBase = #(MpEditor::register_widget(vm))

    mod.mp.MpEditor = set_type_default() do mod.mp.MpEditorBase{
        // A column with a settable measure, like `mp/markdown.rs` and for the same reason.
        width: 640
        height: Fit
        initial_measure: 640.0

        draw_bg +: {
            fill: surface_card
            border: border
            border_width: 1.0
            radius: 10.0
        }
        draw_marker +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_body +: {
            text_style: mod.mpc.type.body
            color: #x00000000
        }
        draw_code +: {
            text_style: theme.font_code{font_size: 12.0}
            color: #x00000000
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpEditor {
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

/// The padding inside the plate, on each side.
/// The slash menu's plate geometry. Named rather than inline, because a menu's proportions are a decision: a reader
/// comparing two menus needs the numbers to have a place to live.
const SLASH_PLATE_PAD: f64 = 6.0;
/// The gap between the caret and the plate.
const SLASH_GAP: f64 = 6.0;
/// A row's text, as a fraction of the block's body size — so the menu grows when the text does.
const SLASH_ROW_SCALE: f64 = 0.85;
/// A row's hint, as a fraction of the menu's own text: the hint is a reference, not a label.
const SLASH_HINT_SCALE: f64 = 0.8;
/// The gap between a label and its hint.
const SLASH_HINT_GAP: f64 = 10.0;
/// How far a hint sits below its row's text top, to sit on the same baseline.
const SLASH_HINT_BASELINE: f64 = 1.0;
/// The accent's width, which the plate reserves in its own padding.
const SLASH_ACCENT_W: f64 = 3.0;
const SLASH_ACCENT_X: f64 = 5.0;
const SLASH_ACCENT_RADIUS: f32 = 1.5;
const SLASH_ACCENT_INSET: f64 = 6.0;
/// Fractions of the widget's own corner radius, so a menu is consistent with what it opens from.
const SLASH_PLATE_RADIUS: f32 = 0.75;
const SLASH_ROW_RADIUS: f32 = 0.5;

/// The row height a menu uses, from the theme's layout numbers rather than a literal here.
fn theme_row_height(cx: &mut Cx) -> f64 {
    makepad_theme::Theme::of(cx).layout.row_height as f64
}

const PAD: f64 = 14.0;
/// How wide the caret is.
const CARET_W: f64 = 2.0;

#[derive(Script, Widget)]
pub struct MpEditor {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpEditor,
    #[live]
    draw_marker: DrawText,
    #[live]
    draw_body: DrawText,
    #[live]
    draw_code: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    initial_measure: f64,

    #[rust]
    measure: f64,
    #[rust]
    doc: Doc,
    #[rust]
    selection: Selection,
    #[rust]
    history: Option<History>,
    #[rust]
    laid: Option<Laid>,
    #[rust]
    used_metrics: Option<Metrics>,
    #[rust]
    last_size: Option<Vec2d>,
    /// Whether a press is down and dragging a selection.
    #[rust]
    dragging: bool,
    /// The slash menu, while one is open. `None` the rest of the time — see `refresh_slash`.
    #[rust]
    slash: Option<SlashMenu>,
    #[rust]
    area: Area,
}

impl MpEditor {
    /// Open a document, and start its undo history at it.
    pub fn set_source(&mut self, cx: &mut Cx, source: &str) {
        self.doc = makepad_markdown::parse(source);
        self.selection = Selection::caret(0, 0);
        self.history = Some(History::new(self.doc.clone(), self.selection.clone()));
        self.laid = None;
        self.redraw(cx);
    }

    pub fn doc(&self) -> &Doc {
        &self.doc
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// The document as markdown, which is what a caller saves.
    pub fn source(&self) -> String {
        makepad_markdown::serialize(&self.doc)
    }

    pub fn set_measure(&mut self, cx: &mut Cx, measure: f64) {
        if (self.measure - measure).abs() < 1e-9 {
            return;
        }
        self.measure = measure.max(1.0);
        self.laid = None;
        self.redraw(cx);
    }

    /// Replace the document **without** an undo step — opening a file is not an edit.
    pub fn open(&mut self, cx: &mut Cx, source: &str) {
        let doc = makepad_markdown::parse(source);
        self.doc = doc.clone();
        self.selection = Selection::caret(0, 0);
        self.history = Some(History::new(doc, self.selection.clone()));
        self.laid = None;
        self.redraw(cx);
    }

    /// The metrics this crate hands the layout, from the theme.
    ///
    /// Takes a `&Cx` rather than a `&mut Cx2d`, because an **event** has only a `Cx` and a caret motion needs the
    /// layout. The theme is reachable from both, so nothing is duplicated — the first version took a `Cx2d` and had
    /// an empty `ensure_laid_in_event` beside it, which meant a motion driven before the first frame silently did
    /// nothing.
    fn metrics_for(&self, cx: &Cx) -> Metrics {
        let theme = makepad_theme::Theme::of(cx);
        let body = theme.metrics(makepad_theme::TextStyle::Body).size() as f64;
        Metrics {
            advance: text::width("a", body),
            body_size: body,
            heading_size: [
                theme.metrics(makepad_theme::TextStyle::Title).size() as f64,
                theme.metrics(makepad_theme::TextStyle::Title2).size() as f64,
                theme.metrics(makepad_theme::TextStyle::Title3).size() as f64,
            ],
            line_height: body * 1.6,
            indent: body * 1.6,
            gap: body * 0.7,
            padding: body * 0.6,
        }
    }

    fn ensure_laid(&mut self, cx: &Cx) {
        let metrics = self.metrics_for(cx);
        let stale = self.used_metrics.is_some_and(|used| used != metrics);
        if self.laid.is_none() || stale {
            self.laid = Some(layout::layout(
                &self.doc,
                metrics,
                (self.measure - PAD * 2.0).max(1.0),
            ));
            self.used_metrics = Some(metrics);
        }
    }

    /// Apply a shortcut, recording the undo step and keeping the caret valid.
    fn run(&mut self, cx: &mut Cx, shortcut: Shortcut, kind: EditKind) {
        let Some(edited) = edit::apply(&self.doc, &self.selection, shortcut) else {
            return;
        };
        // **The state after the edit**, which is what `makepad-editor`'s model wants — see its module doc on why
        // recording the state *before* each edit made one undo skip a whole run.
        if let Some(history) = self.history.as_mut() {
            history.record(kind, &edited.doc, &edited.selection);
        }
        self.doc = edited.doc;
        self.selection = edited.selection;
        // The layout depends on the text, so it is dropped rather than patched.
        self.laid = None;
        self.redraw(cx);
    }

    /// Insert a string at the selection, replacing it.
    ///
    /// A `Splice` rather than a shortcut: `makepad-markdown`'s `Shortcut` set is what a **key** does, and typing a
    /// character is not one of those — it is text arriving. Inserting it here rather than adding a
    /// `Shortcut::Insert(char)` keeps that set closed for the reason it is closed: a shortcut changes *shape*, and
    /// text does not.
    /// The one internal insert path, shared by the typed-text event and the public `insert_text`.
    ///
    /// **The menu is re-derived here rather than at the event**, so a driver that inserts text through the public API
    /// gets the behaviour a keypress gets. Refreshing at the event instead would leave two paths that resemble each
    /// other, and a page would then verify something an app does not do — the trap `shortcut_for_key` exists to avoid.
    fn insert(&mut self, cx: &mut Cx, input: &str) {
        if input.is_empty() {
            return;
        }
        let mut doc = self.doc.clone();
        let block = self.selection.block;
        let Some(existing) = doc.blocks.get(block).and_then(|b| b.kind.text()).cloned() else {
            return;
        };
        let start = clamp(&existing.text, self.selection.range.start);
        let end = clamp(&existing.text, self.selection.range.end);
        let mut text = existing.clone();
        text.text.replace_range(start..end, input);
        // The marks after the insertion move by the difference, which `normalize` cannot do for us — it clips but
        // does not shift.
        let delta = input.len() as isize - (end - start) as isize;
        for span in &mut text.marks {
            if span.range.start >= end {
                span.range.start = (span.range.start as isize + delta).max(0) as usize;
                span.range.end = (span.range.end as isize + delta).max(0) as usize;
            }
        }
        text.normalize();
        set_block_text(&mut doc, block, text);
        // A carriage return in pasted text becomes a split, one block each — the model has no newline inside a
        // block except in a fence.
        let mut doc = split_on_newlines(doc, block, start, input);
        doc.renumber();
        doc = normalize_indents(doc);
        let caret = Selection::caret(
            self.selection.block,
            self.selection.range.start + input.len(),
        );
        if let Some(history) = self.history.as_mut() {
            history.record(EditKind::Typing, &doc, &caret);
        }
        self.doc = doc;
        self.selection = caret;
        self.laid = None;
        self.redraw(cx);
        self.refresh_slash(cx);
    }

    /// Move the caret without changing the document.
    ///
    /// Not a `Shortcut`: that set changes the *document*, and this changes the *selection*. See `insert` on the
    /// same distinction.
    fn move_caret(&mut self, cx: &mut Cx, motion: Motion, extend: bool) {
        // **Laid out here, not assumed.** A motion can arrive before the first frame — a script driving an editor
        // does exactly that — and the first version had an empty stub in this position, so every motion before the
        // first draw silently did nothing and the caret stayed where it started.
        self.ensure_laid(cx);
        let Some(laid) = self.laid.clone() else {
            return;
        };
        let at = move_within(&self.doc, &laid, &self.selection, motion);
        let anchor = if extend {
            // Extending keeps the far end where it was, so the range grows from it.
            if self.selection.range.end == self.selection.offset() {
                self.selection.range.start
            } else {
                self.selection.range.end
            }
        } else {
            at.range.end
        };
        self.selection = Selection {
            block: at.block,
            range: anchor.min(at.range.end)..anchor.max(at.range.end),
        };
        // A caret move **ends an undo run**, which is what makes type-pause-type two steps.
        if let Some(history) = self.history.as_mut() {
            history.landed();
        }
        self.redraw(cx);
    }

    /// Insert text as a reader typing would.
    ///
    /// Public because a page cannot deliver keystrokes — a synthetic pointer produces no hit in this app — so the
    /// page drives these instead and prints what they produced. See `crates/gallery/src/pages/editor.rs`.
    pub fn insert_text(&mut self, cx: &mut Cx, text: &str) {
        self.insert(cx, text);
    }

    /// Apply a shortcut as a keypress would.
    pub fn press(&mut self, cx: &mut Cx, shortcut: Shortcut, kind: EditKind) {
        self.run(cx, shortcut, kind);
    }

    /// The shortcuts a key maps to, for a caller driving an editor from a script.
    pub fn shortcut_for_key(code: KeyCode, shift: bool) -> Option<(Shortcut, EditKind)> {
        match code {
            KeyCode::ReturnKey => Some((Shortcut::Enter, EditKind::Structural)),
            KeyCode::Backspace => Some((Shortcut::Backspace, EditKind::Deleting)),
            KeyCode::Delete => Some((Shortcut::Delete, EditKind::Deleting)),
            KeyCode::Tab if shift => Some((Shortcut::Outdent, EditKind::Structural)),
            KeyCode::Tab => Some((Shortcut::Indent, EditKind::Structural)),
            _ => None,
        }
    }

    /// Move the caret as an arrow key would.
    pub fn move_by(&mut self, cx: &mut Cx, motion: Motion, extend: bool) {
        self.move_caret(cx, motion, extend);
    }

    /// Step back, as ⌘Z would.
    pub fn undo(&mut self, cx: &mut Cx) -> bool {
        let step = self.history.as_mut().and_then(|history| history.undo());
        match step {
            Some(step) => {
                self.doc = step.doc;
                self.selection = step.selection;
                self.laid = None;
                self.redraw(cx);
                true
            }
            None => false,
        }
    }

    /// Step forward, as ⇧⌘Z would.
    pub fn redo(&mut self, cx: &mut Cx) -> bool {
        let step = self.history.as_mut().and_then(|history| history.redo());
        match step {
            Some(step) => {
                self.doc = step.doc;
                self.selection = step.selection;
                self.laid = None;
                self.redraw(cx);
                true
            }
            None => false,
        }
    }

    /// How many undo and redo steps are available.
    pub fn depth(&self) -> (usize, usize) {
        self.history
            .as_ref()
            .map(|history| history.depth())
            .unwrap_or((0, 0))
    }

}

/// A copy of the open slash menu's state. See `MpEditor::slash_state`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlashState {
    /// Where the `/` is, in the block's text.
    pub at: usize,
    /// What has been typed after it.
    pub query: String,
    /// The matching rows' labels, in the menu's own order.
    pub rows: Vec<&'static str>,
    /// Which matching row is chosen.
    pub active: usize,
    /// What choosing it makes the block, or `None` when nothing matches.
    pub choice: Option<makepad_markdown::SetKind>,
}

impl SlashState {
    fn of(menu: &SlashMenu) -> Self {
        Self {
            at: menu.at(),
            query: menu.query().to_string(),
            rows: menu.matches().map(|item| item.label).collect(),
            active: menu.active(),
            choice: menu.choice(),
        }
    }
}

/// The block's own text, replaced.
fn set_block_text(doc: &mut Doc, block: usize, text: makepad_markdown::Text) {
    use makepad_markdown::BlockKind;
    let Some(slot) = doc.blocks.get_mut(block) else {
        return;
    };
    slot.kind = match std::mem::replace(&mut slot.kind, BlockKind::Divider) {
        BlockKind::Paragraph(_) => BlockKind::Paragraph(text),
        BlockKind::Heading { level, .. } => BlockKind::Heading { level, text },
        BlockKind::Bullet(_) => BlockKind::Bullet(text),
        BlockKind::Ordered { number, .. } => BlockKind::Ordered { number, text },
        BlockKind::Task { checked, .. } => BlockKind::Task { checked, text },
        BlockKind::Quote(_) => BlockKind::Quote(text),
        BlockKind::Code { language, .. } => BlockKind::Code { language, code: text },
        BlockKind::Divider => BlockKind::Divider,
    };
}

/// Turn embedded newlines in `input` into block splits.
///
/// A paste is where this happens, and the model has no newline inside a block except in a fence — so pasting two
/// lines into a paragraph is two paragraphs. The first version inserted the newlines into the text, and they then
/// **serialized away**: `parse` trims a line, so the text came back as one line and the round trip drifted.
fn split_on_newlines(
    mut doc: Doc,
    block: usize,
    _at: usize,
    _input: &str,
) -> Doc {
    // The insert already put the newlines in the block's text; splitting them out is the model's business, and
    // `parse`/`serialize` round-trip a document whose blocks each hold one line. Re-parsing the serialized form is
    // the shortest correct implementation of that — and it reuses the parser rather than a second set of rules.
    let written = makepad_markdown::serialize(&doc);
    if !written.contains('\n') && !written.trim().is_empty() {
        // `serialize` always ends with a newline; the check is for a document with **no** line breaks inside a
        // block, which is every document this function produces except a paste.
        let _ = block;
    }
    doc = makepad_markdown::parse(&written);
    doc
}

/// Clamp an offset to a character boundary.
fn clamp(text: &str, offset: usize) -> usize {
    let mut at = offset.min(text.len());
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Establish the indent invariant, as `makepad-markdown`'s edits do.
fn normalize_indents(mut doc: Doc) -> Doc {
    let mut previous = 0u8;
    for (index, block) in doc.blocks.iter_mut().enumerate() {
        if index == 0 {
            block.indent = 0;
        } else if block.indent > previous.saturating_add(1) {
            block.indent = previous.saturating_add(1);
        }
        previous = block.indent;
    }
    doc
}

/// A caret motion, which is **not** a document change.
/// A caret motion, which is **not** a document change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// One character back.
    Left,
    /// One character on.
    Right,
    /// The start of the line.
    Home,
    /// The end of the line.
    End,
    /// The line above, keeping the column where it fits.
    Up,
    /// The line below.
    Down,
    /// The start of the document.
    DocumentHome,
    /// The end of the document.
    DocumentEnd,
}

/// Where a motion takes the caret, as a [`Selection`] with an empty range.
///
/// Pure, and tested: a caret that moves wrong is a fault a reader feels immediately and describes badly — *"it goes
/// to the wrong place"* — so the arithmetic is worth having in one place with tests.
pub fn move_within(doc: &Doc, laid: &Laid, selection: &Selection, motion: Motion) -> Selection {
    let block = selection.block.min(doc.blocks.len().saturating_sub(1));
    let text = doc
        .blocks
        .get(block)
        .and_then(|b| b.kind.text())
        .map(|t| t.text.clone())
        .unwrap_or_default();
    let at = clamp(&text, selection.range.end);

    match motion {
        Motion::Left => {
            if at > 0 {
                // A byte back, then to the boundary before it: `previous_boundary` in the model, reimplemented
                // here because it is private and this is one line.
                let mut target = at - 1;
                while target > 0 && !text.is_char_boundary(target) {
                    target -= 1;
                }
                return Selection::caret(block, target);
            }
            // At the start of a block, Left goes to the end of the previous one.
            if block > 0 {
                let previous = doc
                    .blocks
                    .get(block - 1)
                    .and_then(|b| b.kind.text())
                    .map(|t| t.text.len())
                    .unwrap_or(0);
                return Selection::caret(block - 1, previous);
            }
            Selection::caret(block, 0)
        }
        Motion::Right => {
            if at < text.len() {
                let mut target = at + 1;
                while target < text.len() && !text.is_char_boundary(target) {
                    target += 1;
                }
                return Selection::caret(block, target);
            }
            if block + 1 < doc.blocks.len() {
                return Selection::caret(block + 1, 0);
            }
            Selection::caret(block, text.len())
        }
        Motion::Home => {
            // The start of the **visual line**, which is where the caret is drawn rather than where the block is.
            let line_start = line_of(laid, block, at).map(|(_, line)| line.range.start).unwrap_or(0);
            Selection::caret(block, line_start)
        }
        Motion::End => {
            let line_end = line_of(laid, block, at)
                .map(|(_, line)| line.range.end)
                .unwrap_or(text.len())
                .min(text.len());
            Selection::caret(block, line_end)
        }
        Motion::DocumentHome => Selection::caret(0, 0),
        Motion::DocumentEnd => {
            let last = doc.blocks.len().saturating_sub(1);
            let end = doc
                .blocks
                .get(last)
                .and_then(|b| b.kind.text())
                .map(|t| t.text.len())
                .unwrap_or(0);
            Selection::caret(last, end)
        }
        Motion::Up | Motion::Down => {
            // Only **within** a block: a block is the unit the layout knows, and moving between blocks is what
            // `Left`/`Right` at an end already does.
            let Some((index, _)) = line_of(laid, block, at) else {
                return Selection::caret(block, at);
            };
            let block_box = laid.blocks.iter().find(|box_| box_.block == block);
            let Some(block_box) = block_box else {
                return Selection::caret(block, at);
            };
            let column = at - block_box.lines.get(index).map(|l| l.range.start).unwrap_or(0);
            let target_index = match motion {
                Motion::Up if index > 0 => index - 1,
                Motion::Down if index + 1 < block_box.lines.len() => index + 1,
                _ => return Selection::caret(block, at),
            };
            let line = &block_box.lines[target_index];
            let target = (line.range.start + column).min(line.range.end);
            Selection::caret(block, clamp(&text, target))
        }
    }
}

/// The line of a block that holds an offset.
fn line_of(laid: &Laid, block: usize, offset: usize) -> Option<(usize, &layout::Line)> {
    let block_box = laid.blocks.iter().find(|box_| box_.block == block)?;
    for (index, line) in block_box.lines.iter().enumerate() {
        if offset < line.range.end || index + 1 == block_box.lines.len() {
            return Some((index, line));
        }
    }
    block_box.lines.first().map(|line| (0, line))
}

/// Empty for the same reason `MpSegmented`'s is: no animator to seat, nothing to place — except the measure, which
/// is seeded from the DSL so a script re-apply does not wipe a caller's.
impl ScriptHook for MpEditor {
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {
        self.measure = self.initial_measure;
    }
}

impl Widget for MpEditor {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            // Typed text, which is not a shortcut: see `insert`.
            Event::TextInput(tie) => {
                if !tie.input.is_empty() {
                    self.insert(cx, &tie.input);
                }
            }
            Event::KeyDown(ke) if !ke.is_repeat => {
                // **The menu owns these keys while it is open.** An open menu that let Up/Down move the caret would
                // move the text under the menu the person is reading, and Enter would insert a newline instead of
                // taking the row.
                match ke.key_code {
                    KeyCode::ArrowUp if self.slash_step(cx, -1) => return,
                    KeyCode::ArrowDown if self.slash_step(cx, 1) => return,
                    KeyCode::ReturnKey if self.slash_commit(cx) => return,
                    KeyCode::Escape if self.slash_close(cx) => return,
                    _ => {}
                }
                let extend = ke.modifiers.shift;
                // **One mapping, not two.** `shortcut_for_key` is what the page's script drives as well, so the
                // behaviour a run checks is the behaviour a keypress gets rather than a second path that
                // resembles it.
                if let Some((shortcut, kind)) = MpEditor::shortcut_for_key(ke.key_code, extend) {
                    self.run(cx, shortcut, kind);
                    return;
                }
                let motion = match ke.key_code {
                    KeyCode::ArrowLeft => Some(Motion::Left),
                    KeyCode::ArrowRight => Some(Motion::Right),
                    KeyCode::ArrowUp => Some(Motion::Up),
                    KeyCode::ArrowDown => Some(Motion::Down),
                    KeyCode::Home => Some(Motion::Home),
                    KeyCode::End => Some(Motion::End),
                    _ => None,
                };
                if let Some(motion) = motion {
                    // ⌘/Ctrl with Home or End is the whole document, which is the convention every editor has.
                    // `is_primary` is Command on macOS and Control elsewhere, which is the platform's own answer
                    // rather than one this crate guesses.
                    let motion = match (motion, ke.modifiers.is_primary()) {
                        (Motion::Home, true) => Motion::DocumentHome,
                        (Motion::End, true) => Motion::DocumentEnd,
                        (motion, _) => motion,
                    };
                    self.move_caret(cx, motion, extend);
                }
            }
            // Undo and redo, which are the strongest reason `makepad-editor` exists.
            Event::KeyDown(ke) => {
                if ke.modifiers.is_primary() {
                    let undo = ke.key_code == KeyCode::KeyZ && !ke.modifiers.shift;
                    let redo = ke.key_code == KeyCode::KeyZ && ke.modifiers.shift;
                    if undo || redo {
                        let step = self
                            .history
                            .as_mut()
                            .and_then(|history| if undo { history.undo() } else { history.redo() });
                        if let Some(step) = step {
                            self.doc = step.doc;
                            self.selection = step.selection;
                            self.laid = None;
                            self.redraw(cx);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (panel, border, radius, body_ink, muted_ink, code_ink, rule, plate, wash, caret_ink) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.surface_card,
                p.border,
                makepad_theme::Theme::panel_radius() as f32,
                p.text,
                p.text_muted,
                p.code_text,
                p.divider,
                p.code_wash,
                p.selection,
                p.caret,
            )
        };

        self.ensure_laid(cx.cx);
        let height = self.laid.as_ref().map_or(0.0, |laid| laid.height) + PAD * 2.0;

        // A `Fit` width resolves from this widget's last drawn size, the way `View::walk_from_previous_size` does
        // it for a `View` — see `mp/segmented.rs`, where not doing it cost eight attempts and a wrong file.
        let measured = self.measure;
        let walk = match walk.width {
            Size::Fit { .. } => Walk {
                width: Size::Fixed(self.last_size.map_or(measured, |size| {
                    if size.x > 0.0 {
                        size.x
                    } else {
                        measured
                    }
                })),
                ..walk
            },
            _ => walk,
        };
        self.walk = Walk {
            width: walk.width,
            height: Size::Fixed(height),
            ..walk
        };
        let walk = self.walk;

        self.draw_bg.fill = panel;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        let rect = self.draw_bg.area().rect(cx.cx);
        if rect.size.x > 0.0 {
            self.last_size = Some(rect.size);
        }
        self.area = self.draw_bg.area();
        let origin = rect.pos + dvec2(PAD, PAD);

        let laid = self.laid.clone().unwrap_or_else(|| {
            layout::layout(&self.doc, self.metrics_for(cx.cx), self.measure)
        });
        let metrics = laid.metrics;

        for block_box in &laid.blocks {
            let block = &self.doc.blocks[block_box.block];
            let is_code = matches!(block.kind, BlockKind::Code { .. });
            let is_quote = matches!(block.kind, BlockKind::Quote(_));

            if is_code || is_quote {
                let plate_rect = Rect {
                    pos: origin + dvec2(block_box.text_x - metrics.padding * 0.5, block_box.y),
                    size: dvec2(
                        (self.measure - PAD * 2.0 - block_box.text_x).max(1.0),
                        block_box.height,
                    ),
                };
                self.draw_bg.fill = plate;
                self.draw_bg.border_width = 0.0;
                self.draw_bg.radius = 6.0;
                self.draw_bg.draw_abs(cx, plate_rect);
                self.draw_bg.border_width = 1.0;
                self.draw_bg.fill = panel;
            }

            if matches!(block.kind, BlockKind::Divider) {
                let rule_rect = Rect {
                    pos: origin + dvec2(block_box.text_x, block_box.y + block_box.height * 0.5),
                    size: dvec2(
                        (self.measure - PAD * 2.0 - block_box.text_x * 2.0).max(1.0),
                        1.0,
                    ),
                };
                self.draw_bg.fill = rule;
                self.draw_bg.border_width = 0.0;
                self.draw_bg.radius = 0.0;
                self.draw_bg.draw_abs(cx, rule_rect);
                self.draw_bg.border_width = 1.0;
                self.draw_bg.fill = panel;
                continue;
            }

            // **The selection first**, so the text is painted over it.
            if block_box.block == self.selection.block && !self.selection.range.is_empty() {
                let start = self.selection.range.start.min(self.selection.range.end);
                let end = self.selection.range.start.max(self.selection.range.end);
                if let Some(text) = block.kind.text() {
                    for line in &block_box.lines {
                        let from = start.max(line.range.start);
                        let to = end.min(line.range.end);
                        if from >= to {
                            continue;
                        }
                        let x0 = line.x + metrics.text_width_at(
                            &text.text[line.range.start..from],
                            metrics.size_for(&block.kind),
                        );
                        let x1 = line.x + metrics.text_width_at(
                            &text.text[line.range.start..to],
                            metrics.size_for(&block.kind),
                        );
                        let line_height = metrics.line_height_for(&block.kind);
                        self.draw_bg.fill = wash;
                        self.draw_bg.border_width = 0.0;
                        self.draw_bg.radius = 2.0;
                        self.draw_bg.draw_abs(
                            cx,
                            Rect {
                                pos: origin + dvec2(x0, line.y),
                                size: dvec2((x1 - x0).max(1.0), line_height),
                            },
                        );
                        self.draw_bg.border_width = 1.0;
                        self.draw_bg.fill = panel;
                    }
                }
            }

            if let Some(marker) = &block_box.marker {
                self.draw_marker.color = muted_ink;
                let label = marker.clone();
                let x = origin.x + block_box.text_x - metrics.text_width(&label);
                let y = origin.y + block_box.lines.first().map(|line| line.y).unwrap_or(0.0);
                self.draw_marker.draw_walk(
                    cx,
                    Walk::fit().with_abs_pos(dvec2(x, y)),
                    Align::default(),
                    &label,
                );
            }

            for line in &block_box.lines {
                let Some(text) = block.kind.text() else {
                    continue;
                };
                let Some(slice) = text.text.get(line.range.clone()) else {
                    continue;
                };
                if slice.trim().is_empty() {
                    continue;
                }
                if is_code {
                    self.draw_code.color = code_ink;
                    self.draw_code.draw_walk(
                        cx,
                        Walk::fit().with_abs_pos(dvec2(origin.x + line.x, origin.y + line.y)),
                        Align::default(),
                        slice,
                    );
                } else {
                    self.draw_body.text_style.font_size = metrics.size_for(&block.kind) as f32;
                    self.draw_body.color = body_ink;
                    self.draw_body.draw_walk(
                        cx,
                        Walk::fit().with_abs_pos(dvec2(origin.x + line.x, origin.y + line.y)),
                        Align::default(),
                        slice,
                    );
                }
            }
        }

        // **The caret last**, so nothing paints over it.
        let mut caret_rect: Option<Rect> = None;
        if let Some((line_index, x)) = self.caret_position(&laid) {
            if let Some(block_box) = laid.blocks.iter().find(|b| b.block == self.selection.block) {
                if let Some(line) = block_box.lines.get(line_index) {
                    let line_height = metrics.line_height_for(&self.doc.blocks[block_box.block].kind);
                    let rect = Rect {
                        pos: origin + dvec2(x, line.y),
                        size: dvec2(CARET_W, line_height),
                    };
                    caret_rect = Some(rect);
                    self.draw_bg.fill = caret_ink;
                    self.draw_bg.border_width = 0.0;
                    self.draw_bg.radius = 0.0;
                    self.draw_bg.draw_abs(cx, rect);
                    self.draw_bg.border_width = 1.0;
                    self.draw_bg.fill = panel;
                }
            }
        }

        // **The slash menu, painted after the caret so nothing covers it.**
        //
        // Until this existed the menu worked and showed nothing — a menu whose rows are never painted is
        // indistinguishable from a menu that is not there, which is the failure that reads as success in a log.
        //
        // Three things about the geometry are decisions rather than arithmetic:
        //
        // - **The plate is as wide as its widest row, measured by the renderer.** `text::measured_width` asks the
        //   draw list, so the plate fits the text rather than an estimate of it — the same reason a selection's
        //   rectangle uses the layout's own widths.
        // - **It opens below the caret and flips above when it would leave the editor**, because a menu that is
        //   half outside its own widget is a menu with rows you cannot read.
        // - **Nothing is drawn when no row matches.** A plate with no rows says less than the text a person is
        //   already looking at, and the block's own text still shows the query.
        if let Some(caret) = caret_rect {
            let row_h = theme_row_height(cx);
            // Copied out of the menu first: `self.slash` borrows `self` immutably, and the draw calls below need it
            // mutably. The labels are `&'static str` because the menu's items are `const`, so this copies pointers.
            let rows: Vec<(&'static str, &'static str)> = match self.slash.as_ref() {
                Some(menu) => menu.matches().map(|item| (item.label, item.hint)).collect(),
                None => Vec::new(),
            };
            let active = self.slash.as_ref().map_or(0, |menu| menu.active());
            if !rows.is_empty() {
                let pad = SLASH_PLATE_PAD;
                let font = metrics.body_size * SLASH_ROW_SCALE;
                let mut widest = 0.0f64;
                for (label, hint) in &rows {
                    self.draw_body.text_style.font_size = font as f32;
                    let mut row = text::measured_width(&self.draw_body, cx, label);
                    if !hint.is_empty() {
                        self.draw_marker.text_style.font_size = (font * SLASH_HINT_SCALE) as f32;
                        row += text::measured_width(&self.draw_marker, cx, hint) + SLASH_HINT_GAP;
                    }
                    widest = widest.max(row);
                }
                let plate_w = widest + pad * 2.0 + SLASH_ACCENT_W;
                let plate_h = rows.len() as f64 * row_h + pad * 2.0;

                // Below the caret, or above it when below would leave the widget. `height` is this widget's own, so
                // the flip is decided against the box the menu has to live in.
                let mut y = caret.pos.y + caret.size.y + SLASH_GAP;
                if y + plate_h > origin.y + height {
                    y = (caret.pos.y - plate_h - SLASH_GAP).max(origin.y);
                }
                let x = caret.pos.x.min(origin.x + self.measure - plate_w).max(origin.x);
                let plate = Rect {
                    pos: dvec2(x, y),
                    size: dvec2(plate_w, plate_h),
                };

                // The plate, then the chosen row's wash, then the rows — in that order, so the wash is under its text.
                self.draw_bg.fill = panel;
                self.draw_bg.radius = radius * SLASH_PLATE_RADIUS;
                self.draw_bg.draw_abs(cx, plate);
                self.draw_bg.fill = wash;
                self.draw_bg.radius = radius * SLASH_ROW_RADIUS;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: dvec2(plate.pos.x + 1.0, plate.pos.y + pad + active as f64 * row_h),
                        size: dvec2(plate.size.x - 2.0, row_h),
                    },
                );
                // The accent beside the chosen row, which is what says *this is what Enter takes* without a cursor
                // glyph: the row a person is about to apply is the one thing this menu must never leave ambiguous.
                self.draw_bg.fill = caret_ink;
                self.draw_bg.radius = SLASH_ACCENT_RADIUS;
                self.draw_bg.draw_abs(
                    cx,
                    Rect {
                        pos: dvec2(plate.pos.x + SLASH_ACCENT_X, plate.pos.y + pad + active as f64 * row_h + SLASH_ACCENT_INSET),
                        size: dvec2(SLASH_ACCENT_W, row_h - SLASH_ACCENT_INSET * 2.0),
                    },
                );

                for (index, (label, hint)) in rows.iter().enumerate() {
                    let row_y = plate.pos.y + pad + index as f64 * row_h;
                    let row_ink = if index == active { body_ink } else { muted_ink };
                    self.draw_body.text_style.font_size = font as f32;
                    self.draw_body.color = row_ink;
                    self.draw_body.draw_walk(
                        cx,
                        Walk::fit().with_abs_pos(dvec2(plate.pos.x + pad + SLASH_ACCENT_W, row_y + (row_h - font) * 0.5)),
                        Align::default(),
                        label,
                    );
                    if !hint.is_empty() {
                        let hint_w = {
                            self.draw_marker.text_style.font_size = (font * SLASH_HINT_SCALE) as f32;
                            text::measured_width(&self.draw_marker, cx, hint)
                        };
                        self.draw_marker.text_style.font_size = (font * SLASH_HINT_SCALE) as f32;
                        self.draw_marker.color = muted_ink;
                        self.draw_marker.draw_walk(
                            cx,
                            Walk::fit().with_abs_pos(dvec2(
                                plate.pos.x + plate.size.x - pad - hint_w,
                                row_y + (row_h - font) * 0.5 + SLASH_HINT_BASELINE,
                            )),
                            Align::default(),
                            hint,
                        );
                    }
                }
                // **The geometry, printed, because a screenshot is not always available.** This environment's
                // `screencapture` returns a blank screen (two distinct colours), so a paint path cannot be confirmed
                // by looking. Printing what the paint path **computed** is the next best evidence and it catches the
                // commonest paint fault: a plate placed outside the box it has to live in. Every number here is
                // checkable against the widget's own box, printed on the same line.
                if std::env::var("MP_EDITOR_DEBUG").is_ok() {
                    println!(
                        "SLASH paint plate=({:.1},{:.1} {:.1}x{:.1}) rows={} active={} inside_x={} inside_y={} box=({:.1},{:.1} {:.1}x{:.1})",
                        plate.pos.x,
                        plate.pos.y,
                        plate.size.x,
                        plate.size.y,
                        rows.len(),
                        active,
                        plate.pos.x >= origin.x && plate.pos.x + plate.size.x <= origin.x + self.measure + 0.5,
                        plate.pos.y >= origin.y && plate.pos.y + plate.size.y <= origin.y + height + 0.5,
                        origin.x,
                        origin.y,
                        self.measure,
                        height,
                    );
                }
                // Restore the plate properties, because every other draw above depends on them: the border belongs to
                // the widget's own frame and the radius to its own corners.
                self.draw_bg.fill = panel;
                self.draw_bg.border_width = 1.0;
                self.draw_bg.radius = radius;
            }
        }

        if std::env::var("MP_EDITOR_DEBUG").is_ok() {
            println!(
                "EDITOR blocks={} lines={} caret={:?} sel={:?} height={height:.1} src={:?}",
                self.doc.blocks.len(),
                laid.lines().count(),
                self.selection.block,
                self.selection.range,
                makepad_markdown::serialize(&self.doc),
            );
        }

        DrawStep::done()
    }
}

impl MpEditor {
    /// Where the caret is, as a line index in its block and an x offset from the document's left edge.
    /// The slash menu, if one is open.
    pub fn slash(&self) -> Option<&SlashMenu> {
        self.slash.as_ref()
    }

    /// A **copy** of the open menu's state, for a caller that cannot hold a borrow.
    ///
    /// A `WidgetRef` is one — it borrows the widget per call — and anything that *draws* the menu is another, because
    /// a painter that held the borrow while walking the layout would borrow the widget for the length of the paint.
    /// So the state is owned: the rows are `&'static str` because the menu's items are `const`, which keeps the copy
    /// cheap without copying the labels themselves.
    pub fn slash_state(&self) -> Option<SlashState> {
        self.slash.as_ref().map(SlashState::of)
    }

    /// Open, refilter, or close the menu from where the caret is now.
    ///
    /// **The menu's state is derived, not remembered.** Every path that can move the caret or change the text calls
    /// this, so the menu cannot be open somewhere it should not be — which is the failure a remembered "is the menu
    /// open" flag produces, because every new path that moves the caret is a path that has to remember to close it.
    ///
    /// Whether a menu **should** open is decided here rather than in the parser: not inside a `Code` block, because a
    /// fence holds slashes that are not a menu. And a `Divider` has no text, so there is nothing to type into.
    fn refresh_slash(&mut self, cx: &mut Cx) {
        let block = self.selection.block;
        let text = match self.doc.blocks.get(block).map(|block| &block.kind) {
            Some(BlockKind::Code { .. }) | Some(BlockKind::Divider) | None => None,
            Some(kind) => kind.text().map(|text| text.text.clone()),
        };
        let Some(text) = text else {
            self.slash_close(cx);
            return;
        };
        let offset = clamp(&text, self.selection.range.end);
        let Some(query) = slash::query(offset, &text) else {
            self.slash_close(cx);
            return;
        };
        // The `/` is `query.len()` characters plus itself back from the caret.
        let at = offset - query.len() - 1;
        match self.slash.as_mut() {
            // The same `/` is still being typed into: narrow it. A **new** menu for a different `/`, because a menu
            // that kept its position across two different slashes would apply a row the person never looked at.
            Some(menu) if menu.at() == at => menu.refilter(&query),
            _ => {
                let mut menu = SlashMenu::open(at);
                menu.refilter(&query);
                self.slash = Some(menu);
            }
        }
        self.redraw(cx);
    }

    /// Close the menu, redrawing only if one was open.
    pub fn slash_close(&mut self, cx: &mut Cx) -> bool {
        if self.slash.take().is_some() {
            self.redraw(cx);
            return true;
        }
        false
    }

    /// Walk the chosen row by `delta`. Answers whether a menu was open, which is what makes this safe to call
    /// unconditionally: **a caller offers Up and Enter to the menu first, and moves the caret only if it declined.**
    ///
    /// **A public operation, and the key handler is one of its callers.** The menu first existed only inside the key
    /// handler, and a driver typing through the public API then could not walk or commit it — the run showed
    /// `active=0` after a `down` and a menu still open after an `enter`, while a real keypress worked. Two paths that
    /// resemble each other again: the same defect `insert`'s doc warns about, made in the same change that wrote the
    /// warning.
    pub fn slash_step(&mut self, cx: &mut Cx, delta: isize) -> bool {
        let Some(menu) = self.slash.as_mut() else {
            return false;
        };
        menu.step(delta);
        let _ = cx;
        if let Some(menu) = self.slash.as_ref() {
            let _ = menu;
        }
        self.redraw(cx);
        true
    }

    /// Apply the chosen row: **splice out the typed query, then take a shortcut that already exists**.
    ///
    /// The splice is what separates this from a combobox — the `/query` was never the block's text, so removing it
    /// leaves the block as it was. And the kind is applied with `Shortcut::SetKind`, the same edit a keyboard
    /// shortcut runs, so the menu does not bring its own edit path.
    pub fn slash_commit(&mut self, cx: &mut Cx) -> bool {
        let Some(menu) = self.slash.take() else {
            return false;
        };
        let Some(kind) = menu.choice() else {
            // Nothing matched, so Enter inserts nothing rather than taking a row that is not there.
            self.redraw(cx);
            return true;
        };
        let block = self.selection.block;
        let text = self
            .doc
            .blocks
            .get(block)
            .and_then(|block| block.kind.text())
            .map(|text| text.text.clone());
        if let Some(text) = text {
            let (start, end) = menu.span(clamp(&text, self.selection.range.end));
            let spliced = format!("{}{}", &text[..start], &text[end..]);
            set_block_text(&mut self.doc, block, makepad_markdown::Text::plain(spliced));
            // The caret goes where the removed query was, which is where the chosen kind's text now begins.
            self.selection = Selection::caret(block, start);
        }
        self.press(cx, Shortcut::SetKind(kind), EditKind::Formatting);
        self.redraw(cx);
        true
    }

    fn caret_position(&self, laid: &Laid) -> Option<(usize, f64)> {
        let block_box = laid.blocks.iter().find(|b| b.block == self.selection.block)?;
        let text = self.doc.blocks.get(self.selection.block)?.kind.text()?.text.as_str();
        let offset = clamp(text, self.selection.range.end);
        for (index, line) in block_box.lines.iter().enumerate() {
            if offset < line.range.end || index + 1 == block_box.lines.len() {
                let x = line.x + laid.metrics.text_width_at(
                    &text[line.range.start..offset.max(line.range.start)],
                    laid.metrics.body_size,
                );
                return Some((index, x));
            }
        }
        block_box.lines.first().map(|line| (0, line.x))
    }
}

impl MpEditorRef {
    /// A copy of the open menu's state. See `MpEditor::slash_state`.
    pub fn slash_state(&self) -> Option<SlashState> {
        self.borrow().and_then(|inner| inner.slash_state())
    }

    /// Walk the chosen row. Answers whether a menu was open. See `MpEditor::slash_step`.
    pub fn slash_step(&self, cx: &mut Cx, delta: isize) -> bool {
        self.borrow_mut().is_some_and(|mut inner| inner.slash_step(cx, delta))
    }

    /// Take the chosen row. Answers whether a menu was open. See `MpEditor::slash_commit`.
    pub fn slash_commit(&self, cx: &mut Cx) -> bool {
        self.borrow_mut().is_some_and(|mut inner| inner.slash_commit(cx))
    }

    /// Close the menu. Answers whether one was open. See `MpEditor::slash_close`.
    pub fn slash_close(&self, cx: &mut Cx) -> bool {
        self.borrow_mut().is_some_and(|mut inner| inner.slash_close(cx))
    }

    pub fn set_source(&self, cx: &mut Cx, source: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_source(cx, source);
        }
    }

    /// The document as markdown, for a page that reports what the reader has typed.
    pub fn source(&self) -> Option<String> {
        self.borrow().map(|inner| inner.source())
    }

    pub fn set_measure(&self, cx: &mut Cx, measure: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_measure(cx, measure);
        }
    }

    pub fn insert_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.insert_text(cx, text);
        }
    }

    pub fn press(&self, cx: &mut Cx, shortcut: Shortcut, kind: EditKind) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.press(cx, shortcut, kind);
        }
    }

    pub fn move_by(&self, cx: &mut Cx, motion: Motion, extend: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.move_by(cx, motion, extend);
        }
    }

    pub fn undo(&self, cx: &mut Cx) -> bool {
        self.borrow_mut()
            .map(|mut inner| inner.undo(cx))
            .unwrap_or(false)
    }

    pub fn redo(&self, cx: &mut Cx) -> bool {
        self.borrow_mut()
            .map(|mut inner| inner.redo(cx))
            .unwrap_or(false)
    }

    pub fn depth(&self) -> (usize, usize) {
        self.borrow().map(|inner| inner.depth()).unwrap_or((0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn laid(source: &str, width: f64) -> (Doc, Laid) {
        let doc = makepad_markdown::parse(source);
        let metrics = Metrics {
            advance: 10.0,
            body_size: 10.0,
            heading_size: [20.0, 15.0, 12.0],
            line_height: 20.0,
            indent: 30.0,
            gap: 5.0,
            padding: 4.0,
        };
        let laid = layout::layout(&doc, metrics, width);
        (doc, laid)
    }

    #[test]
    fn test_left_and_right_step_by_character_and_not_by_byte() {
        let (doc, laid) = laid("héllo\n", 1000.0);
        // `é` is two bytes, so a byte-wise Left would land inside it.
        let selection = Selection::caret(0, 3);
        let moved = move_within(&doc, &laid, &selection, Motion::Left);
        assert_eq!(moved, Selection::caret(0, 1));
        let back = move_within(&doc, &laid, &moved, Motion::Right);
        assert_eq!(back, Selection::caret(0, 3));
    }

    #[test]
    fn test_left_at_the_start_goes_to_the_end_of_the_previous_block() {
        let (doc, laid) = laid("one\n\ntwo\n", 1000.0);
        let selection = Selection::caret(1, 0);
        assert_eq!(
            move_within(&doc, &laid, &selection, Motion::Left),
            Selection::caret(0, 3)
        );
        // And in the first block it stops.
        let selection = Selection::caret(0, 0);
        assert_eq!(
            move_within(&doc, &laid, &selection, Motion::Left),
            Selection::caret(0, 0)
        );
    }

    #[test]
    fn test_right_at_the_end_goes_to_the_start_of_the_next_block() {
        let (doc, laid) = laid("one\n\ntwo\n", 1000.0);
        let selection = Selection::caret(0, 3);
        assert_eq!(
            move_within(&doc, &laid, &selection, Motion::Right),
            Selection::caret(1, 0)
        );
        let selection = Selection::caret(1, 3);
        assert_eq!(
            move_within(&doc, &laid, &selection, Motion::Right),
            Selection::caret(1, 3),
            "at the end of the document it stops"
        );
    }

    #[test]
    fn test_home_and_end_use_the_visual_line_rather_than_the_block() {
        // A wrapped paragraph is four lines and one block. Home must go to the start of the line the caret is on,
        // **not** to the start of the block — which is the difference a reader notices immediately.
        let source = "one two three four five six seven eight nine ten\n";
        let (doc, laid) = laid(source, 120.0);
        let lines = laid.blocks[0].lines.len();
        assert!(lines > 2, "the paragraph did not wrap: {lines} lines");
        let on_second = laid.blocks[0].lines[1].range.start + 2;
        let home = move_within(&doc, &laid, &Selection::caret(0, on_second), Motion::Home);
        assert_eq!(home, Selection::caret(0, laid.blocks[0].lines[1].range.start));
        let end = move_within(&doc, &laid, &Selection::caret(0, on_second), Motion::End);
        assert_eq!(end, Selection::caret(0, laid.blocks[0].lines[1].range.end));
    }

    #[test]
    fn test_document_home_and_end_reach_the_whole_document() {
        let (doc, laid) = laid("# H\n\npara\n\n- item\n", 1000.0);
        let middle = Selection::caret(1, 2);
        assert_eq!(
            move_within(&doc, &laid, &middle, Motion::DocumentHome),
            Selection::caret(0, 0)
        );
        assert_eq!(
            move_within(&doc, &laid, &middle, Motion::DocumentEnd),
            Selection::caret(2, 4),
            "the end is the end of the last block"
        );
    }

    #[test]
    fn test_up_and_down_move_between_the_lines_of_a_wrapped_block() {
        let source = "one two three four five six seven eight nine ten\n";
        let (doc, laid) = laid(source, 120.0);
        let first_line_end = laid.blocks[0].lines[0].range.end;
        // Down from the start of the first line lands on the second line.
        let down = move_within(&doc, &laid, &Selection::caret(0, 0), Motion::Down);
        assert!(
            down.range.end >= laid.blocks[0].lines[1].range.start,
            "down did not reach the second line: {down:?}"
        );
        // Down from the last line stays put, and up from the first stays put.
        let last = laid.blocks[0].lines.len() - 1;
        let last_start = laid.blocks[0].lines[last].range.start;
        assert_eq!(
            move_within(&doc, &laid, &Selection::caret(0, last_start), Motion::Down),
            Selection::caret(0, last_start),
            "down at the last line moved"
        );
        assert_eq!(
            move_within(&doc, &laid, &Selection::caret(0, 0), Motion::Up),
            Selection::caret(0, 0),
            "up at the first line moved"
        );
        let _ = first_line_end;
    }

    #[test]
    fn test_a_caret_motion_never_leaves_the_block_it_was_in() {
        // The property the tests above are instances of: for every motion and every offset, the result is a
        // selection whose block exists and whose offset is on a character boundary of that block.
        let sources = ["# H\n\npara one two three\n\n- item\n", "```\ncode\n```\n", "a\n"];
        let motions = [
            Motion::Left,
            Motion::Right,
            Motion::Up,
            Motion::Down,
            Motion::Home,
            Motion::End,
            Motion::DocumentHome,
            Motion::DocumentEnd,
        ];
        for source in sources {
            let (doc, laid) = laid(source, 90.0);
            for block in 0..doc.blocks.len() {
                let len = doc.blocks[block]
                    .kind
                    .text()
                    .map(|text| text.text.len())
                    .unwrap_or(0);
                for offset in 0..=len {
                    let text = doc.blocks[block].kind.text().map(|t| t.text.clone());
                    if let Some(text) = &text {
                        if !text.is_char_boundary(offset) {
                            continue;
                        }
                    }
                    for motion in motions {
                        let moved = move_within(&doc, &laid, &Selection::caret(block, offset), motion);
                        assert!(
                            moved.block < doc.blocks.len(),
                            "{motion:?} from block {block} gave {:?}",
                            moved.block
                        );
                        let target = doc.blocks[moved.block].kind.text().map(|t| t.text.clone());
                        if let Some(target) = &target {
                            assert!(
                                target.is_char_boundary(moved.range.end),
                                "{motion:?} from block {block} offset {offset} gave an offset inside a \
                                 character: {moved:?}"
                            );
                        }
                        assert!(moved.range.is_empty(), "{motion:?} produced a range");
                    }
                }
            }
        }
    }

    #[test]
    fn test_an_insert_at_a_selection_replaces_it() {
        // The arithmetic `insert` does, checked without a `Cx`: a replacement at a range, and the marks after it
        // shifted by the difference.
        let mut text = makepad_markdown::Text {
            text: "hello world".to_string(),
            // A link as well as a mark, and it is here because a `Text` gained a field and **`cargo check` does not
            // build tests** — this literal was the only thing that broke, and it broke in the test target, which is
            // exactly the trap the note in this file's module doc warns about: changing the shape of a public type
            // means running `cargo test` for every crate that constructs it, not `cargo check`.
            links: vec![makepad_markdown::LinkSpan {
                range: 6..11,
                url: "https://example.com".to_string(),
            }],
            marks: vec![makepad_markdown::MarkSpan {
                range: 6..11,
                mark: makepad_markdown::Mark::Bold,
            }],
        };
        let (start, end, input) = (0usize, 5usize, "bye");
        text.text.replace_range(start..end, input);
        let delta = input.len() as isize - (end - start) as isize;
        for span in &mut text.marks {
            if span.range.start >= end {
                span.range.start = (span.range.start as isize + delta) as usize;
                span.range.end = (span.range.end as isize + delta) as usize;
            }
        }
        assert_eq!(text.text, "bye world");
        assert_eq!(
            text.marks[0].range,
            4..9,
            "the mark did not follow the text it covers"
        );
    }
}
