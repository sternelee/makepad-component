/// Unified command input parsing.
///
/// Supported forms:
/// - `@name message...`   → send raw text to the terminal named `name`
/// - `/new note`          → create a note item
/// - `/new terminal NAME[:CWD]` → create a terminal item (optional working dir)
/// - `/new browser URL`   → create a browser item
/// - `/focus NAME`        → focus the terminal named `NAME`
/// - `/rename OLD NEW`    → rename a terminal
/// - `/zoom 1.5`          → set zoom
/// - `/clear`             → clear all whiteboard shapes
/// - `/help`              → show usage
/// - anything else        → sent to the active terminal
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Send raw text (already trimmed of the leading `@target `) to terminal `target`.
    Send {
        target: String,
        text: String,
    },
    NewNote,
    NewTerminal {
        name: String,
        /// Working directory for the new terminal's PTY (None = current dir).
        cwd: Option<String>,
    },
    NewBrowser {
        url: String,
    },
    NewMusicPlayer {
        title: String,
    },
    SetStatus {
        name: String,
        status: String,
    },
    Focus {
        name: String,
    },
    Rename {
        old: String,
        new: String,
    },
    Zoom {
        factor: f32,
    },
    Help,
    /// Clear all whiteboard shapes.
    Clear,
    /// Toggle the grid overlay on the CNVS background.
    Grid,
    /// No recognized prefix: forward to the active terminal.
    Forward {
        text: String,
    },
}

pub fn parse(line: &str) -> Command {
    let line = line.trim();
    if line.is_empty() {
        return Command::Forward {
            text: String::new(),
        };
    }
    if let Some(rest) = line.strip_prefix('@') {
        if let Some((target, text)) = rest.split_once(char::is_whitespace) {
            return Command::Send {
                target: target.to_string(),
                text: text.trim().to_string(),
            };
        }
        return Command::Send {
            target: rest.to_string(),
            text: String::new(),
        };
    }
    if let Some(rest) = line.strip_prefix('/') {
        let mut parts = rest.split_whitespace();
        match parts.next() {
            Some("new") => match parts.next() {
                Some("note") => return Command::NewNote,
                Some("terminal") => {
                    let spec = parts.next().unwrap_or("term").to_string();
                    // NAME[:CWD] — split on the FIRST ':' so CWDs containing
                    // ':' (e.g. `C:/...` on Windows) still work.
                    let (name, cwd) = match spec.split_once(':') {
                        Some((n, c)) => (n.to_string(), Some(c.to_string())),
                        None => (spec, None),
                    };
                    let name = if name.is_empty() {
                        "term".to_string()
                    } else {
                        name
                    };
                    return Command::NewTerminal { name, cwd };
                }
                Some("browser") => {
                    let url = parts.next().unwrap_or("https://github.com").to_string();
                    return Command::NewBrowser { url };
                }
                Some("music") => {
                    let title = parts.next().unwrap_or("Music").to_string();
                    return Command::NewMusicPlayer { title };
                }
                _ => {}
            },
            Some("focus") => {
                if let Some(name) = parts.next() {
                    return Command::Focus {
                        name: name.to_string(),
                    };
                }
            }
            Some("status") => {
                let name = parts.next().unwrap_or("").to_string();
                let status = parts.next().unwrap_or("online").to_string();
                if !name.is_empty() {
                    return Command::SetStatus { name, status };
                }
            }
            Some("zoom") => {
                if let Some(f) = parts.next().and_then(|s| s.parse::<f32>().ok()) {
                    return Command::Zoom { factor: f };
                }
            }
            Some("rename") => {
                let old = parts.next().unwrap_or("").to_string();
                let new = parts.next().unwrap_or("").to_string();
                if !old.is_empty() && !new.is_empty() {
                    return Command::Rename { old, new };
                }
            }
            Some("help") => return Command::Help,
            Some("clear") => return Command::Clear,
            Some("grid") => return Command::Grid,
            _ => {}
        }
    }
    Command::Forward {
        text: line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_at_target() {
        assert_eq!(
            parse("@claude explain this error"),
            Command::Send {
                target: "claude".into(),
                text: "explain this error".into()
            }
        );
    }

    #[test]
    fn parses_bare_target() {
        assert_eq!(
            parse("@codex"),
            Command::Send {
                target: "codex".into(),
                text: String::new()
            }
        );
    }

    #[test]
    fn parses_new_commands() {
        assert_eq!(parse("/new note"), Command::NewNote);
        assert_eq!(
            parse("/new terminal claude"),
            Command::NewTerminal {
                name: "claude".into(),
                cwd: None
            }
        );
        assert_eq!(
            parse("/new terminal myshell:~/projects/foo"),
            Command::NewTerminal {
                name: "myshell".into(),
                cwd: Some("~/projects/foo".into())
            }
        );
        assert_eq!(
            parse("/new browser https://github.com"),
            Command::NewBrowser {
                url: "https://github.com".into()
            }
        );
    }

    #[test]
    fn parses_zoom_and_focus() {
        assert_eq!(parse("/zoom 1.5"), Command::Zoom { factor: 1.5 });
        assert_eq!(
            parse("/focus claude"),
            Command::Focus {
                name: "claude".into()
            }
        );
        assert_eq!(parse("/help"), Command::Help);
    }

    #[test]
    fn parses_clear() {
        assert_eq!(parse("/clear"), Command::Clear);
    }

    #[test]
    fn forwards_plain_text() {
        assert_eq!(
            parse("just some text"),
            Command::Forward {
                text: "just some text".into()
            }
        );
    }
}
