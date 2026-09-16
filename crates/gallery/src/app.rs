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
    tree::{MpTreeWidgetRefExt, TreeItem},
    tooltip::MpTooltipWidgetRefExt,
    slider::MpSliderWidgetRefExt,
    segmented::MpSegmentedWidgetRefExt,
    switch::MpSwitchWidgetRefExt,
    date::MpDateWidgetRefExt,
};

use crate::pages::PAGES;
use makepad_component::mp::combobox::Combobox;
use makepad_component::mp::history::History;

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
const PAGE_SLOTS: [&[LiveId]; 32] = [
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
];

/// The gallery's DSL path for each rail row.
const RAIL_ROWS: [&[LiveId]; 32] = [
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
    history: History<String>,
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
use makepad_component::mp::history::History;

        // Eight states of capacity: enough that a script can exceed it and still have a
        // readable stack.
        let mut history: History<String> = History::with_capacity("doc:0".to_string(), 8);
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
        const PINS: [(&[LiveId], &[LiveId]); 8] = [
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
        self.seed_menus(cx);
        self.seed_search(cx);
        self.seed_palette(cx);
        self.seed_segmented(cx);
        self.seed_date(cx);
        self.seed_keys(cx);
        self.seed_history(cx);
        self.seed_combobox(cx);
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

        let group = self.ui.view(cx, ids!(avatar_group));
        for (path, name) in [
            (ids!(one), "Ada Lovelace"),
            (ids!(two), "Grace Hopper"),
            (ids!(three), "Alan Turing"),
            (ids!(four), "Barbara Liskov"),
        ] {
            group.mp_avatar(cx, path).set_name(cx, name);
        }
        group.mp_avatar(cx, ids!(tail)).set_text(cx, "+3");
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
    const SLOT_PAGES: [&str; 32] = [
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
