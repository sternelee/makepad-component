use makepad_widgets::*;

use crate::terminal::session::TerminalSession;

/// What a canvas item can be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Note,
    MusicPlayer,
    Terminal,
    Browser,
    /// A dropped media file (image / video / PDF); the payload kind is
    /// re-derived from the stored path when needed.
    Media,
}

/// What kind of media a dropped file is (drives the preview renderer).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaKind {
    Image,
    Video,
    Pdf,
}

impl MediaKind {
    /// Classify a file path by extension (case-insensitive). Unknown
    /// extensions return None so the drop can be rejected with a hint.
    pub fn from_path(path: &str) -> Option<MediaKind> {
        let ext = path
            .rsplit('.')
            .next()
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            // Raster formats the makepad Image widget decodes, plus SVG.
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "qoi" | "svg" => {
                Some(MediaKind::Image)
            }
            // Platform playback backends (AVFoundation / Media Foundation /
            // GStreamer) cover these containers; unsupported codecs surface
            // through the Video widget's own error state.
            "mp4" | "m4v" | "mov" | "webm" | "mkv" | "avi" | "ogv" => Some(MediaKind::Video),
            "pdf" => Some(MediaKind::Pdf),
            _ => None,
        }
    }

    /// Short label for status messages.
    pub fn label(self) -> &'static str {
        match self {
            MediaKind::Image => "Image",
            MediaKind::Video => "Video",
            MediaKind::Pdf => "PDF",
        }
    }

    /// Card accent color (avatar chip + selection border).
    pub fn accent(self) -> [f32; 4] {
        match self {
            MediaKind::Image => [0.38, 0.72, 0.98, 1.0],
            MediaKind::Video => [0.70, 0.48, 0.96, 1.0],
            MediaKind::Pdf => [0.96, 0.45, 0.42, 1.0],
        }
    }

    /// Single-letter avatar shown in the card title bar.
    pub fn avatar(self) -> &'static str {
        match self {
            MediaKind::Image => "I",
            MediaKind::Video => "V",
            MediaKind::Pdf => "P",
        }
    }

    /// Default card size in world units, chosen per kind (portrait for PDF
    /// pages, 4:3-ish for images, 16:10-ish for video).
    pub fn default_size(self) -> (f64, f64) {
        match self {
            MediaKind::Image => (360.0, 280.0),
            MediaKind::Video => (460.0, 300.0),
            MediaKind::Pdf => (420.0, 540.0),
        }
    }
}

/// Filename part of a path, used as the default card title.
pub fn path_file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Agent status shown on terminal/agent cards (CNVS-style presence indicator).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AgentStatus {
    #[default]
    Online,
    Busy,
    Idle,
}

impl AgentStatus {
    pub fn label(self) -> &'static str {
        match self {
            AgentStatus::Online => "online",
            AgentStatus::Busy => "busy",
            AgentStatus::Idle => "idle",
        }
    }

    pub fn color(self) -> [f32; 4] {
        match self {
            AgentStatus::Online => [0.38, 0.85, 0.50, 1.0],
            AgentStatus::Busy => [0.95, 0.75, 0.28, 1.0],
            AgentStatus::Idle => [0.55, 0.60, 0.72, 1.0],
        }
    }
}

/// Active drawing tool for a note whiteboard (cnvs-style tool palette).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NoteTool {
    #[default]
    Move,
    Arrow,
    Pen,
    Line,
    Rect,
    Circle,
    Ellipse,
    Polyline,
    Text,
    Eraser,
}

impl NoteTool {
    /// Human-readable label (used in the status bar when the tool changes).
    pub fn label(self) -> &'static str {
        match self {
            NoteTool::Move => "Select/Move",
            NoteTool::Arrow => "Arrow",
            NoteTool::Pen => "Pen",
            NoteTool::Line => "Line",
            NoteTool::Rect => "Rectangle",
            NoteTool::Circle => "Circle",
            NoteTool::Ellipse => "Ellipse",
            NoteTool::Polyline => "Polyline",
            NoteTool::Text => "Text",
            NoteTool::Eraser => "Eraser",
        }
    }
}

