//! `MpBreadcrumb` — where you are, and the way back.
//!
//! ## The current crumb is derived, not passed
//!
//! bezel asks its caller for each crumb's `current` flag. In a breadcrumb the current one **is the last one** — that is what a
//! trail means — so this derives it ([`is_current`]) rather than taking a flag that a caller could set on two crumbs at once,
//! or on none. The same move the menubar made with its single `open`: an invariant in the type rather than in every caller.
//!
//! ## A separator belongs *between* crumbs, so the last has none
//!
//! [`separator_count`] is `count - 1`. A trailing chevron would point at nothing, and the same rule already governs this
//! library's description list, whose hairline sits between rows and not after the last one. Two components, one rule, which
//! is the point of a library.
//!
//! ## The current crumb does not truncate before the others do
//!
//! A trail is read from its end: the crumb you are on is the one that must be legible, and the ancestors behind it are
//! recognisable when elided. So [`allocate`] gives the last crumb its natural width first and shares what is left among the
//! rest, each down to [`MIN_CRUMB`] — and when even that does not fit, the trail overflows and the caller scrolls, exactly as
//! the tab bar does under the same pressure. **That ordering is the whole difference between this and an equal share**, and
//! it is the reason the allocation is a function here rather than a loop in a painter.
//!
//! ## The crumb you are on is not a link
//!
//! [`MpBreadcrumbAction`] reports a navigation for every crumb **except the current one** — a click there is a click on where
//! you already are, and reporting it would make a caller push a second copy of the page it is showing. The hit test finds
//! crumbs only: a chevron between two of them is not one of them, for the same reason the tab bar's add button is not a tab.

use makepad_widgets::*;

/// The gap between a crumb and the chevron beside it.
pub const GAP: f64 = 4.0;

/// The chevron's width.
pub const SEP_W: f64 = 12.0;

/// The narrowest a crumb may be squeezed to before the trail overflows instead.
pub const MIN_CRUMB: f64 = 24.0;

/// The row's height.
pub const ROW_HEIGHT: f64 = 22.0;

/// Whether a crumb is the one the reader is on.
///
/// **The last one, derived** — so no caller can mark two crumbs current, or none. An empty trail has no current crumb, and a
/// one-crumb trail's only crumb is current.
pub fn is_current(index: usize, count: usize) -> bool {
    count > 0 && index + 1 == count
}

/// How many chevrons a trail of `count` crumbs needs.
///
/// **Between the crumbs, not after them.** A trailing chevron points at nothing, and the rule is the same one this library's
/// description list follows with its hairline.
pub fn separator_count(count: usize) -> usize {
    count.saturating_sub(1)
}

/// The crumbs' widths for a trail whose natural widths are `natural`, sharing `available`.
///
/// - the **last** crumb takes its natural width first, up to what is available, because the crumb you are on is the one that
///   must be legible;
/// - the rest share what is left, down to [`MIN_CRUMB`];
/// - **under pressure every crumb takes [`MIN_CRUMB`] and the trail overflows**, which is the same honest failure the tab bar
///   makes: crumbs narrower than that cannot be read, and a trail that squeezed them all in would be showing nothing useful.
pub fn allocate(natural: &[f64], available: f64, gap: f64, sep: f64) -> Vec<f64> {
    let count = natural.len();
    if count == 0 || !available.is_finite() || available <= 0.0 {
        return vec![0.0; count];
    }
    // What the row costs before any crumb is drawn: the gaps and the chevrons between them.
    let chrome = separator_count(count) as f64 * (sep + gap * 2.0);
    let for_crumbs = (available - chrome).max(0.0);
    let mut widths = vec![0.0; count];
    let last = count - 1;
    // The current crumb first, clipped to the room there is rather than to a share of it.
    widths[last] = natural[last].clamp(0.0, for_crumbs);
    let remaining = (for_crumbs - widths[last]).max(0.0);
    if count == 1 {
        return widths;
    }
    let share = remaining / last as f64;
    let each = if share < MIN_CRUMB { MIN_CRUMB } else { share };
    for width in widths.iter_mut().take(last) {
        *width = each.min(remaining.max(MIN_CRUMB));
    }
    widths
}

