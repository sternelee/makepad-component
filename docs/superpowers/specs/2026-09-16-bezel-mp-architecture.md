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
| **3** | Core widgets rebuilt: button, input, checkbox, radio, switch, toggle, slider, select, popover, menu, tooltip, card, group_box, divider, table, tree, scroll_area | each has a gallery page and ≥1 behaviour test | 🚧 **started**: `mp::action`, `mp::surface`, `mp::button`, `crates/gallery` |
| **4** | Extended widgets: combobox, date, pagination, stats, loaders, menubar, titlebar, control_bar, search, palette, floating, hover_card | idem | ⬜ |
| **5** | Layers: markdown, blocks, editor, terminal, canvas | feature parity with bezel's layers, reusing makepad's syntax/code_editor | ⬜ |
| **6** | `gallery` app: rail of per-component pages, one file per page, test asserting each row names an existing file | replaces `component-zoo` | 🚧 **started** — `crates/gallery` exists with 5 pages and the file-exists test; `component-zoo` goes when the pages cover the v2 set |

Old widgets are deleted as their replacement lands, never kept as aliases.

## What Phase 3 has landed so far

`crates/ui/src/mp/` — the v3 component tree, alongside the v2 widget set rather
than replacing it yet (the v2 set is what `component-zoo` and `a2ui` still
build on).

| File | What it is |
|---|---|
| `mp/action.rs` | Reading a widget's own actions out of a batch. The v2 set used `find_widget_action(uid).cast()`, which returns the **first** action for a uid and then hides the mismatch by yielding `T::default()` — see `docs/WIDGETS_PROGRESS_CN.md` §3.2. Three widgets were patched by hand; the rest kept the bug. The walk lives here, once, over the framework's own `filter_widget_actions`. |
| `mp/surface.rs` | `MpSurface` and its planes: page, panel, card, raised, dialog, overlay, sunken, plus the Gaussian-backed glass. |
| `mp/button.rs` | `MpButton`, rebuilt. |
| `mp/control.rs` | The shared contract of every interactive control: the pointer/keyboard signals, the four animator tracks, and the plate helpers. Replaces the thirty-line hit block the five v2 controls each carried a copy of. |
| `mp/checkbox.rs` | `MpCheckbox` — two independent states. |
| `mp/switch.rs` | `MpSwitch` — the same value as a position. |
| `mp/radio.rs` | `MpRadio` — one choice, and the group is the caller's. |
| `mp/layout.rs` | `Row`, `Column`, `Divider`, `DividerVertical`, `Spacer`. The system gap, carried by a prototype so a call site that wants 8pt writes no number. |
| `mp/loaders.rs` | `MpSpinner`, `MpPulse`, `MpProgress`. The payoff for `motion::phase` — its constants are injected as instances from Rust rather than restated in the DSL. |
| `mp/slider.rs` | `MpSlider`, and the four free functions that turn a pointer into a value. |
| `mp/tooltip.rs` | `MpTooltip` — the overlay mechanism, verified; the gallery's hover wiring for it is proven by signal, not by capture. See below. |
| `mp/popover.rs` | `MpPopover` — verified: the panel opens at its trigger's bottom edge and draws over the content below. |
| `mp/icon.rs` | `MpIcon` — a glyph from Makepad's bundled FontAwesome, sized from the control ladder. The smallest component and the one most others want. |
| `mp/status.rs` | `MpBadge`, `MpTag` — six tones, two assembled looks from one shader. A badge reports, a tag classifies. |
| `mp/table.rs` | `MpTable` — columns, rows, row hover and selection. One widget rather than one per cell, with the layout arithmetic in a single function the painter, the hover and the click all read. |
| `mp/tree.rs` | `MpTree` — a flat list where each item carries its depth, with the collapsed set owned by the widget. Ten tests, all on the visibility model. |
| `mp/avatar.rs` | `MpAvatar`, `MpAvatarGroup` — initials with a plate derived from the name, presence reusing the badge tones, and an overlapped group. |

### The ink rule, and the bug that widening a test found

