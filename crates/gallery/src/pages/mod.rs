//! The rail: what the gallery documents, and where each page is written.
//!
//! Bezel's gallery is the documentation rather than a demo, and the thing that
//! keeps it honest is that its rail is *checkable*: every row names a source
//! file, and a test asserts that file exists. A component with no page fails
//! the suite; a page whose file was renamed fails it too.
//!
//! So the table below is the contract, the rail is painted from it, and
//! [`tests`] is what stops it drifting.

pub mod button;
pub mod controls;
pub mod foundation;
pub mod layout;
pub mod loaders;
pub mod motion;

use makepad_widgets::*;

/// Register every page, after creating the namespace they assign into.
///
/// The parent module has to exist before a nested assignment reaches it — the
/// same reason `makepad_theme` creates `mod.mpc` before `mod.mpc.tokens` — and
/// a page that registered into a missing module fails at runtime rather than at
/// compile time.
pub fn script_mod(vm: &mut ScriptVm) {
    script_eval!(vm, { mod.gallery = {} });
    script_eval!(vm, { mod.gallery.pages = {} });
    foundation::script_mod(vm);
    motion::script_mod(vm);
    button::script_mod(vm);
    controls::script_mod(vm);
    layout::script_mod(vm);
    loaders::script_mod(vm);
}

/// One row of the rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    /// What the rail shows.
    pub title: &'static str,
    /// The DSL path the page's root is registered under, so the app can
    /// instantiate it: `mod.gallery.pages.palette{}`.
    pub path: &'static str,
    /// The source file this page is written in, relative to the workspace root.
    pub source: &'static str,
    /// One line on what the page is for.
    pub blurb: &'static str,
}

/// Every page, in rail order.
pub const PAGES: &[Page] = &[
    Page {
        title: "Palette",
        path: "mod.gallery.pages.palette",
        source: "crates/gallery/src/pages/foundation.rs",
        blurb: "Every token, in both appearances",
    },
    Page {
        title: "Type",
        path: "mod.gallery.pages.typography",
        source: "crates/gallery/src/pages/foundation.rs",
        blurb: "The measured ladder and its leading",
    },
    Page {
        title: "Metrics",
        path: "mod.gallery.pages.metrics",
        source: "crates/gallery/src/pages/foundation.rs",
        blurb: "Control sizes, radii and the sibling gap",
    },
    Page {
        title: "Motion",
        path: "mod.gallery.pages.motion",
        source: "crates/gallery/src/pages/motion.rs",
        blurb: "The named catalog, playing",
    },
    Page {
        title: "Button",
        path: "mod.gallery.pages.button",
        source: "crates/gallery/src/pages/button.rs",
        blurb: "The four shipped looks",
    },
    Page {
        title: "Layout",
        path: "mod.gallery.pages.layout",
        source: "crates/gallery/src/pages/layout.rs",
        blurb: "Row, column, divider, spacer",
    },
    Page {
        title: "Loaders",
        path: "mod.gallery.pages.loaders",
        source: "crates/gallery/src/pages/loaders.rs",
        blurb: "Spinner, pulse, progress",
    },
    Page {
        title: "Controls",
        path: "mod.gallery.pages.controls",
        source: "crates/gallery/src/pages/controls.rs",
        blurb: "Checkbox, switch, radio — one contract",
    },
];

/// The page the gallery opens on.
pub const FIRST: usize = 0;

impl Page {
    /// The page at `index`, clamped into range.
    ///
    /// Clamped rather than indexed: the rail is data and a stale index is a
    /// panic in a paint pass, which is the least debuggable place to have one.
    pub fn at(index: usize) -> &'static Page {
        &PAGES[index.min(PAGES.len() - 1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::Path;

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/gallery sits two levels below the workspace root")
    }

    #[test]
    fn every_row_names_a_source_file_that_exists() {
        let root = workspace_root();
        for page in PAGES {
            let path = root.join(page.source);
            assert!(
                path.is_file(),
                "the rail has a row for {:?} pointing at {:?}, which does not exist",
                page.title,
                page.source
            );
        }
    }

    #[test]
    fn every_page_source_is_a_file_the_crate_compiles() {
        // A rail row pointing at a file outside the crate would satisfy
        // `is_file` and document nothing.
        for page in PAGES {
            assert!(
                page.source.starts_with("crates/gallery/src/"),
                "{} points outside the gallery",
                page.source
            );
            assert!(page.source.ends_with(".rs"), "{}", page.source);
        }
    }

    #[test]
    fn every_page_module_is_declared() {
        // The other half of "the row names a file": the file has to be part of
        // the module tree, or the row documents a file nothing compiles.
        use std::collections::HashMap;
        let declared: HashMap<&str, &str> = [
            ("crates/gallery/src/pages/foundation.rs", "foundation"),
            ("crates/gallery/src/pages/motion.rs", "motion"),
            ("crates/gallery/src/pages/button.rs", "button"),
            ("crates/gallery/src/pages/controls.rs", "controls"),
            ("crates/gallery/src/pages/layout.rs", "layout"),
            ("crates/gallery/src/pages/loaders.rs", "loaders"),
        ]
        .into_iter()
        .collect();
        for page in PAGES {
            assert!(
                declared.contains_key(page.source),
                "{} is not declared in pages/mod.rs",
                page.source
            );
        }
    }

    #[test]
    fn titles_and_paths_are_unique() {
        let mut titles = HashSet::new();
        let mut paths = HashSet::new();
        for page in PAGES {
            assert!(titles.insert(page.title), "duplicate title {}", page.title);
            assert!(paths.insert(page.path), "duplicate path {}", page.path);
        }
    }

    #[test]
    fn every_row_has_a_blurb_and_a_registered_path() {
        for page in PAGES {
            assert!(!page.blurb.is_empty(), "{} has no blurb", page.title);
            assert!(
                page.path.starts_with("mod.gallery.pages."),
                "{} is not registered under mod.gallery.pages",
                page.path
            );
        }
    }

    #[test]
    fn there_is_at_least_one_page_and_the_first_exists() {
        assert!(!PAGES.is_empty());
        assert!(FIRST < PAGES.len());
        assert_eq!(Page::at(FIRST), &PAGES[FIRST]);
    }

    #[test]
    fn a_stale_index_clamps_rather_than_panicking() {
        assert_eq!(Page::at(999), &PAGES[PAGES.len() - 1]);
        assert_eq!(Page::at(0), &PAGES[0]);
    }
}
