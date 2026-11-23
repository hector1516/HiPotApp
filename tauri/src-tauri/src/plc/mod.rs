//! PLC Siemens S7-1200 sobre el crate `s7`.
//!
//! Interfaz real del programa HUSSMANN:
//! - DB1000 (COMM01_EXT_UNIT): receta (bitmask), tiempos, tolerancia, start/stop.
//! - DB1001 (COMM01_UNIT_EXT): resultados, Measuring, Finished, Error.
//! - Lectura/escritura de tags (BOOL / WORD / INT / REAL).
//! - Mapa de salidas Q, entradas I y analógicas IA.





use s7::client::Client;
use s7::tcp::Transport;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

pub type PlcClient = Client<Transport>;

// --- Interfaz real del PLC (COMM01_EXT_UNIT / COMM01_UNIT_EXT) ---
// El programa del PLC ya expone dos DB no optimizadas para la app:
//   DB1000 = COMM01_EXT_UNIT : receta, tiempos, tolerancia, start/stop
//   DB1001 = COMM01_UNIT_EXT : resultados y estado
// No usar DB1/DB2/DB3: DB1 es RDM INT y DB2/DB3 son optimizadas.
pub const RECIPE_DB: i32 = 1000;
pub const RESULT_DB: i32 = 1001;

pub const OFF_RECIPE_MASK: i32 = 4; // REAL: bitmask de 18 bits + bit 18 = 110VAC
pub const OFF_START_STOP: i32 = 2; // byte de bits
pub const START_BIT: i32 = 5; // DB1000.DBX2.5
pub const STOP_BIT: i32 = 6; // DB1000.DBX2.6
pub const OFF_TOLERANCE: i32 = 98; // REAL, porcentaje

/// (componente, bit base de A, offset de tiempo Real)
pub const COMPONENTS: &[(&str, i32, i32)] = &[
    ("FAN", 0, 8),
    ("DOOR", 3, 12),
    ("FRAME", 6, 16),
    ("DRAIN", 9, 86),
    ("DEFROST", 12, 90),
    ("LIGHT", 15, 94),
];
pub const VOLTAGE_110_BIT: i32 = 18;

pub const RES_OFFSETS: &[(&str, i32)] = &[
    ("FAN", 4),
    ("DOOR", 8),
    ("FRAME", 12),
    ("DRAIN", 16),
    ("DEFROST", 86),
    ("LIGHT", 90),
];
pub const OFF_RESULT_STATUS: i32 = 2;
pub const RES_MEASURING_BIT: i32 = 1;
pub const RES_FINISHED_BIT: i32 = 2;
pub const RES_ERROR_BIT: i32 = 5;

// --- PLC puente "tonto" (DB100) ---
// DB100: IB0..IB3 en 0..3, QB0..QB3 en 4..7, IW100/102/104/106 en 8/10/12/14.
pub const BRIDGE_DB: i32 = 100;
pub const BRIDGE_INPUT_OFFSET: i32 = 0;
pub const BRIDGE_OUTPUT_OFFSET: i32 = 4;
pub const BRIDGE_ANALOG_OFFSET: i32 = 8;

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
    // El crate s7 usa TcpStream::connect sin timeout; si la IP no existe,
    // Windows puede tardar 20-120 s. Hacemos un pre-chequeo acotado.
    let timeout = Duration::from_millis(timeout_ms.max(200));
    let sock = std::net::SocketAddr::new(addr, 102);
    std::net::TcpStream::connect_timeout(&sock, timeout)
        .map_err(|e| format!("PLC no responde en {}:102: {}", cfg.ip, e))?;
    let mut opts = s7::tcp::Options::new(addr, cfg.rack, cfg.slot, s7::transport::Connection::OP);
    opts.read_timeout = timeout;
    opts.write_timeout = timeout;
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

pub fn read_i_checked(c: &mut PlcClient, i: &str) -> Result<bool, String> {
    let (off, bit) = parse_i(i)?;
    let mut buf = vec![0u8; 1];
    c.eb_read(off, 1, &mut buf).map_err(|e| e.to_string())?;
    Ok(((buf[0] >> bit) & 1) == 1)
}

pub fn read_i(c: &mut PlcClient, i: &str) -> bool {
    read_i_checked(c, i).unwrap_or(false)
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
    out.insert("error".to_string(), json!(false));
    Value::Object(out.into_iter().collect())
}

// --- Receta / ejecución ---