`mp::control::plates::ink_on` picks a plate's label ink, and its first version
chose between the palette's two extremes by a *lightness threshold*. Broadening
the badge test from the three tones it checked to all seven **failed**: light
`busy`, a saturated pink, measures **4.35:1** — under AA — because it sits near
the crossover lightness where black and white give equal contrast and the
palette's own extremes are not pure. The rule now falls back to whichever pure
end reads better, which always clears the floor, and a property test covers ten
plates × both appearances. The same rule the theme's brand plate already used,
for the same reason: a plate that cannot carry a label is not a plate.

Two more traps, both the same one: **`#[derive(Script)]`'s field parser rejects a
fully-qualified path in a field type.** `crate::mp::status::StatusTone` failed
where `StatusTone` behind a `use` parses, exactly as `HashSet<usize>` failed in
`mp/tree.rs`. AGENTS.md's note says commas in generics are the problem; the real
restriction is any non-trivial path — alias it or import it.

And a registration ordering rule with teeth: **a widget that names another
widget's enum in its DSL must register after it.** `MpAvatar` names
`mod.mp.StatusTone` for its presence dot and was registered before `status`,
which is three runtime errors per use site and nothing at compile time.

### The third instance of one fault: a self-painted widget has nothing to size it

`MpTable` drew **nothing** on its first run, with zero `[E]` lines. Its cells are
painted by `draw_abs` inside its own turtle rather than laid out as children, so
no layout pass can measure it — `height: Fit` measured zero, and the plate and
every cell were drawn into a zero-height box. It sizes itself from
`content_height()` now.

That is the third time: the pulse loader (`width: Fit` measured zero, three
invisible cells), the table, and — the same rule seen from the other side —
`MpPopover`'s panel, where a `Fill` child contributes nothing to a `Fit` parent's
width. The rule worth keeping: **a widget painted entirely by its own shader must
state its size, and a `Fit` container cannot measure a `Fill` child.**

### A conclusion that shapes the rest of the family

**A floating surface cannot be composed into a trigger widget**, so higher-level
components — select, combobox, date picker, menu — are **compositions plus app
wiring** rather than single widgets. The reason is the overlay clip rule: a
popover must be `Fill`/`Fill` inside a `Fill`/`Fill` overlay region, and a trigger
lives in the flow at some arbitrary size. So `MpPopover` is always a *sibling* of
its trigger in the page/window overlay region, and the trigger tells it to open.

That is why the gallery's Popover page is the template for the family rather than
a one-off demo: a select is a trigger face (a `MpButton` with a trailing
`MpIcon` chevron), an `MpPopover` in the overlay region with option rows, and one
line of app wiring. `MpTooltipArea` was an attempt to escape this and it could
not work; the constraint is real, and the compositions are the answer rather than
more widgets.
| `mp/input.rs` | `MpTextInput`, `MpField`, `MpTextInputSearch`. The one component with no Rust: the caret, selection, IME, scroll-into-view and platform keys are Makepad's `TextInput`, so this styles it rather than reimplementing it. |

`MpButton` is the demonstration — the v2 button against this one:

| | v2 | v3 |
|---|---|---|
| Palette instance fields in the shader | 16 (`c_solid` … `border`), present only so Rust could read tokens back out | **0** — `Theme::of(cx)` |
| Looks | 9 | **4** — bezel's closed set |
| Durations | literal `0.15` / `0.09` in the animator block | `mod.motion.hover_fade` / `press` |
| Size | a private `MpSize` ladder with its own font sizes | `ControlSize`, the same ladder the theme exports |
| Action read | silently false behind any other action for the same uid | `action::is` |
| Focus | cached in a field, so the ring could lag a frame | read from `cx.has_key_focus` |

`crates/gallery` — the new documentation app. 6 pages (Palette, Type, Metrics,
Motion, Button, Controls), a rail painted from `pages::PAGES`, and a test that
every rail row names a source file that exists *and* is declared in the module
tree.

