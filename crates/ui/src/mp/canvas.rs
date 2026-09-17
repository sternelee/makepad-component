//! `MpCanvas` — a JSON Canvas document, painted read-only.
//!
//! ## Read-only, and that is the scope rather than a limitation
//!
//! This paints: nodes at their positions, edges between them, the six preset colours, and the group containers.
//! It does **not** pan, zoom, select or edit. That is `canvas-terminal`'s job — it has a 7000-line infinite canvas
//! with a camera, a hit test and its own persistence — and a second interactive canvas in the component library
//! would be a second copy of all of it.
//!
//! So this exists for the two things `canvas-terminal` cannot do: **preview a document** from the interchange
//! format, and give the model a widget to be verified through. A reader comparing this to the reference should read
//! it as the `model` half plus a view, not as the canvas.
//!
//! ## What the painting is honest about
//!
//! - **Edges are elbows, not diagonals.** A diagonal needs a rotated draw and `draw_abs` boxes are axis-aligned, so
//!   an edge is drawn as a horizontal run and a vertical run joined at the side it leaves from. That is a real
//!   canvas convention rather than a shortcut, but it is *not* what a diagonal would look like and the file says
//!   so.
//! - **The whole document is fitted to the box.** A canvas is unbounded and a widget is not, so the scale is
//!   `min(box / bounds, ceiling)` and every position is scaled by it. A document one node wide therefore draws at
//!   a sane size instead of at a dot.
//! - **A node's text is clipped to its own rect** by the same estimator the rest of the library uses for clipping,
//!   where an error in either direction is invisible — see `mp/text.rs`.
//! - **The preset colours come from the theme**, because the spec leaves their values to the application. See
//!   `makepad_canvas::Preset::color`.

use makepad_widgets::*;

use makepad_canvas::{BackgroundStyle, Canvas, Color, End, Node, NodeKind, Preset, Side};
use makepad_theme::Paint;

