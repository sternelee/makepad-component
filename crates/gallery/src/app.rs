//! The gallery: a rail of every page, and the page it selects.
//!
//! The app is deliberately thin. Everything it shows is a page module under
//! `pages/`, and the rail is painted from [`pages::PAGES`] — so adding a
//! component means adding a page file and a row, and a test fails if the row
//! names a file that is not there.
//!
//! The one piece of real machinery here is the theme: [`App::handle_startup`]
//! installs it, and the rail's appearance switch replaces it. Nothing else in
//! the app knows a colour.

use makepad_component::mp::table::TableColumn;
use makepad_widgets::*;

use makepad_component::mp::{
    button::{MpButtonStyle, MpButtonWidgetRefExt},
    checkbox::MpCheckboxWidgetRefExt,
    radio::MpRadioWidgetRefExt,
    feedback::MpProgressRingWidgetRefExt,
    loaders::MpProgressWidgetRefExt,
    popover::MpPopoverWidgetRefExt,
    pagination::MpPaginationWidgetRefExt,
    scaffolding::MpKbdWidgetRefExt,
    status::MpBadgeWidgetRefExt,
    list::{ListItem, MpListWidgetRefExt},
    table::MpTableWidgetRefExt,
    avatar::MpAvatarWidgetRefExt,
    avatar_group::MpAvatarGroupWidgetRefExt,
    tree::{MpTreeWidgetRefExt, TreeItem},
    tooltip::MpTooltipWidgetRefExt,
    slider::MpSliderWidgetRefExt,
    segmented::MpSegmentedWidgetRefExt,
    switch::MpSwitchWidgetRefExt,
    floating::MpFloatingWidgetRefExt,
    code::MpCodeBlockWidgetRefExt,
    markdown::MpMarkdownWidgetRefExt,
    editor::MpEditorWidgetRefExt,
    canvas::MpCanvasWidgetRefExt,
    date::MpDateWidgetRefExt,
};

use crate::pages::PAGES;
use makepad_component::mp::combobox::Combobox;
use makepad_editor::SnapshotHistory;
use makepad_component::mp::hover_card::HoverIntent;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    // A rail row: the page's title, with the selected one carrying the
    // selected wash. A row is a button in behaviour and a row in look, so it is
    // built from the button rather than restating its hit handling.
    let RailRow = mod.mp.MpButton{
        width: Fill
        height: mod.mpc.layout.control.regular.height
        style: mod.mp.ButtonStyle.Ghost
        align: Align{x: 0.0, y: 0.5}
        text: "Page"
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "bezel — makepad components"
                window.inner_size: vec2(1180, 860)

                show_bg: true
                draw_bg +: {color: instance(bg)}

                body := View{
                    width: Fill
                    height: Fill
                    flow: Right

                    // ---- the rail ----
                    mod.mp.SurfacePanel{
                        width: 248
                        height: Fill
                        flow: Down
                        spacing: 4
                        padding: Inset{left: 12, right: 12, top: 16, bottom: 16}

                        rail_title := Label{
                            width: Fill, height: Fit
                            draw_text +: {text_style: title3, color: text}
                            text: "bezel"
                        }
                        rail_sub := Label{
                            width: Fill, height: Fit
                            draw_text +: {text_style: caption, color: text_faint}
                            text: "makepad components"
                        }

                        rail_gap := View{width: Fill, height: 12}

                        rail_page_0 := RailRow{text: ""}
                        rail_page_1 := RailRow{text: ""}
                        rail_page_2 := RailRow{text: ""}
                        rail_page_3 := RailRow{text: ""}
                        rail_page_4 := RailRow{text: ""}
                        rail_page_5 := RailRow{text: ""}
                        rail_page_6 := RailRow{text: ""}
                        rail_page_7 := RailRow{text: ""}
                        rail_page_8 := RailRow{text: ""}
                        rail_page_9 := RailRow{text: ""}
                        rail_page_10 := RailRow{text: ""}
                        rail_page_11 := RailRow{text: ""}
                        rail_page_12 := RailRow{text: ""}
                        rail_page_13 := RailRow{text: ""}
                        rail_page_14 := RailRow{text: ""}
                        rail_page_15 := RailRow{text: ""}
                        rail_page_16 := RailRow{text: ""}
                        rail_page_17 := RailRow{text: ""}
                        rail_page_18 := RailRow{text: ""}
                        rail_page_19 := RailRow{text: ""}
                        rail_page_20 := RailRow{text: ""}
                        rail_page_21 := RailRow{text: ""}
                        rail_page_22 := RailRow{text: ""}
                        rail_page_23 := RailRow{text: ""}
                        rail_page_24 := RailRow{text: ""}
                        rail_page_25 := RailRow{text: ""}
                        rail_page_26 := RailRow{text: ""}
                        rail_page_27 := RailRow{text: ""}
                        rail_page_28 := RailRow{text: ""}
                        rail_page_29 := RailRow{text: ""}
                        rail_page_30 := RailRow{text: ""}
                        rail_page_31 := RailRow{text: ""}
                        rail_page_32 := RailRow{text: ""}
                        rail_page_33 := RailRow{text: ""}
                        rail_page_34 := RailRow{text: ""}
                        rail_page_35 := RailRow{text: ""}
                        rail_page_36 := RailRow{text: ""}
                        rail_page_37 := RailRow{text: ""}
                        rail_page_38 := RailRow{text: ""}
                        rail_page_39 := RailRow{text: ""}
                        rail_page_40 := RailRow{text: ""}
                        rail_page_41 := RailRow{text: ""}
                        rail_page_42 := RailRow{text: ""}
                        rail_page_43 := RailRow{text: ""}
                        rail_page_44 := RailRow{text: ""}
                        rail_page_45 := RailRow{text: ""}

                        rail_filler := View{width: Fill, height: Fill}

                        rail_blurb := Label{
                            width: Fill, height: Fit
                            draw_text +: {text_style: caption, color: text_faint}
                            text: ""
                        }
                        rail_flex := View{width: Fill, height: 8}
                        appearance_toggle := mod.mp.MpButton{
                            width: Fill
                            style: mod.mp.ButtonStyle.Default
                            text: "Appearance"
                        }
                    }

                    // ---- the page ----
                    mod.mp.SurfacePage{
                        width: Fill
                        height: Fill
                        flow: Down
                        padding: Inset{left: 28, right: 28, top: 24, bottom: 24}

                        // The library's scroll, not Makepad's bare one: this is
                        // what every page scrolls inside, and a bare
                        // `ScrollYView` would paint its handle from Makepad's own
                        // theme — the one piece of chrome on every page that was
                        // not designed here.
                        mod.mp.MpScroll{
                            width: Fill
                            height: Fill
                            flow: Down
                            spacing: 0

                            page_0 := mod.gallery.pages.palette{}
                            page_1 := mod.gallery.pages.typography{}
                            page_2 := mod.gallery.pages.metrics{}
                            page_3 := mod.gallery.pages.motion{}
                            page_4 := mod.gallery.pages.button{}
                            page_5 := mod.gallery.pages.layout{}
                            page_6 := mod.gallery.pages.loaders{}
                            page_7 := mod.gallery.pages.slider{}
                            page_8 := mod.gallery.pages.overlay{}
                            page_9 := mod.gallery.pages.input{}
                            page_10 := mod.gallery.pages.controls{}
                            page_11 := mod.gallery.pages.popover{}
                            page_12 := mod.gallery.pages.icon{}
                            page_13 := mod.gallery.pages.status{}
                            page_14 := mod.gallery.pages.table{}
                            page_15 := mod.gallery.pages.tree{}
                            page_16 := mod.gallery.pages.avatar{}
                            page_17 := mod.gallery.pages.surface{}
                            page_18 := mod.gallery.pages.list{}
                            page_19 := mod.gallery.pages.select{}
                            page_20 := mod.gallery.pages.feedback{}
                            page_21 := mod.gallery.pages.content{}
                            page_22 := mod.gallery.pages.pagination{}
                            page_23 := mod.gallery.pages.menu{}
                            page_24 := mod.gallery.pages.scroll{}
                            page_25 := mod.gallery.pages.search{}
                            page_26 := mod.gallery.pages.bars{}
                            page_27 := mod.gallery.pages.command_palette{}
                            page_28 := mod.gallery.pages.calendar{}
                            page_29 := mod.gallery.pages.shortcuts{}
                            page_30 := mod.gallery.pages.history{}
                            page_31 := mod.gallery.pages.combobox{}
                            page_32 := mod.gallery.pages.hover_card{}
                            page_33 := mod.gallery.pages.floating{}
                            page_34 := mod.gallery.pages.code{}
                            page_35 := mod.gallery.pages.document{}
                            page_36 := mod.gallery.pages.editor{}
                            page_37 := mod.gallery.pages.canvas{}
                            page_38 := mod.gallery.pages.blocks{}
                            page_39 := mod.gallery.pages.details{}
                            page_40 := mod.gallery.pages.steps{}
                            page_41 := mod.gallery.pages.numbers{}
                            page_42 := mod.gallery.pages.searching{}
                            page_43 := mod.gallery.pages.picking{}
                            page_44 := mod.gallery.pages.menu_cards{}
                            page_45 := mod.gallery.pages.titlebars{}
                        }
                    }
                }
            }
        }
    }
}

/// The gallery's DSL path for each page slot.
///
/// A table rather than five `ids!` at each use site: the rail, the visibility
/// pass and the `Page::path` strings all have to agree, and a table can be
/// asserted against.
const PAGE_SLOTS: [&[LiveId]; 46] = [
    ids!(page_0),
    ids!(page_1),
    ids!(page_2),
    ids!(page_3),
    ids!(page_4),
    ids!(page_5),
    ids!(page_6),
    ids!(page_7),
    ids!(page_8),
    ids!(page_9),
    ids!(page_10),
    ids!(page_11),
    ids!(page_12),
    ids!(page_13),
    ids!(page_14),
    ids!(page_15),
    ids!(page_16),
    ids!(page_17),
    ids!(page_18),
    ids!(page_19),
    ids!(page_20),
    ids!(page_21),
    ids!(page_22),
    ids!(page_23),
    ids!(page_24),
    ids!(page_25),
    ids!(page_26),
    ids!(page_27),
    ids!(page_28),
    ids!(page_29),
    ids!(page_30),
    ids!(page_31),
    ids!(page_32),
    ids!(page_33),
    ids!(page_34),
    ids!(page_35),
    ids!(page_36),
    ids!(page_37),
    ids!(page_38),
    ids!(page_39),
    ids!(page_40),
    ids!(page_41),
    ids!(page_42),
    ids!(page_43),
    ids!(page_44),
    ids!(page_45),
];

/// The gallery's DSL path for each rail row.
const RAIL_ROWS: [&[LiveId]; 46] = [
    ids!(rail_page_0),
    ids!(rail_page_1),
    ids!(rail_page_2),
    ids!(rail_page_3),
    ids!(rail_page_4),
    ids!(rail_page_5),
    ids!(rail_page_6),
    ids!(rail_page_7),
    ids!(rail_page_8),
    ids!(rail_page_9),
    ids!(rail_page_10),
    ids!(rail_page_11),
    ids!(rail_page_12),
    ids!(rail_page_13),
    ids!(rail_page_14),
    ids!(rail_page_15),
    ids!(rail_page_16),
    ids!(rail_page_17),
    ids!(rail_page_18),
    ids!(rail_page_19),
    ids!(rail_page_20),
    ids!(rail_page_21),
    ids!(rail_page_22),
    ids!(rail_page_23),
    ids!(rail_page_24),
    ids!(rail_page_25),
    ids!(rail_page_26),
    ids!(rail_page_27),
    ids!(rail_page_28),
    ids!(rail_page_29),
    ids!(rail_page_30),
    ids!(rail_page_31),
    ids!(rail_page_32),
    ids!(rail_page_33),
    ids!(rail_page_34),
    ids!(rail_page_35),
    ids!(rail_page_36),
    ids!(rail_page_37),
    ids!(rail_page_38),
    ids!(rail_page_39),
    ids!(rail_page_40),
    ids!(rail_page_41),
    ids!(rail_page_42),
    ids!(rail_page_43),
    ids!(rail_page_44),
    ids!(rail_page_45),
];

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    /// Which page the rail has selected.
    #[rust]
    page: usize,
    /// How many times the button page's button has been pressed, so the page
    /// proves the action path rather than only painting.
    #[rust]
    clicks: usize,
    /// Whether to open the first popover on the first laid-out event.
    #[rust]
    want_popover: bool,
    // The undo/redo stack the History page shows.
    //
    // A `//` comment and a `use` alias, not a `///` and a full path: the script
    // derive's field parser rejects doc comments on fields and full paths in types,
    // which is a trap this port has recorded and has now hit twice.
    #[rust]
    history: SnapshotHistory<String>,
    // The combobox the Combobox page shows: the field's text and the item it names.
    //
    // Short name and a `use` alias, not a path: this is the third time this port has hit
    // the script derive's rejection of full paths in a field type — twice in this one
    // session, the second time immediately after fixing the first. The convention it
    // implies is worth stating plainly: **a `#[derive(Script)]` struct's fields are types
    // that must already be in scope by name, and doc comments are not allowed on them at
    // all.**
    #[rust]
    combobox: Combobox,
    /// The palette's command list, in its **original** order.
    ///
    /// The indices `rank` returns are into this, and they are what a selection
    /// reports — never a position in the filtered view.
    #[rust]
    palette_commands: Vec<String>,
    /// The palette's current ranked view, as original indices.
    #[rust]
    palette_ranked: Vec<usize>,
    /// The cursor, as an **original** index rather than a position.
    ///
    /// This is the whole point of `mp/palette.rs`: a cursor held as a position is
    /// correct until the first query that filters its row out, at which point it
    /// has silently moved onto a different command.
    #[rust]
    palette_active: Option<usize>,
    /// The cursor position this app last set **programmatically**.
    ///
    /// Needed because `MpList::select` emits the same `Selected` action a click
    /// does, so without this the app reads its own highlight back as a selection.
    #[rust]
    palette_highlight: Option<usize>,
    /// Whether the pages' data has been installed yet.
    ///
    /// **Not in `handle_startup`.** `Theme::install` calls
    /// `request_script_reapply()`, and the re-apply that follows re-asserts every
    /// widget's DSL and **wipes any `#[live]` field a Rust setter has written**.
    /// Seeding in startup therefore looked like it worked and left every value at
    /// its declared default — a ring at 0, a slider at whatever its DSL said —
    /// with no error anywhere. Doing it on the first event puts it after the
    /// apply has settled.
    #[rust]
    seeded: bool,
    /// Whether to pin the overlay open on the first event.
    ///
    /// A flag rather than a call in `handle_startup`, because startup runs
    /// *before* the tree is laid out: `widget(...).area()` is unlaid and
    /// `rect()` is empty there, so anchoring to it puts the plate at the origin
    /// and it is never seen. The first `handle_event` is after layout, which is
    /// the earliest a caller can anchor anything to a widget.
    #[rust]
    want_tooltip: bool,
}

app_main!(App);

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Install the theme before the first paint, so nothing reads the
        // fallback palette and then corrects a frame later.
        makepad_theme::Theme::install(makepad_theme::Theme::dark(), cx);
        for (index, page) in PAGES.iter().enumerate() {
            self.ui
                .mp_button(cx, RAIL_ROWS[index])
                .set_text(cx, page.title);
        }
        self.show(cx, self.opening_page());
        // Seed the slider readouts from the values the pages were built with,
        // so the page shows the value path working before anyone touches it —
        // and so a slider whose value never reaches its readout is visible in a
        // screenshot rather than only after a drag.

        // `GALLERY_TOOLTIP=1` pins the overlay open, anchored to the first
        // trigger. Same justification as `GALLERY_PAGE`: Makepad exposes no
        // accessibility tree, so a capture script cannot hover a button, and an
        // overlay that can only be shown by a pointer cannot be verified from a
        // script. It is also the only way to see the *hardest* property this
        // page exists for — that the plate draws over the card below it.
        self.want_tooltip = std::env::var("GALLERY_TOOLTIP").is_ok();
        self.want_popover = std::env::var("GALLERY_POPOVER").is_ok();
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (index, row) in RAIL_ROWS.iter().enumerate() {
            if self.ui.mp_button(cx, row).clicked(actions) {
                self.show(cx, index);
                return;
            }
        }

        if self
            .ui
            .mp_button(cx, ids!(appearance_toggle))
            .clicked(actions)
        {
            let appearance = makepad_theme::Appearance::current(cx);
            appearance.other().set(cx);
            return;
        }

        if self.ui.mp_button(cx, ids!(click_me)).clicked(actions) {
            self.clicks += 1;
            self.ui.label(cx, ids!(click_count)).set_text(
                cx,
                &format!(
                    "{} click{}",
                    self.clicks,
                    if self.clicks == 1 { "" } else { "s" }
                ),
            );
        }

        self.handle_hover_tooltips(cx, actions);
        self.handle_popovers(cx, actions);
        self.handle_controls(cx, actions);
        self.handle_sliders(cx, actions);
        self.handle_segmented(cx, actions);
        self.handle_date(cx, actions);
        self.handle_history(cx, actions);
        self.handle_combobox(cx, actions);
        self.handle_palette(cx, actions);
        self.handle_input(cx, actions);
    }
}

