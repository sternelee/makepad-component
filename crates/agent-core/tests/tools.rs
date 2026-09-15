//! Direct tests for the coding tools, against a real temporary workspace.

use std::path::PathBuf;

use agent_core::tools::{FindFiles, RunCommand, SearchFiles};
use agent_core::{Tool, ToolContext};
use serde_json::json;

/// Temporary workspace, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "agent-core-tools-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn find_files_walks_nested_directories_and_honours_the_glob() {
    let workspace = TempDir::new("find");
    workspace.write("src/main.rs", "fn main() {}\n");
    workspace.write("src/widgets/button.rs", "pub struct B;\n");
    workspace.write("docs/guide.md", "# guide\n");
    // Pruned by SKIP_DIRS.
    workspace.write("target/debug/junk.rs", "junk\n");
    workspace.write("node_modules/pkg/index.js", "junk\n");

    let context = ToolContext::new(&workspace.0);
    let outcome = FindFiles
        .invoke(&json!({ "pattern": "**/*.rs" }), &context)
        .expect("find should run");

    assert!(!outcome.is_error);
    let mut lines: Vec<&str> = outcome.content.lines().collect();
    lines.sort_unstable();
    assert_eq!(lines, vec!["src/main.rs", "src/widgets/button.rs"]);
}

#[test]
fn search_files_reports_path_line_and_text() {
    let workspace = TempDir::new("search");
    workspace.write("src/a.rs", "let alpha = 1;\nlet beta = 2;\n");
    workspace.write("src/b.rs", "// alpha again\n");
    workspace.write("notes.txt", "alpha in a text file\n");

    let context = ToolContext::new(&workspace.0);

    let outcome = SearchFiles
        .invoke(&json!({ "pattern": "alpha", "glob": "**/*.rs" }), &context)
        .expect("search");
    let mut lines: Vec<&str> = outcome.content.lines().collect();
    lines.sort_unstable();
    assert_eq!(
        lines,
        vec!["src/a.rs:1:let alpha = 1;", "src/b.rs:1:// alpha again"]
    );

    // Case-insensitive search reaches the text file too.
    let outcome = SearchFiles
        .invoke(
            &json!({ "pattern": "ALPHA", "ignore_case": true }),
            &context,
        )
        .expect("search");
    assert!(outcome.content.contains("notes.txt:1:alpha in a text file"));

    // A bad pattern is an infrastructure error, not a tool failure.
    let error = SearchFiles
        .invoke(&json!({ "pattern": "([unclosed" }), &context)
        .expect_err("invalid regex should be rejected before searching");
    assert!(error.to_string().contains("invalid regex"));
}

#[test]
fn run_command_reports_exit_code_and_output() {
    let workspace = TempDir::new("run");
    let context = ToolContext::new(&workspace.0);

    let outcome = RunCommand
        .invoke(&json!({ "command": "echo hello" }), &context)
        .expect("command should run");
    assert!(!outcome.is_error);
    assert!(
        outcome.content.contains("exit code 0"),
        "{}",
        outcome.content
    );
    assert!(outcome.content.contains("hello"), "{}", outcome.content);

    let outcome = RunCommand
        .invoke(&json!({ "command": "exit 3" }), &context)
        .expect("command should run");
    // A non-zero exit is a *successful* run: the model needs the code, not an
    // exception.
    assert!(!outcome.is_error);
    assert!(
        outcome.content.contains("exit code 3"),
        "{}",
        outcome.content
    );
}

#[test]
fn run_command_kills_a_command_that_overruns_its_timeout() {
    let workspace = TempDir::new("timeout");
    let context = ToolContext::new(&workspace.0);

    let started = std::time::Instant::now();
    let outcome = RunCommand
        .invoke(
            &json!({ "command": "sleep 30", "timeout_ms": 300 }),
            &context,
        )
        .expect("command should run");
    let elapsed = started.elapsed();

    assert!(outcome.is_error, "a timeout is reported as a tool failure");
    assert!(outcome.content.contains("timed out"), "{}", outcome.content);
    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "timeout should fire promptly, took {elapsed:?}"
    );
}

