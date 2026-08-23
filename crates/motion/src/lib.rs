//! Named motion catalog.
//!
//! All durations and easing choices flow through these names — never inline
//! magic numbers in widget DSL or Rust event code.
//!
//! Easing vocabulary mapped onto Makepad animator modes:
//!
//! | Intent                    | Animator mode          |
//! |---------------------------|------------------------|
//! | hover / press / fade      | `Forward { duration }` |
//! | instant state switch      | `Snap`                 |
//! | looping shader time       | `Loop { duration, end }` |
//! | spring-like settle (rare) | `Forward { duration }` + `Ease` track |

/// Instant state switch (disabled toggles, pressed enter). Maps to `Snap`.
pub const INSTANT: f64 = 0.0;
/// Fast feedback: hover washes, chevron rotation, small fades.
pub const FAST: f64 = 0.10;
/// Base transition: default state changes.
pub const BASE: f64 = 0.15;
/// Slow reveal: sheets, dialogs, popovers entering.
pub const SLOW: f64 = 0.25;