impl App {
    /// Which page to open on.
    ///
    /// `GALLERY_PAGE=<index or title>` pins it. The gallery is the thing the
    /// screenshot workflow points at, and Makepad does not expose its widgets
    /// to the accessibility tree — so a capture script has no way to click a
    /// rail row, and an app that can only be driven by a pointer cannot be
    /// verified in a script. Naming the page is the whole affordance.
    fn opening_page(&self) -> usize {
        let Some(want) = std::env::var("GALLERY_PAGE").ok() else {
            return crate::pages::FIRST;
        };
        if let Ok(index) = want.parse::<usize>() {
            return index;
        }
        PAGES
            .iter()
            .position(|p| p.title.eq_ignore_ascii_case(&want))
            .unwrap_or(crate::pages::FIRST)
    }

    /// The controls page's readouts, and the radio group it demonstrates.
    ///
    /// The group is done *here* rather than in the widget on purpose: a radio
    /// owns one value, the caller owns which one is chosen, and this is what
    /// that contract looks like at a call site — three lines and no registry.
    fn handle_controls(&mut self, cx: &mut Cx, actions: &Actions) {
        if let Some(checked) = self.ui.mp_checkbox(cx, ids!(checkbox_one)).checked(actions) {
            self.ui.label(cx, ids!(checkbox_readout)).set_text(
                cx,
                if checked { "checked" } else { "unchecked" },
            );
        }

        if let Some(on) = self.ui.mp_switch(cx, ids!(switch_one)).checked(actions) {
            self.ui
                .label(cx, ids!(switch_readout))
                .set_text(cx, if on { "on" } else { "off" });
        }

        let radios: [&[LiveId]; 4] = [
            ids!(radio_0),
            ids!(radio_1),
            ids!(radio_2),
            ids!(radio_3),
        ];
        let labels = ["Every day", "Weekly", "Never"];
        if let Some(picked) = radios
            .iter()
            .position(|path| self.ui.mp_radio(cx, *path).selected(actions))
        {
            // Clear the others first, then confirm the one that was chosen —
            // this is the whole "group" implementation.
            for path in radios.iter() {
                self.ui.mp_radio(cx, *path).set_selected(cx, false);
            }
            self.ui.mp_radio(cx, radios[picked]).set_selected(cx, true);
            self.ui
                .label(cx, ids!(radio_readout))
                .set_text(cx, labels.get(picked).copied().unwrap_or("?"));
        }
    }

    /// Hover tooltips for the overlay page.
    ///
    /// Through `event.hits`, **not** by comparing rectangles. The first version
    /// did `area.rect(cx).contains(me.abs)` and never fired: an `Area`'s rect is
    /// in pass coordinates while a mouse event's `abs` is in screen coordinates,
    /// so the two only agree when the window happens to be at the origin. That
    /// is the same hand-rolled geometry that v2 used to resolve z-order and hit
    /// handling, and it failed there for the same reason — `hits` exists so that
    /// nobody has to know which space an `Area` is in.
    ///
    /// Called for every event rather than only for `MouseMove`, because the
    /// signal wanted is the *transition* — hover-in shows, hover-out hides — and
    /// a transition only appears on the event that caused it.
    fn handle_hover_tooltips(&mut self, cx: &mut Cx, actions: &Actions) {
        // The control family's shared hover signal, read from the action batch.
        //
        // This is the fix for the page's original mistake. It used to ask
        // `event.hits` for buttons it did not own, and by hand-rolled geometry
        // before that (`area.rect(cx).contains(me.abs)` — a pass-relative rect
        // against a screen-absolute pointer, which only agrees when the window
        // sits at the origin). Now the *trigger* reports its own hover, which is
        // the only widget that can, and this listens.
        //
        // The tooltip still lives here rather than in a trigger, and not by
        // choice: an overlay draw list clips to its widget's rectangle, so a
        // tooltip cannot be owned by a trigger-sized wrapper. See `mp/tooltip.rs`.
        let hovers = makepad_component::mp::control::hovers(actions);
        if hovers.is_empty() {
            return;
        }
        const TRIGGERS: [(&[LiveId], &str); 5] = [
            (ids!(tip_primary), "Runs the primary action"),
            (ids!(tip_default), "Saves without closing"),
            (ids!(tip_ghost), "Dismisses what you were doing"),
            (ids!(tip_danger), "Cannot be undone"),
            (ids!(tip_one), "Anchored from the trigger that reported"),
        ];
        for (path, text) in TRIGGERS {
            let trigger = self.ui.widget(cx, path);
            let uid = trigger.widget_uid();
            for (hover_uid, hover) in &hovers {
                if *hover_uid != uid {
                    continue;
                }
                match hover {
                    makepad_component::mp::control::ControlHover::Entered => {
                        let area = self.ui.widget(cx, path).area();
                        self.ui.mp_tooltip(cx, ids!(tip)).show_for(cx, area, text);
                    }
                    makepad_component::mp::control::ControlHover::Left => {
                        self.ui.mp_tooltip(cx, ids!(tip)).hide(cx);
                    }
                }
            }
        }
    }

    /// Run a scripted undo/redo session and show the resulting stack.
    ///
    /// The script comes from `GALLERY_HISTORY` so the two faults this type exists to
    /// prevent can be *performed* at runtime rather than described: `push:a,push:b,undo,
    /// push:x` abandons `b`, and enough pushes past the capacity drop the oldest state and
    /// move the cursor with it. Each step is printed, because a state-order fault is
    /// invisible in a screenshot.
    fn seed_history(&mut self, cx: &mut Cx) {
        use makepad_component::mp::combobox::Combobox;
use makepad_editor::SnapshotHistory;
use makepad_component::mp::hover_card::HoverIntent;

        // Eight states of capacity: enough that a script can exceed it and still have a
        // readable stack.
        let mut history: SnapshotHistory<String> = SnapshotHistory::with_capacity("doc:0".to_string(), 8);
        let script = std::env::var("GALLERY_HISTORY").unwrap_or_else(|_| {
            // The default session: five edits, undo twice, then a different edit — which
            // is the shape that abandons a redo branch.
            "push:doc:1,push:doc:2,push:doc:3,push:doc:4,push:doc:5,undo,undo,push:doc:9"
                .to_string()
        });
        for step in script.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match step.split_once(':') {
                Some(("push", value)) => {
                    history.push(value.to_string());
                    println!("HISTORY push {value:?} -> {:?}", history.current());
                }
                _ if step == "undo" => {
                    let moved = history.undo().cloned();
                    println!("HISTORY undo -> {moved:?}");
                }
                _ if step == "redo" => {
                    let moved = history.redo().cloned();
                    println!("HISTORY redo -> {moved:?}");
                }
                _ => println!("HISTORY ignoring unrecognised step {step:?}"),
            }
        }

        let (undoable, redoable) = history.depth();
        println!(
            "HISTORY buffer={:?} cursor={} depth=({undoable} back, {redoable} forward)",
            history.entries(),
            history.cursor()
        );

