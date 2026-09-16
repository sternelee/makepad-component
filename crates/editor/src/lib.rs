//! `makepad-editor` — the half of an editor that is not a widget.
//!
//! bezel splits its editor the same way and says why: *"`markdown` holds the document, its markdown wire form,
//! and the painting — all of it testable without a window. What lives here is the half that needs one: focus,
//! keys, the platform input handler, the mouse, undo, and the menus."*
//!
//! So this crate is the part of that half which **does not** need a window — undo and redo, and the policy
//! that decides what one step is — with the keyboard and mouse halves to follow in the crate that has the
//! widgets.
//!
//! ## What is here
//!
//! - [`SnapshotHistory`] — the generic stack: a `Vec` of states and a cursor, with **no** policy. Moved here
//!   from `crates/ui`, a widget crate, which is the wrong home for a data structure with no widgets in it;
//!   moving it is what let the document history be written *on top of* it rather than beside it.
//! - [`History`] — the document's undo, and the coalescing rules that decide what counts as one step.
//! - [`EditKind`] — what an edit did, in the four kinds the coalescing rules need.

pub mod history;
pub mod slash;
pub mod snapshot;

pub use history::{DEFAULT_UNDO_LIMIT, EditKind, History, Step};
pub use slash::{item, items, label, query as slash_query, Item, SlashMenu};
pub use snapshot::SnapshotHistory;