/// A completed (or in-progress) drawing on a note whiteboard.
#[derive(Clone, Debug, PartialEq)]
pub enum NoteShape {
    /// Arrow from `a` to `b` (world coords, note-local).
    Arrow {
        a: makepad_widgets::Vec2d,
        b: makepad_widgets::Vec2d,
    },
    /// Freehand stroke.
    Pen { points: Vec<makepad_widgets::Vec2d> },
    /// Axis-aligned rectangle from corner `a` to `b`.
    Rect {
        a: makepad_widgets::Vec2d,
        b: makepad_widgets::Vec2d,
    },
    /// Circle with center `center` and radius `r` (world units).
    Circle {
        center: makepad_widgets::Vec2d,
        r: f64,
    },
    /// Axis-aligned ellipse inscribed in the bounding box from corner `a`
    /// to corner `b` (world coords).
    Ellipse {
        a: makepad_widgets::Vec2d,
        b: makepad_widgets::Vec2d,
    },
    /// Straight segment `a` → `b`.
    Line {
        a: makepad_widgets::Vec2d,
        b: makepad_widgets::Vec2d,
    },
    /// Multi-segment polyline.
    Polyline { points: Vec<makepad_widgets::Vec2d> },
    /// Text note at `pos`.
    Text {
        pos: makepad_widgets::Vec2d,
        text: String,
    },
}

impl NoteShape {
    /// Distance from `p` (note-local world coords) to the shape's stroke.
    /// Returns None when `p` is not near the stroke (for eraser hits).
    pub fn hit(&self, p: makepad_widgets::Vec2d, tol: f64) -> bool {
        let near = |a: makepad_widgets::Vec2d, b: makepad_widgets::Vec2d| -> f64 {
            // distance point-to-segment
            let abx = b.x - a.x;
            let aby = b.y - a.y;
            let len2 = abx * abx + aby * aby;
            if len2 < 1e-9 {
                return ((p.x - a.x).powi(2) + (p.y - a.y).powi(2)).sqrt();
            }
            let t = (((p.x - a.x) * abx + (p.y - a.y) * aby) / len2).clamp(0.0, 1.0);
            let px = a.x + t * abx;
            let py = a.y + t * aby;
            ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt()
        };
        match self {
            NoteShape::Arrow { a, b } => near(*a, *b) <= tol,
            NoteShape::Line { a, b } => near(*a, *b) <= tol,
            NoteShape::Pen { points } | NoteShape::Polyline { points } => {
                points.windows(2).any(|w| near(w[0], w[1]) <= tol)
            }
            NoteShape::Rect { a, b } => {
                let x0 = a.x.min(b.x);
                let y0 = a.y.min(b.y);
                let x1 = a.x.max(b.x);
                let y1 = a.y.max(b.y);
                let segs = [
                    (
                        makepad_widgets::Vec2d { x: x0, y: y0 },
                        makepad_widgets::Vec2d { x: x1, y: y0 },
                    ),
                    (
                        makepad_widgets::Vec2d { x: x1, y: y0 },
                        makepad_widgets::Vec2d { x: x1, y: y1 },
                    ),
                    (
                        makepad_widgets::Vec2d { x: x1, y: y1 },
                        makepad_widgets::Vec2d { x: x0, y: y1 },
                    ),
                    (
                        makepad_widgets::Vec2d { x: x0, y: y1 },
                        makepad_widgets::Vec2d { x: x0, y: y0 },
                    ),
                ];
                segs.iter().any(|(a, b)| near(*a, *b) <= tol)
            }
            NoteShape::Circle { center, r } => {
                let d = ((p.x - center.x).powi(2) + (p.y - center.y).powi(2)).sqrt();
                (d - r).abs() <= tol
            }
            NoteShape::Ellipse { a, b } => {
                // Ellipse inscribed in the bounding box [a, b]. Hit test by
                // sampling the perimeter (same approach as the draw code).
                let cx = (a.x + b.x) * 0.5;
                let cy = (a.y + b.y) * 0.5;
                let rx = ((b.x - a.x).abs() * 0.5).max(0.5);
                let ry = ((b.y - a.y).abs() * 0.5).max(0.5);
                let n = 48;
                let mut prev = makepad_widgets::Vec2d { x: cx + rx, y: cy };
                for i in 1..=n {
                    let ang = (i as f64 / n as f64) * std::f64::consts::TAU;
                    let cur = makepad_widgets::Vec2d {
                        x: cx + rx * ang.cos(),
                        y: cy + ry * ang.sin(),
                    };
                    if near(prev, cur) <= tol {
                        return true;
                    }
                    prev = cur;
                }
                false
            }
            NoteShape::Text { pos, .. } => {
                let w = 90.0;
                let h = 18.0;
                p.x >= pos.x && p.x <= pos.x + w && p.y >= pos.y && p.y <= pos.y + h
            }
        }
    }

