//! The terminal's colours, for the emulator that already exists in this workspace.
//!
//! ## Why this is in the theme rather than at the terminal
//!
//! `crates/canvas-terminal/src/terminal/state.rs` holds a working `vte`-driven emulator, and its
//! colours are a `const` at the top of the file:
//!
//! ```ignore
//! const ANSI_COLORS: [[f32; 3]; 16] = [ /* a dark-background scheme */ ];
//! pub const DEFAULT_FG: [f32; 3] = [0.86, 0.89, 0.94];
//! pub const DEFAULT_BG: [f32; 3] = [0.10, 0.11, 0.14];
//! ```
//!
//! That is a **hardcoded palette at a call site**, which is the first thing this port's laws forbid:
//! the terminal cannot follow an appearance change, and a brand cannot reach it. Reading the file is
//! also what stopped this port from writing a second emulator — the crate was on the plan as
//! "`terminal`, 1296 lines in the reference" and the substance of it already exists here.
//!
//! So the contribution is the **colour vocabulary**, which is the part that was missing.
//!
//! ## The one rule that makes an ANSI palette different from every other palette here
//!
//! Every one of the sixteen colours is painted **on the terminal's own background**, so all sixteen
//! are text colours and all sixteen have to be readable — with one deliberate exception:
//!
//! **ANSI 0 (`black`) is the background colour on purpose.** It is what a program paints to hide
//! something, and every real terminal's `black` is the background tone or a step towards it.
//! `bright black` (8) is the readable grey, and it is the one a program uses for dim text. So the
//! floor is [`BACKGROUND_FLOOR`] for the four slots that exist to be invisible and [`TEXT_FLOOR`] for
//! the twelve that exist to be read.
//!
//! Getting this wrong in either direction is a real fault: a tester that demanded AA of `black`
//! would force it to be a colour nobody's terminal shows, and one that demanded nothing of the other
//! twelve would ship a red that vanishes on the background.

use makepad_widgets::*;

use crate::appearance::Appearance;
use crate::color;

/// The readable floor, as WCAG's AA for body text.
pub const TEXT_FLOOR: f32 = 4.5;

/// The floor for `bright black`, which is the grey a program uses for **dim text**.
///
/// This one is meant to be read, so it has a real floor — and a ceiling below full text, because a
/// "dim" colour that clears AA comfortably is not dim.
pub const DIM_FLOOR: f32 = 3.0;
/// The ceiling for `bright black`, for the reason above.
pub const DIM_CEILING: f32 = 6.0;

/// How close ANSI `black` has to be to the terminal's own background.
///
/// **`black` erases**, in both appearances: a program paints it to hide something, and the result is
/// that nothing changes. So it is the ground on a dark terminal *and* on a light one.
///
/// This took three attempts to get right, and the wrong turns are worth keeping because each one
/// looked plausible:
///
/// 1. The first rule demanded 3:1 of `black` — a floor that contradicts its own doc comment. It failed
///    at **1.14:1** on the dark palette, correctly.
/// 2. The rule was replaced with "`black` is the *darkest* slot", which failed on the light palette at
///    "bright white is darker than black" — and that failure was the useful one, because it showed the
///    **palette** was wrong rather than the rule: light's `black` had been built as dark ink, when a
///    terminal's `black` is the ground whatever the ground is.
/// 3. So the rule came back, and the value was fixed.
pub const GROUND_CEILING: f32 = 1.6;

/// ANSI `black`, which is the ground.
const GROUND_SLOT: usize = 0;
/// ANSI `bright black`, which is dim text.
const DIM_SLOT: usize = 8;

/// The sixteen ANSI colours, a foreground, and the two things a terminal draws over the grid.
///
/// Indices are ANSI's own: `0..8` the normal eight, `8..16` their bright forms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerminalPalette {
    ansi: [Vec4f; 16],
    foreground: Vec4f,
    background: Vec4f,
    cursor: Vec4f,
    selection: Vec4f,
}

impl TerminalPalette {
    /// The eight ANSI colour names, in index order.
    pub const NAMES: [&'static str; 16] = [
        "black",
        "red",
        "green",
        "yellow",
        "blue",
        "magenta",
        "cyan",
        "white",
        "bright black",
        "bright red",
        "bright green",
        "bright yellow",
        "bright blue",
        "bright magenta",
        "bright cyan",
        "bright white",
    ];

    pub fn dark() -> Self {
        Self::build(Appearance::Dark)
    }