`GALLERY_PAGE=<index or title>` pins the opening page. That is not a debug
leftover: Makepad does not expose its widgets to the accessibility tree, so a
capture script has no way to click a rail row, and an app that can only be
driven by a pointer cannot be verified from a script.

### The control family

Five v2 controls averaged 450 lines each and most of that was the same
thirty-line pointer/keyboard block written five times, already drifted:

| v2 defect | Consequence |
|---|---|
| radio never claimed key focus on a pointer press | click a radio, press Space, nothing happens |
| toggle set its hand cursor only on hover-*in* | the cursor stays a hand after the pointer leaves |
| checkbox and switch disagreed about the activating keys | Tab then Space worked on one and not the other |
| each control cached `focused: bool` | the focus ring appeared a frame after the click |

All of it now comes from `mp::control`, and a control declares only what makes
it itself. In the gallery's Controls page the whole family is 120 lines of
behaviour each against the v2 set's 450.

Three bugs the **screenshot** caught that no test would have:

- A control built `checked: true` painted **unchecked**. The field was true from
  construction but the animator was still in its default `off` state, and the
  paint reads the animator. The symptom is the worst kind: the first click
  appears to do nothing, because the value was already true and `set_checked`
  correctly returns early. Fixed by `control::init_checked` from `on_after_new`,
  using `cut` so a control that starts checked starts checked rather than
  animating into it before the first frame.
- The switch's knob **colour** came from the Rust bool while its **position**
  came from the animator, so the two disagreed at construction — a near-black
  knob on a near-black track, invisible. The shader now mixes `knob_off` toward
  `knob_on` by the same `checked` number that places it, so they cannot drift.
- `use mod.motion.*` does not make `motion.hover_fade` resolvable. A glob import
  brings the *members* into scope, so the path has to be written out in full
  (`mod.motion.hover_fade.duration`). Same trap as `layout.space` vs
  `space` in the surface module, and it failed the same way: "variable motion
  not found in scope", 112 errors deep.

One more Makepad trap, recorded because it will bite again: **a module made from
Rust with `ScriptHeap::new_module` is not reachable as `mod.<name>` from a
script.** `mod.motion` had to be published through a `script_mod!` block
(`mod.motion = #(...)`), the same form `makepad_theme` uses for `mod.mpc`.

### The loaders, and three more Makepad traps

`mp/loaders.rs` is where `motion::phase` finally pays for itself: the period, the
resting opacity and the per-cell stagger are the crate's constants, injected into
the shaders as instances from Rust. Only two lines of arithmetic are restated —
the stagger offset and the pulse wave — because a Makepad shader has no loops and
no way to call into Rust, so a shader that draws a breathing dot has to say what
breathing is. Numbers drift; `fract(phase - i * stagger)` does not.

The clock is a looping animator, not a timer. `Play::Loop` holds `phase` at
`(elapsed / duration) % 1` and reports `must_redraw` while it runs, so a loader
needs no lease, no tick list and no `Instant` — and a window with no loader
mounted schedules nothing. The v2 loaders each carried their own frame
accounting.

Four traps found, three of them only by rendering and probing pixels:

- **`mod.mp = {}` is an assignment, not a declaration.** Two modules each opening
  with it reset the namespace and silently destroy every prototype registered
  before them. Four modules doing so produced 107 runtime errors, all of the form
  "property SurfaceSunken not found in prototype chain" — pointing at the *users*
  of the erased prototypes rather than at the line that erased them. It is
  created once, in `surface.rs`, and nowhere else.
- **`fill_keep` retains the shape.** It composites the colour and leaves `shape`
  in place, so the next primitive *unions* with what was just drawn. The progress
  bar drew its track with `fill_keep` and its fill after, which unioned the two
  and made every value read 100%; the pulse drew three cells with `fill_keep`,
  which merged them into one blob. `fill` consumes the shape, `fill_keep` does
  not — use `fill_keep` when you are about to `stroke` the same shape, and `fill`
  when the next shape is independent. A pixel probe of the 62% bar (uniform
  `#969696` along its whole length) is what found it; no test would have.
