//! The tools a coding agent can call.
//!
//! Every tool takes a JSON object, resolves every path through
//! [`ToolContext::resolve`], and caps what it returns — a model that reads a
//! 400 MB log or greps a build tree has to fail cheaply, not hang the session.

mod files;
mod search;
mod shell;

pub use files::{EditFile, ReadFile, WriteFile};
pub use search::{FindFiles, SearchFiles};
pub use shell::RunCommand;

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::{AgentError, Result};

/// Hard ceiling on any single tool result fed back to the model.
pub const MAX_RESULT_BYTES: usize = 64 * 1024;
/// Lines a bare `read_file` returns when the caller does not say.
pub const DEFAULT_READ_LINES: usize = 400;
/// Files a single `find_files` may return.
pub const MAX_FIND_RESULTS: usize = 200;
/// Matches a single `search_files` may return.
pub const MAX_SEARCH_RESULTS: usize = 200;
/// Directory entries a single walk will visit before giving up.
pub const MAX_WALK_ENTRIES: usize = 20_000;

/// Directories never worth walking for a coding agent. Kept explicit rather
/// than reading `.gitignore` so behaviour is identical in every workspace.
pub(crate) const SKIP_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "target",
    "node_modules",
    ".venv",
    "venv",
    "__pycache__",
    "dist",
    "build",
    ".next",
    ".cache",
];

/// Read a required string argument.
pub(crate) fn arg_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| AgentError::Tool(format!("missing required string argument {key:?}")))
}

/// Read an optional string argument.
pub(crate) fn arg_opt_str<'a>(arguments: &'a Value, key: &str) -> Option<&'a str> {
    arguments.get(key).and_then(Value::as_str)
}

/// Read an optional unsigned integer argument.
pub(crate) fn arg_opt_usize(arguments: &Value, key: &str) -> Option<usize> {
    arguments
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| value as usize)
}

/// Read an optional boolean argument, defaulting to `false`.
pub(crate) fn arg_bool(arguments: &Value, key: &str) -> bool {
    arguments.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// Truncate a tool result at [`MAX_RESULT_BYTES`], marking the cut so the
/// model knows the output is incomplete rather than empty.
pub(crate) fn cap_result(text: String) -> String {
    if text.len() <= MAX_RESULT_BYTES {
        return text;
    }
    let mut cut = MAX_RESULT_BYTES;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!(
        "{}\n… [truncated: showing {} of {} bytes]",
        &text[..cut],
        cut,
        text.len()
    )
}

/// Render a line range with 1-based line numbers, the way a coding agent wants
/// to read a file: `12\tlet x = 1;`.
pub(crate) fn numbered_lines(text: &str, first_line: usize, max_lines: usize) -> (String, usize) {
    let mut out = String::new();
    let mut shown = 0usize;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if number < first_line {
            continue;
        }
        if shown >= max_lines {
            break;
        }
        out.push_str(&format!("{number}\t{line}\n"));
        shown += 1;
    }
    let total = text.lines().count();
    (out, total)
}

/// Walk `root` depth-first, calling `visit` on every regular file.
///
/// `visit` returns `false` to stop the whole walk. Only [`SKIP_DIRS`] are
/// pruned, and the walk gives up after [`MAX_WALK_ENTRIES`] entries so a huge
/// tree cannot stall a turn.
pub(crate) fn walk_files(root: &Path, mut visit: impl FnMut(&Path) -> bool) {
    let mut queue: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut visited = 0usize;
    while let Some(dir) = queue.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            visited += 1;
            if visited > MAX_WALK_ENTRIES {
                return;
            }
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if SKIP_DIRS.contains(&name.as_ref()) {
                    continue;
                }
                queue.push(path);
            } else if file_type.is_file() && !visit(&path) {
                return;
            }
        }
    }
}

