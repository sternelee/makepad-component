//! `makepad-canvas` — a **JSON Canvas** model: parse, serialize, validate.
//!
//! ## What this is, and what it is not
//!
//! [JSON Canvas](https://jsoncanvas.org) is an open format for infinite canvases — nodes at positions, edges
//! between them, and the six preset colours. It is a **published spec with a version and a date**
//! (1.0, 2024-03-11), which is why this crate exists rather than a bespoke model: `canvas-terminal` in this
//! workspace already has a 7000-line infinite canvas with its **own** save format, and the useful contribution is
//! not a second canvas but **interchange** — a canvas document that another tool can read.
//!
//! What is here is the **model**, and it is pure: no window, no painting, no gpui. The reference's `canvas` crate
//! splits the same way and says why — *"`model`, `mindmap`, `layout`, `change`, `clip` and `drag` are pure — no
//! gpui. What a node is comes from `kind`; where it sits from `layout`."* This is the `model` half, and a view or a
//! mindmap is a later layer over it.
//!
//! ## The spec's details, taken from the spec
//!
//! Read from `spec/1.0.md` rather than from memory, which is worth doing because three of them are the kind a
//! confident recollection gets wrong:
//!
//! - **`fromEnd` defaults to `none` and `toEnd` defaults to `arrow`.** The two ends of an edge do **not** share a
//!   default: an arrow at the start of a connection is unusual and the format says so.
//! - **The six presets are red, orange, yellow, green, cyan, purple** — `"1"` through `"6"`. Not the rainbow order
//!   you would guess, and not the ANSI order either.
//! - **The preset values are deliberately undefined.** The spec says so in as many words: *"Specific values for the
//!   preset colors are intentionally not defined so that applications can tailor the presets to their specific
//!   brand colors or color scheme."* So [`Preset::color`] takes the theme, and the six come from the hues this
//!   workspace already has rather than from six new ones.
//!
//! ## What the format does not say, and this crate therefore checks
//!
//! `id` is "a unique ID for the node" and `fromNode`/`toNode` are node ids — but nothing in the JSON stops a
//! document from repeating an id or pointing an edge at a node that is not there. Those are the two faults a
//! *reader* has to handle and a *writer* never produces, so [`Canvas::validate`] reports them and the parser does
//! **not** refuse the document: a canvas with a dangling edge is still readable, and an editor that dropped it
//! would silently delete a user's work.
//!
//! ## The fixed point
//!
//! The same guarantee `makepad-markdown` makes, for the same reason: `parse(serialize(parse(s))) == parse(s)`, so a
//! load/save cycle cannot drift. It is checked over a corpus built from the cases that *would* drift — a node with
//! no colour, a group with a background style, an edge with only one side set, the presets, and a document with
//! faults.

use std::collections::HashMap;

use makepad_theme::Paint;
use makepad_widgets::Vec4f;
use serde::{Deserialize, Serialize};

/// A canvas: nodes at positions, and edges between them.
///
/// Nodes are in **ascending z-order**, which the spec states rather than implies: *"The first node in the array
/// should be displayed below all other nodes, and the last node in the array should be displayed on top."* So the
/// order of this `Vec` is data, not presentation, and serializing preserves it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Canvas {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<Edge>,
}

/// One node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    /// The node's type and its type-specific fields.
    #[serde(flatten)]
    pub kind: NodeKind,
    /// In pixels, and an **integer** in the spec — a canvas document that says `12.5` is not this format, so the
    /// type carries that rather than a comment.
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

/// What a node is, with the type tag flattened into the node object.
///
/// `#[serde(tag = "type", rename_all = "lowercase")]` is what makes the JSON the spec describes: `{"id": ..,
/// "type": "text", "text": ..}` rather than a nested object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NodeKind {
    /// Plain text with Markdown syntax.
    Text { text: String },
    /// A reference to a file, optionally to a heading or block inside it.
    File {
        file: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        subpath: Option<String>,
    },
    /// A URL.
    Link { url: String },
    /// A visual container for other nodes.
    Group {
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        background: Option<String>,
        #[serde(
            rename = "backgroundStyle",
            skip_serializing_if = "Option::is_none"
        )]
        background_style: Option<BackgroundStyle>,
    },
}

