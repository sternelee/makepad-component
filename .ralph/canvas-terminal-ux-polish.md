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
9. Inactive terminal focus affordance: non-selected terminal cards dim their
   live grid and hide the cursor, so it's clear which card receives input.
10. Right-align the terminal status indicator: the agent status dot + label
    is anchored from the right edge with clearance for the min/close buttons
    instead of a fixed offset, so it never overlaps them.
11. Softer card drop shadow: 6 lower-alpha layers that extend further and
    fade smoothly, replacing the 4-layer harsher one.
12. Accent the active workspace tab: amber underline along its bottom edge
    + accent-colored label, so the current space is unmistakable.
13. Round the music player play/pause button (filled disc + sketchy ring),
    matching the round avatar chip; progress bar offset from the circle edge.
14. Card hover backlight: draw a soft glow around a hovered (non-selected)
    card using the already-tracked `hovered` state, so the mouse target is
    clear without competing with the selection glow.
15. Browser card chrome/page divider: a subtle line between the browser
    chrome and the page area, matching the terminal-card divider.
16. Show active tool in status bar: add `NoteTool::label()` and update the
    bottom status text on tool change (e.g. "Tool: Rectangle").
17. Fix the card close button glyph: it was two parallel bars (reads as "=");
    now a proper × (two diagonal strokes) in soft red.
18. Round the music progress bar fill (a disc at the fill's leading edge),
    matching the circular play button and round avatar chip.
19. Dim inactive terminal title/status (×0.7), consistent with the dimmed
    grid and hidden cursor on non-focused cards.
20. Clarify color-swatch active state: thicker accent ring + soft glow on
    the active swatch, more readable neutral rings on the rest.
21. Fix empty-hint to also hide when items are docked/minimized (it only
    checked on-canvas items + shapes).

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

## Reflection (iteration 13)

19 committed improvements are cross-verified (test 11/11 + fmt + code review).
The focused-change pattern is effective and every change reuses existing
helpers/constants, keeping the CNVS hand-drawn aesthetic cohesive.

Screenshot capture is still blocked, so candidates are limited to logically-
verifiable, low-risk changes. At iteration 13 there are still genuine such
improvements (e.g. the note card editing caret, color-swatch active state), so
I'll continue with tightly-scoped changes. If I reach a point where only
subjective styling remains (which I can't visually confirm), I'll note it and
pause rather than churn blind styling.
