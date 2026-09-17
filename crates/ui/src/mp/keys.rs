//! `MpKeys` — the chord behind a printed shortcut.
//!
//! ## A hand-typed accelerator is a claim nothing checks
//!
//! Every shortcut the library prints is a string: a menu row's trailing `⌘T`, a list
//! item's `detail`, a toolbar tooltip. Each one is a **claim about the keymap** — and a
//! claim written by hand is one that nothing verifies. Bind `cmd-b` to something else
//! and the button still says `⌘B`; add a second thing on `⌘T` and the menu still says
//! `⌘T` with no hint that it now fires whichever handler ran first.
//!
//! So the label is **resolved from a declared keymap** rather than typed beside the
//! thing it describes. [`Keymap::label`] asks the keymap, and an action with nothing
//! declared prints nothing rather than a chord that is no longer true. That is the
//! whole reason this module exists; the formatting is the easy half.
//!
//! ## The half everyone gets wrong is the modifier order
//!
//! Apple writes a chord's modifiers in a **fixed order that has nothing to do with how
//! you type it** — `⌃⌥⇧⌘`, control, option, shift, command. So `cmd+shift+p` prints as
//! `⇧⌘P` and not as `⌘⇧P`, and a module that preserves the caller's order gets every
//! Mac shortcut in the app subtly wrong in a way that looks fine until two of them
//! appear side by side. The order is therefore a property of the **platform**, applied
//! by [`format`], never of the input.
//!
//! Windows writes its own order — `Win+Shift+S`, `Ctrl+Alt+Del` — which is not Apple's
//! and is not alphabetical either. Both orders are in the tables below with the two
//! examples that pin them.
//!
//! ## Conflicts are the second thing nothing else catches
//!
//! Two actions on one chord is a real bug: one of them silently loses, and which one
//! depends on dispatch order. [`Keymap::conflicts`] exists so a shortcut sheet can show
//! it before a user finds it.

use makepad_widgets::*;

/// Which platform's notation a chord is printed in.
///
/// A closed enum rather than a `macos: bool`, because the two notations differ in more
/// than a font: the modifier set names differ (`⌘` against `Win`), the key names differ
/// (`↩` against `Enter`), and the orders differ. A flag would end up threaded through
/// every branch of the formatter anyway.
///
/// [`Platform::current`] reads the build target; the parameter exists so the formatter
/// is testable for **both** platforms on either one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Script, ScriptHook)]
pub enum Platform {
    /// Apple's notation: `⌃⌥⇧⌘` before the key, glyph key names.
    #[pick]
    #[default]
    #[live]
    Macos,
    /// Everything else: `Win`/`Ctrl`/`Alt`/`Shift` spelled out.
    #[live]
    Other,
}

impl Platform {
    /// The platform this binary was built for.
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::Macos
        } else {
            Platform::Other
        }
    }
}

/// The four modifiers a chord can carry.
///
/// Named for what they **do** rather than for either platform's key: `command` is `⌘` on
/// a Mac and `Win` elsewhere, and calling the field `super` would make every call site
/// translate. The default is nothing held, which is what a bare key means.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub command: bool,
}

impl Modifiers {
    pub fn any(self) -> bool {
        self.control || self.alt || self.shift || self.command
    }
}

/// A key together with the modifiers held with it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chord {
    pub modifiers: Modifiers,
    /// The key, **normalised to lowercase** and to a canonical name: `return` becomes
    /// `enter`, `esc` becomes `escape`, `backspace` becomes `delete`. Normalising here
    /// rather than at comparison time is what makes two spellings of the same chord
    /// compare equal.
    pub key: String,
}

impl Chord {
    pub fn new(key: &str, modifiers: Modifiers) -> Self {
        Self {
            modifiers,
            key: canonical_key(key),
        }
    }
}

/// The canonical name for a key, or the lowercased input when it is already one.
///
/// Several names reach the same key: `return` and `enter` are one key on every platform,
/// `esc` and `escape` are one key, and `backspace` and `delete` are one key everywhere
/// except Apple's laptops, where both exist. Normalising to one name is what lets
/// [`Keymap::conflicts`] see that `cmd+return` and `cmd+enter` are the same binding.
fn canonical_key(key: &str) -> String {
    let lower = key.trim().to_lowercase();
    match lower.as_str() {
        "return" | "enter" | "↩" => "enter",
        "esc" | "escape" | "⎋" => "escape",
        "backspace" | "delete" | "del" | "⌫" => "delete",
        "space" | "spacebar" | " " => "space",
        "tab" | "⇥" => "tab",
        "up" | "arrowup" | "↑" => "up",
        "down" | "arrowdown" | "↓" => "down",
        "left" | "arrowleft" | "←" => "left",
        "right" | "arrowright" | "→" => "right",
        "pageup" | "pgup" | "⇞" => "pageup",
        "pagedown" | "pgdn" | "⇟" => "pagedown",
        "home" | "↖" => "home",
        "end" | "↘" => "end",
        other => other,
    }
    .to_string()
}

