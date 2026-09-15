//! Chat view over a terminal-hosted agent CLI.

pub mod adapter;
pub mod event;
pub mod fold;

pub use adapter::{ChatMode, CliAdapter};
pub use event::ChatEvent;
pub use fold::{ChatCardState, Row};
