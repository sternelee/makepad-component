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
///
/// Runtime `vm.eval()` uses the crate-root module context (same as `main.rs`
/// `script_mod!`) so that `mod.prelude.widgets.*` resolves identically.
/// The `use` statement is prepended because eval'd code does not inherit
/// imports from the calling site.
pub fn script_mod_from_code(code: &str) -> ScriptMod {
    let wrapped = format!("use mod.prelude.widgets.*\n{}", code);
    ScriptMod {
        cargo_manifest_path: env!("CARGO_MANIFEST_DIR").to_string(),
        // must match the crate root so `mod.prelude` resolves to the same
        // prelude object that `main.rs script_mod!` sees
        module_path: "raycast_launcher".to_string(),
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