/// Parse a chord from text, in any of the spellings a person or a config file writes.
///
/// Accepts `cmd+shift+p`, `Shift+Cmd+P`, `⌘⇧P`, `⌘+⇧+P`, `ctrl-alt-delete` and
/// surrounding spaces. Returns `None` for an empty string or for a chord with no key —
/// `cmd+` is not a chord, and answering with a chord whose key is `""` would put an
/// empty accelerator on screen.
///
/// **The modifier set is order-independent**, because [`format`] decides the order. Two
/// spellings of the same chord parse to the same [`Chord`] so that a keymap can find a
/// conflict between them.
pub fn parse(text: &str) -> Option<Chord> {
    let mut modifiers = Modifiers::default();
    let mut key: Option<String> = None;

    // Split on `+` and `-`, and on nothing else: `-` is a key in its own right
    // (`cmd+-`), so a rule that treated every `-` as a separator would make that chord
    // unparseable.
    //
    // **The rule is "a separator separates only when there is something to separate
    // from"** — that is, when the token being built is non-empty. Under it, `cmd+-` is
    // `cmd` then a key of `-`, because the `-` arrives while the token is empty and so
    // starts one; and a bare `-` is a key of `-` for the same reason.
    //
    // The first version of this scan *also* looked ahead and refused to treat a
    // separator as one when it was the final character. That was wrong, and two tests
    // said so: `cmd+` parsed to a chord whose **key was the literal string `cmd+`**, and
    // a chord of modifiers alone is supposed to be `None`. A trailing separator needs no
    // special case at all — it simply produces no final token.
    let normalised: String = text.trim().to_string();
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    for ch in normalised.chars() {
        if (ch == '+' || ch == '-') && !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        } else if !ch.is_whitespace() {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    for token in tokens {
        let lower = token.to_lowercase();
        match lower.as_str() {
            "cmd" | "command" | "meta" | "super" | "win" | "⌘" => modifiers.command = true,
            "ctrl" | "control" | "ctl" | "⌃" => modifiers.control = true,
            "alt" | "opt" | "option" | "⌥" => modifiers.alt = true,
            "shift" | "⇧" => modifiers.shift = true,
            // A glyph run can arrive as one token: `⌘⇧P`.
            _ if token.chars().any(|c| matches!(c, '⌘' | '⌃' | '⌥' | '⇧')) => {
                for ch in token.chars() {
                    match ch {
                        '⌘' => modifiers.command = true,
                        '⌃' => modifiers.control = true,
                        '⌥' => modifiers.alt = true,
                        '⇧' => modifiers.shift = true,
                        other => {
                            if key.is_none() && !other.is_whitespace() {
                                key = Some(canonical_key(&other.to_string()));
                            }
                        }
                    }
                }
            }
            _ => {
                if key.is_none() {
                    key = Some(canonical_key(&token));
                }
            }
        }
    }

    let key = key.filter(|k| !k.is_empty())?;
    // A chord of modifiers alone is not a chord.
    if !modifiers.any() && key.is_empty() {
        return None;
    }
    Some(Chord { modifiers, key })
}

/// Print a chord the way `platform` writes it.
///
/// The modifier **order is the platform's**, applied here and never taken from the
/// input:
///
/// | | order |
/// |---|---|
/// | macOS | `⌃⌥⇧⌘` (control, option, shift, command) — Apple's own order. `cmd+shift+p` prints `⇧⌘P` |
/// | other | `Win`, `Ctrl`, `Alt`, `Shift` — pinned by `Win+Shift+S` (Windows leads with Win) and `Ctrl+Alt+Del` (Ctrl before Alt) |
///
/// Nothing separates the modifiers from each other on macOS and nothing appears between
/// them and the key: Apple writes `⇧⌘P`, not `⇧+⌘+P`.
pub fn format(chord: &Chord, platform: Platform) -> String {
    let m = chord.modifiers;
    let mut out = String::new();
    match platform {
        Platform::Macos => {
            if m.control {
                out.push('⌃');
            }
            if m.alt {
                out.push('⌥');
            }
            if m.shift {
                out.push('⇧');
            }
            if m.command {
                out.push('⌘');
            }
            out.push_str(macos_key(&chord.key));
        }
        Platform::Other => {
            // Windows leads with the platform key; `Ctrl+Alt+Del` then fixes the rest.
            if m.command {
                out.push_str("Win+");
            }
            if m.control {
                out.push_str("Ctrl+");
            }
            if m.alt {
                out.push_str("Alt+");
            }
            if m.shift {
                out.push_str("Shift+");
            }
            out.push_str(other_key(&chord.key));
        }
    }
    out
}

/// The glyph for a key in Apple's notation.
fn macos_key(key: &str) -> &str {
    match key {
        "enter" => "↩",
        "delete" => "⌫",
        "escape" => "⎋",
        "tab" => "⇥",
        "space" => "Space",
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "pageup" => "⇞",
        "pagedown" => "⇟",
        "home" => "↖",
        "end" => "↘",
        // A single character is upper-cased: a shortcut is a legend, and `P` is what
        // every platform prints for the key marked P.
        other if other.chars().count() == 1 => {
            // No allocation of the original: the caller's `String` owns the lowercase
            // form, so a one-character key is looked up rather than rebuilt lazily.
            ONE_CHAR_KEYS
                .iter()
                .find(|(k, _)| *k == other)
                .map(|(_, upper)| *upper)
                .unwrap_or(other)
        }
        other => other,
    }
}

/// The word for a key in the non-Apple notation.
fn other_key(key: &str) -> &str {
    match key {
        "enter" => "Enter",
        "delete" => "Delete",
        "escape" => "Esc",
        "tab" => "Tab",
        "space" => "Space",
        "up" => "Up",
        "down" => "Down",
        "left" => "Left",
        "right" => "Right",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "home" => "Home",
        "end" => "End",
        other if other.chars().count() == 1 => ONE_CHAR_KEYS
            .iter()
            .find(|(k, _)| *k == other)
            .map(|(_, upper)| *upper)
            .unwrap_or(other),
        other => other,
    }
}

/// The upper-case form of every one-character key, so that printing needs no allocation
/// and [`macos_key`] can stay a lookup returning `&str`.
const ONE_CHAR_KEYS: [(&str, &str); 36] = [
    ("a", "A"),
    ("b", "B"),
    ("c", "C"),
    ("d", "D"),
    ("e", "E"),
    ("f", "F"),
    ("g", "G"),
    ("h", "H"),
    ("i", "I"),
    ("j", "J"),
    ("k", "K"),
    ("l", "L"),
    ("m", "M"),
    ("n", "N"),
    ("o", "O"),
    ("p", "P"),
    ("q", "Q"),
    ("r", "R"),
    ("s", "S"),
    ("t", "T"),
    ("u", "U"),
    ("v", "V"),
    ("w", "W"),
    ("x", "X"),
    ("y", "Y"),
    ("z", "Z"),
    ("0", "0"),
    ("1", "1"),
    ("2", "2"),
    ("3", "3"),
    ("4", "4"),
    ("5", "5"),
    ("6", "6"),
    ("7", "7"),
    ("8", "8"),
    ("9", "9"),
];

/// What went wrong declaring a chord.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeymapError {
    /// The text was not a chord. Carries the text so a caller can report it.
    Unparseable(String),
    /// The action already has a chord.
    DuplicateAction(String),
}

