//! Mapa configurable de salidas lógicas ↔ bits Q del PLC puente.
//!
//! La secuencia de la PC no usa direcciones Q fijas: pide una salida lógica
//! y este mapa la traduce al bit configurado. El mapa se valida antes de
//! guardarlo y antes de iniciar un ciclo.

use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const LOGICAL_OUTPUTS: &[(&str, &str, &str)] = &[
    ("KM2_120", "KM2 · 120 V", "Q0.0"),
    ("KM1_220", "KM1 · 220 V", "Q0.1"),
    ("RANGE_20A", "K2/A4 · rango 0–20 A", "Q0.2"),
    ("RANGE_1A", "K8/A2 · rango 0–1 A", "Q0.3"),
    ("RANGE_10A", "K9/A3 · rango 0–10 A", "Q0.4"),
    ("FAN", "FAN", "Q0.5"),
    ("LIGHT", "LIGHT", "Q0.6"),
    ("FRAME", "FRAME", "Q0.7"),
    ("DOOR", "DOOR", "Q1.0"),
    ("DRAIN", "DRAIN", "Q1.1"),
    ("HEATERS", "HEATERS / DEFROST", "Q2.0"),
];

#[derive(Clone, Debug, PartialEq)]
pub struct OutputMap {
    pub outputs: BTreeMap<String, String>,
}

impl Default for OutputMap {
    fn default() -> Self {
        Self::defaults()
    }
}

impl OutputMap {
    pub fn defaults() -> Self {
        Self {
            outputs: LOGICAL_OUTPUTS
                .iter()
                .map(|(key, _, q)| (key.to_string(), q.to_string()))
                .collect(),
        }
    }

    pub fn from_config(config: &Value) -> Self {
        let stored = match config.get("output_map") {
            Some(Value::String(text)) => serde_json::from_str::<Value>(text).ok(),
            Some(value) => Some(value.clone()),
            None => None,
        };
        let Some(value) = stored else {
            return Self::defaults();
        };
        let source = value
            .get("outputs")
            .and_then(|v| v.as_object())
            .or_else(|| value.as_object());
        let Some(source) = source else {
            return Self::defaults();
        };
        let mut map = Self::defaults();
        for (key, q) in source {
            if let Some(q) = q.as_str() {
                if LOGICAL_OUTPUTS.iter().any(|(known, _, _)| known == key) {
                    map.outputs.insert(key.clone(), q.to_string());
                }
            }
        }
        map
    }

    pub fn q(&self, key: &str) -> Result<&str, String> {
        let q = self
            .outputs
            .get(key)
            .filter(|q| !q.trim().is_empty())
            .ok_or_else(|| format!("Salida lógica sin mapear: {}", key))?;
        validate_q(q)?;
        Ok(q)
    }

    pub fn all_addresses(&self) -> Vec<String> {
        let set: BTreeSet<String> = self
            .outputs
            .values()
            .filter(|q| validate_q(q).is_ok())
            .cloned()
            .collect();
        set.into_iter().collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        for (key, _, _) in LOGICAL_OUTPUTS {
            let q = self
                .outputs
                .get(*key)
                .ok_or_else(|| format!("Falta la salida lógica: {}", key))?;
            validate_q(q).map_err(|e| format!("{}: {}", key, e))?;
        }
        let mut seen = BTreeMap::new();
        for (key, q) in &self.outputs {
            if !LOGICAL_OUTPUTS.iter().any(|(known, _, _)| known == key) {
                return Err(format!("Salida lógica desconocida: {}", key));
            }
            if let Some(previous) = seen.insert(q.clone(), key.clone()) {
                return Err(format!(
                    "Salida duplicada: {} y {} apuntan a {}",
                    previous, key, q
                ));
            }
        }
        Ok(())
    }

    pub fn validate_payload(payload: &Value) -> Result<Self, String> {
        let source = payload
            .get("outputs")
            .unwrap_or(payload)
            .as_object()
            .ok_or_else(|| "El mapa debe tener un objeto 'outputs'".to_string())?;
        let mut outputs = BTreeMap::new();
        for (key, _, _) in LOGICAL_OUTPUTS {
            let q = source
                .get(*key)
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Falta la salida lógica: {}", key))?;
            outputs.insert(key.to_string(), q.trim().to_string());
        }
        let map = Self { outputs };
        map.validate()?;
        Ok(map)
    }

