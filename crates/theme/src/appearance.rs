//! Which palette is in force, and the switch that moves it.

/// The two shipped appearances. A closed enum: a palette has no third setting,
/// and an app that wants its own hues rotates this one through [`Brand`]
/// instead of inventing an appearance.
///
/// [`Brand`]: crate::brand::Brand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Appearance {
    #[default]
    Dark,
    Light,
}

impl Appearance {
    /// Whether this appearance paints light ink on dark ground.
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    /// The appearance `cx` is currently in.
    ///
    /// Falls back to [`Appearance::Dark`] before anything installed a theme,
    /// so a widget painting during the first frame reads a real palette rather
    /// than nothing.
    pub fn current(cx: &mut makepad_widgets::Cx) -> Self {
        crate::theme::Theme::of(cx).appearance
    }

    /// Make this the appearance in force and repaint.
    pub fn set(self, cx: &mut makepad_widgets::Cx) {
        crate::theme::Theme::with(cx, |theme| theme.appearance = self);
    }

    /// Flip to the other one and repaint.
    pub fn toggle(self, cx: &mut makepad_widgets::Cx) {
        self.other().set(cx)
    }

    pub const fn other(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_dark() {
        assert_eq!(Appearance::default(), Appearance::Dark);
    }

    #[test]
    fn test_is_dark_matches_the_variant() {
        assert!(Appearance::Dark.is_dark());
        assert!(!Appearance::Light.is_dark());
    }

    #[test]
    fn test_other_is_an_involution() {
        for a in [Appearance::Dark, Appearance::Light] {
            assert_eq!(a.other().other(), a);
            assert_ne!(a.other(), a);
        }
    }
}
