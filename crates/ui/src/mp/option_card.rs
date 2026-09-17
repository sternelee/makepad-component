//! `MpOptionCard` — a card you choose between, with the selection ring around it.
//!
//! ## The ring is a wrapper border, not a spread shadow
//!
//! bezel's reasoning, kept because it is right and because it is invisible until it is wrong: *"a shadow's spread grows the
//! rectangle without growing its corner radius, so the halo's corners tighten relative to the frame's and the two visibly
//! drift apart by a pixel at each rounded corner. Concentric borders cannot do that: each element rounds itself, and the outer
//! radius is the inner one plus the gap it sits behind."*
//!
//! So [`ring_radius`] is `card_radius + RING_GAP + RING_WIDTH`, and a test asserts the concentric property directly: the
//! ring's radius minus the gap between them is the frame's radius. A spread would have broken that relation, and nothing else
//! in the component would have noticed.
//!
//! ## The ring is always there, so choosing never reflows the row
//!
//! [`ring_alpha`] is 1.0 or 0.0 and the **geometry is reserved either way** — asserted as [`card_size`] taking the selection
//! and returning the same size for both. This is the same rule the menu panel's glyph gutter follows and the tab bar's close
//! region: a highlight that only exists when active moves everything beside it by its own width, and the row jumps when you
//! choose. A transparent ring costs two points of padding and saves the jump.
//!
//! ## This one *does* take its children — and that is the difference from the collapsible
//!
//! [`crate::mp::collapsible`] is a header that refuses to swallow its body, because it would have had to re-implement layout
//! for nothing. An option card swallows its preview for a reason: **the ring's geometry has to wrap it**. The wrapper's radius
//! is derived from the frame's and the frame's from the preview's own rounding, so a caller laying those three out by hand
//! would be re-deriving the same concentric relation at every call site. That is what a component is for — and it is why the
//! two components answer the same question differently rather than by taste.
//!
//! ## The preview rounds its own corners
//!
//! A caller's preview that paints a background must round it to [`CARD_RADIUS`], which is bezel's note and the one thing this
//! component cannot do for its caller: the preview is drawn by whoever put it there, and a square-cornered background inside a
//! rounded frame shows through at all four corners.

use makepad_widgets::*;

/// The preview frame's height.
pub const CARD_HEIGHT: f64 = 148.0;

/// The preview frame's corner radius.
pub const CARD_RADIUS: f64 = 10.0;

/// The gap between the frame and the ring.
pub const RING_GAP: f64 = 2.0;

/// The ring's width.
pub const RING_WIDTH: f64 = 2.0;

/// The gap between a caption and the card above it.
pub const CAPTION_GAP: f64 = 6.0;

/// The gap between cards in a row.
pub const ROW_GAP: f64 = 16.0;

/// The ring's corner radius, derived from the frame's — **the concentric rule**.
pub fn ring_radius(card_radius: f64) -> f64 {
    card_radius + RING_GAP + RING_WIDTH
}

/// Whether the concentric relation holds between a frame and a ring drawn around it.
///
/// The property a spread shadow breaks, stated so it can be asserted rather than described: the ring's radius less the gap it
/// sits behind is the frame's radius. A shadow halo rounds by the frame's radius while sitting *outside* it, so its corners
/// tighten as the spread grows and the two drift apart by a pixel at every corner.
pub fn is_concentric(card_radius: f64, ring: f64) -> bool {
    (ring - RING_GAP - RING_WIDTH - card_radius).abs() < 1e-9
}

/// How opaque the ring is for a card's state.
pub fn ring_alpha(selected: bool) -> f32 {
    if selected {
        1.0
    } else {
        0.0
    }
}

