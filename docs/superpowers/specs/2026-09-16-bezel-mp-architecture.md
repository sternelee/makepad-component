# makepad-component v3: a full parity port of gpui-bezel

> Status: in progress · Started 2026-09-16
> Reference: `/Users/sternelee/www/github/gpui-bezel` (crates `theme` → `motion` → `ui` → layers)
> Baseline: makepad-component `crates/{theme,motion,ui}` at commit `d928189`

## Why a port and not a repair

The v2 restyle (2026-08-23, `docs/superpowers/specs/2026-08-23-bezel-restyle-design.md`)
moved the tokens and repainted the widgets, but kept the old architecture. Three
consequences are measurable in the tree today:

| Symptom | Evidence |
|---|---|
| No type system | Every call site writes `theme.font_regular{font_size: 13.0}` by hand — 40+ distinct sizes across the crate, no ladder |
| No layout system | Radii are four `const`s; sibling gaps, control heights and insets are literals at each call site |
| Tokens are written 4× | `mod.mpc_theme.dark`, `.light`, the active copy, and `Tokens::get_by_name` each enumerate the same 40 names |
| Motion is a stub | `crates/motion` is 22 lines / 4 constants; durations are inline in each widget's animator block |
| Widgets are untested | 37 tests in the whole crate, all but 3 in `a2ui/*` |

Bezel's answer to each is structural: a measured type ladder
(`NSFont.preferredFont(forTextStyle:)`), a layout metrics layer
(`NSStackView().spacing`), one token struct resolved once, a motion catalog with
unit-tested phase math, and 44 per-widget behaviour test files.

## Laws (adopted from bezel `CONTRIBUTING.md`, adapted to Makepad)

1. **Style flows through the environment.** Components read `Theme::of(cx)` at
   paint time and write resolved values into their shader instance fields. No
   color, font or size parameters on a widget's API. The caller overrides by
   setting the DSL field, never by a Rust argument.
2. **SwiftUI vocabulary.** Widgets are named for their SwiftUI analog —
   `MpToggle`, `MpDivider`, `MpGroupBox`, `MpMaterial`. Stateless paint is a
   method on `Theme`. A closed enum selects between shipped looks; free-form
   radius/color/padding are never parameters.
3. **Motion is named.** Every animation comes from the `MotionSpec` catalog in
   `motion`; pure phase math lives in `motion::phase` and is unit-tested. No
   inline durations or curves.
4. **Numbers drive layout, colors are paint.** No layout depends on which color
   is painted. A number reaches `Theme` when the platform names it; a widget's
   own metric stays with the widget.
5. **Measured, and dated.** Every number on `Theme` records where it was read
   and when. Where the platform names one value we ship one and no more.

## Layering

```
crates/theme     tokens + appearance + typography + layout + material   (depends on makepad-widgets alone)
crates/motion    the animation vocabulary + phase math                  (depends on nothing)
crates/ui        components (+ a2ui)                                    (depends on theme + motion)
crates/gallery   the documentation: a rail of every component, live     (new, replaces component-zoo)
```

## Makepad capability map (recon 2026-09-16)

| bezel concept | Makepad facility | Verdict |
|---|---|---|
| `Theme::of(cx)` gpui `Global` | `Cx::get_global_ref::<T>()` / `set_global` | direct |
| `Hsla` | `Vec4f` RGBA, gamma sRGB | direct (math already in `theme::color`) |
| `StyleRefinement` merge | DSL `+:` partial override | direct |
| `RenderOnce` + keyed state | `Widget` derive + `#[rust]` fields | direct |
| `Animation`/`MotionSpec` | `Animator` + `Play` (`Forward{delay,duration}`, `Snap`, `Loop{…}`) + `Ease` (incl. `Bezier{cp0..cp3}`) | direct |
| `px()` layout | `Walk` (`Fill`/`Fit`/`Fixed`), `Inset`, `Align` | direct |
| `Sdf2d` rounded rect | `Sdf2d.box/rect/stroke/fill/glow`, `viewport(self.pos * self.rect_size)` | direct |
| `box_shadow` | `DrawQuad` `shadow_color/shadow_radius/shadow_offset` | direct |
| `NSGlassEffectView` material | `widgets::backdrop` (`GaussStack` + `DrawGaussScene`, real mip-chain blur) | portable, simplified |
| tree-sitter highlighting | `makepad_syntax` / `makepad_code_editor` in the makepad tree | reuse, do not rewrite |
| gpui entity tree | `WidgetRef` + `Scope` | direct |
| `#[gpui::test]` app harness | plain `#[test]` on pure modules; `Widget` unit tests via `CxHeadless`-less struct tests | partial — pure logic is tested, paint is verified by the gallery |

## Theme, resolved

One struct, resolved once per appearance change, read at paint time:

