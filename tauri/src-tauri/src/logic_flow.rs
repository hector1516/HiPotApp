//! Flujo editable y restringido de la secuencia.
//!
//! No es un editor de código: expone solamente tiempos, orden de componentes,
//! tolerancia de voltaje y parada por fallo. El motor valida el flujo antes de
//! usarlo y rechaza cualquier flujo incompleto.

use serde_json::{json, Map, Value};

pub const COMPONENT_NAMES: [&str; 6] = ["FAN", "DOOR", "FRAME", "DRAIN", "DEFROST", "LIGHT"];

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentStep {
    pub name: String,
    pub enabled: bool,
    pub test_time_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogicFlow {
    pub stabilize_ms: u64,
    pub default_test_time_ms: u64,
    pub voltage_tolerance_pct: f64,
    pub stop_on_fail: bool,
    pub components: Vec<ComponentStep>,
}

impl Default for LogicFlow {
    fn default() -> Self {
        Self::defaults()
    }
}

impl LogicFlow {
    pub fn defaults() -> Self {
        Self {
            stabilize_ms: 2400,
            default_test_time_ms: 0,
            voltage_tolerance_pct: 5.0,
            stop_on_fail: true,
            components: COMPONENT_NAMES
                .iter()
                .map(|name| ComponentStep {
                    name: name.to_string(),
                    enabled: true,
                    test_time_ms: 0,
                })
                .collect(),
        }
    }

    pub fn from_config(config: &Value) -> Self {
        Self::from_config_key(config, "logic_flow_published")
    }

    pub fn from_config_key(config: &Value, key: &str) -> Self {
        let stored = match config.get(key) {
            Some(Value::String(text)) => serde_json::from_str::<Value>(text).ok(),
            Some(value) => Some(value.clone()),
            None => None,
        };
        let Some(value) = stored else {
            return Self::defaults();
        };
        let source = value
            .get("flow")
            .and_then(|v| v.as_object())
            .or_else(|| value.as_object());
        let Some(source) = source else {
            return Self::defaults();
        };

        let mut flow = Self::defaults();
        if let Some(v) = source.get("stabilize_ms").and_then(|v| v.as_u64()) {
            flow.stabilize_ms = v;
        }
        if let Some(v) = source.get("default_test_time_ms").and_then(|v| v.as_u64()) {
            flow.default_test_time_ms = v;
        }
        if let Some(v) = source.get("voltage_tolerance_pct").and_then(|v| v.as_f64()) {
            flow.voltage_tolerance_pct = v;
        }
        if let Some(v) = source.get("stop_on_fail").and_then(|v| v.as_bool()) {
            flow.stop_on_fail = v;
        }

        if let Some(steps) = source.get("components").and_then(|v| v.as_array()) {
            let mut ordered: Vec<ComponentStep> = Vec::new();
            for step in steps {
                let Some(name) = step.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };
                if !COMPONENT_NAMES.contains(&name) {
                    continue;
                }
                if ordered.iter().any(|existing| existing.name == name) {
                    continue;
                }
                ordered.push(ComponentStep {
                    name: name.to_string(),
                    enabled: step.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                    test_time_ms: step.get("test_time_ms").and_then(|v| v.as_u64()).unwrap_or(0),
                });
            }
            for default_step in &flow.components {
                if !ordered.iter().any(|existing| existing.name == default_step.name) {
                    ordered.push(default_step.clone());
                }
            }
            flow.components = ordered;
        }
        flow
    }

    pub fn component(&self, name: &str) -> Option<&ComponentStep> {
        self.components.iter().find(|step| step.name == name)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(100..=60_000).contains(&self.stabilize_ms) {
            return Err("stabilize_ms debe estar entre 100 y 60000".to_string());
        }
        if self.default_test_time_ms > 600_000 {
            return Err("default_test_time_ms máximo 600000".to_string());
        }
        if !(0.5..=20.0).contains(&self.voltage_tolerance_pct) {
            return Err("voltage_tolerance_pct debe estar entre 0.5 y 20".to_string());
        }

        let mut seen = std::collections::BTreeSet::new();
        for step in &self.components {
            if !COMPONENT_NAMES.contains(&step.name.as_str()) {
                return Err(format!("Componente desconocido: {}", step.name));
            }
            if !seen.insert(step.name.clone()) {
                return Err(format!("Componente duplicado: {}", step.name));
            }
            if step.test_time_ms > 600_000 {
                return Err(format!("{}: test_time_ms máximo 600000", step.name));
            }
        }
        for name in COMPONENT_NAMES {
            if !seen.contains(name) {
                return Err(format!("Falta el componente: {}", name));
            }
        }
        if let Some(step) = self.components.iter().find(|step| !step.enabled) {
            return Err(format!(
                "{}: los componentes se habilitan o deshabilitan en la receta, no en el flujo",
                step.name
            ));
        }
        Ok(())
    }

    pub fn validate_payload(payload: &Value) -> Result<Self, String> {
        let source = payload
            .get("flow")
            .unwrap_or(payload)
            .as_object()
            .ok_or_else(|| "El flujo debe ser un objeto".to_string())?;
        let stabilize_ms = source
            .get("stabilize_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(2400);
        let default_test_time_ms = source
            .get("default_test_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let voltage_tolerance_pct = source
            .get("voltage_tolerance_pct")
            .and_then(|v| v.as_f64())
            .unwrap_or(5.0);
        let stop_on_fail = source
            .get("stop_on_fail")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let steps = source
            .get("components")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "El flujo debe tener 'components'".to_string())?;
        let mut components = Vec::with_capacity(steps.len());
        for step in steps {
            let name = step
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Componente sin nombre".to_string())?;
            components.push(ComponentStep {
                name: name.to_string(),
                enabled: step.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                test_time_ms: step.get("test_time_ms").and_then(|v| v.as_u64()).unwrap_or(0),
            });
        }
        let flow = Self {
            stabilize_ms,
            default_test_time_ms,
            voltage_tolerance_pct,
            stop_on_fail,
            components,
        };
        flow.validate()?;
        Ok(flow)
    }

    pub fn to_json(&self) -> Value {
        json!({
            "stabilize_ms": self.stabilize_ms,
            "default_test_time_ms": self.default_test_time_ms,
            "voltage_tolerance_pct": self.voltage_tolerance_pct,
            "stop_on_fail": self.stop_on_fail,
            "components": self.components.iter().map(|step| json!({
                "name": step.name,
                "enabled": step.enabled,
                "test_time_ms": step.test_time_ms,
            })).collect::<Vec<_>>(),
        })
    }

    pub fn storage_json(&self) -> String {
        self.to_json().to_string()
    }

    pub fn components_map(&self) -> Map<String, Value> {
        let mut map = Map::new();
        for step in &self.components {
            map.insert(
                step.name.clone(),
                json!({ "enabled": step.enabled, "test_time_ms": step.test_time_ms }),
            );
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow_with(components: Value) -> Value {
        json!({
            "stabilize_ms": 2400,
            "default_test_time_ms": 0,
            "voltage_tolerance_pct": 5.0,
            "stop_on_fail": true,
            "components": components,
        })
    }

    fn component(name: &str, enabled: bool, test_time_ms: u64) -> Value {
        json!({ "name": name, "enabled": enabled, "test_time_ms": test_time_ms })
    }

    #[test]
    fn defaults_match_current_engine() {
        let flow = LogicFlow::defaults();
        assert_eq!(flow.stabilize_ms, 2400);
        assert_eq!(flow.voltage_tolerance_pct, 5.0);
        assert!(flow.stop_on_fail);
        assert_eq!(flow.components.len(), 6);
        assert_eq!(flow.components[0].name, "FAN");
        assert!(flow.components.iter().all(|step| step.enabled));
    }

    #[test]
    fn validates_reorder_and_times() {
        let payload = flow_with(json!([
            component("LIGHT", true, 3000),
            component("FAN", true, 0),
            component("DOOR", true, 0),
            component("FRAME", true, 0),
            component("DRAIN", true, 0),
            component("DEFROST", true, 0)
        ]));
        let flow = LogicFlow::validate_payload(&payload).unwrap();
        assert_eq!(flow.components[0].name, "LIGHT");
        assert_eq!(flow.components[0].test_time_ms, 3000);
        assert!(flow.components[1].enabled);
    }

    #[test]
    fn rejects_duplicate_component() {
        let payload = flow_with(json!([
            component("FAN", true, 0),
            component("FAN", true, 0),
            component("DOOR", true, 0),
            component("FRAME", true, 0),
            component("DRAIN", true, 0),
            component("DEFROST", true, 0)
        ]));
        let err = LogicFlow::validate_payload(&payload).unwrap_err();
        assert!(err.contains("duplicado"), "{}", err);
    }

    #[test]
    fn rejects_unknown_component() {
        let payload = flow_with(json!([
            component("FAN", true, 0),
            component("DOOR", true, 0),
            component("FRAME", true, 0),
            component("DRAIN", true, 0),
            component("DEFROST", true, 0),
            component("LIGHT", true, 0),
            component("OTRO", true, 0)
        ]));
        let err = LogicFlow::validate_payload(&payload).unwrap_err();
        assert!(err.contains("OTRO"), "{}", err);
    }

    #[test]
    fn rejects_disabling_components_in_the_flow() {
        let payload = flow_with(json!([
            component("FAN", false, 0),
            component("DOOR", true, 0),
            component("FRAME", true, 0),
            component("DRAIN", true, 0),
            component("DEFROST", true, 0),
            component("LIGHT", true, 0)
        ]));
        let err = LogicFlow::validate_payload(&payload).unwrap_err();
        assert!(err.contains("receta"), "{}", err);
    }

    #[test]
    fn rejects_missing_component() {
        let payload = flow_with(json!([
            component("FAN", true, 0),
            component("DOOR", true, 0),
            component("FRAME", true, 0),
            component("DRAIN", true, 0),
            component("DEFROST", true, 0)
        ]));
        let err = LogicFlow::validate_payload(&payload).unwrap_err();
        assert!(err.contains("LIGHT"), "{}", err);
    }

    #[test]
    fn from_config_merges_stored_order_and_values() {
        let stored = json!({
            "stabilize_ms": 3000,
            "default_test_time_ms": 5000,
            "voltage_tolerance_pct": 7.0,
            "stop_on_fail": false,
            "components": [
                component("LIGHT", true, 2000),
                component("FAN", true, 0)
            ]
        })
        .to_string();
        let cfg = json!({ "logic_flow_published": stored });
        let flow = LogicFlow::from_config(&cfg);
        assert_eq!(flow.stabilize_ms, 3000);
        assert_eq!(flow.default_test_time_ms, 5000);
        assert_eq!(flow.voltage_tolerance_pct, 7.0);
        assert!(!flow.stop_on_fail);
        assert_eq!(flow.components[0].name, "LIGHT");
        assert_eq!(flow.components[0].test_time_ms, 2000);
        assert!(flow.component("DOOR").is_some());
    }

    #[test]
    fn reads_draft_key_independently_from_published() {
        let published = LogicFlow::defaults().storage_json();
        let mut draft_value = LogicFlow::defaults();
        draft_value.stabilize_ms = 5000;
        let cfg = json!({
            "logic_flow_published": published,
            "logic_flow_draft": draft_value.storage_json()
        });
        assert_eq!(LogicFlow::from_config(&cfg).stabilize_ms, 2400);
        assert_eq!(
            LogicFlow::from_config_key(&cfg, "logic_flow_draft").stabilize_ms,
            5000
        );
    }

    #[test]
    fn storage_round_trip_preserves_flow() {
        let payload = flow_with(json!([
            component("LIGHT", true, 3000),
            component("FAN", true, 0),
            component("DOOR", true, 0),
            component("FRAME", true, 0),
            component("DRAIN", true, 0),
            component("DEFROST", true, 0)
        ]));
        let flow = LogicFlow::validate_payload(&payload).unwrap();
        let cfg = json!({ "logic_flow_published": flow.storage_json() });
        let back = LogicFlow::from_config(&cfg);
        assert_eq!(back.components[0].name, "LIGHT");
        assert_eq!(back.components[0].test_time_ms, 3000);
        assert!(back.components[1].enabled);
    }
}
