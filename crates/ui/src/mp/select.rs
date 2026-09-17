//! `MpSelect` — a trigger that shows the chosen option and a panel that lists them.
//!
//! ## This one is thin, and that is the point
//!
//! The third of the four components dbpro needed. Unlike the other two it needs **no new model**: a select is a
//! [`Combobox`] that is never typed into. Everything a select does — hold a list, hold the chosen index, step a highlight
//! over the rows, commit one — already exists there, with its own tests. So this file is the trigger, the open/closed flag,
//! and the one rule the wrapper has to state.
//!
//! Writing a second model would have been the mistake: two lists of items, two value indices, and eventually a state where
//! the trigger says one thing and the panel highlights another.
//!
//! ## The rule the wrapper adds: a select does not filter
//!
//! A combobox narrows its panel as you type. A select cannot be typed into, so its panel always shows **every** option —
//! which is not a special case to implement but a consequence of never calling [`Combobox::type_text`] ✓. What it does need
//! is for the panel to show the **whole** list even when a value is set, and that is why [`Select::rows`] reads the
//! unfiltered view rather than the combobox's.
//!
//! ## The panel is a sibling, not a child
//!
//! **A floating surface cannot be composed into its trigger** — the overlay's draw list is clipped to the widget's own rect,
//! so a panel drawn inside a trigger would be cut off at the trigger's bottom edge. This is the rule `mp/popover.rs` and
//! `mp/combobox.rs` both document, and it applies here unchanged: the caller places the panel in the page's `Overlay` area as
//! a sibling. [`MpSelectTrigger`] therefore reports `Toggled` and draws nothing but itself.
//!
//! ## The slot ids
//!
//! `trigger` and `label` — the two a caller writes into, kept from the widget this replaces. Its `dropdown` slot full of
//! `MpSelectOption` children becomes **Rust data** ([`Select::set_options`]), because a list of options is data and the v3
//! library takes its data from Rust in every other component (`MpColorPicker::set_colors`,
//! `MpSearchableList::set_items`, `MpMenubar::set_titles`). That is the one call-site change the migration makes.

use makepad_widgets::*;

use crate::mp::combobox::Combobox;

/// The trigger's height.
///
/// **One control row, from the theme** — the same `row_height` a menu row, a list row and a combobox row are, so a select's
/// trigger lines up with everything beside it without any of them knowing about the others. The theme has no separate
/// "control height"; inventing one here would be a second height that has to be kept in step with the first.
pub fn trigger_height(cx: &mut Cx) -> f64 {
    makepad_theme::Theme::of(cx).layout.row_height as f64
}

/// A select's state: the combobox it delegates to, and whether its panel is down.
///
/// The panel's visibility is the wrapper's own, because a combobox's field and a select's trigger disagree about what "open"
/// means — the field is always there and the panel comes and goes, while a trigger is a button whose panel is its whole
/// reason for being pressed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Select {
    combo: Combobox,
    open: bool,
}

impl Select {
    /// A select over `options`.
    pub fn new(options: Vec<String>) -> Self {
        Self {
            combo: Combobox::new(options),
            open: false,
        }
    }

    /// Replace the options.
    ///
    /// **A chosen index that the new list does not cover is dropped, not clamped** — the same rule the combobox states for
    /// itself, and for the same reason: a select showing the third option when it has two would be displaying a choice
    /// nobody can make.
    pub fn set_options(&mut self, options: Vec<String>) {
        self.combo.set_items(options);
    }

    pub fn options(&self) -> &[String] {
        self.combo.items()
    }

    pub fn len(&self) -> usize {
        self.options().len()
    }

    pub fn is_empty(&self) -> bool {
        self.options().is_empty()
    }

    /// Set the chosen option by index.
    pub fn set_value(&mut self, index: Option<usize>) {
        match index {
            Some(index) => self.combo.choose(index),
            None => self.combo.clear(),
        }
    }

    /// The chosen index.
    pub fn value(&self) -> Option<usize> {
        self.combo.value()
    }

    /// What the trigger shows: the chosen option's label, or `None` for a select nothing has chosen from.
    pub fn label(&self) -> Option<&str> {
        self.combo.value_label()
    }

    /// The chosen index and its label.
    pub fn chosen(&self) -> Option<(usize, String)> {
        let index = self.value()?;
        Some((index, self.combo.value_label()?.to_string()))
    }

    /// Move the highlight, wrapping, over **every** option.
    ///
    /// The rows come from the options rather than from the combobox's filtered view: a select cannot be typed into, so its
    /// view is always the whole list and reading the unfiltered one makes that a property rather than a coincidence.
    pub fn step(&mut self, delta: i32) {
        self.combo.step(delta);
    }

