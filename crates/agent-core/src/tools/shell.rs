//! Running a shell command in the workspace.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::{arg_opt_str, arg_opt_usize, arg_str, cap_result, string_property};
use crate::error::{AgentError, Result};
use crate::tool::{Tool, ToolContext, ToolOutcome, ToolSpec};

/// Default wall-clock budget for one command.
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;
/// Hard ceiling, so a model cannot request an unbounded wait.
pub const MAX_TIMEOUT_MS: u64 = 600_000;

/// Run a shell command with the workspace as the working directory.
pub struct RunCommand;

impl Tool for RunCommand {
    fn spec(&self) -> ToolSpec {
        ToolSpec::new(
            "run_command",
            "Run a shell command in the workspace and return its combined \
             stdout/stderr. The command is killed when it exceeds `timeout_ms`.",
            json!({
                "type": "object",
                "properties": {
                    "command": string_property("Shell command line to execute"),
                    "cwd": string_property("Working directory relative to the workspace root (default: root)"),
                    "timeout_ms": { "type": "integer", "description": format!("Timeout in milliseconds (default {DEFAULT_TIMEOUT_MS}, max {MAX_TIMEOUT_MS})") }
                },
                "required": ["command"]
            }),
        )
    }

    fn invoke(&self, arguments: &Value, context: &ToolContext) -> Result<ToolOutcome> {
        let command = arg_str(arguments, "command")?;
        let cwd = match arg_opt_str(arguments, "cwd") {
            Some(path) => context.resolve(path)?,
            None => context.root().to_path_buf(),
        };
        let timeout = Duration::from_millis(
            arg_opt_usize(arguments, "timeout_ms")
                .map(|ms| ms as u64)
                .unwrap_or(DEFAULT_TIMEOUT_MS)
                .clamp(1, MAX_TIMEOUT_MS),
        );

        let mut process = shell(command);
        process
            .current_dir(&cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = process
            .spawn()
            .map_err(|error| AgentError::Tool(format!("could not spawn {command:?}: {error}")))?;

        // Drain both pipes on their own threads: a child that fills a pipe
        // buffer would otherwise block while we sit in `try_wait`, and the
        // "timeout" would never fire.
        let stdout = child.stdout.take().map(drain_on_thread);
        let stderr = child.stderr.take().map(drain_on_thread);

        let deadline = Instant::now() + timeout;
        let mut timed_out = false;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => {
                    if Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        timed_out = true;
                        break None;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(error) => {
                    let _ = child.kill();
                    return Err(AgentError::Tool(format!("waiting on {command:?}: {error}")));
                }
            }
        };

        let mut output = String::new();
        if let Some(handle) = stdout {
            output.push_str(&handle.join().unwrap_or_default());
        }
        if let Some(handle) = stderr {
            let text = handle.join().unwrap_or_default();
            if !text.is_empty() {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(&text);
            }
        }

        let header = if timed_out {
            format!("command timed out after {}ms", timeout.as_millis())
        } else {
            match status {
                Some(status) => match status.code() {
                    Some(code) => format!("exit code {code}"),
                    None => "terminated by signal".to_owned(),
                },
                None => "killed".to_owned(),
            }
        };
        let body = if output.trim().is_empty() {
            "(no output)".to_owned()
        } else {
            output
        };
        let text = cap_result(format!("{header}\n{body}"));
        Ok(if timed_out {
            ToolOutcome::err(text)
        } else {
            ToolOutcome::ok(text)
        })
    }
}

/// Build the platform shell invocation for one command line.
fn shell(command: &str) -> Command {
    #[cfg(windows)]
    {
        let mut command_line = Command::new("cmd");
        command_line.arg("/C").arg(command);
        command_line
    }
    #[cfg(not(windows))]
    {
        let mut command_line = Command::new("/bin/sh");
        command_line.arg("-c").arg(command);
        command_line
    }
}

/// Read a pipe to EOF on a background thread so the child never blocks.
fn drain_on_thread(mut pipe: impl Read + Send + 'static) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        // Bound what one command can pull in; the result is capped again
        // before it reaches the model.
        let _ = pipe.by_ref().take(4 * 1024 * 1024).read_to_end(&mut buffer);
        String::from_utf8_lossy(&buffer).into_owned()
    })
}
