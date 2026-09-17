//! `MpDescriptionList` — key/value rows, for the detail beside a thing.
//!
//! ## A fixed set of slots, and why that is not a shortcut
//!
//! The list declares **eight row slots** in the DSL and fills the first N. The alternative is instantiating a row per
//! item at runtime, and this port has done that elsewhere — `mp/code.rs` builds a run per span, and the A2UI renderer
//! grows pools. The difference is what a row *is* here: two labels and a hairline, with no data of their own and no
//! per-row identity. A pool of eight that is filled and hidden is **simpler to reason about than a growable one**, and
//! what it costs is a bound — which is honest, because a detail list with more than eight rows is a table.
//!
//! The bound is [`SLOTS`], it is public, and [`rows_shown`] is the one place `min` happens so the fill and the hide
//! cannot disagree about which slot is last.
//!
//! ## Themed, unlike the widget this replaces
//!
//! The v2 list named `TEXT_MUTED`, `SURFACE_CARD` and `BORDER` — theme *constants* read at declaration time, which is
//! what `mod.mpc_theme` was. Here the colours are `instance(<token>)` bindings resolved by the theme, so switching
//! appearance re-colours the list without re-declaring it, and the same three tokens the rest of the library paints
//! with are the ones this uses.
//!
//! ## What was dropped, and why
//!
//! The v2 widget carried `bordered: bool` and a five-step `size`. Neither survives: **a free `bordered` is exactly the
//! kind of knob this port's rules forbid** — a caller choosing a border is a caller choosing a colour, a radius and a
//! padding without saying so — and a size step is what the theme's typography is for. A list of key/value rows is one
//! thing, and it looks like the theme says.

use makepad_widgets::*;

/// How many rows the widget can show.
///
/// Public because it is a **contract**, not an implementation detail: a caller that hands over twelve items and expects
/// twelve rows would otherwise find out by looking at the screen. A ninth row is a table.
pub const SLOTS: usize = 8;

/// How many of `count` items are shown.
///
/// The single place the bound is applied, so filling the slots and hiding the rest cannot disagree about which is last
/// — the defect that puts a hairline after a row that is not there.
pub fn rows_shown(count: usize) -> usize {
    count.min(SLOTS)
}

/// Whether the hairline under row `index` is shown, given `count` items.
///
/// **The divider belongs *between* rows, so the last visible row has none.** That is a rule rather than a detail: a rule
/// under the final row reads as a boundary to something that is not there.
pub fn shows_divider(index: usize, count: usize) -> bool {
    index + 1 < rows_shown(count)
}

/// A row, on the way in. Owned strings rather than borrowed, because the list keeps them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescriptionItem {
    pub label: String,
    pub value: String,
}

impl DescriptionItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// A `(label, value)` pair, so a caller can hand over a slice of tuples.
impl From<(&str, &str)> for DescriptionItem {
    fn from((label, value): (&str, &str)) -> Self {
        Self::new(label, value)
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// One row: the content, then the hairline under it.
    mod.mp.MpDescriptionRow = mod.widgets.View{
        width: Fill
        height: Fit
        flow: Down

        content := View{
            width: Fill
            height: Fit
            padding: Inset{top: 6, bottom: 6}
            flow: Right
            spacing: 12
            align: Align{y: 0.5}

            desc_label := Label{
                width: 120
                height: Fit
                draw_text +: {
                    text_style: body
                    color: text_muted
                }
                text: ""
            }
            desc_value := Label{
                width: Fill
                height: Fit
                draw_text +: {
                    text_style: body
                    color: text
                }
                text: ""
            }
        }

        // `mod.mp.Divider` rather than a hand-rolled 0.5px View: the v2 row built its own zero-height plate with its own
        // colour, which is a divider re-invented per component. This one is the library's, so a change to how the library
        // divides things reaches every divider in it.
        row_divider := mod.mp.Divider{}
    }

    mod.mp.MpDescriptionListBase = #(MpDescriptionList::register_widget(vm))

    mod.mp.MpDescriptionList = set_type_default() do mod.mp.MpDescriptionListBase{
        width: Fill
        height: Fit
        flow: Down
        padding: Inset{left: 16, right: 16, top: 8, bottom: 8}

        show_bg: true
        draw_bg +: {
            color: instance(surface_card)
            border_radius: instance(12.0)
            border_size: instance(1.0)
            border_color: instance(border)
        }

        row0 := mod.mp.MpDescriptionRow{}
        row1 := mod.mp.MpDescriptionRow{}
        row2 := mod.mp.MpDescriptionRow{}
        row3 := mod.mp.MpDescriptionRow{}
        row4 := mod.mp.MpDescriptionRow{}
        row5 := mod.mp.MpDescriptionRow{}
        row6 := mod.mp.MpDescriptionRow{}
        row7 := mod.mp.MpDescriptionRow{}
    }
}

/// The list. Rows are `mod.mp.MpDescriptionRow` aliases reached by id.
#[derive(Script, ScriptHook, Widget)]
pub struct MpDescriptionList {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// The items last set, so a re-apply — which the theme asks for on an appearance change — does not lose them.
    ///
    /// `#[rust]` rather than `#[live]`, and that is load-bearing: a `#[live]` field is written back to its declared
    /// value whenever the script is re-applied, so an item list kept there would be emptied by a theme change. See this
    /// port's note on the rule.
    #[rust]
    items: Vec<DescriptionItem>,
}

/// The id path of row `index`.
fn row_id(index: usize) -> LiveId {
    LiveId::from_str(&format!("row{index}"))
}

/// The id path of a row's label or value.
fn cell_id(index: usize, cell: &str) -> [LiveId; 3] {
    // `id!` for one id and `ids!` for a path: the macros differ by exactly that, and using `ids!` where a single id is
    // wanted gives a one-element slice rather than the id.
    let cell = match cell {
        "desc_label" => id!(desc_label),
        _ => id!(desc_value),
    };
    [row_id(index), id!(content), cell]
}

impl Widget for MpDescriptionList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl MpDescriptionList {
    /// Fill the list, hiding what is left over.
    pub fn set_items(&mut self, cx: &mut Cx, items: &[DescriptionItem]) {
        self.items = items.to_vec();
        self.apply(cx);
    }