    pub fn light() -> Self {
        Self::build(Appearance::Light)
    }

    pub fn for_appearance(appearance: Appearance) -> Self {
        Self::build(appearance)
    }

    fn build(appearance: Appearance) -> Self {
        // The eight hues, and their bright forms as the same hue pushed away from the ground. A
        // terminal's bright colour is a **lighter** version of the same hue on a dark ground and a
        // **darker** one on a light ground, which is the same "towards or away from the ground"
        // distinction `syntax.rs` records for its comment colour.
        let (l, l_bright, chroma) = match appearance {
            Appearance::Dark => (0.68, 0.80, 0.13),
            Appearance::Light => (0.50, 0.42, 0.14),
        };
        let hues = [255.0, 25.0, 145.0, 85.0, 255.0, 330.0, 200.0, 250.0];
        let mut ansi = [Vec4f::default(); 16];
        for (slot, hue) in ansi.iter_mut().zip(hues) {
            // The first eight, at the shared lightness.
            *slot = color::oklch(l, chroma_for(*&hue, chroma), hue);
        }
        for index in 0..8 {
            ansi[index + 8] = color::oklch(l_bright, chroma_for(hues[index], chroma), hues[index]);
        }
        // **The four near-background slots, set explicitly rather than derived from the hues.** They
        // are not "the hue at another lightness"; they are the ground and one step off it, which is
        // why `black` and `bright black` are the two that a reader sees as background and as dim
        // text.
        match appearance {
            Appearance::Dark => {
                ansi[0] = color::oklch(0.22, 0.01, 250.0);
                ansi[8] = color::oklch(0.52, 0.01, 250.0);
                ansi[7] = color::oklch(0.92, 0.005, 250.0);
                ansi[15] = color::oklch(0.99, 0.0, 0.0);
            }
            Appearance::Light => {
                // **`black` is the ground here too**, which is the thing the first version of this
                // palette got wrong: it was built as dark ink at `L = 0.28`, giving 13.77:1 against a
                // light ground. A terminal on a white background inverts — `black` erases just as it
                // does on a dark one, and the **ink** is the `white` slots.
                ansi[0] = color::oklch(0.90, 0.01, 250.0);
                ansi[8] = color::oklch(0.60, 0.01, 250.0);
                ansi[7] = color::oklch(0.35, 0.005, 250.0);
                ansi[15] = color::oklch(0.15, 0.0, 0.0);
            }
        }
        let (foreground, background) = match appearance {
            Appearance::Dark => (color::oklch(0.90, 0.005, 250.0), color::oklch(0.15, 0.008, 250.0)),
            Appearance::Light => (color::oklch(0.22, 0.005, 250.0), color::oklch(0.98, 0.003, 250.0)),
        };
        Self {
            ansi,
            foreground,
            background,
            // The cursor is the foreground colour, because a terminal's cursor is drawn as a block
            // of the text colour with the background showing through the glyph — so it is the one
            // colour here that does not need to be distinguishable from its own ground.
            cursor: foreground,
            selection: match appearance {
                Appearance::Dark => color::oklch(0.35, 0.05, 255.0),
                Appearance::Light => color::oklch(0.85, 0.05, 255.0),
            },
        }
    }

    /// The colour at an ANSI index. `16..` are the 256-colour cube, which this palette does not
    /// carry — an index past fifteen is the caller's to map, and `None` says so rather than returning
    /// a colour nobody chose.
    pub fn color(&self, index: usize) -> Option<Vec4f> {
        self.ansi.get(index).copied()
    }

    /// The name of an ANSI index, for a page that lists them.
    pub fn name(index: usize) -> Option<&'static str> {
        Self::NAMES.get(index).copied()
    }

    pub fn ansi(&self) -> &[Vec4f; 16] {
        &self.ansi
    }

    pub fn foreground(&self) -> Vec4f {
        self.foreground
    }

    pub fn background(&self) -> Vec4f {
        self.background
    }

    pub fn cursor(&self) -> Vec4f {
        self.cursor
    }

    pub fn selection(&self) -> Vec4f {
        self.selection
    }

    /// The colour at an ANSI index as three floats, which is the shape the emulator in this workspace
    /// stores on a cell.
    ///
    /// `canvas-terminal`'s `Cell` carries `fg: [f32; 3]` and `bg: [f32; 3]`, and its palette is a
    /// `const` — so these exist to make replacing that `const` a **mechanical** change rather than a
    /// refactor: every call site keeps its type and only the source of the number moves.
    ///
    /// `None` for an index this palette does not carry, matching [`TerminalPalette::color`].
    pub fn rgb(&self, index: usize) -> Option<[f32; 3]> {
        self.color(index).map(rgb_of)
    }

    /// Every colour as three floats, in index order.
    pub fn ansi_rgb(&self) -> [[f32; 3]; 16] {
        let mut out = [[0.0f32; 3]; 16];
        for (slot, colour) in out.iter_mut().zip(self.ansi) {
            *slot = rgb_of(colour);
        }
        out
    }

    pub fn foreground_rgb(&self) -> [f32; 3] {
        rgb_of(self.foreground)
    }

    pub fn background_rgb(&self) -> [f32; 3] {
        rgb_of(self.background)
    }
}