impl NodeKind {
    /// The type name the JSON carries.
    pub fn type_name(&self) -> &'static str {
        match self {
            NodeKind::Text { .. } => "text",
            NodeKind::File { .. } => "file",
            NodeKind::Link { .. } => "link",
            NodeKind::Group { .. } => "group",
        }
    }

    /// The `subpath`, if this is a file node and it has one.
    pub fn subpath(&self) -> Option<&str> {
        match self {
            NodeKind::File { subpath, .. } => subpath.as_deref(),
            _ => None,
        }
    }
}

/// How a group's background image is rendered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundStyle {
    Cover,
    Ratio,
    Repeat,
}

/// One edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    #[serde(rename = "fromNode")]
    pub from_node: String,
    #[serde(rename = "fromSide", skip_serializing_if = "Option::is_none")]
    pub from_side: Option<Side>,
    #[serde(rename = "fromEnd", skip_serializing_if = "Option::is_none")]
    pub from_end: Option<End>,
    #[serde(rename = "toNode")]
    pub to_node: String,
    #[serde(rename = "toSide", skip_serializing_if = "Option::is_none")]
    pub to_side: Option<Side>,
    #[serde(rename = "toEnd", skip_serializing_if = "Option::is_none")]
    pub to_end: Option<End>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl Edge {
    /// The shape at the start of the edge, **with the spec's default applied**.
    ///
    /// The two ends do not share a default: `fromEnd` is `none` and `toEnd` is `arrow`. A reader that assumed one
    /// default for both would draw an arrow at the start of every connection.
    pub fn from_end(&self) -> End {
        self.from_end.unwrap_or(End::Plain)
    }

    /// The shape at the end of the edge, with the spec's default applied.
    pub fn to_end(&self) -> End {
        self.to_end.unwrap_or(End::Arrow)
    }
}

/// Which side of a node an edge attaches to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

/// The shape at an edge's end.
///
/// `Plain` rather than `None`, because the JSON word is `none` and a variant called `None` beside `Option`'s would
/// make every `match` in a caller read ambiguously.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    /// No endpoint shape. **The JSON word is `none`** and the variant is `Plain`, because a variant called `None`
    /// beside `Option`'s would make every `match` in a caller read ambiguously.
    #[serde(rename = "none")]
    Plain,
    #[serde(rename = "arrow")]
    Arrow,
}

/// A node's or an edge's colour: one of the six presets, or a hex string.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    /// A preset, as the spec's `"1"` through `"6"`.
    Preset(Preset),
    /// `#RRGGBB` or `#RRGGBBAA`.
    Hex(String),
}

/// The six preset colours.
///
/// A closed set, and the spec's own list rather than a plausible one: red, orange, yellow, green, cyan, purple.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    #[serde(rename = "1")]
    Red,
    #[serde(rename = "2")]
    Orange,
    #[serde(rename = "3")]
    Yellow,
    #[serde(rename = "4")]
    Green,
    #[serde(rename = "5")]
    Cyan,
    #[serde(rename = "6")]
    Purple,
}

impl Preset {
    /// The number the JSON carries.
    pub fn index(self) -> u8 {
        match self {
            Preset::Red => 1,
            Preset::Orange => 2,
            Preset::Yellow => 3,
            Preset::Green => 4,
            Preset::Cyan => 5,
            Preset::Purple => 6,
        }
    }