#[test]
fn run_command_refuses_to_leave_the_workspace() {
    let workspace = TempDir::new("escape");
    let context = ToolContext::new(&workspace.0);
    let error = RunCommand
        .invoke(&json!({ "command": "pwd", "cwd": "../.." }), &context)
        .expect_err("a cwd outside the root must be rejected");
    assert!(error.to_string().contains("outside the workspace root"));
}

// ---------------------------------------------------------------------------
// Sandbox regressions (S1). The lexical check alone is defeated by one
// `ln -s`, so containment is judged through the filesystem as well.
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn a_symlink_pointing_out_of_the_workspace_is_refused() {
    let workspace = TempDir::new("symlink-out");
    let outside = TempDir::new("outside");
    std::fs::write(outside.0.join("secret.txt"), "TOP SECRET\n").unwrap();
    std::os::unix::fs::symlink(outside.0.join("secret.txt"), workspace.0.join("link.txt")).unwrap();

    let context = ToolContext::new(&workspace.0);
    let error = agent_core::tools::ReadFile
        .invoke(&json!({ "path": "link.txt" }), &context)
        .expect_err("reading through a symlink out of the workspace must be refused");
    assert!(
        error.to_string().contains("outside the workspace root"),
        "unexpected error: {error}"
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_directory_pointing_out_of_the_workspace_is_refused() {
    let workspace = TempDir::new("symlink-dir");
    let outside = TempDir::new("outside-dir");
    std::fs::write(outside.0.join("secret.txt"), "TOP SECRET\n").unwrap();
    std::os::unix::fs::symlink(&outside.0, workspace.0.join("escape")).unwrap();

    let context = ToolContext::new(&workspace.0);
    let error = agent_core::tools::ReadFile
        .invoke(&json!({ "path": "escape/secret.txt" }), &context)
        .expect_err("a symlinked directory must not be traversable");
    assert!(
        error.to_string().contains("outside the workspace root"),
        "unexpected error: {error}"
    );

    // A write target that does not exist *inside* the link is refused too: the
    // deepest existing part of the path is the link itself.
    let error = agent_core::tools::WriteFile
        .invoke(
            &json!({ "path": "escape/new.txt", "content": "x" }),
            &context,
        )
        .expect_err("writing through a symlinked directory must be refused");
    assert!(error.to_string().contains("outside the workspace root"));
    assert!(!outside.0.join("new.txt").exists());
}

#[cfg(unix)]
#[test]
fn a_symlink_that_stays_inside_the_workspace_still_works() {
    let workspace = TempDir::new("symlink-in");
    std::fs::write(workspace.0.join("real.txt"), "inside\n").unwrap();
    std::os::unix::fs::symlink(workspace.0.join("real.txt"), workspace.0.join("alias.txt"))
        .unwrap();

    let context = ToolContext::new(&workspace.0);
    let outcome = agent_core::tools::ReadFile
        .invoke(&json!({ "path": "alias.txt" }), &context)
        .expect("an in-workspace symlink is legitimate");
    assert!(!outcome.is_error, "{}", outcome.content);
    assert!(outcome.content.contains("inside"));
}

#[test]
fn writing_a_file_that_does_not_exist_yet_still_resolves() {
    // The containment check has to work when the target is absent, which is the
    // normal case for `write_file`.
    let workspace = TempDir::new("new-file");
    let context = ToolContext::new(&workspace.0);
    let outcome = agent_core::tools::WriteFile
        .invoke(
            &json!({ "path": "deep/nested/new.txt", "content": "hello" }),
            &context,
        )
        .expect("creating a nested file inside the workspace should work");
    assert!(!outcome.is_error, "{}", outcome.content);
    assert_eq!(
        std::fs::read_to_string(workspace.0.join("deep/nested/new.txt")).unwrap(),
        "hello"
    );
}
