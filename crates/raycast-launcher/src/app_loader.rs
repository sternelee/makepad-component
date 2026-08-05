use makepad_script::{ScriptMod, ScriptTrap::NoTrap, ScriptValue};
use makepad_widgets::*;
use serde::{Deserialize, Serialize};
use std::fs;

/// Build a `ScriptMod` from raw Splash code for `vm.eval()`.
/// Symbols like View/CheckBox must be injected via `vm.set_injected_global`
/// before eval, since runtime `use` does not support wildcard imports.
/// Used by the AI chat → Splash app generation pipeline.
#[allow(dead_code)]
pub fn script_mod_from_code(code: &str) -> ScriptMod {
    ScriptMod {
        cargo_manifest_path: env!("CARGO_MANIFEST_DIR").to_string(),
        module_path: "raycast_launcher".to_string(),
        file: file!().to_string(),
        line: 0,
        column: 0,
        code: format!("let __splash_templates = {}\n__splash_templates", code),
        values: vec![],
    }
}

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
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
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
        let heap = vm.heap_mut();
        // Splash `mod` resolves to heap.modules (vm.module(id!(mod)) returns ZERO
        // because "mod" is a scope variable, not a registered module name).
        let mod_obj = heap.modules;
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        if let Some(state_obj) = state_val.as_object() {
            let app_state = json_to_script_value(heap, state);
            heap.set_value_def(state_obj, ScriptValue::from_id(id!(app)), app_state);
            log!(
                "inject_app_state: wrote app.state with {} keys",
                state.as_object().map_or(0, |o| o.len())
            );
        } else {
            log!("inject_app_state: mod.state not found");
        }
    });
}

fn script_value_to_json(
    heap: &makepad_script::ScriptHeap,
    value: ScriptValue,
) -> serde_json::Value {
    if value.is_nil() {
        return serde_json::Value::Null;
    }
    if let Some(boolean) = value.as_bool() {
        return serde_json::Value::Bool(boolean);
    }
    if let Some(number) = value.as_f64() {
        if number.fract() == 0.0
            && number.is_finite()
            && number >= i64::MIN as f64
            && number <= i64::MAX as f64
        {
            return serde_json::Value::Number((number as i64).into());
        }
        return serde_json::Number::from_f64(number)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null);
    }
    if value.is_string_like() {
        return heap
            .string_with(value, |_, string| {
                serde_json::Value::String(string.to_string())
            })
            .unwrap_or(serde_json::Value::String(String::new()));
    }
    if let Some(array) = value.as_array() {
        let mut items = Vec::with_capacity(heap.array_len(array));
        for index in 0..heap.array_len(array) {
            items.push(script_value_to_json(
                heap,
                heap.array_index(array, index, NoTrap),
            ));
        }
        return serde_json::Value::Array(items);
    }
    if let Some(object) = value.as_object() {
        let mut map = serde_json::Map::new();
        for index in 0..heap.iter_len(object) {
            let kv = heap.iter_key_value(object, index, NoTrap);
            let Some(key) = script_key_to_string(heap, kv.key) else {
                continue;
            };
            map.insert(key, script_value_to_json(heap, kv.value));
        }
        return serde_json::Value::Object(map);
    }
    if let Some(id) = value.as_id() {
        return serde_json::Value::String(id.to_string());
    }
    serde_json::Value::Null
}

fn script_key_to_string(heap: &makepad_script::ScriptHeap, value: ScriptValue) -> Option<String> {
    if let Some(id) = value.as_id() {
        return Some(id.to_string());
    }
    heap.string_with(value, |_, string| string.to_string())
}

pub fn read_app_state(cx: &mut Cx) -> Option<serde_json::Value> {
    cx.with_vm(|vm| {
        let heap = vm.heap();
        let mod_obj = heap.modules;
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        let state_obj = state_val.as_object()?;
        let app_val = heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
        Some(script_value_to_json(heap, app_val))
    })
}

/// Write a search query string into `mod.state.app.search` in the Splash VM.
/// Called from Rust when mode_input changes, so the Splash body can filter on reload.
pub fn set_splash_search(cx: &mut Cx, search: &str) {
    cx.with_vm(|vm| {
        let heap = vm.heap_mut();
        let mod_obj = heap.modules;
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        let Some(state_obj) = state_val.as_object() else {
            return;
        };
        let app_val = heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
        let Some(app_obj) = app_val.as_object() else {
            return;
        };
        let sv = heap.new_string_from_str(search);
        heap.set_value_def(app_obj, ScriptValue::from_id(id!(search)), sv);
    });
}

/// Read the `version` counter from `mod.state.app.version` in the Splash VM.
/// Returns 0 if the field doesn't exist or isn't accessible.
pub fn read_todo_version(cx: &mut Cx) -> i64 {
    cx.with_vm(|vm| {
        let heap = vm.heap();
        let mod_obj = heap.modules;
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        let state_obj = state_val.as_object()?;
        let app_val = heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
        let app_obj = app_val.as_object()?;
        let ver_val = heap.value(app_obj, ScriptValue::from_id(id!(version)), NoTrap);
        Some(ver_val.as_f64().unwrap_or(0.0) as i64)
    })
    .unwrap_or(0)
}

pub fn save_app_state(cx: &mut Cx, path: &str) -> Result<(), String> {
    let mut descriptor = load_app_descriptor(path)
        .ok_or_else(|| format!("Failed to load app descriptor: {}", path))?;
    descriptor.state =
        read_app_state(cx).ok_or_else(|| "mod.state.app is not available".to_string())?;
    let json = serde_json::to_string_pretty(&descriptor)
        .map_err(|error| format!("Failed to serialize app descriptor: {}", error))?;
    fs::write(path, json).map_err(|error| format!("Failed to write app descriptor: {}", error))
}
