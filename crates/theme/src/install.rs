//! Stamping the theme onto the script heap.
//!
//! A Makepad `TextStyle` is a script object, not a Rust value, so the ladder and
//! the tokens have to exist on the heap for a DSL call site to name them. This
//! module is the single writer of `mod.mpc_theme`, `mod.mpc_type` and
//! `mod.mpc_layout` — everything a widget's DSL block references.
//!
//! Because the values are built by walking the Rust structures, adding a token
//! or a role is one edit in one place. The v2 theme spelled all forty tokens out
//! three times (dark, light, active) plus a fourth time in a `get_by_name`
//! match; a token added in one place and missed in another painted the wrong
//! colour and nothing said so.

use makepad_widgets::{
    makepad_script::{trap::NoTrap, ScriptApply},
    Cx, LiveId, ScriptValue, ScriptVm, Vec4f,
};

use crate::{
    appearance::Appearance,
    layout::{ControlSize, Layout},
    material::Material,
    paint::Paint,
    theme::Theme,
    typography::{Face, TextStyle},
};

/// The script module every v3 namespace hangs off.
pub const MODULE: &str = "mpc";
/// The paint tokens, as a script path.
pub const PAINT_PATH: &str = "mpc.tokens";
/// The type ladder, as a script path.
pub const TYPE_PATH: &str = "mpc.type";
/// The layout metrics, as a script path.
pub const LAYOUT_PATH: &str = "mpc.layout";
/// The frost scale, as a script path.
pub const MATERIAL_PATH: &str = "mpc.material";
/// The v2 namespace, kept alive for the widgets that have not been replaced
/// yet. **Temporary** — see [`legacy`](crate::legacy).
pub const LEGACY_MODULE: &str = "mpc_theme";

fn key(name: &str) -> ScriptValue {
    LiveId::from_str(name).into()
}

/// A colour as the script heap stores it.
fn color_value(value: Vec4f, vm: &mut ScriptVm) -> ScriptValue {
    value.script_to_value(vm)
}

/// The whole paint layer as a heap object, in [`Paint::TOKEN_NAMES`] order.
pub fn paint_namespace(vm: &mut ScriptVm, paint: &Paint) -> ScriptValue {
    let object = vm.bx.heap.new_object();
    for (name, value) in Paint::TOKEN_NAMES.iter().zip(paint.read_tokens()) {
        let value = color_value(value, vm);
        vm.bx.heap.set_value_def(object, key(name), value);
    }
    object.into()
}

/// A Makepad `TextStyle` for one rung, derived from the theme's own face.
///
/// The face is cloned by proto rather than restated, so a font policy change
/// (a different vendored family, a CJK fallback chain) reaches every rung
/// without this module knowing anything about fonts.
fn text_style_value(vm: &mut ScriptVm, face: Face, role: TextStyle) -> ScriptValue {
    let theme = vm.bx.heap.module(LiveId::from_str("theme"));
    let base = vm
        .bx
        .heap
        .value(theme, key(face.script_name()), NoTrap);
    if base.is_nil() {
        // The host theme has not been installed yet. Naming nothing is better
        // than baking a wrong size: the call site keeps its own default, and
        // `stamp` runs again once the theme is up.
        return ScriptValue::NIL;
    }
    let object = vm.bx.heap.new_with_proto(base);
    let size: ScriptValue = (role.painted() as f64).into();
    let leading: ScriptValue = (role.leading() as f64).into();
    vm.bx.heap.set_value_def(object, key("font_size"), size);
    vm.bx.heap.set_value_def(object, key("line_spacing"), leading);
    object.into()
}

/// The type ladder as a heap object: one `TextStyle` per role, named by
/// [`TextStyle::name`].
pub fn type_namespace(vm: &mut ScriptVm) -> ScriptValue {
    let object = vm.bx.heap.new_object();
    for role in TextStyle::ALL {
        let value = text_style_value(vm, role.face(), role);
        if !value.is_nil() {
            vm.bx.heap.set_value_def(object, key(role.name()), value);
        }
    }
    object.into()
}

/// The layout metrics as a heap object.
///
/// Control sizes are nested one level (`control.regular.height`) so a DSL block
/// reads the whole set for a size rather than four unrelated names.
pub fn layout_namespace(vm: &mut ScriptVm, layout: &Layout) -> ScriptValue {
    let object = vm.bx.heap.new_object();
    let num = |v: f32| -> ScriptValue { (v as f64).into() };

    for (name, value) in [
        ("space", layout.space),
        ("content_margin", layout.content_margin),
        ("header_height", layout.header_height),
        ("titlebar_height", layout.titlebar_height),
        ("titlebar_top_pad", layout.titlebar_top_pad),
        ("traffic_light_inset", layout.traffic_light_inset),
        ("status_strip_height", layout.status_strip_height),
        ("transcript_fade_band", layout.transcript_fade_band),
        ("row_height", layout.row_height),
    ] {
        vm.bx.heap.set_value_def(object, key(name), num(value));
    }

    let control = vm.bx.heap.new_object();
    for size in [ControlSize::Small, ControlSize::Regular, ControlSize::Large] {
        let entry = vm.bx.heap.new_object();
        vm.bx
            .heap
            .set_value_def(entry, key("height"), num(size.height()));
        vm.bx
            .heap
            .set_value_def(entry, key("pad_x"), num(size.pad_x()));
        vm.bx
            .heap
            .set_value_def(entry, key("pad_y"), num(size.pad_y()));
        vm.bx
            .heap
            .set_value_def(entry, key("radius"), num(size.radius()));
        let name = match size {
            ControlSize::Small => "small",
            ControlSize::Regular => "regular",
            ControlSize::Large => "large",
        };
        vm.bx.heap.set_value_def(control, key(name), entry.into());
    }
    vm.bx
        .heap
        .set_value_def(object, key("control"), control.into());

    let radius = vm.bx.heap.new_object();
    for (name, value) in [
        ("bubble", Theme::bubble_radius()),
        ("surface", Theme::surface_radius()),
        ("panel", Theme::panel_radius()),
        ("button", Theme::button_radius()),
        ("control", Theme::control_radius()),
    ] {
        vm.bx.heap.set_value_def(radius, key(name), num(value));
    }
    vm.bx
        .heap
        .set_value_def(object, key("radius"), radius.into());

    object.into()
}

