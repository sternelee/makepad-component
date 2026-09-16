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

use makepad_widgets::*;

use makepad_component::mp::{
    button::{MpButtonStyle, MpButtonWidgetRefExt},
    checkbox::MpCheckboxWidgetRefExt,
    radio::MpRadioWidgetRefExt,
    loaders::MpProgressWidgetRefExt,
    popover::MpPopoverWidgetRefExt,
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
const PAGE_SLOTS: [&[LiveId]; 12] = [
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
];

/// The gallery's DSL path for each rail row.
const RAIL_ROWS: [&[LiveId]; 12] = [
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
        self.seed_readouts(cx);
        // `GALLERY_TOOLTIP=1` pins the overlay open, anchored to the first
        // trigger. Same justification as `GALLERY_PAGE`: Makepad exposes no
        // accessibility tree, so a capture script cannot hover a button, and an
        // overlay that can only be shown by a pointer cannot be verified from a
        // script. It is also the only way to see the *hardest* property this
        // page exists for — that the plate draws over the card below it.
        self.want_tooltip = std::env::var("GALLERY_TOOLTIP").is_ok();
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
    const SLOT_PAGES: [&str; 12] = [
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
