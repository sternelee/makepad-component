// Plot widgets - matplotlib-style plotting library for Makepad

pub mod area;
pub mod bar;
pub mod bubble;
pub mod colormap;
pub mod contour;
pub mod dual;
pub mod financial;
pub mod gauge;
pub mod heatmap;
pub mod hexbin;
pub mod histogram;
pub mod line;
pub mod pie;
pub mod polar;
pub mod scale;
pub mod scatter;
pub mod scatter3d;
pub mod stack;
pub mod stem;
pub mod surface3d;
pub mod treemap;
pub mod types;

// Re-export everything for backwards compatibility
pub use area::*;
pub use bar::*;
pub use bubble::*;
pub use colormap::*;
pub use contour::*;
pub use dual::*;
pub use financial::*;
pub use gauge::*;
pub use heatmap::*;
pub use hexbin::*;
pub use histogram::*;
pub use line::*;
pub use pie::*;
pub use polar::*;
pub use scale::*;
pub use scatter::*;
pub use scatter3d::*;
pub use stack::*;
pub use stem::*;
pub use surface3d::*;
pub use treemap::*;
pub use types::*;

use makepad_widgets::*;

pub fn live_design(cx: &mut Cx) {
    types::live_design(cx);
    line::live_design(cx);
    bar::live_design(cx);
    scatter::live_design(cx);
    pie::live_design(cx);
    histogram::live_design(cx);
    stem::live_design(cx);
    heatmap::live_design(cx);
    polar::live_design(cx);
    contour::live_design(cx);
    surface3d::live_design(cx);
    scatter3d::live_design(cx);
    dual::live_design(cx);
    financial::live_design(cx);
    gauge::live_design(cx);
    treemap::live_design(cx);
    bubble::live_design(cx);
    area::live_design(cx);
    stack::live_design(cx);
    hexbin::live_design(cx);
}
