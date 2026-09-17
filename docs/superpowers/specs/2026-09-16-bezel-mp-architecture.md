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
| `mp/text.rs` | Measuring and clipping one line, shared by the three row-painting widgets. |
| `mp/list.rs` | `MpList`, `MpMenu` — a glyph, a label and a trailing detail; a menu is the same widget with one flag. |
| `mp/scroll.rs` | `MpScroll`, `MpScrollBoth` — the scroll bar, themed. |
| `mp/search.rs` | `rank` / `matches` / `MpSearch` — the match behind a command palette. |
| `mp/bars.rs` | `MpTitlebar`, `MpControlBar`, `MpMenubar` — the three horizontal chrome strips. |
| `mp/palette.rs` | `remap` / `step` / `original` / `MpPalette` — the query, the list, and the cursor. |
| `mp/segmented.rs` | `slot_at` / `slot_span` / `valid` / `MpSegmented` — the track and the plate. |
| `mp/date.rs` | `is_leap` / `weekday` / `month_grid` / `shift_month` / `MpDate` — the calendar. |
| `mp/keys.rs` | `parse` / `format` / `Keymap` — the chord behind a printed shortcut. |
| `mp/history.rs` | `History<T>` — undo/redo as a value, not a widget. |
| `mp/combobox.rs` | `Combobox` / `MpCombobox` / `MpComboboxPanel` — a field over a list. |
| `mp/focus.rs` | `FocusRegistry` / `register` / `handle_key` — tab traversal. |
| `mp/hover_card.rs` | `HoverIntent` / `Presence` / `Change` / `MpHoverCard` — a card on hover. |
| `mp/floating.rs` | `Floating` / `MpFloating` / `MpFloatingLayer` — a panel the reader drags. |
| `mp/icons.rs` | `glyph::*` / `ALL` / `codepoint` — the declared glyph set. |

And in `crates/theme`: `syntax.rs` (the kind vocabulary, the palette, the span contract) and
`terminal.rs` (the sixteen ANSI colours in both appearances).

### `floating` was not what its name suggested

The audit listed bezel's `floating.rs` as a missing **positioning primitive**, and that was a
guess. Reading it showed something else: a **draggable panel** — a meter, an inspector, a
detached preview — and the guess had been wrong in a way worth recording, because it is the third
time in this port that a filename stood for a concept I had assumed rather than read.

The design insight it carries is the reason it is a component rather than a `View` with a drag
handler: **the drag has to be heard on a layer larger than the thing being dragged.** A panel
that listens on itself stalls the moment the pointer outruns a frame — the pointer lands outside
a box-sized hitbox, the panel stops hearing moves, and the box is stranded behind the cursor.
`mp/scroll.rs`'s thumb already does the same against its track.

So the gesture is split, and the split is the API:

- **The press is heard on the box**, because a press has to be inside the panel or a click
  anywhere on the page would start dragging it.
- **The moves and the release are heard on a layer the app names**:
  `mp_floating(..).drag(cx, event, layer)`. Naming it is what makes the requirement explicit
  rather than a comment about a widget that draws an invisible full-size box and hopes.

### The grab offset, and why nothing is clamped

A drag that put the panel's top-left at the pointer would **snap** the moment it started: grab a
panel by its title bar and it would jump so the pointer held its corner. So the press records
where *inside the panel* it landed, and every move keeps that point under the pointer.

That is also why **nothing is clamped**. A panel dragged half off the window stays there, and
because the offset is preserved it can always be dragged back — clamping would fight the reader
for no benefit. Verified at runtime: `move:-500,-500` gives `pos=(-520,-510)`, and dragging back
works.

### The threshold is the difference between a drag and a shaky click

A press that moves one pixel is a click with a hand tremor in it, and a panel that moves on one
shifts every time its title bar is clicked. `DRAG_THRESHOLD` is 3.0 — the order of gpui's own
`DRAG_THRESHOLD`, which bezel adopts; the number matters far less than there being one.

The measurement is from the **press**, not from the previous move, so slow drift across many
small moves accumulates to a drag. Measuring per-move would let a pointer crawl across the panel
forever without ever starting the gesture — which is exactly how a reader drags something slowly.

Runtime evidence, from the default script:

```
FLOAT press at (40,20) held
FLOAT move to (41,21) moved=false pos=(20,10) dragging=false
FLOAT move to (240,120) moved=true  pos=(220,110) dragging=true
FLOAT move to (40,20)  moved=true  pos=(20,10)  dragging=true
```

The second line is the shaky-click rule; the third shows the offset (`240-20, 120-10`).

### A parsing bug the logging rule caught immediately

The script's first version separated steps with `,` and coordinates with `,`, so `press:40,20`
split into the steps `press:40` and `20`. Every step then failed to parse and **the log came out
empty** — which is exactly what "print every decision" is for, because a gesture that decided
nothing and a script that ran nothing look identical in a screenshot of a box. Steps are
separated by `;` now.

### The timing is the module, because Makepad owns none of it

gpui owns hover-card timing: a tooltip there has a 500ms delay built in and stays alive while
the pointer is inside it, which is why bezel's `hover_card.rs` is 120 lines of **content** with
no timing in it at all. Makepad owns nothing, so this port had to write the machine — and it is
the substance, because all four ways it goes wrong are visible the moment a reader moves a
mouse:

1. **A card that opens instantly flickers.** The pointer crossing the window passes dozens of
   triggers, and each one that opens is a flash of content nobody asked for.
2. **A card that opens on a passing pointer must not.** The same delay from the other side:
   leaving before it elapses has to *cancel*, not merely not-yet-open.
3. **A card the pointer can enter must not close when the pointer enters it.** This is what
   separates a hover card from a tooltip — closing on "left the trigger" closes the thing the
   reader is reaching for.
4. **A card must not flicker across the gap** between trigger and card, which belongs to
   neither.

`Presence` is three states rather than a `hovered: bool` precisely because "inside the card"
and "inside the trigger" have to be told apart: one of them is the arm that stops the close.

**The numbers have sources.** `DEFAULT_DELAY_MS = 500` is gpui's own tooltip delay, so a control
here opens when the same control would open under the reference implementation — the same
convention as every other number that reached the theme because a platform named it.
`DEFAULT_GRACE_MS = 150` is chosen: it only has to cover a hand crossing a small gap.

### Verified at runtime, because a hover is not photographable

A synthetic pointer produces no hover event in this app, so the card can only be *pictured*
pinned, and the **timing cannot be photographed at all**. `GALLERY_HOVER` drives a scripted
presence sequence through the same machine and prints every decision:

| script | result |
|---|---|
| `trigger:200,outside:100,` ×2 | **nothing decided** — no opens, no closes, `dwell=0` |
| `trigger:520,outside:200` | `Opened at 512ms`, `Closed at 160ms` |
| `trigger:600,outside:60,card:300` | `Opened at 512ms`, then **still open** across a 60ms gap and into the card |

The first row is the flicker prevention proven by the **absence** of decisions. The third is the
hoverable property proven by the card surviving a gap crossing — the rule that makes a hover
card a hover card.

One reporting bug was fixed on the way: the log printed the tick's *starting* time, so a
machine with a 500ms delay opened "after 496ms" — a number that appears to contradict the number
it is demonstrating, which is worse than no log at all. It prints the decision time now.

### A v3 module that registered into a v2 one

The v3 controls' tab traversal ran through `crates/ui/src/widgets/focus.rs` — a **v2** module —
while everything else about a v3 control lives in `mp/`. That is the kind of seam that makes a
migration stall: `mp/focus.rs` is now the home, the v2 module is a set of forwarders into it,
and both feed **one registry**. Two registries would mean `tab` walking half the controls on a
page that has both kinds, and the gallery has exactly such pages while the port is in progress.

### The bug the port was written around

The first version put the disabled guard at the **call site**:

```ignore
if !self.disabled { focus::register(cx, uid, area) }
```

Both call sites honoured it, and it is still wrong. What it misses is the control that
**becomes** disabled: it stops calling `register`, so nothing removes it, and its `Area` is
still valid because the control is still drawn — so it stays in the tab order and **tab lands
on a control the reader cannot use**. Taking `disabled` as a parameter means a disabled control
is **removed** when it draws rather than merely not added, so the stale entry cannot exist. The
distinction is the one `mp/list.rs` makes about a value that is no longer in its list: dropping
says "not available", and not-adding says nothing about what was there before.

Two tests hold it: one that a control which becomes disabled leaves the order, and one that a
control reaching disabled by either route produces the same order.

### A surface can claim `tab`

`claim_tab` / `release_tab` stand the traversal down for a surface where `tab` belongs to
something else — a document that nests a list, an editor that inserts indentation. The claim
**persists until released** rather than being per-event, because it describes who owns the key
while that surface has focus. `handle_key` returns `false` while it is claimed, so the key falls
through instead of being consumed. This is bezel's `CLAIMS_TAB` marker, expressed as a flag
because Makepad has no key-context stack to predicate on.

### A limitation recorded rather than repaired

The v2 widgets keep their own `if !self.disabled` guards, so they keep the stale-entry bug. The
forwarder passes `disabled: false`. Fixing it means seven v2 call sites, and those seven widgets
are scheduled for deletion by phase 6 — repairing them would be work spent on code the plan
already removes. Recorded in the forwarder's own doc so it is not mistaken for an oversight.

### A combobox is the only control that holds two things which can disagree

Every other control in the library holds one value. A combobox holds **what is typed** and
**what is chosen**, they agree after a choice, and they disagree the moment the reader types.
The question answered on every keystroke is *"is what is in the field still the thing that
was chosen?"*

**The rule: a choice survives typing only while the text still names it.** Type one more
character and the value is cleared — a stale value is worse than no value, because "nothing
chosen" is a state an app can handle and "something else chosen" is not. The failure it
prevents is invisible in the way this port keeps running into: the field shows `Split Right`,
the app's value still says `New Terminal`, and nothing complains until the next action runs
the wrong command.

Two consequences, one of which was in the first version's doc comment **and wrong**:

- Clearing the text clears the value.
- **A cleared value does not come back by retyping.** The obvious alternative — remembering
  the last choice so an exact match restores it — makes the value a *third thing*: neither
  what the text says nor nothing at all. That is the stale-value fault in a quieter costume.
  The doc claimed the opposite and its own test said so.

### The default highlighted row is *no* row