- **A widget whose only drawing is a `draw_bg` shader has nothing to size it.**
  `width: Fit` measured zero, so the pulse drew three invisible cells in an empty
  row. The widget now derives its walk from `cell` at paint.
- **`use mod.motion.*` does not make `motion.x` resolvable**, and
  `script_eval!` does not retain a `mod.*` assignment the way a `script_mod!`
  block does.

The gallery's rail is also hand-maintained against `PAGES` (`ids!` needs
literals), which drifted: `GALLERY_PAGE=Loaders` opened the Layout page because
the two lists disagreed about which slot held which page. There is now a
`SLOT_PAGES` table in Rust naming what each slot holds, and a test that asserts it
against `PAGES` — the only place the two can be compared. It earned itself
immediately: adding the Slider page failed that test before the build finished,
with `slot 7 holds mod.gallery.pages.controls but PAGES[7] is "Slider"`.

### The slider, and the disabled hole

`mp/slider.rs` moves the pixel-to-value arithmetic out of the widget into four
free functions — `clamp`, `snap`, `fraction`, `value_at` — because that is the
part worth testing and the v2 version could not be tested at all: it was a method
on a widget that needed a live `Area` to run.

Two rules the tests settled, and the second one changed the implementation:

- **The step grid is anchored at `min`, not at zero.** 5..10 by 2 offers 5, 7, 9.
  A zero-anchored grid offers 6, 8, 10, which puts the slider's own minimum out
  of reach.
- **`min` and `max` are members of the grid.** 0..1 by 0.3 offers 0, 0.3, 0.6,
  0.9 *and* 1. Without it, dragging to the far end of the track stops at 0.9 and
  the maximum is unreachable by any gesture — a bug the user finds and the author
  does not. The first implementation clamped instead, which is what the
  reachability test caught; a value *at* an end is now that end, before the grid
  is consulted, because a step larger than the whole range has exactly one grid
  member and both ends are equidistant from it.

Then the screenshot found the same class of bug the controls had, in two more
places:

- **`disabled` was tracked and never read.** The slider's shader had a `disabled`
  instance from `mod.mp.ControlAnimator` and the pixel function never mentioned
  it, so a disabled slider was pixel-identical to an enabled one. It now cuts the
  whole control's coverage, so an unavailable slider still reads as *this*
  slider.
- **`set_disabled` animates, so it cannot seat an initial state.** A control
  built `disabled: true` faded from 0 and, with nothing driving frames, stayed
  there: it rendered enabled. `control::init_disabled` does it with `cut`, the
  same rule as `init_checked`. Applied to the button, slider, checkbox, switch
  and radio — the button had the same hole and the controls page had not shown
  it, because the earlier screenshot only checked the ones whose *value* was set
  at construction rather than their `disabled` flag.

The gallery's slider page also seeds its readouts from the values the pages were
built with, so the value path is visible in a screenshot rather than only after a
drag — and a slider whose value never reaches its readout is a visible defect
rather than a latent one.

### The text field, and when not to write Rust

Every other component here owns its shader and resolves the theme in Rust at
paint. A text field is the exception and it is worth stating why: the hard parts
of one are the caret, the selection, the IME composition, the scroll-into-view
and the platform key handling, and Makepad already ships all of them. Writing a
second one would be the largest file in the crate and the least interesting.

So `mp/input.rs` is a table of token assignments — and the reason it is worth
having is that the assignments are not obvious. Makepad's field has fourteen
colour slots across four layers (`draw_bg`, `draw_text`, `draw_selection`,
`draw_cursor`), each with `hover`/`focus`/`down`/`empty`/`disabled` variants, and
a wrapper that fills in only the base `color` leaves the rest carrying the stock
theme's colours, which show up as blue edges and a green caret on a neutral
palette.