    /// Translate this shape in-place by `delta` (world units). Used by the
    /// Move tool to drag existing shapes.
    pub fn translate(&mut self, delta: makepad_widgets::Vec2d) {
        let d = |p: &mut makepad_widgets::Vec2d| {
            p.x += delta.x;
            p.y += delta.y;
        };
        match self {
            NoteShape::Arrow { a, b }
            | NoteShape::Line { a, b }
            | NoteShape::Rect { a, b }
            | NoteShape::Ellipse { a, b } => {
                d(a);
                d(b);
            }
            NoteShape::Circle { center, .. } => d(center),
            NoteShape::Pen { points } | NoteShape::Polyline { points } => {
                for p in points.iter_mut() {
                    d(p);
                }
            }
            NoteShape::Text { pos, .. } => d(pos),
        }
    }
}

/// A drawn whiteboard shape together with the ink color and stroke width
/// it was drawn with. Storing the style per shape lets the color/width
/// picker apply to each stroke independently.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawnShape {
    pub shape: NoteShape,
    /// RGBA ink color.
    pub color: [f32; 4],
    /// Stroke width in world units.
    pub width: f64,
    /// Random seed driving the hand-drawn wobble. Fixed at creation so the
    /// sketchy stroke keeps its exact shape across frames (no flicker).
    pub seed: u32,
}

/// A single canvas item (note, terminal, or browser).
pub enum CanvasItem {
    Note {
        id: u64,
        world: Rect,
        title: String,
        /// Editable body text rendered inside the note card.
        body: String,
        /// Font size for the note body.
        font_size: f32,
        /// Index into a color palette for the note body text.
        color_idx: usize,
    },
    MusicPlayer {
        id: u64,
        world: Rect,
        title: String,
        /// Current play progress 0..1.
        progress: f32,
        /// Whether the player is "playing" (animates visualizer).
        playing: bool,
    },
    Terminal {
        id: u64,
        world: Rect,
        title: String,
        /// Presence/status indicator for agent-style terminal cards.
        status: AgentStatus,
        session: Option<Box<TerminalSession>>,
    },
    Browser {
        id: u64,
        world: Rect,
        title: String,
        url: String,
    },
    /// A dropped media file previewing on the canvas.
    Media {
        id: u64,
        world: Rect,
        /// File name (card title).
        title: String,
        /// Absolute filesystem path of the source file.
        path: String,
        kind: MediaKind,
    },
}

impl CanvasItem {
    pub fn id(&self) -> u64 {
        match self {
            CanvasItem::Note { id, .. }
            | CanvasItem::MusicPlayer { id, .. }
            | CanvasItem::Terminal { id, .. }
            | CanvasItem::Browser { id, .. }
            | CanvasItem::Media { id, .. } => *id,
        }
    }

    pub fn kind(&self) -> ItemKind {
        match self {
            CanvasItem::Note { .. } => ItemKind::Note,
            CanvasItem::MusicPlayer { .. } => ItemKind::MusicPlayer,
            CanvasItem::Terminal { .. } => ItemKind::Terminal,
            CanvasItem::Browser { .. } => ItemKind::Browser,
            CanvasItem::Media { .. } => ItemKind::Media,
        }
    }

    pub fn world(&self) -> Rect {
        match self {
            CanvasItem::Note { world, .. }
            | CanvasItem::MusicPlayer { world, .. }
            | CanvasItem::Terminal { world, .. }
            | CanvasItem::Browser { world, .. }
            | CanvasItem::Media { world, .. } => *world,
        }
    }

    pub fn world_mut(&mut self) -> &mut Rect {
        match self {
            CanvasItem::Note { world, .. }
            | CanvasItem::MusicPlayer { world, .. }
            | CanvasItem::Terminal { world, .. }
            | CanvasItem::Browser { world, .. }
            | CanvasItem::Media { world, .. } => world,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            CanvasItem::Note { title, .. }
            | CanvasItem::MusicPlayer { title, .. }
            | CanvasItem::Terminal { title, .. }
            | CanvasItem::Browser { title, .. }
            | CanvasItem::Media { title, .. } => title,
        }
    }

    #[allow(dead_code)]
    pub fn title_mut(&mut self) -> &mut String {
        match self {
            CanvasItem::Note { title, .. }
            | CanvasItem::MusicPlayer { title, .. }
            | CanvasItem::Terminal { title, .. }
            | CanvasItem::Browser { title, .. }
            | CanvasItem::Media { title, .. } => title,
        }
    }