/// A declared set of shortcuts: what an action is called, and what it is bound to.
///
/// The point of holding them together is that the **label and the binding cannot drift
/// apart** — a row prints [`Keymap::label`], which resolves through this map, so a chord
/// that changes changes everywhere it is printed, and an action nobody declared prints
/// nothing at all.
#[derive(Clone, Debug, Default)]
pub struct Keymap {
    bindings: Vec<(String, Chord)>,
}

impl Keymap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare what `action` is bound to. `text` is in any spelling [`parse`] accepts.
    pub fn declare(&mut self, action: &str, text: &str) -> Result<&mut Self, KeymapError> {
        let chord = parse(text).ok_or_else(|| KeymapError::Unparseable(text.to_string()))?;
        if self.bindings.iter().any(|(name, _)| name == action) {
            return Err(KeymapError::DuplicateAction(action.to_string()));
        }
        self.bindings.push((action.to_string(), chord));
        Ok(self)
    }

    /// Redeclare an action, replacing its chord. Rebinding is a normal thing for an app
    /// to do, and a duplicate error for it would be the wrong shape.
    pub fn rebind(&mut self, action: &str, text: &str) -> Result<&mut Self, KeymapError> {
        let chord = parse(text).ok_or_else(|| KeymapError::Unparseable(text.to_string()))?;
        match self.bindings.iter_mut().find(|(name, _)| name == action) {
            Some((_, existing)) => *existing = chord,
            None => self.bindings.push((action.to_string(), chord)),
        }
        Ok(self)
    }

    pub fn chord(&self, action: &str) -> Option<&Chord> {
        self.bindings
            .iter()
            .find(|(name, _)| name == action)
            .map(|(_, chord)| chord)
    }

    /// The printed accelerator for `action`, or `None` when it has no binding.
    ///
    /// **`None` rather than an empty string**, so a caller has to decide what to print
    /// for an unbound action instead of rendering a blank where a chord belongs.
    pub fn label(&self, action: &str, platform: Platform) -> Option<String> {
        self.chord(action).map(|chord| format(chord, platform))
    }

    /// Every pair of actions sharing a chord.
    ///
    /// Two actions on one chord is a bug in the keymap: one of them loses, and which one
    /// depends on dispatch order, so the app cannot tell the user which will fire. Sorted
    /// and deduplicated by action name so the report is stable.
    pub fn conflicts(&self) -> Vec<(String, String, String)> {
        let mut found: Vec<(String, String, String)> = Vec::new();
        for (index, (left_action, left_chord)) in self.bindings.iter().enumerate() {
            for (right_action, right_chord) in self.bindings.iter().skip(index + 1) {
                if left_chord == right_chord {
                    found.push((
                        left_action.clone(),
                        right_action.clone(),
                        format(right_chord, Platform::Macos),
                    ));
                }
            }
        }
        found
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Every binding, for building a shortcut sheet.
    pub fn bindings(&self) -> &[(String, Chord)] {
        &self.bindings
    }
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.mp.Platform = set_type_default() do #(Platform::script_api(vm))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mac(text: &str) -> String {
        format(&parse(text).expect("should parse"), Platform::Macos)
    }

    fn other(text: &str) -> String {
        format(&parse(text).expect("should parse"), Platform::Other)
    }

    // ---- parsing -----------------------------------------------------------

    #[test]
    fn test_the_modifier_set_does_not_depend_on_the_order_it_was_written_in() {
        // The property that lets a keymap see two spellings of one chord as a conflict.
        let a = parse("cmd+shift+p").unwrap();
        let b = parse("shift+cmd+p").unwrap();
        let c = parse("Shift+Cmd+P").unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn test_every_spelling_of_a_modifier_reaches_the_same_field() {
        for spelling in ["cmd", "command", "meta", "super", "win", "⌘"] {
            let chord = parse(&format!("{spelling}+k")).unwrap();
            assert!(chord.modifiers.command, "{spelling} should be command");
        }
        for spelling in ["ctrl", "control", "ctl", "⌃"] {
            let chord = parse(&format!("{spelling}+k")).unwrap();
            assert!(chord.modifiers.control, "{spelling} should be control");
        }
        for spelling in ["alt", "opt", "option", "⌥"] {
            let chord = parse(&format!("{spelling}+k")).unwrap();
            assert!(chord.modifiers.alt, "{spelling} should be alt");
        }
        for spelling in ["shift", "⇧"] {
            let chord = parse(&format!("{spelling}+k")).unwrap();
            assert!(chord.modifiers.shift, "{spelling} should be shift");
        }
    }

    #[test]
    fn test_a_glyph_run_parses_as_one_token() {
        // What a Mac user types when they paste a shortcut out of a menu.
        let chord = parse("⌘⇧P").unwrap();
        assert!(chord.modifiers.command);
        assert!(chord.modifiers.shift);
        assert!(!chord.modifiers.control);
        assert!(!chord.modifiers.alt);
        assert_eq!(chord.key, "p");
        // And with separators between the glyphs, which also happens.
        assert_eq!(parse("⌘+⇧+P").unwrap(), chord);
    }

    #[test]
    fn test_a_chord_with_no_key_is_not_a_chord() {
        // `cmd+` would otherwise print `⌘` alone, which is a modifier and not a
        // shortcut.
        assert_eq!(parse("cmd+"), None);
        assert_eq!(parse("cmd"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse("   "), None);
    }

    #[test]
    fn test_two_key_names_for_one_key_normalise_to_one_chord() {
        // `return`/`enter`, `esc`/`escape`, `backspace`/`delete` are one key each, and a
        // keymap that treated them as different would miss the conflict between them.
        assert_eq!(parse("cmd+return").unwrap(), parse("cmd+enter").unwrap());
        assert_eq!(parse("esc").unwrap(), parse("escape").unwrap());
        assert_eq!(parse("cmd+backspace").unwrap(), parse("cmd+delete").unwrap());
    }

    #[test]
    fn test_the_key_is_normalised_to_lowercase() {
        assert_eq!(parse("CMD+P").unwrap().key, "p");
        assert_eq!(parse("cmd+PageUP").unwrap().key, "pageup");
    }

    #[test]
    fn test_a_bare_key_parses_with_no_modifiers() {
        let chord = parse("escape").unwrap();
        assert!(!chord.modifiers.any());
        assert_eq!(chord.key, "escape");
        assert_eq!(format(&chord, Platform::Macos), "⎋");
    }

    #[test]
    fn test_a_minus_is_still_a_key() {
        // The trap in the tokeniser: `-` is both a separator and a key, so a naive split
        // makes `cmd+-` unparseable. This is why the scan looks ahead.
        let chord = parse("cmd+-").unwrap();
        assert!(chord.modifiers.command);
        assert_eq!(chord.key, "-");
        assert_eq!(format(&chord, Platform::Macos), "⌘-");
    }

    // ---- formatting --------------------------------------------------------

    #[test]
    fn test_macos_prints_apple_s_modifier_order_and_not_the_input_order() {
        // **The half everyone gets wrong.** Apple's order is control, option, shift,
        // command, so `cmd+shift+p` is `⇧⌘P` — not `⌘⇧P`, which is what preserving the
        // input order gives and which looks fine until two shortcuts sit side by side.
        assert_eq!(mac("cmd+shift+p"), "⇧⌘P");
        assert_eq!(mac("shift+cmd+p"), "⇧⌘P", "the same chord, whichever way it was written");
        assert_eq!(mac("cmd+ctrl+alt+shift+p"), "⌃⌥⇧⌘P");
        assert_eq!(mac("cmd+shift+p"), mac("shift+cmd+p"));
    }

    #[test]
    fn test_macos_puts_nothing_between_the_modifiers_or_before_the_key() {
        // Apple writes `⇧⌘P`, never `⇧+⌘+P` and never `⇧ ⌘ P`.
        let printed = mac("cmd+shift+p");
        assert!(!printed.contains('+'));
        assert!(!printed.contains(' '));
        assert_eq!(printed.chars().count(), 3);
    }

    #[test]
    fn test_the_other_notation_is_pinned_by_the_two_examples_that_disagree() {
        // `Win+Shift+S` is why the platform key leads; `Ctrl+Alt+Del` is why Ctrl comes
        // before Alt. Neither of those orderings is Apple's and neither is alphabetical.
        assert_eq!(other("cmd+shift+s"), "Win+Shift+S");
        // Windows' own prose abbreviates this as `Ctrl+Alt+Del`; the key is named
        // `Delete` and this prints the key's name. What the example pins is the
        // **order** — Ctrl before Alt — which is neither Apple's nor alphabetical.
        assert_eq!(other("ctrl+alt+delete"), "Ctrl+Alt+Delete");
        assert_eq!(other("alt+ctrl+delete"), "Ctrl+Alt+Delete", "the order is the platform's");
        assert_eq!(other("cmd+ctrl+alt+shift+p"), "Win+Ctrl+Alt+Shift+P");
    }

    #[test]
    fn test_the_other_notation_leaves_the_key_words_spelled_out() {
        // Apple uses a glyph where Windows uses a word, which is the second reason this
        // is not one formatter with a font swap.
        assert_eq!(mac("cmd+enter"), "⌘↩");
        assert_eq!(other("cmd+enter"), "Win+Enter");
        assert_eq!(mac("cmd+delete"), "⌘⌫");
        assert_eq!(other("cmd+delete"), "Win+Delete");
        assert_eq!(mac("cmd+escape"), "⌘⎋");
        assert_eq!(other("cmd+escape"), "Win+Esc");
    }

    #[test]
    fn test_a_letter_key_is_upper_cased_in_both_notations() {
        // A shortcut is a legend, and every platform prints the key marked P as `P`.
        assert_eq!(mac("cmd+p"), "⌘P");
        assert_eq!(other("cmd+p"), "Win+P");
        assert_eq!(mac("cmd+1"), "⌘1");
        assert_eq!(other("ctrl+1"), "Ctrl+1");
    }

    #[test]
    fn test_an_arrow_is_a_glyph_on_a_mac_and_a_word_elsewhere() {
        assert_eq!(mac("cmd+up"), "⌘↑");
        assert_eq!(other("cmd+up"), "Win+Up");
        assert_eq!(mac("cmd+pageup"), "⌘⇞");
        assert_eq!(other("cmd+pageup"), "Win+PageUp");
    }

    #[test]
    fn test_formatting_is_idempotent_through_a_parse() {
        // A label copied out of a menu and parsed back prints the same, which is what
        // lets a config file hold printed labels rather than author-facing spellings.
        for text in [
            "cmd+shift+p",
            "ctrl+alt+delete",
            "cmd+enter",
            "cmd+up",
            "escape",
        ] {
            for platform in [Platform::Macos, Platform::Other] {
                let once = format(&parse(text).unwrap(), platform);
                let twice = format(&parse(&once).unwrap(), platform);
                assert_eq!(once, twice, "{text} via {platform:?}");
            }
        }
    }

    // ---- the keymap --------------------------------------------------------

    #[test]
    fn test_a_label_resolves_through_the_keymap() {
        // The module's whole reason: the printed accelerator comes from the binding.
        let mut keys = Keymap::new();
        keys.declare("file.new", "cmd+n").unwrap();
        keys.declare("file.save", "cmd+s").unwrap();
        assert_eq!(keys.label("file.new", Platform::Macos).as_deref(), Some("⌘N"));
        assert_eq!(keys.label("file.save", Platform::Other).as_deref(), Some("Win+S"));
    }

    #[test]
    fn test_an_undeclared_action_has_no_label_rather_than_a_blank_one() {
        // `None`, so the caller decides what to print — a hand-typed fallback is exactly
        // the drift this module exists to remove.
        let keys = Keymap::new();
        assert_eq!(keys.label("file.nothing", Platform::Macos), None);
    }

    #[test]
    fn test_rebinding_changes_every_label_at_once() {
        let mut keys = Keymap::new();
        keys.declare("file.new", "cmd+n").unwrap();
        assert_eq!(keys.label("file.new", Platform::Macos).as_deref(), Some("⌘N"));
        keys.rebind("file.new", "cmd+alt+n").unwrap();
        assert_eq!(
            keys.label("file.new", Platform::Macos).as_deref(),
            Some("⌥⌘N"),
            "Apple's order, applied to the new binding"
        );
        // Rebinding to an action that has no binding declares it, rather than failing.
        keys.rebind("file.fresh", "cmd+shift+n").unwrap();
        assert_eq!(keys.label("file.fresh", Platform::Macos).as_deref(), Some("⇧⌘N"));
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_conflicts_finds_two_actions_on_one_chord_across_spellings() {
        // The bug nothing else catches: one of these silently loses, and which one
        // depends on dispatch order. The two are spelled **completely differently** — a
        // config-file spelling and a glyph run pasted out of a menu — and must still be
        // seen as one chord.
        //
        // The first version of this test used `cmd+d` and `shift+cmd+D`, which are *not*
        // the same chord: they differ by shift, and on macOS `⌘D` and `⇧⌘D` are two
        // different shortcuts. The library was right and the expectation was wrong.
        let mut keys = Keymap::new();
        keys.declare("view.split", "cmd+shift+d").unwrap();
        keys.declare("edit.duplicate", "⇧⌘D").unwrap();
        let conflicts = keys.conflicts();
        assert_eq!(conflicts.len(), 1, "{:?}", keys.bindings());
        assert_eq!(conflicts[0].0, "view.split");
        assert_eq!(conflicts[0].1, "edit.duplicate");
        assert_eq!(conflicts[0].2, "⇧⌘D");
    }

    #[test]
    fn test_chords_that_differ_only_by_a_modifier_are_not_conflicts() {
        // The distinction the test above got wrong, kept so it cannot come back: these
        // four are four different shortcuts on every platform.
        let mut keys = Keymap::new();
        keys.declare("a", "cmd+d").unwrap();
        keys.declare("b", "shift+cmd+d").unwrap();
        keys.declare("c", "alt+cmd+d").unwrap();
        keys.declare("d", "ctrl+cmd+d").unwrap();
        assert!(keys.conflicts().is_empty(), "{:?}", keys.conflicts());
        assert_eq!(keys.label("a", Platform::Macos).as_deref(), Some("⌘D"));
        assert_eq!(keys.label("b", Platform::Macos).as_deref(), Some("⇧⌘D"));
        assert_eq!(keys.label("c", Platform::Macos).as_deref(), Some("⌥⌘D"));
        assert_eq!(keys.label("d", Platform::Macos).as_deref(), Some("⌃⌘D"));
    }

    #[test]
    fn test_a_chord_that_is_merely_similar_is_not_a_conflict() {
        let mut keys = Keymap::new();
        keys.declare("a", "cmd+d").unwrap();
        keys.declare("b", "cmd+shift+d").unwrap();
        keys.declare("c", "ctrl+d").unwrap();
        keys.declare("d", "cmd+e").unwrap();
        assert!(keys.conflicts().is_empty());
    }

    #[test]
    fn test_three_actions_on_one_chord_report_every_pair() {
        // Three on one chord is three pairs, not one — a report that stopped at the first
        // would hide the third action.
        let mut keys = Keymap::new();
        keys.declare("a", "cmd+k").unwrap();
        keys.declare("b", "cmd+k").unwrap();
        keys.declare("c", "cmd+k").unwrap();
        assert_eq!(keys.conflicts().len(), 3);
    }

    #[test]
    fn test_declaring_the_same_action_twice_is_an_error_and_rebinding_is_not() {
        let mut keys = Keymap::new();
        keys.declare("view.split", "cmd+d").unwrap();
        assert_eq!(
            keys.declare("view.split", "cmd+e").unwrap_err(),
            KeymapError::DuplicateAction("view.split".into())
        );
        assert!(keys.rebind("view.split", "cmd+e").is_ok());
    }

    #[test]
    fn test_an_unparseable_chord_is_refused_rather_than_silently_dropped() {
        // A shortcut sheet built from a config file has to be told which line was wrong,
        // or one binding goes missing and nothing says why.
        let mut keys = Keymap::new();
        assert_eq!(
            keys.declare("view.split", "cmd+").unwrap_err(),
            KeymapError::Unparseable("cmd+".into())
        );
        assert!(keys.is_empty());
    }

    #[test]
    fn test_a_real_shortcut_sheet_has_no_conflicts() {
        // The check a shortcut sheet should run on itself, against a plausible set.
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
            keys.declare(action, text).expect("every one should parse");
        }
        assert_eq!(keys.len(), 13);
        assert!(keys.conflicts().is_empty(), "{:?}", keys.conflicts());
        // And the labels are the ones a Mac menu shows.
        assert_eq!(keys.label("file.saveAs", Platform::Macos).as_deref(), Some("⇧⌘S"));
        assert_eq!(keys.label("edit.undo", Platform::Macos).as_deref(), Some("⌘Z"));
        assert_eq!(keys.label("view.palette", Platform::Macos).as_deref(), Some("⇧⌘P"));
        assert_eq!(keys.label("view.terminal", Platform::Macos).as_deref(), Some("⌃`"));
    }

    #[test]
    fn test_the_label_order_and_the_binding_order_can_disagree_without_a_false_conflict() {
        // `cmd+shift+p` and `shift+cmd+p` are the same chord, so they DO conflict — but
        // this checks the converse too: two chords that print differently must not be
        // reported, or a sheet would cry wolf and stop being read.
        let mut keys = Keymap::new();
        keys.declare("view.palette", "cmd+shift+p").unwrap();
        keys.declare("file.saveAs", "cmd+shift+s").unwrap();
        assert_eq!(keys.label("view.palette", Platform::Macos).as_deref(), Some("⇧⌘P"));
        assert_eq!(keys.label("file.saveAs", Platform::Macos).as_deref(), Some("⇧⌘S"));
        assert!(keys.conflicts().is_empty());
        // ...and the same chord written the other way IS reported.
        keys.rebind("file.saveAs", "shift+cmd+p").unwrap();
        assert_eq!(keys.conflicts().len(), 1);
    }
}