/// Envía la receta real al PLC.
///
/// La app maneja un rango de corriente por componente. El PLC tiene 3 canales
/// de medición (A/B/C); se elige el menor que cubra el máximo de la receta:
///   A: <= 1 A, B: <= 10 A, C: resto (hasta 20 A).
/// El bit 18 selecciona 110 VAC; en caso contrario el PLC usa 220 VAC.
pub fn send_recipe(
    c: &mut PlcClient,
    voltage: f64,
    components: &Value,
    test_time: f64,
) -> Result<(), String> {
    let mut mask: i32 = 0;
    if let Some(arr) = components.as_array() {
        for comp in arr {
            let name = comp
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_uppercase();
            let enabled = comp
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if !enabled || name.is_empty() {
                continue;
            }
            if let Some((_, base, _)) = COMPONENTS.iter().find(|(n, _, _)| *n == name) {
                let max = comp.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let variant = if max <= 1.0 {
                    0
                } else if max <= 10.0 {
                    1
                } else {
                    2
                };
                mask |= 1i32 << (*base + variant);
            }
        }
    }
    if voltage < 150.0 {
        mask |= 1i32 << VOLTAGE_110_BIT;
    }
    write_real(c, RECIPE_DB, OFF_RECIPE_MASK, mask as f64)?;

    // Tiempo de prueba: el de la receta (10 s por defecto).
    let t = if test_time > 0.0 { test_time } else { 10.0 };
    for (_, _, time_off) in COMPONENTS {
        write_real(c, RECIPE_DB, *time_off, t)?;
    }
    // Tolerancia de voltaje: 5 %.
    write_real(c, RECIPE_DB, OFF_TOLERANCE, 5.0)?;
    Ok(())
}

pub fn start(c: &mut PlcClient) -> Result<(), String> {
    write_bool(c, RECIPE_DB, OFF_START_STOP, true, START_BIT)?;
    std::thread::sleep(Duration::from_millis(180));
    write_bool(c, RECIPE_DB, OFF_START_STOP, false, START_BIT)
}

pub fn stop(c: &mut PlcClient) -> Result<(), String> {
    write_bool(c, RECIPE_DB, OFF_START_STOP, true, STOP_BIT)?;
    std::thread::sleep(Duration::from_millis(180));
    write_bool(c, RECIPE_DB, OFF_START_STOP, false, STOP_BIT)
}

fn maybe_bool(c: &mut PlcClient, db: i32, offset: i32, bit: i32) -> Value {
    match read_bool(c, db, offset, bit) {
        Ok(v) => json!(v),
        Err(_) => Value::Null,
    }
}

fn maybe_real(c: &mut PlcClient, db: i32, offset: i32) -> Value {
    match read_real(c, db, offset) {
        Ok(v) => json!(v),
        Err(_) => Value::Null,
    }
}

/// Lectura cruda de la interfaz de la app, sin depender de nombres lógicos.
/// Sirve para mostrar direcciones y valores reales en la pantalla Live.
pub fn read_raw_monitor(c: &mut PlcClient) -> Value {
    let db1000 = json!({
        "start": maybe_bool(c, RECIPE_DB, OFF_START_STOP, START_BIT),
        "stop": maybe_bool(c, RECIPE_DB, OFF_START_STOP, STOP_BIT),
        "mask": maybe_real(c, RECIPE_DB, OFF_RECIPE_MASK),
        "fan_time": maybe_real(c, RECIPE_DB, 8),
        "door_time": maybe_real(c, RECIPE_DB, 12),
        "frame_time": maybe_real(c, RECIPE_DB, 16),
        "drain_time": maybe_real(c, RECIPE_DB, 86),
        "defrost_time": maybe_real(c, RECIPE_DB, 90),
        "light_time": maybe_real(c, RECIPE_DB, 94),
        "tolerance": maybe_real(c, RECIPE_DB, OFF_TOLERANCE),
    });
    let db1001 = json!({
        "measuring": maybe_bool(c, RESULT_DB, OFF_RESULT_STATUS, RES_MEASURING_BIT),
        "finished": maybe_bool(c, RESULT_DB, OFF_RESULT_STATUS, RES_FINISHED_BIT),
        "estop": maybe_bool(c, RESULT_DB, OFF_RESULT_STATUS, 4),
        "error": maybe_bool(c, RESULT_DB, OFF_RESULT_STATUS, RES_ERROR_BIT),
        "fan": maybe_real(c, RESULT_DB, 4),
        "door": maybe_real(c, RESULT_DB, 8),
        "frame": maybe_real(c, RESULT_DB, 12),
        "drain": maybe_real(c, RESULT_DB, 16),
        "defrost": maybe_real(c, RESULT_DB, 86),
        "light": maybe_real(c, RESULT_DB, 90),
    });
    json!({ "DB1000": db1000, "DB1001": db1001 })
}