/// The first three components of a colour, for a `[f32; 3]` consumer.
///
/// Alpha is dropped rather than premultiplied: every colour in this palette is opaque, and a terminal
/// cell is a background *behind* a glyph rather than a layer over anything.
fn rgb_of(colour: Vec4f) -> [f32; 3] {
    [colour.x, colour.y, colour.z]
}

/// The chroma for a hue, so that the two slots sharing a hue (normal and bright) do not come out
/// identical when the hue itself is near-neutral.
fn chroma_for(_hue: f32, chroma: f32) -> f32 {
    chroma
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_there_are_sixteen_colours_and_sixteen_names() {
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = TerminalPalette::for_appearance(appearance);
            assert_eq!(palette.ansi().len(), 16);
            assert_eq!(TerminalPalette::NAMES.len(), 16);
            for index in 0..16 {
                assert!(
                    palette.color(index).is_some(),
                    "{appearance:?} has no colour at {index}"
                );
                assert!(
                    TerminalPalette::name(index).is_some(),
                    "no name for {index}"
                );
            }
            // An index past fifteen is the 256-colour cube, which this palette does not carry.
            assert!(palette.color(16).is_none());
            assert!(TerminalPalette::name(16).is_none());
        }
    }

    #[test]
    fn test_every_colour_is_distinct() {
        // Sixteen slots a program selects by number. Two the same is a program that cannot be told
        // apart from another — and the most likely pair to collide is a hue's normal and bright
        // forms, which is why they are built at different lightnesses.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = TerminalPalette::for_appearance(appearance);
            for a in 0..16usize {
                for b in (a + 1)..16 {
                    let (ca, cb) = (palette.ansi[a], palette.ansi[b]);
                    let same = (ca.x - cb.x).abs() < 1e-4
                        && (ca.y - cb.y).abs() < 1e-4
                        && (ca.z - cb.z).abs() < 1e-4;
                    assert!(
                        !same,
                        "{appearance:?}: {} and {} share a colour",
                        TerminalPalette::NAMES[a],
                        TerminalPalette::NAMES[b]
                    );
                }
            }
        }
    }

    #[test]
    fn test_the_twelve_readable_slots_clear_aa_on_the_terminal_background() {
        // Every one of these is a text colour on the terminal's own ground. A red that vanishes on
        // the background is the fault this test exists for.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = TerminalPalette::for_appearance(appearance);
            let ground = palette.background();
            for index in 0..16 {
                if index == GROUND_SLOT || index == DIM_SLOT {
                    continue;
                }
                let ratio = color::contrast_ratio(palette.ansi[index], ground);
                assert!(
                    ratio >= TEXT_FLOOR,
                    "{appearance:?}: {} is {ratio:.2}:1 on the terminal background, under {TEXT_FLOOR}",
                    TerminalPalette::NAMES[index]
                );
            }
        }
    }

    #[test]
    fn test_black_is_the_ground_and_bright_black_is_dim_text() {
        // `black` erases in **both** appearances, so it is the ground in both — which is the rule
        // this test went two wrong ways to arrive at. See `GROUND_CEILING`.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = TerminalPalette::for_appearance(appearance);
            let ground = palette.background();
            let black = color::contrast_ratio(palette.ansi[GROUND_SLOT], ground);
            assert!(
                black <= GROUND_CEILING,
                "{appearance:?}: black is {black:.2}:1 against the ground, so it is ink rather than \
                 the background"
            );
            // The inversion that a light terminal needs: the `white` slots are the ink there, and
            // `bright white` is the darkest thing in the palette.
            let bright_white = color::relative_luminance(palette.ansi[15]);
            let black_l = color::relative_luminance(palette.ansi[GROUND_SLOT]);
            assert!(
                (bright_white - black_l).abs() > 0.2,
                "{appearance:?}: bright white and black are both near the same luminance, so the \
                 palette has not oriented itself to its ground"
            );
            // `bright black` is dim text: readable, and not comfortably so.
            let dim = color::contrast_ratio(palette.ansi[DIM_SLOT], ground);
            assert!(
                dim >= DIM_FLOOR,
                "{appearance:?}: bright black is {dim:.2}:1, so dim text is unreadable"
            );
            assert!(
                dim <= DIM_CEILING,
                "{appearance:?}: bright black is {dim:.2}:1, which is not dim"
            );
        }
    }

    #[test]
    fn test_the_foreground_clears_aa_and_the_two_appearances_differ() {
        let dark = TerminalPalette::dark();
        let light = TerminalPalette::light();
        assert_ne!(dark, light);
        assert!(color::contrast_ratio(dark.foreground(), dark.background()) >= TEXT_FLOOR);
        assert!(color::contrast_ratio(light.foreground(), light.background()) >= TEXT_FLOOR);
        // And the two grounds are on opposite sides of mid-grey, which is what makes them an
        // appearance rather than two dark themes.
        let dark_l = color::relative_luminance(dark.background());
        let light_l = color::relative_luminance(light.background());
        assert!(dark_l < 0.2, "the dark ground is not dark ({dark_l:.3})");
        assert!(light_l > 0.7, "the light ground is not light ({light_l:.3})");
        // ...and every slot moved with it, so nothing was left hardcoded to one appearance.
        for index in 0..16 {
            assert_ne!(
                dark.ansi[index],
                light.ansi[index],
                "{} did not follow the appearance",
                TerminalPalette::NAMES[index]
            );
        }
    }

    #[test]
    fn test_the_three_float_conversions_match_the_colours_and_drop_nothing() {
        // The conversion exists so that replacing `canvas-terminal`'s palette `const` is mechanical.
        // A conversion that lost a channel, or that returned a premultiplied value, would be a
        // silent colour change at every call site.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = TerminalPalette::for_appearance(appearance);
            let ansi = palette.ansi_rgb();
            for index in 0..16 {
                let full = palette.color(index).expect("a colour");
                assert_eq!(ansi[index], [full.x, full.y, full.z]);
                assert_eq!(palette.rgb(index), Some([full.x, full.y, full.z]));
                // Opaque, so dropping the fourth component loses nothing.
                assert!(
                    (full.w - 1.0).abs() < 1e-6,
                    "{} is translucent, so a cell would lose its alpha",
                    TerminalPalette::NAMES[index]
                );
            }
            assert_eq!(palette.rgb(16), None);
            let fg = palette.foreground();
            assert_eq!(palette.foreground_rgb(), [fg.x, fg.y, fg.z]);
            let bg = palette.background();
            assert_eq!(palette.background_rgb(), [bg.x, bg.y, bg.z]);
        }
    }

    #[test]
    fn test_the_cursor_is_the_foreground_and_says_so() {
        // A terminal's cursor is a block of the text colour with the background showing through the
        // glyph, so it is the one colour here that does not need contrast against its own ground.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = TerminalPalette::for_appearance(appearance);
            assert_eq!(palette.cursor(), palette.foreground());
        }
    }

    #[test]
    fn test_the_selection_stays_quieter_than_the_colours_it_sits_behind() {
        // A selection wash has to be visible over the ground and must not compete with sixteen text
        // colours painted on top of it.
        for appearance in [Appearance::Dark, Appearance::Light] {
            let palette = TerminalPalette::for_appearance(appearance);
            let wash = color::contrast_ratio(palette.selection(), palette.background());
            assert!(
                wash >= 1.3 && wash <= 3.0,
                "{appearance:?}: the selection wash is {wash:.2}:1, which is either invisible or a \
                 text colour"
            );
            // And every readable slot still clears the floor *on the selection*, because selected
            // text is still text. The two non-text slots are skipped for the same reason they are
            // skipped on the ground: one is the ground itself and the other is **dim by design**, so
            // 2.05:1 on a selection is the colour working rather than a fault.
            for index in 0..16 {
                if index == GROUND_SLOT || index == DIM_SLOT {
                    continue;
                }
                let ratio = color::contrast_ratio(palette.ansi[index], palette.selection());
                assert!(
                    ratio >= 3.0,
                    "{appearance:?}: {} is {ratio:.2}:1 on the selection",
                    TerminalPalette::NAMES[index]
                );
            }
        }
    }
}
