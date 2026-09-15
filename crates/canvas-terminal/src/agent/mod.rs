//! Agent cards on the canvas: the client that talks to the daemon and the
//! card-side state the canvas renders.

mod card;
mod client;

pub use card::{AgentCardState, Row};
pub use client::AgentClient;
