## Goal
Iteratively improve the UI/UX of the canvas-terminal Makepad app. Each iteration should make a small, concrete, visually-verifiable improvement and verify with a screenshot.

## Baseline (current state)
A canvas workspace (CNVS-style) with:
- Starfield/sky gradient background (looks good)
- Left icon toolbar (cursor-arrow selected, pen-squiggle, line, square, circle, "A"+I-beam, diamond, color swatches, line-width samples)
- Bottom command capsule (`⌘ @name text · /new terminal NAME ...`) with `+` button on left and mic on right
- Bottom status label ("Ready — drag items, scroll to pan...")
- Terminal cards ("claude — zsh" with avatar, status dot, minimize/close buttons)

## Constraints
- Preserve functionality; all tests must pass (cargo +stable test -p canvas-terminal, 11 tests)
- Only edit files under crates/canvas-terminal/
- Use clean line-icon style for toolbar icons; keep the hand-drawn wobble for actual canvas strokes
- Follow existing constants/types (TERM_CELL_W, PAL_ACCENT, NoteTool, etc.)
- Verify each iteration with cargo +stable build, cargo +stable test, cargo +stable fmt, and a screenshot
- Use the CEF-safe build: PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH" cargo +stable build -p canvas-terminal

## Candidates across iterations (pick impactful ones)
1. Empty-canvas onboarding: a subtle placeholder hint in the middle of the empty canvas (e.g. "⌘ click ＋ or type /new to add items") that fades once items exist.
2. Terminal card empty-space: prompt sits at top with huge blank grid below — ensure the cursor/prompt renders on the first visible row and consider a subtle content hint when the grid is empty.
3. Command bar polish: placeholder text color/contrast, focus ring, suggestion list styling.
4. Status bar: more useful/legible text, subtle accent.
5. Toolbar: active-tool affordance (accent border), tool separation/grouping with dividers, icon size/spacing.
6. Card hover affordances, smooth selection glow, shadows.
7. Color swatch/width presets contrast & active state.
8. Consistent spacing/typography scale across cards, command bar, status.
9. Anything else that measurably improves perceived polish.

## Per-iteration checklist
1. Choose one focused improvement.
2. Read the relevant draw/hit-test code.
3. Implement the change minimally.
4. cargo +stable build, cargo +stable test, cargo +stable fmt --check.
5. Launch app, screenshot, crop the relevant region, visually confirm.
6. Note what changed and whether it looks better; revert if it regresses.