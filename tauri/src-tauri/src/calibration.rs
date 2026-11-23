//! Calibración de canales analógicos del PLC puente DB100.
//!
//! La calibración es de dos puntos por canal físico. Un canal sin calibrar no
//! devuelve un valor de ingeniería: la UI y el motor deben tratarlo como
//! "sin calibrar", nunca como cero válido.

use serde_json::{json, Map, Value};

pub const ADC_FULL_SCALE: f64 = 27648.0;
pub const CHANNEL_NAMES: [&str; 4] = ["IA1", "IA2", "IA3", "IA4"];
pub const DEFAULT_FULL_SCALE: [f64; 4] = [1.0, 10.0, 20.0, 220.0];

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelCalibration {
    pub channel: String,
    pub enabled: bool,
    pub raw_low: f64,
    pub eng_low: f64,
    pub raw_high: f64,
    pub eng_high: f64,
    pub low_set: bool,
    pub high_set: bool,
}

impl ChannelCalibration {
    pub fn disabled(channel: &str) -> Self {
        Self {
            channel: channel.to_string(),
            enabled: false,
            raw_low: 0.0,
            eng_low: 0.0,
            raw_high: ADC_FULL_SCALE,
            eng_high: 0.0,
            low_set: false,
            high_set: false,
        }
    }

    pub fn new(
        channel: &str,
        raw_low: f64,
        eng_low: f64,
        raw_high: f64,
        eng_high: f64,
    ) -> Self {
        Self {
            channel: channel.to_string(),
            enabled: true,
            raw_low,
            eng_low,
            raw_high,
            eng_high,
            low_set: true,
            high_set: true,
        }
    }

