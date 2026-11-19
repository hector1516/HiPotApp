//! Estado del PLC en modo simulación.
//!
//! Permite operar la UI sin PLC físico: las escrituras se guardan en memoria
//! y las lecturas devuelven el último valor escrito (o un valor plausible).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static SIM: OnceLock<Mutex<HashMap<String, f64>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, f64>> {
    SIM.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn set(key: &str, value: f64) {
    if let Ok(mut map) = store().lock() {
        map.insert(key.to_string(), value);
    }
}

pub fn get(key: &str, default: f64) -> f64 {
    store()
        .lock()
        .ok()
        .and_then(|map| map.get(key).copied())
        .unwrap_or(default)
}

// --- Tags ---
pub fn set_tag(id: i64, value: f64) {
    set(&format!("tag:{}", id), value);
}
pub fn tag(id: i64) -> f64 {
    get(&format!("tag:{}", id), 0.0)
}

// --- Salidas / entradas ---
pub fn set_q(addr: &str, value: bool) {
    set(&format!("q:{}", addr), if value { 1.0 } else { 0.0 });
}
pub fn q(addr: &str) -> bool {
    get(&format!("q:{}", addr), 0.0) != 0.0
}
pub fn set_i(addr: &str, value: bool) {
    set(&format!("i:{}", addr), if value { 1.0 } else { 0.0 });
}
pub fn i(addr: &str) -> bool {
    get(&format!("i:{}", addr), 0.0) != 0.0
}
pub fn set_ia(addr: &str, value: f64) {
    set(&format!("ia:{}", addr), value);
}
pub fn ia(addr: &str) -> f64 {
    get(&format!("ia:{}", addr), 0.0)
}

// --- Receta ---
pub fn set_recipe(voltage: f64, components: &serde_json::Value) {
    set("recipe:voltage", voltage);
    if let Some(arr) = components.as_array() {
        for comp in arr {
            let name = comp
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            if name.is_empty() {
                continue;
            }
            let min = comp.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max = comp.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let enabled = comp
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            set(&format!("comp:{}:min", name), min);
            set(&format!("comp:{}:max", name), max);
            set(
                &format!("comp:{}:enabled", name),
                if enabled { 1.0 } else { 0.0 },
            );
        }
    }
}

// --- Resultados ---
pub fn set_result(name: &str, value: f64) {
    set(&format!("result:{}", name), value);
}
pub fn result(name: &str) -> f64 {
    get(&format!("result:{}", name), 0.0)
}

/// Dispara un test simulado de un componente y devuelve un valor dentro de rango.
pub fn test_component(name: &str) -> f64 {
    let name = name.trim().to_uppercase();
    set("mode", 2.0);
    set("result:running", 1.0);
    set("result:complete", 1.0);
    set("result:pass", 1.0);

    let min = get(&format!("comp:{}:min", name), f64::NAN);
    let max = get(&format!("comp:{}:max", name), f64::NAN);
    let value = if min.is_finite() && max.is_finite() && max > min {
        (min + max) / 2.0
    } else {
        match name.as_str() {
            "FAN" => 0.9,
            "DOOR" => 2.25,
            "FRAME" => 0.85,
            "DRAIN" => 2.15,
            "DEFROST" => 0.7,
            "LIGHT" => 0.35,
            _ => 0.5,
        }
    };
    set_result(&name, value);
    set_result("TOTAL", value);
    set("result:running", 0.0);
    value
}

pub fn set_mode(mode: i32) {
    set("mode", mode as f64);
}
pub fn mode() -> i32 {
    get("mode", 1.0) as i32
}
