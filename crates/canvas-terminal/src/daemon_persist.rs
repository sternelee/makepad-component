//! Daemon-owned session registry: what the daemon remembers about its
//! sessions' *identity* across its own restarts.
//!
//! This is deliberately separate from [`crate::persist`], which is the
//! GUI's canvas layout (cards, camera, whiteboard). That file only helps a
//! *GUI* restart find its way back to daemon sessions that are still alive;
//! it says nothing if the *daemon* itself restarts (an update, a crash, a
//! machine reboot that supervises it back up). Borrowing herdr's principle
//! that a shared runtime fact belongs in server state, not only in a
//! client's local file: session name, how to relaunch it (argv/cwd), and
//! which CLI conversation it was hosting (provider + the CLI's own session
//! id) are recorded here, by the daemon, as they change. A daemon that
//! restarts loads this file and can `Resume` a session by name even if no
//! GUI ever reconnects, or if the GUI's own `workspace.json` is stale/absent.
//!
//! Same on-disk discipline as `persist.rs`: plain data, atomic writes, and a
//! foreign/corrupt file is treated as "no registry" rather than a crash.

use serde::{Deserialize, Serialize};

/// Current on-disk format. Bumped when a change would make old files load
/// incorrectly; older files are then discarded (this is a resume cache, not
/// a document — losing it just means the next restart falls back to fresh
/// shells).
pub const REGISTRY_FORMAT_VERSION: u32 = 1;

/// Where the registry is kept: the daemon's own file, next to (but distinct
/// from) the GUI's `workspace.json`.
pub fn default_path() -> std::path::PathBuf {
    let base = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"));
    #[cfg(target_os = "macos")]
    {
        base.join("Library/Application Support/canvas-terminal/daemon-sessions.json")
    }
    #[cfg(target_os = "windows")]
    {
        base.join("AppData/Local/canvas-terminal/daemon-sessions.json")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        base.join(".local/share/canvas-terminal/daemon-sessions.json")
    }
}

/// The registry path for a specific daemon endpoint `label`.
///
/// The default label (`crate::ipc::LABEL`) uses [`default_path`]. Any other
/// label — an isolated test daemon, or a second instance started via
/// `CANVAS_TERMINAL_DAEMON_LABEL` — gets its own file under the system temp
/// directory, keyed by that label. This mirrors `rmux-ipc`'s own socket
/// endpoint for a non-default label (also temp-dir-scoped), so an isolated
/// instance can never race on or clobber the default installation's
/// persistent registry, and leaves nothing behind in the real application
/// support directory.
pub fn path_for_label(label: &str) -> std::path::PathBuf {
    if label == crate::ipc::LABEL {
        return default_path();
    }
    let safe_label: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect();
    std::env::temp_dir().join(format!("canvas-terminal-daemon-sessions-{safe_label}.json"))
}

/// One session's identity, as the daemon needs it to relaunch/resume.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SavedSession {
    pub name: String,
    /// The argv the PTY was spawned with (shell + `-lc` + command, same
    /// shape `spawn_session` builds `ChildCommand` from).
    #[serde(default)]
    pub argv: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Which CLI hosts the chat (`"pi"` / `"claude"` / `"codex"`), when the
    /// session ever had its chat parser turned on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_provider: Option<String>,
    /// The CLI's own session id, when it reported one — resume relaunches
    /// with `--session`/`--resume` so the conversation survives a daemon
    /// restart, not just a GUI restart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_session_id: Option<String>,
}

/// The whole registry: every session identity the daemon has ever recorded.
/// Entries are not pruned on session exit — a session that exits is still
/// "known" and resumable (its CLI conversation may not be), so `Resume`
/// keeps working after a plain `Kill` was actually a crash the daemon
/// reaped. Entries are only replaced (by name) on the next `Create`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedRegistry {
    pub version: u32,
    #[serde(default)]
    pub sessions: Vec<SavedSession>,
}

impl SavedRegistry {
    pub fn empty() -> Self {
        Self {
            version: REGISTRY_FORMAT_VERSION,
            sessions: Vec::new(),
        }
    }

    /// True when the file on disk cannot be applied to this build.
    pub fn is_foreign_version(&self) -> bool {
        self.version != REGISTRY_FORMAT_VERSION
    }

    /// Insert or replace the entry named `session.name`.
    pub fn upsert(&mut self, session: SavedSession) {
        if let Some(existing) = self.sessions.iter_mut().find(|s| s.name == session.name) {
            *existing = session;
        } else {
            self.sessions.push(session);
        }
    }

    pub fn get(&self, name: &str) -> Option<&SavedSession> {
        self.sessions.iter().find(|s| s.name == name)
    }

