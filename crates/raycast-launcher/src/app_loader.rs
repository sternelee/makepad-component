use makepad_script::{ScriptMod, ScriptTrap::NoTrap, ScriptValue};
use makepad_widgets::*;
use serde::{Deserialize, Serialize};

/// App descriptor loaded from JSON.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppDescriptor {
    #[serde(default)]
    pub app: AppInfo,
    /// Complete Splash code string that defines widget templates and app UI.
    #[serde(default)]
    pub splash_code: String,
    /// Initial app state (injected into `mod.state.app`).
    #[serde(default)]
    pub state: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
}

/// Load an app descriptor from a JSON file path.
pub fn load_app_descriptor(path: &str) -> Option<AppDescriptor> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Construct a ScriptMod from a Splash code string.
/// Wraps with imports that are implicitly available in `script_mod!` blocks
/// but missing in runtime `vm.eval()` context.
pub fn script_mod_from_code(code: &str) -> ScriptMod {
    // Prepend imports so widget types (View, Button, Fill, etc.), draw/shader
    // types (uniform, instance, Sdf2d), and theme are in scope — matching the
    // compile-time `script_mod!` environment where the code was authored.
    let wrapped = format!(
        "use mod.prelude.widgets.*\nuse mod.prelude.draw.*\ntheme = mod.prelude.theme\n\n{}",
        code
    );
    ScriptMod {
        cargo_manifest_path: env!("CARGO_MANIFEST_DIR").to_string(),
        module_path: module_path!().to_string(),
        file: file!().to_string(),
        line: 0,
        column: 0,
        code: wrapped,
        values: vec![],
    }
}

/// Convert a JSON value to a ScriptValue.
fn json_to_script_value(
    heap: &mut makepad_script::ScriptHeap,
    value: &serde_json::Value,
) -> ScriptValue {
    match value {
        serde_json::Value::Null => ScriptValue::NIL,
        serde_json::Value::Bool(b) => ScriptValue::from_bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ScriptValue::from_f64(i as f64)
            } else {
                ScriptValue::from_f64(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => heap.new_string_from_str(s),
        serde_json::Value::Array(arr) => {
            let script_arr = heap.new_array();
            for item in arr {
                let val = json_to_script_value(heap, item);
                heap.array_push(script_arr, val, NoTrap);
            }
            script_arr.into()
        }
        serde_json::Value::Object(obj) => {
            let script_obj = heap.new_object();
            for (key, val) in obj {
                let key_id = makepad_live_id::LiveId::from_str(key);
                let val = json_to_script_value(heap, val);
                heap.set_value_def(script_obj, ScriptValue::from_id(key_id), val);
            }
            script_obj.into()
        }
    }
}

/// Inject app state into `mod.state.app`.
pub fn inject_app_state(cx: &mut Cx, state: &serde_json::Value) {
    cx.with_vm(|vm| {
        let mod_obj = vm.module(id!(mod));
        let heap = vm.heap_mut();
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        if let Some(state_obj) = state_val.as_object() {
            let app_state = json_to_script_value(heap, state);
            heap.set_value_def(state_obj, ScriptValue::from_id(id!(app)), app_state);
        }
    });
}
