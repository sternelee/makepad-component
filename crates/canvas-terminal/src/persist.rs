//! Canvas persistence: what the GUI remembers across restarts.
//!
//! Two rules shape this module:
//!
//! * **The daemon owns the transcript.** An agent card is saved as its
//!   identity (name + cursor) and geometry, not as a copy of its
//!   conversation. On launch the card re-attaches and replays the journal,
//!   which is the authority; keeping a second copy here would eventually
//!   disagree with it.
//! * **Plain data only.** Makepad's `Vec2d`/`Rect`/enums are mirrored as
//!   serde structs so this module round-trips without the widget crate and
//!   can be unit-tested as pure data.
//!
//! STATUS (2026-09-15): model + tests written, but the test run is pending —
//! the machine currently refuses every link step (Xcode license relaunch).
//! `cargo check` passes; re-run `cargo test -p canvas-terminal persist`
//! before building on this.

use serde::{Deserialize, Serialize};

/// Current on-disk format. Bumped when a change would make old files load
/// incorrectly; older files are then discarded rather than migrated (the
/// canvas is a convenience cache, not a document).
pub const CANVAS_FORMAT_VERSION: u32 = 1;

/// Where the canvas state is kept. Matches the platform data directory used
/// by the rest of the app's tooling (CEF bundle, terminal sockets).
pub fn default_path() -> std::path::PathBuf {
    let base = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"));
    #[cfg(target_os = "macos")]
    {
        base.join("Library/Application Support/canvas-terminal/workspace.json")
    }
    #[cfg(target_os = "windows")]
    {
        base.join("AppData/Local/canvas-terminal/workspace.json")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        base.join(".local/share/canvas-terminal/workspace.json")
    }
}

/// A 2D point in world coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A world-space rectangle: top-left corner plus size.
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SavedRect {
    pub pos: Point,
    pub size: Point,
}

/// Which kind of card this is, and the minimum needed to recreate it.
///
/// Terminals and agents are re-created by re-attaching to their daemon
/// sessions, so only the name/identity is stored; a terminal whose session
/// has exited is reported as unrecoverable rather than silently respawned.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SavedItem {
    Note {
        title: String,
        body: String,
        font_size: f32,
        color_idx: usize,
        /// Wall-clock ms of the last edit. Absent in canvases saved before the
        /// note cards carried an "edited …" stamp.
        #[serde(default)]
        edited_ms: i64,
        rect: SavedRect,
    },
    Terminal {
        name: String,
        command: String,
        /// PTY working directory, when known. Absent in canvases saved
        /// before this field existed, and possibly empty even in newer
        /// ones (e.g. a re-attached session whose original cwd was never
        /// observed). A dead session with no cwd on file falls back to
        /// spawning fresh in the daemon's own working directory.
        #[serde(default)]
        cwd: String,
        rect: SavedRect,
    },
    Browser {
        title: String,
        url: String,
        rect: SavedRect,
    },
    MusicPlayer {
        title: String,
        progress: f32,
        rect: SavedRect,
    },
    Media {
        title: String,
        path: String,
        rect: SavedRect,
    },
    Agent {
        name: String,
        cwd: String,
        /// Which CLI hosts the chat ("pi" / "claude" / "codex").
        provider: String,
        /// The CLI's own session id, when it reported one: the restore path
        /// relaunches with `--session`/`--resume`, so the conversation
        /// survives a full app restart (daemon included).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cli_session_id: Option<String>,
        rect: SavedRect,
    },
}

impl SavedItem {
    /// Accessors for the canvas wiring, which maps `CanvasItem` <-> `SavedItem`
    /// on save and restore. Unused until that wiring lands; kept next to the
    /// data they belong to.
    #[allow(dead_code)]
    pub fn title(&self) -> &str {
        match self {
            Self::Note { title, .. }
            | Self::Browser { title, .. }
            | Self::MusicPlayer { title, .. }
            | Self::Media { title, .. } => title,
            // A terminal or agent card is titled after its daemon session.
            Self::Terminal { name, .. } | Self::Agent { name, .. } => name,
        }
    }

    #[allow(dead_code)]
    pub fn rect(&self) -> SavedRect {
        match self {
            Self::Note { rect, .. }
            | Self::Terminal { rect, .. }
            | Self::Browser { rect, .. }
            | Self::MusicPlayer { rect, .. }
            | Self::Media { rect, .. }
            | Self::Agent { rect, .. } => *rect,
        }
    }
}

/// One workspace: camera, cards, and the whiteboard drawings.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SavedWorkspace {
    #[serde(default)]
    pub name: String,
    pub camera_pan: Point,
    pub camera_zoom: f32,
    #[serde(default)]
    pub items: Vec<SavedItem>,
    /// Cards parked as top-bar tabs. Kept separate from `items` so a card is
    /// never saved twice, and `#[serde(default)]` so canvases written before
    /// the tab strip still load.
    #[serde(default)]
    pub minimized: Vec<SavedItem>,
    /// Whiteboard strokes, in world coordinates, with the ink they were
    /// drawn in so each keeps its own style.
    #[serde(default)]
    pub shapes: Vec<SavedShape>,
}