/// The whole trail's width for the widths [`allocate`] returned.
pub fn trail_width(widths: &[f64], gap: f64, sep: f64) -> f64 {
    let crumbs: f64 = widths.iter().sum();
    crumbs + separator_count(widths.len()) as f64 * (sep + gap * 2.0)
}

/// Whether a trail laid out for `available` overflowed it.
pub fn overflows(widths: &[f64], available: f64, gap: f64, sep: f64) -> bool {
    trail_width(widths, gap, sep) > available + 1e-9
}

/// Which crumb an `x` lands on, or `None`.
///
/// `x` is relative to the row's leading edge. **Crumbs only**: the chevrons between them are places where nothing is, so a
/// click that lands on one reports nothing rather than the crumb beside it — the same rule that keeps a separator from being
/// a row in a menu.
pub fn crumb_at(x: f64, widths: &[f64], gap: f64, sep: f64) -> Option<usize> {
    if !x.is_finite() || x < 0.0 {
        return None;
    }
    let mut edge = 0.0;
    for (index, width) in widths.iter().enumerate() {
        if *width <= 0.0 {
            continue;
        }
        if x >= edge && x < edge + width {
            return Some(index);
        }
        edge += width;
        // The gap, the chevron and the gap on the other side, before the next crumb.
        if index + 1 < widths.len() {
            edge += gap + sep + gap;
        }
    }
    None
}

/// Where a crumb starts, for a painter that needs the same arithmetic the hit test uses.
pub fn crumb_origin(index: usize, widths: &[f64], gap: f64, sep: f64) -> f64 {
    let mut x = 0.0;
    for (at, width) in widths.iter().enumerate() {
        if at == index {
            return x;
        }
        x += width + gap + sep + gap;
    }
    x
}

/// What a breadcrumb reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpBreadcrumbAction {
    /// A crumb that is not the current one was chosen, carrying its index.
    Navigated(usize),
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    mod.mp.MpBreadcrumbBase = #(MpBreadcrumb::register_widget(vm))

    mod.mp.MpBreadcrumb = set_type_default() do mod.mp.MpBreadcrumbBase{
        width: Fill
        height: 22.0

        draw_crumb +: {text_style: mod.mpc.type.callout, color: mod.mpc.tokens.text_muted}
        draw_separator +: {text_style: mod.mpc.type.caption, color: mod.mpc.tokens.text_faint}
    }
}

/// A trail of crumbs, the last of them current.
#[derive(Script, ScriptHook, Widget)]
pub struct MpBreadcrumb {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_crumb: DrawText,
    #[live]
    draw_separator: DrawText,
    /// The chevron drawn between crumbs.
    #[live]
    separator: ArcStringMut,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// The trail, outermost first. **The caller's data**, like every other list in this port.
    #[rust]
    trail: Vec<String>,
    #[rust]
    hovered: Option<usize>,
    #[rust]
    origin: DVec2,
    #[rust]
    area: Area,
}

impl MpBreadcrumb {
    /// Replace the trail. The **last** crumb becomes the current one; a caller does not say which it is.
    pub fn set_trail(&mut self, cx: &mut Cx, trail: Vec<String>) {
        self.trail = trail;
        if self.hovered.is_some_and(|index| index >= self.trail.len()) {
            self.hovered = None;
        }
        self.redraw(cx);
    }

    pub fn trail(&self) -> &[String] {
        &self.trail
    }

    /// The current crumb, which is the last one.
    pub fn current(&self) -> Option<&str> {
        self.trail.last().map(String::as_str)
    }

    /// The ancestors — every crumb but the current one, which is what a caller navigates to.
    pub fn ancestors(&self) -> &[String] {
        let len = self.trail.len();
        &self.trail[..len.saturating_sub(1)]
    }

