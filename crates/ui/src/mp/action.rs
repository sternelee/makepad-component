//! Reading a widget's own actions out of an [`Actions`] batch.
//!
//! Two ways to get this wrong, and the v2 set used both.
//!
//! [`WidgetActionsApi::find_widget_action`] returns the **first** action whose
//! `widget_uid` matches. A widget uid carries more than one action in a single
//! batch — focus and hover are dispatched as widget actions too — so the first
//! match is routinely not the one the caller asked about.
//! [`WidgetAction::cast`] then makes that invisible: it returns `T::default()`
//! when the `downcast_ref` fails, so `matches!(action.cast(), MpButtonAction::Clicked)`
//! is false both for "a different action of mine arrived first" and for "I
//! emitted `None`", with no way to tell them apart.
//!
//! The v2 set hit this and it is recorded in `docs/WIDGETS_PROGRESS_CN.md` §3.2
//! as a serious bug: a collapsible's trigger rotated its chevron and never
//! expanded, with the log showing the uid matched and both downstream casts
//! false. Three widgets were patched to walk the list by hand; the rest kept the
//! broken form.
//!
//! So the walk lives here, once, over the framework's
//! [`filter_widget_actions`](WidgetActionsApi::filter_widget_actions) — which
//! exists for exactly this and whose own doc comment warns about the first-match
//! pitfall — and a widget asks for the action it wants:
//!
//! ```ignore
//! impl MpButton {
//!     pub fn clicked(&self, actions: &Actions) -> bool {
//!         action::is::<MpButtonAction>(self.widget_uid(), actions, |a| {
//!             matches!(a, MpButtonAction::Clicked)
//!         })
//!     }
//! }
//! ```

use makepad_widgets::*;

/// Every action of type `T` this widget emitted, in dispatch order.
///
/// `T` does not have to be the widget's whole action enum — a widget with two
/// enums can call this twice and get both.
///
/// The bound is the framework's own [`WidgetActionTrait`] rather than
/// `'static`, because `downcast_ref` on the boxed action is what does the work
/// and that is the trait it is defined on.
pub fn of_type<T: WidgetActionTrait>(uid: WidgetUid, actions: &Actions) -> Vec<&T> {
    actions
        .filter_widget_actions(uid)
        .filter_map(|action| action.action.downcast_ref::<T>())
        .collect()
}

/// Whether this widget emitted an action of type `T` that `test` accepts.
///
/// The predicate form rather than "did it emit `Clicked`" because the caller
/// owns the enum, and a helper that knew about `Clicked` would have to be
/// rewritten for every widget in the crate.
pub fn is<T: WidgetActionTrait>(
    uid: WidgetUid,
    actions: &Actions,
    test: impl Fn(&T) -> bool,
) -> bool {
    of_type::<T>(uid, actions).into_iter().any(test)
}

