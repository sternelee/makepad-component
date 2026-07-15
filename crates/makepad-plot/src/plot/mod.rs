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

pub fn script_mod(vm: &mut ScriptVm) {
    types::script_mod(vm);
    line::script_mod(vm);
    bar::script_mod(vm);
    scatter::script_mod(vm);
    pie::script_mod(vm);
    histogram::script_mod(vm);
    stem::script_mod(vm);
    heatmap::script_mod(vm);
    polar::script_mod(vm);
    contour::script_mod(vm);
    surface3d::script_mod(vm);
    scatter3d::script_mod(vm);
    dual::script_mod(vm);
    financial::script_mod(vm);
    gauge::script_mod(vm);
    treemap::script_mod(vm);
    bubble::script_mod(vm);
    area::script_mod(vm);
    stack::script_mod(vm);
    hexbin::script_mod(vm);
}