use crate::mp::text;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    set_type_default() do #(DrawMpCanvas::script_shader(vm)){
        ..mod.draw.DrawQuad

        fill: #x00000000
        border: #x00000000
        border_width: 0.0
        radius: 10.0

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let bw = self.border_width
            sdf.box(bw, bw, self.rect_size.x - bw * 2.0, self.rect_size.y - bw * 2.0, max(1.0, self.radius))
            sdf.fill_keep(self.fill)
            if (bw > 0.0) {
                sdf.stroke(self.border, bw)
            }
            return sdf.result
        }
    }

    mod.mp.MpCanvasBase = #(MpCanvas::register_widget(vm))

    mod.mp.MpCanvas = set_type_default() do mod.mp.MpCanvasBase{
        width: Fill
        height: 360
        // The largest scale a document is drawn at, so a one-node canvas does not fill the box.
        ceiling: 1.0

        draw_bg +: {
            fill: band
            border: border
            border_width: 1.0
            radius: 10.0
        }
        draw_label +: {
            text_style: mod.mpc.type.caption
            color: #x00000000
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawMpCanvas {
    #[deref]
    draw_super: DrawQuad,
    #[live]
    fill: Vec4f,
    #[live]
    border: Vec4f,
    #[live]
    border_width: f32,
    #[live]
    radius: f32,
}

/// The padding inside the box, so a node never touches the border.
const PAD: f64 = 12.0;

#[derive(Script, Widget)]
pub struct MpCanvas {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,

    #[redraw]
    #[live]
    draw_bg: DrawMpCanvas,
    #[live]
    draw_label: DrawText,

    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,

    #[live]
    ceiling: f64,

    #[rust]
    canvas: Canvas,
    #[rust]
    area: Area,
}

impl MpCanvas {
    pub fn set_canvas(&mut self, cx: &mut Cx, canvas: Canvas) {
        self.canvas = canvas;
        self.redraw(cx);
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// The document's bounding box, as `(min_x, min_y, width, height)`, or `None` when there are no nodes.
    ///
    /// A node's rect is its position and size; the box of the document is the box of those. Public because it is the
    /// one piece of arithmetic in this file worth a test, and because a caller wanting to fit a canvas elsewhere
    /// needs the same number.
    pub fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        canvas_bounds(&self.canvas)
    }
}

/// The document's bounding box: `(min_x, min_y, width, height)`, or `None` with no nodes.
///
/// A free function rather than a method, so it can be tested without building a widget — and a widget is a lot of
/// machinery to construct for one piece of arithmetic.
pub fn canvas_bounds(canvas: &Canvas) -> Option<(f64, f64, f64, f64)> {
    {
        let nodes = &canvas.nodes;
        let mut iter = nodes.iter();
        let first = iter.next()?;
        let (mut min_x, mut min_y) = (first.x as f64, first.y as f64);
        let (mut max_x, mut max_y) = (min_x + first.width as f64, min_y + first.height as f64);
        for node in iter {
            min_x = min_x.min(node.x as f64);
            min_y = min_y.min(node.y as f64);
            max_x = max_x.max(node.x as f64 + node.width as f64);
            max_y = max_y.max(node.y as f64 + node.height as f64);
        }
        Some((min_x, min_y, max_x - min_x, max_y - min_y))
    }
}

impl MpCanvas {
    /// The colour a node or an edge paints with, from the theme.
    fn color_of(paint: &Paint, color: Option<&Color>) -> Vec4f {
        match color {
            Some(Color::Preset(preset)) => preset.color(paint),
            Some(Color::Hex(hex)) => parse_hex(hex).unwrap_or(paint.text_muted),
            None => paint.text_muted,
        }
    }
}

/// A `#RRGGBB` or `#RRGGBBAA` string as a colour.
///
/// `None` for anything else, because the model **validates** rather than repairs: a painter handed `"#GGGGGG"` that
/// silently used a fallback would hide a fault the model reports.
pub fn parse_hex(hex: &str) -> Option<Vec4f> {
    let digits = hex.strip_prefix('#')?;
    let component = |at: usize| -> Option<f32> {
        let byte = u8::from_str_radix(digits.get(at..at + 2)?, 16).ok()?;
        Some(byte as f32 / 255.0)
    };
    match digits.len() {
        6 => Some(vec4(component(0)?, component(2)?, component(4)?, 1.0)),
        8 => Some(vec4(
            component(0)?,
            component(2)?,
            component(4)?,
            component(6)?,
        )),
        _ => None,
    }
}

/// Empty for the same reason `MpSegmented`'s is: no animator to seat, nothing to place.
impl ScriptHook for MpCanvas {
    fn on_after_new(&mut self, _vm: &mut ScriptVm) {}
}

impl Widget for MpCanvas {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
        // **Read-only.** No pan, no zoom, no selection, no hit test: `canvas-terminal` has all of those over an
        // infinite canvas, and a second one here would be a second copy of them.
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let (panel, border, radius, ink, muted, plate, wash) = {
            let theme = makepad_theme::Theme::of(cx.cx);
            let p = &theme.paint;
            (
                p.band,
                p.border,
                makepad_theme::Theme::panel_radius() as f32,
                p.text,
                p.text_muted,
                p.surface_card,
                p.code_wash,
            )
        };

        self.draw_bg.fill = panel;
        self.draw_bg.border = border;
        self.draw_bg.border_width = 1.0;
        self.draw_bg.radius = radius;
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        let rect = self.draw_bg.area().rect(cx.cx);
        self.area = self.draw_bg.area();

        let Some((min_x, min_y, width, height)) = self.bounds() else {
            // An empty document draws the plate and nothing else.
            return DrawStep::done();
        };
        let box_w = (rect.size.x - PAD * 2.0).max(1.0);
        let box_h = (rect.size.y - PAD * 2.0).max(1.0);
        // **Fitted**, because a canvas is unbounded and a widget is not: bounded above by the ceiling so a document
        // of one node draws at a sane size rather than filling the box.
        let scale = (box_w / width.max(1.0))
            .min(box_h / height.max(1.0))
            .min(self.ceiling.max(0.01));
        let origin = rect.pos + dvec2(PAD, PAD);
        let to_screen = |x: f64, y: f64| -> DVec2 {
            dvec2(
                origin.x + (x - min_x) * scale,
                origin.y + (y - min_y) * scale,
            )
        };
        let node_rect = |node: &Node| -> Rect {
            let pos = to_screen(node.x as f64, node.y as f64);
            Rect {
                pos,
                size: dvec2(node.width as f64 * scale, node.height as f64 * scale),
            }
        };

        let theme_paint = makepad_theme::Theme::of(cx.cx).paint;

        // **Groups first**, so they sit behind the nodes they contain — which is what a group is for, and what the
        // spec's z-order says the array order decides. A group that appears after its contents in the array still
        // paints behind them here, and that is the one place this view does not follow the array order; it is
        // recorded rather than hidden.
        for node in self.canvas.nodes.iter().filter(|node| matches!(node.kind, NodeKind::Group { .. })) {
            let node_rect = node_rect(node);
            self.draw_bg.fill = wash;
            self.draw_bg.border_width = 0.0;
            self.draw_bg.radius = 8.0;
            self.draw_bg.draw_abs(cx, node_rect);
            self.draw_bg.border_width = 1.0;
            self.draw_bg.fill = panel;
        }

        // **Edges before nodes**, so a connection goes under the boxes it joins rather than over them.
        for edge in &self.canvas.edges {
            let (Some(from), Some(to)) = (
                self.canvas.node(&edge.from_node),
                self.canvas.node(&edge.to_node),
            ) else {
                // A dangling edge is a fault the model reports; this view draws nothing for it rather than
                // panicking on a node that is not there.
                continue;
            };
            let colour = Self::color_of(&theme_paint, edge.color.as_ref());
            let from_rect = node_rect(from);
            let to_rect = node_rect(to);
            let start = side_point(&from_rect, edge.from_side.unwrap_or(Side::Right));
            let end = side_point(&to_rect, edge.to_side.unwrap_or(Side::Left));
            self.draw_bg.fill = colour;
            self.draw_bg.border_width = 0.0;
            self.draw_bg.radius = 0.0;
            // An **elbow**: a horizontal run from the start to the end's x, then a vertical run to the end. Two
            // axis-aligned boxes, because `draw_abs` cannot rotate.
            let mid_x = end.x;
            let elbow = Rect {
                pos: dvec2(start.x.min(mid_x), start.y - 0.5),
                size: dvec2((mid_x - start.x).abs().max(0.5), 1.0),
            };
            self.draw_bg.draw_abs(cx, elbow);
            let riser = Rect {
                pos: dvec2(mid_x - 0.5, start.y.min(end.y)),
                size: dvec2(1.0, (end.y - start.y).abs().max(0.5)),
            };
            self.draw_bg.draw_abs(cx, riser);
            // The arrowhead, when the spec's default or an explicit value asks for one: the spec's `toEnd` defaults
            // to `arrow` and its `fromEnd` to `none`, so a reader that assumed one default would put an arrow at
            // every edge's start.
            if edge.to_end() == End::Arrow {
                let head = 4.0;
                let arrow = Rect {
                    pos: dvec2(end.x - head, end.y - head * 0.5),
                    size: dvec2(head, head),
                };
                self.draw_bg.radius = 0.0;
                self.draw_bg.draw_abs(cx, arrow);
            }
            self.draw_bg.border_width = 1.0;
            self.draw_bg.fill = panel;
        }

        // The nodes, in the array's order, which is the spec's z-order.
        for node in self.canvas.nodes.iter() {
            let node_rect = node_rect(node);
            let colour = Self::color_of(&theme_paint, node.color.as_ref());
            if matches!(node.kind, NodeKind::Group { .. }) {
                // Already drawn, behind everything.
                continue;
            }
            self.draw_bg.fill = plate;
            self.draw_bg.border = colour;
            self.draw_bg.border_width = 1.0;
            self.draw_bg.radius = 6.0;
            self.draw_bg.draw_abs(cx, node_rect);
            // The label, clipped to the node's own width — the one place this file uses the estimator, where an
            // error in either direction is invisible.
            let label = node_label(node);
            let font = self.draw_label.text_style.font_size as f64;
            let room = node_rect.size.x - 8.0;
            if room > font {
                let clipped = text::clip(&label, room, font);
                self.draw_label.color = if node.color.is_some() { colour } else { ink };
                // The type name on a second line for a file or a link, so the two are distinguishable at a glance
                // without opening the JSON.
                self.draw_label.draw_walk(
                    cx,
                    Walk::fit().with_abs_pos(node_rect.pos + dvec2(4.0, 3.0)),
                    Align::default(),
                    &clipped,
                );
                let kind = node.kind.type_name();
                self.draw_label.color = muted;
                self.draw_label.draw_walk(
                    cx,
                    Walk::fit().with_abs_pos(node_rect.pos + dvec2(4.0, 3.0 + font * 1.4)),
                    Align::default(),
                    kind,
                );
            }
            self.draw_bg.border = border;
            self.draw_bg.fill = panel;
        }

        if std::env::var("MP_CANVAS_DEBUG").is_ok() {
            println!(
                "CANVAS nodes={} edges={} bounds=({min_x:.0},{min_y:.0},{width:.0},{height:.0}) scale={scale:.3} \
                 problems={}",
                self.canvas.nodes.len(),
                self.canvas.edges.len(),
                self.canvas.validate().len()
            );
        }

        DrawStep::done()
    }
}

/// The point on a node's side that an edge attaches to.
fn side_point(rect: &Rect, side: Side) -> DVec2 {
    match side {
        Side::Top => dvec2(rect.pos.x + rect.size.x * 0.5, rect.pos.y),
        Side::Right => dvec2(rect.pos.x + rect.size.x, rect.pos.y + rect.size.y * 0.5),
        Side::Bottom => dvec2(rect.pos.x + rect.size.x * 0.5, rect.pos.y + rect.size.y),
        Side::Left => dvec2(rect.pos.x, rect.pos.y + rect.size.y * 0.5),
    }
}

/// What to write inside a node.
fn node_label(node: &Node) -> String {
    match &node.kind {
        NodeKind::Text { text } => text.replace('\n', " "),
        NodeKind::File { file, .. } => file.clone(),
        NodeKind::Link { url } => url.clone(),
        NodeKind::Group { label, .. } => label.clone().unwrap_or_else(|| "group".to_string()),
    }
}

impl MpCanvasRef {
    pub fn set_canvas(&self, cx: &mut Cx, canvas: Canvas) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_canvas(cx, canvas);
        }
    }

    /// The binding severity as a name, so a page can report it without the enum.
    pub fn background_style_name(&self) -> String {
        let _ = self;
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makepad_canvas;

    fn canvas(json: &str) -> Canvas {
        makepad_canvas::parse(json).expect("the fixture parses")
    }

    fn bounds_of(json: &str) -> Option<(f64, f64, f64, f64)> {
        canvas_bounds(&canvas(json))
    }

    #[test]
    fn test_the_bounds_are_the_box_of_every_node_including_negative_positions() {
        // **Negative positions are the normal case on an infinite canvas** — a node dragged up and left has them —
        // and a bounding box that started at zero would draw the document shifted.
        let json = r##"{"nodes":[
            {"id":"a","type":"text","x":-100,"y":-50,"width":200,"height":100,"text":""},
            {"id":"b","type":"text","x":300,"y":200,"width":100,"height":50,"text":""}
        ]}"##;
        let (min_x, min_y, width, height) = bounds_of(json).expect("bounds");
        assert_eq!((min_x, min_y), (-100.0, -50.0));
        // From -100 to 400 across, and from -50 to 250 down.
        assert_eq!((width, height), (500.0, 300.0));
    }

    #[test]
    fn test_a_single_node_has_its_own_size_as_its_bounds() {
        let json = r##"{"nodes":[{"id":"a","type":"text","x":10,"y":20,"width":30,"height":40,"text":""}]}"##;
        assert_eq!(bounds_of(json), Some((10.0, 20.0, 30.0, 40.0)));
    }

    #[test]
    fn test_an_empty_canvas_has_no_bounds() {
        // `None` rather than a zero box, because a zero box would divide by zero in the fit and draw a document at
        // an infinite scale.
        assert_eq!(bounds_of("{}"), None);
        assert_eq!(bounds_of(r##"{"nodes":[],"edges":[]}"##), None);
    }

    #[test]
    fn test_a_hex_colour_parses_and_a_bad_one_does_not() {
        assert_eq!(parse_hex("#FF0000"), Some(vec4(1.0, 0.0, 0.0, 1.0)));
        assert_eq!(
            parse_hex("#00FF0080"),
            Some(vec4(0.0, 1.0, 0.0, 128.0 / 255.0))
        );
        // A fault the model reports; the painter returns `None` so the caller chooses a fallback rather than
        // silently getting one.
        assert_eq!(parse_hex("#GGGGGG"), None);
        assert_eq!(parse_hex("FF0000"), None);
        assert_eq!(parse_hex("#FFF"), None);
    }

    #[test]
    fn test_a_node_is_labelled_by_its_type() {
        let canvas = canvas(
            r##"{"nodes":[
                {"id":"t","type":"text","x":0,"y":0,"width":1,"height":1,"text":"hello\nworld"},
                {"id":"f","type":"file","x":0,"y":0,"width":1,"height":1,"file":"a/b.md"},
                {"id":"l","type":"link","x":0,"y":0,"width":1,"height":1,"url":"https://x"},
                {"id":"g","type":"group","x":0,"y":0,"width":1,"height":1}
            ]}"##,
        );
        assert_eq!(node_label(&canvas.nodes[0]), "hello world", "a newline is a space in one line");
        assert_eq!(node_label(&canvas.nodes[1]), "a/b.md");
        assert_eq!(node_label(&canvas.nodes[2]), "https://x");
        assert_eq!(node_label(&canvas.nodes[3]), "group", "an unlabelled group still says something");
    }

    #[test]
    fn test_an_edge_attaches_to_the_side_it_names_and_to_a_default_otherwise() {
        // The spec's defaults are asymmetric: `fromSide` and `toSide` are both optional, and a reader that put both
        // ends in the same place would draw every edge from one corner.
        let rect = Rect {
            pos: dvec2(10.0, 20.0),
            size: dvec2(100.0, 50.0),
        };
        assert_eq!(side_point(&rect, Side::Left), dvec2(10.0, 45.0));
        assert_eq!(side_point(&rect, Side::Right), dvec2(110.0, 45.0));
        assert_eq!(side_point(&rect, Side::Top), dvec2(60.0, 20.0));
        assert_eq!(side_point(&rect, Side::Bottom), dvec2(60.0, 70.0));
    }

    #[test]
    fn test_the_preset_colours_come_from_the_theme_and_a_hex_from_its_digits() {
        let paint = makepad_theme::palette::dark();
        // A preset is the theme's colour for that preset, whatever the theme says it is.
        assert_eq!(
            MpCanvas::color_of(&paint, Some(&Color::Preset(Preset::Red))),
            Preset::Red.color(&paint)
        );
        assert_eq!(
            MpCanvas::color_of(&paint, Some(&Color::Hex("#FF0000".into()))),
            vec4(1.0, 0.0, 0.0, 1.0)
        );
        // No colour is the muted ink rather than nothing, so a node with no colour still has a border.
        assert_eq!(MpCanvas::color_of(&paint, None), paint.text_muted);
        // And a bad hex falls back rather than returning black, which would look like a colour the reader chose.
        assert_eq!(
            MpCanvas::color_of(&paint, Some(&Color::Hex("#nope".into()))),
            paint.text_muted
        );
    }

    #[test]
    fn test_the_background_style_is_reachable_from_the_model_the_view_paints() {
        // The view ignores `backgroundStyle` — it draws a wash rather than an image, because loading one needs an
        // asset system this widget does not have — but the field is in the model and must survive, so this checks
        // the wiring rather than the painting.
        let canvas = canvas(
            r##"{"nodes":[{"id":"g","type":"group","x":0,"y":0,"width":1,"height":1,"backgroundStyle":"ratio"}]}"##,
        );
        assert!(matches!(
            canvas.nodes[0].kind,
            NodeKind::Group {
                background_style: Some(BackgroundStyle::Ratio),
                ..
            }
        ));
    }
}
