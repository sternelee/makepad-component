//! Tool contract, permission gate, and the registry the session loop drives.

use serde::{Deserialize, Serialize};

use crate::cancel::CancelToken;
use crate::error::{AgentError, Result};

/// The schema handed to the model so it knows a tool exists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// JSON Schema for the arguments object.
    pub parameters: serde_json::Value,
}

impl ToolSpec {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// A tool the model asked to run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// What a tool produced.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolOutcome {
    pub content: String,
    /// True when the tool failed. The text still goes back to the model —
    /// a failed tool is information, not an abort.
    pub is_error: bool,
}

impl ToolOutcome {
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
        }
    }

    pub fn err(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
        }
    }
}

/// Where a tool is allowed to operate.
///
/// Every path a tool touches is resolved through here, so a model cannot walk
/// out of the workspace with `../../..` **or through a symlink**: a link inside
/// the workspace that points outside it is rejected, because the lexical check
/// alone is defeated by one `ln -s`.
#[derive(Clone, Debug)]
pub struct ToolContext {
    root: std::path::PathBuf,
    /// Canonical form of `root`, resolved on first use. `None` until then, and
    /// `None` permanently when the root does not exist.
    real_root: std::sync::OnceLock<Option<std::path::PathBuf>>,
}

impl ToolContext {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            root: root.into(),
            real_root: std::sync::OnceLock::new(),
        }
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// Canonical form of `root`, or `None` when it does not exist yet.
    ///
    /// Cached because it costs a syscall and never changes for a session.
    fn canonical_root(&self) -> Option<&std::path::Path> {
        self.real_root
            .get_or_init(|| self.root.canonicalize().ok())
            .as_deref()
    }

    /// Resolve `path` against the workspace root and reject anything that
    /// escapes it. Relative paths are joined to the root; absolute paths are
    /// allowed only when they are already inside it.
    pub fn resolve(&self, path: &str) -> Result<std::path::PathBuf> {
        use std::path::{Component, Path, PathBuf};

        /// Lexical normalization only, so a path that does not exist yet can
        /// still be checked. Symlinks are handled separately by
        /// [`ToolContext::reject_symlink_escape`].
        fn normalize(path: &Path) -> PathBuf {
            let mut out = PathBuf::new();
            for component in path.components() {
                match component {
                    Component::CurDir => {}
                    Component::ParentDir => {
                        out.pop();
                    }
                    other => out.push(other.as_os_str()),
                }
            }
            out
        }

        let candidate = Path::new(path);
        let joined = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            self.root.join(candidate)
        };
        let normalized = normalize(&joined);
        if !normalized.starts_with(normalize(&self.root)) {
            return Err(AgentError::Tool(format!(
                "path {path:?} is outside the workspace root {}",
                self.root.display()
            )));
        }
        self.reject_symlink_escape(&normalized)?;
        Ok(normalized)
    }

    /// Second line of defence: the lexical check passes for a path that is
    /// spelled inside the workspace but *reaches* outside it through a symlink.
    ///
    /// Containment is judged on the deepest part of the path that exists, since
    /// a write target usually does not exist yet.
    fn reject_symlink_escape(&self, normalized: &std::path::Path) -> Result<()> {
        // A root that does not exist cannot contain a symlink, and comparing a
        // canonical path against a non-existent lexical root would reject every
        // legitimate path. The lexical check above already covers this case.
        let Some(root) = self.canonical_root() else {
            return Ok(());
        };
        let mut probe = normalized;
        let existing = loop {
            if probe.exists() {
                break probe;
            }
            match probe.parent() {
                Some(parent) => probe = parent,
                // Nothing on this path exists, so nothing can redirect it.
                None => return Ok(()),
            }
        };
        let Ok(real) = existing.canonicalize() else {
            // A path we cannot canonicalize is one we cannot vouch for.
            return Err(AgentError::Tool(format!(
                "path {} could not be resolved for containment checks",
                normalized.display()
            )));
        };
        if !real.starts_with(root) {
            return Err(AgentError::Tool(format!(
                "path {} resolves to {}, outside the workspace root {}",
                normalized.display(),
                real.display(),
                root.display()
            )));
        }
        Ok(())
    }
}

/// A capability the runtime can offer the model.
pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    fn invoke(&self, arguments: &serde_json::Value, context: &ToolContext) -> Result<ToolOutcome>;
}

/// The set of tools a session exposes.
#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// The default coding tool set, rooted at `context`.
    pub fn coding() -> Self {
        let mut registry = Self::new();
        registry.add(crate::tools::ReadFile);
        registry.add(crate::tools::WriteFile);
        registry.add(crate::tools::EditFile);
        registry.add(crate::tools::FindFiles);
        registry.add(crate::tools::SearchFiles);
        registry.add(crate::tools::RunCommand);
        registry
    }

    pub fn add(&mut self, tool: impl Tool + 'static) -> &mut Self {
        self.tools.push(Box::new(tool));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.iter().map(|tool| tool.spec()).collect()
    }

    pub fn spec(&self, name: &str) -> Option<ToolSpec> {
        self.tools
            .iter()
            .map(|tool| tool.spec())
            .find(|spec| spec.name == name)
    }

    /// Run one invocation. An unknown tool name is reported to the model as a
    /// failed tool rather than raised, so one bad name does not end the turn.
    pub fn invoke(&self, invocation: &ToolInvocation, context: &ToolContext) -> ToolOutcome {
        let Some(tool) = self
            .tools
            .iter()
            .find(|tool| tool.spec().name == invocation.name)
        else {
            return ToolOutcome::err(format!("no such tool: {}", invocation.name));
        };
        match tool.invoke(&invocation.arguments, context) {
            Ok(outcome) => outcome,
            Err(error) => ToolOutcome::err(error.to_string()),
        }
    }
}

/// Whether a tool call may run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDecision {
    Allow,
    /// Refused, with a reason shown to the model.
    Deny(String),
}

/// The gate every tool call passes through before it executes.
///
/// A non-interactive policy answers immediately; an interactive one (a canvas
/// card waiting for a click) blocks until the user decides. Keeping this a
/// trait is what lets the headless tests and the UI share one session loop.
///
/// `decide` runs on the session's worker thread, so blocking is allowed and
/// expected — that agent's turn is supposed to wait. `cancel` is passed in
/// because a blocking gate needs a way out: without it, cancelling a turn that
/// is sitting on an approval prompt would leave it waiting out the whole
/// timeout.
pub trait PermissionGate: Send + Sync {
    fn decide(&self, invocation: &ToolInvocation, cancel: &CancelToken) -> PermissionDecision;
}

/// Allow every call.
pub struct AllowAll;

impl PermissionGate for AllowAll {
    fn decide(&self, _invocation: &ToolInvocation, _cancel: &CancelToken) -> PermissionDecision {
        PermissionDecision::Allow
    }
}

/// Deny every call.
pub struct DenyAll;

impl PermissionGate for DenyAll {
    fn decide(&self, _invocation: &ToolInvocation, _cancel: &CancelToken) -> PermissionDecision {
        PermissionDecision::Deny("all tool calls are denied by policy".into())
    }
}

/// Allow only the named tools.
pub struct AllowList(pub Vec<String>);

impl PermissionGate for AllowList {
    fn decide(&self, invocation: &ToolInvocation, _cancel: &CancelToken) -> PermissionDecision {
        if self.0.iter().any(|name| name == &invocation.name) {
            PermissionDecision::Allow
        } else {
            PermissionDecision::Deny(format!(
                "tool {:?} is not on the allow list",
                invocation.name
            ))
        }
    }
}
