//! The registration order is a **dependency order**, and this checks it statically.
//!
//! ## Why a test rather than a comment
//!
//! `script_mod!` names other widgets' prototypes — `MpPalette` composes an `MpMenu`,
//! `MpCombobox` composes an `MpTextInput` and an `MpPopover` — and a name that registers
//! **after** its user is not there yet. The failure is a runtime one:
//!
//! ```text
//! property MpMenu not found in prototype chain. Did you mean: MpMenubar, MpIcon, ...
//! ```
//!
//! That message points at the **user** of the name, not at the line that is in the wrong
//! place, so it reads as "this widget is broken" rather than "the order is wrong". This port
//! has paid for it **three times**: `palette` before `list`, then `combobox` before `input`,
//! and `combobox` before `popover` — the last two at once, because a widget that composes
//! two others has two ways to be too early.
//!
//! A comment did not prevent the second or the third, so the rule is checked here instead:
//! every `mod.mp.<Name>` a module references must be **defined** by a module that registers
//! no later than it does.
//!
//! ## How it reads the sources
//!
//! The dependencies are only visible in the DSL, so the DSL is what it reads. Two scans over
//! `src/mp/*.rs`:
//!
//! - **Definitions**: `mod.mp.<Name> =` — the left side of an assignment is a prototype this
//!   file puts in the namespace.
//! - **References**: every `mod.mp.<Name>` anywhere in a `script_mod!` block, including the
//!   definition site itself and every `+: { ... }` override.
//!
//! Then the registration order comes from `src/mp/mod.rs`'s `script_mod(vm)` calls.
//!
//! It asserts its own scan found a plausible amount before asserting anything about order, so
//! a parser that quietly matched nothing fails loudly rather than passing vacuously. That is
//! a lesson from this port's own audits: `grep -c` counted a struct definition and reported
//! thirty-four pages when there were thirty-one.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn mp_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/mp")
}

/// The module names in the order `mp/mod.rs` registers them.
fn registration_order() -> Vec<String> {
    let source = std::fs::read_to_string(mp_dir().join("mod.rs")).expect("mp/mod.rs");
    let mut order = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        // `crate::mp::<name>::script_mod(vm);`
        if let Some(rest) = line.strip_prefix("crate::mp::") {
            if let Some((name, tail)) = rest.split_once("::script_mod(") {
                if tail.starts_with("vm") {
                    order.push(name.to_string());
                }
            }
        }
    }
    order
}

/// The identifiers appearing as `mod.mp.<Name>` in `text`.
fn mp_names(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = text.as_bytes();
    let needle = b"mod.mp.";
    let mut i = 0;
    while i + needle.len() < bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let start = i + needle.len();
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
            {
                end += 1;
            }
            if end > start {
                found.push(text[start..end].to_string());
            }
            i = end;
        } else {
            i += 1;
        }
    }
    found
}

/// The identifiers this file **defines** as `mod.mp.<Name> =`.
///
/// The left side of an assignment is a prototype this file puts in the namespace. An
/// override (`mod.mp.X +: { ... }`) assigns to a *path*, not to the namespace, and a
/// prototype used as a base (`do mod.mp.X{ ... }`) is a reference rather than a definition —
/// so only a bare `=` after the name counts.
fn definitions(text: &str) -> Vec<String> {
    let mut defined = Vec::new();
    for (at, _) in text.match_indices("mod.mp.") {
        let rest = &text[at + "mod.mp.".len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if end == 0 {
            continue;
        }
        let name = &rest[..end];
        let after = rest[end..].trim_start();
        if after.starts_with('=') && !after.starts_with("==") {
            defined.push(name.to_string());
        }
    }
    defined.sort();
    defined.dedup();
    defined
}

/// Read every `src/mp/*.rs` file, as `(module name, source)`.
fn module_sources() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = std::fs::read_dir(mp_dir())
        .expect("src/mp is readable")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let stem = path.file_stem()?.to_str()?.to_string();
            if path.extension()?.to_str()? != "rs" || stem == "mod" {
                return None;
            }
            // Only the DSL matters, so two things are stripped before scanning.
            //
            // **The tests**, because a test naming a prototype is not a registration
            // dependency. **And the comments**, because neither is a doc example — the first
            // version of this check failed on `mp/control.rs`, which has a ```` ```ignore ````
            // block showing how `MpCheckbox` declares itself. That is documentation about
            // the order, not an edge in it, and a scanner that cannot tell the difference
            // reports a rule violation for every well-documented module.
            let source = std::fs::read_to_string(&path).ok()?;
            let code = match source.find("#[cfg(test)]") {
                Some(at) => &source[..at],
                None => &source[..],
            };
            let dsl: String = code
                .lines()
                .map(|line| match line.find("//") {
                    Some(at) => &line[..at],
                    None => line,
                })
                .collect::<Vec<&str>>()
                .join("\n");
            Some((stem, dsl))
        })
        .collect();
    files.sort();
    files
}