    pub fn to_json(&self) -> Value {
        let mut outputs = Map::new();
        for (key, _, _) in LOGICAL_OUTPUTS {
            outputs.insert(
                key.to_string(),
                json!(self.outputs.get(*key).cloned().unwrap_or_default()),
            );
        }
        json!({ "outputs": Value::Object(outputs) })
    }

    pub fn storage_json(&self) -> String {
        self.to_json().to_string()
    }
}

fn validate_q(q: &str) -> Result<(), String> {
    let rest = q
        .trim()
        .strip_prefix('Q')
        .ok_or_else(|| format!("Dirección inválida: {}", q))?;
    let (byte, bit) = rest
        .split_once('.')
        .ok_or_else(|| format!("Dirección inválida: {}", q))?;
    let byte: u8 = byte
        .parse()
        .map_err(|_| format!("Byte inválido: {}", q))?;
    let bit: u8 = bit
        .parse()
        .map_err(|_| format!("Bit inválido: {}", q))?;
    if byte > 3 || bit > 7 {
        return Err(format!("Dirección fuera de QB0..QB3: {}", q));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload_with(key: &str, value: &str) -> Value {
        let mut value_json = OutputMap::defaults().to_json();
        value_json["outputs"][key] = json!(value);
        value_json
    }

    #[test]
    fn defaults_preload_current_outputs() {
        let map = OutputMap::defaults();
        assert_eq!(map.q("KM2_120").unwrap(), "Q0.0");
        assert_eq!(map.q("KM1_220").unwrap(), "Q0.1");
        assert_eq!(map.q("RANGE_1A").unwrap(), "Q0.3");
        assert_eq!(map.q("FAN").unwrap(), "Q0.5");
        assert_eq!(map.q("HEATERS").unwrap(), "Q2.0");
        assert_eq!(map.validate(), Ok(()));
    }

    #[test]
    fn parses_stored_map_and_preserves_defaults_for_missing_keys() {
        let stored = json!({ "outputs": { "FAN": "Q1.2" } }).to_string();
        let cfg = json!({ "output_map": stored });
        let map = OutputMap::from_config(&cfg);
        assert_eq!(map.q("FAN").unwrap(), "Q1.2");
        assert_eq!(map.q("LIGHT").unwrap(), "Q0.6");
    }

    #[test]
    fn rejects_duplicate_output_bits() {
        let payload = payload_with("FAN", "Q0.6");
        let err = OutputMap::validate_payload(&payload).unwrap_err();
        assert!(err.contains("duplicada"), "{}", err);
    }

    #[test]
    fn rejects_invalid_output_address() {
        let payload = payload_with("FAN", "Q4.0");
        let err = OutputMap::validate_payload(&payload).unwrap_err();
        assert!(err.contains("fuera") || err.contains("inválida"), "{}", err);
    }

    #[test]
    fn rejects_missing_logical_output() {
        let mut payload = OutputMap::defaults().to_json();
        payload["outputs"].as_object_mut().unwrap().remove("FAN");
        let err = OutputMap::validate_payload(&payload).unwrap_err();
        assert!(err.contains("FAN"), "{}", err);
    }

    #[test]
    fn all_addresses_are_unique() {
        let mut map = OutputMap::defaults();
        assert_eq!(map.all_addresses().len(), LOGICAL_OUTPUTS.len());
        map.outputs.insert("FAN".to_string(), "Q0.6".to_string());
        assert_eq!(map.all_addresses().len(), LOGICAL_OUTPUTS.len() - 1);
    }

    #[test]
    fn storage_round_trip_preserves_map() {
        let payload = payload_with("DOOR", "Q2.1");
        let map = OutputMap::validate_payload(&payload).unwrap();
        let cfg = json!({ "output_map": map.storage_json() });
        let back = OutputMap::from_config(&cfg);
        assert_eq!(back.q("DOOR").unwrap(), "Q2.1");
        assert_eq!(back.q("DRAIN").unwrap(), "Q1.1");
    }
}
