use makepad_widgets::*;

use crate::terminal::session::TerminalSession;

/// What a canvas item can be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Note,
    Terminal,
    Browser,
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
    /// Short label / glyph for the vertical tool palette.
    pub fn label(self) -> &'static str {
        match self {
            NoteTool::Move => "✥",
            NoteTool::Arrow => "↗",
            NoteTool::Pen => "✏",
            NoteTool::Line => "╱",
            NoteTool::Rect => "▭",
            NoteTool::Circle => "○",
            NoteTool::Ellipse => "◯",
            NoteTool::Polyline => "⌁",
            NoteTool::Text => "T",
            NoteTool::Eraser => "▤",
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
                let mut prev = makepad_widgets::Vec2d {
                    x: cx + rx,
                    y: cy,
                };
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
}

/// A single canvas item (note, terminal, or browser).
pub enum CanvasItem {
    Note {
        id: u64,
        world: Rect,
        title: String,
    },
    Terminal {
        id: u64,
        world: Rect,
        title: String,
        session: Option<Box<TerminalSession>>,
    },
    Browser {
        id: u64,
        world: Rect,
        title: String,
        url: String,
    },
}

impl CanvasItem {
    pub fn id(&self) -> u64 {
        match self {
            CanvasItem::Note { id, .. }
            | CanvasItem::Terminal { id, .. }
            | CanvasItem::Browser { id, .. } => *id,
        }
    }

    pub fn kind(&self) -> ItemKind {
        match self {
            CanvasItem::Note { .. } => ItemKind::Note,
            CanvasItem::Terminal { .. } => ItemKind::Terminal,
            CanvasItem::Browser { .. } => ItemKind::Browser,
        }
    }

    pub fn world(&self) -> Rect {
        match self {
            CanvasItem::Note { world, .. }
            | CanvasItem::Terminal { world, .. }
            | CanvasItem::Browser { world, .. } => *world,
        }
    }

    pub fn world_mut(&mut self) -> &mut Rect {
        match self {
            CanvasItem::Note { world, .. }
            | CanvasItem::Terminal { world, .. }
            | CanvasItem::Browser { world, .. } => world,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            CanvasItem::Note { title, .. }
            | CanvasItem::Terminal { title, .. }
            | CanvasItem::Browser { title, .. } => title,
        }
    }

    #[allow(dead_code)]
    pub fn title_mut(&mut self) -> &mut String {
        match self {
            CanvasItem::Note { title, .. }
            | CanvasItem::Terminal { title, .. }
            | CanvasItem::Browser { title, .. } => title,
        }
    }

    pub fn session(&self) -> Option<&TerminalSession> {
        match self {
            CanvasItem::Terminal { session, .. } => session.as_deref(),
            CanvasItem::Note { .. } | CanvasItem::Browser { .. } => None,
        }
    }

    #[allow(dead_code)]
    pub fn session_mut(&mut self) -> Option<&mut TerminalSession> {
        match self {
            CanvasItem::Terminal { session, .. } => session.as_deref_mut(),
            CanvasItem::Note { .. } | CanvasItem::Browser { .. } => None,
        }
    }

    pub fn url(&self) -> Option<&str> {
        match self {
            CanvasItem::Browser { url, .. } => Some(url),
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
}
