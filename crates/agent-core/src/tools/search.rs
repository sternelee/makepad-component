//! Finding files and searching their contents.

use serde_json::{json, Value};

use super::{
    arg_bool, arg_opt_str, arg_str, cap_result, glob_match, relative_display, string_property,
    walk_files, MAX_FIND_RESULTS, MAX_SEARCH_RESULTS,
};
use crate::error::{AgentError, Result};
use crate::tool::{Tool, ToolContext, ToolOutcome, ToolSpec};

/// List workspace files matching a glob.
pub struct FindFiles;

impl Tool for FindFiles {
    fn spec(&self) -> ToolSpec {
        ToolSpec::new(
            "find_files",
            "List workspace files whose path matches a glob. `**` spans \
             directories, `*` and `?` stay within one path segment \
             (e.g. `src/**/*.rs`). Build and vendor directories are skipped.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": string_property("Glob matched against the path relative to the workspace root"),
                    "path": string_property("Directory to search under, relative to the workspace root (default: root)")
                },
                "required": ["pattern"]
            }),
        )
    }

    fn invoke(&self, arguments: &Value, context: &ToolContext) -> Result<ToolOutcome> {
        let pattern = arg_str(arguments, "pattern")?;
        let root = context.root().to_path_buf();
        let search_root = match arg_opt_str(arguments, "path") {
            Some(path) => context.resolve(path)?,
            None => root.clone(),
        };

        let mut matches: Vec<String> = Vec::new();
        let mut truncated = false;
        walk_files(&search_root, |path| {
            let relative = relative_display(&root, path);
            if glob_match(pattern, &relative) {
                matches.push(relative);
                if matches.len() >= MAX_FIND_RESULTS {
                    truncated = true;
                    return false;
                }
            }
            true
        });

        if matches.is_empty() {
            return Ok(ToolOutcome::ok(format!("no files match {pattern:?}")));
        }
        matches.sort();
        let mut body = matches.join("\n");
        if truncated {
            body.push_str(&format!("\n… [stopped at {MAX_FIND_RESULTS} matches]"));
        }
        Ok(ToolOutcome::ok(cap_result(body)))
    }
}

/// Search file contents by regular expression.
pub struct SearchFiles;

impl Tool for SearchFiles {
    fn spec(&self) -> ToolSpec {
        ToolSpec::new(
            "search_files",
            "Search workspace file contents with a regular expression. Returns \
             `path:line:text` for each match. Build and vendor directories are \
             skipped, and binary files are ignored.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": string_property("Rust `regex` crate pattern"),
                    "path": string_property("Directory to search under, relative to the workspace root (default: root)"),
                    "glob": string_property("Only search files matching this glob (e.g. `**/*.rs`)"),
                    "ignore_case": { "type": "boolean", "description": "Case-insensitive match" }
                },
                "required": ["pattern"]
            }),
        )
    }

    fn invoke(&self, arguments: &Value, context: &ToolContext) -> Result<ToolOutcome> {
        let pattern = arg_str(arguments, "pattern")?;
        let root = context.root().to_path_buf();
        let search_root = match arg_opt_str(arguments, "path") {
            Some(path) => context.resolve(path)?,
            None => root.clone(),
        };
        let file_glob = arg_opt_str(arguments, "glob").map(str::to_owned);
        let ignore_case = arg_bool(arguments, "ignore_case");

        let expression = if ignore_case {
            format!("(?i){pattern}")
        } else {
            pattern.to_owned()
        };
        let regex = regex::Regex::new(&expression)
            .map_err(|error| AgentError::Tool(format!("invalid regex {pattern:?}: {error}")))?;

        let mut hits: Vec<String> = Vec::new();
        let mut truncated = false;
        walk_files(&search_root, |path| {
            let relative = relative_display(&root, path);
            if let Some(glob) = &file_glob {
                if !glob_match(glob, &relative) {
                    return true;
                }
            }
            let Ok(bytes) = std::fs::read(path) else {
                return true;
            };
            // Skip anything that is not plausibly UTF-8 text.
            let Ok(text) = std::str::from_utf8(&bytes) else {
                return true;
            };
            for (index, line) in text.lines().enumerate() {
                if regex.is_match(line) {
                    hits.push(format!("{relative}:{}:{}", index + 1, line.trim_end()));
                    if hits.len() >= MAX_SEARCH_RESULTS {
                        truncated = true;
                        return false;
                    }
                }
            }
            true
        });

        if hits.is_empty() {
            return Ok(ToolOutcome::ok(format!("no matches for {pattern:?}")));
        }
        let mut body = hits.join("\n");
        if truncated {
            body.push_str(&format!("\n… [stopped at {MAX_SEARCH_RESULTS} matches]"));
        }
        Ok(ToolOutcome::ok(cap_result(body)))
    }
}