    pub fn session(&self) -> Option<&TerminalSession> {
        match self {
            CanvasItem::Terminal { session, .. } => session.as_deref(),
            CanvasItem::Note { .. }
            | CanvasItem::MusicPlayer { .. }
            | CanvasItem::Browser { .. }
            | CanvasItem::Media { .. } => None,
        }
    }

    #[allow(dead_code)]
    pub fn session_mut(&mut self) -> Option<&mut TerminalSession> {
        match self {
            CanvasItem::Terminal { session, .. } => session.as_deref_mut(),
            CanvasItem::Note { .. }
            | CanvasItem::MusicPlayer { .. }
            | CanvasItem::Browser { .. }
            | CanvasItem::Media { .. } => None,
        }
    }

    pub fn url(&self) -> Option<&str> {
        match self {
            CanvasItem::Browser { url, .. } => Some(url),
            _ => None,
        }
    }

    /// Media payload kind of a Media item.
    pub fn media_kind(&self) -> Option<MediaKind> {
        match self {
            CanvasItem::Media { kind, .. } => Some(*kind),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn url_mut(&mut self) -> Option<&mut String> {
        match self {
            CanvasItem::Browser { url, .. } => Some(url),
            _ => None,
        }
    }

    pub fn body(&self) -> Option<&str> {
        match self {
            CanvasItem::Note { body, .. } => Some(body),
            _ => None,
        }
    }

    pub fn body_mut(&mut self) -> Option<&mut String> {
        match self {
            CanvasItem::Note { body, .. } => Some(body),
            _ => None,
        }
    }

    pub fn note_font_size(&self) -> Option<f32> {
        match self {
            CanvasItem::Note { font_size, .. } => Some(*font_size),
            _ => None,
        }
    }

    pub fn note_color_idx(&self) -> Option<usize> {
        match self {
            CanvasItem::Note { color_idx, .. } => Some(*color_idx),
            _ => None,
        }
    }

    pub fn agent_status(&self) -> Option<AgentStatus> {
        match self {
            CanvasItem::Terminal { status, .. } => Some(*status),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn agent_status_mut(&mut self) -> Option<&mut AgentStatus> {
        match self {
            CanvasItem::Terminal { status, .. } => Some(status),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_kind_classifies_images() {
        assert_eq!(
            MediaKind::from_path("/tmp/photo.PNG"),
            Some(MediaKind::Image)
        );
        assert_eq!(MediaKind::from_path("cat.jpeg"), Some(MediaKind::Image));
        assert_eq!(
            MediaKind::from_path("/a/b/anim.gif"),
            Some(MediaKind::Image)
        );
        assert_eq!(MediaKind::from_path("logo.svg"), Some(MediaKind::Image));
    }

    #[test]
    fn media_kind_classifies_videos() {
        assert_eq!(
            MediaKind::from_path("/Users/me/clip.mp4"),
            Some(MediaKind::Video)
        );
        assert_eq!(
            MediaKind::from_path("screen recording.MOV"),
            Some(MediaKind::Video)
        );
        assert_eq!(MediaKind::from_path("clip.webm"), Some(MediaKind::Video));
    }

    #[test]
    fn media_kind_classifies_pdf() {
        assert_eq!(
            MediaKind::from_path("/docs/report.pdf"),
            Some(MediaKind::Pdf)
        );
    }

    #[test]
    fn media_kind_rejects_unknown_extensions() {
        assert_eq!(MediaKind::from_path("/tmp/notes.txt"), None);
        assert_eq!(MediaKind::from_path("archive.tar.gz"), None);
        assert_eq!(MediaKind::from_path("no_extension"), None);
    }

    #[test]
    fn media_kind_defaults_and_labels_cover_all_variants() {
        for kind in [MediaKind::Image, MediaKind::Video, MediaKind::Pdf] {
            assert!(!kind.label().is_empty());
            assert!(!kind.avatar().is_empty());
            let (w, h) = kind.default_size();
            assert!(w > 0.0 && h > 0.0);
        }
    }

    #[test]
    fn path_file_name_extracts_last_component() {
        assert_eq!(path_file_name("/a/b/c.png"), "c.png");
        assert_eq!(path_file_name("plain.txt"), "plain.txt");
        assert_eq!(path_file_name("/"), "");
    }
}