    /// Whether `index` is the crumb the reader is on.
    pub fn is_current(&self, index: usize) -> bool {
        is_current(index, self.trail.len())
    }

    /// The laid-out widths for an available width, so a caller can ask what the painter asks.
    pub fn widths(&self, cx: &mut Cx, available: f64) -> Vec<f64> {
        let natural: Vec<f64> = self
            .trail
            .iter()
            .map(|crumb| crate::mp::text::measured_width(&self.draw_crumb, cx, crumb))
            .collect();
        allocate(&natural, available, GAP, SEP_W)
    }
}

impl Widget for MpBreadcrumb {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let available = self.area.rect(cx).size.x;
        let widths = self.widths(cx, available);
        if let Event::MouseDown(me) = event {
            let x = me.abs.x - self.origin.x;
            if let Some(index) = crumb_at(x, &widths, GAP, SEP_W) {
                // **The crumb you are on is not a link.** A click on it is a click on where the reader already is, and
                // reporting it would have a caller push a second copy of the page it is showing.
                if !is_current(index, self.trail.len()) {
                    cx.widget_action(self.uid, MpBreadcrumbAction::Navigated(index));
                }
            }
        }
        if let Event::MouseMove(me) = event {
            let x = me.abs.x - self.origin.x;
            let hovered = crumb_at(x, &widths, GAP, SEP_W)
                .filter(|index| !is_current(*index, self.trail.len()));
            if self.hovered != hovered {
                self.hovered = hovered;
                self.redraw(cx);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (muted, faint, ink) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (theme.paint.text_muted, theme.paint.text_faint, theme.paint.text)
        };
        let placed = cx.walk_turtle(walk);
        self.origin = placed.pos;
        self.area = self.draw_crumb.area();
        let available = placed.size.x;
        let natural: Vec<f64> = self
            .trail
            .iter()
            .map(|crumb| crate::mp::text::measured_width(&self.draw_crumb, cx.cx, crumb))
            .collect();
        let widths = allocate(&natural, available, GAP, SEP_W);
        // The chevron, from the DSL so a caller can set it — a breadcrumb's separator is a glyph, and which glyph is a
        // question with more than one right answer.
        let separator = if self.separator.as_ref().is_empty() {
            "\u{203A}".to_string()
        } else {
            self.separator.as_ref().to_string()
        };

        for (index, crumb) in self.trail.iter().enumerate() {
            let x = placed.pos.x + crumb_origin(index, &widths, GAP, SEP_W);
            let current = is_current(index, self.trail.len());
            self.draw_crumb.color = if current { ink } else { muted };
            // **Truncated to its own width**, with the current crumb getting the width it asked for — see `allocate`.
            let clipped = crate::mp::tab_bar::clip_to(crumb, widths[index], cx.cx, &self.draw_crumb);
            self.draw_crumb.draw_abs(cx, dvec2(x, placed.pos.y + 3.0), &clipped);
            if !current {
                self.draw_separator.color = faint;
                self.draw_separator.draw_abs(
                    cx,
                    dvec2(x + widths[index] + GAP, placed.pos.y + 4.0),
                    &separator,
                );
            }
        }
        DrawStep::done()
    }
}