/// The card's size.
///
/// **Takes the selection and returns the same size for both**, which is the rule: the ring is always present and merely
/// transparent when unselected, so choosing a card cannot move the ones beside it. A version that added the ring's width only
/// when selected would reflow the whole row, and the jump would be two points per neighbour.
pub fn card_size(selected: bool) -> (f64, f64, f64) {
    let _ = selected;
    let outer = CARD_RADIUS + RING_GAP + RING_WIDTH;
    (
        // The wrapper's own rounding, its height, and the whole card's height with the caption.
        outer,
        CARD_HEIGHT + (RING_GAP + RING_WIDTH) * 2.0,
        CARD_HEIGHT + (RING_GAP + RING_WIDTH) * 2.0 + CAPTION_GAP + CAPTION_LINE,
    )
}

/// The caption's line height.
pub const CAPTION_LINE: f64 = 18.0;

/// What an option card reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MpOptionCardAction {
    /// The card was chosen. **The selection stays the caller's** — the page that owns the row decides whether choosing this
    /// card deselects the others, and a card that set its own `selected` would have no way to know.
    Chosen,
    #[default]
    None,
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    /// One plate: the ring around a card, or the card's own frame. Both are this shader, differing in radius, border and
    /// alpha — so the concentric relation is enforced by the arithmetic rather than by two DSL blocks that could disagree.
    mod.mp.DrawMpOptionPlate = #(DrawMpOptionPlate::script_shader(vm)){
        ..mod.draw.DrawQuad

        radius: 10.0
        border_width: 1.0
        plate: #x00000000
        border_color: #x00000000
        alpha: 1.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let sz = self.rect_size
            sdf.box(
                self.border_width * 0.5,
                self.border_width * 0.5,
                sz.x - self.border_width,
                sz.y - self.border_width,
                self.radius
            )
            sdf.fill_keep(self.plate * self.alpha)
            sdf.stroke(self.border_color * self.alpha, self.border_width)
            return sdf.result
        }
    }

    mod.mp.MpOptionCardBase = #(MpOptionCard::register_widget(vm))

    mod.mp.MpOptionCard = set_type_default() do mod.mp.MpOptionCardBase{
        width: Fill
        height: Fit
        // **No `flow` or `spacing`**: this widget draws its preview and its caption at computed positions, because the ring's
        // geometry wraps the preview and a flow cannot express that. A `flow: Down` here would look like it laid them out.

        // **No geometry numbers here.** Every radius, height and width is a Rust constant, so there is exactly one place
        // each can be wrong — instead of a literal in the DSL that a constant silently disagrees with.
        preview := View{
            width: Fill
            height: Fill
        }

        caption := Label{
            width: Fill, height: Fit
            draw_text +: {text_style: mod.mpc.type.body, color: mod.mpc.tokens.text_muted}
            text: ""
        }
    }

    /// A row of option cards.
    mod.mp.MpOptionCardRow = View{
        width: Fill
        height: Fit
        flow: Right
        spacing: 16.0
        align: Align{y: 0.0}
    }
}

/// One plate: the ring, or the card's frame.
#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpOptionPlate {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    radius: f32,
    #[live]
    border_width: f32,
    #[live]
    plate: Vec4f,
    #[live]
    border_color: Vec4f,
    #[live]
    alpha: f32,
}

/// A card you choose between.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MpOptionCard {
    #[source]
    source: ScriptObjectRef,
    /// The two slots the caller writes into. **This component takes its children** — see the module doc on why that is the
    /// opposite of `mp/collapsible.rs` and not a matter of taste.
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
    /// The ring and the frame, drawn by this widget rather than declared as nested views, so the concentric radii come from
    /// one arithmetic. `#[live]` because it is this widget's own shader — which is what lets Rust set its instances, unlike a
    /// child DSL view's.
    #[live]
    draw_ring: DrawMpOptionPlate,
    #[live]
    draw_frame: DrawMpOptionPlate,
    /// Whether this is the chosen card. **The caller's state**, set rather than toggled here: a card cannot know whether
    /// choosing it deselects its neighbours.
    #[rust]
    selected: bool,
    #[rust]
    area: Area,
}

