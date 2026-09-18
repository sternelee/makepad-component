//! A2UI Protocol Implementation for Makepad
//!
//! A2UI (Agent-to-UI) is a declarative JSON protocol for AI agents to generate
//! rich, interactive UIs. This module implements the A2UI renderer for Makepad.
//!
//! # Architecture
//!
//! ```text
//! A2UI JSON Messages
//!        ↓
//! A2uiMessageProcessor
//!        ↓
//! ┌──────┴──────┐
//! │             │
//! DataModel  ComponentTree
//!    │             │
//!    └──────┬──────┘
//!           ↓
//!    A2uiSurface (Widget)
//!           ↓
//!    Makepad Native Widgets
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use makepad_component::a2ui::*;
//!
//! // Parse A2UI JSON message
//! let json = r#"{"beginRendering": {"surfaceId": "main", "root": "root"}}"#;
//! let message: A2uiMessage = serde_json::from_str(json)?;
//!
//! // Process message
//! processor.process_message(message);
//! ```

// **The three that need a socket, and the threads that read it.** Everything else in this module is the protocol
// and its renderer, which are arithmetic and need nothing from the host at all — so gating these three is what
// lets the whole renderer, chart bridge included, build for a browser.
//
// **Both conditions, and the second is not redundant.** `net` is the caller's switch; `not(target_arch = "wasm32")`
// is the platform's, because the socket dependencies are not declared for the web at all (see `Cargo.toml`). A
// caller who leaves `net` on — which is the default — still gets a clean wasm build, which is the whole point:
// the platform decides, so no consumer has to know.
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
mod a2a_client;
pub mod chart_bridge;
mod data_model;
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
mod host;
mod message;
mod processor;
mod registry;
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
mod sse;
mod surface;
mod value;

#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
pub use a2a_client::*;
pub use data_model::*;
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
pub use host::*;
pub use message::*;
pub use processor::*;
pub use registry::*;
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
pub use sse::*;
pub use surface::*;
pub use value::*;

use makepad_widgets::*;

/// Initialize A2UI script components
pub fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
    crate::a2ui::surface::script_mod(vm)
}
