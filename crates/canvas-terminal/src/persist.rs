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
        rect: SavedRect,
    },
    Terminal {
        name: String,
        command: String,
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
        provider: String,
        /// The daemon session this card talks to. `cursor.seq` tells the
        /// re-attach how far the card had rendered; the transcript itself
        /// replays from the daemon's journal.
        session_id: u64,
        epoch: u64,
        seq: u64,
        rect: SavedRect,
    },
}

impl SavedItem {
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
                        rect,
                    },
                    SavedItem::Terminal {
                        name: "claude".into(),
                        command: "zsh".into(),
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
                        provider: "kimi-k2.5".into(),
                        session_id: 42,
                        epoch: 7,
                        seq: 1234,
                        rect,
                    },
                ],
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

        // The agent card keeps its identity so re-attach can find the session.
        match &ws.items[5] {
            SavedItem::Agent {
                name,
                session_id,
                epoch,
                seq,
                ..
            } => {
                assert_eq!(name, "demo");
                assert_eq!(*session_id, 42);
                assert_eq!(*epoch, 7);
                assert_eq!(*seq, 1234);
            }
            other => panic!("expected the agent item, got {other:?}"),
        }
        assert_eq!(ws.items[0].title(), "note");
    }

    #[test]
    fn a_foreign_version_is_reported_not_loaded() {
        let canvas = SavedCanvas::empty();
        let json = canvas.to_json().unwrap().replace("\"version\":1", "\"version\":999");
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
            provider: "scripted".into(),
            session_id: 1,
            epoch: 2,
            seq: 3,
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