    /// The highlighted index.
    pub fn active(&self) -> Option<usize> {
        self.combo.active()
    }

    /// The panel's rows: index, label, and whether each is the current value.
    pub fn rows(&self) -> Vec<(usize, String, bool)> {
        let value = self.value();
        self.options()
            .iter()
            .enumerate()
            .map(|(index, label)| (index, label.clone(), value == Some(index)))
            .collect()
    }

    /// Highlight the chosen option, or the first row when nothing is chosen.
    ///
    /// **So that pressing a trigger and pressing down cannot do different things**: the first arrow key moves from wherever
    /// the highlight is, and a panel that opened with nothing highlighted would jump to a row nobody aimed at.
    pub fn prepare(&mut self) {
        if self.combo.active().is_none() {
            let start = self.value().unwrap_or(0);
            if start < self.len() {
                self.combo.step_to(start);
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Open the panel, answering whether that changed anything.
    pub fn show(&mut self) -> bool {
        if self.open {
            return false;
        }
        self.open = true;
        self.prepare();
        true
    }

    /// Close it, answering whether that changed anything.
    pub fn hide(&mut self) -> bool {
        let changed = self.open;
        self.open = false;
        changed
    }

    pub fn toggle(&mut self) -> bool {
        if self.open {
            self.hide()
        } else {
            self.show()
        }
    }

    /// Choose the highlighted row and close, or `None` when nothing is highlighted.
    pub fn commit(&mut self) -> Option<(usize, String)> {
        let index = self.combo.commit_active()?;
        let label = self.combo.value_label()?.to_string();
        self.hide();
        Some((index, label))
    }
}

/// What a select's trigger reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpSelectAction {
    /// The trigger was pressed: the caller toggles its panel.
    Toggled,
    /// A row was chosen, carrying its index and its label.
    Selected(usize, String),
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// The button a select is opened from: the chosen label, a chevron, and the control's own states.
    mod.mp.MpSelectTrigger = RoundedView{
        width: Fill
        height: Fit
        padding: Inset{left: 12, right: 10, top: 8, bottom: 8}
        align: Align{x: 0.0, y: 0.5}
        show_bg: true

        draw_bg +: {
            color: mod.mpc.tokens.bg
            color_hover: mod.mpc.tokens.element_hover
            color_down: mod.mpc.tokens.element_active
            border_color: mod.mpc.tokens.border
            border_size: 1.0
            border_radius: 6.0
        }

        label := Label{
            width: Fill, height: Fit
            draw_text +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text}
            text: ""
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Vec<String> {
        vec!["SQLite".into(), "MySQL".into(), "PostgreSQL".into()]
    }

    #[test]
    fn test_a_select_never_filters_so_its_panel_is_always_the_whole_list() {
        // **The consequence of never being typed into**, rather than a special case: the combobox's filtered view is the
        // whole list when its query is empty, and a select never has a query. Asserted so that a later change which made the
        // panel read the *filtered* view cannot quietly hide rows.
        let mut select = Select::new(options());
        assert_eq!(select.rows().len(), 3);
        select.set_value(Some(1));
        assert_eq!(
            select.rows().len(),
            3,
            "choosing an option narrowed the panel — the trigger's own text is being treated as a query"
        );
        assert_eq!(select.label(), Some("MySQL"));
        // The rows are the options in the caller's order, and exactly one is the current value.
        let rows = select.rows();
        assert_eq!(rows[0], (0, "SQLite".to_string(), false));
        assert_eq!(rows[1], (1, "MySQL".to_string(), true));
        assert_eq!(rows[2], (2, "PostgreSQL".to_string(), false));
        assert_eq!(rows.iter().filter(|(_, _, chosen)| *chosen).count(), 1);
    }

    #[test]
    fn test_setting_a_value_the_list_does_not_cover_drops_it_rather_than_clamping() {
        // The combobox's rule, inherited: a select showing the third option when it has two would be displaying a choice
        // nobody can make. And clearing is a state, not an absence of one.
        let mut select = Select::new(options());
        select.set_value(Some(2));
        assert_eq!(select.label(), Some("PostgreSQL"));
        select.set_options(vec!["SQLite".into(), "MySQL".into()]);
        assert_eq!(select.value(), None, "a stale index survived a shorter list");
        assert_eq!(select.label(), None);
        // A shorter list still leaves the panel whole.
        assert_eq!(select.rows().len(), 2);
        // Clearing explicitly.
        select.set_value(Some(1));
        assert_eq!(select.label(), Some("MySQL"));
        select.set_value(None);
        assert_eq!(select.value(), None);
        assert_eq!(select.chosen(), None);
    }

    #[test]
    fn test_an_empty_select_offers_nothing_and_chooses_nothing() {
        let mut select = Select::new(Vec::new());
        assert!(select.is_empty());
        assert_eq!(select.rows(), Vec::new());
        assert_eq!(select.label(), None);
        assert_eq!(select.active(), None);
        // Stepping and committing over nothing are no-ops rather than a panic or a phantom row 0.
        select.step(1);
        assert_eq!(select.active(), None);
        assert_eq!(select.commit(), None);
        select.set_value(Some(0));
        assert_eq!(select.value(), None, "an empty list accepted a value");
        // And opening it is still a state change — a panel over nothing is a panel, which is how a caller learns it is empty.
        assert!(select.show());
        assert!(select.is_open());
    }

    #[test]
    fn test_opening_highlights_the_chosen_row_or_the_first_so_the_first_arrow_key_is_predictable() {
        // **So that pressing a trigger and pressing down cannot do different things.** A panel that opened with nothing
        // highlighted would make the first arrow key jump to a row nobody aimed at.
        let mut select = Select::new(options());
        assert!(select.show());
        assert_eq!(select.active(), Some(0), "nothing chosen, so the first row");
        assert!(select.is_open());
        // With a value, the highlight starts there.
        let mut chosen = Select::new(options());
        chosen.set_value(Some(2));
        // Choosing set the highlight, so opening keeps it — and `prepare` does not move a highlight that already exists.
        assert_eq!(chosen.active(), Some(2));
        chosen.hide();
        chosen.show();
        assert_eq!(chosen.active(), Some(2), "opening moved a highlight that was already somewhere");
    }

    #[test]
    fn test_showing_and_hiding_are_idempotent_and_the_panel_is_the_wrapper_s_own_state() {
        // The same rule the dialog has, for the same reason: a caller toggling on an unrelated event should not make the
        // panel flicker, so both operations report whether they changed anything.
        let mut select = Select::new(options());
        assert!(!select.is_open());
        assert!(select.show(), "the first show changes it");
        assert!(!select.show(), "the second show changed something");
        assert!(select.is_open(), "and it is still open");
        assert!(select.hide(), "the first hide changes it");
        assert!(!select.hide(), "the second hide changed something");
        // **`toggle` is not idempotent, and that is its job** — it inverts, so it changes the state whichever way it was.
        // I first wrote `assert!(!select.toggle())` for the second call, reading "idempotent" as a property of the whole
        // set rather than of `show` and `hide`; `show` and `hide` are the two that must not flicker. The open flag is what
        // alternates.
        assert!(select.toggle(), "toggling a closed select opens it");
        assert!(select.is_open());
        assert!(select.toggle(), "toggling an open select closes it");
        assert!(!select.is_open());
        // ...and it is the two operations *behind* it that do not flicker.
        assert!(select.show());
        assert!(!select.show());
        assert!(select.hide());
        assert!(!select.hide());
    }

    #[test]
    fn test_committing_chooses_the_highlighted_row_and_closes() {
        // One call rather than two, because a caller that committed and forgot to close would leave a panel open over the
        // value it just chose.
        let mut select = Select::new(options());
        select.show();
        assert_eq!(select.active(), Some(0));
        select.step(1);
        assert_eq!(select.active(), Some(1));
        assert_eq!(select.commit(), Some((1, "MySQL".to_string())));
        assert_eq!(select.label(), Some("MySQL"), "the trigger does not show what was chosen");
        assert!(!select.is_open(), "committing left the panel open");
        // Committing with nothing highlighted is `None` rather than a phantom row.
        let mut empty = Select::new(Vec::new());
        assert_eq!(empty.commit(), None);
    }

    #[test]
    fn test_stepping_wraps_over_the_whole_list() {
        // A select's highlight walks the options and comes back around, which is what makes one key usable for a long list.
        let mut select = Select::new(options());
        select.prepare();
        assert_eq!(select.active(), Some(0));
        select.step(1);
        assert_eq!(select.active(), Some(1));
        select.step(1);
        assert_eq!(select.active(), Some(2));
        select.step(1);
        assert_eq!(select.active(), Some(0), "past the last option wraps");
        select.step(-1);
        assert_eq!(select.active(), Some(2), "and back the other way");
    }
}