    /// The preset's name, for a report that has to say which one.
    pub fn type_name(self) -> &'static str {
        match self {
            Preset::Red => "red",
            Preset::Orange => "orange",
            Preset::Yellow => "yellow",
            Preset::Green => "green",
            Preset::Cyan => "cyan",
            Preset::Purple => "purple",
        }
    }

    pub const ALL: [Preset; 6] = [
        Preset::Red,
        Preset::Orange,
        Preset::Yellow,
        Preset::Green,
        Preset::Cyan,
        Preset::Purple,
    ];

    /// The colour, from the theme.
    ///
    /// **The spec leaves these undefined on purpose** — *"so that applications can tailor the presets to their
    /// specific brand colors or color scheme"* — so they come from the paints this workspace already has rather
    /// than from six new tokens: the theme's danger, warning, success, accent and the two syntax hues. A brand that
    /// changes those changes a canvas that says `"1"`, which is the whole point of the spec's choice.
    pub fn color(self, paint: &Paint) -> Vec4f {
        match self {
            Preset::Red => paint.danger,
            Preset::Orange => paint.warning,
            Preset::Yellow => paint.diff_hunk_bg,
            Preset::Green => paint.success,
            Preset::Cyan => paint.code_text,
            Preset::Purple => paint.accent,
        }
    }
}

/// Something wrong with a canvas: readable, but not what the format requires.
///
/// Reported rather than refused, because a reader that dropped a document with a dangling edge would delete a
/// user's work over a fault it can still display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// Two nodes, or two edges, share an id. The spec says "a unique ID".
    DuplicateId { id: String },
    /// An edge names a node that is not in the document, or one that is not a node at all.
    DanglingEdge {
        edge: String,
        missing: String,
    },
    /// A `subpath` that does not start with `#`, which the spec states as always true.
    SubpathWithoutHash { node: String, subpath: String },
    /// A node or an edge with an empty id.
    EmptyId { what: &'static str, index: usize },
    /// A colour string that is neither a preset nor a hex colour.
    BadColor { what: String, value: String },
}

/// Parse a JSON Canvas document.
///
/// An error only for JSON that does not fit the format at all — a missing required field, a bad type. Anything the
/// format *allows* to be wrong (**duplicate ids, dangling edges**) parses, and [`Canvas::validate`] reports it. See
/// [`Problem`] on why.
pub fn parse(text: &str) -> Result<Canvas, serde_json::Error> {
    serde_json::from_str(text)
}

/// Write a JSON Canvas document.
///
/// `to_string_pretty` with a trailing newline, because a canvas is a file a person opens in an editor — and the
/// trailing newline is what makes a save cycle leave a text file alone.
pub fn serialize(canvas: &Canvas) -> String {
    let mut out = serde_json::to_string_pretty(canvas).unwrap_or_else(|_| "{}".to_string());
    out.push('\n');
    out
}

impl Canvas {
    /// Whether `text` is a hex colour this format accepts: `#` and six or eight hex digits.
    pub fn is_hex_color(text: &str) -> bool {
        let Some(digits) = text.strip_prefix('#') else {
            return false;
        };
        matches!(digits.len(), 6 | 8) && digits.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Everything wrong with this canvas, in document order.
    ///
    /// Empty is the normal answer. See [`Problem`] on why these are reported rather than refused.
    pub fn validate(&self) -> Vec<Problem> {
        let mut problems = Vec::new();

        let mut node_ids: HashMap<&str, usize> = HashMap::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if node.id.is_empty() {
                problems.push(Problem::EmptyId {
                    what: "node",
                    index,
                });
            } else if node_ids.insert(&node.id, index).is_some() {
                problems.push(Problem::DuplicateId {
                    id: node.id.clone(),
                });
            }
            if let Some(subpath) = node.kind.subpath() {
                if !subpath.starts_with('#') {
                    problems.push(Problem::SubpathWithoutHash {
                        node: node.id.clone(),
                        subpath: subpath.to_string(),
                    });
                }
            }
            if let Some(Color::Hex(hex)) = &node.color {
                if !Self::is_hex_color(hex) {
                    problems.push(Problem::BadColor {
                        what: format!("node {}", node.id),
                        value: hex.clone(),
                    });
                }
            }
        }

        let mut edge_ids: HashMap<&str, usize> = HashMap::new();
        for (index, edge) in self.edges.iter().enumerate() {
            if edge.id.is_empty() {
                problems.push(Problem::EmptyId {
                    what: "edge",
                    index,
                });
            } else if edge_ids.insert(&edge.id, index).is_some() {
                problems.push(Problem::DuplicateId {
                    id: edge.id.clone(),
                });
            }
            for end in [&edge.from_node, &edge.to_node] {
                if !node_ids.contains_key(end.as_str()) {
                    problems.push(Problem::DanglingEdge {
                        edge: edge.id.clone(),
                        missing: end.clone(),
                    });
                }
            }
            if let Some(Color::Hex(hex)) = &edge.color {
                if !Self::is_hex_color(hex) {
                    problems.push(Problem::BadColor {
                        what: format!("edge {}", edge.id),
                        value: hex.clone(),
                    });
                }
            }
        }

        problems
    }