`palette::remap` defaults an absent cursor to the top row, because a **palette's** Enter must
always run something and its top row is its best answer. A combobox is the opposite: nothing
is highlighted until the reader moves, and committing with nothing highlighted chooses
**nothing** — entering an item the reader never saw is how a combobox runs the wrong command.
The identity arithmetic is shared; this policy is not, and **six tests failed at once** when
the first version used the palette's default directly.

### The registration order is now checked, not documented

`script_mod!` names other widgets' prototypes, and a name that registers *after* its user is
not there yet. The failure is runtime-only and its message points at the **user** of the name
rather than the line in the wrong place:

```text
property MpMenu not found in prototype chain. Did you mean: MpMenubar, MpIcon, ...
```

This port paid for it **three times** — `palette` before `list`, then `combobox` before
`input` and `popover` (twice in one file, because a widget composing two others has two ways
to be too early). A comment prevented neither the second nor the third, so
`crates/ui/tests/registration_order.rs` checks it statically: it reads `src/mp/*.rs`, strips
tests and comments, collects each module's `mod.mp.<Name> =` definitions and every
`mod.mp.<Name>` reference in its DSL, and asserts every referenced prototype is defined by a
module registering **no later** than its user. It also asserts every module that *has* DSL is
registered at all, since an unregistered `script_mod!` is a prototype that silently does not
exist.

Two things about writing that check are worth recording:

1. **It asserts its own scan found something plausible** before asserting anything about
   order — a parser that matched nothing would pass vacuously. This port has already been
   bitten by a `grep -c` that counted a struct definition and reported thirty-four pages when
   there were thirty-one.
2. **It was falsified before it was trusted.** Planting the exact historical bug made it fail
   with `palette (registered #24) uses mod.mp.MpMenu, which list defines at #25`, and restoring
   made it pass. A check that has never failed is not evidence.

It also reported one **false positive** first: `mp/control.rs`, whose ````

```ignore ````
doc block shows how `MpCheckbox` declares itself. That is documentation *about* the order
rather than an edge in it, so the scanner strips comments — a check that cannot tell the
difference reports a violation for every well-documented module.

### `syntax`: the half that does not depend on how spans were produced

bezel's `syntax` runs tree-sitter and returns `(byte range, HighlightKind)` spans. The classifier
is a **material open decision** for this port — tree-sitter plus a grammar per language, a
hand-written tokenizer like the one Makepad's own editor ships, or a bridge to that editor — and
the decision is deliberately not taken here. What is taken is the half bezel also separates out:
*"there is no color and no rendering here — kinds map to colors through `SyntaxPalette::color`."*

So `crates/theme/src/syntax.rs` carries the **kind vocabulary** (13 kinds, a closed set like every
other vocabulary in this theme), the **palette**, and the **span contract** every classifier has to
satisfy — `normalize`, which sorts into document order, clips to the source, and drops overlaps.
That split is not tidiness: it is what lets the classifier be chosen later without touching a
colour, a test, or a call site.

`normalize` exists rather than a convention because painting two overlapping spans paints one
twice, and nesting them paints the inner one then the outer one's background over it — a colour
that is subtly wrong rather than an error. The three faults it fixes are the three a real grammar
produces: spans **out of document order** (a query with several patterns descends in pattern
order), **overlapping** (one pattern's `(identifier)` contains another's `(function_name)`), and
**past the end** (the grammar parsed a stale buffer — the document can be edited between the parse
and the paint, and a span one byte past the end is a panic in a slicing renderer). Its test walks
**every permutation of a four-span set**, so an accidental dependence on input order cannot hide
in one of them.

### Two real bugs the palette's tests found, and one bad test of my own

1. **A contrast measured against a translucent ground is measured against a different colour.**
   `code_wash` is `ink(1.0, 0.08)` in dark — a *translucent* white — and `contrast_ratio` treats its
   argument as opaque, so comparing against the raw wash compares against **pure white** in dark
   and **pure black** in light. The first version of the test reported `2.38:1` for every kind in
   dark mode and looked like a palette fault. The fix is `Paint::code_ground()`, which composites
   the wash over the page — added as an **API** so the mistake cannot recur, with a test asserting
   the two differ by more than 0.3 in luminance. Checked across the theme: every other contrast
   test uses an opaque ground (`bg`, `solid`, `accent_strong`, the surface ladder), so this was the
   new test's flaw rather than a systemic one.

2. **"Recede" is a different direction in each appearance.** A comment sits closer to its ground
   than the code does. On a dark ground that means darker; on a light ground it means **lighter**.
   The first version used one sign for both. Getting the direction right was not enough either:
   `l + 0.12` receded so far that the light comment measured **3.76:1**, under the floor — which
   only became visible *after* the ground was flattened. Both comments now clear **5.09** and
   **5.05**.

3. **My test's proxy for chroma was not a proxy for chroma.** The assertion that `Invalid` is the
   loudest kind used the spread between sRGB channels, and it failed on `Function`: red at chroma
   0.16 and blue at 0.11 do not have channel spreads in that order, because conversion to sRGB is
   not chroma-preserving across hues. The claim was about the palette's **intent**, and the intent
   is the table — so it is asserted against the table now, with one line confirming the conversion
   produced a colour so the test is not a table checked against itself.

### Reading `blocks` changed the plan

`blocks` was next by size — 123 lines, the smallest of the six missing crates. Reading it showed it
is a **seam *on* markdown**: *"a fence already round trips byte for byte, already holds a caret, and
already degrades to its own source where nothing paints it — so a block is a renderer over a ```chart
fence rather than a new `BlockKind`."* Its whole substance is `render(language, code) -> Option<…>`
routed from a fence tag that markdown produces. Porting it before `markdown` exists would build the
dependent before the dependency, so it is deferred — the same "read it before assuming" rule that
`stack`, `stats`, `menu` and `floating` each paid for once.

### `markdown`: the document half, and a fixed point rather than an inverse

`editor` was next on the plan (2906 lines) and **reading it moved the order again**: its own doc says
*"`markdown` holds the document, its markdown wire form, and the painting — all of it testable without
a window. What lives here is the half that needs one."* So `editor` depends on `markdown`, and porting
it first would be the `blocks` mistake a second time. That is the sixth time reading a reference
changed what this port did next, after `stack`, `stats`, `menu`, `floating` and `terminal`.

`makepad-markdown` is the **document half**: the model, `parse`, `serialize`. The shape is Notion's
rather than CommonMark's, and bezel's reasoning is why — *a flat list of blocks with an indent level*,
because editing a flat list means Enter splits, Backspace merges and Tab indents (list operations on
one `Vec`), while on a nested tree "the previous block" is a traversal and every edit is a restructure.

The trade is stated rather than hidden: **arbitrarily nested CommonMark does not survive** — a list
inside a quote inside a list flattens, and there are no tables or reference links. What *is* guaranteed
is the property the crate is built around:

> `parse(serialize(parse(s))) == parse(s)` for every input `s`.

**Weaker than "`serialize` is the inverse of `parse`", and it has to be.** `_italic_` parses to an
italic mark and serializes as `*italic*`, so the *text* changes on the first save; what cannot change
is the **document**, which is the thing an editor holds. The tests make the distinction explicit:
`test_the_fixed_point_is_not_byte_equality_and_underscore_italic_proves_it` asserts both halves.

Two rules carry it, and both are decisions rather than consequences:

- **One canonical spelling per construct.** `***` and `___` serialize as `---`; `_italic_` as
  `*italic*`; `+ item` and `* item` as `- item`. The document is unchanged by each, which is the whole
  advantage of a fixed point over byte equality.
- **A blank line between blocks, except between two list items.** Requiring *equal indents* rather than
  "both are list items" put a blank line between `- outer` and its indented child — which ends the list
  on the next parse, and broke every nested list in the corpus at once.

`Doc::is_well_formed` checks the **indent invariant** the serializer relies on — the first block is at
0 and no block is more than one level deeper than the block before it — and the round-trip test asserts
it on every result rather than trusting the parser to establish it.

### Six real bugs the property test found, and one of mine

Writing the property first and then making it hold found faults that no example would have:

1. **Consecutive list items came out as `- a- b`** — the serializer pushed only the blank line between
   blocks and never the *line break* that always ends one. Every list in the corpus failed at once.
2. **`Quote` dropped its marks** — `> quote with \`code\`` came back as `> quote with code`, because the
   quote writer used `text.text` instead of going through `write_inline`.
3. **An unmatched `**` was deleted from the text.** `a ** marker` parsed to `a  marker`: the delimiter
   had been *consumed* and never written back, which is content loss rather than a cosmetic fault. The
   fix is a two-pass scan — find the pairs with a stack, remove only those.
4. **The second version of that scan looped forever** (`cargo test` hung past a 20-minute timeout): it
   looked a byte up with `position(..)` and had a branch where neither a skip nor a copy ran, so the
   cursor stopped moving. The shape now copies a character **or** skips a known delimiter range and
   nothing else, with an `debug_assert` that the cursor reached the end.
5. **Nested lists gained a blank line** between a parent and its child.
6. **An indented first block violated the invariant** — `"  indented paragraph"` parsed at level 1, which
   cannot be written back faithfully. The first block is now forced to 0.

And one of mine: **marks were given the highlight-span contract**, where overlaps are resolved because
two foreground colours cannot compose. Marks are the opposite — `**bold with _italic_ inside**` is two
marks on overlapping ranges and both must survive — and the first version of the test **asserted the
dropping**, which made the wrong behaviour look intended. `Text::normalize` keeps every nesting now,
and the doc says why the two contracts differ.

### The layout, because it is the half that can be verified

`markdown`'s paint half is 2041 lines of element building, and **visual verification became
unavailable this session** — `screencapture` stopped producing files entirely (`could not create image
from rect`). Painting 2000 lines that cannot be looked at would be exactly the unverifiable work this
port keeps recording, so the **layout** went first instead: line breaking, block positions, the caret
and hit testing are pure arithmetic on a `Doc` and a `Metrics`, and a document's layout is where its
faults actually are.

Three decisions in it, each with a test:

- **The width estimate errs high, on purpose.** A character is `advance`, doubled for the full-width
  ranges. There is deliberately **no narrow correction**, so a line of `i` and `l` wraps earlier than it
  has to — because the lesson `mp/text.rs` paid for twice is that an estimate coming out *under* puts
  content past the edge it was laid out to, while one coming out *over* leaves a little air.
- **A fence is not rewrapped.** Rewrapping code changes what the reader reads, so a code block's lines
  are its own newlines however wide they are, and the paint half clips instead.
