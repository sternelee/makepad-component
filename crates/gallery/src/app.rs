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
    switch::MpSwitchWidgetRefExt,
};

use crate::pages::PAGES;

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

                        ScrollYView{
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
const PAGE_SLOTS: [&[LiveId]; 24] = [
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
];

/// The gallery's DSL path for each rail row.
const RAIL_ROWS: [&[LiveId]; 24] = [
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
        const PINS: [(&[LiveId], &[LiveId]); 7] = [
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
        makepad_component::widgets::focus::handle_key(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
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
    const SLOT_PAGES: [&str; 24] = [
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
