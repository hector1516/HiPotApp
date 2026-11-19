//! PLC Siemens S7-1200 (DB1 + Q / I / IA) sobre el crate `s7`.
//!
//! Replica la lógica de `App/python/plc_manager.py`:
//! - Offsets de receta y resultados en DB1.
//! - Lectura/escritura de tags (BOOL / WORD / INT / REAL).
//! - Mapa de salidas Q, entradas I y analógicas IA.





use s7::client::Client;
use s7::tcp::Transport;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

pub type PlcClient = Client<Transport>;

// --- Offsets DB1 (doc 06) ---
pub const OFF_VOLTAGE: i32 = 0;
pub const COMP_OFFSETS: &[(&str, i32, i32, i32)] = &[
    ("FAN", 4, 8, 12),
    ("DOOR", 16, 20, 24),
    ("FRAME", 28, 32, 36),
    ("DRAIN", 40, 44, 48),
    ("DEFROST", 52, 56, 60),
    ("LIGHT", 64, 68, 72),
];
pub const OFF_START: i32 = 100;
pub const OFF_STOP: i32 = 101;
pub const OFF_COMPLETE: i32 = 102;
pub const OFF_PASS: i32 = 103;
pub const OFF_RUNNING: i32 = 104;
pub const OFF_MODE: i32 = 110;
pub const RES_OFFSETS: &[(&str, i32)] = &[
    ("FAN", 200),
    ("DOOR", 204),
    ("FRAME", 208),
    ("DRAIN", 212),
    ("DEFROST", 216),
    ("LIGHT", 220),
    ("TOTAL", 224),
];

// --- Mapa I/O ---
pub const Q_MAP: &[(&str, i32, i32)] = &[
    ("Q0.0", 0, 0), ("Q0.1", 0, 1), ("Q0.2", 0, 2), ("Q0.3", 0, 3),
    ("Q0.4", 0, 4), ("Q0.5", 0, 5), ("Q0.6", 0, 6), ("Q0.7", 0, 7),
    ("Q1.0", 1, 0), ("Q1.1", 1, 1), ("Q2.0", 2, 0), ("Q2.1", 2, 1),
    ("Q2.2", 2, 2),
];
pub const I_MAP: &[(&str, i32, i32)] = &[("I0.0", 0, 0), ("I0.1", 0, 1), ("I0.2", 0, 2)];
pub const IA_MAP: &[(&str, i32)] = &[("IA1", 0), ("IA2", 2), ("IA3", 4), ("IA4", 6)];

#[derive(Clone, Debug)]
pub struct PlcConfig {
    pub ip: String,
    pub rack: u16,
    pub slot: u16,
}

/// Lee la configuración desde el JSON de `get_config`.
pub fn config_from(cfg: &Value) -> PlcConfig {
    let s = |key: &str, default: &str| {
        cfg.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    };
    PlcConfig {
        ip: s("plc_ip", "192.168.1.10"),
        rack: s("plc_rack", "0").parse().unwrap_or(0),
        slot: s("plc_slot", "1").parse().unwrap_or(1),
    }
}

pub fn config_to_json(cfg: &PlcConfig) -> Value {
    json!({
        "ip": cfg.ip,
        "rack": cfg.rack,
        "slot": cfg.slot,
    })
}

