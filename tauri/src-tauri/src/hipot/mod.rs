//! Hipot 3865 por RS-232 (crate `serialport`).
//!
//! Protocolo según `docs/06-comunicaciones.md`: 8N1, comandos ASCII terminados
//! en CRLF, lectura de respuesta hasta CRLF y parseo de mediciones.

use serde_json::{json, Map, Value};
use serialport::{ClearBuffer, DataBits, Parity, SerialPort, StopBits};
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
/// Cada TX/RX queda registrado en app_logs (categoría HIPOT).
pub fn command(cfg: &HipotConfig, command: &str) -> Result<Value, String> {
    let started = std::time::Instant::now();
    let result = command_inner(cfg, command);
    let elapsed = started.elapsed().as_millis() as i64;
    match &result {
        Ok(value) => {
            let response = value.get("response").and_then(|v| v.as_str()).unwrap_or("");
            log_tx_rx(command, response, None, elapsed);
        }
        Err(error) => log_tx_rx(command, "", Some(error), elapsed),
    }
    result
}

fn command_inner(cfg: &HipotConfig, command: &str) -> Result<Value, String> {
    ensure_open(cfg)?;
    let mut guard = port_slot()
        .lock()
        .map_err(|_| "puerto bloqueado".to_string())?;
    let (_, _, port) = guard.as_mut().ok_or_else(|| "puerto no abierto".to_string())?;

    // Descarta datos viejos para no confundir la respuesta de este comando.
    let _ = port.clear(ClearBuffer::Input);
    port.write_all(format!("{}\r\n", command.trim()).as_bytes())
        .map_err(|e| format!("Error escribiendo: {}", e))?;
    let _ = port.flush();

    std::thread::sleep(Duration::from_millis(150));
    let mut response_bytes: Vec<u8> = Vec::new();
    let mut buf = [0u8; 256];
    loop {
        match port.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                response_bytes.extend_from_slice(&buf[..n]);
                // El equipo termina cada respuesta con CRLF (readline en el manual).
                if response_bytes.contains(&b'\n') || response_bytes.len() >= 4096 {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
            Err(e) => return Err(format!("Error leyendo: {}", e)),
        }
    }
    let response = String::from_utf8_lossy(&response_bytes).trim().to_string();

    Ok(json!({
        "response": response,
        "parsed": parse_measurement(&response),
    }))
}

fn log_tx_rx(command: &str, response: &str, error: Option<&str>, duration_ms: i64) {
    let detail = if let Some(err) = error {
        format!("TX: {} | Error: {}", command, err)
    } else if response.is_empty() {
        format!("TX: {} | RX: (sin respuesta)", command)
    } else {
        format!("TX: {} | RX: {}", command, response)
    };
    let _ = crate::db::repo::log_add(
        if error.is_some() { "ERROR" } else { "INFO" },
        "HIPOT",
        "tx_rx",
        Some(&detail),
        None,
        Some(command),
        Some(response),
        Some(duration_ms),
        error.is_none(),
        error,
    );
}

/// Detecta respuestas explícitas de error del equipo.
fn has_error(response: &str) -> bool {
    let upper = response.trim().to_uppercase();
    upper == "?"
        || upper.contains("ERROR")
        || upper.contains("ERR")
        || upper.contains("INVALID")
        || upper.contains("NO RECONOCIDO")
}

/// Aplica los parámetros de la receta antes de START.
/// Los valores en 0 o negativos se omiten.
pub fn apply_settings(
    cfg: &HipotConfig,
    voltage: f64,
    current_max: f64,
    time: f64,
) -> Result<Vec<Value>, String> {
    let mut responses = Vec::new();
    if voltage > 0.0 {
        let value = command(cfg, &format!("VOLTAGE:{}", voltage))?;
        let response = value.get("response").and_then(|v| v.as_str()).unwrap_or("");
        if has_error(response) {
            return Err(format!("VOLTAGE rechazado por el Hipot: {}", response));
        }
        responses.push(value);
    }
    if current_max > 0.0 {
        let value = command(cfg, &format!("CURRENT_MAX:{}", current_max))?;
        let response = value.get("response").and_then(|v| v.as_str()).unwrap_or("");
        if has_error(response) {
            return Err(format!("CURRENT_MAX rechazado por el Hipot: {}", response));
        }
        responses.push(value);
    }
    if time > 0.0 {
        let value = command(cfg, &format!("TIME:{}", time))?;
        let response = value.get("response").and_then(|v| v.as_str()).unwrap_or("");
        if has_error(response) {
            return Err(format!("TIME rechazado por el Hipot: {}", response));
        }
        responses.push(value);
    }
    Ok(responses)
}

/// Comando de alto nivel: START / STOP / ABORT / STATUS / MEASURE / RESULT.
pub fn test_action(cfg: &HipotConfig, action: &str) -> Result<Value, String> {
    let action = action.trim().to_uppercase();
    let cmd = match action.as_str() {
        "START" => "START",
        "STOP" => "STOP",
        "ABORT" => "ABORT",
        "MEASURE" | "MEASURE?" => "MEASURE?",
        "RESULT" | "RESULT?" => "RESULT?",
        _ => "STATUS?",
    };
    let mut value = command(cfg, cmd)?;
    if cmd == "START" {
        let response = value.get("response").and_then(|v| v.as_str()).unwrap_or("");
        if has_error(response) {
            return Err(format!("START rechazado por el Hipot: {}", response));
        }
    }
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

        // El equipo puede incluir la unidad KΩ; debe ignorarse sin afectar el parseo.
        let k = parse_measurement("V:120.5 I:1.23 R:456.7 KΩ PASS");
        assert_eq!(k["voltage"], json!(120.5));
        assert_eq!(k["resistance"], json!(456.7));
        assert_eq!(k["pass"], json!(true));
    }

    #[test]
    fn config_defaults() {
        let cfg = config_from(&json!({}));
        assert_eq!(cfg.port, "COM3");
        assert_eq!(cfg.baud, 9600);
    }

    #[test]
    fn detects_explicit_errors() {
        assert!(has_error("ERROR"));
        assert!(has_error("ERR 01"));
        assert!(has_error("?"));
        assert!(!has_error(""));
        assert!(!has_error("OK"));
        assert!(!has_error("V:120.5 I:1.23 R:456.7 KΩ PASS"));
    }
}
