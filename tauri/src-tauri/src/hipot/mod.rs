//! Hipot 3865 por USB/RS-232 (crate `serialport`).
//!
//! Protocolo real según el Apéndice D del manual del 3865:
//! - 38400 baud, 8N1, sin handshaking.
//! - Comandos ASCII terminados en NL (0x0A).
//! - El equipo responde ACK (0x06) o NAK (0x15).
//! - Consultas principales: `*IDN?`, `TD?`, `RD n?`, `LF?`, `ST?`.
//! - Ejecución: `TEST` inicia, `RESET` detiene/aborta.
//! - Configuración de paso: `ADD ACW,...` o comandos `EV`, `EH`, etc.

use serde_json::{json, Map, Value};
use serialport::{ClearBuffer, DataBits, Parity, SerialPort, StopBits};
use std::io::{Read, Write};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

type OpenPort = (String, u32, Box<dyn SerialPort>);

static PORT: OnceLock<Mutex<Option<OpenPort>>> = OnceLock::new();

const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
const TEMP_FILE: u32 = 2000;
const TEMP_NAME: &str = "HIPOTAPP";

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
        baud: s("hipot_baud", "38400").parse().unwrap_or(38400),
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

/// Consulta de conexión (abre el puerto).
pub fn check_connection(cfg: &HipotConfig) -> Result<(), String> {
    ensure_open(cfg)
}

fn is_query(command: &str) -> bool {
    command.trim().ends_with('?')
}

/// Envía un comando y devuelve la respuesta cruda + datos parseados.
pub fn command(cfg: &HipotConfig, command: &str) -> Result<Value, String> {
    let response = send_raw(cfg, command)?;
    let parsed = parse_response(command, &response);
    Ok(json!({
        "command": command,
        "response": response,
        "parsed": parsed,
    }))
}

fn parse_response(command: &str, response: &str) -> Value {
    let upper = command.trim().to_uppercase();
    if upper.starts_with("TD?") || (upper.starts_with("RD ") && upper.ends_with('?')) {
        parse_td(response)
    } else if upper.starts_with("*IDN?") {
        parse_idn(response)
    } else {
        json!({})
    }
}

fn send_raw(cfg: &HipotConfig, command: &str) -> Result<String, String> {
    let started = Instant::now();
    let result = send_raw_inner(cfg, command);
    let elapsed = started.elapsed().as_millis() as i64;
    match &result {
        Ok(response) => log_tx_rx(command, response, None, elapsed),
        Err(error) => log_tx_rx(command, "", Some(error), elapsed),
    }
    result
}

fn send_raw_inner(cfg: &HipotConfig, command: &str) -> Result<String, String> {
    ensure_open(cfg)?;
    let mut guard = port_slot()
        .lock()
        .map_err(|_| "puerto bloqueado".to_string())?;
    let (_, _, port) = guard.as_mut().ok_or_else(|| "puerto no abierto".to_string())?;

    // Descarta datos viejos para no confundir la respuesta de este comando.
    let _ = port.clear(ClearBuffer::Input);
    port.write_all(format!("{}\n", command.trim()).as_bytes())
        .map_err(|e| format!("Error escribiendo: {}", e))?;
    let _ = port.flush();

    let query = is_query(command);
    let mut raw: Vec<u8> = Vec::new();
    let mut buf = [0u8; 256];
    loop {
        match port.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                raw.extend_from_slice(&buf[..n]);
                if raw.contains(&b'\n') {
                    break;
                }
                // Los comandos que no son consulta solo devuelven ACK/NAK.
                if !query && (raw[0] == ACK || raw[0] == NAK) {
                    break;
                }
                if raw.len() >= 4096 {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
            Err(e) => return Err(format!("Error leyendo: {}", e)),
        }
    }

    if raw.is_empty() {
        return Err("Sin respuesta del Hipot".to_string());
    }

    if raw[0] == NAK {
        let text = String::from_utf8_lossy(&raw[1..])
            .replace('\r', "")
            .replace('\n', "")
            .trim()
            .to_string();
        return Err(if text.is_empty() {
            "NAK del Hipot".to_string()
        } else {
            format!("NAK del Hipot: {}", text)
        });
    }

    let saw_ack = raw[0] == ACK;
    if saw_ack {
        raw.remove(0);
    }
    let response = String::from_utf8_lossy(&raw)
        .replace('\r', "")
        .replace('\n', "")
        .trim()
        .to_string();

    if response.is_empty() {
        if saw_ack {
            Ok("ACK".to_string())
        } else {
            Err("Respuesta vacía del Hipot".to_string())
        }
    } else {
        Ok(response)
    }
}