/// Abre una conexión nueva. El llamador decide si la reutiliza para varias operaciones.
pub fn connect(cfg: &PlcConfig, timeout_ms: u64) -> Result<PlcClient, String> {
    let addr = cfg
        .ip
        .parse::<std::net::IpAddr>()
        .map_err(|_| format!("IP inválida: {}", cfg.ip))?;
    let mut opts = s7::tcp::Options::new(addr, cfg.rack, cfg.slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_millis(timeout_ms);
    opts.write_timeout = Duration::from_millis(timeout_ms);
    let transport = s7::tcp::Transport::connect(opts).map_err(|e| e.to_string())?;
    s7::client::Client::new(transport).map_err(|e| e.to_string())
}

// --- Primitivas DB ---

pub fn read_real(c: &mut PlcClient, db: i32, start: i32) -> Result<f64, String> {
    let mut buf = vec![0u8; 4];
    c.ag_read(db, start, 4, &mut buf).map_err(|e| e.to_string())?;
    Ok(f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as f64)
}

pub fn write_real(c: &mut PlcClient, db: i32, start: i32, value: f64) -> Result<(), String> {
    let mut buf = (value as f32).to_be_bytes().to_vec();
    c.ag_write(db, start, 4, &mut buf).map_err(|e| e.to_string())
}

pub fn read_word(c: &mut PlcClient, db: i32, start: i32) -> Result<u16, String> {
    let mut buf = vec![0u8; 2];
    c.ag_read(db, start, 2, &mut buf).map_err(|e| e.to_string())?;
    Ok(u16::from_be_bytes([buf[0], buf[1]]))
}

pub fn write_word(c: &mut PlcClient, db: i32, start: i32, value: u16) -> Result<(), String> {
    let mut buf = value.to_be_bytes().to_vec();
    c.ag_write(db, start, 2, &mut buf).map_err(|e| e.to_string())
}

pub fn read_bool(c: &mut PlcClient, db: i32, start: i32, bit: i32) -> Result<bool, String> {
    let mut buf = vec![0u8; 1];
    c.ag_read(db, start, 1, &mut buf).map_err(|e| e.to_string())?;
    Ok(((buf[0] >> bit) & 1) == 1)
}

pub fn write_bool(c: &mut PlcClient, db: i32, start: i32, value: bool, bit: i32) -> Result<(), String> {
    let mut buf = vec![0u8; 1];
    c.ag_read(db, start, 1, &mut buf).map_err(|e| e.to_string())?;
    if value {
        buf[0] |= 1 << bit;
    } else {
        buf[0] &= !(1 << bit);
    }
    c.ag_write(db, start, 1, &mut buf).map_err(|e| e.to_string())
}

// --- Tags ---

pub fn read_tag(
    c: &mut PlcClient,
    db: i32,
    offset: i32,
    data_type: &str,
    bit: i32,
) -> Result<Value, String> {
    match data_type.to_uppercase().as_str() {
        "BOOL" => Ok(json!(read_bool(c, db, offset, bit)?)),
        "WORD" | "INT" => Ok(json!(read_word(c, db, offset)?)),
        "REAL" => Ok(json!(read_real(c, db, offset)?)),
        other => Err(format!("Tipo no soportado: {}", other)),
    }
}

pub fn write_tag(
    c: &mut PlcClient,
    db: i32,
    offset: i32,
    data_type: &str,
    value: &Value,
    bit: i32,
) -> Result<(), String> {
    match data_type.to_uppercase().as_str() {
        "BOOL" => write_bool(c, db, offset, value_as_bool(value), bit),
        "WORD" | "INT" => write_word(c, db, offset, value_as_i64(value) as u16),
        "REAL" => write_real(c, db, offset, value_as_f64(value)),
        other => Err(format!("Tipo no soportado: {}", other)),
    }
}

pub fn value_as_f64(value: &Value) -> f64 {
    if let Some(n) = value.as_f64() {
        return n;
    }
    if let Some(s) = value.as_str() {
        return s.trim().replace(',', ".").parse().unwrap_or(0.0);
    }
    if let Some(b) = value.as_bool() {
        return if b { 1.0 } else { 0.0 };
    }
    0.0
}

pub fn value_as_i64(value: &Value) -> i64 {
    if let Some(n) = value.as_i64() {
        return n;
    }
    if let Some(f) = value.as_f64() {
        return f as i64;
    }
    if let Some(s) = value.as_str() {
        return s.trim().parse().unwrap_or(0);
    }
    if let Some(b) = value.as_bool() {
        return if b { 1 } else { 0 };
    }
    0
}

pub fn value_as_bool(value: &Value) -> bool {
    if let Some(b) = value.as_bool() {
        return b;
    }
    if let Some(n) = value.as_f64() {
        return n != 0.0;
    }
    if let Some(s) = value.as_str() {
        let s = s.trim().to_lowercase();
        return s == "true" || s == "1" || s == "yes" || s == "on";
    }
    false
}

// --- I/O directo ---

fn parse_q(q: &str) -> Result<(i32, i32), String> {
    let addr = q.trim().to_uppercase();
    if let Some((_, off, bit)) = Q_MAP.iter().find(|(a, _, _)| *a == addr) {
        return Ok((*off, *bit));
    }
    let rest = addr.trim_start_matches('Q');
    let mut parts = rest.split('.');
    let off = parts
        .next()
        .and_then(|v| v.parse::<i32>().ok())
        .ok_or_else(|| format!("Dirección Q inválida: {}", q))?;
    let bit = parts.next().and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
    Ok((off, bit))
}

fn parse_i(i: &str) -> Result<(i32, i32), String> {
    let addr = i.trim().to_uppercase();
    if let Some((_, off, bit)) = I_MAP.iter().find(|(a, _, _)| *a == addr) {
        return Ok((*off, *bit));
    }
    let rest = addr.trim_start_matches('I');
    let mut parts = rest.split('.');
    let off = parts
        .next()
        .and_then(|v| v.parse::<i32>().ok())
        .ok_or_else(|| format!("Dirección I inválida: {}", i))?;
    let bit = parts.next().and_then(|v| v.parse::<i32>().ok()).unwrap_or(0);
    Ok((off, bit))
}

pub fn read_q(c: &mut PlcClient, q: &str) -> bool {
    let (off, bit) = match parse_q(q) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let mut buf = vec![0u8; 1];
    c.ab_read(off, 1, &mut buf)
        .map(|_| ((buf[0] >> bit) & 1) == 1)
        .unwrap_or(false)
}

pub fn write_q(c: &mut PlcClient, q: &str, value: bool) -> Result<(), String> {
    let (off, bit) = parse_q(q)?;
    let mut buf = vec![0u8; 1];
    c.ab_read(off, 1, &mut buf).map_err(|e| e.to_string())?;
    if value {
        buf[0] |= 1 << bit;
    } else {
        buf[0] &= !(1 << bit);
    }
    c.ab_write(off, 1, &mut buf).map_err(|e| e.to_string())
}

pub fn read_i(c: &mut PlcClient, i: &str) -> bool {
    let (off, bit) = match parse_i(i) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let mut buf = vec![0u8; 1];
    c.eb_read(off, 1, &mut buf)
        .map(|_| ((buf[0] >> bit) & 1) == 1)
        .unwrap_or(false)
}

pub fn read_ia(c: &mut PlcClient, ia: &str) -> f64 {
    let off = IA_MAP
        .iter()
        .find(|(a, _)| *a == ia.trim().to_uppercase())
        .map(|(_, off)| *off)
        .unwrap_or(0);
    let mut buf = vec![0u8; 2];
    c.eb_read(off, 2, &mut buf)
        .map(|_| u16::from_be_bytes([buf[0], buf[1]]) as f64 / 10.0)
        .unwrap_or(0.0)
}

pub fn read_io(c: &mut PlcClient) -> Value {
    let mut outputs = BTreeMap::new();
    for (addr, _, _) in Q_MAP {
        outputs.insert((*addr).to_string(), json!(read_q(c, addr)));
    }
    let mut inputs = BTreeMap::new();
    for (addr, _, _) in I_MAP {
        inputs.insert((*addr).to_string(), json!(read_i(c, addr)));
    }
    let mut analog = BTreeMap::new();
    for (addr, _) in IA_MAP {
        analog.insert((*addr).to_string(), json!(read_ia(c, addr)));
    }
    json!({ "outputs": outputs, "inputs": inputs, "analog": analog })
}

/// I/O con todos los valores en reposo (para cuando no hay conexión).
pub fn empty_io() -> Value {
    let mut outputs = BTreeMap::new();
    for (addr, _, _) in Q_MAP {
        outputs.insert((*addr).to_string(), json!(false));
    }
    let mut inputs = BTreeMap::new();
    for (addr, _, _) in I_MAP {
        inputs.insert((*addr).to_string(), json!(false));
    }
    let mut analog = BTreeMap::new();
    for (addr, _) in IA_MAP {
        analog.insert((*addr).to_string(), json!(0.0));
    }
    json!({ "outputs": outputs, "inputs": inputs, "analog": analog })
}

/// Resultados en reposo (sin conexión).
pub fn empty_results() -> Value {
    let mut out = BTreeMap::new();
    for (name, _) in RES_OFFSETS {
        out.insert((*name).to_string(), json!(0.0));
    }
    out.insert("complete".to_string(), json!(false));
    out.insert("pass".to_string(), json!(false));
    out.insert("running".to_string(), json!(false));
    Value::Object(out.into_iter().collect())
}

// --- Receta / ejecución ---

pub fn send_recipe(c: &mut PlcClient, voltage: f64, components: &Value) -> Result<(), String> {
    write_real(c, 1, OFF_VOLTAGE, voltage)?;
    if let Some(arr) = components.as_array() {
        for comp in arr {
            let name = comp
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            if let Some((_, min_off, max_off, en_off)) =
                COMP_OFFSETS.iter().find(|(n, _, _, _)| *n == name)
            {
                let min = comp.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let max = comp.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let enabled = comp
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                write_real(c, 1, *min_off, min)?;
                write_real(c, 1, *max_off, max)?;
                write_bool(c, 1, *en_off, enabled, 0)?;
            }
        }
    }
    Ok(())
}

pub fn start(c: &mut PlcClient) -> Result<(), String> {
    write_bool(c, 1, OFF_START, true, 0)
}

pub fn stop(c: &mut PlcClient) -> Result<(), String> {
    write_bool(c, 1, OFF_STOP, true, 0)
}

pub fn read_results(c: &mut PlcClient) -> Value {
    let mut out = BTreeMap::new();
    for (name, off) in RES_OFFSETS {
        out.insert((*name).to_string(), json!(read_real(c, 1, *off).unwrap_or(0.0)));
    }
    out.insert(
        "complete".to_string(),
        json!(read_bool(c, 1, OFF_COMPLETE, 0).unwrap_or(false)),
    );
    out.insert(
        "pass".to_string(),
        json!(read_bool(c, 1, OFF_PASS, 0).unwrap_or(false)),
    );
    out.insert(
        "running".to_string(),
        json!(read_bool(c, 1, OFF_RUNNING, 0).unwrap_or(false)),
    );
    Value::Object(out.into_iter().collect())
}

pub fn set_mode(c: &mut PlcClient, mode: i32) -> Result<(), String> {
    if mode != 1 && mode != 2 {
        return Err("mode debe ser 1 (auto) o 2 (manual)".to_string());
    }
    write_word(c, 1, OFF_MODE, mode as u16)
}

pub fn get_mode(c: &mut PlcClient) -> Result<i32, String> {
    Ok(read_word(c, 1, OFF_MODE)? as i32)
}

pub fn test_component(c: &mut PlcClient, component: &str, voltage: f64) -> Result<Value, String> {
    let name = component.trim().to_uppercase();
    set_mode(c, 2)?;
    for (comp, _, _, en_off) in COMP_OFFSETS {
        write_bool(c, 1, *en_off, *comp == name, 0)?;
    }
    write_real(c, 1, OFF_VOLTAGE, voltage)?;
    write_bool(c, 1, OFF_START, true, 0)?;
    std::thread::sleep(Duration::from_millis(600));
    let all = read_results(c);
    let value = all.get(&name).and_then(|v| v.as_f64()).unwrap_or(0.0);
    Ok(json!({ "component": name, "value": value, "all": all }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_q_and_i_addresses() {
        assert_eq!(parse_q("Q0.1").unwrap(), (0, 1));
        assert_eq!(parse_q("Q2.2").unwrap(), (2, 2));
        assert_eq!(parse_q("Q3.4").unwrap(), (3, 4));
        assert_eq!(parse_i("I0.2").unwrap(), (0, 2));
        assert_eq!(parse_i("I1.3").unwrap(), (1, 3));
        assert!(parse_q("X").is_err());
    }

    #[test]
    fn converts_json_values() {
        assert!(value_as_bool(&json!("true")));
        assert!(!value_as_bool(&json!("0")));
        assert_eq!(value_as_i64(&json!("12")), 12);
        assert!((value_as_f64(&json!("1,5")) - 1.5).abs() < 0.001);
        assert!((value_as_f64(&json!(2.25)) - 2.25).abs() < 0.001);
    }
}