Two more instances of a trap this codebase has now hit three times: **a glob
import brings a module's members into scope, not its nested modules.** `use
mod.mpc.layout.*` gives `space` and `row_height` but not `control`, so
`height: control.regular.height` is the form that resolves and `regular.height`
is not. The same mistake in `surface.rs` (`layout.space` vs `space`) and in
`control.rs` (`motion.hover_fade` vs `hover_fade`) produced 112 and 215 runtime
errors respectively; this one produced 45. The rule is worth writing down: in a
`script_mod!` block, either write the whole path from `mod`, or import the exact
module whose members you name — never a parent.

### Two findings from building it

- **A container cannot read a global in Makepad.** `View::draw_bg` is a fixed
  `DrawQuad` in Rust, so a DSL block naming a token has the value *copied* at
  script-apply time. `Theme::install` therefore calls
  `request_script_reapply()` as well as `redraw_all()`. This is not a
  workaround — it is how Makepad's own `mod.theme` switch works — and it is why
  the codebase has two update paths: a leaf widget that owns its shader resolves
  the theme in Rust every paint, a container has its values re-applied.
- **`instance(..)` in a custom `script_shader` block is wrong.** The storage
  class comes from the Rust struct's `#[live]` fields; `instance()` there asks
  for an object where a number is expected, producing 12 runtime errors of the
  form `type mismatch for property hover: expected f32, got object`.
  `instance()` is only for *overriding* an inherited field's storage class.
- **`mod.mp` has to be created** before `mod.mp.Surface = …` — same rule as
  `mod.mpc = {}` in the theme. Assigning into a missing module produced 215
  runtime errors that all pointed at innocuous-looking lines.

## The overlay: fixed, and what is left of it

The tooltip drew nothing, and two separate faults had to be found. Both were
invisible to `cargo check` and to any test; both were found by taking a
screenshot.

1. **A zero-sized overlay has nothing to composite.** The widget was declared
   `width: 0, height: 0`, on the theory that a tooltip should not take space in
   the layout. It never appeared. It is `Fill`/`Fill` in an `Overlay` flow, which
   is how Makepad's own `Tooltip` is declared — and the consequence is a *usage*
   rule, not a bug: the tooltip must be a child of an `Overlay`-flow parent,
   **beside** the content it covers rather than inside the content's column,
   because inside a column a Fill-height overlay competes with the content for
   height.
2. **The plate must be `Fit`-sized, so it cannot live on the widget.** With the
   widget `Fill`/`Fill`, putting the plate on its own `draw_bg` painted a
   full-window rectangle — a near-white sheet over the entire application. The
   fix is Makepad's: `draw_bg` carries the turtle and paints **nothing**
   (`pixel: fn() { return vec4(0.0, 0.0, 0.0, 0.0) }`), and the plate is a
   `Fit`-sized `content` child that shrink-wraps the label.

A third, smaller one: an `Area` is empty until its widget has been laid out, so
anchoring during startup put the plate at the window corner. `show_for`'s doc now
says so, and the gallery waits for a non-empty rect.

**Verified:** `GALLERY_TOOLTIP=1` shows the plate drawn over a following sibling —
a button the tooltip *precedes* in the tree — anchored from a trigger's `Area`. So
the `DrawList2d` mechanism and the anchoring both work.

**A wrapper cannot host a tooltip, and this is structural.** The obvious fix for
the hover — an `MpTooltipArea` holding its trigger and its own tooltip — was
built, and it cannot work: **an overlay draw list clips to its widget's
rectangle**, and a tooltip's plate is positioned outside a trigger-sized box. So
the constraint is *one tooltip per overlay region*, `Fill`/`Fill` inside a
`Fill`/`Fill` `Overlay` parent, with triggers causing it to be shown rather than
owning one. Established by building the wrapper, watching it not draw, and then
calling `show()` on it directly — which isolated the fault to the draw path
rather than the hover.

What that implies for the hover: the trigger must *signal* the one tooltip, via a
hover action from `mp::control` — and that work is not done.

The trigger signals it now. `mp::control` gained a **shared** `ControlHover`
action (`Entered`/`Left`) emitted by every control straight after `handle` — one
shared type rather than one per widget, because the listener is a single tooltip
and it does not care *which* control was hovered, only that one was and where it
is. The gallery listens on the action batch and anchors to the trigger that
reported.