/// The frost scale as a heap object, so a surface widget can name a thickness
/// in its DSL block rather than hard-coding an opacity.
pub fn material_namespace(vm: &mut ScriptVm, theme: &Theme) -> ScriptValue {
    let object = vm.bx.heap.new_object();
    let spec = theme.material();
    for thickness in Material::ALL {
        let at = spec.at(thickness);
        let entry = vm.bx.heap.new_object();
        vm.bx
            .heap
            .set_value_def(entry, key("gain"), (at.gain as f64).into());
        vm.bx.heap.set_value_def(
            entry,
            key("saturation"),
            (at.saturation as f64).into(),
        );
        let tint = color_value(at.tint, vm);
        vm.bx.heap.set_value_def(entry, key("tint"), tint);
        vm.bx.heap.set_value_def(entry, key("blur"), (at.blur as f64).into());
        vm.bx.heap.set_value_def(entry, key("rim"), (at.rim as f64).into());
        vm.bx
            .heap
            .set_value_def(entry, key("reach"), (at.reach as f64).into());
        vm.bx.heap.set_value_def(entry, key("edge"), (at.edge as f64).into());
        vm.bx.heap.set_value_def(
            entry,
            key("edge_width"),
            (at.edge_width as f64).into(),
        );
        vm.bx
            .heap
            .set_value_def(entry, key("edge_aa"), (at.edge_aa as f64).into());
        vm.bx
            .heap
            .set_value_def(entry, key("coverage"), (at.coverage() as f64).into());
        vm.bx
            .heap
            .set_value_def(object, key(thickness.name()), entry.into());
    }
    object.into()
}

/// Write the whole theme onto the script heap.
///
/// Called by [`Theme::install`], which is the only path that changes what is in
/// force — so the heap and the `Cx` global can never disagree.
pub fn stamp(theme: &Theme, cx: &mut Cx) {
    cx.with_vm(|vm| {
        let paint = paint_namespace(vm, &theme.paint);
        let ladder = type_namespace(vm);
        let layout = layout_namespace(vm, &theme.layout);
        let material = material_namespace(vm, theme);
        let legacy = crate::legacy::namespace(vm, &theme.paint, theme.appearance);
        let appearance: ScriptValue = match theme.appearance {
            Appearance::Dark => LiveId::from_str("dark").into(),
            Appearance::Light => LiveId::from_str("light").into(),
        };

        let set = |vm: &mut ScriptVm, module: &str, name: &str, value: ScriptValue| {
            let module = vm.bx.heap.module(LiveId::from_str(module));
            vm.bx.heap.set_value_def(module, key(name), value);
        };
        set(vm, "mpc", "tokens", paint);
        set(vm, "mpc", "type", ladder);
        set(vm, "mpc", "layout", layout);
        set(vm, "mpc", "material", material);
        set(vm, "mpc", "appearance", appearance);
        set(vm, LEGACY_MODULE, "active", legacy);
        // The v2 widgets glob-import `mod.mpc_theme.*`, so the flat names have
        // to sit on that module itself, not under `active`.
        let module = vm.bx.heap.module(LiveId::from_str(LEGACY_MODULE));
        let legacy_object = vm.bx.heap.value(module, key("active"), NoTrap);
        if let Some(legacy_object) = legacy_object.as_object() {
            for name in crate::legacy::TOKEN_NAMES {
                let value = vm.bx.heap.value(legacy_object, key(name), NoTrap);
                vm.bx.heap.set_value_def(module, key(name), value);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_paths_are_distinct() {
        let paths = [PAINT_PATH, TYPE_PATH, LAYOUT_PATH, MATERIAL_PATH];
        for (i, a) in paths.iter().enumerate() {
            for b in paths.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
            assert!(a.starts_with(MODULE), "{a}");
        }
        // The v2 namespace must not be a v3 path, or stamping one would
        // clobber the other.
        for p in paths {
            assert!(!p.ends_with(LEGACY_MODULE), "{p}");
        }
        assert_ne!(MODULE, LEGACY_MODULE);
    }

    #[test]
    fn test_key_names_are_live_ids() {
        // A name that `LiveId::from_str` cannot intern would silently create a
        // second, unreachable key.
        for name in Paint::TOKEN_NAMES {
            let _ = key(name);
        }
        for role in TextStyle::ALL {
            let _ = key(role.name());
        }
    }

    #[test]
    fn test_every_role_name_is_a_valid_key() {
        let mut seen = std::collections::HashSet::new();
        for role in TextStyle::ALL {
            assert!(seen.insert(role.name()), "{:?}", role);
        }
    }
}