- **A click in the gap between two blocks belongs to the block above it**, which is what an editor does
  and what this module's own doc said — the first version tested `y < block.y + block.height` and
  therefore fell through to the *next* block, so the doc was a lie. The test caught it.

The strongest test is a **round-trip through the caret**: for every character boundary in a
multi-block document, the point the caret is drawn at must hit-test back to that same offset. That is
the property that ties the two halves together, and the fault it prevents — clicking where the caret is
drawn puts it somewhere else — is the one a reader notices most.

### The paint half, and the `#[live]` wipe arriving through a new door

`MpMarkdown` is the widget: the theme's metrics in, glyphs out at the coordinates the layout computed. It
sits in `crates/ui` with the other widgets while `makepad-markdown` stays **dependency-free**, the same
arrangement as `MpCodeBlock` and `syntax` — and the same reason: a pure crate's tests run in a tenth of a
second, which is what let the fixed-point property be checked over nine thousand generated documents.

Two decisions in it:

- **A wrap width the caller sets, not `width: Fill`.** A self-drawn widget states its height before it
  draws, the height depends on the wrap, and the wrap depends on the width — so `Fill` is a circle. Taking
  a number breaks it at the cost of the caller knowing its column. The alternative, a measure-then-draw
  pass, has the wrong height on the first frame and jumps once — a flicker on every resize, and precisely
  the class of fault this session cannot check.
- **The painter reads the layout's x and never computes its own.** A marker's width, a quote's padding and
  a line's left edge are the layout's numbers, so the two cannot disagree about where a line starts.

**The `#[live]` wipe arrived through a new door.** `measure` was `#[live]` and mutated through a setter,
which is the fault this port has recorded twice — `Theme::install` raises `request_script_reapply()`, the
re-apply re-asserts every widget's DSL, and every `#[live]` field a setter wrote goes back to its declared
value. It showed up as **two documents in two columns both laying out at the DSL's 640**, the narrow one
silently not narrower and nothing in any log. The value is `#[rust]` now, seated from `initial_measure` in
`on_after_new`.

And the fix's first attempt was wrong in an instructive way: it also had an `on_after_apply` copying the
DSL value across again, which is *the wipe it was written to prevent, one apply later*. `on_after_new` is
the only hook that runs before a caller can set anything.

### `MpEditor`: the half that needs a window, and it is the small half

bezel's split is the reason this was cheap: *"`markdown` holds the document, its markdown wire form, and the
painting — all of it testable without a window. What lives here is the half that needs one."* Every part of that
half except the surface already existed here, so `MpEditor` is only **which key means which shortcut, where the
caret is, where a click lands, and painting it** — 8 tests against the 135 the layers under it carry.

Two decisions in it:

- **Inserting text is not a `Shortcut`.** `makepad-markdown`'s `Shortcut` set is what a **key** does, and a
  character arriving is not one of those — it is text. Adding a `Shortcut::Insert(char)` would open a set that is
  closed for a reason: a shortcut changes the document's *shape*, and text does not. Same for caret motions, which
  change the *selection*.
- **`metrics_for` takes a `&Cx`, not a `&mut Cx2d`.** The theme is reachable from both, and an **event** has only a
  `Cx` — so the first version took a `Cx2d` and had an empty `ensure_laid_in_event` beside it, which made every
  caret motion driven before the first frame silently do nothing. The gallery's script drives motions before the
  first draw, so the fault was visible in one run: `end` did nothing and the typing landed at offset 0.

**Named absences rather than oversights**: no clipboard (⌘C/⌘V need a pasteboard and `makepad-clipboard` is in this
workspace), no IME composition display, no menus, block handles, comments or links — the four `editor.rs`
submodules on the reference's list that are about a *document editor* rather than an editing surface — and no
horizontal scrolling.

### `makepad-editor`: a data structure that was in the wrong crate

`History<T>` lived in `crates/ui/src/mp/history.rs` — a **widget** crate — while its own doc said the one thing
it cannot do is decide when two edits are one undo step, because *"only the caller knows what an edit means"*.
That caller is the editor, so the crate now exists and the stack moved into it, renamed `SnapshotHistory` to
free the name `History` for the document policy — which is bezel's vocabulary for the same split.

**Moving it is what let the document history be written *on top of* it rather than beside it.** The
alternative — a second stack in `markdown`, which must stay dependency-free — would have been sixty duplicated
lines and a second place to get the cursor arithmetic wrong. Fifteen tests moved with the stack, which is why
`makepad-component`'s count went *down* by fifteen.

### The undo model: states in the stack, with an explicit base

The policy is the substance, and the four rules are the ones a reader feels:

| a run of | is |
|---|---|
| typing | **one** step, however many characters |
| deleting | one step — and **not** the same as typing, so type-then-delete undoes to the text, not to the deletion undone |
| a structural change (Enter, Tab, a kind change, a merge) | **its own** step, always |
| a caret move | the **end** of the run before it |

The first version of the model was wrong in a way worth keeping: it recorded the state **before** each edit, so
the stack's current entry was the state before the most recent *run* — and pressing undo after typing `ab`,
pausing, and typing `cd` gave the **empty document** rather than `ab`. One undo skipped a run. The tests caught
it, and the fault was the model rather than the arithmetic: the stack now holds **states**, `History::new`
takes the document an editor has open as an explicit **base** (without which an undo cannot reach the state
before the first edit), and a run overwrites its own entry so its snapshot is the state after its *last* edit.

Two more of my errors the tests caught, both of the same kind: `parse("\n")` gives a document with **no blocks
at all** (a blank line is a separator, not a block), so a fixture that indexed block 0 of it panicked; and an
inline loop in one test still recorded *before* applying after the shared helper had been converted, which made
that test fail a step later than the others and looked like an off-by-one in the undo.

### The editing half: where the flat model's claim is cashed

`lib.rs` says the shape was chosen because *"editing a flat list means Enter splits, Backspace merges and
Tab indents — all list operations"*. `edit.rs` is where that claim is cashed: **every operation is a `Vec`
operation plus a byte splice**, and none of them traverses anything.

The rules are mostly Notion's, and each has a reason: Enter at the end of a heading gives a **paragraph**
(that is how you stop writing a heading); Enter on an **empty** list item outdents it and at level 0 turns it
into a paragraph (without that rule there is no way out of a list); Backspace at the start of an indented
item **outdents** rather than merging (merging is a much bigger edit than the reader asked for); Tab is
**bounded by the invariant**, which is what keeps the serializer's assumption true.

Two structural decisions turned out to matter more than any individual rule:

1. **An invariant that every operation must remember is one that one will forget.** Each operation already
   tried to maintain the indent invariant, and it was not enough: removing the first block left whatever
   became first at its old indent, and one random step produced `[Task at indent 1]` as a whole document. It
   is now established **once**, after whatever happened, alongside the renumber and the fence-mark clearing.
2. **A guarantee is only as strong as what the wire form can express.** The property tests began by asserting
   `parse(serialize(doc)) == doc` after every edit, which is **false and cannot be true**: markdown has no
   spelling for an empty paragraph, for text with a leading or trailing space, or for an empty list item
   whose marker is trimmed. The honest property is the one an editor needs — **the wire form is stable across
   a save/load/save** — and getting *that* to hold found the real bugs.

### Ten bugs the property tests found, none of which an example would have

The exhaustive sweep (every shortcut at every offset of every block kind) and the random session (4000
shortcuts) found, in order:

| # | fault |
|---|---|
| 1 | a shortcut that changed nothing returned `Some(unchanged)`, so the caller could not tell a no-op from an edit |
| 2 | a mark inside a **code block** survived an edit and was silently lost on save — breaking the fixed point |
| 3 | an **empty heading** parsed back as a paragraph whose text was `#`, because the marker rule required a space |
| 4 | an **empty paragraph** wrote a line of its own, giving two blank lines where one belongs — and the parser absorbed one, so the second save was shorter |
| 5 | an **indented fence** re-indented its own content on every save |
| 6 | a split could leave a block whose text **started with a space** (which `parse` trims) |
| 7 | and one whose text **ended** with one, because advancing a single offset past the spaces *gives them to the head* |
| 8 | an **empty bullet** was unrepresentable: `- ` trims to `-`, so it read back as a paragraph whose text is a minus |
| 9 | **numbering did not follow the wire form**: an empty paragraph between two ordered items is skipped when written, so the two become consecutive and renumber to `1, 2` on the next read — the numbering has to be computed on what is *written*, because that is what the next parse sees |
| 10 | the **emitted** sequence's indents were not normalised: with a skipped leading block, the wire form's first block carried an indent that `parse` forces to 0 |

Plus three of my own test errors, each of which had the library right: a selection offset taken from the
*source* rather than the parsed text, a Backspace expectation that removed the character *at* the caret rather
than before it, and a "tab carries the children" test that tried to indent a block the invariant already
allowed no deeper.

**What is not verified:** the paint itself, because screen capture is unavailable. What *is* verified, from
the widget's own printed numbers on the actual page: 14 blocks in both columns, **18 lines at measure 620
and 23 at 300**, heights 520.70 and 624.70, and `drawn` matching the computed height exactly in both — with
the fixed point holding on the displayed document (`parse → serialize → parse` identical, and a second
write byte-identical). bezel's `render.rs` is 2041 lines of gpui element
building, and this port has the model but no renderer — so the gallery has **no page for this crate
yet**, because there is nothing to paint. That is stated rather than papered over with a page showing
the source as text.

### `terminal`: the emulator already exists, and its palette was the missing part

`terminal` was next on the plan as "1296 lines in the reference". Reading it showed an
`alacritty_terminal`-backed emulator — bytes in, grid out — and reading **this workspace** showed the
same substance already here: `canvas-terminal/src/terminal/state.rs` is a **`vte`-driven
`TerminalState`** with a `Cell` grid, `feed(bytes)`, `resize`, `scroll_display`, `cursor_visible` and
`selected_text`, and `canvas-terminal` already depends on **`vte = "0.14"`**. Writing a second
emulator would have been the fifth time this port duplicated something it already had (after `stack`,
`stats`, `menu` and `floating`).

What that file *does* have is a defect this port's first law forbids:

```ignore
const ANSI_COLORS: [[f32; 3]; 16] = [ /* a dark-background scheme */ ];
pub const DEFAULT_FG: [f32; 3] = [0.86, 0.89, 0.94];
pub const DEFAULT_BG: [f32; 3] = [0.10, 0.11, 0.14];
```