    /// The node with this id.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// The edges that touch this node, in either direction.
    pub fn edges_of(&self, id: &str) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|edge| edge.from_node == id || edge.to_node == id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example the spec's own front matter implies, written by hand so the expected JSON is visible.
    const SAMPLE: &str = r##"{
  "nodes": [
    { "id": "a", "type": "text", "x": 0, "y": 0, "width": 200, "height": 100, "text": "# Hi" },
    { "id": "b", "type": "file", "x": 300, "y": 0, "width": 200, "height": 100, "file": "notes/x.md", "subpath": "#section" },
    { "id": "c", "type": "link", "x": 0, "y": 200, "width": 200, "height": 100, "url": "https://jsoncanvas.org", "color": "#FF0000" },
    { "id": "g", "type": "group", "x": -20, "y": -20, "width": 540, "height": 340, "label": "Group", "background": "bg.png", "backgroundStyle": "cover" }
  ],
  "edges": [
    { "id": "e1", "fromNode": "a", "fromSide": "right", "toNode": "b", "toSide": "left", "color": "1", "label": "see" },
    { "id": "e2", "fromNode": "b", "toNode": "c" }
  ]
}
"##;

    fn sample() -> Canvas {
        parse(SAMPLE).expect("the sample parses")
    }

    #[test]
    fn test_the_sample_parses_into_the_shape_the_spec_describes() {
        let canvas = sample();
        assert_eq!(canvas.nodes.len(), 4);
        assert_eq!(canvas.edges.len(), 2);
        assert_eq!(canvas.nodes[0].id, "a");
        assert_eq!(canvas.nodes[0].kind.type_name(), "text");
        assert!(matches!(&canvas.nodes[0].kind, NodeKind::Text { text } if text == "# Hi"));
        assert_eq!(canvas.nodes[0].x, 0);
        assert_eq!(canvas.nodes[0].width, 200);
        assert!(canvas.nodes[0].color.is_none());
        // The file node's subpath, and the group's style.
        assert_eq!(canvas.nodes[1].kind.subpath(), Some("#section"));
        assert!(matches!(
            &canvas.nodes[3].kind,
            NodeKind::Group { background_style: Some(BackgroundStyle::Cover), .. }
        ));
    }

    #[test]
    fn test_a_preset_and_a_hex_colour_both_parse_and_stay_distinct() {
        // `canvasColor` is a union of one of six numbers and a hex string, and the two must not be confused: a
        // parser that turned `"1"` into a hex string would paint every preset node black.
        let canvas = sample();
        assert_eq!(canvas.nodes[2].color, Some(Color::Hex("#FF0000".to_string())));
        assert_eq!(canvas.edges[0].color, Some(Color::Preset(Preset::Red)));
        // And all six presets parse as presets.
        for preset in Preset::ALL {
            let json = format!("\"nodes\": [{{ \"id\": \"n\", \"type\": \"text\", \"x\": 0, \"y\": 0, \"width\": 1, \"height\": 1, \"text\": \"\", \"color\": \"{}\" }}]", preset.index());
            let text = format!("{{ {json} }}");
            let canvas = parse(&text).expect("a preset parses");
            assert_eq!(
                canvas.nodes[0].color,
                Some(Color::Preset(preset)),
                "preset {} did not survive",
                preset.index()
            );
        }
    }

    #[test]
    fn test_the_preset_order_is_the_spec_s_list_and_not_the_rainbow() {
        // Read from the spec rather than remembered: red, orange, yellow, green, **cyan**, purple.
        let numbers: Vec<u8> = Preset::ALL.iter().map(|preset| preset.index()).collect();
        assert_eq!(numbers, vec![1, 2, 3, 4, 5, 6]);
        let canvas = parse(
            r##"{"nodes":[{"id":"n","type":"text","x":0,"y":0,"width":1,"height":1,"text":"","color":"5"}]}"##,
        )
        .expect("parses");
        assert_eq!(canvas.nodes[0].color, Some(Color::Preset(Preset::Cyan)));
    }

    #[test]
    fn test_the_two_ends_of_an_edge_have_different_defaults() {
        // **The detail a confident recollection gets wrong.** `fromEnd` defaults to `none`, `toEnd` to `arrow`.
        let canvas = sample();
        let edge = &canvas.edges[1];
        assert_eq!(edge.from_end(), End::Plain);
        assert_eq!(edge.to_end(), End::Arrow);
        // An explicit value is kept, on either end.
        let explicit = parse(
            r##"{"nodes":[],"edges":[{"id":"e","fromNode":"a","toNode":"b","fromEnd":"arrow","toEnd":"none"}]}"##,
        )
        .expect("parses");
        assert_eq!(explicit.edges[0].from_end(), End::Arrow);
        assert_eq!(explicit.edges[0].to_end(), End::Plain);
    }

    #[test]
    fn test_the_document_reaches_the_fixed_point() {
        // `parse(serialize(parse(s))) == parse(s)`, the guarantee `makepad-markdown` makes and for the same reason:
        // a load/save cycle cannot drift.
        let canvas = sample();
        let written = serialize(&canvas);
        let reread = parse(&written).expect("what we wrote parses");
        assert_eq!(reread, canvas, "the document changed:\n{written}");
        // And a second write is byte-identical, so the file settles after one save.
        assert_eq!(serialize(&reread), written);
    }

    #[test]
    fn test_every_corpus_entry_reaches_the_fixed_point() {
        // Built from the cases that **would** drift: absent optionals, each node type, each preset, a hex colour,
        // an edge with one side set, and a document with faults in it.
        let corpus = [
            r##"{}"##,
            r##"{"nodes":[]}"##,
            r##"{"nodes":[],"edges":[]}"##,
            SAMPLE,
            r##"{"nodes":[{"id":"n","type":"text","x":-5,"y":5,"width":1,"height":1,"text":"a\nb"}]}"##,
            r##"{"nodes":[{"id":"n","type":"file","x":0,"y":0,"width":1,"height":1,"file":"a.md"}]}"##,
            r##"{"nodes":[{"id":"n","type":"file","x":0,"y":0,"width":1,"height":1,"file":"a.md","subpath":"#h"}]}"##,
            r##"{"nodes":[{"id":"n","type":"link","x":0,"y":0,"width":1,"height":1,"url":"https://x"}]}"##,
            r##"{"nodes":[{"id":"n","type":"group","x":0,"y":0,"width":1,"height":1}]}"##,
            r##"{"nodes":[{"id":"n","type":"group","x":0,"y":0,"width":1,"height":1,"backgroundStyle":"ratio"}]}"##,
            r##"{"nodes":[{"id":"n","type":"text","x":0,"y":0,"width":1,"height":1,"text":"","color":"6"}]}"##,
            r##"{"nodes":[{"id":"n","type":"text","x":0,"y":0,"width":1,"height":1,"text":"","color":"#00FF00CC"}]}"##,
            // Faults the format allows: they parse, and `validate` reports them.
            r##"{"nodes":[{"id":"n","type":"text","x":0,"y":0,"width":1,"height":1,"text":""},{"id":"n","type":"text","x":0,"y":0,"width":1,"height":1,"text":""}]}"##,
            r##"{"nodes":[],"edges":[{"id":"e","fromNode":"ghost","toNode":"ghost2"}]}"##,
            r##"{"nodes":[{"id":"n","type":"file","x":0,"y":0,"width":1,"height":1,"file":"a.md","subpath":"nohash"}]}"##,
            r##"{"nodes":[],"edges":[{"id":"e","fromNode":"a","toNode":"b","fromSide":"bottom","toSide":"left","fromEnd":"none"}]}"##,
        ];
        for source in corpus {
            let first = parse(source).unwrap_or_else(|e| panic!("{source} did not parse: {e}"));
            let written = serialize(&first);
            let second = parse(&written)
                .unwrap_or_else(|e| panic!("what we wrote did not parse: {e}\n{written}"));
            assert_eq!(first, second, "the document changed:\n{source}\n{written}");
            assert_eq!(
                serialize(&second),
                written,
                "the text did not settle:\n{source}"
            );
        }
    }

    #[test]
    fn test_z_order_is_the_array_order_and_survives_a_round_trip() {
        // The spec says the first node is below and the last on top, so the order is **data** rather than
        // presentation — a serializer that sorted by position would reorder the painting.
        let canvas = parse(
            r##"{"nodes":[
                {"id":"bottom","type":"text","x":100,"y":100,"width":1,"height":1,"text":""},
                {"id":"top","type":"text","x":0,"y":0,"width":1,"height":1,"text":""}
            ]}"##,
        )
        .expect("parses");
        assert_eq!(canvas.nodes[0].id, "bottom");
        let reread = parse(&serialize(&canvas)).expect("parses");
        assert_eq!(reread.nodes[0].id, "bottom", "the z-order changed");
    }

    #[test]
    fn test_the_document_does_not_carry_fields_it_was_not_given() {
        // `skip_serializing_if` on every optional: a canvas written from a model with no colours must not gain
        // `"color": null` everywhere, or the file is not the one the reader wrote.
        let canvas = parse(
            r##"{"nodes":[{"id":"n","type":"text","x":0,"y":0,"width":1,"height":1,"text":"a"}]}"##,
        )
        .expect("parses");
        let written = serialize(&canvas);
        assert!(!written.contains("null"), "{written}");
        assert!(!written.contains("color"), "{written}");
        assert!(!written.contains("edges"), "{written}");
        // And it ends with a newline, so a text editor leaves the file alone.
        assert!(written.ends_with('\n'));
    }

    #[test]
    fn test_an_empty_canvas_writes_an_empty_object() {
        // `{}` rather than `{"nodes":[],"edges":[]}`: both are valid, and the shorter one is what a person would
        // write. The fixed point holds either way, which the corpus checks.
        assert_eq!(serialize(&Canvas::default()), "{}\n");
    }

    #[test]
    fn test_validation_reports_the_two_faults_the_format_allows() {
        // Duplicate ids and an edge to a node that is not there. Both parse — a reader that refused them would
        // delete a user's work over a fault it can still display.
        let canvas = parse(
            r##"{"nodes":[
                {"id":"a","type":"text","x":0,"y":0,"width":1,"height":1,"text":""},
                {"id":"a","type":"text","x":0,"y":0,"width":1,"height":1,"text":""}
            ],"edges":[
                {"id":"e1","fromNode":"a","toNode":"ghost"},
                {"id":"e2","fromNode":"ghost","toNode":"a"}
            ]}"##,
        )
        .expect("it parses, faults and all");
        let problems = canvas.validate();
        assert!(problems.contains(&Problem::DuplicateId { id: "a".into() }));
        assert!(problems.contains(&Problem::DanglingEdge {
            edge: "e1".into(),
            missing: "ghost".into()
        }));
        assert!(problems.contains(&Problem::DanglingEdge {
            edge: "e2".into(),
            missing: "ghost".into()
        }));
    }

    #[test]
    fn test_validation_accepts_a_canvas_a_writer_would_produce() {
        assert!(sample().validate().is_empty());
        // ...and reports a subpath without its `#`, which the spec states as always true.
        let canvas = parse(
            r##"{"nodes":[{"id":"n","type":"file","x":0,"y":0,"width":1,"height":1,"file":"a.md","subpath":"nohash"}]}"##,
        )
        .expect("parses");
        assert_eq!(
            canvas.validate(),
            vec![Problem::SubpathWithoutHash {
                node: "n".into(),
                subpath: "nohash".into()
            }]
        );
    }

    #[test]
    fn test_a_hex_colour_is_checked_rather_than_trusted() {
        assert!(Canvas::is_hex_color("#FF0000"));
        assert!(Canvas::is_hex_color("#FF0000CC"));
        assert!(!Canvas::is_hex_color("FF0000"), "no hash");
        assert!(!Canvas::is_hex_color("#FF00"), "four digits is not one of the two lengths");
        assert!(!Canvas::is_hex_color("#GGGGGG"), "not hex digits");
        // A bad colour is reported rather than silently kept, because a painter handed `"#GGGGGG"` would draw
        // whatever its parser's fallback is.
        let canvas = parse(
            r##"{"nodes":[{"id":"n","type":"text","x":0,"y":0,"width":1,"height":1,"text":"","color":"#GGGGGG"}]}"##,
        )
        .expect("parses");
        assert!(matches!(canvas.validate().first(), Some(Problem::BadColor { .. })));
    }

    #[test]
    fn test_an_empty_id_is_reported_on_a_node_and_on_an_edge() {
        let canvas = parse(
            r##"{"nodes":[{"id":"","type":"text","x":0,"y":0,"width":1,"height":1,"text":""}],
                "edges":[{"id":"","fromNode":"","toNode":""}]}"##,
        )
        .expect("parses");
        let problems = canvas.validate();
        assert!(problems.contains(&Problem::EmptyId {
            what: "node",
            index: 0
        }));
        assert!(problems.contains(&Problem::EmptyId {
            what: "edge",
            index: 0
        }));
    }

    #[test]
    fn test_the_six_presets_are_all_distinct_colours_from_the_theme() {
        // The spec leaves the values to the application, so they come from the paints this workspace has. Two
        // presets the same colour would make the format's six indistinguishable, which is the format's whole
        // point.
        for paint in [
            makepad_theme::palette::dark(),
            makepad_theme::palette::light(),
        ] {
            let colours: Vec<Vec4f> = Preset::ALL.iter().map(|preset| preset.color(&paint)).collect();
            for (a, first) in colours.iter().enumerate() {
                for (b, second) in colours.iter().enumerate() {
                    if a == b {
                        continue;
                    }
                    assert!(
                        first != second,
                        "presets {} and {} share a colour",
                        Preset::ALL[a].index(),
                        Preset::ALL[b].index()
                    );
                }
            }
        }
    }

    #[test]
    fn test_the_neighbours_of_a_node_are_its_edges_in_either_direction() {
        let canvas = sample();
        let of_a = canvas.edges_of("a");
        assert_eq!(of_a.len(), 1, "a is only the start of e1");
        let of_b = canvas.edges_of("b");
        assert_eq!(of_b.len(), 2, "b is the end of e1 and the start of e2");
        assert!(canvas.edges_of("ghost").is_empty());
        assert!(canvas.node("b").is_some());
        assert!(canvas.node("ghost").is_none());
    }

    #[test]
    fn test_json_that_is_not_a_canvas_is_an_error_rather_than_a_default() {
        // A missing required field is a **format** error, unlike the faults `validate` reports: a node without a
        // position cannot be displayed anywhere, so there is nothing to read.
        assert!(parse(r##"{"nodes":[{"id":"n"}]}"##).is_err());
        assert!(parse(r##"{"nodes":[{"id":"n","type":"text","x":0,"y":0}]}"##).is_err());
        assert!(parse("not json").is_err());
        // And an unknown type is an error too, rather than a node that paints nothing.
        assert!(parse(
            r##"{"nodes":[{"id":"n","type":"hologram","x":0,"y":0,"width":1,"height":1}]}"##
        )
        .is_err());
    }
}