    pub fn unit(&self) -> &'static str {
        unit_for(&self.channel)
    }

    pub fn is_valid(&self) -> bool {
        self.enabled
            && self.low_set
            && self.high_set
            && self.raw_low.is_finite()
            && self.raw_high.is_finite()
            && self.eng_low.is_finite()
            && self.eng_high.is_finite()
            && self.raw_low >= 0.0
            && self.raw_high <= ADC_FULL_SCALE
            && self.raw_high > self.raw_low
            && (self.eng_high - self.eng_low).abs() > f64::EPSILON
    }

    pub fn to_eng(&self, raw: f64) -> Option<f64> {
        if !self.is_valid() || !raw.is_finite() {
            return None;
        }
        let span = self.raw_high - self.raw_low;
        Some(self.eng_low + (raw - self.raw_low) * (self.eng_high - self.eng_low) / span)
    }

    pub fn to_json(&self) -> Value {
        json!({
            "enabled": self.enabled,
            "raw_low": self.raw_low,
            "eng_low": self.eng_low,
            "raw_high": self.raw_high,
            "eng_high": self.eng_high,
            "low_set": self.low_set,
            "high_set": self.high_set,
            "unit": self.unit(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalogCalibration {
    pub channels: Vec<ChannelCalibration>,
}

impl Default for AnalogCalibration {
    fn default() -> Self {
        Self::defaults()
    }
}

impl AnalogCalibration {
    pub fn defaults() -> Self {
        Self {
            channels: CHANNEL_NAMES
                .iter()
                .map(|name| ChannelCalibration::disabled(name))
                .collect(),
        }
    }

    pub fn from_config(config: &Value) -> Self {
        let stored = match config.get("analog_calibration") {
            Some(Value::String(text)) => serde_json::from_str::<Value>(text).ok(),
            Some(value) => Some(value.clone()),
            None => None,
        };
        stored
            .as_ref()
            .and_then(|value| Self::validate_payload(value).ok())
            .unwrap_or_else(Self::defaults)
    }

    pub fn get(&self, channel: &str) -> Option<&ChannelCalibration> {
        self.channels.iter().find(|ch| ch.channel == channel)
    }

    pub fn to_eng(&self, channel: &str, raw: u16) -> Option<f64> {
        self.get(channel).and_then(|ch| ch.to_eng(raw as f64))
    }

    pub fn validate_payload(payload: &Value) -> Result<Self, String> {
        let source = payload
            .get("channels")
            .unwrap_or(payload)
            .as_object()
            .ok_or_else(|| "La calibración debe tener un objeto 'channels'".to_string())?;

        if let Some(unknown) = source
            .keys()
            .find(|name| !CHANNEL_NAMES.contains(&name.as_str()))
        {
            return Err(format!("Canal desconocido: {}", unknown));
        }

        let mut channels = Vec::with_capacity(CHANNEL_NAMES.len());
        for name in CHANNEL_NAMES {
            let entry = match source.get(name) {
                Some(value) => value,
                None => {
                    channels.push(ChannelCalibration::disabled(name));
                    continue;
                }
            };
            let obj = entry
                .as_object()
                .ok_or_else(|| format!("{} debe ser un objeto", name))?;
            let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            if !enabled {
                channels.push(ChannelCalibration::disabled(name));
                continue;
            }
            let raw_low = number_field(obj, name, "raw_low")?;
            let eng_low = number_field(obj, name, "eng_low")?;
            let raw_high = number_field(obj, name, "raw_high")?;
            let eng_high = number_field(obj, name, "eng_high")?;
            let low_set = obj.get("low_set").and_then(|v| v.as_bool()).unwrap_or(false);
            let high_set = obj.get("high_set").and_then(|v| v.as_bool()).unwrap_or(false);
            let ch = ChannelCalibration {
                channel: name.to_string(),
                enabled: true,
                raw_low,
                eng_low,
                raw_high,
                eng_high,
                low_set,
                high_set,
            };
            if !ch.raw_low.is_finite()
                || !ch.raw_high.is_finite()
                || ch.raw_low < 0.0
                || ch.raw_high > ADC_FULL_SCALE
                || ch.raw_high <= ch.raw_low
            {
                return Err(format!(
                    "{}: raw_high debe ser mayor que raw_low y estar entre 0 y {}",
                    name, ADC_FULL_SCALE
                ));
            }
            if (ch.eng_high - ch.eng_low).abs() <= f64::EPSILON {
                return Err(format!("{}: los valores de ingeniería deben ser distintos", name));
            }
            if !ch.low_set || !ch.high_set {
                return Err(format!(
                    "{}: debe capturar el punto bajo y el punto alto antes de guardar",
                    name
                ));
            }
            channels.push(ch);
        }
        Ok(Self { channels })
    }

    pub fn to_json(&self) -> Value {
        let mut channels = Map::new();
        for ch in &self.channels {
            channels.insert(ch.channel.clone(), ch.to_json());
        }
        json!({ "channels": Value::Object(channels) })
    }

    pub fn storage_json(&self) -> String {
        self.to_json().to_string()
    }
}

pub fn next_version(current: &str) -> u64 {
    current
        .trim()
        .parse::<u64>()
        .unwrap_or(0)
        .saturating_add(1)
}

pub fn unit_for(channel: &str) -> &'static str {
    match channel {
        "IA4" => "V",
        _ => "A",
    }
}

fn number_field(
    obj: &Map<String, Value>,
    channel: &str,
    field: &str,
) -> Result<f64, String> {
    let value = obj
        .get(field)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("{}: falta el valor numérico '{}'", channel, field))?;
    if !value.is_finite() {
        return Err(format!("{}: '{}' no es finito", channel, field));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(ch: &str, raw_low: f64, eng_low: f64, raw_high: f64, eng_high: f64) -> ChannelCalibration {
        ChannelCalibration::new(ch, raw_low, eng_low, raw_high, eng_high)
    }

    #[test]
    fn uncalibrated_channel_returns_none() {
        let ch = ChannelCalibration::disabled("IA1");
        assert!(!ch.is_valid());
        assert_eq!(ch.to_eng(12000.0), None);
    }

    #[test]
    fn one_point_capture_is_not_valid() {
        let mut ch = ChannelCalibration::disabled("IA1");
        ch.enabled = true;
        ch.raw_low = 5530.0;
        ch.raw_high = 27648.0;
        ch.eng_low = 0.0;
        ch.eng_high = 1.0;
        ch.low_set = true;
        ch.high_set = false;
        assert!(!ch.is_valid());
        assert_eq!(ch.to_eng(27648.0), None);
    }

    #[test]
    fn validate_payload_rejects_channel_without_both_points() {
        let payload = json!({
            "channels": {
                "IA1": {
                    "enabled": true,
                    "raw_low": 0.0,
                    "eng_low": 0.0,
                    "raw_high": 27648.0,
                    "eng_high": 1.0,
                    "low_set": true,
                    "high_set": false
                }
            }
        });
        let err = AnalogCalibration::validate_payload(&payload).unwrap_err();
        assert!(err.contains("punto"), "{}", err);
    }

    #[test]
    fn two_point_calibration_converts_raw_to_engineering() {
        let ch = channel("IA1", 5530.0, 0.0, 27648.0, 1.0);
        assert!((ch.to_eng(5530.0).unwrap() - 0.0).abs() < 1e-9);
        assert!((ch.to_eng(27648.0).unwrap() - 1.0).abs() < 1e-9);
        assert!((ch.to_eng(16589.0).unwrap() - 0.5).abs() < 1e-3);
    }

    #[test]
    fn invalid_calibration_is_not_usable() {
        let ch = channel("IA2", 1000.0, 0.0, 1000.0, 10.0);
        assert!(!ch.is_valid());
        assert_eq!(ch.to_eng(1000.0), None);
    }

    #[test]
    fn defaults_are_uncalibrated_with_expected_units() {
        let cal = AnalogCalibration::defaults();
        assert_eq!(cal.channels.len(), 4);
        for name in CHANNEL_NAMES {
            let ch = cal.get(name).unwrap();
            assert!(!ch.is_valid());
            assert_eq!(ch.unit(), if name == "IA4" { "V" } else { "A" });
        }
        assert_eq!(cal.to_eng("IA4", 1000), None);
    }

    #[test]
    fn parses_calibration_from_config_json() {
        let stored = json!({
            "channels": {
                "IA2": {
                    "enabled": true,
                    "raw_low": 5530.0,
                    "eng_low": 0.0,
                    "raw_high": 27648.0,
                    "eng_high": 10.0,
                    "low_set": true,
                    "high_set": true
                }
            }
        })
        .to_string();
        let cfg = json!({ "analog_calibration": stored });
        let cal = AnalogCalibration::from_config(&cfg);
        let ch = cal.get("IA2").unwrap();
        assert!(ch.is_valid());
        assert!((cal.to_eng("IA2", 16589).unwrap() - 5.0).abs() < 0.01);
        assert_eq!(cal.to_eng("IA1", 16589), None);
    }

    #[test]
    fn validate_payload_accepts_partial_calibration_and_fills_all_channels() {
        let payload = json!({
            "channels": {
                "IA4": {
                    "enabled": true,
                    "raw_low": 0.0,
                    "eng_low": 0.0,
                    "raw_high": 27648.0,
                    "eng_high": 220.0,
                    "low_set": true,
                    "high_set": true
                }
            }
        });
        let cal = AnalogCalibration::validate_payload(&payload).unwrap();
        assert_eq!(cal.channels.len(), 4);
        assert!(cal.get("IA4").unwrap().is_valid());
        assert!(!cal.get("IA1").unwrap().is_valid());
        assert!((cal.to_eng("IA4", 13824).unwrap() - 110.0).abs() < 0.01);
    }

    #[test]
    fn validate_payload_rejects_inverted_raw_points() {
        let payload = json!({
            "channels": {
                "IA1": {
                    "enabled": true,
                    "raw_low": 20000.0,
                    "eng_low": 0.0,
                    "raw_high": 10000.0,
                    "eng_high": 1.0
                }
            }
        });
        let err = AnalogCalibration::validate_payload(&payload).unwrap_err();
        assert!(err.contains("raw_high"), "{}", err);
    }

    #[test]
    fn validate_payload_rejects_out_of_range_raw() {
        let payload = json!({
            "channels": {
                "IA1": {
                    "enabled": true,
                    "raw_low": 0.0,
                    "eng_low": 0.0,
                    "raw_high": 40000.0,
                    "eng_high": 1.0
                }
            }
        });
        let err = AnalogCalibration::validate_payload(&payload).unwrap_err();
        assert!(err.contains("27648"), "{}", err);
    }

    #[test]
    fn version_increments_and_recovers_from_invalid_text() {
        assert_eq!(next_version("0"), 1);
        assert_eq!(next_version("7"), 8);
        assert_eq!(next_version("no-numero"), 1);
        assert_eq!(next_version(""), 1);
    }

    #[test]
    fn storage_round_trip_preserves_calibration() {
        let cal = AnalogCalibration {
            channels: vec![
                channel("IA1", 0.0, 0.0, 27648.0, 1.0),
                ChannelCalibration::disabled("IA2"),
                ChannelCalibration::disabled("IA3"),
                channel("IA4", 0.0, 0.0, 27648.0, 220.0),
            ],
        };
        let text = cal.storage_json();
        let cfg = json!({ "analog_calibration": text });
        let back = AnalogCalibration::from_config(&cfg);
        assert_eq!(back.get("IA1").unwrap().raw_high, 27648.0);
        assert!(back.get("IA1").unwrap().is_valid());
        assert!(!back.get("IA2").unwrap().is_valid());
        assert!((back.to_eng("IA4", 27648).unwrap() - 220.0).abs() < 1e-9);
    }
}