A **hardcoded palette at a call site**: the terminal cannot follow an appearance change and a brand
cannot reach it. So the contribution is `TerminalPalette` — the sixteen ANSI colours, a foreground, a
background, a cursor and a selection, in **both** appearances, with `rgb`/`ansi_rgb` conversions so
that replacing that `const` is mechanical rather than a refactor.

### An ANSI palette is judged by a different rule, and stating it took three attempts

Every one of the sixteen is painted on the terminal's own background, so all sixteen are text colours
— **except one**:

**`black` erases.** A program paints it to hide something, so the *result* is that nothing changes,
and it is the ground on a dark terminal and on a light one. The rule went two wrong ways before
landing there, and both wrong turns are kept because each looked right:

1. A floor of **3:1** for `black`, which contradicts its own doc comment. It failed at **1.14:1** on
   the dark palette — correctly.
2. Replaced with "`black` is the *darkest* slot", which failed on the light palette at **"bright white
   is darker than black"**. **That failure was the useful one**: it showed the *palette* was wrong
   rather than the rule — light's `black` had been built as dark ink at `L = 0.28`, giving **13.77:1**
   against a light ground, when a terminal's `black` is the ground whatever the ground is. A terminal
   on white **inverts**: the `white` slots are the ink.
3. The rule came back, and the value was fixed.

So the floors are per-slot and its own doc says which: [`GROUND_CEILING`] for `black` (it must be
*close* to the ground), a floor and a ceiling for `bright black` (real dim text), and
[`TEXT_FLOOR`] for the other fourteen.

**The wiring is done.** `canvas-terminal/src/terminal/state.rs` now reads its colours from
`makepad_theme::TerminalPalette`: the sixteen ANSI slots through `ansi(index)` and the foreground and background
through `default_fg()`/`default_bg()`, with the `const` array gone. Two things about it:

- **One palette per process, and that is a limitation worth naming.** A `Cell` stores its colours **by value**, so a
  cell painted before an appearance change keeps the old colour until it is rewritten — which means switching to the
  light palette needs the grid **reset**, not just the palette changed. Reading through a `OnceLock` therefore gives
  one appearance for the life of the process, and the fix would be a terminal that repaints from history, which that
  file does not keep.
- **An index past fifteen falls back to the default foreground** rather than inventing a colour, because the theme
  answers `None` for the 256-colour cube it does not model — which is what a terminal without the cube does.

Verified: `canvas-terminal` builds; its **39 tests pass** (including the daemon end-to-end ones); and a real run
paints with **zero `[E]`** — the log shows `canvas: terminal cell 10.00x22.00px (font_size 12.5pt) — grid follows
the face`, so the grid lays out and draws. **The colours themselves were not confirmed visually**: the window is
slower to present than the capture window I gave it (it starts a tokio runtime and decodes PNGs first), so the
screenshot came back uniform. The change is mechanical — the same sixteen colours, now sourced from the theme — and
the crate's own suite covers the path, but "the terminal still looks right" is a claim this run did not make.

**The original deferral, for the record.** `canvas-terminal`'s `Cell` carries
`fg: [f32; 3]` and its palette is a `const`, so switching it ripples through `Cell::default()` and the
grid initialisation in a 12,000-line crate with daemon end-to-end tests — and at the time of writing,
screen capture in this session was returning black, so the terminal's rendering could not be checked
after the change. The `rgb` conversions exist to make that change mechanical; doing it blind and
calling it done would be the fault this port keeps recording.

### Three of bezel's modules were already covered in substance

The audit compares *filenames*, which over-reports the gap. Three of bezel's `ui` modules
were already present under another name:

| bezel | here |
|---|---|
| `stack.rs` — "the two stacks, at the system gap" | `mp/layout.rs::Row` / `Column` — same semantics, including that a row centres across and a column stretches |
| `stats.rs` | `mp/scaffolding.rs::MpStatCard` / `MpStatRow` |
| `menu.rs` | `mp/list.rs::MpMenu` (a list with `show_row_lines: false`) |

So the honest remaining count is smaller than the filename diff says, and the spec records
which of the differences are names rather than behaviour.

### An undo stack is wrong in a way nobody reports

The button lights up and the document goes somewhere it was never in. Every fault is a
**state-order** fault, so none shows in a screenshot and none throws. Two actually happen,
and both are performed at runtime on the History page rather than described:

1. **A push after an undo must abandon the redo branch.** Undo twice, edit again, and the
   forward history is gone — a stack that kept it lets redo walk into a state that never
   followed from what is on screen. Runtime evidence, from the default script:
   ```
   HISTORY undo -> Some("doc:3")
   HISTORY push "doc:9" -> "doc:9"
   HISTORY buffer=["doc:0","doc:1","doc:2","doc:3","doc:9"] cursor=4 depth=(4 back, 0 forward)
   ```
   `doc:4` and `doc:5` are gone and redo is closed.
2. **A bounded history must move the cursor when it drops the front.** Ten pushes into a
   capacity of eight:
   ```
   HISTORY buffer=["n3".."n10"] cursor=5 depth=(5 back, 2 forward)
   HISTORY undo -> Some("n9")
   HISTORY undo -> Some("n8")
   ```
   The cursor is genuinely wherever it was, so undo steps one state at a time. Without the
   `cursor -= dropped` the cursor would be out of bounds and `current()` would silently fall
   back to `entries[0]` — the one fault here that produces a **wrong document** rather than a
   wrong button.

**One `Vec` with a cursor, not two stacks.** The two-stack shape makes the truncation rule
an operation on two things that must stay consistent, and the bounded case an operation on
one of them while the other holds the states that were just dropped. A `Vec` plus a cursor
keeps the current state, the undoable depth and the redoable depth as three readings of one
number.

The property test is a **400-step walk across five capacities** that asserts the invariants
after *every* operation — the cursor is a valid index, the buffer never exceeds its
capacity, `current()` is the entry at the cursor, and `back + forward + 1 == len` — then
walks to the floor and the ceiling and back. That covers the boundary cases nobody writes a
test for.

It does **not** coalesce (ten keystrokes are ten steps here) and has no transactions: both
are policies layered on top and both need to know what a state *means*, which is the
caller's business.

### A hand-typed accelerator is a claim nothing checks

Every shortcut the library prints — a menu row's trailing `⇧⌘P`, a list item's `detail`, a
tooltip — is a **claim about the keymap**. A claim written by hand is one that nothing
verifies: bind `cmd-b` elsewhere and the control still says `⌘B`. So the label is
resolved from a declared `Keymap` rather than typed beside the thing it describes, and an
action with nothing declared prints **nothing** rather than a chord that is no longer
true. `Keymap::conflicts` closes the other half: two actions on one chord is a bug where
one silently loses and which one depends on dispatch order.

**The half everyone gets wrong is the modifier order.** Apple writes `⌃⌥⇧⌘` regardless of
how the chord was typed, so `cmd+shift+p` prints `⇧⌘P` and not `⌘⇧P` — a formatter that
preserves the input order gets every Mac shortcut subtly wrong, in a way that looks fine
until two of them sit side by side. Windows has its own order, pinned by two examples that
disagree with each other: `Win+Shift+S` (the platform key leads) and `Ctrl+Alt+Del` (Ctrl
before Alt). Neither is Apple's and neither is alphabetical, so this is not one formatter
with the letters swapped.

Verified at runtime: a 13-action sheet declares with **0 conflicts**, and a deliberately
broken map of 6 reports **3**, including `⇧⌘D` found from `cmd+shift+d` and the glyph run
`⇧⌘D` — a config spelling and a menu paste recognized as one chord.

### `cargo check` does not build tests, and that is how a broken suite was committed

Adding two fields to `Metrics` broke `MpMarkdown`'s own **test** fixture — and `cargo check -p makepad-component`
passed, because `check` builds the library and not the test target. The commit was made on that green signal and the
suite was red: `error[E0063]: missing fields body_size and heading_size`.

`cargo check` is this port's habit for the crates that are not the one being changed, and it is the right tool for
*"does this still build against the rest of the workspace"*. It is **not** evidence that a suite passes, and a
struct change is exactly the case where the two diverge. The rule that follows: when a change touches a **public
type's shape**, run `cargo test` for every crate that constructs it, not `cargo check`.

### Screen capture came back, and the first thing it showed

`GALLERY_PAGE=Document` verified `MpMarkdown`'s paint, which had been unverified for two turns: a heading, a
wrapped paragraph, nested bullets, an ordered list **starting at three**, tasks, a quote and a fence on their
plates, and `*markdown*` correctly *not* emphasised inside a fence. The layout's numbers and the pixels agree.

It also showed a **new defect**: a heading renders at the body size, because the layout uses **one line height
for every block** — so per-kind typography needs `Metrics` to carry a size per block kind, and half-fixing it by
painting a larger heading would overlap the next block. Recorded rather than half-fixed.

And it showed that the segmented control's label was **still clipped** (`Previ`) *after* the DPI factor, which
had been my explanation for it. So the DPI factor was not the answer either.

### The answer was to stop estimating, which took four attempts

The three earlier attempts each made the *estimate* better — a larger pad, a glyph correction, then the missing
DPI factor — and the fourth was to remove the estimate from the geometry:

```ignore
let laid = draw.layout(cx, 0.0, 0.0, None, false, Align::default(), text);
laid.size_in_lpxs.width as f64 * DPI
```

`DrawText::layout` is **public** and returns the size the renderer will use. So `text::measured_width` gives a
real number, and the estimator stays in `text.rs` for **clipping**, where an error in either direction is
invisible.

**And it cost another screenshot to find the unit**: `size_in_lpxs` is in Makepad's **layout pixels**, which are
96-dpi, while `draw_abs` coordinates and every number in `text.rs` are in points — so the measurement is
multiplied by `DPI` like everything else. Without it the measured label came out 25% under and the same clipping
returned by a different route. **That is the third time this factor has been the answer**, so it now has one
home and a doc comment naming the two ways it is met.

### Six attempts on the clipped panel, and the four things it is *not*

The segmented control's panel paints ~193 while its walk says 276.3, and `Preview` is invisible. Six attempts:

1. **A bigger pad** (`SLOT_PAD_X` 14 → 16) to absorb an estimate's error. No change.
2. **A glyph correction** in `mp/text.rs` for symbols. No change.
3. **The DPI factor** the estimate had always been missing. No change.
4. **Removing the estimate entirely** — `DrawText::layout` *is* public and returns the renderer's own number, so
   `text::measured_width` replaced it (and cost a screenshot to find that `size_in_lpxs` needs the same DPI
   conversion). The geometry got **more** correct and the paint did not change.