That change also removed the last copy of the hit contract: `mp/button.rs` still
carried its own hand-written pointer/keyboard block, because it was written
before `control.rs` existed. It uses `control::handle` now, so all five controls
have one implementation, and a button can be a tooltip trigger for the same
reason a checkbox can.

**Recorded: a synthetic pointer warp does not produce a hover event in this app**,
so the hover path cannot be verified from a capture script. The evidence is that
a ghost button under the warped pointer shows none of its hover wash.
`GALLERY_TOOLTIP=1` exists because of it — it exercises the plate and the
anchoring without a pointer. What *is* unit-tested is the signal's plumbing
(`control::hover_tests`: the action is shared, an unrelated action in the batch
is not a hover, and the uid rides along so a listener can anchor). What is not
verified end to end is the link a real pointer takes:
`Hit::FingerHoverIn` → `Signals::hover_in` → `ControlHover::Entered` → the plate.
Every link but the first is covered; the first is the same branch that drives the
hover washes on every control in the crate.

**The gallery's hover detection asked
`event.hits(cx, trigger_area)` from the app for a button it does not own, and
Makepad resolves one hit per event — the widget under the pointer consumes it, so
the second call reports nothing. The first version did something worse
(`area.rect(cx).contains(me.abs)`), comparing a pass-relative rect against a
screen-absolute pointer, which only agrees when the window is at the origin.

Both are the hand-rolled geometry that got v2 into trouble, and the conclusion is
a design one rather than a patch: **hovering belongs in the trigger**, which is
the widget that receives `Hit::FingerHoverIn` in its own `handle_event` — and
`mp::control::Signals::hover_in` already computes exactly that. So the component
that should exist is **`MpTooltipArea`**: a container that owns its trigger
geometry, shows on hover-in and hides on hover-out, keeping the app out of it. It
is also how bezel does it (`hover_card`).

That is the next thing to build, and the rest of the overlay family (popover,
menu, select, combobox) follows the same two rules the tooltip established: a
`Fill`/`Fill` overlay sibling of the content, with a `Fit`-sized plate inside it.

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

## The popover works — and what hid it

Four faults, the last of which concealed the first two.

1. The panels were nested in their trigger rows, so a `Fill`/`Fill` popover had no
   rectangle to draw in. They are siblings of the content at the page root now.
2. `MpPopover` forwarded its view's children, so `panel` drew **inline**, beside
   its trigger, open or shut. It returns `DrawStep::done()` and draws the panel
   only in the overlay, as `MpTooltip` does.
3. **`self.pin_popover(cx)` was never called.** The helper existed and the
   environment variable was read, but nothing invoked it — so with
   `GALLERY_POPOVER=1` the popover was never opened, and every conclusion drawn
   from those runs was about a widget that had not been asked to do anything. A
   log line at the top of `draw_walk` showed 6 calls, never `open=true`, which is
   what exposed it.
4. A layout rule that looked like a fault: **a `Fill` child contributes nothing to
   a `Fit` parent's width**, so a `Fit` panel measures to its widest *intrinsic*
   child. The form panel was 143pt wide with a 260pt field inside it, clipping
   everything past the label. A panel of `Fill` rows must name its width.

**A synthetic pointer produces no hit at all in this app.** Every `event.hits`
call in a run reports `Nothing` — for every control, over 127,564 calls — so
neither a hover nor a click can be delivered from a capture script, and
`GALLERY_TOOLTIP=1` / `GALLERY_POPOVER=1` are the only way to exercise a floating
surface. That also retracts an earlier claim in this file that "a click is
synthesizable": it is not, here.

## A workflow finding worth keeping: `act_ui` takes look-image coordinates

Several wrong turns came from passing **screen points** to `act_ui`. Its
coordinates are in the *look image*'s space — a 900×506 stretched rendering of the
window — not screen points and not the window's own points, and it rejects a value
outside those bounds. Mapping a click needs the look image's size, not the
window's frame. The error message says so ("outside the latest look image bounds"),
which is how it was finally caught.