    /// The items currently held.
    pub fn items(&self) -> &[DescriptionItem] {
        &self.items
    }

    /// Re-apply the held items, which is also what a theme change needs.
    pub fn apply(&mut self, cx: &mut Cx) {
        let shown = rows_shown(self.items.len());
        for index in 0..SLOTS {
            let row = self.view.view(cx, &[row_id(index)]);
            if index >= shown {
                row.set_visible(cx, false);
                continue;
            }
            row.set_visible(cx, true);
            let item = &self.items[index];
            self.view
                .label(cx, &cell_id(index, "desc_label"))
                .set_text(cx, &item.label);
            self.view
                .label(cx, &cell_id(index, "desc_value"))
                .set_text(cx, &item.value);
            // **The hairline belongs between rows**, so the last visible one has none — a rule, in one place, so the
            // fill and the hide above cannot disagree about which row is last.
            self.view
                .view(cx, &[row_id(index), id!(row_divider)])
                .set_visible(cx, shows_divider(index, self.items.len()));
        }
        self.redraw(cx);
    }
}

impl MpDescriptionListRef {
    pub fn set_items(&self, cx: &mut Cx, items: &[DescriptionItem]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_items(cx, items);
        }
    }

    pub fn items(&self) -> Vec<DescriptionItem> {
        self.borrow().map(|inner| inner.items.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_bound_is_the_one_field_of_slots() {
        // Zero, some, exactly the bound, and past it. The two calls that must agree are `rows_shown` (in `apply`) and
        // `shows_divider` (the rule under each row), so they are checked against **each other** here rather than against
        // a copied number.
        assert_eq!(rows_shown(0), 0);
        assert_eq!(rows_shown(3), 3);
        assert_eq!(rows_shown(SLOTS), SLOTS);
        assert_eq!(rows_shown(SLOTS + 4), SLOTS, "the bound is not applied");
    }

    #[test]
    fn test_the_last_visible_row_has_no_rule_under_it() {
        // **The rule, not a detail.** A rule under the final row reads as a boundary to something that is not there.
        assert!(!shows_divider(0, 1), "one row has a rule under it");
        assert!(shows_divider(0, 2));
        assert!(!shows_divider(1, 2), "the last of two rows has a rule under it");

        // ...and it holds **at the bound**, where the count and the shown rows stop being equal — the case a rule written
        // against `count` instead of `rows_shown` gets wrong.
        assert!(shows_divider(SLOTS - 2, SLOTS + 3));
        assert!(!shows_divider(SLOTS - 1, SLOTS + 3), "a rule under the last slot there is");
        for count in 0..(SLOTS + 4) {
            let shown = rows_shown(count);
            if shown > 0 {
                assert!(
                    !shows_divider(shown - 1, count),
                    "count={count} shows a rule under its last row"
                );
            }
        }
    }

    #[test]
    fn test_no_row_at_or_past_the_bound_is_ever_asked_to_show_a_rule() {
        // The other half: the slots that are hidden must not be given a divider either, or a hidden row would be
        // half-shown — invisible content with a visible rule under it, which is the worst of both.
        for count in 0..(SLOTS + 4) {
            let shown = rows_shown(count);
            for index in shown..SLOTS {
                assert!(
                    !shows_divider(index, count),
                    "hidden slot {index} was given a rule at count={count}"
                );
            }
        }
    }

    #[test]
    fn test_an_item_is_built_from_a_pair() {
        let item: DescriptionItem = ("Version", "1.0").into();
        assert_eq!(item, DescriptionItem::new("Version", "1.0"));
        assert_eq!(item.label, "Version");
        assert_eq!(item.value, "1.0");
    }
}