5. **Moving the width decision from draw time to `set_segments`** — a parent reads its children's *declared*
   walks before calling their `draw_walk`, so a widget that sets its own width while drawing is a frame too late
   every frame. Kept, because it is right, and it did not fix this.
6. **`width: 400` in the DSL.** This one is decisive: the **labels spread out** (following `rect.size.x`) and the
   **panel stayed at ~193**. So the panel is not drawn from the walk the widget is given.

What that bought, and why it is recorded: the fault is **not the measurement**, **not the declared width**, and
**everything a child draws is clipped to its parent's cell** — `draw_abs` as well as `begin`, since the magenta
slot markers lost their third bar for the same reason. The one structural fact verified on the way is that **two
`Fill` siblings in a bar split the remaining space**, which is now written into `mp/bars.rs` beside the spacer.

7. **Reading Makepad's source instead of guessing again.** `View::walk_from_previous_size` resolves a `Fit`
   dimension from that **view's own** last measured size (`view_size`, written at the end of `View::draw_walk`) —
   which is how a `Fit` view converges and why a `Fit` view inside a `Row` works. **A custom `Widget` is not a
   `View` and has no `view_size`**, so as a `Fit` child it was allocated nothing and everything it drew was
   clipped to an empty cell. That mechanism is real and this widget now does its own version of it — **and it did
   not fix this either**, which is itself a narrowing: `last_size` converges on the *requested* size (276.3),
   because `draw_bg.area()` reports the walk rather than the painted box. So the resolution is already correct and
   **the clip happens after it**.

8. **A `Fixed` width on the *container*** — which is the fix. The mechanism is one sentence:

   > `View::walk_from_previous_size` resolves a `Fit` width from **that view's own previous area**, **not from its
   > children's content.**

   So a `Fit` container first given less than its child needs is a **self-reinforcing fixed point**: a small box,
   so it clips, so its area is small, so it asks for the small box again. And **everything a child draws is clipped
   to that box** — `draw_abs` as much as `begin` — so nothing the control drew could escape it.

   **Verified by contrast as well as by the fix**: the same control on the Controls page never showed the defect,
   because its container is a `Fill` row. `Fill` ✓, `Fit` with an explicit width ✓, `Fit` without ✗.

**The lesson is the shape of the search**: a widget whose geometry is *provably correct* while its paint is not has
a **parent** problem, and the measurement is the wrong place to look — which is where seven of the eight attempts
went. The rule is written where the slots are declared (`mp/bars.rs`), because it is the caller's to know what its
slot holds.

### One thing the measurement did *not* fix, recorded as an open issue

With the labels measured, the control's geometry is **provably right**. `MP_SEG_DEBUG` printed the measured
widths (`Source=55.9 Split=36.7 Preview=64.1`), the box at `276.3`, every label's x, and the area each draw
landed in — all three inside the box, `Preview` ending at **262.3 of 276.3**. And the **painted** box measures
about **195** from the screenshot, with `Preview` invisible.

So the walk this widget computes is not the box it draws, by about eighty points — and the fault is **not in
that file**, because the numbers it hands the renderer are self-consistent, and independent unit tests cover the
slot arithmetic. Recorded in the source and here, because it is the second time this port has met the same
shape (`mp/markdown.rs` set a width and read back a different one) and the next person should start from *"the
walk is truncated between the widget and the paint"* rather than from *"the measurement is wrong"*, which is
where three earlier attempts went.

### An estimate cannot be exact, and the direction that hurts is *under*

`mp/text.rs::width` estimates rather than measures, and that is right for **clipping**
(being a point either way is invisible). It is wrong for **placement**, where an
under-estimate puts text past the edge it was aligned to. This port paid for it twice:

1. `mp/segmented.rs` — an underestimated label made a control's box too narrow and the
   Bars toolbar drew `Preview` as **`Previ`**.
2. The Shortcuts page — a list's trailing chord is right-aligned, and `⇧⌘S` was drawn
   with its `S` **past the panel**, under the page's scroll bar.

Both had one cause: **a glyph is not an average character.** `⌘`, `⇧`, `⌥` and `⌃` were
each counted at the average Latin advance when their real advance is about **1.25 em**
against `ADVANCE = 0.508 em`. The correction that reaches it is `1.45`, and it is derived
rather than guessed. That closed most of the gap; `DETAIL_SLACK` closes the rest, and it
is the same decision `mp/segmented.rs` reached — **a slot carrying the measurement's error
margin beats a label that collides.**

### A test that asserted a proxy for a property it could not see

Raising the symbol correction turned two tests in `mp/scaffolding.rs` red. They asserted
that `text::width` gives `⌘` and `A` the same estimate, "because the mono face is what
makes a row of caps align".

**The proxy was the wrong thing to assert, and it came apart.** `MpKbd` does not measure
anything: it draws with `Walk::fit()` and lets Makepad lay the text out in
`theme.font_code`, so the advance that aligns a row of caps is the **font's** and this
crate's estimator is not involved. The property the tests stood for was untouched while
both went red. They are removed, with a note saying why, and what is asserted instead is
the estimator's own behaviour in `mp/text.rs` — where it belongs.

### The declared font size is not the painted one

`mp/text.rs` estimated every width from the **declared** `font_size`, and Makepad lays text out at
96 dpi — so a `font_size` is multiplied by `96 / 72` before a glyph is placed, and **every estimate in
this port was about 25% under the paint.**

That is the real cause of two faults that were each "fixed" with padding before the constant was
found:

| symptom | the padding fix | the real cause |
|---|---|---|
| a segmented control's last label clipped — `Preview` drawn as `Previ` | `SLOT_PAD_X` raised to 16 to carry "the measurement's error margin" | the estimate was 25% under |
| a list's trailing chord past its panel, under the scroll bar | `DETAIL_SLACK` added, after a symbol correction was raised | the same 25% |

Both fudges stay at their values — the two points are indistinguishable in the rendered control, and
a number whose fault was fixed elsewhere is the last one to change — but their **reasons** are
corrected, because a fudge documented as load-bearing when it is not is how the next person tunes the
wrong thing.

**The derivation, and the measurement that pinned it.** Both mono faces this workspace bundles
advance **0.6 em** per character, read from their own `hmtx` tables rather than assumed
(`LiberationMono-Regular.ttf` is 1229/2048, `jetbrains_mono_variable.ttf` is 600/1000, and every
glyph in each takes the same advance). `MpCodeBlock` then drew a 16-character run at 12pt and read
back the `Rect` — 153.62 wide, **9.6016pt per character** — and the same run at 24pt came back 307.25,
**19.2031**. The ratio is `0.8001 / 0.6 = 1.3336`, and `96 / 72 = 1.3333`, at both sizes. So
`MONO_ADVANCE = 0.6 × 4/3` and `ADVANCE = 0.508 × 4/3`.

Verified after the fix, from the widget's own numbers rather than a picture: `advance = 9.600` against
the paint's `9.6016`, so **the estimate and the paint agree to 0.0016pt per character** — 0.13pt over
the 84-character widest line, where the gap had been 200pt.

### The estimate's font size must be the painted one

`mp/list.rs` measured its rows with `theme.metrics(Body)` / `theme.metrics(Caption)` while
*painting* them with whatever `text_style` the DSL gave each `DrawText`. Those agree only
if `mod.mpc.type.*` happens to resolve to the theme's own metrics. They did not, and the
fix is the one `mp/segmented.rs` already records: **read the size back from the draw
target.** Naming the size once, from the thing that paints, is what makes the measurement
and the paint incapable of disagreeing.

`MP_LIST_DEBUG=1` prints a row's rect width, padding, both font sizes, the clipped detail,
its estimated width and the aligned x — because a right-alignment fault is invisible in a
log and a screenshot only shows the symptom, so the numbers have to come out of the
widget. That is how the fault above was located.

### A right calendar is wrong on a small fraction of dates

That is the worst possible failure rate: it survives every look-at-it test and then
puts an appointment on the wrong day. So the arithmetic is the module and the grid is
the easy part, and the tests are aimed at the two ways it goes wrong invisibly:

1. **The leap rule.** A century is a leap year only when divisible by 400 — 2000 is,
1930 and 2100 are not — so an approximation is correct for three years in four.
2. **The weekday of a date**, which needs a real algorithm. The one here is Hinnant's
`days_from_civil`, and the test that it is right is **not an anchor date**: it is a
second, independent computation that walks one day at a time from the epoch and
advances the weekday by hand — forward *and* backward, because the backward walk is
what exercises the era division's `- 399` (without it every date before 1970 is a day
off and no modern date shows it). Two more properties carry the weight no anchor can:
the **400-year cycle** (400 years is exactly 146097 days, a whole number of weeks, so
every date shares its weekday 400 years later — true only if the century rules are all
right), and the grid invariant checked over a **whole Gregorian cycle** (1970..2370 ×
12 months × 2 week starts): every day present, in order, in consecutive cells.

The weekday arithmetic was then checked **against the OS's own calendar** for seven
dates spanning 56 years — `2026-09-01 Tuesday`, `2026-09-17 Thursday`, `1970-01-01
Thursday`, `2000-01-01 Saturday`, `2024-12-25 Wednesday`, `2026-02-01 Sunday`,
`2026-05-01 Friday` — all seven matching.

### The library does not read the clock, and that caught a real bug

`today` is set by the caller. A widget calling `SystemTime::now()` itself would be
untestable, would disagree with the app's own idea of today across midnight, and would
make "is this cell today" uncheckable without waiting a day.

Having made that split, the *app* then got it wrong in a way worth recording: it
computed today as `epoch_seconds / 86400`, which is the **UTC** day. On this machine
(`UTC+8`) that is the *previous* day for the first eight hours of every local day — the
page printed `today=2026-09-16` while the system clock said `2026-09-17`, and a calendar
ringing the wrong day is wrong in exactly the way this module's own doc calls the worst
failure rate. The fix is `date_from_epoch_seconds(seconds, offset_seconds)`: the offset
is a **parameter**, because a library cannot know the zone, and the app asks the OS
once via `libc::localtime_r`'s `tm_gmtoff`. Tests now cover the case that broke, both
directions (a zone east of UTC can be a day ahead, one west can be a day behind), and
the pre-epoch sign case where `div_euclid` floors and `/` would truncate.