impl MpBreadcrumbRef {
    pub fn set_trail(&self, cx: &mut Cx, trail: Vec<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_trail(cx, trail);
        }
    }

    pub fn current(&self) -> Option<String> {
        self.borrow().and_then(|inner| inner.current().map(str::to_string))
    }

    pub fn navigated(&self, actions: &Actions) -> Option<usize> {
        actions
            .find_widget_action(self.widget_uid())
            .and_then(|action| match action.cast::<MpBreadcrumbAction>() {
                MpBreadcrumbAction::Navigated(index) => Some(index),
                MpBreadcrumbAction::None => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_current_crumb_is_the_last_one_and_a_caller_cannot_say_otherwise() {
        // **An invariant in the type rather than in every caller.** bezel takes a `current` flag per crumb, which allows two
        // crumbs to be current at once or none at all; a trail means the last one, so it is derived.
        assert!(is_current(2, 3));
        assert!(!is_current(1, 3));
        assert!(!is_current(0, 3));
        // One crumb is current.
        assert!(is_current(0, 1));
        // An empty trail has no current crumb, and no index is current in it — including index 0, which is the trap a
        // `index + 1 == count` written without the count check would fall into.
        assert!(!is_current(0, 0));
        assert!(!is_current(5, 0));
    }

    #[test]
    fn test_a_separator_sits_between_crumbs_so_the_last_has_none() {
        // A trailing chevron points at nothing — the same rule this library's description list follows with its hairline,
        // which is why two components share it.
        assert_eq!(separator_count(0), 0);
        assert_eq!(separator_count(1), 0, "a lone crumb has nothing to be separated from");
        assert_eq!(separator_count(2), 1);
        assert_eq!(separator_count(5), 4);
        // A trail of `n` crumbs has `n - 1` chevrons and one more crumb than chevron — stated as the property rather than
        // as five numbers. (My first version wrote `count.max(1).max(count)` here, which is just `count.max(1)`: an
        // assertion that could not fail.)
        for count in 1..8usize {
            assert_eq!(separator_count(count) + 1, count, "count {count}");
            assert!(separator_count(count) < count, "more chevrons than crumbs: {count}");
        }
    }

    #[test]
    fn test_the_current_crumb_keeps_its_width_and_the_ancestors_share_what_is_left() {
        // **The rule that separates this from an equal share.** A trail is read from its end, so the crumb you are on is the
        // one that must be legible and the ancestors are recognisable when elided.
        let natural = [60.0, 80.0, 120.0];
        let available = 400.0;
        let widths = allocate(&natural, available, GAP, SEP_W);
        assert_eq!(widths[2], 120.0, "the current crumb did not keep its width");
        // Two chevrons, each with a gap on both sides: the chrome is spent before the crumbs share.
        let chrome = 2.0 * (SEP_W + GAP * 2.0);
        assert_eq!(widths[0], widths[1], "the ancestors did not share equally");
        assert!((widths[0] * 2.0 - (available - chrome - 120.0)).abs() < 1e-9);
        assert_eq!(trail_width(&widths, GAP, SEP_W), available);
    }

    #[test]
    fn test_under_pressure_the_ancestors_take_the_minimum_and_the_trail_overflows() {
        // **The same honest failure the tab bar makes**: crumbs narrower than the minimum cannot be read, so the trail
        // overflows and the caller scrolls rather than showing unreadable slivers.
        let natural = [90.0, 90.0, 90.0, 90.0];
        let available = 200.0;
        let widths = allocate(&natural, available, GAP, SEP_W);
        assert!(widths.iter().all(|width| *width >= MIN_CRUMB), "a crumb was squeezed below its minimum");
        assert!(overflows(&widths, available, GAP, SEP_W), "the trail did not overflow");
        // ...and with room it does not overflow, which is the contrast that makes the assertion above meaningful.
        let roomy = allocate(&natural, 900.0, GAP, SEP_W);
        assert!(!overflows(&roomy, 900.0, GAP, SEP_W));
    }

    #[test]
    fn test_a_crumb_narrower_than_the_space_is_clipped_to_it_not_left_to_overflow() {
        // The current crumb keeps its width *up to what is available* — a long current crumb on a narrow row takes the row
        // rather than running off it.
        let widths = allocate(&[40.0, 500.0], 120.0, GAP, SEP_W);
        assert!(widths[1] <= 120.0, "the current crumb ran off the row");
        assert_eq!(widths[1], 120.0 - (SEP_W + GAP * 2.0));
        // And a one-crumb trail spends no chrome at all.
        let single = allocate(&[90.0], 200.0, GAP, SEP_W);
        assert_eq!(single, vec![90.0]);
        assert_eq!(trail_width(&single, GAP, SEP_W), 90.0);
    }

    #[test]
    fn test_a_point_lands_on_a_crumb_and_the_chevrons_between_them_are_nothing() {
        // **Crumbs only.** A click that lands on a chevron reports nothing rather than the crumb beside it — the same rule
        // that keeps a separator from being a row in a menu, and the tab bar's add button from being a tab.
        let widths = [60.0, 80.0, 120.0];
        let gap = GAP;
        let sep = SEP_W;
        assert_eq!(crumb_at(0.0, &widths, gap, sep), Some(0));
        assert_eq!(crumb_at(59.9, &widths, gap, sep), Some(0));
        // The chrome between crumb 0 and crumb 1: gap + chevron + gap.
        let after_first = 60.0;
        assert_eq!(crumb_at(after_first + 0.1, &widths, gap, sep), None, "the gap was a crumb");
        assert_eq!(crumb_at(after_first + gap + sep * 0.5, &widths, gap, sep), None, "the chevron was a crumb");
        assert_eq!(crumb_at(after_first + gap + sep + gap - 0.1, &widths, gap, sep), None);
        assert_eq!(crumb_at(crumb_origin(1, &widths, gap, sep), &widths, gap, sep), Some(1));
        assert_eq!(crumb_at(crumb_origin(2, &widths, gap, sep) + 1.0, &widths, gap, sep), Some(2));
        // Past the end, and nonsense points.
        let end = trail_width(&widths, gap, sep);
        assert_eq!(crumb_at(end, &widths, gap, sep), None);
        assert_eq!(crumb_at(f64::NAN, &widths, gap, sep), None);
        assert_eq!(crumb_at(-1.0, &widths, gap, sep), None);
        assert_eq!(crumb_at(0.0, &[], gap, sep), None);
    }

    #[test]
    fn test_every_crumb_is_reachable_and_the_origins_agree_with_the_hit_test() {
        // The property tying the painter to the hit test: aiming at a crumb's centre lands on it, and `crumb_origin` — which
        // the painter uses — is the same arithmetic `crumb_at` walks.
        let widths = allocate(&[60.0, 80.0, 120.0, 45.0], 900.0, GAP, SEP_W);
        for index in 0..widths.len() {
            let start = crumb_origin(index, &widths, GAP, SEP_W);
            assert_eq!(crumb_at(start + 0.5, &widths, GAP, SEP_W), Some(index));
            assert_eq!(
                crumb_at(start + widths[index] - 0.5, &widths, GAP, SEP_W),
                Some(index),
                "the right edge of {index}"
            );
        }
        // The origins advance by the crumb plus the chrome, so a trail tiles from zero.
        assert_eq!(crumb_origin(0, &widths, GAP, SEP_W), 0.0);
        for index in 1..widths.len() {
            assert_eq!(
                crumb_origin(index - 1, &widths, GAP, SEP_W) + widths[index - 1] + GAP + SEP_W + GAP,
                crumb_origin(index, &widths, GAP, SEP_W)
            );
        }
    }

    #[test]
    fn test_an_empty_trail_lays_out_nothing() {
        assert_eq!(allocate(&[], 400.0, GAP, SEP_W), Vec::<f64>::new());
        assert_eq!(trail_width(&[], GAP, SEP_W), 0.0);
        assert!(!overflows(&[], 0.0, GAP, SEP_W));
        // And a nonsense width allocates nothing rather than a `NaN` every rect inherits.
        assert_eq!(allocate(&[60.0, 90.0], 0.0, GAP, SEP_W), vec![0.0, 0.0]);
        assert_eq!(allocate(&[60.0, 90.0], f64::NAN, GAP, SEP_W), vec![0.0, 0.0]);
        assert_eq!(allocate(&[60.0, 90.0], -10.0, GAP, SEP_W), vec![0.0, 0.0]);
    }
}