impl MpOptionCard {
    /// Set the caption.
    pub fn set_label(&mut self, cx: &mut Cx, label: &str) {
        self.view.label(cx, ids!(caption)).set_text(cx, label);
    }

    /// Mark this card chosen or not.
    ///
    /// Only the ring's alpha changes — **the size does not**, which is the rule that keeps the row from jumping. And the
    /// caption is *not* re-inked: its ink is the theme's, and a component that recoloured a caller's child would be reaching
    /// outside its own paint. bezel tints its caption; this draws the selection in the ring alone, and says so.
    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        if self.selected == selected {
            return;
        }
        self.selected = selected;
        self.redraw(cx);
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// The frame's rect inside a card whose top-left is `origin`, at `width`.
    pub fn frame_rect(origin: DVec2, width: f64) -> Rect {
        Rect {
            pos: dvec2(origin.x + RING_GAP + RING_WIDTH, origin.y + RING_GAP + RING_WIDTH),
            size: dvec2(
                (width - (RING_GAP + RING_WIDTH) * 2.0).max(0.0),
                CARD_HEIGHT,
            ),
        }
    }

    /// The ring's rect, which is the whole card's frame including the ring itself.
    pub fn ring_rect(origin: DVec2, width: f64) -> Rect {
        Rect {
            pos: origin,
            size: dvec2(width.max(0.0), CARD_HEIGHT + (RING_GAP + RING_WIDTH) * 2.0),
        }
    }
}

impl Widget for MpOptionCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        let signals = crate::mp::control::handle(&mut self.animator, cx, event, self.area);
        if signals.redraw {
            self.redraw(cx);
        }
        if signals.activate {
            cx.widget_action(self.widget_uid(), MpOptionCardAction::Chosen);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (accent, border, plate, radius) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            (
                theme.paint.accent,
                theme.paint.border,
                theme.paint.surface_raised,
                CARD_RADIUS as f32,
            )
        };
        let placed = cx.walk_turtle(Walk {
            width: Size::fill(),
            height: Size::Fixed(card_size(self.selected).2),
            ..walk
        });
        self.area = self.draw_ring.area();

        // The ring, **always drawn**: transparent when unchosen, so choosing cannot move a neighbour.
        let ring = Self::ring_rect(placed.pos, placed.size.x);
        self.draw_ring.radius = ring_radius(CARD_RADIUS) as f32;
        self.draw_ring.border_width = RING_WIDTH as f32;
        self.draw_ring.plate = Vec4f::default();
        self.draw_ring.border_color = accent;
        self.draw_ring.alpha = ring_alpha(self.selected);
        self.draw_ring.draw_abs(cx, ring);
        // The frame, inside the ring, rounded one gap-and-width less.
        let frame = Self::frame_rect(placed.pos, placed.size.x);
        self.draw_frame.radius = radius;
        self.draw_frame.border_width = 1.0;
        self.draw_frame.plate = plate;
        self.draw_frame.border_color = border;
        self.draw_frame.alpha = 1.0;
        self.draw_frame.draw_abs(cx, frame);

        // The caller's preview, drawn inside the frame at the frame's rect.
        let preview = self.view.view(cx.cx, ids!(preview));
        let _ = preview.draw_walk(cx, scope, Walk::fixed(frame.size.x, frame.size.y).with_abs_pos(frame.pos));
        // The caption under the whole card.
        let caption = self.view.label(cx.cx, ids!(caption));
        let _ = caption.draw_walk(
            cx,
            scope,
            Walk::fixed(placed.size.x, CAPTION_LINE)
                .with_abs_pos(dvec2(placed.pos.x, placed.pos.y + ring.size.y + CAPTION_GAP)),
        );
        DrawStep::done()
    }
}