Verified after the fix: the app prints `today=2026-09-17 (offset +28800s)`, matching the
system's local date.

### A glyph in the wrong face is tofu — and the first explanation of this was wrong

The calendar's month arrows were `\u{f053}` and `\u{f054}` and drew as two tofu boxes. The
note written at the time said FontAwesome's chevrons "are not in this port's icon subset".

**That was false, and it stayed in the spec until it was checked.** Parsing
`fa-solid-900.ttf` — the face Makepad ships as `theme.font_icons` — shows **1976** codepoints
with both chevrons among them. The real cause is the **face the draw target uses**:
`draw_weekday` declares `text_style: mod.mpc.type.caption`, the *text* face, and a character a
font does not have is tofu — no error, no warning, nothing in any log.

So there are two questions with one symptom, and `mp/icons.rs` plus `tests/icons.rs` now answer
both mechanically:

1. **Is the glyph in FontAwesome?** `icons::ALL` is the declared set, and the test parses the
   font's `cmap` table — the table a renderer consults — and asserts every entry is present.
2. **Is it drawn with the face that carries it?** A file that writes a FontAwesome codepoint must
   reach `theme.font_icons` either directly or through `MpIcon`. A file with neither is drawing an
   icon glyph with a text face.

Both were **falsified before being trusted**. Planting the calendar's original mistake — a
`\u{f053}` in a file whose only face is the text face — makes the second check report
`["date.rs"]`. Planting an undeclared `\u{f999}` makes the first report `["date.rs: U+F999"]`.

The falsification also found a **gap in the check itself**: the first version restricted its scan
to the Private Use Area, `0xE000..=0xF8FF`, so a planted `F999` was **skipped entirely** — it is
above the ceiling. FontAwesome's base is `F000`, not the PUA's, and Pro draws above `F8FF`; the
bound is `>= F000` now. A check that only looks where it expects to find something is a check
that cannot find the thing it is for.

The arrows themselves are still `‹` and `›`: they are two marks in a header, and a text-face mark
is cheaper than a second `DrawText` carrying the icon face.

### A widget that must exist rather than be composed, and the two bugs its first render showed

`MpSegmented` cannot be a composition the way `MpPopover` is. A segmented control's
whole visual identity is that the segments are **adjacent** — one rounded track with a
plate inside it — and three independent buttons have three rounded outlines with gaps
between them. The Bars page had been using exactly that as a stated interim, and said
in the page that it was one. This module replaced it.

Its two pure functions are the hit test and the plate, and they have to describe the
same slots, so the test that matters is that they **agree at every boundary**. Both
were written before their guards, which paid off twice:

1. **The NaN guard covered the position and not the box.** A `NaN` width passes
   `width <= 0.0` (every comparison against NaN is false) and passes `x >= width`, so
   the whole guard was skipped, `slot_w` became NaN, and `NaN.floor() as usize`
   **saturates to 0** — a control whose layout had not settled reported a hit on slot 0
   for any pointer position, including one outside it. The test was written first; the
   guard was not.

2. **The font size was written instead of read.** `MpSegmented` declares
   `text_style: mod.mpc.type.body` and `MpSegmentedSmall` overrides it to `caption`,
   and the first `draw_walk` assigned the Body size over the top of whatever the DSL had
   chosen — so the small variant drew at the body size *and* the width estimate used a
   different number than the paint. Reading `self.draw_label.text_style.font_size` back
   fixes both at once, and it is the same rule `mp/pagination.rs` states: set the drawn
   size from the rung the arithmetic uses, so the two cannot disagree.

That second bug is the one a screenshot found and no log did: the control's box came
out too narrow, so the Bars toolbar drew `Preview` as **`Previ`**.

### The measurement is a heuristic, and one control has to pay for it

`mp/text.rs::width` is a base advance with narrow and wide corrections — an estimate.
It is good enough where this crate has used it so far, because a table cell or a
pagination digit is clipped or centred by it and an error of a point is invisible. A
segmented control is different: the estimate **is** the box, so an underestimate makes
the last label overflow and be clipped. `SLOT_PAD_X` therefore carries the
measurement's error margin as well as the label's air, and says so. A real text
measurement would let it be chosen for the air alone; Makepad exposes one only
internally, in the HUD.

### A cursor held as a position is wrong

`MpPalette`'s composition is nothing new — a field, a divider, a list, all of which
already existed. What is new is the part in between: a query that re-ranks on every
keystroke, and a **cursor that follows its own row while the view shrinks underneath
it**.

A cursor held as a *position* is correct until the first query that filters its row
out, at which point it has silently moved onto a different command. **Running the
wrong command leaves no trace in any log.** So the cursor is held as an **original
index** and remapped by identity.

Proven at runtime rather than argued: with the cursor on `Duplicate` in the view for
`de` (`Delete, Duplicate, Command Palette`), a second query of `du` narrows the view
to `[Duplicate]`, and the app prints

```
PALETTE query="de" ranked=[...] cursor_pos=Some(1) active_original=Some(6)
PALETTE query="du" ranked=["Duplicate"] cursor_pos=Some(0) active_original=Some(6)
```

— the **position moved 1 → 0 while the command stayed 6**. A position-held cursor
would have stayed at 1, which is out of range for a one-row view.

### Two traps found only by running it

1. **`MpList::select` emits the same `Selected` action a click does**, and the
   action carries a *position*, not an identity. A caller that re-filters and
   re-selects in one pass therefore reads its own highlight back as a selection —
   and if the view shrank in between, the position is **stale and can be out of
   range**. `remap` returning `None` for it is not a fix; the caller has to remember
   the position it asked for. Filtering by *identity* is not sufficient either,
   because an out-of-range position resolves to `None`, which is not equal to the
   current cursor, so a `selected != active` check lets it through — observed as
   `chosen_by_click original=None` at the end of the very run that proved the shrink.

2. **Registration order, again.** `mp/palette.rs` composes an `MpMenu` in its own
   `script_mod!`, so it must register *after* `mp/list.rs`. It was placed with the
   other compositions near the top and failed at runtime with "property MpMenu not
   found in prototype chain" — a message that points at the **user** of the name, not
   at the line that is in the wrong place. That is the second time this port has paid
   for this rule.

### A new page can overwrite an old one, silently

The Command Palette page's first version registered the DSL path
`mod.gallery.pages.palette` — which the **colour** palette page already owned. So
`crates/gallery/src/pages/palette.rs` was *overwriting that page's DSL tree*: the
colour page's rail row would simply have rendered a command palette, and nothing
would have errored.

The crate's own uniqueness tests caught it — first the duplicate **title**, then the
duplicate **path**, then the slot-table mismatch — and each fix revealed the next,
because the three are separately maintained. The rail's four hand-maintained tables
(`PAGES`, `PAGE_SLOTS`, `RAIL_ROWS`, `SLOT_PAGES`) are hand-maintained *because*
`ids!` needs literals, and this is the second defect their tests have caught (the
first was `GALLERY_PAGE=Loaders` opening the Layout page).

The lesson generalises past this crate: **a DSL path is a global namespace, and a
new page claiming a name does not conflict — it replaces.** Verified after the fix
that the colour Palette page renders its own "Surface ladder" again and the new page
renders under its own title.

### A page whose evidence is not a screenshot

Typing cannot be delivered by the screenshot script (synthetic pointers do not hit in
this app), so the Palette page takes its query from `GALLERY_PALETTE_QUERY` and runs
the *same* `apply_palette` the `changed` handler runs, printing the query, the ranked
view, the cursor position and the active original index. **A screenshot cannot show
which original command a cursor is on, and that is the only part of this that can go
wrong invisibly** — so for this page stdout is the stronger evidence, and it is the
first page in the port where that is true.

### Three strips, one module — and a height only where the platform has one

All three bars are horizontal strips that hold other things, and a strip's only real
decisions are how tall it is, how much air it has at the sides, and where its content
sits vertically. So they are three prototypes in one module, by the rule this crate
has used since `mp/control.rs`: two widgets needing the same arithmetic is one
module, not two copies.

**Only the titlebar reads a height, and it reads the theme's.** `layout.titlebar_height`,
`layout.titlebar_top_pad` and `layout.traffic_light_inset` were already in this port's
layout tokens from the reference, with their source noted where they are defined, so
nothing in `bars.rs` was chosen. `MpControlBar` and `MpMenubar` take `Fit` — their
height is their content's. That is the decision worth stating: a control bar is 32pt
in one app and 44pt in another, and a library that picks one is a library that will
be overridden.

The traffic-light inset is a **spacer, not padding**, because macOS draws its window
buttons inside the titlebar's own rectangle — padding would move the bar's background
too, and the background is supposed to run *under* the buttons. The Bars page shows
the consequence: the titlebar's content is indented past the inset while the control
bars below align to `layout.space`, which is a visible difference and is correct.

A menubar's triggers are `MpButton` ghosts rather than labels, so they get the press,
focus and hover behaviour every other control already has instead of a second
implementation of the same three states.

### Two runtime-only findings from this page

`script_mod!` is validated at **runtime**, not by `cargo check` — so `cargo build`
passing is not the gate, and the gate is `[E] = 0` on a run. Three names invented for
the first version of this page (`MpButtonStyle`, `MpIconButton`, `MpSegmented`) all
failed at runtime with useful suggestions; the type is `ButtonStyle`, and there is no
icon-button or segmented prototype.

And **`text: ""` is load-bearing on an icon-only button**: `MpButton` ships a
placeholder label, so setting only `glyph` gives an icon *and* the word "Button".
Nothing errored — the two titlebar buttons rendered as `＋ Button` and `✓ Button` —
which is the fourth time in this port that a silent failure and a working feature
looked identical in the log.

### A ranking is a weighted score, not a comparison order

The first version of `mp/search.rs` ranked matches with a derived `Ord` on three
keys — first hit, longest run, word starts — and it was wrong, in a way that took
three passes to see because **two of the preferences point in opposite directions**:

- `nt` must find `New Terminal` rather than a substring of an untyped word, so a
  **word start has to be able to dominate**.
- `term` must find `New Terminal` rather than `The remote endpoint`, which also
  matches two word starts (`T` of `The`, `r` of `remote`) but scatters — so a
  **contiguous run has to be able to dominate a word start**.

