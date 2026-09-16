//! The script-heap contract: what a widget's DSL block can actually name.
//!
//! The theme is Rust data, but a Makepad DSL call site reads it as script
//! objects. That seam is where the v2 theme went wrong — a token added to the
//! Rust struct and missed in the script emission painted the wrong colour with
//! no error anywhere — so it is tested against a real script VM rather than
//! asserted about from Rust.

use makepad_theme::{install, theme::Theme, Appearance};
use makepad_widgets::{
    makepad_platform::Cx,
    makepad_script::{trap::NoTrap, ScriptValue},
    LiveId,
};

/// A VM whose heap carries the Makepad widget theme and the component theme.
///
/// Order matters and is the app's order too: the component namespaces are
/// declared in terms of `mod.prelude.widgets_internal`, so the widget prelude
/// has to exist first.
fn cx() -> Cx {
    let mut cx = Cx::new(Box::new(|_, _| {}));
    cx.with_vm(makepad_widgets::script_mod);
    cx.with_vm(makepad_theme::script_mod);
    cx
}

/// A `Vec4f` stamped on the heap is a pod, not a packed colour, so its
/// components are read out of the pod's word array.
fn vec4_component(cx: &mut Cx, value: ScriptValue, index: usize) -> f32 {
    let pod = value
        .as_pod()
        .unwrap_or_else(|| panic!("value is not a pod: is_color={}", value.is_color()));
    cx.with_vm(|vm| f32::from_bits(vm.bx.heap.pod_data(pod).1[index]))
}

fn lookup(cx: &mut Cx, path: &[&str]) -> ScriptValue {
    cx.with_vm(|vm| {
        let mut object = vm.bx.heap.module(LiveId::from_str(path[0]));
        for name in &path[1..] {
            object = vm
                .bx
                .heap
                .value(object, LiveId::from_str(name).into(), NoTrap)
                .as_object()
                .unwrap_or_else(|| panic!("{} is not an object", path.join(".")));
        }
        object.into()
    })
}

fn field(cx: &mut Cx, path: &[&str], name: &str) -> ScriptValue {
    let object = lookup(cx, path).as_object().expect("object");
    cx.with_vm(|vm| vm.bx.heap.value(object, LiveId::from_str(name).into(), NoTrap))
}

fn number(cx: &mut Cx, path: &[&str], name: &str) -> f64 {
    field(cx, path, name)
        .as_f64()
        .unwrap_or_else(|| panic!("{}.{name} is not a number", path.join(".")))
}

#[test]
fn the_paint_namespace_carries_every_token() {
    let mut cx = cx();
    let tokens = lookup(&mut cx, &["mpc", "tokens"]);
    let object = tokens.as_object().expect("mpc.tokens");
    cx.with_vm(|vm| {
        for name in makepad_theme::Paint::TOKEN_NAMES {
            let value = vm.bx.heap.value(object, LiveId::from_str(name).into(), NoTrap);
            // A `Vec4f` crossing into the heap becomes a *pod*, which is a
            // distinct value variant from an object — so `is_object()` alone
            // would reject every token.
            assert!(
                value.is_pod() || value.is_object() || value.is_color() || value.is_number(),
                "mpc.tokens.{name} is missing or not a value (nil={})",
                value.is_nil()
            );
        }
    });
}

#[test]
fn a_token_reads_back_as_the_palette_it_was_stamped_from() {
    let mut cx = cx();
    let accent = field(&mut cx, &["mpc", "tokens"], "accent");
    let expected = makepad_theme::palette::dark().accent;
    for (index, want) in [expected.x, expected.y, expected.z, expected.w].into_iter().enumerate() {
        let got = vec4_component(&mut cx, accent, index);
        assert!(
            (got - want).abs() < 1e-4,
            "mpc.tokens.accent component {index}: {got} want {want}"
        );
    }
}

#[test]
fn two_different_tokens_do_not_stamp_the_same_field() {
    // The v2 failure mode in one assertion: a walk that reads the wrong slot
    // leaves two names holding one colour and nothing says so.
    let mut cx = cx();
    let bg = field(&mut cx, &["mpc", "tokens"], "bg");
    let overlay = field(&mut cx, &["mpc", "tokens"], "surface_overlay");
    let text = field(&mut cx, &["mpc", "tokens"], "text");
    assert_ne!(bg, overlay);
    assert_ne!(bg, text);
    assert_ne!(overlay, text);
}

#[test]
fn the_type_ladder_carries_all_eleven_roles() {
    let mut cx = cx();
    let ladder = lookup(&mut cx, &["mpc", "type"]);
    let object = ladder.as_object().expect("mpc.type");
    cx.with_vm(|vm| {
        for role in makepad_theme::typography::TextStyle::ALL {
            let value = vm
                .bx
                .heap
                .value(object, LiveId::from_str(role.name()).into(), NoTrap);
            assert!(!value.is_nil(), "mpc.type.{} is missing", role.name());
            let style = value.as_object().unwrap_or_else(|| {
                panic!("mpc.type.{} is not an object", role.name())
            });
            let size = vm
                .bx
                .heap
                .value(style, LiveId::from_str("font_size").into(), NoTrap);
            assert!(
                (size.as_f64().unwrap_or(-1.0) - role.size() as f64).abs() < 1e-4,
                "mpc.type.{} font_size is {:?}, want {}",
                role.name(),
                size.as_f64(),
                role.size()
            );
        }
    });
}

