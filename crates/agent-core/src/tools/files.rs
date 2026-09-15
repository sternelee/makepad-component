//! File reading and mutation.

use serde_json::{json, Value};

use super::{
    arg_bool, arg_opt_usize, arg_str, cap_result, numbered_lines, string_property,
    DEFAULT_READ_LINES,
};
use crate::error::{AgentError, Result};
use crate::tool::{Tool, ToolContext, ToolOutcome, ToolSpec};

/// Read a file as numbered lines.
pub struct ReadFile;

impl Tool for ReadFile {
    fn spec(&self) -> ToolSpec {
        ToolSpec::new(
            "read_file",
            "Read a UTF-8 text file from the workspace. Returns numbered lines.",
            json!({
                "type": "object",
                "properties": {
                    "path": string_property("Path relative to the workspace root"),
                    "offset": { "type": "integer", "description": "First line to return, 1-based (default 1)" },
                    "limit": { "type": "integer", "description": format!("Maximum lines to return (default {DEFAULT_READ_LINES})") }
                },
                "required": ["path"]
            }),
        )
    }

    fn invoke(&self, arguments: &Value, context: &ToolContext) -> Result<ToolOutcome> {
        let path = context.resolve(arg_str(arguments, "path")?)?;
        let first_line = arg_opt_usize(arguments, "offset").unwrap_or(1).max(1);
        let limit = arg_opt_usize(arguments, "limit").unwrap_or(DEFAULT_READ_LINES);

        let bytes = std::fs::read(&path)?;
        let text = String::from_utf8_lossy(&bytes);
        let (body, total) = numbered_lines(&text, first_line, limit);
        if body.is_empty() {
            return Ok(ToolOutcome::ok(format!(
                "(no content: file has {total} lines, requested from line {first_line})"
            )));
        }
        let last = first_line + body.lines().count().saturating_sub(1);
        Ok(ToolOutcome::ok(cap_result(format!(
            "{body}… [lines {first_line}–{last} of {total}]"
        ))))
    }
}

/// Create or overwrite a file.
pub struct WriteFile;

impl Tool for WriteFile {
    fn spec(&self) -> ToolSpec {
        ToolSpec::new(
            "write_file",
            "Create or overwrite a workspace file with the given content. Parent \
             directories are created as needed.",
            json!({
                "type": "object",
                "properties": {
                    "path": string_property("Path relative to the workspace root"),
                    "content": string_property("Full file content to write")
                },
                "required": ["path", "content"]
            }),
        )
    }

    fn invoke(&self, arguments: &Value, context: &ToolContext) -> Result<ToolOutcome> {
        let path = context.resolve(arg_str(arguments, "path")?)?;
        let content = arg_str(arguments, "content")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(ToolOutcome::ok(format!(
            "wrote {} bytes to {}",
            content.len(),
            path.display()
        )))
    }
}

/// Replace an exact snippet in an existing file.
pub struct EditFile;

impl Tool for EditFile {
    fn spec(&self) -> ToolSpec {
        ToolSpec::new(
            "edit_file",
            "Replace an exact snippet in a workspace file. `old` must appear \
             exactly once so an edit can never land in the wrong place; include \
             surrounding context to make it unique.",
            json!({
                "type": "object",
                "properties": {
                    "path": string_property("Path relative to the workspace root"),
                    "old": string_property("Exact text to replace, including indentation"),
                    "new": string_property("Replacement text"),
                    "replace_all": { "type": "boolean", "description": "Replace every occurrence instead of requiring a unique one" }
                },
                "required": ["path", "old", "new"]
            }),
        )
    }

    fn invoke(&self, arguments: &Value, context: &ToolContext) -> Result<ToolOutcome> {
        let path = context.resolve(arg_str(arguments, "path")?)?;
        let old = arg_str(arguments, "old")?;
        let new = arg_str(arguments, "new")?;
        let replace_all = arg_bool(arguments, "replace_all");

        if old.is_empty() {
            return Err(AgentError::Tool("`old` must not be empty".into()));
        }

        let text = std::fs::read_to_string(&path)?;
        let occurrences = text.matches(old).count();
        if occurrences == 0 {
            return Ok(ToolOutcome::err(format!(
                "`old` was not found in {}",
                path.display()
            )));
        }
        if occurrences > 1 && !replace_all {
            return Ok(ToolOutcome::err(format!(
                "`old` occurs {occurrences} times in {}; add surrounding context or pass replace_all",
                path.display()
            )));
        }

        let updated = if replace_all {
            text.replace(old, new)
        } else {
            text.replacen(old, new, 1)
        };
        std::fs::write(&path, &updated)?;
        Ok(ToolOutcome::ok(format!(
            "replaced {occurrences} occurrence(s) in {}",
            path.display()
        )))
    }
}
