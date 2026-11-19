//! Hipot 3865 por RS-232 (crate `serialport`).
//!
//! Replica `App/python/hipot_manager.py`: 8N1, comandos ASCII terminados en
//! CRLF, lectura de respuesta y parseo de mediciones. Incluye modo simulación.

use serde_json::{json, Map, Value};
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::io::{Read, Write};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

type OpenPort = (String, u32, Box<dyn SerialPort>);

static PORT: OnceLock<Mutex<Option<OpenPort>>> = OnceLock::new();

fn port_slot() -> &'static Mutex<Option<OpenPort>> {
    PORT.get_or_init(|| Mutex::new(None))
}

#[derive(Clone, Debug)]
pub struct HipotConfig {
    pub port: String,
    pub baud: u32,
}

pub fn config_from(cfg: &Value) -> HipotConfig {
    let s = |key: &str, default: &str| {
        cfg.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    };
    HipotConfig {
        port: s("hipot_port", "COM3"),
        baud: s("hipot_baud", "9600").parse().unwrap_or(9600),
    }
}

pub fn config_to_json(cfg: &HipotConfig) -> Value {
    json!({
        "port": cfg.port,
        "baud": cfg.baud,
    })
}

/// Lista los puertos serie disponibles.
pub fn list_ports() -> Vec<String> {
    serialport::available_ports()
        .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
        .unwrap_or_default()
}

fn open_port(cfg: &HipotConfig) -> Result<Box<dyn SerialPort>, String> {
    serialport::new(&cfg.port, cfg.baud)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(1000))
        .open()
        .map_err(|e| format!("No se pudo abrir {}: {}", cfg.port, e))
}

/// Abre el puerto si no está abierto o si cambió la configuración.
pub fn ensure_open(cfg: &HipotConfig) -> Result<(), String> {
    let mut guard = port_slot()
        .lock()
        .map_err(|_| "puerto bloqueado".to_string())?;
    if let Some((port, baud, _)) = guard.as_ref() {
        if port == &cfg.port && *baud == cfg.baud {
            return Ok(());
        }
    }
    *guard = None;
    let handle = open_port(cfg)?;
    *guard = Some((cfg.port.clone(), cfg.baud, handle));
    Ok(())
}

pub fn close() {
    if let Ok(mut guard) = port_slot().lock() {
        *guard = None;
    }
}

/// Envía un comando y devuelve la respuesta cruda + mediciones parseadas.
pub fn command(cfg: &HipotConfig, command: &str) -> Result<Value, String> {
    ensure_open(cfg)?;
    let mut guard = port_slot()
        .lock()
        .map_err(|_| "puerto bloqueado".to_string())?;
    let (_, _, port) = guard.as_mut().ok_or_else(|| "puerto no abierto".to_string())?;

    port.write_all(format!("{}\r\n", command.trim()).as_bytes())
        .map_err(|e| format!("Error escribiendo: {}", e))?;
    let _ = port.flush();

    std::thread::sleep(Duration::from_millis(450));
    let mut buf = [0u8; 1024];
    let response = match port.read(&mut buf) {
        Ok(0) => String::new(),
        Ok(n) => String::from_utf8_lossy(&buf[..n]).trim().to_string(),
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => String::new(),
        Err(e) => return Err(format!("Error leyendo: {}", e)),
    };

    Ok(json!({
        "response": response,
        "parsed": parse_measurement(&response),
    }))
}

/// Comando de alto nivel: START / STOP / STATUS.
pub fn test_action(cfg: &HipotConfig, action: &str) -> Result<Value, String> {
    let action = action.trim().to_uppercase();
    let cmd = match action.as_str() {
        "START" => "START",
        "STOP" => "STOP",
        "ABORT" => "ABORT",
        _ => "STATUS?",
    };
    let mut value = command(cfg, cmd)?;
    if let Some(obj) = value.as_object_mut() {
        obj.insert("action".to_string(), json!(action));
    }
    Ok(value)
}

/// Consulta de conexión (abre el puerto).
pub fn check_connection(cfg: &HipotConfig) -> Result<(), String> {
    ensure_open(cfg)
}

/// Parseo de respuestas tipo `V:120.5 I:1.23 R:456.7 PASS`.
pub fn parse_measurement(data: &str) -> Value {
    let mut map = Map::new();
    for part in data.split_whitespace() {
        if let Some(raw) = part.strip_prefix("V:") {
            if let Ok(v) = raw.parse::<f64>() {
                map.insert("voltage".to_string(), json!(v));
            }
        } else if let Some(raw) = part.strip_prefix("I:") {
            if let Ok(v) = raw.parse::<f64>() {
                map.insert("current".to_string(), json!(v));
            }
        } else if let Some(raw) = part.strip_prefix("R:") {
            if let Ok(v) = raw.parse::<f64>() {
                map.insert("resistance".to_string(), json!(v));
            }
        } else if part == "PASS" {
            map.insert("pass".to_string(), json!(true));
        } else if part == "FAIL" {
            map.insert("pass".to_string(), json!(false));
        }
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_measurement() {
        let v = parse_measurement("V:120.5 I:1.23 R:456.7 PASS");
        assert_eq!(v["voltage"], json!(120.5));
        assert_eq!(v["current"], json!(1.23));
        assert_eq!(v["resistance"], json!(456.7));
        assert_eq!(v["pass"], json!(true));

        let f = parse_measurement("V:208 I:0.5 R:100 FAIL");
        assert_eq!(f["pass"], json!(false));
    }

    #[test]
    fn config_defaults() {
        let cfg = config_from(&json!({}));
        assert_eq!(cfg.port, "COM3");
        assert_eq!(cfg.baud, 9600);
    }
}