fn log_tx_rx(command: &str, response: &str, error: Option<&str>, duration_ms: i64) {
    let detail = if let Some(err) = error {
        format!("TX: {} | Error: {}", command, err)
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

/// Comando de alto nivel: START / STOP / STATUS / PROBE, o comando crudo.
pub fn test_action(cfg: &HipotConfig, action: &str) -> Result<Value, String> {
    let action = action.trim().to_uppercase();
    let cmd: &str = match action.as_str() {
        "START" | "TEST" => "TEST",
        "STOP" | "ABORT" | "RESET" => "RESET",
        "STATUS" | "TD" | "MEASURE" | "RESULT" => "TD?",
        "IDN" | "PROBE" => "*IDN?",
        _ => action.as_str(),
    };
    let mut value = command(cfg, cmd)?;
    if let Some(obj) = value.as_object_mut() {
        obj.insert("action".to_string(), json!(action));
    }
    Ok(value)
}

/// Prepara un paso ACW en un archivo temporal y lo deja seleccionado.
pub fn prepare_test(
    cfg: &HipotConfig,
    voltage: f64,
    current_max: f64,
    time: f64,
) -> Result<Value, String> {
    if voltage <= 0.0 {
        return Err("Hipot: voltaje no configurado en la receta".to_string());
    }
    if current_max <= 0.0 {
        return Err("Hipot: límite de corriente no configurado en la receta".to_string());
    }
    if time <= 0.0 {
        return Err("Hipot: tiempo no configurado en la receta".to_string());
    }

    let file = ensure_temp_file(cfg)?;
    let initial_steps = query_step_count(cfg);
    let attempts = initial_steps.max(1) as usize;
    for _ in 0..attempts {
        // Si no hay paso, el equipo responde NAK; se ignora.
        let _ = send_raw(cfg, "SD 1");
    }
    let remaining = query_step_count(cfg);
    if remaining > 0 {
        return Err(format!(
            "Hipot: no se pudieron borrar los pasos anteriores (quedan {})",
            remaining
        ));
    }

    let add = format!(
        "ADD ACW,{},{},0,0.1,{},0.0,5,OFF,60,OFF,1.50,0.00,0.00",
        fmt_num(voltage),
        fmt_num(current_max),
        fmt_num(time)
    );
    send_raw(cfg, &add)?;
    send_raw(cfg, "SS 1")?;

    Ok(json!({
        "file": file,
        "steps_deleted": initial_steps,
        "add": add,
        "step": 1,
    }))
}

/// Ejecuta un ciclo completo: prepara el paso, inicia TEST y espera PASS/FAIL.
pub fn run_test(
    cfg: &HipotConfig,
    voltage: f64,
    current_max: f64,
    time: f64,
) -> Result<Value, String> {
    let _ = command(cfg, "RESET");
    let prep = prepare_test(cfg, voltage, current_max, time)?;
    let start = command(cfg, "TEST")?;
    let timeout = Duration::from_secs((time.max(1.0).ceil() as u64).saturating_add(30));
    let started = Instant::now();
    let mut last = json!({});
    let mut saw_progress = false;

    loop {
        std::thread::sleep(Duration::from_millis(500));
        let mut status = None;
        for _ in 0..3 {
            match command(cfg, "TD?") {
                Ok(value) => {
                    status = Some(value);
                    break;
                }
                Err(_) => std::thread::sleep(Duration::from_millis(200)),
            }
        }
        let status = match status {
            Some(value) => value,
            None => {
                let _ = command(cfg, "RESET");
                return Err("Hipot: no responde a TD? durante la prueba".to_string());
            }
        };
        let parsed = status
            .get("parsed")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let status_text = parsed
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_uppercase();
        let in_progress = matches!(
            status_text.as_str(),
            "DWELL" | "RAMP" | "DELAY" | "TEST" | "TESTING"
        ) || status_text.contains("PROCESS");
        if in_progress {
            saw_progress = true;
        }

        if parsed.get("pass").is_some() {
            // Evita tomar como resultado un PASS viejo del ciclo anterior.
            if started.elapsed() >= Duration::from_secs(1) || saw_progress {
                last = parsed;
                break;
            }
        }

        last = parsed;
        if started.elapsed() > timeout {
            let _ = command(cfg, "RESET");
            return Err("Hipot: timeout esperando PASS/FAIL".to_string());
        }
    }

    let _ = command(cfg, "RESET");

    if last.get("pass").is_none() {
        return Err("Hipot: sin resultado PASS/FAIL".to_string());
    }

    Ok(json!({
        "prep": prep,
        "start": start,
        "result": last,
    }))
}

fn ensure_temp_file(cfg: &HipotConfig) -> Result<u32, String> {
    for n in (1998..=TEMP_FILE).rev() {
        match file_name(cfg, n) {
            Some(name) if name.to_uppercase().contains(TEMP_NAME) => {
                send_raw(cfg, &format!("FL {}", n))?;
                return Ok(n);
            }
            Some(_) => continue,
            None => {
                if send_raw(cfg, &format!("FN {},{}", n, TEMP_NAME)).is_ok() {
                    send_raw(cfg, &format!("FL {}", n))?;
                    return Ok(n);
                }
            }
        }
    }
    Err("Hipot: no se pudo preparar el archivo temporal HIPOTAPP (revise conexión, memoria y nivel de acceso)".to_string())
}

fn file_name(cfg: &HipotConfig, n: u32) -> Option<String> {
    let value = command(cfg, &format!("LF {}?", n)).ok()?;
    let response = value
        .get("response")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if response.is_empty() || response.eq_ignore_ascii_case("ACK") {
        None
    } else {
        Some(response)
    }
}

fn query_step_count(cfg: &HipotConfig) -> i64 {
    match command(cfg, "ST?") {
        Ok(value) => {
            let response = value
                .get("response")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            parse_first_int(response)
        }
        Err(_) => 0,
    }
}

fn parse_first_int(text: &str) -> i64 {
    let mut digits = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if !digits.is_empty() {
            break;
        }
    }
    digits.parse().unwrap_or(0)
}

fn fmt_num(value: f64) -> String {
    let mut text = format!("{:.3}", value);
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

/// Parsea la respuesta de `TD?` / `RD n?`:
/// `<paso>, <tipo>, <estado>, <medidor1>, <medidor2>, <medidor3>`.
pub fn parse_td(data: &str) -> Value {
    let data = data.trim();
    if data.is_empty() || data.eq_ignore_ascii_case("ACK") {
        return json!({});
    }
    let parts: Vec<&str> = data.split(',').map(|s| s.trim()).collect();
    if parts.len() < 6 {
        return json!({ "raw": data });
    }

    let step = parts[0].parse::<i64>().ok();
    let test_type = parts[1].to_uppercase();
    let status = parts[2].to_uppercase();
    let m1 = parts[3].parse::<f64>().ok();
    let m2 = parts[4].parse::<f64>().ok();
    let m3 = parts[5].parse::<f64>().ok();

    let in_progress = matches!(
        status.as_str(),
        "" | "DWELL" | "RAMP" | "DELAY" | "TEST" | "TESTING"
    ) || status.contains("PROCESS");
    let pass = if status == "PASS" {
        Some(true)
    } else if !in_progress {
        Some(false)
    } else {
        None
    };

    let mut map = Map::new();
    map.insert("raw".to_string(), json!(data));
    if let Some(step) = step {
        map.insert("step".to_string(), json!(step));
    }
    map.insert("test_type".to_string(), json!(test_type));
    map.insert("status".to_string(), json!(status));
    if let Some(pass) = pass {
        map.insert("pass".to_string(), json!(pass));
    }

    match test_type.as_str() {
        "ACW" => {
            if let Some(v) = m1 {
                map.insert("voltage".to_string(), json!(v * 1000.0));
            }
            if let Some(i) = m2 {
                map.insert("current".to_string(), json!(i));
            }
            if let Some(t) = m3 {
                map.insert("duration".to_string(), json!(t));
            }
        }
        "DCW" => {
            if let Some(v) = m1 {
                map.insert("voltage".to_string(), json!(v * 1000.0));
            }
            if let Some(i) = m2 {
                map.insert("current".to_string(), json!(i / 1000.0));
            }
            if let Some(t) = m3 {
                map.insert("duration".to_string(), json!(t));
            }
        }
        "IR" => {
            if let Some(v) = m1 {
                map.insert("voltage".to_string(), json!(v));
            }
            if let Some(r) = m2 {
                map.insert("resistance".to_string(), json!(r));
            }
            if let Some(t) = m3 {
                map.insert("duration".to_string(), json!(t));
            }
        }
        _ => {
            if let Some(v) = m1 {
                map.insert("meter1".to_string(), json!(v));
            }
            if let Some(v) = m2 {
                map.insert("meter2".to_string(), json!(v));
            }
            if let Some(v) = m3 {
                map.insert("meter3".to_string(), json!(v));
            }
        }
    }

    Value::Object(map)
}

fn parse_idn(data: &str) -> Value {
    let parts: Vec<&str> = data.split(',').map(|s| s.trim()).collect();
    json!({
        "idn": data,
        "vendor": parts.first().copied().unwrap_or(""),
        "model": parts.get(1).copied().unwrap_or(""),
        "serial": parts.get(2).copied().unwrap_or(""),
        "firmware": parts.get(3).copied().unwrap_or(""),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults() {
        let cfg = config_from(&json!({}));
        assert_eq!(cfg.port, "COM3");
        assert_eq!(cfg.baud, 38400);
    }

    #[test]
    fn parses_td_pass() {
        let v = parse_td("1, ACW, PASS, 0.30, 0.296, 0.5");
        assert_eq!(v["voltage"], json!(300.0));
        assert_eq!(v["current"], json!(0.296));
        assert_eq!(v["duration"], json!(0.5));
        assert_eq!(v["pass"], json!(true));
    }

    #[test]
    fn parses_td_fail() {
        let v = parse_td("2, ACW, HI-LMT, 0.30, 0.192, 0.2");
        assert_eq!(v["pass"], json!(false));
    }

    #[test]
    fn parses_td_in_progress() {
        let v = parse_td("1, ACW, Dwell, 0.30, 0.296, 0.5");
        assert!(v.get("pass").is_none());
    }

    #[test]
    fn parses_td_dcw_and_ir() {
        let dcw = parse_td("1, DCW, PASS, 1.50, 2000, 0.5");
        assert_eq!(dcw["voltage"], json!(1500.0));
        assert_eq!(dcw["current"], json!(2.0));

        let ir = parse_td("1, IR, PASS, 500, 1.25, 0.5");
        assert_eq!(ir["voltage"], json!(500.0));
        assert_eq!(ir["resistance"], json!(1.25));
    }

    #[test]
    fn parses_idn() {
        let v = parse_idn("Associated Research Inc., 3865, 12345, 1.2.3");
        assert_eq!(v["model"], json!("3865"));
        assert_eq!(v["serial"], json!("12345"));
    }

    #[test]
    fn formats_numbers() {
        assert_eq!(fmt_num(500.0), "500");
        assert_eq!(fmt_num(0.005), "0.005");
        assert_eq!(fmt_num(60.0), "60");
        assert_eq!(fmt_num(1.5), "1.5");
    }

    #[test]
    fn parses_first_int() {
        assert_eq!(parse_first_int("3"), 3);
        assert_eq!(parse_first_int("Step Total: 2"), 2);
        assert_eq!(parse_first_int("ACK"), 0);
    }
}