/// The first action of type `T` this widget emitted, if any.
///
/// Used where the action carries data — a selected index, a new value — rather
/// than only a signal.
pub fn first<T: WidgetActionTrait>(uid: WidgetUid, actions: &Actions) -> Option<&T> {
    of_type::<T>(uid, actions).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A widget's own action.
    #[derive(Clone, Debug, Default, PartialEq)]
    enum Fake {
        #[default]
        None,
        Clicked,
        Changed(u32),
    }

    /// The kind of unrelated action the framework dispatches for the same uid.
    #[derive(Clone, Debug, Default, PartialEq)]
    struct FocusNoise;

    fn uid(n: u64) -> WidgetUid {
        WidgetUid(n)
    }

    fn action<T: Clone + std::fmt::Debug + Send + Sync + 'static>(
        uid: WidgetUid,
        value: T,
    ) -> Action {
        Box::new(WidgetAction {
            data: None,
            action: Box::new(value),
            widget_uid: uid,
            group: None,
        })
    }

    /// The batch that broke the v2 widgets: another action for the same uid,
    /// dispatched first.
    fn poisoned_batch(uid: WidgetUid) -> Vec<Action> {
        vec![action(uid, FocusNoise), action(uid, Fake::Clicked)]
    }

    #[test]
    fn test_the_first_action_for_a_uid_is_not_the_one_asked_for() {
        // Establishes the bug this module exists for, so a reader does not have
        // to take it on faith.
        let uid = uid(7);
        let batch = poisoned_batch(uid);
        let first = batch
            .find_widget_action(uid)
            .expect("the uid has actions");
        assert!(
            first.action.downcast_ref::<Fake>().is_none(),
            "the first action for this uid is another widget's"
        );
    }

    #[test]
    fn test_cast_hides_the_difference_the_first_match_loses() {
        // The other half of the v2 bug: `cast` returns the default variant, so
        // "not my action" and "my action's None" are the same value.
        let uid = uid(7);
        let batch = poisoned_batch(uid);
        let found = batch.find_widget_action(uid).unwrap();
        assert_eq!(found.cast::<Fake>(), Fake::None);
        assert_ne!(found.cast::<Fake>(), Fake::Clicked);
    }

    #[test]
    fn test_walking_finds_the_action_behind_the_noise() {
        let uid = uid(7);
        let batch = poisoned_batch(uid);
        assert!(is::<Fake>(uid, &batch, |a| *a == Fake::Clicked));
    }

    #[test]
    fn test_another_widgets_actions_are_not_claimed() {
        let batch = poisoned_batch(uid(7));
        assert!(!is::<Fake>(uid(8), &batch, |a| *a == Fake::Clicked));
        assert!(of_type::<Fake>(uid(8), &batch).is_empty());
    }

    #[test]
    fn test_a_batch_with_no_action_for_the_uid_is_empty_not_a_panic() {
        let batch: Vec<Action> = Vec::new();
        assert!(!is::<Fake>(uid(1), &batch, |_| true));
        assert!(first::<Fake>(uid(1), &batch).is_none());
    }

    #[test]
    fn test_a_none_action_is_not_a_click() {
        // The distinction `cast` erased: the widget did emit an action, and it
        // was `None`.
        let uid = uid(4);
        let batch = vec![action(uid, Fake::None)];
        assert!(of_type::<Fake>(uid, &batch).len() == 1);
        assert!(!is::<Fake>(uid, &batch, |a| *a == Fake::Clicked));
    }

    #[test]
    fn test_every_matching_action_is_returned_in_order() {
        let uid = uid(3);
        let batch: Vec<Action> = [1u32, 2, 3]
            .into_iter()
            .map(|value| action(uid, Fake::Changed(value)))
            .collect();
        let values: Vec<u32> = of_type::<Fake>(uid, &batch)
            .into_iter()
            .filter_map(|a| match a {
                Fake::Changed(v) => Some(*v),
                _ => None,
            })
            .collect();
        assert_eq!(values, vec![1, 2, 3]);
        assert_eq!(first::<Fake>(uid, &batch), Some(&Fake::Changed(1)));
    }

    #[test]
    fn test_a_predicate_that_accepts_nothing_reports_nothing() {
        let uid = uid(7);
        let batch = poisoned_batch(uid);
        assert!(!is::<Fake>(uid, &batch, |_| false));
    }

    #[test]
    fn test_two_action_enums_from_one_widget_do_not_shadow_each_other() {
        // A widget with a public action and an internal one: asking for either
        // must not depend on which was dispatched first.
        #[derive(Clone, Debug, Default, PartialEq)]
        enum Internal {
            #[default]
            Idle,
            Ticked,
        }
        let uid = uid(11);
        let batch = vec![action(uid, Internal::Ticked), action(uid, Fake::Clicked)];
        assert!(is::<Fake>(uid, &batch, |a| *a == Fake::Clicked));
        assert!(is::<Internal>(uid, &batch, |a| *a == Internal::Ticked));
        assert_eq!(of_type::<Fake>(uid, &batch).len(), 1);
    }
}
