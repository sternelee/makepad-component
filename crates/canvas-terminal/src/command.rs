/// Unified command input parsing.
///
/// Supported forms:
/// - `@name message...`   → send raw text to the terminal named `name`
/// - `/new note`          → create a note item
/// - `/new terminal NAME` → create a terminal item
/// - `/new browser URL`   → create a browser item
/// - `/focus NAME`        → focus the terminal named `NAME`
/// - `/zoom 1.5`          → set zoom
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
    },
    NewBrowser {
        url: String,
    },
    Focus {
        name: String,
    },
    Zoom {
        factor: f32,
    },
    Help,
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
                    let name = parts.next().unwrap_or("term").to_string();
                    return Command::NewTerminal { name };
                }
                Some("browser") => {
                    let url = parts.next().unwrap_or("https://github.com").to_string();
                    return Command::NewBrowser { url };
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
            Some("zoom") => {
                if let Some(f) = parts.next().and_then(|s| s.parse::<f32>().ok()) {
                    return Command::Zoom { factor: f };
                }
            }
            Some("help") => return Command::Help,
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
                name: "claude".into()
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
    fn forwards_plain_text() {
        assert_eq!(
            parse("just some text"),
            Command::Forward {
                text: "just some text".into()
            }
        );
    }
}