No lexicographic order satisfies both. `first_hit, run, boundaries` passed `term`
and got `nt` wrong; the documented `boundaries, run, first_hit` passed `nt` and got
`term` wrong; and **each order passed a majority of the tests**, which is how the
disagreement survived the first reading. The fix is the thing real matchers do —
`WORD_START = 8`, `CONTIGUOUS = 4`, chosen rather than measured, with the position
kept out of the score entirely and used only as a tie-break. `or` fixes the ratio:
it must find `Off Road` (two word starts, 16) over `Word` (one contiguous run, 4).

Two lessons worth keeping:

1. **A derived `Ord` makes the field order the rule.** The struct declared
   `first_hit` first while its own doc comment claimed word starts outranked
   everything, and the tests passed anyway. Code and comment disagreeing is
   invisible when a majority of cases agree under both.
2. **Folding a tie-break into the score is not a tie-break.** Subtracting the
   match position ("a mild preference for matching sooner") tied `New Terminal`'s
   20 down to `The remote endpoint`'s 16 and handed the row to the worse match. A
   tie-break only breaks ties if it runs after the score.

The page shows all four cases seeded **from `rank()` itself** rather than from
expected output written into the page — a page showing the expected answer is a
second copy of the tests, and a page calling the function regresses visibly.

### The chrome nobody looked at

Every page in the gallery scrolled inside a bare Makepad `ScrollYView`, whose handle
is painted from **Makepad's own theme** (`theme.color_outset` and its hover and drag
siblings) — not from this palette. So the one piece of chrome on every page was the
one piece that was not designed here, and it stayed that way through twenty-four
pages because **a scroll bar is chrome and nobody looks at chrome**.

`mp/scroll.rs` themes it and the gallery's page area uses `MpScroll` now, so the fix
lands on every page rather than on a demo page. The general lesson: a component
library that does not theme the parts of the framework it uses has a hole in it
exactly where nobody looks, and "I built the components" is not the same claim as
"every pixel on the screen came from this library".
| `mp/feedback.rs` | `MpProgressRing`, `MpSkeleton` — the determinate ring and the shape of content that has not arrived. |
| `mp/scaffolding.rs` | `MpGroupBox`, `MpGroupBoxPlain`, `MpKbd`, `MpStatCard`, `MpStatRow`, `MpEmptyState` — the assembly-only components: no Rust, and each is a rhythm rather than a drawing. |
| `mp/pagination.rs` | `MpPagination` — **the arithmetic is tested and the row renders; the page numbers do not draw.** See below. |

### A missing origin and a missing paint look identical in a screenshot

`MpPagination` rendered its plates at the right widths with the current page's wash
in the right cell and **no numbers**. That read as a text-painting problem for
three build-and-look cycles, and was exhausted as one: the colour was visible, the
label non-empty, the widget does capture an `Area` (checked by grep, after a false
claim that it did not), the text style was set from Rust as well as from the DSL,
and the same `draw_abs`-outside-the-turtle pattern renders in `mp/table.rs`.

It was arithmetic. `cell.pos.x` already carried `origin.x`; the y expression did
not carry `origin.y`, so every digit was drawn at the top of the *window*, behind
the rail. The table computes `origin + dvec2(x, y)` and never had the fault.

**The rule:** when text does not appear, ask *where it would have landed* before
asking whether it was drawn. A missing origin and a missing paint produce the same
screenshot, and the position question is answerable by reading the expression. This
widget now carries a test that asserts the arithmetic rather than the render.

### One arithmetic, three widgets

`MpTable` and `MpTree` each grew their own line-measurement — one with a
narrow/wide correction and one with a flat estimate — which means two widgets
could clip the same string at different points. `mp/text.rs` holds it once now and
all three call it, including the right-alignment the table and the list both need.

The general shape is worth naming, because it is the same one `mp::control` has:
**two widgets that need the same arithmetic are one module, not two copies.**
Three of this crate's recorded faults came from a duplicated copy — the canvas
terminal's minimised strip and `mp::control`'s hit contract both had a layout
written twice, and a row clickable where it was not drawn is what that produces.

### The glass is real, and the page that proved it had to be built twice

`mp.SurfaceGlass` — Makepad's mip-chain backdrop blur with the measured SwiftUI
frost numbers — had never been rendered since it was written. The Surface page was
added to check it, and its **first version proved nothing**: the backdrop was five
plates from the surface ladder, all within a few levels of each other, so the glass
looked identical whether it blurred or not — and a plain tinted rectangle would
have looked identical too.

Rebuilt over **maximum-contrast stripes** the reading is unambiguous: through the
tile the bars are soft and lifted toward the tint, and outside it they are
hard-edged white on black. That is what distinguishes a real backdrop blur from a
fill, and it is the reason the page's own caption says so.

The general lesson, and it applies to every visual claim in this document: **a
verification page whose subject has nothing to act on verifies nothing.** Flat
colours cannot show a blur, equal tones cannot show a hairline, and a screenshot of
either is not evidence.

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

# 与 bezel 的对等审计：方法、结果、还差什么（2026-09-17）

## 之前的「44 个模块 vs bezel 34」是没有意义的数字

那是一个**代理指标**：我自己目录里的文件数，对着一个我凭印象记下的数字。两者都不是「组件」。这一轮把它换成一个能回答问题的检查：

```
ls crates/ui/src/mp/ | sed 's/\.rs$//' | sort > /tmp/mp.txt
ls crates/ui/src/widgets/*.rs crates/ui/src/*.rs | sed 's|.*/||;s|\.rs$||' | sort -u > /tmp/bezel_all.txt
comm -23 /tmp/bezel_all.txt /tmp/mp.txt      # bezel 有、我没有同名模块的
```

bezel 的 `crates/ui/src/` 是 35 个模块名（`widgets/` 只有 9 个文件，因为 `buttons.rs`/`content.rs`/`controls.rs` 是**分组模块**）。11 个名字在我这里没有同名模块，逐个查证后：

| bezel 模块 | bezel 导出的类型 | 我这边 | 结论 |
| --- | --- | --- | --- |
| `buttons` / `content` / `controls` | — | 我是按组件拆的（`button.rs`、`checkbox.rs`…） | **不是缺口**：分组方式不同 |
| `lib` | — | — | 不是组件 |
| `stack` | 无（22 行） | `mod.row()`/`mod.column()` 是 gpui 的 `HStack`/`VStack` 助手，makepad 的 `flow: Right/Down` 就是它 | **不是缺口** |
| `history` | `SnapshotHistory` | `crates/editor/src/history.rs` 的 `SnapshotHistory<T>` | **已覆盖**（当初刻意移进数据 crate） |
| `control_bar` | `Shape` | `mp/bars.rs` 的 `MpControlBar` | **已覆盖** |
| `menu` | `Item`（含 `Item::Submenu`）、`Cursor`、`Hit`、`card()` | 无 | **缺口**：带子菜单的下拉菜单卡 |
| `menubar` | `Menu`、`Menubar`、`MenubarEvent` | 无 | **缺口**：菜单栏本体 |
| `stats` | `Stats` | 无 | **缺口**：FPS/CPU/GPU/内存计量表 |
| `titlebar` | `DragState` | 无 | **缺口**：自绘标题栏 + 拖拽 |

**所以答案不是「44 > 34，领先了」，而是「还差 4 个，其中 2 个是真组件」。**

**进度更新（同一天的后续提交）**：`menu` 这一格的**模型与面板都已落地** —— `mp/menu.rs`
（`Item`/`Cursor`/`Hit` + `next_selectable`/`items_at`/`open_depth`，14 测试）与 `mp/menu_card.rs`
（`reserves_gutter`/`panel_width`/`row_height`/`row_top`/`row_rect`/`panel_height`/`row_at`/
`submenu_origin`/`within`，10 测试），gallery 第 45 页 "Menu Cards"。运行时证据：
`MENUS plain rows=6 width=180 gutter=false` / `glyphs … gutter=true` / `described … width=280`，`[E]=0`。

**四个缺口现在都有了模块与 gallery 预览**（同一天的继续）：

| 缺口 | 落地 | 运行时证据 |
| --- | --- | --- |
| `menu` | `mp/menu.rs`（模型，14 测试）+ `mp/menu_card.rs`（面板，10） | `MENUS plain width=180 gutter=false` / `glyphs gutter=true` / `described width=280` |
| `menubar` | `mp/menubar.rs`（状态机，14）+ `mp/menubar_strip.rs`（标题条，6） | `hover_closed=none … hover_open=changed`；`strip content_width=190 rects=[File@+0w58, Edit@+58w61, View@+119w71]` |
| `stats` | `mp/stats.rs`（模型 + 控件，9） | `STATS counted=74 span_frames=75 excluded=1 span=1.250s fps=59.2` |
| `titlebar` | `mp/titlebar.rs`（模型 + 控件，7） | `TITLEBAR region width=320 controls=80 draggable=[0,240) inside_at_end=false` |

**仍未做**：A2UI 的 `calendar` 池（需要一个第 6 个 v2-only 组件：一个**可配置的 timetable 网格**，不是月历），
以及阶段 6 的删除（**需要用户明确同意**）。

**这一轮由「测量」而不是「阅读」抓到的两个真缺陷**：

1. **`HOLD` 与 timer 周期相等 → 读数抖动。** 实测跨度 `1.0s, 2.0s, 1.0s, 2.0s`：timer 的真实周期落在标称值头发丝
   之下（`delta=Some(0.9998091250000001)`），`elapsed >= HOLD` 失败，读数只好等下一个 tick。**一个随机抖动会落在
   两侧的边界不是 hold。** `TICK = 0.25` 给出 4× 余量。
2. **单位阶梯漏了 KiB**：1024 字节打印成 `0.0 MiB` —— 一个测量值被四舍五入掉、显示成了零。

**还有一个「格式化自己输入」的诊断**：`span` 在 `take` 之后才读，于是每次读数都打印 `span_frames=0`；更早的一行
写死了 `own_ticks_excluded=yes` —— 那是我写的字符串，不是代码算出的数。现在打印的是平台帧号与计数之差，
**`excluded=0` 会说明规则根本没触发**。

## 这 4 个缺口各自是什么性质，不能一概而论