/// One whiteboard drawing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedShape {
    /// arrow | pen | line | rect | circle | ellipse | polyline | text
    pub kind: String,
    #[serde(default)]
    pub points: Vec<Point>,
    /// Radius, for `circle`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub color: [f32; 4],
    pub width: f64,
    /// The hand-drawn wobble seed; stored so a restored stroke keeps its
    /// exact shape instead of wobbling differently on every launch.
    pub seed: u32,
}

impl SavedShape {
    /// From a drawn shape (whiteboard layer).
    pub fn from_shape(shape: &crate::items::DrawnShape) -> Self {
        use crate::items::NoteShape;
        let (kind, points, radius, text) = match &shape.shape {
            NoteShape::Arrow { a, b } => ("arrow", vec![*a, *b], None, None),
            NoteShape::Pen { points } => ("pen", points.clone(), None, None),
            NoteShape::Rect { a, b } => ("rect", vec![*a, *b], None, None),
            NoteShape::Circle { center, r } => ("circle", vec![*center], Some(*r), None),
            NoteShape::Ellipse { a, b } => ("ellipse", vec![*a, *b], None, None),
            NoteShape::Line { a, b } => ("line", vec![*a, *b], None, None),
            NoteShape::Polyline { points } => ("polyline", points.clone(), None, None),
            NoteShape::Text { pos, text } => ("text", vec![*pos], None, Some(text.clone())),
        };
        Self {
            kind: kind.to_owned(),
            points: points
                .into_iter()
                .map(|p| Point { x: p.x, y: p.y })
                .collect(),
            radius,
            text,
            color: shape.color,
            width: shape.width,
            seed: shape.seed,
        }
    }

    /// Back into a drawn shape; `None` for an unknown kind (forward
    /// compatibility: a newer file may carry kinds this build cannot draw).
    pub fn to_shape(&self) -> Option<crate::items::DrawnShape> {
        use crate::items::NoteShape;
        let pts: Vec<makepad_widgets::Vec2d> = self
            .points
            .iter()
            .map(|p| makepad_widgets::Vec2d { x: p.x, y: p.y })
            .collect();
        let first = || pts.first().copied();
        let second = || pts.get(1).copied();
        let shape = match self.kind.as_str() {
            "arrow" => NoteShape::Arrow {
                a: first()?,
                b: second()?,
            },
            "pen" => NoteShape::Pen { points: pts },
            "rect" => NoteShape::Rect {
                a: first()?,
                b: second()?,
            },
            "circle" => NoteShape::Circle {
                center: first()?,
                r: self.radius?,
            },
            "ellipse" => NoteShape::Ellipse {
                a: first()?,
                b: second()?,
            },
            "line" => NoteShape::Line {
                a: first()?,
                b: second()?,
            },
            "polyline" => NoteShape::Polyline { points: pts },
            "text" => NoteShape::Text {
                pos: first()?,
                text: self.text.clone().unwrap_or_default(),
            },
            _ => return None,
        };
        Some(crate::items::DrawnShape {
            shape,
            color: self.color,
            width: self.width,
            seed: self.seed,
        })
    }
}

/// The whole saved canvas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedCanvas {
    pub version: u32,
    pub current: usize,
    pub workspaces: Vec<SavedWorkspace>,
}

impl SavedCanvas {
    pub fn empty() -> Self {
        Self {
            version: CANVAS_FORMAT_VERSION,
            current: 0,
            workspaces: vec![SavedWorkspace::default()],
        }
    }

    /// True when the file on disk cannot be applied to this build.
    pub fn is_foreign_version(&self) -> bool {
        self.version != CANVAS_FORMAT_VERSION
    }

    /// Serialize with stable field order and a trailing newline.
    pub fn to_json(&self) -> serde_json::Result<String> {
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        Ok(json)
    }

