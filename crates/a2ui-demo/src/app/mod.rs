//! A2UI Demo Application
//!
//! Demonstrates the A2UI protocol rendering with:
//! - Static mode: Load product catalog JSON data directly
//! - Streaming mode: Connect to A2A server for payment checkout UI

pub mod audio_player;
mod logic;
mod sample_data;
mod theme;

pub use logic::*;
pub use theme::*;