```
Theme {
    appearance: Appearance,
    // paint — 39 color tokens (surfaces, ink ladder, solid/accent plates, status, code, diff)
    // type   — the 11-role ladder, each with size + leading + weight
    // layout — Sizing (5 control sizes), radii (4, concentric), gaps, insets, chrome heights
    // material — Glass variants + frost thickness scale
    // syntax — 24 highlight roles
    brand: Brand,
}
```

`Theme::of(cx)` returns `&Theme` from the `Cx` global; `set_brand` / `set_appearance`
replace the global and call `cx.redraw_all()`. The script heap copy
(`mod.mpc_theme.*`) is **generated from the same struct** by one `#(expr)` per
namespace, ending the 4× duplication.

## Motion, resolved

`MotionSpec { delay, duration, ease }` → applied as an `Animator` `Play`.

```
pub const HOVER_FADE: MotionSpec = MotionSpec::new(0.0, 0.15, Ease::Tailwind);  // bezel's measured hover
pub const PRESS:      MotionSpec = MotionSpec::new(0.0, 0.09, Ease::Tailwind);
pub const EXPAND:     MotionSpec = ...
pub const COLLAPSE:   MotionSpec = ...
pub const PANEL:      MotionSpec = ...
pub const SPIN:       MotionSpec = ...   // loop
```

`motion::phase` holds the pure math (stagger, spring-ish decay, progress
clamping) with unit tests, mirroring `bezel/crates/motion/src/phase.rs`.

## Delivery phases

| Phase | Content | Done when | Status |
|---|---|---|---|
| **0** | this spec | — | ✅ |
| **1** | `theme` v3: color, brand, typography, layout, material, paint, `Theme`, install | `cargo test -p makepad-theme` green; contrast tests for light+dark; script namespace verified against a real VM | ✅ **117 tests** (104 unit + 13 script-VM integration + 1 doc) |
| **2** | `motion` v2: catalog + phase math + the script/`Play` adapter | `cargo test -p makepad-motion` green | ✅ **53 tests** |
| **3** | Core widgets rebuilt: button, input, checkbox, radio, switch, toggle, slider, select, popover, menu, tooltip, card, group_box, divider, table, tree, scroll_area | each has a gallery page and ≥1 behaviour test | ⬜ |
| **4** | Extended widgets: combobox, date, pagination, stats, loaders, menubar, titlebar, control_bar, search, palette, floating, hover_card | idem | ⬜ |
| **5** | Layers: markdown, blocks, editor, terminal, canvas | feature parity with bezel's layers, reusing makepad's syntax/code_editor | ⬜ |
| **6** | `gallery` app: rail of per-component pages, one file per page, test asserting each row names an existing file | replaces `component-zoo` | ⬜ |

Old widgets are deleted as their replacement lands, never kept as aliases.

## What Phase 1 and 2 actually verify

Measured on 2026-09-16, after the two crates landed.

| Check | Command | Result |
|---|---|---|
| Theme unit + integration + doc tests | `cargo test -p makepad-theme` | 104 + 13 + 1 passed |
| Motion tests | `cargo test -p makepad-motion` | 53 passed |
| No regression in the v2 widget set | `cargo test -p makepad-component` | 37 passed (unchanged) |
| Every crate still compiles | `cargo check -p {makepad-component,makepad-plot,component-zoo,a2ui-demo,makepad-clipboard,raycast-launcher,dbpro}` | all pass |
| `component-zoo` runs clean | run 18s, `grep -c '\[E\]'` | **0** |
| `a2ui-demo` runs clean | run 18s, `grep -c '\[E\]'` | **0** |

The script-heap seam is tested against a real `Cx` + script VM
(`crates/theme/tests/script.rs`), not asserted about from Rust: it enumerates
every token, every ladder role, every control size and every frost thickness on
the heap, checks that two different names do not hold one colour, and checks
that `Theme::install` restamps after an appearance or brand change.

## The v2 compatibility bridge

`crates/theme/src/legacy.rs` re-emits the v2 `mod.mpc_theme` namespace (flat
upper-case tokens plus nested `dark`/`light`) from the v3 palette, so the
~80 v2 widgets keep running — and inherit the corrected contrast — while their
replacements land. It exists because the v3 names are lower-case under
`mod.mpc.tokens` and the v2 set cannot be migrated in one pass without either a
broken window or an 80-file rename that fixes nothing.

Two things it cost, both worth recording:

- The v2 `TOKEN_NAMES` list did **not** contain `TRANSPARENT`, though the v2
  script namespace emitted it. Six v2 widgets paint with it and the shaders that
  `mix()` it fail to compile without it — dropping it produced 394 runtime
  shader errors across `component-zoo`. It is re-emitted.
- `MpThemeState` and `Theme::install` both write the namespaces. During the
  port `Theme::install` wins; `MpThemeState` is deleted in Phase 3 rather than
  reconciled.

**Delete `legacy.rs`, the `LEGACY_MODULE` constant, the `mod.mpc_theme` line in
`lib.rs`'s `script_mod!` and the legacy half of `install::stamp` with the last
v2 widget.**