/// Path relative to `root`, with `/` separators, for stable output on Windows.
pub(crate) fn relative_display(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Match a glob pattern against a `/`-separated relative path.
///
/// `**` spans zero or more segments, `*` and `?` stay inside one segment:
/// `src/**/*.rs` matches `src/main.rs` and `src/widgets/button.rs`.
pub(crate) fn glob_match(pattern: &str, path: &str) -> bool {
    /// Wildcard match inside a single path segment.
    fn segment_match(pattern: &str, name: &str) -> bool {
        let pattern: Vec<char> = pattern.chars().collect();
        let name: Vec<char> = name.chars().collect();
        let (mut pi, mut ni) = (0usize, 0usize);
        let mut star: Option<usize> = None;
        let mut star_resume = 0usize;
        while ni < name.len() {
            if pi < pattern.len() && (pattern[pi] == '?' || pattern[pi] == name[ni]) {
                pi += 1;
                ni += 1;
            } else if pi < pattern.len() && pattern[pi] == '*' {
                star = Some(pi);
                star_resume = ni;
                pi += 1;
            } else if let Some(star_at) = star {
                pi = star_at + 1;
                star_resume += 1;
                ni = star_resume;
            } else {
                return false;
            }
        }
        while pi < pattern.len() && pattern[pi] == '*' {
            pi += 1;
        }
        pi == pattern.len()
    }

    fn match_segments(pattern: &[&str], path: &[&str]) -> bool {
        let Some((head, rest)) = pattern.split_first() else {
            return path.is_empty();
        };
        if *head == "**" {
            // `**` consumes zero segments (try the rest here) or one more.
            if match_segments(rest, path) {
                return true;
            }
            return !path.is_empty() && match_segments(pattern, &path[1..]);
        }
        match path.split_first() {
            Some((segment, remaining)) => {
                segment_match(head, segment) && match_segments(rest, remaining)
            }
            None => false,
        }
    }

    let pattern: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match_segments(&pattern, &path)
}

/// JSON Schema fragment for a required string property.
pub(crate) fn string_property(description: &str) -> Value {
    serde_json::json!({ "type": "string", "description": description })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolContext;

    #[test]
    fn glob_star_stays_within_one_segment() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "src/main.rs"));
        assert!(glob_match("src/*.rs", "src/main.rs"));
        assert!(!glob_match("src/*.rs", "src/widgets/button.rs"));
    }

    #[test]
    fn glob_double_star_spans_directories() {
        assert!(glob_match("src/**/*.rs", "src/main.rs"));
        assert!(glob_match("src/**/*.rs", "src/widgets/button.rs"));
        assert!(glob_match("src/**/*.rs", "src/a/b/c/d.rs"));
        assert!(!glob_match("src/**/*.rs", "crates/main.rs"));
        assert!(glob_match("**/*.md", "docs/a/b.md"));
        assert!(glob_match("**", "anything/at/all.txt"));
    }

    #[test]
    fn glob_question_mark_matches_one_character() {
        assert!(glob_match("a?c.rs", "abc.rs"));
        assert!(!glob_match("a?c.rs", "ac.rs"));
        assert!(!glob_match("a?c.rs", "abbc.rs"));
    }

    #[test]
    fn glob_multiple_stars_in_one_segment() {
        assert!(glob_match("*_test.rs", "button_test.rs"));
        assert!(glob_match("a*b*c", "aXXbYYc"));
        assert!(!glob_match("a*b*c", "aXXbYY"));
    }

    #[test]
    fn numbered_lines_windows_the_file_and_reports_total() {
        let text = "one\ntwo\nthree\nfour\n";
        let (body, total) = numbered_lines(text, 2, 2);
        assert_eq!(body, "2\ttwo\n3\tthree\n");
        assert_eq!(total, 4);
    }

    #[test]
    fn numbered_lines_past_the_end_is_empty_but_reports_total() {
        let (body, total) = numbered_lines("one\n", 9, 5);
        assert!(body.is_empty());
        assert_eq!(total, 1);
    }

    #[test]
    fn cap_result_leaves_short_text_alone_and_marks_cuts() {
        assert_eq!(cap_result("short".into()), "short");
        let long = "x".repeat(MAX_RESULT_BYTES + 10);
        let capped = cap_result(long);
        assert!(capped.contains("[truncated: showing"));
    }

    #[test]
    fn resolve_rejects_escapes_and_accepts_inside_paths() {
        let context = ToolContext::new("/tmp/agent-core-root");
        assert!(context.resolve("src/main.rs").is_ok());
        assert!(context.resolve("./src/../src/main.rs").is_ok());
        assert!(context.resolve("../../etc/passwd").is_err());
        assert!(context.resolve("/etc/passwd").is_err());
        assert!(context.resolve("/tmp/agent-core-root-other/x").is_err());
    }
}
