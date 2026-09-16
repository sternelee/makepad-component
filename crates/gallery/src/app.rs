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
                            page_7 := mod.gallery.pages.controls{}
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
const PAGE_SLOTS: [&[LiveId]; 8] = [
    ids!(page_0),
    ids!(page_1),
    ids!(page_2),
    ids!(page_3),
    ids!(page_4),
    ids!(page_5),
    ids!(page_6),
    ids!(page_7),
];

/// The gallery's DSL path for each rail row.
const RAIL_ROWS: [&[LiveId]; 8] = [
    ids!(rail_page_0),
    ids!(rail_page_1),
    ids!(rail_page_2),
    ids!(rail_page_3),
    ids!(rail_page_4),
    ids!(rail_page_5),
    ids!(rail_page_6),
    ids!(rail_page_7),
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

        self.handle_controls(cx, actions);
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
    /// The *DSL* is what actually wires a slot to a page — `page_5 :=
    /// mod.gallery.pages.layout{}` is a literal in a `script_mod!` block — so
    /// the only way to compare it against `PAGES` is to restate it here and
    /// assert the two agree. Without this the order can drift silently, and it
    /// did: `GALLERY_PAGE=Loaders` opened the Layout page, because the two
    /// lists disagreed about which slot was which.
    const SLOT_PAGES: [&str; 8] = [
        "mod.gallery.pages.palette",
        "mod.gallery.pages.typography",
        "mod.gallery.pages.metrics",
        "mod.gallery.pages.motion",
        "mod.gallery.pages.button",
        "mod.gallery.pages.layout",
        "mod.gallery.pages.loaders",
        "mod.gallery.pages.controls",
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
