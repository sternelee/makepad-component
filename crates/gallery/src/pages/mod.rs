//! The rail: what the gallery documents, and where each page is written.
//!
//! Bezel's gallery is the documentation rather than a demo, and the thing that
//! keeps it honest is that its rail is *checkable*: every row names a source
//! file, and a test asserts that file exists. A component with no page fails
//! the suite; a page whose file was renamed fails it too.
//!
//! So the table below is the contract, the rail is painted from it, and
//! [`tests`] is what stops it drifting.

pub mod avatar;
pub mod bars;
pub mod button;
pub mod calendar;
pub mod content;
pub mod controls;
pub mod document;
pub mod editor;
pub mod feedback;
pub mod foundation;
pub mod floating;
pub mod history;
pub mod hover_card;
pub mod icon;
pub mod input;
pub mod layout;
pub mod list;
pub mod loaders;
pub mod scroll;
pub mod search;
pub mod select;
pub mod shortcuts;
pub mod slider;
pub mod status;
pub mod surface;
pub mod table;
pub mod canvas;
pub mod code;
pub mod combobox;
pub mod tree;
pub mod menu;
pub mod motion;
pub mod overlay;
pub mod pagination;
pub mod palette;
pub mod popover;

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
    bars::script_mod(vm);
    button::script_mod(vm);
    calendar::script_mod(vm);
    controls::script_mod(vm);
    layout::script_mod(vm);
    loaders::script_mod(vm);
    shortcuts::script_mod(vm);
    slider::script_mod(vm);
    input::script_mod(vm);
    overlay::script_mod(vm);
    popover::script_mod(vm);
    floating::script_mod(vm);
    history::script_mod(vm);
    hover_card::script_mod(vm);
    document::script_mod(vm);
    editor::script_mod(vm);
    icon::script_mod(vm);
    status::script_mod(vm);
    table::script_mod(vm);
    canvas::script_mod(vm);
    code::script_mod(vm);
    combobox::script_mod(vm);
    tree::script_mod(vm);
    avatar::script_mod(vm);
    surface::script_mod(vm);
    list::script_mod(vm);
    select::script_mod(vm);
    feedback::script_mod(vm);
    content::script_mod(vm);
    pagination::script_mod(vm);
    palette::script_mod(vm);
    menu::script_mod(vm);
    scroll::script_mod(vm);
    search::script_mod(vm);
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
        title: "Slider",
        path: "mod.gallery.pages.slider",
        source: "crates/gallery/src/pages/slider.rs",
        blurb: "Grab-anywhere drag, steps, arrows",
    },
    Page {
        title: "Overlay",
        path: "mod.gallery.pages.overlay",
        source: "crates/gallery/src/pages/overlay.rs",
        blurb: "A tooltip, and the pass that puts it on top",
    },
    Page {
        title: "Input",
        path: "mod.gallery.pages.input",
        source: "crates/gallery/src/pages/input.rs",
        blurb: "The text field and its states",
    },
    Page {
        title: "Controls",
        path: "mod.gallery.pages.controls",
        source: "crates/gallery/src/pages/controls.rs",
        blurb: "Checkbox, switch, radio — one contract",
    },
    Page {
        title: "Popover",
        path: "mod.gallery.pages.popover",
        source: "crates/gallery/src/pages/popover.rs",
        blurb: "A floating panel, opened by a click",
    },
    Page {
        title: "Icon",
        path: "mod.gallery.pages.icon",
        source: "crates/gallery/src/pages/icon.rs",
        blurb: "The bundled glyph face on the size ladder",
    },
    Page {
        title: "Status",
        path: "mod.gallery.pages.status",
        source: "crates/gallery/src/pages/status.rs",
        blurb: "Badges and tags, six tones",
    },
    Page {
        title: "Table",
        path: "mod.gallery.pages.table",
        source: "crates/gallery/src/pages/table.rs",
        blurb: "Columns, rows, and row selection",
    },
    Page {
        title: "Tree",
        path: "mod.gallery.pages.tree",
        source: "crates/gallery/src/pages/tree.rs",
        blurb: "A flat list with a hierarchy",
    },
    Page {
        title: "Avatar",
        path: "mod.gallery.pages.avatar",
        source: "crates/gallery/src/pages/avatar.rs",
        blurb: "Initials, presence, and the group",
    },
    Page {
        title: "Surface",
        path: "mod.gallery.pages.surface",
        source: "crates/gallery/src/pages/surface.rs",
        blurb: "The planes, the radii, and the glass",
    },
    Page {
        title: "List",
        path: "mod.gallery.pages.list",
        source: "crates/gallery/src/pages/list.rs",
        blurb: "Rows of things, three shapes",
    },
    Page {
        title: "Select",
        path: "mod.gallery.pages.select",
        source: "crates/gallery/src/pages/select.rs",
        blurb: "A face, a panel and the wiring between",
    },
    Page {
        title: "Feedback",
        path: "mod.gallery.pages.feedback",
        source: "crates/gallery/src/pages/feedback.rs",
        blurb: "The ring and the skeleton",
    },
    Page {
        title: "Content",
        path: "mod.gallery.pages.content",
        source: "crates/gallery/src/pages/content.rs",
        blurb: "Group boxes and key caps",
    },
    Page {
        title: "Pagination",
        path: "mod.gallery.pages.pagination",
        source: "crates/gallery/src/pages/pagination.rs",
        blurb: "One function, four cases",
    },
    Page {
        title: "Menu",
        path: "mod.gallery.pages.menu",
        source: "crates/gallery/src/pages/menu.rs",
        blurb: "Commands, grouped, with shortcuts",
    },
    Page {
        title: "Scroll",
        path: "mod.gallery.pages.scroll",
        source: "crates/gallery/src/pages/scroll.rs",
        blurb: "The chrome every page already used",
    },
    Page {
        title: "Search",
        path: "mod.gallery.pages.search",
        source: "crates/gallery/src/pages/search.rs",
        blurb: "The match behind a palette",
    },
    Page {
        title: "Bars",
        path: "mod.gallery.pages.bars",
        source: "crates/gallery/src/pages/bars.rs",
        blurb: "Titlebar, control bar, menubar",
    },
    Page {
        // "Command Palette", not "Palette": the colour palette page already owns
        // that title, and the uniqueness test above caught the collision on the
        // first run. A rail with two rows reading "Palette" would have opened
        // whichever came first and looked like the other one was broken.
        title: "Command Palette",
        path: "mod.gallery.pages.command_palette",
        source: "crates/gallery/src/pages/palette.rs",
        blurb: "The query, the list, and the cursor",
    },
    Page {
        title: "Calendar",
        path: "mod.gallery.pages.calendar",
        source: "crates/gallery/src/pages/calendar.rs",
        blurb: "The month grid and its arithmetic",
    },
    Page {
        title: "Shortcuts",
        path: "mod.gallery.pages.shortcuts",
        source: "crates/gallery/src/pages/shortcuts.rs",
        blurb: "Chords resolved from a keymap",
    },
    Page {
        title: "History",
        path: "mod.gallery.pages.history",
        source: "crates/gallery/src/pages/history.rs",
        blurb: "Undo and redo, as a stack",
    },
    Page {
        title: "Combobox",
        path: "mod.gallery.pages.combobox",
        source: "crates/gallery/src/pages/combobox.rs",
        blurb: "A field over a list, and the choice it holds",
    },
    Page {
        title: "Hover Card",
        path: "mod.gallery.pages.hover_card",
        source: "crates/gallery/src/pages/hover_card.rs",
        blurb: "A card on hover, and its timing",
    },
    Page {
        title: "Floating Panel",
        path: "mod.gallery.pages.floating",
        source: "crates/gallery/src/pages/floating.rs",
        blurb: "A panel the reader drags",
    },
    Page {
        title: "Code",
        path: "mod.gallery.pages.code",
        source: "crates/gallery/src/pages/code.rs",
        blurb: "Source classified, coloured and painted",
    },
    Page {
        title: "Document",
        path: "mod.gallery.pages.document",
        source: "crates/gallery/src/pages/document.rs",
        blurb: "A markdown document, laid out and painted",
    },
    Page {
        title: "Editor",
        path: "mod.gallery.pages.editor",
        source: "crates/gallery/src/pages/editor.rs",
        blurb: "Keys, a caret, a selection and undo",
    },
    Page {
        title: "Canvas",
        path: "mod.gallery.pages.canvas",
        source: "crates/gallery/src/pages/canvas.rs",
        blurb: "A JSON Canvas document, painted",
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
            ("crates/gallery/src/pages/bars.rs", "bars"),
            ("crates/gallery/src/pages/button.rs", "button"),
            ("crates/gallery/src/pages/calendar.rs", "calendar"),
            ("crates/gallery/src/pages/controls.rs", "controls"),
            ("crates/gallery/src/pages/layout.rs", "layout"),
            ("crates/gallery/src/pages/loaders.rs", "loaders"),
            ("crates/gallery/src/pages/shortcuts.rs", "shortcuts"),
            ("crates/gallery/src/pages/slider.rs", "slider"),
            ("crates/gallery/src/pages/input.rs", "input"),
            ("crates/gallery/src/pages/overlay.rs", "overlay"),
            ("crates/gallery/src/pages/popover.rs", "popover"),
            ("crates/gallery/src/pages/floating.rs", "floating"),
            ("crates/gallery/src/pages/history.rs", "history"),
            ("crates/gallery/src/pages/hover_card.rs", "hover_card"),
            ("crates/gallery/src/pages/document.rs", "document"),
            ("crates/gallery/src/pages/editor.rs", "editor"),
            ("crates/gallery/src/pages/icon.rs", "icon"),
            ("crates/gallery/src/pages/status.rs", "status"),
            ("crates/gallery/src/pages/table.rs", "table"),
            ("crates/gallery/src/pages/canvas.rs", "canvas"),
            ("crates/gallery/src/pages/code.rs", "code"),
            ("crates/gallery/src/pages/combobox.rs", "combobox"),
            ("crates/gallery/src/pages/tree.rs", "tree"),
            ("crates/gallery/src/pages/avatar.rs", "avatar"),
            ("crates/gallery/src/pages/surface.rs", "surface"),
            ("crates/gallery/src/pages/list.rs", "list"),
            ("crates/gallery/src/pages/select.rs", "select"),
            ("crates/gallery/src/pages/feedback.rs", "feedback"),
            ("crates/gallery/src/pages/content.rs", "content"),
            ("crates/gallery/src/pages/pagination.rs", "pagination"),
            ("crates/gallery/src/pages/palette.rs", "palette"),
            ("crates/gallery/src/pages/menu.rs", "menu"),
            ("crates/gallery/src/pages/scroll.rs", "scroll"),
            ("crates/gallery/src/pages/search.rs", "search"),
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
