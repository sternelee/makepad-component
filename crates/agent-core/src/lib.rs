//! Headless agent runtime.
//!
//! The runtime knows nothing about Makepad or the canvas: it is a provider
//! driver, a tool loop, and a replayable event journal, so it can be exercised
//! from a test, hosted in the terminal daemon, or driven from a canvas card
//! without changing.
//!
//! ```text
//!   AgentSession ──uses──> Provider (OpenAI-compatible, ...)
//!        │                     ▲
//!        │                     └── CancelToken   (cancel reaches inside a call)
//!        ├──uses──> ToolRegistry (read/write/edit/find/search/run)
//!        ├──uses──> PermissionGate (policy now, canvas approval later)
//!        └──emits─> Sequenced<AgentEvent>  +  journal + conversation history
//!              └──> ResumeCursor (session_id, epoch, seq) for reconnect
//! ```
//!
//! Contracts the rest of the system depends on:
//!
//! * Dropping a session never blocks; `cancel` lands between provider stream
//!   chunks, not at the next model call.
//! * `history()` is complete and `is_busy()` is false by the time
//!   `TurnFinished` is published.
//! * A prompt is either a new turn or refused when the queue is full; a steer
//!   only ever reaches the turn it was written for.
//! * `ResumeCursor` is `(session_id, epoch, seq)`, so a cursor from a previous
//!   process is recognised as stale instead of matched against a reused id.
//!
//! ```
//! use agent_core::{AgentSessionConfig, Message};
//!
//! let config = AgentSessionConfig::new("/tmp/workspace");
//! assert_eq!(config.max_tool_iterations, 24);
//! assert!(!config.system_prompt.is_empty());
//! assert_eq!(Message::user("hi").role.as_str(), "user");
//! ```

pub mod cancel;
pub mod error;
pub mod event;
pub mod identity;
pub mod message;
pub mod provider;
pub mod session;
pub mod tool;
pub mod tools;

pub use cancel::CancelToken;
pub use error::{AgentError, Result};
pub use event::{AgentEvent, Sequenced};
pub use identity::{process_epoch, Replay, ResumeCursor, SessionId};
pub use message::{Message, Role, ToolCall};
pub use provider::openai::OpenAiConfig;
pub use provider::scripted::{ScriptedProvider, ScriptedTurn};
pub use provider::{
    CompletionRequest, CompletionResponse, OpenAiProvider, Provider, ProviderEvent, Usage,
};
pub use session::{AgentSession, AgentSessionConfig, SessionStream, DEFAULT_SYSTEM_PROMPT};
pub use tool::{
    AllowAll, AllowList, DenyAll, PermissionDecision, PermissionGate, Tool, ToolContext,
    ToolInvocation, ToolOutcome, ToolRegistry, ToolSpec,
};
