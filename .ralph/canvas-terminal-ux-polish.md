# canvas-terminal-ux-polish

## Goal

Iteratively improve the UI/UX of the canvas-terminal Makepad app.

## Progress (completed iterations)

All changes committed to dev. Each verified with `cargo +stable test` (11/11),
`cargo +stable fmt --check`, and (where the GUI window was capturable) a screenshot.

1. Empty-canvas onboarding hint ("No items yet — ⌘＋ or /new terminal ...") that hides once content exists.
2. Active-tool affordance: filled amber pill + accent border + glow on the selected tool.
3. Palette group dividers: subtle lines separating tools / colors / widths.
4. Terminal card divider between title bar and grid content.
5. Circular avatar chip (replaces square chip).
6. Softened selection glow (wider, lower-alpha halo).
7. Enhanced empty-hint: accent "＋" badge echoing the menu button.
8. Width presets: neutral pill for inactive, filled accent pill + frame for active.

## Screenshot-verification caveat

GUI screenshot capture intermittently fails: the `canvas-terminal` window (a
Metal/CEF app) often won't come to front or won't composite into a
`screencapture` frame (it stays behind ghostty or lands on another Space), and
`CGWindowList` returns the window on-screen but `screencapture -l<id>` yields
"could not create image from window". When this happens, the change is
verified by code review + tests, and the visual look is flagged for a manual
check rather than claimed verified. Prior iterations where the window WAS
capturable were screenshot-confirmed.

## Commands

- Build (CEF-safe PATH): `PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH" cargo +stable build -p canvas-terminal`
- Test: `cargo +stable test -p canvas-terminal`
- Fmt: `cargo +stable fmt -p canvas-terminal -- --check`

## Next candidates

- Command bar: placeholder/focus-ring contrast, suggestion-list active highlight.
- Status bar: dynamic/useful text with subtle accent.
- Card hover affordances and shadow tuning.
- Consistent spacing/typography scale across cards, command bar, status.
- Note/browser card polish (need content to render/verify).