- **`menu` + `menubar`（合计 1092 行）是真组件**，而且是常用件：一个 `Item::Submenu` 行、一个 `Cursor`（哪些子菜单打开、哪一行是 live，指针与键盘**都**移动它，所以两者不可能对同一行有分歧）、一个 `Hit`（指针做了什么，返回给调用方，**动作仍归调用方**）。我这边的 `mp/combobox.rs` 有面板+行的模型，但**没有子菜单**。这是下一个该做的。
- **`titlebar` 是**真缺口**——这条我先前写错了，而且是没有查证就写下的。** 我当时写「makepad 有 `cx.start_dragging()`」，查过之后：
  - `CxOsOp::StartDragging(items)` 是 **拖放文件**，不是拖窗口 ✗ 我记错了；
  - 但 `CxOsOp` 里有 `HideWindowButtons()` / `ShowWindowButtons()` / `SetWindowTitle(..)`，而
    `platform/src/window.rs:805` 有 `pub fn reposition(&self, cx, position: Vec2d)` ✓ **所以自绘标题栏是可移植的**：
    隐藏原生按钮，指针拖动时 `window.reposition(cx, pos)` 即可。
  - **一个真实的平台警告**：`linux_wayland.rs:935` 对 `RepositionWindow` 是**空实现** —— 所以 Wayland 下自绘标题栏
    无法用拖拽移动窗口。这是「平台上的一条限制」，不是「整个组件不适用」。
  
  结论：`titlebar` 从「不算缺口」改成**真缺口，可做**。
- **`stats` 是诊断件，依赖 gpui 内部**：它数的是「本窗口的渲染次数」，而那个数字之所以等于帧率，是因为 gpui 对每个未缓存 view 每帧渲染一次；`Painter::woken` 用来区分「哪些是它自己 tick 引起的」。makepad 没有对应的 `woken`，GPU 占用也没有对应测量点。**移植会得到一个只剩 FPS+内存的版本**——那是一个诚实的降级，但必须写清楚降了什么，而不是假装对等。

## 这一轮由「运行」而不是「阅读」抓到的三个真缺陷

三次都是同一类：**代码与它自己的注释/声明不一致，而 `cargo build` 对此是绿的**。

1. **`columns_clamped(0)`**：v2 把 0 映射到 8（一个默认网格），我写成 `.max(1)`，把「未设置」读成了「最小的网格」。协议里**根本没有 `columns` 字段**，所以 A2UI 渲染器画出的每一个取色器都是**一列九个**——运行时打印 `columns=1` 才看见。
2. **`MpAvatarRow` 的注释在撒谎**：注释说负外边距是「脸大小的比例，`-28%`」，代码写的是 `-8` 字面量，于是 40pt 的脸重叠仍是 8 而非注释承诺的 11。v2 原版是一张五行的字面量表（5/7/8/10/12），**没有任何出处**。`OVERLAP_RATIO = 0.28` 把那行表折成注释描述的那一个数：在它覆盖的每个尺寸上都复现原表（28→8，40→11），且对没人量过的尺寸也有定义。
3. **`mod.mpc.ControlSize.Medium` 不存在**：`ControlSize` 只有 `Small`/`Regular`/`Large`。**脚本里写错的变体名 `cargo build` 不会报错**，它在运行时才炸。

三个都不是靠读 diff 找到的，靠的是**开起来看打印**。

## 两个被测试而不是被人抓到的静默失败

同一个模式出现两次：**`str.replace` 的锚点不存在时它什么都不做，并且报告成功**。

- gallery 的 `picking::script_mod(vm)` 没被插入（锚点写成了 `radio::script_mod(vm)`，那个文件里没有）→ 运行时 `[E]=1 property picking not found`。
- `pages/mod.rs` 的 `declared` 表里没有 `picking` 条目（锚点同样不存在）→ **测试** `every_page_module_is_declared` 抓到。

**教训**：编辑之后的锚点断言（`assert anchor in s`）不是仪式，是唯一能在同一个命令里发现自己什么都没改的办法。

# 阶段 6 的真实前置：v2 半边的使用者盘查（2026-09-17）

之前写「阶段 6 = 删 80 个 v2 文件 + `component-zoo`」，并把它标为「只差用户同意」。**这个判断是错的**，
因为我只是猜的。把 `crate::widgets::`（v2 路径）的所有引用查一遍之后：

```
grep -rn "crate::widgets::\|makepad_component::widgets::" crates/*/src crates/*/tests | grep -v "^crates/ui/src/widgets/"
```

盘查结果（`crates/ui/src/widgets/` 自身除外）：

| 使用者 | 用了什么 | 性质 |
| --- | --- | --- |
| `crates/ui/src/lib.rs` | `crate::widgets::script_mod(vm)` | **注册本身**，删 v2 时自然一起去掉 |
| `crates/component-zoo/` | 整个 v2 展馆（9250 行） | **就是要删的那个 app** |
| `crates/a2ui-demo/` | `widgets::button::MpButtonAction`、`widgets::focus::handle_key` | **已修**：两者 v3 都有同名变体/函数，两行改成 `mp::` 即可 |
| `crates/dbpro/` | **12 个 v2 组件**：`mod.widgets.MpTree`、`MpTable`、`MpTextArea`、`MpButton`(+Ghost/Prominent/Secondary)、`MpDialog`、`MpDropdown`、`MpInput`(+Password)、`MpScrollXYArea`、`MpSelect`(+Trigger/Option)、`MpSplitPane`；外加 `widgets::sizing::MpSize` 与 `use widgets::*` | **真前置**：删掉 v2 半边会让 dbpro 完全不可编译 |

**所以「删 v2」不是一个删除动作，而是一个 app 迁移动作**：`dbpro` 是一个完整的数据库 GUI，它的 DSL 里直接写着
`mod.widgets.*`。在它迁移到 `mod.mp.*` 之前，v2 半边**仍然有真实使用者**，删不得。

另外两条也要记：

- `widgets::sizing::MpSize` 在 v3 里**没有对应类型**——v3 是逐组件丢掉 `MpSize` 的（`step_indicator` 与 `color_picker`
  的模块文档都写着为什么：五档尺寸是主题排版的事，调用方选一个就等于同时选了字号/圆角/内边距）。dbpro 迁移时
  要么改用 `ControlSize`，要么把它自己那处尺寸写成常量。
- **v3 没有 `MpTable`、`MpTree`、`MpTextArea` 的完全对等物**吗？有的：`mp/table.rs`、`mp/tree.rs`、`mp/editor.rs`
  都在，但 API 形状不同（v3 用 Rust 侧的数据 API，不是 DSL 里写子节点）。**这正是 dbpro 迁移不是改个前缀的原因。**

**结论：阶段 6 的完成判据不是「用户同意」这一条，而是**
1. `dbpro` 迁移到 `mod.mp.*`（一个 app 重写，含 `MpSize` 的替换）；
2. 用户明确同意删除 `component-zoo`（9250 行）与 80 个 v2 文件（27,172 行）。

第 1 条之前没有人知道它存在，因为没人查过引用。

## 实测 dbpro 迁移的规模，而不是猜

「不是改个前缀」这句话本身也是猜的。做了一次实验：把 dbpro 里 `mod.widgets.Mp*` **只换前缀**成 `mod.mp.Mp*`，
`use widgets::*` 换成 `use mp::*`，`widgets::sizing::MpSize` 换成 `ControlSize`，然后编译。

**结果：88 个编译错误。**（实验已回滚，工作区干净，dbpro 仍是 0 错误。）

错误不是命名空间的，是 API 形状的：
- `TreeItem` 这类**类型在 v3 里不存在**——`mp/tree.rs` 持有自己的条目类型，dbpro 的 `Vec<TreeItem>` 是 v2 的形状；
- `makepad_theme` **不是 dbpro 的依赖**，所以 `ControlSize` 拿不到（要先加依赖）；
- `mp/table.rs`/`mp/editor.rs` 的单元格与文本通过 Rust 侧的数据 API 驱动，而 dbpro 的 DSL 里直接写 `mod.widgets.MpTable{...}`
  的子节点。

**88 是一个 app 重写的规模，不是一次重命名。** 所以它是独立的一轮工作，不该顺手开始；记在这里，让下一轮从
测量过的地方开始，而不是从猜测开始。

# 对等审计用了错的标尺：bezel 的模块表 vs 本仓库的 v2 组件表（2026-09-17）

## 两个问题，两个不同的答案

我之前所有「对等完成」的结论都建立在**一把错的尺子**上：拿 bezel 的 **35 个模块名** 去比。问题是
**bezel 一个模块装多个组件**（`buttons.rs`/`content.rs`/`controls.rs` 是分组），所以那把尺子量不出组件。

现在用另一把尺子：**本仓库自己的 v2 组件表**（`crates/ui/src/widgets/*.rs`，一个文件一个组件，76 个）
对 **v3 的 DSL 组件名**（`mod.mp.Mp*` 定义，去掉 `Base`/尺寸变体后约 57 个）。

| | 结论 |
| --- | --- |
| **对 bezel 的对等** | **完成**：4 个真缺口（`menu`/`menubar`/`stats`/`titlebar`）本轮全部落地，其余名字逐个查证后确认不是缺口 |
| **本仓库 v2 → v3 的迁移** | **未完成**：76 个 v2 组件模块里 **26 个在 v3 没有对应组件** |

**两个结论都对，因为它们回答的不是同一个问题。** 目标是「类似 gpui-bezel 的组件库」，按那把尺子完成了；
但本仓库的 v2 半边比 bezel 大得多（很多组件是它自己的扩展，bezel 根本没有），所以「删掉 v2」这件事按
bezel 的尺子看永远看不出还差什么。

## v3 里没有的 26 个 v2 组件

```
accordion alert attachment breadcrumb bubble card collapsible dialog dropdown link
message modal notification option_card orb page_flip rating select sheet sidebar
split_pane status_bar stepper tab toggle toggle_group
```

（`focus`/`scaffolding`/`status`/`text`/`divider`/`separator` 不在此列：它们是支撑模块，v3 里都在。）

## 其中 4 个卡住 dbpro

`dbpro` 的 DSL 里用了 `MpDialog`（4 处）、`MpDropdown`（1）、`MpSelect`（2 + trigger）、`MpSplitPane`（1），
**这 4 个 v3 都没有** ✗ 所以 dbpro 的迁移不是「把前缀换掉 + 调和 API」，
而是**先要有这 4 个组件** —— 我先前把 88 个编译错误当成「API 形状差异」，其实其中一部分是「组件不存在」。

**下一轮的入口因此很具体**：先移植 `MpDialog`、`MpDropdown`、`MpSelect`、`MpSplitPane`（按 dbpro 的使用量，
`dialog` 优先，17 个调用点），再动 dbpro，最后才是阶段 6 的删除。