        let items: Vec<ListItem> = history
            .entries()
            .iter()
            .enumerate()
            .map(|(index, state)| {
                let mut item = ListItem::new(state.clone());
                if index == history.cursor() {
                    item = item.detail("current");
                } else if index > history.cursor() {
                    // Ahead of the cursor: reachable by redo, and receding while it is.
                    item = item.detail("redo");
                }
                item
            })
            .collect();
        self.history = history;
        self.ui.mp_list(cx, ids!(history_states)).set_items(cx, items);
        self.ui.label(cx, ids!(history_depth)).set_text(
            cx,
            &format!(
                "{undoable} to undo, {redoable} to redo \u{b7} {} states retained of a capacity of 8 \u{b7} current {:?}",
                self.history.len(),
                self.history.current(),
            ),
        );
        self.ui.label(cx, ids!(history_note)).set_text(
            cx,
            &format!(
                "script: {script}. Set GALLERY_HISTORY to change it \u{2014} push:a,push:b,undo,push:x abandons b, and eight pushes past the capacity drop the front and move the cursor with it.",
            ),
        );
        self.buttons(cx);
    }

    /// Show what the two history buttons would do, so the page reports whether each is
    /// available rather than only painting them.
    fn buttons(&mut self, cx: &mut Cx) {
        let (undoable, redoable) = self.history.depth();
        self.ui
            .mp_button(cx, ids!(history_undo))
            .set_disabled(cx, undoable == 0);
        self.ui
            .mp_button(cx, ids!(history_redo))
            .set_disabled(cx, redoable == 0);
    }

    /// Undo and redo from the page's buttons.
    fn handle_history(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut moved = false;
        if self.ui.mp_button(cx, ids!(history_undo)).clicked(actions) {
            let state = self.history.undo().cloned();
            println!("HISTORY button undo -> {state:?}");
            moved = true;
        }
        if self.ui.mp_button(cx, ids!(history_redo)).clicked(actions) {
            let state = self.history.redo().cloned();
            println!("HISTORY button redo -> {state:?}");
            moved = true;
        }
        if moved {
            let (undoable, redoable) = self.history.depth();
            let items: Vec<ListItem> = self
                .history
                .entries()
                .iter()
                .enumerate()
                .map(|(index, state)| {
                    let mut item = ListItem::new(state.clone());
                    if index == self.history.cursor() {
                        item = item.detail("current");
                    } else if index > self.history.cursor() {
                        item = item.detail("redo");
                    }
                    item
                })
                .collect();
            self.ui.mp_list(cx, ids!(history_states)).set_items(cx, items);
            self.ui.label(cx, ids!(history_depth)).set_text(
                cx,
                &format!(
                    "{undoable} to undo, {redoable} to redo \u{b7} {} states retained of a capacity of 8 \u{b7} current {:?}",
                    self.history.len(),
                    self.history.current(),
                ),
            );
            self.buttons(cx);
        }
    }

    /// Fill the combobox and, if asked, drive it from a script.
    ///
    /// `GALLERY_COMBOBOX` is a comma-separated list of `type:X`, `step`, `back`, `commit`
    /// and `clear`, applied through the same `Combobox` the page's field and list are wired
    /// to. Every step prints the state, because the two things a combobox holds agreeing or
    /// disagreeing is invisible in a screenshot and this is a state machine.
    fn seed_combobox(&mut self, cx: &mut Cx) {
        self.combobox = Combobox::new(
            [
                "New Terminal",
                "Toggle Terminal",
                "Split Right",
                "Close Window",
                "Command Palette",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
        let script = std::env::var("GALLERY_COMBOBOX").unwrap_or_default();
        for step in script.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match step.split_once(':') {
                Some(("type", text)) => self.combobox.type_text(text),
                _ if step == "step" => self.combobox.step(1),
                _ if step == "back" => self.combobox.step(-1),
                _ if step == "commit" => {
                    let chosen = self.combobox.commit_active();
                    println!("COMBO commit -> {chosen:?}");
                }
                _ if step == "clear" => self.combobox.clear(),
                _ => println!("COMBO ignoring unrecognised step {step:?}"),
            }
            println!(
                "COMBO after {step:?} query={:?} value={:?} offered={:?} highlight={:?}",
                self.combobox.query(),
                self.combobox.value(),
                self.combobox.filtered(),
                self.combobox.active(),
            );
        }
        // The field shows the query the script produced, so the page and the run agree.
        let query = self.combobox.query().to_string();
        if !query.is_empty() {
            self.ui
                .text_input(cx, ids!(combobox_field))
                .set_text(cx, &query);
        }
        self.paint_combobox(cx, &script);
    }

    /// Push the combobox's state to the page: the panel's rows and the readouts.
    fn paint_combobox(&mut self, cx: &mut Cx, script: &str) {
        let offered = self.combobox.filtered();
        let items: Vec<ListItem> = offered
            .iter()
            .map(|index| {
                let mut item = ListItem::new(self.combobox.items()[*index].clone());
                if Some(*index) == self.combobox.active() {
                    item = item.detail("enter");
                }
                item
            })
            .collect();
        self.ui.mp_list(cx, ids!(combobox_list)).set_items(cx, items);

        let value = match self.combobox.value_label() {
            Some(label) => format!(
                "value = {label:?} at index {}",
                self.combobox.value().unwrap()
            ),
            None => "value = nothing chosen".to_string(),
        };
        let offered_text = if offered.is_empty() {
            "offered = (nothing matches)".to_string()
        } else {
            format!(
                "offered = {:?}",
                offered
                    .iter()
                    .map(|i| self.combobox.items()[*i].as_str())
                    .collect::<Vec<_>>()
            )
        };
        let highlight = match self.combobox.active() {
            Some(index) => format!(
                "highlight = {} (index {index}), what Enter would take",
                self.combobox.items()[index]
            ),
            None => "highlight = nothing, so Enter takes nothing".to_string(),
        };
        self.ui
            .label(cx, ids!(combo_query))
            .set_text(cx, &format!("query = {:?}", self.combobox.query()));
        self.ui.label(cx, ids!(combo_value)).set_text(cx, &value);
        self.ui
            .label(cx, ids!(combo_offered))
            .set_text(cx, &offered_text);
        self.ui
            .label(cx, ids!(combo_highlight))
            .set_text(cx, &highlight);
        self.ui.label(cx, ids!(combo_script)).set_text(
            cx,
            &format!(
                "script: {}. Set GALLERY_COMBOBOX to change it: type:spl,step,commit chooses Split Right; type:Split Righ clears the value.",
                if script.is_empty() { "(none)" } else { script },
            ),
        );
    }

    /// The field and the list, wired to the one `Combobox`.
    fn handle_combobox(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut changed = false;
        if let Some(text) = self.ui.text_input(cx, ids!(combobox_field)).changed(actions) {
            self.combobox.type_text(&text);
            changed = true;
        }
        if let Some(position) = self
            .ui
            .mp_list(cx, ids!(combobox_list))
            .row_selected(actions)
        {
            // A click on a row is a commit of that row, and it follows the same rule as
            // Enter: the index is resolved through the *current view* rather than used as a
            // filtered position.
            let view = self.combobox.filtered();
            if let Some(original) = makepad_component::mp::palette::original(&view, position) {
                self.combobox.choose(original);
                let label = self.combobox.query().to_string();
                self.ui
                    .text_input(cx, ids!(combobox_field))
                    .set_text(cx, &label);
                changed = true;
            }
        }
        if changed {
            let script = std::env::var("GALLERY_COMBOBOX").unwrap_or_default();
            self.paint_combobox(cx, &script);
        }
    }

    /// Run a scripted pointer through the hover card's timing machine.
    ///
    /// `GALLERY_HOVER` is a list of `presence:milliseconds` steps — `trigger`, `card`,
    /// `outside`. Every change the machine decides is printed, because **the timing is not
    /// photographable**: a synthetic pointer produces no hover event in this app, so the only
    /// evidence for the delay, the cancellation, the stay-open arm and the grace period is a
    /// log. The default script is a session that exercises all four.
    fn seed_hover_card(&mut self, cx: &mut Cx) {
        use makepad_component::mp::hover_card::{Change, HoverIntent, Presence};

        let mut intent = HoverIntent::default();
        let script = std::env::var("GALLERY_HOVER").unwrap_or_else(|_| {
            // A sweep that must amount to nothing, a rest that opens, a gap crossing, reading
            // the card, and leaving. The same sequence the module's own test walks.
            "trigger:150,outside:60,trigger:150,outside:60,trigger:150,outside:60,\
             trigger:520,outside:80,card:900,outside:200"
                .to_string()
        });
        let mut log: Vec<String> = Vec::new();
        for step in script.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let Some((presence, ms)) = step.split_once(':') else {
                log.push(format!("ignoring {step:?}"));
                continue;
            };
            let presence = match presence {
                "trigger" => Presence::OnTrigger,
                "card" => Presence::OnCard,
                "outside" => Presence::Outside,
                other => {
                    log.push(format!("ignoring unknown presence {other:?}"));
                    continue;
                }
            };
            let Ok(ms) = ms.parse::<f64>() else {
                log.push(format!("ignoring unparseable duration {ms:?}"));
                continue;
            };
            // Ticked at a frame's worth, because a real caller ticks per frame and a machine
            // that only works for one big jump is not the machine being shipped.
            let mut elapsed = 0.0;
            while elapsed < ms {
                let dt = 16.0_f64.min(ms - elapsed);
                match intent.update(presence, dt) {
                    // `elapsed + dt`, not `elapsed`: the decision happened at the **end** of
                    // this tick. Printing the time before it read as "after 496ms: OPENED"
                    // for a machine whose delay is 500ms — a number that appears to
                    // contradict the number it is demonstrating, which is worse than no log
                    // at all.
                    Change::Opened => {
                        log.push(format!("after {:>4.0}ms {presence:?}: OPENED", elapsed + dt));
                        println!("HOVER {step}: Opened at {:.0}ms", elapsed + dt);
                    }
                    Change::Closed => {
                        log.push(format!("after {:>4.0}ms {presence:?}: CLOSED", elapsed + dt));
                        println!("HOVER {step}: Closed at {:.0}ms", elapsed + dt);
                    }
                    Change::Nothing => {}
                }
                elapsed += dt;
            }
        }
        println!(
            "HOVER final open={} dwell={:.0}ms",
            intent.is_open(),
            intent.dwell_ms()
        );

        self.ui.label(cx, ids!(hover_log)).set_text(
            cx,
            &format!(
                "{} changes decided: {}",
                log.len(),
                if log.is_empty() {
                    "none".to_string()
                } else {
                    log.join(" \u{b7} ")
                },
            ),
        );
        self.ui.label(cx, ids!(hover_replay)).set_text(
            cx,
            &format!(
                "script: {script}. Set GALLERY_HOVER to change it. Final state: {}.",
                if intent.is_open() { "open" } else { "closed" },
            ),
        );
        self.ui.label(cx, ids!(hover_numbers)).set_text(
            cx,
            &format!(
                "delay = {:.0}ms (gpui's own tooltip delay), grace = {:.0}ms (chosen). \
                 A card with a zero delay opens on the first tick and closes on the first \
                 departure, which is what a caller asking for an immediate card gets.",
                intent.delay_ms(),
                intent.grace_ms(),
            ),
        );
    }

    /// Run a scripted drag through the panel's machine.
    ///
    /// `GALLERY_FLOAT` is a list of `press:x,y`, `move:x,y` and `release` steps separated by
    /// `;`, because `,` belongs to the coordinate. Every decision
    /// is printed, because "the panel did not move" and "the panel moved by the right amount"
    /// are indistinguishable in a screenshot of a box — and a synthetic pointer produces no
    /// press in this app, so the gesture has to be scripted.
    fn seed_floating(&mut self, cx: &mut Cx) {
        use makepad_component::mp::floating::Floating;

        // The panel starts here, and the default script's first press lands 20 across and 10
        // down inside it, so the grab offset is visible in the numbers rather than merely
        // claimed.
        let start = dvec2(20.0, 10.0);
        self.ui.mp_floating(cx, ids!(float_panel)).set_pos(cx, start);
        let mut machine = Floating::new(start);

        let script = std::env::var("GALLERY_FLOAT").unwrap_or_else(|_| {
            // Sub-threshold, then a real drag, then back. The second step must decide nothing:
            // that is the shaky-click rule.
            // `;` between steps and `,` inside a coordinate. The first version used `,` for
            // both, so `press:40,20` split into the steps `press:40` and `20` — every step then
            // failed to parse and the log came out **empty**, which is exactly what the
            // "print every decision" rule is for: a gesture that decided nothing and a script
            // that ran nothing look identical in a screenshot.
            "press:40,20;move:41,21;move:240,120;move:40,20;release".to_string()
        });
        let mut log: Vec<String> = Vec::new();
        for step in script.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            let (verb, coords) = match step.split_once(':') {
                Some((verb, coords)) => (verb, Some(coords)),
                None => (step, None),
            };
            let point = || -> Option<Vec2d> {
                let coords = coords?;
                let (x, y) = coords.split_once(',')?;
                Some(dvec2(x.trim().parse().ok()?, y.trim().parse().ok()?))
            };
            match verb {
                "press" => match point() {
                    Some(at) => {
                        machine.press(at);
                        log.push(format!("press at ({:.0},{:.0})", at.x, at.y));
                        println!("FLOAT press at ({:.0},{:.0}) held", at.x, at.y);
                    }
                    None => log.push(format!("ignoring {step:?}")),
                },
                "move" => match point() {
                    Some(at) => {
                        let moved = machine.move_to(at);
                        log.push(format!(
                            "move to ({:.0},{:.0}) -> {}",
                            at.x,
                            at.y,
                            if moved { "moved" } else { "did not move" }
                        ));
                        println!(
                            "FLOAT move to ({:.0},{:.0}) moved={moved} pos=({:.0},{:.0}) dragging={}",
                            at.x,
                            at.y,
                            machine.pos().x,
                            machine.pos().y,
                            machine.is_dragging()
                        );
                    }
                    None => log.push(format!("ignoring {step:?}")),
                },
                "release" => {
                    machine.release();
                    log.push("release".to_string());
                }
                other => log.push(format!("ignoring unknown step {other:?}")),
            }
        }
        // The panel the reader sees is placed where the machine ended, so the picture and the
        // log agree — the same rule the combobox page follows with its field.
        self.ui
            .mp_floating(cx, ids!(float_panel))
            .set_pos(cx, machine.pos());
        println!(
            "FLOAT final pos=({:.0},{:.0}) dragging={}",
            machine.pos().x,
            machine.pos().y,
            machine.is_dragging()
        );

        self.ui.label(cx, ids!(float_log)).set_text(
            cx,
            &format!(
                "{} steps: {}",
                log.len(),
                if log.is_empty() {
                    "none".to_string()
                } else {
                    log.join(" \u{b7} ")
                }
            ),
        );
        self.ui.label(cx, ids!(float_state)).set_text(
            cx,
            &format!(
                "script: {script}. Final position ({:.0}, {:.0}), grabbed 20 across and 10 down. \
                 Nothing clamps: a panel dragged half off the window stays there, and the grab \
                 offset means it can always be dragged back.",
                machine.pos().x,
                machine.pos().y,
            ),
        );
    }

    /// Classify a JSON document and hand it to the code block.
    ///
    /// The wiring the module docs describe as "one line": the classifier produces spans, the widget
    /// paints them, and neither knows about the other. The legend is built the same way — one word
    /// per kind, each word spanned with that kind — so it is drawn by the widget rather than
    /// hand-coloured, and cannot fall out of step with the palette.
    fn seed_code(&mut self, cx: &mut Cx) {
        use makepad_component::mp::code::MpCodeBlockWidgetRefExt;
        use makepad_theme::syntax::HighlightKind;

        // An A2UI message with three deliberate faults, so the three cases the widget is built around
        // are on screen rather than only in a test: a multi-line token, an escaped quote, and a
        // misspelling. `classify` is per-line JSON, so the comment markers below are what a reader
        // would see rather than something JSON has.
        let source = r#"{
  "beginRendering": {
    "surfaceId": "main",
    "root": "card"
  },
  "surfaceUpdate": {
    "components": [
      { "id": "card", "component": { "Card": { "child": "title" } } },
      { "id": "title", "component": { "Text": { "text": "Hello \"world\" — 世界" } } }
    ]
  },
  "dataModelUpdate": { "contents": [ { "key": "count", "valueNumber": -1.5e3 } ] },
  "deleted": nul,
  "ok": true
}"#;

        let spans = makepad_syntax::classify(source, "json").unwrap_or_default();
        let (mut counts, mut total) = ([0usize; 13], 0usize);
        for span in &spans {
            counts[makepad_theme::syntax::SyntaxPalette::index_of(span.kind)] += 1;
            total += span.range.len();
        }
        println!(
            "CODE classified {} chars of json into {} spans covering {total} bytes",
            source.len(),
            spans.len()
        );
        self.ui
            .mp_code_block(cx, ids!(code_json))
            .set_highlighted(cx, source, &spans);

        // The legend, built from the kind table itself.
        let names: Vec<&str> = HighlightKind::ALL
            .iter()
            .map(|kind| match kind {
                HighlightKind::Keyword => "keyword",
                HighlightKind::Function => "function",
                HighlightKind::Type => "type",
                HighlightKind::Constant => "constant",
                HighlightKind::Variable => "variable",
                HighlightKind::String => "string",
                HighlightKind::Number => "number",
                HighlightKind::Comment => "comment",
                HighlightKind::Operator => "operator",
                HighlightKind::Punctuation => "punctuation",
                HighlightKind::Attribute => "key",
                HighlightKind::Tag => "tag",
                HighlightKind::Invalid => "invalid",
            })
            .collect();
        // Two rows of words, each word one span, so the legend uses the same paint path as the code.
        let mut legend = String::new();
        let mut legend_spans = Vec::new();
        for (index, name) in names.iter().enumerate() {
            if index == 7 {
                legend.push('\n');
            }
            let start = legend.len();
            legend.push_str(name);
            legend_spans.push(makepad_theme::syntax::Highlight {
                range: start..legend.len(),
                kind: HighlightKind::ALL[index],
            });
            legend.push_str("  ");
        }
        self.ui
            .mp_code_block(cx, ids!(code_legend))
            .set_highlighted(cx, &legend, &legend_spans);
        println!("CODE legend built with {} spans", legend_spans.len());

        let palette = makepad_theme::syntax::SyntaxPalette::for_appearance(
            makepad_theme::Appearance::Dark,
        );
        let present: Vec<String> = HighlightKind::ALL
            .iter()
            .filter(|kind| counts[makepad_theme::syntax::SyntaxPalette::index_of(**kind)] > 0)
            .map(|kind| {
                format!(
                    "{kind:?} x{}",
                    counts[makepad_theme::syntax::SyntaxPalette::index_of(*kind)]
                )
            })
            .collect();
        self.ui.label(cx, ids!(code_stats)).set_text(
            cx,
            &format!(
                "{} spans over {} bytes of source; kinds present: {}",
                spans.len(),
                source.len(),
                present.join(", ")
            ),
        );
        // The traps, reported rather than only painted: a page that claims a case works should say
        // which case it put on screen.
        let invalid = counts[makepad_theme::syntax::SyntaxPalette::index_of(HighlightKind::Invalid)];
        let strings =
            counts[makepad_theme::syntax::SyntaxPalette::index_of(HighlightKind::String)];
        self.ui.label(cx, ids!(code_traps)).set_text(
            cx,
            &format!(
                "cases on screen: {invalid} invalid span (the misspelled `nul`), {strings} string spans \
                 (one of them containing an escaped quote), and the palette's Invalid colour is the \
                 most saturated of the thirteen at {:.2},{:.2},{:.2}",
                palette
                    .color(HighlightKind::Invalid)
                    .x,
                palette.color(HighlightKind::Invalid).y,
                palette.color(HighlightKind::Invalid).z,
            ),
        );
    }

    /// Give the document pages their source and report what the model and the layout produced.
    ///
    /// The **fixed point is checked here, on the document being displayed** — the tests check it over a
    /// corpus, and this checks the one a reader is looking at. That is the strongest thing this page can
    /// say without a screenshot, which this session does not have.
    fn seed_document(&mut self, cx: &mut Cx) {
        use makepad_component::mp::markdown::MpMarkdownWidgetRefExt;

        let source = "# A document\n\n\
            A paragraph with **bold**, *italic* and `code` in it, long enough that it wraps onto a\n\
            second line in a narrow column.\n\n\
            - a bullet\n\
            - another bullet\n\
            \u{20} - a nested one\n\n\
            3. ordered, starting at three\n\
            4. and continuing\n\n\
            - [ ] a task\n\
            - [x] a finished task\n\n\
            > a quote, which is one block however many lines it has\n\
            > like this\n\n\
            ```rust\n\
            fn main() {\n\
                println!(\"not *markdown* inside a fence\");\n\
            }\n\
            ```\n\n\
            ---\n\n\
            A last paragraph, so the divider has something on both sides.\n";

        // The model's own property, on this document.
        let first = makepad_markdown::parse(source);
        let written = makepad_markdown::serialize(&first);
        let second = makepad_markdown::parse(&written);
        let holds = first == second;
        let stable = written == makepad_markdown::serialize(&second);
        println!(
            "DOCUMENT blocks={} fixed_point={holds} stable_after_one_write={stable} wire_bytes={}",
            first.blocks.len(),
            written.len()
        );
        for (index, block) in first.blocks.iter().enumerate() {
            let kind = match &block.kind {
                makepad_markdown::BlockKind::Paragraph(_) => "paragraph",
                makepad_markdown::BlockKind::Heading { .. } => "heading",
                makepad_markdown::BlockKind::Bullet(_) => "bullet",
                makepad_markdown::BlockKind::Ordered { .. } => "ordered",
                makepad_markdown::BlockKind::Task { .. } => "task",
                makepad_markdown::BlockKind::Quote(_) => "quote",
                makepad_markdown::BlockKind::Code { .. } => "code",
                makepad_markdown::BlockKind::Divider => "divider",
            };
            println!("DOCUMENT   [{index}] indent={} {kind}", block.indent);
        }

        // Two columns, so the wrap is doing something a reader can see the effect of.
        for (id, measure) in [(ids!(doc_full), 620.0f64), (ids!(doc_narrow), 300.0f64)] {
            let view = self.ui.mp_markdown(cx, id);
            view.set_measure(cx, measure);
            view.set_source(cx, source);
        }

        self.ui.label(cx, ids!(doc_stats)).set_text(
            cx,
            &format!(
                "{} blocks, {} of them with a marker; the narrow column wraps the same {} blocks \\
                 into more lines \u{2014} printed on the next run of the page",
                first.blocks.len(),
                first.blocks
                    .iter()
                    .filter(|block| matches!(
                        block.kind,
                        makepad_markdown::BlockKind::Bullet(_)
                            | makepad_markdown::BlockKind::Ordered { .. }
                            | makepad_markdown::BlockKind::Task { .. }
                            | makepad_markdown::BlockKind::Quote(_)
                    ))
                    .count(),
                first.blocks.len(),
            ),
        );
        self.ui.label(cx, ids!(doc_fixed_point)).set_text(
            cx,
            &format!(
                "the fixed point holds on the document above: parse \u{2192} serialize \u{2192} parse is the \
                 same document ({holds}), and a second write is byte-identical ({stable}). The wire form is \
                 {} bytes.",
                written.len()
            ),
        );
        // The wire form, painted by the code block — so the round trip is visible as text as well as
        // asserted as a boolean.
        if let Some(spans) = makepad_syntax::classify(&written, "json") {
            let _ = spans;
        }
        self.ui
            .mp_code_block(cx, ids!(doc_wire))
            .set_highlighted(cx, &written, &[]);
    }

    /// Give the editor its document and, if asked, drive it from a script.
    ///
    /// `GALLERY_EDITOR` is a comma-separated list of steps: `type:text`, `enter`, `backspace`, `delete`, `tab`,
    /// `outdent`, `left`, `right`, `up`, `down`, `home`, `end`, `undo`, `redo`. Every step is applied through the
    /// **same methods a keypress applies** — `insert_text`, `press`, `move_by`, `undo`, `redo` — so a run checks
    /// the behaviour a key gets rather than a second path that resembles it.
    fn seed_editor(&mut self, cx: &mut Cx) {
        use makepad_editor::EditKind;
        use makepad_markdown::edit::Shortcut;
        use makepad_component::mp::editor::{Motion, MpEditorWidgetRefExt};

        let source = "# A note\n\nAn editor over the document model. Click in it, or drive it from the \\
                      environment.\n\n- a bullet\n- another\n\n";
        let script = std::env::var("GALLERY_EDITOR").unwrap_or_else(|_| {
            // A session that exercises every rule: type at a caret, split with Enter, merge with Backspace,
            // indent, move, and undo twice — which has to land on the text and then on the split. It ends with the
            // slash menu: `/`, a query that narrows it, a walk down it, and Enter to take the row.
            "end,type: one,type: two,enter,type:next,home,tab,left,left,type:X,undo,undo,\
             enter,type:/,slash,type:head,slash,slash-down,slash,slash-down,slash,\
             slash-up,slash,slash-enter,slash,enter,type:/,type:co,slash"
                .to_string()
        });

        let view = self.ui.mp_editor(cx, ids!(editor_surface));
        view.set_measure(cx, 620.0);
        view.set_source(cx, source);

        for step in script.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match step.split_once(':') {
                Some(("type", text)) => view.insert_text(cx, text),
                _ => match step {
                    "enter" => view.press(cx, Shortcut::Enter, EditKind::Structural),
                    "backspace" => view.press(cx, Shortcut::Backspace, EditKind::Deleting),
                    "delete" => view.press(cx, Shortcut::Delete, EditKind::Deleting),
                    "tab" => view.press(cx, Shortcut::Indent, EditKind::Structural),
                    "outdent" => view.press(cx, Shortcut::Outdent, EditKind::Structural),
                    // Print the slash menu's whole state, so the run's own output is the evidence for what the
                    // wiring did rather than a claim about it.
                    "slash" => match view.slash_state() {
                        Some(state) => println!(
                            "SLASH open at={} query={:?} rows={:?} active={} choice={:?}",
                            state.at, state.query, state.rows, state.active, state.choice
                        ),
                        None => println!("SLASH closed"),
                    },
                    // The menu's **public** operations, which the key handler calls too — so a scripted step and a
                    // keypress cannot diverge.
                    "slash-down" => {
                        let consumed = view.slash_step(cx, 1);
                        println!("SLASH step +1 consumed={consumed}");
                    }
                    "slash-up" => {
                        let consumed = view.slash_step(cx, -1);
                        println!("SLASH step -1 consumed={consumed}");
                    }
                    "slash-enter" => {
                        let consumed = view.slash_commit(cx);
                        println!("SLASH commit consumed={consumed}");
                    }
                    "slash-esc" => {
                        let consumed = view.slash_close(cx);
                        println!("SLASH close consumed={consumed}");
                    }
                    "left" => view.move_by(cx, Motion::Left, false),
                    "right" => view.move_by(cx, Motion::Right, false),
                    "up" => view.move_by(cx, Motion::Up, false),
                    "down" => view.move_by(cx, Motion::Down, false),
                    "home" => view.move_by(cx, Motion::Home, false),
                    "end" => view.move_by(cx, Motion::End, false),
                    "undo" => {
                        let moved = view.undo(cx);
                        println!("EDITOR undo -> {moved}");
                    }
                    "redo" => {
                        let moved = view.redo(cx);
                        println!("EDITOR redo -> {moved}");
                    }
                    other => println!("EDITOR ignoring unknown step {other:?}"),
                },
            }
            println!("EDITOR after {step:?} -> {:?}", view.source().unwrap_or_default());
        }

        let written = view.source().unwrap_or_default();
        let (undoable, redoable) = view.depth();
        println!("EDITOR final undoable={undoable} redoable={redoable} bytes={}", written.len());

        self.ui
            .mp_code_block(cx, ids!(editor_wire))
            .set_highlighted(cx, &written, &[]);
        self.ui.label(cx, ids!(editor_state)).set_text(
            cx,
            &format!(
                "{undoable} to undo, {redoable} to redo \u{b7} the document is {} bytes of markdown",
                written.len()
            ),
        );
        self.ui.label(cx, ids!(editor_script)).set_text(
            cx,
            &format!(
                "script: {script}. Set GALLERY_EDITOR to change it \u{2014} the steps are applied through the same \
                 methods a keypress applies, so this is the behaviour a key gets."
            ),
        );
    }

    /// Parse a JSON Canvas document, check it, and paint it.
    ///
    /// The **fixed point is checked on the document being displayed** rather than over a corpus — the corpus is the
    /// crate's own test — and a **deliberately broken** document is validated too, because a report that only ever
    /// prints nothing is indistinguishable from a report that does not work.
    fn seed_canvas(&mut self, cx: &mut Cx) {
        use makepad_component::mp::canvas::MpCanvasWidgetRefExt;

        // A document with every node type, a group, four presets, two hex colours, and edges whose sides and ends
        // exercise the spec's asymmetric defaults.
        let source = r##"{
  "nodes": [
    { "id": "group", "type": "group", "x": -40, "y": -50, "width": 620, "height": 330,
      "label": "A group", "color": "5" },
    { "id": "text", "type": "text", "x": 0, "y": 0, "width": 180, "height": 90,
      "text": "A text node with **markdown** in it", "color": "1" },
    { "id": "file", "type": "file", "x": 240, "y": 0, "width": 160, "height": 80,
      "file": "notes/canvas.md", "subpath": "#json-canvas", "color": "4" },
    { "id": "link", "type": "link", "x": 0, "y": 160, "width": 180, "height": 70,
      "url": "https://jsoncanvas.org", "color": "2" },
    { "id": "hex", "type": "text", "x": 240, "y": 160, "width": 160, "height": 70,
      "text": "A hex colour, not a preset", "color": "#FF66CC" }
  ],
  "edges": [
    { "id": "e1", "fromNode": "text", "fromSide": "right", "toNode": "file", "toSide": "left",
      "color": "1", "label": "writes" },
    { "id": "e2", "fromNode": "text", "fromSide": "bottom", "toNode": "link", "toSide": "top" },
    { "id": "e3", "fromNode": "file", "fromSide": "bottom", "fromEnd": "arrow",
      "toNode": "hex", "toSide": "top", "toEnd": "none" },
    { "id": "e4", "fromNode": "hex", "fromSide": "right", "toNode": "text", "toSide": "right",
      "toSide_note": "an elbow that goes back around", "color": "#8899FF" }
  ]
}
"##;

        let canvas = makepad_canvas::parse(source).expect("the document parses");
        let problems = canvas.validate();
        let written = makepad_canvas::serialize(&canvas);
        let reread = makepad_canvas::parse(&written).expect("what we wrote parses");
        let holds = reread == canvas;
        let stable = makepad_canvas::serialize(&reread) == written;
        println!(
            "CANVAS nodes={} edges={} problems={} fixed_point={holds} stable={stable} wire_bytes={}",
            canvas.nodes.len(),
            canvas.edges.len(),
            problems.len(),
            written.len()
        );

        // A document with the faults the format allows: a duplicate id and an edge to a node that is not there.
        let broken_source = r##"{
  "nodes": [
    { "id": "same", "type": "text", "x": 0, "y": 0, "width": 10, "height": 10, "text": "a" },
    { "id": "same", "type": "file", "x": 0, "y": 0, "width": 10, "height": 10, "file": "a.md", "subpath": "nohash" }
  ],
  "edges": [ { "id": "e", "fromNode": "ghost", "toNode": "same" } ]
}"##;
        let broken = makepad_canvas::parse(broken_source).expect("it parses, faults and all");
        let broken_problems = broken.validate();
        for problem in &broken_problems {
            println!("CANVAS problem: {problem:?}");
        }

        let view = self.ui.mp_canvas(cx, ids!(canvas_view));
        view.set_canvas(cx, canvas.clone());

        let presets = makepad_canvas::Preset::ALL
            .iter()
            .map(|preset| format!("{}={}", preset.index(), preset.type_name()))
            .collect::<Vec<_>>()
            .join(" ");

        self.ui.label(cx, ids!(canvas_stats)).set_text(
            cx,
            &format!(
                "{} nodes, {} edges, {} bytes of wire form \u{b7} the six presets in the spec's order: {presets}",
                canvas.nodes.len(),
                canvas.edges.len(),
                written.len()
            ),
        );
        self.ui.label(cx, ids!(canvas_fixed_point)).set_text(
            cx,
            &format!(
                "the fixed point holds on the document above: parse \u{2192} serialize \u{2192} parse is the same \
                 document ({holds}), and a second write is byte-identical ({stable})"
            ),
        );
        self.ui.label(cx, ids!(canvas_problems)).set_text(
            cx,
            &format!(
                "the good document reports {} problems \u{2014} a writer never produces any. The three faults the \
                 format allows are reported rather than refused, because a reader that dropped a document with a \
                 dangling edge would delete a user's work over a fault it can still display.",
                problems.len()
            ),
        );
        self.ui.label(cx, ids!(canvas_broken)).set_text(
            cx,
            &format!(
                "a deliberately broken document reports {}: {}\u{2003}",
                broken_problems.len(),
                broken_problems
                    .iter()
                    .map(|problem| format!("{problem:?}"))
                    .collect::<Vec<_>>()
                    .join(" \u{b7} ")
            ),
        );
        self.ui
            .mp_code_block(cx, ids!(canvas_wire))
            .set_highlighted(cx, &written, &[]);
    }

    /// Fill the title bars and print the drag arithmetic.
    ///
    /// **The arithmetic is the evidence**, because the drag itself cannot run here: a widget cannot move a window, so the
    /// bar reports a delta and an application applies it. What is printed is the same `drag_region`/`is_draggable` the widget
    /// calls, plus a scripted walk of `DragState` — which is where the two rules that matter live: each report is measured
    /// from the **last** one (so a long drag does not drift) and a non-finite movement is **dropped**, not clamped.
    fn seed_titlebars(&mut self, cx: &mut Cx) {
        use makepad_component::mp::titlebar::{
            drag_region, is_draggable, DragState, MpTitlebarWidgetRefExt, TITLEBAR_HEIGHT,
        };

        for (id, name) in [
            (ids!(titlebar_plain), "plain"),
            (ids!(titlebar_controls), "controls"),
            (ids!(titlebar_narrow), "narrow"),
            (ids!(titlebar_short), "short"),
        ] {
            let bar = self.ui.mp_titlebar(cx, id);
            bar.set_title(cx, "Makepad Component");
            println!("TITLEBAR {name} title_set=true height={TITLEBAR_HEIGHT}");
        }

        // The geometry rule, at two widths a caller would actually use: the draggable region ends where the controls begin.
        for width in [320.0f64, 180.0] {
            let (_, end) = drag_region(width, 80.0);
            println!(
                "TITLEBAR region width={width} controls=80 draggable=[0,{end}) inside_at_0={} inside_at_end={}",
                is_draggable(0.0, width, 80.0),
                is_draggable(end, width, 80.0),
            );
        }

        // The drag walk: measured from the last report, and a `NaN` dropped rather than clamped.
        let mut drag = DragState::default();
        drag.begin(dvec2(100.0, 100.0));
        let first = drag.take_delta(dvec2(104.0, 103.0));
        let second = drag.take_delta(dvec2(110.0, 103.0));
        let bad = drag.take_delta(dvec2(f64::NAN, 103.0));
        let after_bad = drag.take_delta(dvec2(114.0, 103.0));
        println!(
            "TITLEBAR drag first={first:?} second={second:?} nan={bad:?} after_nan={after_bad:?} still_dragging={}",
            drag.is_dragging(),
        );
    }

    /// Fill the three menu panels and print what each decided.
    ///
    /// **The printed lines are the evidence**, because the two decisions that matter are invisible in a picture: whether the
    /// gutter was reserved and what width the panel took. A screenshot shows a panel; only the numbers show that the gutter
    /// is absent when nothing uses it.
    fn seed_menu_cards(&mut self, cx: &mut Cx) {
        use makepad_component::mp::menu::Item;
        use makepad_component::mp::menu_card::{
            line_height, panel_height, panel_width, row_at, MpMenuCardWidgetRefExt,
        };

        let plain = vec![
            Item::action("New file").with_keystroke("\u{2318}N"),
            Item::action("Open folder").with_keystroke("\u{2318}O"),
            Item::Separator,
            Item::action("Word wrap").checked(true),
            Item::action("Save").disabled(),
            Item::submenu("Share", vec![Item::action("Copy link"), Item::action("Email")]),
        ];
        let glyphs = vec![
            Item::action("New file").with_icon("file").with_keystroke("\u{2318}N"),
            Item::action("Open folder").with_icon("folder"),
            Item::Separator,
            Item::action("Save").with_icon("save"),
            Item::submenu("Share", vec![Item::action("Copy link")]).with_icon("share"),
        ];
        let described = vec![
            Item::action("Open folder").with_description("Choose a folder to work in"),
            Item::action("Duplicate").with_description("A copy beside the original"),
            Item::Separator,
            Item::action("Delete").disabled().with_description("Moved to the bin, restorable"),
        ];

        for (id, rows, name) in [
            (ids!(menu_plain), plain, "plain"),
            (ids!(menu_glyphs), glyphs, "glyphs"),
            (ids!(menu_described), described, "described"),
        ] {
            let count = rows.len();
            let card = self.ui.mp_menu_card(cx, id);
            card.set_items(cx, rows);
            let line = line_height(cx);
            let width = card.width();
            let height = {
                // The panel's height needs the rows, so it is recomputed here from the same function the card uses.
                let rows = card.items();
                panel_height(&rows, line)
            };
            let gutter = card.items().iter().any(|item| match item {
                Item::Action { icon, .. } | Item::Submenu { icon, .. } => icon.is_some(),
                Item::Separator => false,
            });
            // Where a `y` lands, which is the half of the geometry a picture cannot show at all.
            let third = row_at(&card.items(), height / 3.0, line);
            println!(
                "MENUS {name} rows={count} width={width} height={height} gutter={gutter} width_from=panel_width third_row={third:?}"
            );
        }

        // **And the bar's rules, driven in the running app.** The state machine has no widget to photograph, so the preview
        // is the transitions themselves: a menubar is defined by what one open menu makes the other titles do, and that is
        // invisible in any still picture. Each line below is one key or one hover, in order.
        self.run_menubar_preview();
    }

    /// Drive `mp::menubar::Bar` through the rules that make a bar a bar, printing each transition.
    ///
    /// The evidence unit tests cannot give: **this runs in the app**, with the theme installed and the script VM live, so a
    /// panic or a wrong branch here is a real one. The four rules it walks are the ones the component exists for.
    fn run_menubar_preview(&mut self) {
        use makepad_component::mp::menu::Item;
        use makepad_component::mp::menubar::{Bar, Menu, MpMenubarHit, Outcome};

        let menus = vec![
            Menu::new(
                "File",
                vec![
                    Item::action("New Window"),
                    Item::submenu("Open Recent", vec![Item::action("notes.md")]),
                    Item::Separator,
                    Item::action("Close").disabled(),
                ],
            ),
            Menu::new("Edit", vec![Item::action("Undo"), Item::action("Redo")]),
        ];
        let mut bar = Bar::new(menus.clone());
        let show = |outcome: &Outcome| match outcome {
            Outcome::None => "none",
            Outcome::Changed => "changed",
            Outcome::Chose(..) => "chose",
        };
        // A **hover with nothing open does nothing** — the asymmetry that separates a bar from a row of dropdowns.
        let idle = bar.hover_switch(1);
        // Then open, and hover a sibling: that switches with no click.
        let opened = bar.toggle(0);
        let switched = bar.hover_switch(1);
        println!(
            "MENUBAR hover_closed={} open={} hover_open={} now_open={:?}",
            show(&idle),
            show(&opened),
            show(&switched),
            bar.open(),
        );

        // The cursor walks to the submenu row and `right` goes **in** rather than crossing, because a submenu row that
        // swallowed the key would be a dead key on the only row with somewhere to go.
        bar.toggle(0);
        let first = bar.step_item(1);
        let second = bar.step_item(1);
        let deeper = bar.go_deeper();
        println!(
            "MENUBAR enter={} to_submenu_row={} right_descends={} nested={} still_in_menu={:?}",
            show(&first),
            show(&second),
            show(&deeper),
            bar.cursor().nested(),
            bar.open(),
        );

        // `escape` backs out one level and leaves the bar up; a second one closes it. That is the difference between "back
        // out" and "cancel everything".
        let escaped_once = bar.dismiss();
        let was_open_after_one = bar.is_open();
        let escaped_twice = bar.dismiss();
        println!(
            "MENUBAR escape1={} still_open={was_open_after_one} escape2={} open_after={:?}",
            show(&escaped_once),
            show(&escaped_twice),
            bar.open(),
        );

        // The pointer moves the same cursor the keyboard does, so the two cannot disagree about which row a submenu hangs
        // off; and `enter` on a closed bar drops the first menu, so the key always means "act on this control".
        bar.toggle(0);
        let pointed = bar.hit(&MpMenubarHit::Point(vec![0]));
        let live = bar.cursor().row();
        let chosen = bar.confirm();
        let closed = !bar.is_open();
        let opened_from_closed = bar.confirm();
        println!(
            "MENUBAR point={} live_row={live:?} enter={} bar_closed_after_choose={closed} enter_when_closed={} open_now={:?}",
            show(&pointed),
            show(&chosen),
            show(&opened_from_closed),
            bar.open(),
        );
    }

    /// Fill the swatch grids on the picking page, and print the grid's shape and what a few points pick.
    ///
    /// **The points are the point.** A picture shows a grid; only the printed hit tests show that the gaps pick nothing
    /// and that the space past a partial row's last column does either — which is the logic this component exists to get
    /// right.
    fn seed_picking(&mut self, cx: &mut Cx) {
        use makepad_component::mp::color_picker::{
            cell_index, grid_height, grid_width, rows, MpColorPickerWidgetRefExt, DEFAULT_COLUMNS, GAP, SWATCH,
        };

        let hexes = [
            "#E5484D", "#F76B15", "#FFB224", "#46A758", "#12A594", "#0090FF", "#3E63DD", "#8E4EC6", "#E93D82",
        ];
        let palette: Vec<Vec4f> = hexes
            .iter()
            .map(|hex| {
                let channel = |at: usize| {
                    u8::from_str_radix(&hex[at..at + 2], 16).unwrap_or(0) as f32 / 255.0
                };
                vec4(channel(1), channel(3), channel(5), 1.0)
            })
            .collect();

        for (id, columns) in [
            (ids!(picking_one_col), DEFAULT_COLUMNS),
            (ids!(picking_four), 4usize),
            (ids!(picking_column), 1usize),
        ] {
            let view = self.ui.mp_color_picker(cx, id);
            view.set_colors(cx, palette.clone());
            view.set_columns(cx, columns);
            // The middle swatch selected, so the inner ring is on screen rather than only in the code.
            view.set_selected(cx, Some(palette.len() / 2));
            let count = palette.len();
            let stride = SWATCH + GAP;
            // Aim at three points and report what each picks: a cell's centre, the gap to its right, and the point where
            // a column past the last one would be.
            let centre = cell_index(SWATCH * 0.5, SWATCH * 0.5, count, columns);
            let in_the_gap = cell_index(SWATCH + GAP * 0.5, SWATCH * 0.5, count, columns);
            let past_the_row = cell_index(columns as f64 * stride, SWATCH * 0.5, count, columns);
            println!(
                "PICKING swatches={count} columns={columns} rows={} grid={}x{} centre={centre:?} gap={in_the_gap:?} past_row={past_the_row:?}",
                rows(count, columns),
                grid_width(columns),
                grid_height(count, columns),
            );
        }
    }

    /// Fill the searchable lists on the searching page, and print what each holds and shows.
    ///
    /// The third case is the one that matters: **select a row, then narrow the list**, and print the selection before and
    /// after — because the defect this widget was written around is exactly a selection that moves when the filter does,
    /// and it is invisible until you type.
    fn seed_searching(&mut self, cx: &mut Cx) {
        use makepad_component::mp::searchable_list::{
            rows, total_matches, MpSearchableListWidgetRefExt, SLOTS,
        };

        let six: Vec<String> = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta"]
            .iter()
            .map(|item| item.to_string())
            .collect();
        let many: Vec<String> = (1..=SLOTS + 8)
            .map(|index| format!("Component {index}"))
            .collect();

        for (id, items) in [
            (ids!(searching_six), &six),
            (ids!(searching_selected), &six),
            (ids!(searching_many), &many),
        ] {
            let view = self.ui.mp_searchable_list(cx, id);
            view.set_items(cx, items.clone());
            println!(
                "SEARCHING items={} shown={} total={} at query=\"\"",
                items.len(),
                rows(items, "", None).len(),
                total_matches(items, ""),
            );
        }

        // **The selection test, on the page.** Select item 2 (`Gamma`) and narrow to `et`, which **hides `Gamma`** —
        // `Alpha` and `Gamma` both contain `a`, so a query of `a` would have left it on screen and proved nothing. What
        // the run has to show is that the selection is still `Gamma` while `Gamma` is not among the rows. The v2 widget
        // would have reported whatever landed at position 2 of the narrowed list; this one reports `Gamma`.
        let view = self.ui.mp_searchable_list(cx, ids!(searching_selected));
        view.set_selected(cx, Some(2));
        let before = view.selected_text();
        view.set_query(cx, "et");
        // The selected item must be **absent from the rows** for this to be evidence at all.
        assert!(
            !view.rows().iter().any(|row| row.text == "Gamma"),
            "the narrowed list still shows the selection, so the case proves nothing"
        );
        let narrowed = view.rows();
        let after = view.selected_text();
        println!(
            "SEARCHING select=2 before={before:?} narrowed_to={:?} after={after:?} still_gamma={}",
            narrowed.iter().map(|row| row.text.as_str()).collect::<Vec<_>>(),
            after.as_deref() == Some("Gamma"),
        );
        // ...and cleared, so the next thing typed is the reader's own query rather than this one.
        view.set_query(cx, "");
        let wide = view.rows();
        println!(
            "SEARCHING cleared shown={} still_selected={:?}",
            wide.len(),
            view.selected_text()
        );
    }

    /// Fill the number inputs on the numbers page, and print what each holds after it disagreed with its input.
    ///
    /// **Every case prints the value the widget landed on, not the value it was sent** — because three of the four are
    /// cases where those differ, and a print of the input would show the disagreement as an absence of one.
    fn seed_numbers(&mut self, cx: &mut Cx) {
        use makepad_component::mp::number_input::{format_number, nudge, MpNumberInputWidgetRefExt};

        // (id, value sent, min sent, max sent, step, decimals)
        let cases = [
            (ids!(numbers_plain), 5.0, 0.0, 100.0, 1.0, 0usize),
            (ids!(numbers_fractional), 2.5, 0.0, 10.0, 0.5, 2),
            (ids!(numbers_clamped), 999.0, 0.0, 10.0, 1.0, 0),
            // **The bounds the wrong way round**, which is the case the v2 widget panicked on.
            (ids!(numbers_backwards), 5.0, 100.0, 0.0, 1.0, 0),
        ];
        for (id, value, min, max, step, decimals) in cases {
            let view = self.ui.mp_number_input(cx, id);
            view.set_bounds(cx, min, max, step, decimals);
            view.set_value(cx, value);
            let (low, high, held_step, held_decimals) = view
                .borrow()
                .map(|inner| inner.bounds())
                .unwrap_or((min, max, step, decimals));
            let held = view.value();
            // One step up from where it landed, so the page's own output shows the grid and the bound in the same line.
            let up = nudge(held, low, high, held_step, 1.0);
            println!(
                "NUMBERS sent={} held={} clamped={} bounds=({low}, {high}) step={held_step} shown={:?} up={}",
                format_number(value, held_decimals),
                format_number(held, held_decimals),
                (held - value).abs() > f64::EPSILON,
                format_number(held, held_decimals),
                format_number(up, held_decimals),
            );
        }
    }

    /// Fill the step indicators on the steps page, and print where each stands.
    ///
    /// The three cases are the current step at the **start**, in the **middle**, and past the end in the truncating case
    /// — because the connector rule is only visible when there are steps on both sides of the current one.
    fn seed_steps(&mut self, cx: &mut Cx) {
        use makepad_component::mp::step_indicator::{
            connector_passed, state_of, steps_shown, MpStepIndicatorWidgetRefExt, StepState, SLOTS,
        };

        let five: Vec<String> = ["Cart", "Address", "Payment", "Review", "Done"]
            .iter()
            .map(|title| title.to_string())
            .collect();
        let ten: Vec<String> = (1..=SLOTS + 2)
            .map(|index| format!("Step {index}"))
            .collect();

        for (id, titles, current) in [
            (ids!(steps_third), &five, 2usize),
            (ids!(steps_first), &five, 0usize),
            (ids!(steps_over), &ten, SLOTS + 1),
        ] {
            let view = self.ui.mp_step_indicator(cx, id);
            view.set_items(cx, titles);
            view.set_step(cx, current);
            // **Every step's state and every connector's**, printed: the two rules are different and a page that only
            // showed the picture could not say which was in force.
            let states: Vec<&str> = (0..titles.len())
                .map(|index| match state_of(index, current) {
                    StepState::Passed => "passed",
                    StepState::Active => "active",
                    StepState::Upcoming => "upcoming",
                })
                .collect();
            let connectors: Vec<bool> = (0..titles.len())
                .map(|index| connector_passed(index, current))
                .collect();
            println!(
                "STEPS offered={} shown={} current={current} states={states:?} connectors={connectors:?}",
                titles.len(),
                steps_shown(titles.len()),
            );
        }
    }

    /// Fill the description lists on the details page, and print what each holds.
    ///
    /// The three cases are the **bound** from both sides — inside it, exactly at it, and past it — because the past-it
    /// case is the one a caller meets by accident and the page is the only place it is visible.
    fn seed_details(&mut self, cx: &mut Cx) {
        use makepad_component::mp::description_list::{DescriptionItem, MpDescriptionListWidgetRefExt, SLOTS};

        let four: Vec<DescriptionItem> = vec![
            // The tuple form, which is what a caller with literal rows writes — the `From` impl is what makes it work,
            // and `"x".into()` inside a tuple does not (the tuple's element type is uninferred).
            ("Operating system", "macOS 26").into(),
            ("Renderer", "Metal").into(),
            ("Toolkit", "Makepad").into(),
            ("License", "MIT OR Apache-2.0").into(),
        ];

        let full: Vec<DescriptionItem> = (0..SLOTS)
            .map(|index| DescriptionItem::new(format!("Row {}", index + 1), format!("Value {}", index + 1)))
            .collect();

        let over: Vec<DescriptionItem> = (0..SLOTS + 2)
            .map(|index| {
                if index == 0 {
                    // Long enough to wrap, so the label and value columns can be seen lining up across two lines.
                    DescriptionItem::new(
                        "Release notes",
                        "A value long enough to wrap onto a second line, which is where a row's alignment \
                         either holds or does not.",
                    )
                } else {
                    DescriptionItem::new(format!("Row {}", index + 1), format!("Value {}", index + 1))
                }
            })
            .collect();

        for (id, items) in [
            (ids!(details_four), &four),
            (ids!(details_full), &full),
            (ids!(details_over), &over),
        ] {
            let view = self.ui.mp_description_list(cx, id);
            view.set_items(cx, items);
            // **What the widget was given and what it will show**, because the bound is the whole point of the page and a
            // list that silently truncates looks exactly like one that fits.
            println!(
                "DETAILS offered={} shown={} slots={SLOTS}",
                items.len(),
                makepad_component::mp::description_list::rows_shown(items.len()),
            );
        }
    }

    /// **The primitive phase 6 needs, proven at runtime.**
    ///
    /// The A2UI renderer builds widgets **from Rust** into a pool, because how many a surface needs is not known when
    /// the DSL is written. That was recorded as the thing making a v3 port *architectural* — you would have to
    /// instantiate a `script_mod!` widget dynamically, which was assumed not to exist.
    ///
    /// It exists, and this is it:
    ///
    /// - `cx.with_vm(T::script_new_with_default)` — a widget from Rust with its type's defaults, no DSL declaration
    /// - `script_apply_eval!(cx, value, { ... })` — apply DSL properties to it
    /// - `T::script_from_value(vm, vm.bx.heap.value(vm.module(id!(widgets)), id, NoTrap))` — a widget from a
    ///   `script_mod!` **prototype by id** (the A2UI surface's `new_from_mod`)
    ///
    /// All three were already used by the renderer against **v2** widgets. What was never checked is whether a **v3**
    /// (`mp`) widget answers the same three calls. So this checks it, and reads a value back through a public getter:
    /// a property applied in script that comes back out in Rust is the whole path — instantiation, script
    /// application, and the value landing.
    ///
    /// A `Cx` is needed for all of it, so this cannot be a unit test; it runs in the app and prints, which is why the
    /// gate is the run's own output and not the green build.
    fn seed_v3_pool(&mut self, cx: &mut Cx) {
        use makepad_component::mp::button::{MpButton, MpButtonStyle};

        // (1) From Rust, with the type's defaults — no DSL declaration anywhere.
        let mut button = cx.with_vm(MpButton::script_new_with_default);
        // (2) A DSL property applied to it, including an enum from the shared `mod.mp` module.
        script_apply_eval!(cx, button, {
            style: mod.mp.ButtonStyle.Ghost
            text: "from rust"
        });
        // (3) Read back through the public getter: if the script application had silently done nothing, this is
        // where it shows, because the default is `Default` and not `Ghost`.
        let landed = button.style() == MpButtonStyle::Ghost;
        println!(
            "V3POOL script_new_with_default=MpButton ok=true style_applied_from_script={landed} style={:?}",
            button.style()
        );

        // And the same widget from a `script_mod!` prototype by id, which is the other half: `mp` widgets are
        // registered under `mod.mp`, not `mod.widgets`, so the module has to be named.
        let from_mod: Option<MpButton> = cx.with_vm(|vm| {
            let mp = vm.module(id!(mp));
            let value = vm.bx.heap.value(mp, id!(MpButton).into(), NoTrap);
            if value.is_err() {
                return None;
            }
            Some(MpButton::script_from_value(vm, value))
        });
        match from_mod {
            Some(button) => println!(
                "V3POOL instantiate_from_prototype mod.mp.MpButton ok=true style={:?}",
                button.style()
            ),
            None => println!("V3POOL instantiate_from_prototype mod.mp.Button ok=false"),
        }

        // A second widget, and a different kind: a `#[live]` numeric property read back on a widget that is not a
        // button. One widget working could be that widget's quirk.
        let mut slider = cx.with_vm(makepad_component::mp::slider::MpSlider::script_new_with_default);
        script_apply_eval!(cx, slider, {
            min: 5.0
            max: 25.0
        });
        println!(
            "V3POOL script_new_with_default=MpSlider ok=true min={} max={}",
            slider.min(),
            slider.max()
        );
    }

    /// Parse a ```chart fence and drive a real plot with what it produced.
    ///
    /// The **fence drives the plot**, which is the whole point of the seam: the block returns series and the page
    /// hands them to a widget it declared. And a second fence that is **prose** is shown **declining**, because a
    /// block that claimed every fence would turn a document's shell session into an empty chart.
    fn seed_blocks(&mut self, cx: &mut Cx) {
        use makepad_blocks::{self as blocks, Block};
        use makepad_plot::LinePlotWidgetRefExt;

        let fence = "title: Monthly users\n\
                     xlabel: Month\n\
                     ylabel: Active users\n\
                     series: Desktop\n\
                     1, 12\n\
                     2, 19\n\
                     3, 27\n\
                     4, 31\n\
                     5, 44\n\
                     series: Mobile\n\
                     1, 5\n\
                     2, 14\n\
                     3, 22\n\
                     4, 38\n\
                     5, 51\n";

        let plot = self.ui.line_plot(cx, ids!(plot));
        plot.clear();

        let mut line = String::new();
        let parsed = match blocks::render("chart", fence) {
            Some(Block::Chart(chart)) => {
                if let Some(title) = &chart.title {
                    plot.set_title(title.clone());
                }
                if let Some(label) = &chart.x_label {
                    plot.set_xlabel(label.clone());
                }
                if let Some(label) = &chart.y_label {
                    plot.set_ylabel(label.clone());
                }
                println!(
                    "BLOCKS chart title={:?} xlabel={:?} ylabel={:?} series={}",
                    chart.title,
                    chart.x_label,
                    chart.y_label,
                    chart.series.len()
                );
                for series in &chart.series {
                    println!(
                        "BLOCKS   series {:?} points={} x={:?} y={:?}",
                        series.label,
                        series.x.len(),
                        series.x,
                        series.y
                    );
                    line.push_str(&format!(
                        "{}: {} points, y {} to {}\n",
                        series.label,
                        series.x.len(),
                        series.y.first().copied().unwrap_or(0.0),
                        series.y.last().copied().unwrap_or(0.0)
                    ));
                }
                chart.series
            }
            _ => Vec::new(),
        };
        for series in parsed {
            plot.add_series(series);
        }

        // The fence itself, painted as code.
        self.ui
            .mp_code_block(cx, ids!(blocks_fence))
            .set_highlighted(cx, fence, &[]);

        let languages = blocks::languages().join(", ");
        self.ui.label(cx, ids!(blocks_stats)).set_text(
            cx,
            &format!("tags the router answers to: {languages}\n{line}"),
        );

        // **The decliner.** A fence for a tag nothing claims, and a fence for `chart` whose body is prose: both come
        // back as `None`, so the caller renders the fence's own text — which is what a document does when it has
        // nothing better, and the reason a `chart` fence is safe to enable everywhere.
        let prose = "Just a paragraph of prose that happens to be fenced.\nNo numbers, no series.";
        let unclaimed = blocks::render("mermaid", "graph TD\nA-->B\n");
        let declined = blocks::render("chart", prose);
        println!(
            "BLOCKS unclaimed_tag_is_none={} prose_chart_is_none={}",
            unclaimed.is_none(),
            declined.is_none()
        );
        self.ui.label(cx, ids!(blocks_declined)).set_text(
            cx,
            &format!(
                "a `mermaid` fence answers None ({}), and a `chart` fence whose body is **prose** answers None too \
                 ({}). A block that claimed every fence would turn a document's shell session into an empty chart.",
                unclaimed.is_none(),
                declined.is_none()
            ),
        );
    }

    /// Declare a keymap and fill the sheet from it.
    ///
    /// **Nothing below writes a chord.** Each row's trailing text is
    /// `Keymap::label(action, platform)`, so the two columns are two views of one
    /// declaration and a rebinding moves both at once.
    fn seed_keys(&mut self, cx: &mut Cx) {
        use makepad_component::mp::keys::{Keymap, Platform};

        let mut keys = Keymap::new();
        for (action, text) in [
            ("file.new", "cmd+n"),
            ("file.open", "cmd+o"),
            ("file.save", "cmd+s"),
            ("file.saveAs", "shift+cmd+s"),
            ("edit.undo", "cmd+z"),
            ("edit.redo", "shift+cmd+z"),
            ("edit.find", "cmd+f"),
            ("view.palette", "shift+cmd+p"),
            ("view.split", "cmd+d"),
            ("view.terminal", "ctrl+`"),
            ("nav.file", "cmd+p"),
            ("nav.line", "cmd+l"),
            ("window.close", "cmd+w"),
        ] {
            keys.declare(action, text).expect("every binding is a chord");
        }

        // The labels are resolved, so these lists cannot drift from the bindings.
        let rows = |platform: Platform| -> Vec<ListItem> {
            keys.bindings()
                .iter()
                .map(|(action, _)| {
                    ListItem::new(action)
                        .detail(keys.label(action, platform).unwrap_or_else(|| "(unbound)".into()))
                })
                .collect()
        };
        self.ui
            .mp_list(cx, ids!(keys_mac))
            .set_items(cx, rows(Platform::Macos));
        self.ui
            .mp_list(cx, ids!(keys_other))
            .set_items(cx, rows(Platform::Other));

        // A deliberately broken keymap, because a report that prints nothing is
        // indistinguishable from one that does not work.
        let mut broken = Keymap::new();
        for (action, text) in [
            ("view.split", "cmd+shift+d"),
            ("edit.duplicate", "\u{21e7}\u{2318}D"),
            ("file.save", "cmd+s"),
            ("file.saveAll", "cmd+s"),
            ("nav.next", "ctrl+tab"),
            ("tab.next", "ctrl+tab"),
        ] {
            broken
                .declare(action, text)
                .expect("every binding is a chord");
        }
        let found = broken.conflicts();
        let conflicts: Vec<ListItem> = found
            .iter()
            .map(|(left, right, chord)| {
                ListItem::new(format!("{left}  \u{2194}  {right}")).detail(chord.clone())
            })
            .collect();
        println!(
            "KEYS declared {} in the clean map ({} conflicts), {} in the broken map ({} conflicts)",
            keys.len(),
            keys.conflicts().len(),
            broken.len(),
            found.len()
        );
        for (left, right, chord) in &found {
            println!("KEYS conflict {chord}: {left} and {right}");
        }
        self.ui
            .mp_list(cx, ids!(keys_conflicts))
            .set_items(cx, conflicts);

        self.ui.label(cx, ids!(keys_note)).set_text(
            cx,
            &format!(
                "{} actions declared \u{b7} {} conflicts \u{b7} file.saveAs prints {} on macOS and {} elsewhere, from one binding",
                keys.len(),
                keys.conflicts().len(),
                keys.label("file.saveAs", Platform::Macos).unwrap_or_default(),
                keys.label("file.saveAs", Platform::Other).unwrap_or_default(),
            ),
        );
    }

    /// Give the calendars a real today and an initial selection.
    ///
    /// **The app reads the clock, not the library.** `SystemTime` gives seconds since
    /// the epoch; this divides by 86400 to get a day count and hands it to the date
    /// module's inverse. That keeps the arithmetic — the part that can be wrong —
    /// testable, and it keeps "today" the app's business rather than a widget's.
    fn seed_date(&mut self, cx: &mut Cx) {
        use makepad_component::mp::date::{date_from_epoch_seconds, YearMonth};
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        // **Not `seconds / 86400`.** That is the UTC day, and on this machine (UTC+8)
        // it is the *previous* day for the first eight hours of every local day: the
        // page printed `today=2026-09-16` while the system clock said `2026-09-17`.
        // A library cannot know the zone, so it takes the offset as a parameter and
        // this is where it comes from — the OS, once, at seed time.
        let offset = local_utc_offset_seconds();
        let (year, month, day) = date_from_epoch_seconds(seconds, offset);
        let today = (year, month, day);
        // A fixed selection, so the page shows a plate as well as a ring and the two
        // are distinguishable.
        let selected = (2026, 9, 9);

        // Every calendar gets the same today and selection, including the
        // Monday-first one — which had no id at all in the first version of the page,
        // so it was never seeded and opened on January 1970 while the others showed
        // September 2026. Nothing errored; the month was just a different one.
        for path in [ids!(cal_sunday), ids!(cal_monday), ids!(cal_worst)] {
            self.ui.mp_date(cx, path).set_today(cx, Some(today));
            self.ui.mp_date(cx, path).set_selected(cx, Some(selected));
        }
        self.ui
            .mp_date(cx, ids!(cal_worst))
            .show(cx, YearMonth::new(2026, 5));
        for path in [ids!(cal_sunday), ids!(cal_monday)] {
            self.ui.mp_date(cx, path).show(cx, YearMonth::new(2026, 9));
        }

        println!(
            "DATE today={year:04}-{month:02}-{day:02} (offset {offset:+}s, epoch second {seconds}) selected={selected:?}"
        );
        self.ui.label(cx, ids!(cal_readout)).set_text(
            cx,
            &format!("selected = {:04}-{:02}-{:02}", selected.0, selected.1, selected.2),
        );
        self.ui.label(cx, ids!(cal_today_note)).set_text(
            cx,
            &format!("today = {year:04}-{month:02}-{day:02} (zone offset {offset:+}s)"),
        );
    }

    /// Follow the calendar's month arrows and day clicks on the page's readout.
    fn handle_date(&mut self, cx: &mut Cx, actions: &Actions) {
        for path in [ids!(cal_sunday), ids!(cal_monday), ids!(cal_worst)] {
            if let Some((y, m, d)) = self.ui.mp_date(cx, path).day_selected(actions) {
                println!("DATE selected {y:04}-{m:02}-{d:02}");
                self.ui
                    .label(cx, ids!(cal_readout))
                    .set_text(cx, &format!("selected = {y:04}-{m:02}-{d:02}"));
                // The other calendar follows, so the two grids never disagree about
                // which day is chosen.
                for other in [ids!(cal_sunday), ids!(cal_monday), ids!(cal_worst)] {
                    self.ui.mp_date(cx, other).set_selected(cx, Some((y, m, d)));
                }
            }
            if let Some(view) = self.ui.mp_date(cx, path).view_changed(actions) {
                println!("DATE view changed to {:04}-{:02}", view.year, view.month);
            }
        }
    }

    /// Report a segmented control's selection, so the page proves the action path
    /// rather than only the paint.
    fn handle_segmented(&mut self, cx: &mut Cx, actions: &Actions) {
        for (path, name) in [
            (ids!(view_mode), "view mode"),
            (ids!(seg_wide), "wide"),
            (ids!(seg_small), "small"),
        ] {
            if let Some(index) = self.ui.mp_segmented(cx, path).selected(actions) {
                println!("SEGMENTED {name} -> {index}");
                if name == "wide" {
                    let label = self
                        .ui
                        .mp_segmented(cx, path)
                        .borrow()
                        .and_then(|inner| inner.segments().get(index).cloned())
                        .unwrap_or_default();
                    self.ui
                        .label(cx, ids!(seg_readout))
                        .set_text(cx, &format!("Selected({index}) = {label:?}"));
                }
            }
        }
    }

    /// Re-rank the palette, remap the cursor, and select it.
    ///
    /// The single place the palette's state changes — the `changed` handler and the
    /// `GALLERY_PALETTE_QUERY` seed both call it, so the check a run performs is the
    /// behaviour a keystroke gets rather than a second path that resembles it.
    fn apply_palette(&mut self, cx: &mut Cx, query: &str) {
        self.palette_ranked =
            makepad_component::mp::search::rank(&self.palette_commands, query);
        // By identity: the original index the cursor was on, if it survived.
        let position =
            makepad_component::mp::palette::remap(self.palette_active, &self.palette_ranked);
        self.palette_active = position
            .and_then(|p| makepad_component::mp::palette::original(&self.palette_ranked, p));

        let items: Vec<ListItem> = self
            .palette_ranked
            .iter()
            .map(|i| ListItem::new(&self.palette_commands[*i]))
            .collect();
        self.ui.mp_list(cx, ids!(palette_list)).set_items(cx, items);
        if let Some(p) = position {
            self.palette_highlight = Some(p);
            self.ui.mp_list(cx, ids!(palette_list)).select(cx, p);
        } else {
            self.palette_highlight = None;
        }

        // The evidence, as a line of stdout rather than a reading of pixels: a
        // screenshot cannot show which *original* command a cursor is on, and that
        // is the only part of this that can go wrong invisibly.
        let names: Vec<&str> = self
            .palette_ranked
            .iter()
            .map(|i| self.palette_commands[*i].as_str())
            .collect();
        println!(
            "PALETTE query={query:?} ranked={names:?} cursor_pos={position:?} active_original={:?}",
            self.palette_active
        );
    }

    /// Fill every segmented control on the pages that have one.
    ///
    /// Segments come from Rust, like `MpList`'s rows: a list of labels is not
    /// expressible as a DSL literal.
    fn seed_segmented(&mut self, cx: &mut Cx) {
        let labels = |names: &[&str]| names.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        self.ui
            .mp_segmented(cx, ids!(view_mode))
            .set_segments(cx, labels(&["Source", "Split", "Preview"]));
        self.ui.mp_segmented(cx, ids!(view_mode)).set_active(cx, Some(1));
        self.ui
            .mp_segmented(cx, ids!(seg_wide))
            .set_segments(cx, labels(&["Day", "Week", "Month", "Year"]));
        self.ui.mp_segmented(cx, ids!(seg_wide)).set_active(cx, Some(2));
        self.ui
            .mp_segmented(cx, ids!(seg_small))
            .set_segments(cx, labels(&["On", "Off"]));
        self.ui.mp_segmented(cx, ids!(seg_small)).set_active(cx, Some(0));
    }

    /// Fill the palette and, if asked, drive it from the environment.
    fn seed_palette(&mut self, cx: &mut Cx) {
        const COMMANDS: [&str; 10] = [
            "New Terminal",
            "Toggle Terminal",
            "Go to File",
            "Command Palette",
            "Split Right",
            "Delete",
            "Duplicate",
            "Move to Space…",
            "Rename…",
            "Close Window",
        ];
        self.palette_commands = COMMANDS.iter().map(|s| s.to_string()).collect();
        let query = std::env::var("GALLERY_PALETTE_QUERY").unwrap_or_default();
        if !query.is_empty() {
            self.ui
                .text_input(cx, ids!(palette_field))
                .set_text(cx, &query);
        }
        // Move the cursor off the first row before applying, when the environment
        // asks for it, so a run can show the cursor *following a row* rather than
        // merely starting on it. This is the case the module exists for.
        if std::env::var("GALLERY_PALETTE_CURSOR_DOWN").is_ok() && !query.is_empty() {
            let first_view =
                makepad_component::mp::search::rank(&self.palette_commands, &query);
            if first_view.len() > 1 {
                self.palette_active = Some(first_view[1]);
            }
        }
        self.apply_palette(cx, &query);
        // A second query, to show the cursor **following a row through a shrink** —
        // the case `mp/palette.rs` exists for. `de` leaves the cursor on
        // `Duplicate` at position 1; `du` filters every row before it away, so the
        // position has to move to 0 while the command stays `Duplicate`. A cursor
        // held as a position would have stayed at 1 and been out of range.
        if let Ok(second) = std::env::var("GALLERY_PALETTE_QUERY2") {
            println!(
                "PALETTE before-second-query cursor_pos={:?} active_original={:?}",
                self.palette_ranked.iter().position(|i| Some(*i) == self.palette_active),
                self.palette_active
            );
            self.ui
                .text_input(cx, ids!(palette_field))
                .set_text(cx, &second);
            self.apply_palette(cx, &second);
        }
        let active = self
            .palette_active
            .map(|i| self.palette_commands[i].clone())
            .unwrap_or_else(|| "—".to_string());
        self.ui.label(cx, ids!(palette_state)).set_text(
            cx,
            &format!(
                "query {:?} · {} of {} commands · cursor on {:?}",
                query,
                self.palette_ranked.len(),
                self.palette_commands.len(),
                active
            ),
        );
        self.ui.label(cx, ids!(palette_cursor_note)).set_text(
            cx,
            "cursor_pos is the position in the filtered view; active_original is the command it stands for.              A selection is reported as the latter, always.",
        );
    }

    /// The palette's live filter.
    fn handle_palette(&mut self, cx: &mut Cx, actions: &Actions) {
        let field = self.ui.text_input(cx, ids!(palette_field));
        if let Some(text) = field.changed(actions) {
            self.apply_palette(cx, &text);
        }
        // A row was selected. Only a **click** counts, and the check has to be by
        // position against what this app set itself.
        //
        // `MpList::select` emits the same `Selected` action a click does, and the
        // action carries a **position**, not an identity. So a caller that
        // re-filters and re-selects in one pass reads its own highlight back — and
        // if the view shrank in between, the position it reads is stale and can be
        // out of range. That is not hypothetical: the run that proved the cursor
        // survives a shrink ended with `chosen_by_click original=None`, because a
        // position of 1 arrived after the view had gone from three rows to one.
        //
        // Filtering by identity (comparing originals) is not enough — `None` is not
        // equal to the current cursor either. The only sound check is against the
        // position this app asked for.
        if let Some(position) = self.ui.mp_list(cx, ids!(palette_list)).row_selected(actions) {
            if Some(position) == self.palette_highlight {
                // Our own highlight coming back. Not a choice.
            } else if let Some(selected) =
                makepad_component::mp::palette::original(&self.palette_ranked, position)
            {
                self.palette_active = Some(selected);
                self.palette_highlight = Some(position);
                println!("PALETTE chosen_by_click position={position} original={selected:?}");
            }
        }
    }

    /// Echo what the text field reports, which is the page's own check that the
    /// action path reaches an app rather than only the widget.
    fn handle_input(&mut self, cx: &mut Cx, actions: &Actions) {
        let field = self.ui.text_input(cx, ids!(echo_input));
        if let Some(text) = field.changed(actions) {
            self.ui
                .label(cx, ids!(echo_out))
                .set_text(cx, &format!("Changed(\"{text}\")"));
        } else if let Some((text, _)) = field.returned(actions) {
            self.ui
                .label(cx, ids!(echo_out))
                .set_text(cx, &format!("Returned(\"{text}\")"));
        }
    }

    /// Open a popover without a pointer.
    ///
    /// The same affordance as `GALLERY_TOOLTIP`, and needed for a stronger
    /// reason: a synthetic pointer produces **no** hit at all in this app —
    /// every `event.hits` in a run reports `Nothing`, for every control, over
    /// 100k calls — so a click cannot be delivered and the popover's own path is
    /// unreachable from a capture script. This exercises the widget: the state,
    /// the anchor, the overlay pass and the plate.
    fn pin_popover(&mut self, cx: &mut Cx) {
        if !self.want_popover {
            return;
        }
        // Every (trigger, panel) pair any page declares. Only the current page's
        // widgets are laid out, so the others' triggers have empty rects and are
        // skipped — which is what lets one environment variable serve every page
        // rather than one variable per page.
        const PINS: [(&[LiveId], &[LiveId]); 9] = [
            (ids!(pop_form), ids!(pop_form_panel)),
            (ids!(pop_menu), ids!(pop_menu_panel)),
            (ids!(pop_tall), ids!(pop_tall_panel)),
            (ids!(select_face_a), ids!(select_panel_a)),
            (ids!(select_face_b), ids!(select_panel_b)),
            (ids!(select_face_c), ids!(select_panel_c)),
            // Only the first menu on the menu page: its panel is the tallest of
            // the three, and pinning all three at once overlays them so completely
            // that the destructive row at its bottom cannot be seen — which makes
            // the capture useless for the one thing that page exists to check.
            (ids!(menu_face_a), ids!(menu_panel_a)),
            (ids!(combo_field), ids!(combo_panel)),
            (ids!(hover_card_anchor), ids!(hover_card)),
        ];
        // **Re-asserted on every event, not fired once.** A popover closes on any
        // press outside its panel, and a capture run is not a clean room: raising
        // the window, or a stray synthetic press, would dismiss it before the
        // screenshot. Firing once made the capture non-deterministic — the panel
        // present in one run and gone in the next — and a verification affordance
        // that only sometimes works is worse than none. `open_for` on an
        // already-open popover is a no-op beyond re-anchoring.
        for (trigger, panel) in PINS {
            let area = self.ui.widget(cx, trigger).area();
            let rect = area.rect(cx);
            // An `Area` is empty until its widget has been laid out, and anchoring
            // to an empty rect puts the panel in the window corner.
            if rect.size.x > 0.0 && rect.size.y > 0.0 {
                self.ui.mp_popover(cx, panel).open_for(cx, area);
            }
        }
    }

    /// Install every page's data.
    ///
    /// Called once, on the first event — see `App::seeded` for why it cannot be
    /// `handle_startup`.
    fn seed_all(&mut self, cx: &mut Cx) {
        // **A permanent regression guard, not an experiment.** `MpSlider::value`
        // is `#[live]`, and this line is the only place in the gallery that drives
        // such a field from Rust — the readout beside it is seeded from the
        // widget's own value, so one screenshot proves the write reached both the
        // widget and the app. It was written to answer whether `#[live]` state can
        // hold a Rust write at all (it can, which refuted the first explanation
        // for the ring's lost value), and it stays because that is exactly the
        // property a regression would break silently.
        self.ui
            .mp_slider(cx, ids!(read_continuous))
            .set_value(cx, 0.9);
        self.seed_readouts(cx);
        self.seed_tables(cx);
        self.seed_trees(cx);
        self.seed_avatars(cx);
        self.seed_lists(cx);
        self.seed_selects(cx);
        self.seed_feedback(cx);
        self.seed_content(cx);
        self.seed_pagination(cx);
        self.seed_titlebars(cx);
        self.seed_menu_cards(cx);
        self.seed_search(cx);
        self.seed_palette(cx);
        self.seed_segmented(cx);
        self.seed_date(cx);
        self.seed_keys(cx);
        self.seed_history(cx);
        self.seed_combobox(cx);
        self.seed_hover_card(cx);
        self.seed_floating(cx);
        self.seed_code(cx);
        self.seed_document(cx);
        self.seed_editor(cx);
        self.seed_titlebars(cx);
        self.seed_menu_cards(cx);
        self.seed_picking(cx);
        self.seed_searching(cx);
        self.seed_numbers(cx);
        self.seed_steps(cx);
        self.seed_details(cx);
        self.seed_v3_pool(cx);
        self.seed_canvas(cx);
        self.seed_blocks(cx);
    }

    /// Fill the table page's tables.
    ///
    /// Rows come from Rust because that is where a table's data lives; a table
    /// declared in the DSL with a hundred literal rows would be a table nobody
    /// could use for anything.
    fn seed_tables(&mut self, cx: &mut Cx) {
        let table = self.ui.mp_table(cx, ids!(build_table));
        table.set_columns(
            cx,
            vec![
                TableColumn::new("space"),
                TableColumn::new("kind").width(90.0),
                TableColumn::new("priority").width(80.0).end(),
                TableColumn::new("count").width(70.0).end(),
            ],
        );
        let rows: Vec<Vec<String>> = [
            ("agent-workbench", "terminal", "high", "3"),
            ("nightly-sync", "cron", "low", "12"),
            ("cef-browsers", "browser", "high", "2"),
            ("design-notes", "note", "normal", "1"),
            ("receipts-2026-q1", "media", "low", "48"),
            ("a-very-long-space-name-that-must-clip-instead-of-overrunning", "note", "normal", "7"),
        ]
        .into_iter()
        .map(|(a, b, c, d)| vec![a.to_string(), b.to_string(), c.to_string(), d.to_string()])
        .collect();
        table.set_rows(cx, rows);

        // The empty table: columns and no rows, which is what a table looks like
        // before its data arrives.
        let empty = self.ui.mp_table(cx, ids!(empty_table));
        empty.set_columns(
            cx,
            vec![
                TableColumn::new("nothing").width(120.0),
                TableColumn::new("here yet"),
            ],
        );
        empty.set_rows(cx, Vec::new());
    }

    /// Fill the tree page's trees.
    ///
    /// The collapsed state is set from Rust for the same reason the rows are: it
    /// is data. A capture run cannot click a chevron — a synthetic pointer
    /// produces no hit at all in this app — so a page that could only be
    /// expanded by hand would show one state and assume the rest.
    fn seed_trees(&mut self, cx: &mut Cx) {
        let source = |label: &str, depth: usize| TreeItem::new(label, depth);

        // A small repository tree: two directories, files and a nested one.
        let repo = vec![
            source("src/", 0),
            source("tree.rs", 1),
            source("table.rs", 1),
            source("widgets/", 1),
            source("button.rs", 2),
            source("slider.rs", 2),
            source("tests/", 0),
            source("tree.rs", 1),
        ];
        self.ui.mp_tree(cx, ids!(full_tree)).set_items(cx, repo.clone());

        // The same list with `src/` shut and `tests/` open: the second subtree
        // must survive the first one's collapse.
        let collapsed = self.ui.mp_tree(cx, ids!(collapsed_tree));
        collapsed.set_items(cx, repo.clone());
        collapsed.set_collapsed(cx, 0, true);

        let deep = vec![
            source("workspace", 0),
            source("crates", 1),
            source("ui", 2),
            source("src", 3),
            source("mp", 4),
            source("tree.rs", 5),
            source("table.rs", 5),
            source("gallery", 3),
            source("pages", 4),
            source("tree.rs", 5),
        ];
        self.ui.mp_tree(cx, ids!(deep_tree)).set_items(cx, deep);

        let long = vec![
            source("a-very-long-directory-name-that-will-not-fit-in-its-row/", 0),
            source("an-equally-long-file-name-inside-it-that-also-cannot-fit.rs", 1),
            source("short.rs", 1),
        ];
        self.ui.mp_tree(cx, ids!(long_tree)).set_items(cx, long);
    }

    /// Give the avatar page's faces their names.
    ///
    /// `set_name` sets the initials *and* the plate, in one call, because the two
    /// cannot be allowed to disagree — which is the whole reason the tone is
    /// derived from the name rather than chosen.
    fn seed_avatars(&mut self, cx: &mut Cx) {
        for (path, name) in [
            (ids!(avatar_small), "Ada Lovelace"),
            (ids!(avatar_regular), "Grace Brewster Hopper"),
            (ids!(avatar_large), "Alan Turing"),
            (ids!(av_a), "Ada Lovelace"),
            (ids!(av_b), "Grace Hopper"),
            (ids!(av_c), "Alan Turing"),
            (ids!(av_d), "Barbara Liskov"),
            (ids!(av_e), "Ken Thompson"),
            // The second row repeats the first, in order: the page's own check
            // that a derived colour is stable rather than assigned.
            (ids!(av_a2), "Ada Lovelace"),
            (ids!(av_b2), "Grace Hopper"),
            (ids!(av_c2), "Alan Turing"),
            (ids!(av_d2), "Barbara Liskov"),
            (ids!(av_e2), "Ken Thompson"),
            (ids!(av_row_a), "Ada Lovelace"),
            (ids!(av_row_b), "Grace Brewster Hopper"),
            (ids!(av_row_c), "Alan Turing"),
        ] {
            self.ui.mp_avatar(cx, path).set_name(cx, name);
        }

        use makepad_component::mp::status::StatusTone;
        for (path, tone) in [
            (ids!(pres_off), StatusTone::Neutral),
            (ids!(pres_ok), StatusTone::Success),
            (ids!(pres_away), StatusTone::Warning),
            (ids!(pres_failed), StatusTone::Danger),
            (ids!(pres_busy), StatusTone::Busy),
        ] {
            self.ui.mp_avatar(cx, path).set_name(cx, "Ada Lovelace");
            self.ui.mp_avatar(cx, path).set_presence(cx, tone);
        }
        // The one with no dot at all: a person who is simply not tracked for
        // presence, which is different from one who is offline.
        self.ui.mp_avatar(cx, ids!(pres_none)).set_name(cx, "Alan Turing");

        // The **hand-written row**: five faces named by hand, which is what a DSL block can do. Its margins are literals
        // tuned at the 28pt face.
        let row = self.ui.view(cx, ids!(avatar_row));
        for (path, name) in [
            (ids!(one), "Ada Lovelace"),
            (ids!(two), "Grace Hopper"),
            (ids!(three), "Alan Turing"),
            (ids!(four), "Barbara Liskov"),
        ] {
            row.mp_avatar(cx, path).set_name(cx, name);
        }
        row.mp_avatar(cx, ids!(tail)).set_text(cx, "+3");

        // The **data-driven group**: the same faces from a list, with the overlap a ratio of the face. Two groups with the
        // same look and different jobs — and the difference is visible in what each can be told, not in how it draws.
        let names: Vec<String> = [
            "Ada Lovelace",
            "Grace Hopper",
            "Alan Turing",
            "Barbara Liskov",
            "Margaret Hamilton",
            "Katherine Johnson",
            "Edsger Dijkstra",
        ]
        .iter()
        .map(|name| name.to_string())
        .collect();
        let group = self.ui.mp_avatar_group(cx, ids!(avatar_group));
        group.set_avatars(cx, &names);
        group.set_limit(cx, 4);
        // Print what the group decided, because the decision is the component: **four faces and then `+3`** — five
        // circles, not four. A limit is not a count of circles.
        println!(
            "AVATARS group members={} limit={} circles={} from=mp::avatar_group::MpAvatarGroup",
            names.len(),
            group.limit(),
            group.circles(),
        );
    }

    /// Fill the list page's lists.
    fn seed_lists(&mut self, cx: &mut Cx) {
        // A sidebar: glyphs, one row carrying a detail.
        self.ui.mp_list(cx, ids!(sidebar_list)).set_items(
            cx,
            vec![
                ListItem::new("Terminals").glyph("\u{f120}").detail("4"),
                ListItem::new("Browsers").glyph("\u{f0ac}").detail("2"),
                ListItem::new("Notes").glyph("\u{f044}"),
                ListItem::new("Music").glyph("\u{f001}"),
                ListItem::new("Media").glyph("\u{f03e}").detail("48"),
                ListItem::new("Archive").glyph("\u{f187}"),
            ],
        );

        // A picker's options: no glyphs at all, and the labels keep the same
        // origin as the sidebar's.
        self.ui.mp_list(cx, ids!(options_list)).set_items(
            cx,
            vec![
                ListItem::new("Every day"),
                ListItem::new("Every week").detail("Mon"),
                ListItem::new("Every month").detail("1st"),
                ListItem::new("Never"),
            ],
        );

        // A command palette: glyph, name, shortcut at the far edge.
        self.ui.mp_list(cx, ids!(palette_list)).set_items(
            cx,
            vec![
                ListItem::new("Format Document").glyph("\u{f0d7}").detail("⇧⌥F"),
                ListItem::new("Toggle Terminal").glyph("\u{f120}").detail("⌃`"),
                ListItem::new("Go to File").glyph("\u{f002}").detail("⌘P"),
                ListItem::new("Save All").glyph("\u{f0c7}").detail("⌥⌘S"),
                ListItem::new("A command with a very long name so the label clips").glyph("\u{f013}").detail("⌃⇧⌘P"),
            ],
        );

        self.ui
            .mp_list(cx, ids!(empty_list))
            .set_items(cx, Vec::new());
    }

    /// Fill the select page's option lists, and show the value path.
    ///
    /// The third select is *selected programmatically* at startup, which is the
    /// only way to demonstrate the path a click would take: a synthetic pointer
    /// produces no hit at all in this app, so the selection cannot be delivered
    /// from a capture script. What this proves is the half that matters — the
    /// option reaching the face and the readout — and `mp/popover.rs` records why
    /// the other half is unreachable from here.
    fn seed_selects(&mut self, cx: &mut Cx) {
        let intervals: Vec<ListItem> = vec![
            ListItem::new("Every day"),
            ListItem::new("Every week").detail("Mon"),
            ListItem::new("Every month").detail("1st"),
            ListItem::new("Never"),
        ];
        for path in [ids!(select_list_a), ids!(select_list_b)] {
            self.ui.mp_list(cx, path).set_items(cx, intervals.clone());
        }
        let regions: Vec<ListItem> = vec![
            ListItem::new("Auto"),
            ListItem::new("us-east-1").detail("1"),
            ListItem::new("eu-west-1").detail("2"),
            ListItem::new("ap-northeast-1").detail("1"),
        ];
        self.ui
            .mp_list(cx, ids!(select_list_c))
            .set_items(cx, regions.clone());

        // The faces start with a value, as a form would.
        self.ui
            .mp_button(cx, ids!(select_face_a))
            .set_text(cx, "Every day");
        self.ui
            .mp_button(cx, ids!(select_face_b))
            .set_text(cx, "Auto");

        // The value path: select one, and show it reaching both the face and the
        // readout beside it.
        self.ui.mp_list(cx, ids!(select_list_c)).select(cx, 2);
        self.ui
            .mp_button(cx, ids!(select_face_c))
            .set_text(cx, "eu-west-1");
        self.ui
            .label(cx, ids!(select_readout))
            .set_text(cx, "chose eu-west-1 (option 2)");
    }

    /// Set the feedback page's ring values, and its one live one.
    fn seed_feedback(&mut self, cx: &mut Cx) {
        for (path, value) in [
            (ids!(ring_0), 0.0),
            (ids!(ring_25), 0.25),
            (ids!(ring_50), 0.5),
            (ids!(ring_75), 0.75),
            (ids!(ring_90), 0.9),
            (ids!(ring_100), 1.0),
            (ids!(ring_small), 0.66),
            (ids!(ring_regular), 0.66),
            (ids!(ring_large), 0.66),
            (ids!(ring_row), 0.42),
        ] {
            self.ui.mp_progress_ring(cx, path).set_value(cx, value);
        }
    }

    /// Set the content page's key caps and its group box's list.
    fn seed_content(&mut self, cx: &mut Cx) {
        // The stat row: four metrics with their trends. The tones are the same six
        // a badge uses, so a rise and a fall are the library's status vocabulary
        // rather than the arrow's colour.
        use makepad_component::mp::status::StatusTone;
        for (card, value, label, delta, tone, note) in [
            (ids!(stat_a), "12", "Terminals", "+3", StatusTone::Success, "since Monday"),
            (ids!(stat_b), "2m 14s", "Median build", "-8s", StatusTone::Success, "vs last week"),
            (ids!(stat_c), "4", "Failing tests", "+2", StatusTone::Danger, "on main"),
            (ids!(stat_d), "62%", "Cache hit rate", "—", StatusTone::Neutral, "no baseline"),
        ] {
            let card_ref = self.ui.widget(cx, card);
            card_ref.label(cx, ids!(stat_value)).set_text(cx, value);
            card_ref.label(cx, ids!(stat_label)).set_text(cx, label);
            card_ref.label(cx, ids!(stat_note)).set_text(cx, note);
            card_ref.mp_badge(cx, ids!(stat_delta)).set_text(cx, delta);
            card_ref.mp_badge(cx, ids!(stat_delta)).set_tone(cx, tone);
        }

        // A chord as three caps rather than one cap with a string in it: the caps
        // share a face and a height, so three of them read as keys pressed
        // together and one string reads as a label.
        self.ui.mp_kbd(cx, ids!(kbd_cmd)).set_text(cx, "⌘");
        self.ui.mp_kbd(cx, ids!(kbd_shift)).set_text(cx, "⇧");
        self.ui.mp_kbd(cx, ids!(kbd_p)).set_text(cx, "P");

        use makepad_component::mp::list::ListItem;
        self.ui
            .mp_list(cx, ids!(group_list))
            .set_items(
                cx,
                vec![
                    ListItem::new("agent-workbench").glyph("\u{f120}").detail("3"),
                    ListItem::new("cef-browsers").glyph("\u{f0ac}").detail("2"),
                    ListItem::new("nightly-sync").glyph("\u{f017}").detail("1"),
                ],
            );
    }

    /// Set the pagination page's four cases.
    ///
    /// From Rust, and that is deliberate: it exercises the setter path for a
    /// `#[live]`-declared field, which is the property
    /// `MpProgressRing` was found not to honour. A pagination whose page cannot be
    /// set from Rust would render its DSL page and look correct.
    fn seed_pagination(&mut self, cx: &mut Cx) {
        for (path, page, total, siblings) in [
            (ids!(pag_short), 3, 5, 1),
            (ids!(pag_first), 1, 20, 1),
            (ids!(pag_middle), 10, 20, 1),
            (ids!(pag_last), 20, 20, 1),
            (ids!(pag_wide), 10, 40, 2),
        ] {
            let p = self.ui.mp_pagination(cx, path);
            p.set_total(cx, total);
            p.set_page(cx, page);
            let _ = siblings;
        }
    }

    /// Fill the search page from `rank()` itself.
    ///
    /// The candidate set is a real palette's, chosen to contain every case the
    /// ranking rules were written for: a command with a two-word label, a
    /// scattered match across two words, a long contiguous run, and a
    /// path-shaped label whose separators are word starts.
    ///
    /// Running the function rather than copying its output into the page is
    /// deliberate. A page that shows the expected answer is a second copy of the
    /// tests; a page that calls the function shows what a reader will actually
    /// see, and regresses visibly if the function does.
    fn seed_search(&mut self, cx: &mut Cx) {
        const CANDIDATES: [&str; 8] = [
            "New Terminal",
            "Toggle Terminal",
            "Go to File",
            "Command Palette",
            "Split Right",
            "Rename\u{2026}",
            "Delete",
            "The remote endpoint",
        ];
        const OR_SET: [&str; 4] = ["Word", "Off Road", "Go to File", "Order"];
        let all: Vec<String> = CANDIDATES.iter().map(|s| s.to_string()).collect();
        let or_all: Vec<String> = OR_SET.iter().map(|s| s.to_string()).collect();

        for (path, candidates, query) in [
            (ids!(search_all), &all, ""),
            (ids!(search_nt), &all, "nt"),
            (ids!(search_term), &all, "term"),
            (ids!(search_or), &or_all, "or"),
        ] {
            let items: Vec<ListItem> = makepad_component::mp::search::rank(candidates, query)
                .into_iter()
                .map(|i| ListItem::new(&candidates[i]))
                .collect();
            self.ui.mp_list(cx, path).set_items(cx, items);
        }
    }

    /// Fill the menu page's three menus.
    ///
    /// Built from data, which is the point of the separator being a *row's* flag:
    /// a menu is assembled from a list of commands and their group starts, with no
    /// second structure for the sections and no call per line.
    fn seed_menus(&mut self, cx: &mut Cx) {
        self.ui.mp_list(cx, ids!(menu_list_a)).set_items(
            cx,
            vec![
                ListItem::new("New Terminal").glyph("\u{f120}").detail("⌘T"),
                ListItem::new("Split Right").glyph("\u{f0db}").detail("⌘D"),
                ListItem::new("Rename…").glyph("\u{f044}").starts_group(),
                ListItem::new("Duplicate").glyph("\u{f24d}").detail("⌘⇧D"),
                ListItem::new("Move to Space…").glyph("\u{f0c9}").starts_group(),
                ListItem::new("Delete").glyph("\u{f1f8}").detail("⌘⌫").destructive(),
            ],
        );

        self.ui.mp_list(cx, ids!(menu_list_b)).set_items(
            cx,
            vec![
                ListItem::new("Single").detail("⌘1"),
                ListItem::new("Two columns").detail("⌘2"),
                ListItem::new("Three columns").detail("⌘3"),
                ListItem::new("Focus mode").detail("⌘⇧F"),
            ],
        );

        self.ui.mp_list(cx, ids!(menu_list_c)).set_items(
            cx,
            vec![
                ListItem::new("Small"),
                ListItem::new("Regular"),
                ListItem::new("Large").starts_group(),
                ListItem::new("Fill the window"),
            ],
        );
    }

    /// Write each slider's starting value into its readout.
    fn seed_readouts(&mut self, cx: &mut Cx) {
        const PAIRS: [(&[LiveId], &[LiveId]); 7] = [
            (ids!(read_continuous), ids!(out_continuous)),
            (ids!(read_stepped), ids!(out_stepped)),
            (ids!(read_thirds), ids!(out_thirds)),
            (ids!(read_anchor), ids!(out_anchor)),
            (ids!(read_negative), ids!(out_negative)),
            (ids!(read_small), ids!(out_small)),
            (ids!(drive_source), ids!(drive_bar)),
        ];
        for (slider, readout) in PAIRS {
            let value = self.ui.mp_slider(cx, slider).value();
            if readout == ids!(drive_bar) {
                self.ui.mp_progress(cx, readout).set_value(cx, value);
            } else {
                self.ui
                    .label(cx, readout)
                    .set_text(cx, &format!("{value:.2}"));
            }
        }
    }

    /// Click a trigger to open or close its panel.
    ///
    /// The whole point of the popover page being verifiable where the tooltip's
    /// is not: a **click is synthesizable**, so a capture script can exercise
    /// this path end to end — the trigger's action, this listener, the anchor
    /// from an `Area`, and the plate. A hover is not, which is why the tooltip
    /// needs `GALLERY_TOOLTIP=1` instead.
    fn handle_popovers(&mut self, cx: &mut Cx, actions: &Actions) {
        const TRIGGERS: [(&[LiveId], &[LiveId]); 3] = [
            (ids!(pop_form), ids!(pop_form_panel)),
            (ids!(pop_menu), ids!(pop_menu_panel)),
            (ids!(pop_tall), ids!(pop_tall_panel)),
        ];
        for (trigger, panel) in TRIGGERS {
            if !self.ui.mp_button(cx, trigger).clicked(actions) {
                continue;
            }
            let popover = self.ui.mp_popover(cx, panel);
            if popover.is_open() {
                popover.close(cx);
            } else {
                // Anchored to the trigger that was clicked, so the panel is
                // placed from the widget's own `Area` rather than from a
                // rectangle this function computed.
                let area = self.ui.widget(cx, trigger).area();
                popover.open_for(cx, area);
            }
        }
    }

    /// The slider page's readouts, and the one slider that drives something.
    ///
    /// A readout per slider is the page's own test: it shows the *value* the
    /// widget reported rather than only that a knob moved, and the last pair
    /// shows the value leaving the slider and arriving at another widget.
    fn handle_sliders(&mut self, cx: &mut Cx, actions: &Actions) {
        const READOUTS: [(&[LiveId], &[LiveId]); 7] = [
            (ids!(read_continuous), ids!(out_continuous)),
            (ids!(read_stepped), ids!(out_stepped)),
            (ids!(read_thirds), ids!(out_thirds)),
            (ids!(read_anchor), ids!(out_anchor)),
            (ids!(read_negative), ids!(out_negative)),
            (ids!(read_small), ids!(out_small)),
            (ids!(drive_source), ids!(drive_bar)),
        ];
        for (slider, readout) in READOUTS {
            let Some(value) = self.ui.mp_slider(cx, slider).changed(actions) else {
                continue;
            };
            if readout == ids!(drive_bar) {
                self.ui.mp_progress(cx, readout).set_value(cx, value);
            } else {
                // The last pair has a progress bar where the others have a
                // label, so the branch is on what the path holds rather than on
                // an index nobody can read.
                self.ui
                    .label(cx, readout)
                    .set_text(cx, &format!("{value:.2}"));
            }
        }
    }

    /// Make `page` the visible one and mark its rail row selected.
    ///
    /// Visibility rather than rebuilding: a gallery page holds live widget
    /// state — the button page's counter, a scrolled position — and swapping
    /// the tree out would throw it away every time the rail moved.
    fn show(&mut self, cx: &mut Cx, page: usize) {
        let page = page.min(PAGES.len() - 1);
        self.page = page;
        for (index, slot) in PAGE_SLOTS.iter().enumerate() {
            self.ui.widget(cx, slot).set_visible(cx, index == page);
        }
        for (index, row) in RAIL_ROWS.iter().enumerate() {
            // The current page is a raised plate; the others are quiet text.
            // Two existing looks rather than a third "selected" one, because a
            // nav row's selected state is the same plate a selected row paints
            // anywhere else in the library.
            self.ui.mp_button(cx, row).set_style(
                cx,
                if index == page {
                    MpButtonStyle::Default
                } else {
                    MpButtonStyle::Ghost
                },
            );
        }
        self.ui
            .label(cx, ids!(rail_blurb))
            .set_text(cx, PAGES[page].blurb);
        self.ui.redraw(cx);
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        makepad_component::script_mod(vm);
        // The pages before the startup block, because the block instantiates
        // them by path.
        crate::pages::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if self.want_tooltip {
            // Wait for the trigger to have a real rectangle. An `Area` is empty
            // until the widget that owns it has been laid out, and the first
            // `handle_event` can arrive before the first draw — anchoring to an
            // empty rect puts the plate at the pass origin, which is what the
            // first working capture showed: a correct plate in the wrong place.
            let trigger = self.ui.widget(cx, ids!(tip_primary)).area();
            let rect = trigger.rect(cx);
            if rect.size.x > 0.0 && rect.size.y > 0.0 {
                self.want_tooltip = false;
                self.ui
                    .mp_tooltip(cx, ids!(tip))
                    .show_for(cx, trigger, "Runs the primary action");
            }
        }
        // **Both pins must be called here or they do nothing.** The tooltip's
        // was inlined at some point and this one's call was lost in an edit,
        // which cost a long hunt: with `GALLERY_POPOVER=1` set and no call, the
        // popover was never opened at all, so every "the panel does not draw"
        // conclusion drawn from it was about a widget that had not been asked to
        // do anything.
        self.pin_popover(cx);
        if !self.seeded {
            self.seeded = true;
            self.seed_all(cx);
        }
        self.match_event(cx, event);
        // The floating panel's drag, heard on the page's layer rather than on the panel's own
        // box — a pointer that outruns a frame lands outside the box, and a panel listening on
        // itself would stop hearing moves there.
        let layer = self.ui.widget(cx, ids!(float_layer)).area();
        self.ui.mp_floating(cx, ids!(float_panel)).drag(cx, event, layer);
        // Tab and Shift-Tab traversal. Makepad has a single key-focus area on
        // `Cx` and no traversal, so the app owns the pass.
        // The v3 traversal, which owns the one registry the v2 widgets also feed.
        makepad_component::mp::focus::handle_key(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

/// How far the local zone is ahead of UTC, in seconds.
///
/// The one thing the date module cannot be asked for, because it is not arithmetic —
/// it is a fact about where the machine is. Asked of the OS with `localtime_r`, which
/// is POSIX; a Windows build would need `_get_timezone`, and saying so here is better
/// than a silent zero.
fn local_utc_offset_seconds() -> i64 {
    // Safety: `localtime_r` writes into the `tm` this owns and reads a time this owns;
    // neither pointer escapes.
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&now, &mut tm).is_null() {
            // Falling back to UTC rather than to a guess. It is wrong for part of
            // every day, which is precisely why it is logged.
            return 0;
        }
        tm.tm_gmtoff as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_slot_tables_cover_every_page() {
        // `ids!` needs literals, so the slot tables are hand-maintained while
        // `PAGES` is data — which means adding a page without adding a slot
        // compiles, and the new page simply never becomes visible. This is what
        // turns that into a failing test instead.
        assert_eq!(
            PAGE_SLOTS.len(),
            PAGES.len(),
            "a page was added to or removed from PAGES without a slot"
        );
        assert_eq!(
            RAIL_ROWS.len(),
            PAGES.len(),
            "a page was added to or removed from PAGES without a rail row"
        );
    }

    /// The page each slot holds, in `PAGE_SLOTS` order.
    ///
    /// **Append new pages at the end of `PAGES`, `PAGE_SLOTS`, `RAIL_ROWS` and
    /// this table, and nowhere else.** Inserting one in the middle means editing
    /// four ordered lists, and a missed edit is invisible until a rail row opens
    /// the wrong page — which is what happened adding the Overlay page, twice in
    /// one change: the DSL slots and this table each disagreed with `PAGES`
    /// about which index held which page, and the test caught the second only
    /// after the first was fixed.
    ///
    /// The *DSL* is what actually wires a slot to a page — `page_5 :=
    /// mod.gallery.pages.layout{}` is a literal in a `script_mod!` block — so
    /// the only way to compare it against `PAGES` is to restate it here and
    /// assert the two agree. Without this the order can drift silently, and it
    /// did: `GALLERY_PAGE=Loaders` opened the Layout page, because the two
    /// lists disagreed about which slot was which.
    const SLOT_PAGES: [&str; 46] = [
        "mod.gallery.pages.palette",
        "mod.gallery.pages.typography",
        "mod.gallery.pages.metrics",
        "mod.gallery.pages.motion",
        "mod.gallery.pages.button",
        "mod.gallery.pages.layout",
        "mod.gallery.pages.loaders",
        "mod.gallery.pages.slider",
        "mod.gallery.pages.overlay",
        "mod.gallery.pages.input",
        "mod.gallery.pages.controls",
        "mod.gallery.pages.popover",
        "mod.gallery.pages.icon",
        "mod.gallery.pages.status",
        "mod.gallery.pages.table",
        "mod.gallery.pages.tree",
        "mod.gallery.pages.avatar",
        "mod.gallery.pages.surface",
        "mod.gallery.pages.list",
        "mod.gallery.pages.select",
        "mod.gallery.pages.feedback",
        "mod.gallery.pages.content",
        "mod.gallery.pages.pagination",
        "mod.gallery.pages.menu",
        "mod.gallery.pages.scroll",
        "mod.gallery.pages.search",
        "mod.gallery.pages.bars",
        "mod.gallery.pages.command_palette",
        "mod.gallery.pages.calendar",
        "mod.gallery.pages.shortcuts",
        "mod.gallery.pages.history",
        "mod.gallery.pages.combobox",
        "mod.gallery.pages.hover_card",
        "mod.gallery.pages.floating",
        "mod.gallery.pages.code",
        "mod.gallery.pages.document",
        "mod.gallery.pages.editor",
        "mod.gallery.pages.canvas",
        "mod.gallery.pages.blocks",
        "mod.gallery.pages.details",
        "mod.gallery.pages.steps",
        "mod.gallery.pages.numbers",
        "mod.gallery.pages.searching",
        "mod.gallery.pages.picking",
        "mod.gallery.pages.menu_cards",
        "mod.gallery.pages.titlebars",
    ];

    #[test]
    fn test_each_slot_holds_the_page_pages_says_it_does() {
        assert_eq!(SLOT_PAGES.len(), PAGES.len());
        for (index, page) in PAGES.iter().enumerate() {
            assert_eq!(
                SLOT_PAGES[index], page.path,
                "slot {index} holds {} but PAGES[{index}] is {:?}",
                SLOT_PAGES[index], page.title
            );
        }
    }

    #[test]
    fn test_no_two_slots_are_the_same_id() {
        // A copy-paste in the table would make two pages show at once, and the
        // second would paint over the first.
        let mut seen = std::collections::HashSet::new();
        for (index, slot) in PAGE_SLOTS.iter().enumerate() {
            assert!(seen.insert(slot[0]), "slot {index} is a duplicate");
        }
        let mut seen = std::collections::HashSet::new();
        for (index, row) in RAIL_ROWS.iter().enumerate() {
            assert!(seen.insert(row[0]), "rail row {index} is a duplicate");
        }
    }
}