    pub fn from_json(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text)
    }

    /// Write atomically: the temporary file lands in the same directory and
    /// is renamed over the target, so a crash mid-write cannot leave a
    /// truncated canvas behind.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;

        let json = self
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::File::create(&tmp)?.write_all(json.as_bytes())?;
        std::fs::rename(&tmp, path)
    }

    /// Read a canvas, treating any decode failure as "no canvas" — a corrupt
    /// file should cost the user their layout once, not block every launch.
    pub fn load(path: &std::path::Path) -> Option<SavedCanvas> {
        let text = std::fs::read_to_string(path).ok()?;
        match Self::from_json(&text) {
            Ok(canvas) if !canvas.is_foreign_version() => Some(canvas),
            // Foreign or corrupt: log-less, but the caller sees None and
            // starts from an empty canvas rather than crashing.
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(canvas: &SavedCanvas) -> SavedCanvas {
        let json = canvas.to_json().expect("serialize");
        SavedCanvas::from_json(&json).expect("deserialize")
    }

    #[test]
    fn empty_canvas_round_trips() {
        let restored = roundtrip(&SavedCanvas::empty());
        assert_eq!(restored.version, CANVAS_FORMAT_VERSION);
        assert_eq!(restored.current, 0);
        assert_eq!(restored.workspaces.len(), 1);
    }

    #[test]
    fn every_item_kind_round_trips() {
        let rect = SavedRect {
            pos: Point { x: 10.0, y: 20.0 },
            size: Point { x: 480.0, y: 400.0 },
        };
        let canvas = SavedCanvas {
            version: CANVAS_FORMAT_VERSION,
            current: 0,
            workspaces: vec![SavedWorkspace {
                name: "main".into(),
                camera_pan: Point { x: -3.5, y: 12.0 },
                camera_zoom: 1.25,
                items: vec![
                    SavedItem::Note {
                        title: "note".into(),
                        body: "body".into(),
                        font_size: 13.0,
                        color_idx: 2,
                        edited_ms: 1_773_329_520_000,
                        rect,
                    },
                    SavedItem::Terminal {
                        name: "claude".into(),
                        command: "zsh".into(),
                        cwd: "/tmp/ws".into(),
                        rect,
                    },
                    SavedItem::Browser {
                        title: "docs".into(),
                        url: "https://github.com".into(),
                        rect,
                    },
                    SavedItem::MusicPlayer {
                        title: "music".into(),
                        progress: 0.5,
                        rect,
                    },
                    SavedItem::Media {
                        title: "pic".into(),
                        path: "/tmp/a.png".into(),
                        rect,
                    },
                    SavedItem::Agent {
                        name: "demo".into(),
                        cwd: "/tmp/ws".into(),
                        provider: "pi".into(),
                        cli_session_id: Some("sid-1".into()),
                        rect,
                    },
                ],
                // A card parked as a tab is part of the workspace too.
                minimized: vec![SavedItem::Note {
                    title: "parked".into(),
                    body: "# parked\n- [ ] later".into(),
                    font_size: 16.0,
                    color_idx: 3,
                    edited_ms: 1_773_329_520_000,
                    rect,
                }],
                shapes: vec![SavedShape {
                    kind: "pen".into(),
                    points: vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }],
                    radius: None,
                    text: None,
                    color: [0.1, 0.2, 0.3, 1.0],
                    width: 2.0,
                    seed: 99,
                }],
            }],
        };

        let restored = roundtrip(&canvas);
        let ws = &restored.workspaces[0];
        assert_eq!(ws.items.len(), 6);
        assert_eq!(ws.camera_zoom, 1.25);
        assert_eq!(ws.shapes.len(), 1);
        assert_eq!(ws.shapes[0].seed, 99);
        assert_eq!(ws.shapes[0].color, [0.1, 0.2, 0.3, 1.0]);

        // The agent card keeps its identity so restore can resume the CLI
        // session even after a full app restart.
        match &ws.items[5] {
            SavedItem::Agent {
                name,
                provider,
                cli_session_id,
                ..
            } => {
                assert_eq!(name, "demo");
                assert_eq!(provider, "pi");
                assert_eq!(cli_session_id.as_deref(), Some("sid-1"));
            }
            other => panic!("expected the agent item, got {other:?}"),
        }
        assert_eq!(ws.items[0].title(), "note");
    }

    #[test]
    fn a_foreign_version_is_reported_not_loaded() {
        let canvas = SavedCanvas::empty();
        // Mutate through serde_json::Value so the test does not depend on
        // the pretty-printer's exact spacing.
        let mut value: serde_json::Value =
            serde_json::from_str(&canvas.to_json().unwrap()).expect("parses");
        value["version"] = serde_json::json!(999);
        let json = value.to_string();
        let parsed = SavedCanvas::from_json(&json).expect("parses");
        assert!(parsed.is_foreign_version());
    }

    #[test]
    fn corrupt_json_is_an_error_not_a_panic() {
        assert!(SavedCanvas::from_json("{ not json").is_err());
        assert!(SavedCanvas::load(std::path::Path::new("/nonexistent/x.json")).is_none());
    }

    #[test]
    fn save_is_atomic_and_reloadable() {
        let dir = std::env::temp_dir().join(format!(
            "ct-persist-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("workspace.json");

        let mut canvas = SavedCanvas::empty();
        canvas.workspaces[0].items.push(SavedItem::Agent {
            name: "demo".into(),
            cwd: "/tmp".into(),
            provider: "pi".into(),
            cli_session_id: None,
            rect: SavedRect::default(),
        });
        canvas.save(&path).expect("save");

        // The temp file is gone; only the final artifact remains.
        assert!(!path.with_extension("json.tmp").exists());
        let loaded = SavedCanvas::load(&path).expect("reload");
        assert_eq!(loaded.workspaces[0].items.len(), 1);

        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    use std::time::SystemTime;
}