/// Lee toda la interfaz DB100 del PLC puente.
pub fn read_bridge(c: &mut PlcClient) -> Result<Value, String> {
    let mut inputs = [0u8; 4];
    c.ag_read(BRIDGE_DB, BRIDGE_INPUT_OFFSET, 4, &mut inputs)
        .map_err(|e| e.to_string())?;
    let mut outputs = [0u8; 4];
    c.ag_read(BRIDGE_DB, BRIDGE_OUTPUT_OFFSET, 4, &mut outputs)
        .map_err(|e| e.to_string())?;
    let mut analog = [0u16; 4];
    for (i, slot) in analog.iter_mut().enumerate() {
        *slot = read_word(c, BRIDGE_DB, BRIDGE_ANALOG_OFFSET + (i as i32 * 2))?;
    }

    let mut input_map = BTreeMap::new();
    for byte in 0..4 {
        for bit in 0..8 {
            input_map.insert(
                format!("I{}.{}", byte, bit),
                json!((inputs[byte] >> bit) & 1 == 1),
            );
        }
    }
    let mut output_map = BTreeMap::new();
    for byte in 0..4 {
        for bit in 0..8 {
            output_map.insert(
                format!("Q{}.{}", byte, bit),
                json!((outputs[byte] >> bit) & 1 == 1),
            );
        }
    }
    let mut analog_map = BTreeMap::new();
    for (i, value) in analog.iter().enumerate() {
        analog_map.insert(format!("IA{}", i + 1), json!(value));
    }

    Ok(json!({
        "connected": true,
        "bridge": true,
        "inputs": input_map,
        "outputs": output_map,
        "analog": analog_map,
        "raw_inputs": inputs,
        "raw_outputs": outputs,
        "raw_analog": analog,
        "estop": (inputs[0] & 1) == 1,
    }))
}

/// Escribe una salida lógica en QB0..QB3 del PLC puente.
pub fn write_bridge_output(c: &mut PlcClient, q: &str, value: bool) -> Result<(), String> {
    let (byte, bit) = parse_q(q)?;
    if !(0..4).contains(&byte) {
        return Err(format!("Salida fuera de QB0..QB3: {}", q));
    }
    let offset = BRIDGE_OUTPUT_OFFSET + byte;
    let mut buf = [0u8; 1];
    c.ag_read(BRIDGE_DB, offset, 1, &mut buf)
        .map_err(|e| e.to_string())?;
    if value {
        buf[0] |= 1 << bit;
    } else {
        buf[0] &= !(1 << bit);
    }
    c.ag_write(BRIDGE_DB, offset, 1, &buf)
        .map_err(|e| e.to_string())
}

pub fn read_results(c: &mut PlcClient) -> Value {
    let mut out = BTreeMap::new();
    for (name, off) in RES_OFFSETS {
        out.insert((*name).to_string(), json!(read_real(c, RESULT_DB, *off).unwrap_or(0.0)));
    }
    out.insert(
        "complete".to_string(),
        json!(read_bool(c, RESULT_DB, OFF_RESULT_STATUS, RES_FINISHED_BIT).unwrap_or(false)),
    );
    out.insert(
        "pass".to_string(),
        json!(false),
    );
    out.insert(
        "running".to_string(),
        json!(read_bool(c, RESULT_DB, OFF_RESULT_STATUS, RES_MEASURING_BIT).unwrap_or(false)),
    );
    out.insert(
        "error".to_string(),
        json!(read_bool(c, RESULT_DB, OFF_RESULT_STATUS, RES_ERROR_BIT).unwrap_or(false)),
    );
    Value::Object(out.into_iter().collect())
}

pub fn set_mode(_c: &mut PlcClient, mode: i32) -> Result<(), String> {
    if mode != 1 && mode != 2 {
        return Err("mode debe ser 1 (auto) o 2 (manual)".to_string());
    }
    // El programa PLC no tiene registro de modo; la secuencia se controla
    // con START_MEASURE/STOP y la receta.
    Ok(())
}

pub fn get_mode(_c: &mut PlcClient) -> Result<i32, String> {
    Ok(1)
}

pub fn test_component(c: &mut PlcClient, component: &str, voltage: f64) -> Result<Value, String> {
    let name = component.trim().to_uppercase();
    let components = json!([{
        "name": name,
        "enabled": true,
        "min": 0.0,
        "max": 20.0
    }]);
    send_recipe(c, voltage, &components, 1.0)?;
    start(c)?;
    std::thread::sleep(Duration::from_millis(1500));
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