#[test]
fn a_text_style_keeps_the_host_face_and_gains_the_ladders_numbers() {
    // The whole reason the ladder clones by proto: a font policy change has to
    // reach every rung without this crate knowing about fonts.
    let mut cx = cx();
    let body = field(&mut cx, &["mpc", "type"], "body");
    let object = body.as_object().expect("body");
    let has_family = cx.with_vm(|vm| {
        let family = vm
            .bx
            .heap
            .value(object, LiveId::from_str("font_family").into(), NoTrap);
        !family.is_nil()
    });
    assert!(has_family, "the ladder dropped the host font family");
}

#[test]
fn the_layout_namespace_carries_the_metrics_and_the_control_ladder() {
    let mut cx = cx();
    assert_eq!(
        number(&mut cx, &["mpc", "layout"], "space"),
        makepad_theme::Theme::SPACE as f64
    );
    assert_eq!(
        number(&mut cx, &["mpc", "layout"], "content_margin"),
        makepad_theme::Theme::CONTENT_MARGIN as f64
    );
    for (name, size) in [
        ("small", makepad_theme::ControlSize::Small),
        ("regular", makepad_theme::ControlSize::Regular),
        ("large", makepad_theme::ControlSize::Large),
    ] {
        let object = lookup(&mut cx, &["mpc", "layout", "control", name]);
        let got = cx.with_vm(|vm| {
            vm.bx
                .heap
                .value(
                    object.as_object().unwrap(),
                    LiveId::from_str("height").into(),
                    NoTrap,
                )
                .as_f64()
        });
        assert_eq!(got, Some(size.height() as f64), "control.{name}.height");
    }
}

#[test]
fn the_radii_are_on_the_heap_for_a_dsl_block_to_name() {
    let mut cx = cx();
    for (name, value) in [
        ("bubble", Theme::bubble_radius()),
        ("surface", Theme::surface_radius()),
        ("panel", Theme::panel_radius()),
        ("button", Theme::button_radius()),
        ("control", Theme::control_radius()),
    ] {
        assert_eq!(
            number(&mut cx, &["mpc", "layout", "radius"], name),
            value as f64,
            "radius.{name}"
        );
    }
}

#[test]
fn the_frost_scale_is_on_the_heap() {
    let mut cx = cx();
    for thickness in makepad_theme::Material::ALL {
        let object = lookup(&mut cx, &["mpc", "material", thickness.name()]);
        assert!(!object.is_nil(), "{}", thickness.name());
        let coverage = cx.with_vm(|vm| {
            vm.bx
                .heap
                .value(
                    object.as_object().unwrap(),
                    LiveId::from_str("coverage").into(),
                    NoTrap,
                )
                .as_f64()
        });
        assert_eq!(
            coverage,
            Some(thickness.opacity() as f64),
            "{}",
            thickness.name()
        );
    }
}

#[test]
fn install_restamps_the_namespaces_for_the_new_appearance() {
    let mut cx = cx();
    let background_before = field(&mut cx, &["mpc", "tokens"], "bg");
    Theme::install(Theme::light(), &mut cx);
    let background_after = field(&mut cx, &["mpc", "tokens"], "bg");
    assert_ne!(
        background_before, background_after,
        "installing the light theme left the dark page colour on the heap"
    );
    // ...and the ladder survives the restamp rather than being cleared.
    let body = field(&mut cx, &["mpc", "type"], "body");
    assert!(!body.is_nil(), "the restamp dropped the type ladder");
}

#[test]
fn installing_a_brand_restamps_the_tokens() {
    let mut cx = cx();
    let before = field(&mut cx, &["mpc", "tokens"], "surface");
    Theme::install(
        Theme::branded(
            &makepad_theme::Brand {
                tint: makepad_theme::brand::Tint::new(257.417, 0.046),
                ..Default::default()
            },
            Appearance::Dark,
        ),
        &mut cx,
    );
    let after = field(&mut cx, &["mpc", "tokens"], "surface");
    assert_ne!(before, after, "a brand change did not reach the script heap");
}

#[test]
fn the_v2_namespace_still_resolves_for_the_widgets_that_use_it() {
    // Temporary, like `makepad_theme::legacy` itself: the v2 widget set glob
    // imports `mod.mpc_theme.*`, and `MpThemeState` sweeps `dark`/`light`.
    let mut cx = cx();
    let legacy = lookup(&mut cx, &["mpc_theme"]);
    assert!(!legacy.is_nil());
    for name in makepad_theme::TOKEN_NAMES {
        let value = field(&mut cx, &["mpc_theme"], name);
        assert!(!value.is_nil(), "mod.mpc_theme.{name} disappeared");
    }
    assert!(!lookup(&mut cx, &["mpc_theme", "dark"]).is_nil());
    assert!(!lookup(&mut cx, &["mpc_theme", "light"]).is_nil());
}

#[test]
fn the_legacy_namespace_follows_an_appearance_switch() {
    let mut cx = cx();
    let before = field(&mut cx, &["mpc_theme"], "BG");
    Theme::install(Theme::light(), &mut cx);
    let after = field(&mut cx, &["mpc_theme"], "BG");
    assert_ne!(before, after);
}

#[test]
fn the_namespace_constants_name_real_paths() {
    // A constant that no longer matches what `stamp` writes is a silent
    // no-op for every consumer that imports it.
    let mut cx = cx();
    for path in [
        install::PAINT_PATH,
        install::TYPE_PATH,
        install::LAYOUT_PATH,
        install::MATERIAL_PATH,
    ] {
        let parts: Vec<&str> = path.split('.').collect();
        assert!(!lookup(&mut cx, &parts).is_nil(), "{path}");
    }
}