    /// Update (or create) the chat identity fields for `name`, preserving
    /// whatever argv/cwd is already on file. Used when the daemon learns a
    /// session's hosted CLI and its own session id straight from the live
    /// PTY stream (`SessionInfo`), independent of whether `Create`/`Resume`
    /// already registered the session's argv/cwd.
    pub fn set_chat(
        &mut self,
        name: &str,
        chat_provider: Option<String>,
        cli_session_id: Option<String>,
    ) {
        if let Some(existing) = self.sessions.iter_mut().find(|s| s.name == name) {
            if chat_provider.is_some() {
                existing.chat_provider = chat_provider;
            }
            if cli_session_id.is_some() {
                existing.cli_session_id = cli_session_id;
            }
        } else {
            self.sessions.push(SavedSession {
                name: name.to_owned(),
                argv: Vec::new(),
                cwd: None,
                chat_provider,
                cli_session_id,
            });
        }
    }

    /// Rename a known entry in place (a no-op if `old` is not on file).
    pub fn rename(&mut self, old: &str, new: &str) {
        if let Some(existing) = self.sessions.iter_mut().find(|s| s.name == old) {
            existing.name = new.to_owned();
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        let mut json = serde_json::to_string_pretty(self)?;
        json.push('\n');
        Ok(json)
    }

    pub fn from_json(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text)
    }

    /// Write atomically: temp file in the same directory, then renamed over
    /// the target, so a crash mid-write cannot leave a truncated registry.
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

    /// Read a registry, treating any decode failure as "no registry" — a
    /// corrupt file should cost the daemon its resume cache once, not block
    /// every startup.
    pub fn load(path: &std::path::Path) -> Option<SavedRegistry> {
        let text = std::fs::read_to_string(path).ok()?;
        match Self::from_json(&text) {
            Ok(registry) if !registry.is_foreign_version() => Some(registry),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(registry: &SavedRegistry) -> SavedRegistry {
        let json = registry.to_json().expect("serialize");
        SavedRegistry::from_json(&json).expect("deserialize")
    }

    #[test]
    fn empty_registry_round_trips() {
        let restored = roundtrip(&SavedRegistry::empty());
        assert_eq!(restored.version, REGISTRY_FORMAT_VERSION);
        assert!(restored.sessions.is_empty());
    }

    #[test]
    fn upsert_replaces_by_name() {
        let mut registry = SavedRegistry::empty();
        registry.upsert(SavedSession {
            name: "demo".into(),
            argv: vec!["zsh".into()],
            cwd: Some("/tmp".into()),
            chat_provider: None,
            cli_session_id: None,
        });
        registry.upsert(SavedSession {
            name: "demo".into(),
            argv: vec!["zsh".into()],
            cwd: Some("/tmp".into()),
            chat_provider: Some("pi".into()),
            cli_session_id: Some("sid-1".into()),
        });
        assert_eq!(registry.sessions.len(), 1);
        let entry = registry.get("demo").expect("present");
        assert_eq!(entry.chat_provider.as_deref(), Some("pi"));
        assert_eq!(entry.cli_session_id.as_deref(), Some("sid-1"));
    }

    #[test]
    fn full_entry_round_trips() {
        let mut registry = SavedRegistry::empty();
        registry.upsert(SavedSession {
            name: "writer".into(),
            argv: vec!["/bin/zsh".into(), "-lc".into(), "zsh".into()],
            cwd: Some("/Users/me/project".into()),
            chat_provider: Some("claude".into()),
            cli_session_id: Some("abc-123".into()),
        });
        let restored = roundtrip(&registry);
        let entry = restored.get("writer").expect("present");
        assert_eq!(entry.argv, vec!["/bin/zsh", "-lc", "zsh"]);
        assert_eq!(entry.cwd.as_deref(), Some("/Users/me/project"));
        assert_eq!(entry.chat_provider.as_deref(), Some("claude"));
        assert_eq!(entry.cli_session_id.as_deref(), Some("abc-123"));
    }

    #[test]
    fn a_foreign_version_is_reported_not_loaded() {
        let registry = SavedRegistry::empty();
        let mut value: serde_json::Value =
            serde_json::from_str(&registry.to_json().unwrap()).expect("parses");
        value["version"] = serde_json::json!(999);
        let json = value.to_string();
        let parsed = SavedRegistry::from_json(&json).expect("parses");
        assert!(parsed.is_foreign_version());
    }

    #[test]
    fn corrupt_json_is_an_error_not_a_panic() {
        assert!(SavedRegistry::from_json("{ not json").is_err());
        assert!(SavedRegistry::load(std::path::Path::new("/nonexistent/x.json")).is_none());
    }

    #[test]
    fn save_is_atomic_and_reloadable() {
        let dir = std::env::temp_dir().join(format!(
            "ct-daemon-persist-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("daemon-sessions.json");

        let mut registry = SavedRegistry::empty();
        registry.upsert(SavedSession {
            name: "demo".into(),
            argv: vec!["zsh".into()],
            cwd: None,
            chat_provider: None,
            cli_session_id: None,
        });
        registry.save(&path).expect("save");

        assert!(!path.with_extension("json.tmp").exists());
        let loaded = SavedRegistry::load(&path).expect("reload");
        assert_eq!(loaded.sessions.len(), 1);

        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