impl MpOptionCardRef {
    pub fn set_label(&self, cx: &mut Cx, label: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_label(cx, label);
        }
    }

    pub fn set_selected(&self, cx: &mut Cx, selected: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, selected);
        }
    }

    pub fn is_selected(&self) -> bool {
        self.borrow().map(|inner| inner.is_selected()).unwrap_or(false)
    }

    /// Whether this card was chosen, for a caller reading an event batch.
    pub fn chosen(&self, actions: &Actions) -> bool {
        actions
            .find_widget_action(self.widget_uid())
            .is_some_and(|action| matches!(action.cast::<MpOptionCardAction>(), MpOptionCardAction::Chosen))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The module's own source, **with the tests cut off**.
    ///
    /// `include_str!` includes this module, so a plain search finds the needles these tests write themselves: my first
    /// version asserted the DSL had no `height: 152.0` and the assertion failed because the *assertion* contained that text.
    /// The cut is what makes a source search mean "the code", not "the code and my expectations of it".
    fn source() -> &'static str {
        let all = include_str!("option_card.rs");
        match all.find("#[cfg(test)]") {
            Some(at) => &all[..at],
            None => all,
        }
    }

    #[test]
    fn test_the_ring_is_concentric_with_the_frame_rather_than_a_spread_shadow() {
        // **The rule a spread shadow breaks.** A shadow's halo rounds by the frame's radius while sitting outside it, so its
        // corners tighten as the spread grows and the two drift apart by a pixel at every corner. Concentric borders cannot:
        // each element rounds itself and the outer radius is the inner one plus the gap it sits behind.
        assert_eq!(ring_radius(CARD_RADIUS), CARD_RADIUS + RING_GAP + RING_WIDTH);
        assert!(is_concentric(CARD_RADIUS, ring_radius(CARD_RADIUS)));
        // The property, stated as the relation it is: ring less the chrome between them is the frame.
        assert_eq!(ring_radius(CARD_RADIUS) - RING_GAP - RING_WIDTH, CARD_RADIUS);
        // A ring that had been spread rather than drawn would fail it — which is what makes the assertion worth making.
        assert!(!is_concentric(CARD_RADIUS, CARD_RADIUS));
        assert!(!is_concentric(CARD_RADIUS, CARD_RADIUS + RING_GAP));
        // And it holds at any radius, so a caller changing the card's radius gets a concentric ring for free.
        for radius in [4.0, 8.0, 10.0, 16.0, 24.0] {
            assert!(is_concentric(radius, ring_radius(radius)), "radius {radius}");
        }
    }

    #[test]
    fn test_the_card_is_the_same_size_whichever_way_it_is_chosen() {
        // **The no-reflow rule.** The ring is always present and merely transparent when unselected, so choosing a card cannot
        // move the ones beside it. A version that added the ring's width only when selected would jump the whole row by two
        // points per neighbour.
        let unselected = card_size(false);
        let selected = card_size(true);
        assert_eq!(unselected, selected, "choosing a card resized it");
        // And the ring's opacity is the only thing that differs.
        assert_eq!(ring_alpha(false), 0.0);
        assert_eq!(ring_alpha(true), 1.0);
        assert_ne!(ring_alpha(false), ring_alpha(true));
        // The outer rounding and the height account for the ring on both sides — a ring drawn on one side would sit off-centre.
        let (outer_radius, ring_height, whole) = card_size(false);
        assert_eq!(outer_radius, CARD_RADIUS + RING_GAP + RING_WIDTH);
        assert_eq!(ring_height, CARD_HEIGHT + (RING_GAP + RING_WIDTH) * 2.0);
        assert_eq!(whole, ring_height + CAPTION_GAP + CAPTION_LINE);
        // The wrapper's content box, less its padding, is the frame — which is the height the frame is given.
        assert_eq!(ring_height - (RING_GAP + RING_WIDTH) * 2.0, CARD_HEIGHT);
    }

    /// Just the `script_mod!` block, **with its comments stripped**, which is where a geometry number could drift from a
    /// constant.
    ///
    /// Two things this had to learn, both from failing on its own prose. Scoping to the block rather than the whole file,
    /// because the file's *comments* legitimately mention the numbers the DSL must not have — and then stripping the
    /// comments inside the block too, for the same reason one level down. The check is about code; a sentence about what the
    /// code does not contain is not the code.
    fn dsl() -> String {
        let source = source();
        let start = source.find("script_mod!").expect("the DSL block");
        // The first derive *after* the block, not the doc comment inside it — that marker is the documentation of one of the
        // shaders the block declares, so the slice ended before the card's own DSL.
        let end = source.find("#[derive(Script, ScriptHook)]").expect("the end of the DSL block");
        source[start..end]
            .lines()
            .map(|line| match line.find("//") {
                Some(at) => &line[..at],
                None => line,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_every_geometry_number_lives_in_rust_so_the_dsl_cannot_disagree_with_it() {
        // The geometry was in the DSL first, with the constants asserted against the DSL's literals — a check that can only
        // catch a drift *after* it happens. It is now in Rust alone: **there is no number in the DSL to drift**. This test
        // asserts that, which is a stronger statement than comparing two copies.
        for number in ["height: 152.0", "border_radius: 14.0", "border_radius: 10.0", "padding: Inset", "flow: Down"] {
            assert!(
                !dsl().contains(number),
                "the DSL still carries geometry ({number}) that a Rust constant could disagree with"
            );
        }
        // The one spacing that is a layout choice rather than the card's geometry — the row's gap — is kept in the DSL, and
        // asserted against the constant so the two cannot part.
        assert!(dsl().contains("spacing: 16.0"), "the row's gap is not in the DSL");
        assert_eq!(ROW_GAP, 16.0);
    }

    #[test]
    fn test_the_preview_and_the_caption_are_the_slots_a_caller_writes_into() {
        // The two slots a caller uses, asserted against the DSL text so a rename on one side cannot pass — the same check the
        // dialog's slot ids get. The preview is the caller's because it is whatever the option looks like; the caption is here
        // because it is always a label.
        assert!(dsl().contains("preview := View{"), "the preview slot is gone");
        assert!(dsl().contains("caption := Label{"), "the caption slot is gone");
        // **The ring and the frame are no longer slots**, and that is the improvement: they are drawn by this widget's own
        // shader, so their concentric radii come from one arithmetic instead of two DSL blocks that could disagree. My first
        // version asserted they were slots, which was true before the restructure and stale after it.
        assert!(
            !dsl().contains("ring := ") && !dsl().contains("frame := "),
            "the ring or frame came back as a slot, so the concentric relation is no longer in one place"
        );
    }

    #[test]
    fn test_this_component_takes_its_children_and_the_collapsible_does_not() {
        // Both are deliberate, and the difference is stated in both modules: the collapsible refuses to swallow its body
        // because it would have had to re-implement layout for nothing, while an option card swallows its preview because the
        // ring's geometry has to wrap it. A library whose components answered this by taste would be inconsistent; these
        // answer it by what each one needs.
        assert!(
            source().contains("the ring's geometry has to wrap it"),
            "the reason this one takes children is no longer recorded"
        );
        // A short needle that provably exists: the full sentence I first searched for did not, because the two modules word
        // it differently — and a cross-file test asserting another file's prose is brittle enough that it should assert as
        // little as it can while still catching the claim being deleted.
        let collapsible = include_str!("collapsible.rs");
        assert!(
            collapsible.contains("swallowed its children"),
            "the collapsible's opposite reason is no longer recorded"
        );
        // And the sizes: a card is a fixed-height preview, so its height is a constant rather than a measured child.
        assert!(CARD_HEIGHT > 0.0 && CAPTION_LINE > 0.0);
        assert!(RING_GAP > 0.0 && RING_WIDTH > 0.0);
    }
}