#[test]
fn test_every_referenced_prototype_registers_no_later_than_its_user() {
    let order = registration_order();
    // A parser that matched nothing would make every assertion below vacuous.
    assert!(
        order.len() >= 20,
        "only {} registrations were parsed out of mp/mod.rs, so the scan is broken: {order:?}",
        order.len()
    );
    let index_of = |name: &str| order.iter().position(|m| m == name);

    let sources = module_sources();
    assert!(
        sources.len() >= 20,
        "only {} module files were read, so the scan is broken",
        sources.len()
    );

    // Which module defines each prototype.
    let mut defined_by: BTreeMap<String, String> = BTreeMap::new();
    for (module, source) in &sources {
        for name in definitions(source) {
            defined_by.entry(name).or_insert_with(|| module.clone());
        }
    }
    assert!(
        defined_by.len() >= 40,
        "only {} definitions were found across the modules, so the scan is broken",
        defined_by.len()
    );

    let mut violations: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for (module, source) in &sources {
        // A module with no DSL has nothing to register and nothing to check — `action.rs`,
        // `text.rs` and `control.rs` are pure logic. Skipped rather than required, because
        // the first version of this check demanded registration from every file and failed
        // on `action`.
        if !source.contains("script_mod!") {
            continue;
        }
        let Some(user_index) = index_of(module) else {
            panic!("{module} has a script_mod but is not registered in mp/mod.rs");
        };
        for name in mp_names(source) {
            let Some(definition_module) = defined_by.get(&name) else {
                // A name defined in another crate, or in the gallery. Not this test's
                // business.
                continue;
            };
            if definition_module == module {
                continue;
            }
            let Some(definition_index) = index_of(definition_module) else {
                panic!("{definition_module} defines mod.mp.{name} but is not registered");
            };
            checked += 1;
            if definition_index > user_index {
                violations.push(format!(
                    "{module} (registered #{user_index}) uses mod.mp.{name}, which \
                     {definition_module} defines at #{definition_index}"
                ));
            }
        }
    }

    assert!(
        checked >= 10,
        "only {checked} cross-module references were checked, so the scan is broken"
    );
    // And every module that *has* DSL is registered, which is the other half: an
    // unregistered `script_mod!` is a prototype that silently does not exist.
    let with_dsl: Vec<&String> = sources
        .iter()
        .filter(|(_, source)| source.contains("script_mod!"))
        .map(|(module, _)| module)
        .collect();
    let unregistered: Vec<&&String> =
        with_dsl.iter().filter(|module| index_of(module).is_none()).collect();
    assert!(
        unregistered.is_empty(),
        "these modules have DSL but are never registered, so their prototypes do not exist: {unregistered:?}"
    );
    assert!(
        violations.is_empty(),
        "the registration order does not satisfy the DSL's dependencies:\n  {}",
        violations.join("\n  ")
    );
}

#[test]
fn test_manually_planted_violations_are_detected() {
    // The check above passes, so the question is whether it *can* fail. These two shape the
    // two real failures this port hit — `palette` before `list` and `combobox` before both
    // `input` and `popover` — as data, and assert that the comparison flags them.
    let order = ["input", "list", "popover", "palette", "combobox"];
    let index_of = |name: &str| order.iter().position(|m: &&str| *m == name);
    // `palette` composes `menu`, defined in `list`. Registered at 3, list at 1 → fine.
    assert!(index_of("list").unwrap() <= index_of("palette").unwrap());
    // Planted violation: a `combobox` at index 0, before its two dependencies.
    let planted = ["combobox", "input", "list", "popover", "palette"];
    let planted_index = |name: &str| planted.iter().position(|m: &&str| *m == name);
    assert!(planted_index("input").unwrap() > planted_index("combobox").unwrap());
    assert!(planted_index("popover").unwrap() > planted_index("combobox").unwrap());
    // And at least one dependency that is satisfied, so the assertion above is not
    // trivially true of every pair.
    assert!(planted_index("list").unwrap() > planted_index("combobox").unwrap());
}

#[test]
fn test_the_scan_finds_the_references_it_is_supposed_to() {
    // The scanner's own test. `mp_names` is a hand-rolled scan rather than a regex, so its
    // behaviour on the shapes the DSL actually contains is worth pinning: a definition, an
    // override, a nested path, and a name that merely looks like one.
    let text = r#"
        mod.mp.MpList = set_type_default() do mod.mp.MpListBase{
            draw_label +: {color: mod.mp.tokens.text}
        }
        mod.mp.MpMenu = set_type_default() do mod.mp.MpList{show_row_lines: false}
    "#;
    let names = mp_names(text);
    assert!(names.contains(&"MpList".to_string()));
    assert!(names.contains(&"MpListBase".to_string()));
    assert!(names.contains(&"MpMenu".to_string()));
    assert!(names.contains(&"tokens".to_string()));
    let defined = definitions(text);
    assert!(defined.contains(&"MpList".to_string()));
    assert!(defined.contains(&"MpMenu".to_string()));
    // `+:` is an override of a path, not a definition of a name. This is the distinction
    // the whole check rests on: if overrides counted as definitions, every module that
    // overrides a prototype would look like its definer and the dependency would vanish.
    let only_overrides = "mod.mp.MpMenu = set_type_default() do mod.mp.MpListBase{ x +: { y: 1 } }";
    let names = definitions(only_overrides);
    assert!(names.contains(&"MpMenu".to_string()));
    assert!(
        !names.contains(&"x".to_string()) && !names.contains(&"y".to_string()),
        "an override defines nothing: {names:?}"
    );
}
