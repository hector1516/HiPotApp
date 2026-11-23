// HiPotApp: aplicación Tauri 2 + núcleo Rust (PLC, Hipot, BD, impresión).
// Un solo ejecutable, sin sidecar Python ni puertos HTTP.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config_io;
mod db;
mod hipot;
mod plc;

use tauri::{Emitter, Manager};
use std::sync::{Mutex, OnceLock};

static CONFIG_CACHE: OnceLock<Mutex<serde_json::Value>> = OnceLock::new();

fn default_config_json() -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, value) in db::seed::DEFAULT_CONFIGS {
        map.insert(
            (*key).to_string(),
            serde_json::Value::String((*value).to_string()),
        );
    }
    serde_json::Value::Object(map)
}

fn config_cache() -> &'static Mutex<serde_json::Value> {
    CONFIG_CACHE.get_or_init(|| Mutex::new(default_config_json()))
}

/// Configuración en memoria. Nunca bloquea esperando SQL Express.
#[tauri::command]
fn get_config() -> serde_json::Value {
    config_cache()
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| default_config_json())
}

fn reload_config_from_db() {
    let value = db::repo::config_get_all();
    let non_empty = value
        .as_object()
        .map(|obj| !obj.is_empty())
        .unwrap_or(false);
    if non_empty {
        if let Ok(mut guard) = config_cache().lock() {
            *guard = value;
        }
    }
}

#[tauri::command]
fn app_health() -> serde_json::Value {
    let db_ready = db::is_ready();
    let db_error = db::last_error();
    let cfg = get_config();
    let plc_ip = cfg
        .get("plc_ip")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    serde_json::json!({
        "status": if db_ready { "healthy" } else if db_error.is_some() { "error" } else { "starting" },
        "runtime": "rust",
        "database": db_ready,
        "database_engine": "SQL Express",
        "db_ready": db_ready,
        "db_initializing": !db_ready && db_error.is_none(),
        "db_error": db_error,
        "plc_configured": !plc_ip.is_empty(),
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

fn initialize_db_in_background() {
    std::thread::spawn(|| match db::init() {
        Ok(()) => {
            reload_config_from_db();
            db::set_ready(true);
        }
        Err(e) if e.contains("ya se está inicializando") => {}
        Err(e) => db::set_error(e),
    });
}

#[tauri::command]
fn db_retry() -> String {
    initialize_db_in_background();
    "retrying".to_string()
}

#[tauri::command]
fn set_config(values: serde_json::Value) -> String {
    match db::repo::config_set(&values) {
        Ok(_) => {
            if let Ok(mut cache) = config_cache().lock() {
                if let (Some(dst), Some(src)) = (cache.as_object_mut(), values.as_object()) {
                    for (key, value) in src {
                        dst.insert(key.clone(), value.clone());
                    }
                }
            }
            "saved".to_string()
        }
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_status(ip: String, rack: u16, slot: u16) -> bool {
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = match ip.parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_millis(1000);
    opts.write_timeout = Duration::from_millis(1000);
    let transport = match s7::tcp::Transport::connect(opts) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let mut client = match s7::client::Client::new(transport) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut buf = vec![0u8; 1];
    client.ab_read(0, 1, &mut buf).is_ok()
}

#[tauri::command]
fn plc_read_tag(ip: String, rack: u16, slot: u16, db: i32, offset: i32, data_type: String, bit: i32) -> Result<serde_json::Value, String> {
    let cfg = plc::PlcConfig { ip, rack, slot };
    let mut client = plc::connect(&cfg, 2000)?;
    plc::read_tag(&mut client, db, offset, &data_type, bit)
}

#[tauri::command]
fn plc_write_tag(ip: String, rack: u16, slot: u16, db: i32, offset: i32, data_type: String, value: serde_json::Value, bit: i32) -> Result<String, String> {
    let cfg = plc::PlcConfig { ip, rack, slot };
    let mut client = plc::connect(&cfg, 2000)?;
    plc::write_tag(&mut client, db, offset, &data_type, &value, bit)?;
    Ok("written".to_string())
}

#[tauri::command]
fn show_touch_keyboard() {
    #[cfg(target_os = "windows")]
    {
        let tabtip = "C:\\Program Files\\Common Files\\Microsoft Shared\\Ink\\TabTip.exe";
        let _ = std::process::Command::new(tabtip).spawn();
    }
}

#[tauri::command]
fn open_external(url: String) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn();
    }
}

// --- PLC admin / I/O / manual (Rust, producción) ---

fn config_flag(key: &str) -> bool {
    get_config()
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        == "1"
}

fn plc_cfg() -> plc::PlcConfig {
    plc::config_from(&get_config())
}

#[tauri::command]
fn plc_config_get() -> serde_json::Value {
    plc::config_to_json(&plc_cfg())
}

#[tauri::command]
fn plc_config_set(ip: String, rack: u16, slot: u16) -> String {
    set_config(serde_json::json!({
        "plc_ip": ip,
        "plc_rack": rack.to_string(),
        "plc_slot": slot.to_string(),
    }))
}

#[tauri::command]
fn plc_status_info() -> serde_json::Value {
    let config = get_config();
    let cfg = plc::config_from(&config);
    let address = estop_address(&config);
    if !config_flag("plc_enabled") {
        return serde_json::json!({
            "connected": false, "enabled": false, "configured": false,
            "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot,
            "estop_address": address,
            "error": "PLC no habilitado. Actívelo en Configuración → PLC."
        });
    }
    match plc::connect(&cfg, 1500) {
        Ok(mut client) => match read_estop_source(&mut client, &config) {
            Ok(raw) => serde_json::json!({
                "connected": true, "enabled": true, "configured": true, "read_ok": true,
                "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot,
                "estop_address": address, "estop_raw": raw
            }),
            Err(e) => serde_json::json!({
                "connected": false, "enabled": true, "configured": true, "read_ok": false,
                "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot,
                "estop_address": address,
                "error": format!("Conectó al PLC pero no pudo leer {}: {}", address, e)
            }),
        },
        Err(e) => serde_json::json!({
            "connected": false, "enabled": true, "configured": true, "read_ok": false,
            "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot,
            "estop_address": address, "error": e
        }),
    }
}

#[tauri::command]
fn plc_probe() -> serde_json::Value {
    let config = get_config();
    let cfg = plc::config_from(&config);
    let enabled = config_flag("plc_enabled");
    let address = estop_address(&config);
    match plc::connect(&cfg, 1500) {
        Ok(mut client) => match read_estop_source(&mut client, &config) {
            Ok(raw) => serde_json::json!({
                "connected": true, "enabled": enabled, "read_ok": true,
                "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot,
                "estop_address": address, "estop_raw": raw
            }),
            Err(e) => serde_json::json!({
                "connected": false, "enabled": enabled, "read_ok": false,
                "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot,
                "estop_address": address,
                "error": format!("Conectó al PLC pero no pudo leer {}: {}", address, e)
            }),
        },
        Err(e) => serde_json::json!({
            "connected": false, "enabled": enabled, "read_ok": false,
            "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot,
            "estop_address": address, "error": e
        }),
    }
}

#[tauri::command]
fn plc_emergency_stop() -> serde_json::Value {
    let config = get_config();
    let cfg = plc::config_from(&config);
    let address = estop_address(&config);
    let source = estop_source(&config);
    // Por defecto se asume NC (normalmente cerrado): true = sano, false = presionado.
    let nc = config
        .get("estop_nc")
        .and_then(|v| v.as_str())
        .unwrap_or("1")
        != "0";
    if !config_flag("plc_enabled") {
        return serde_json::json!({
            "active": false,
            "raw": null,
            "address": address,
            "source": source,
            "connected": false,
            "enabled": false,
            "inverted": nc,
            "error": "PLC no habilitado. Actívelo en Configuración → PLC."
        });
    }
    match plc::connect(&cfg, 800) {
        Ok(mut client) => match read_estop_source(&mut client, &config) {
            Ok(raw) => {
                let active = if nc { !raw } else { raw };
                serde_json::json!({
                    "active": active,
                    "raw": raw,
                    "address": address,
                    "source": source,
                    "connected": true,
                    "inverted": nc,
                })
            }
            Err(e) => serde_json::json!({
                "active": false,
                "raw": null,
                "address": address,
                "source": source,
                "connected": false,
                "inverted": nc,
                "error": format!("No se pudo leer {}: {}", address, e),
            }),
        },
        Err(e) => serde_json::json!({
            "active": false,
            "raw": null,
            "address": address,
            "source": source,
            "connected": false,
            "error": e,
        }),
    }
}

fn estop_source(config: &serde_json::Value) -> String {
    config
        .get("estop_source")
        .and_then(|v| v.as_str())
        .unwrap_or("DB")
        .to_uppercase()
}

fn estop_address(config: &serde_json::Value) -> String {
    if estop_source(config) == "DB" {
        let db = config
            .get("estop_db")
            .and_then(|v| v.as_str())
            .unwrap_or("1001");
        let offset = config
            .get("estop_offset")
            .and_then(|v| v.as_str())
            .unwrap_or("2");
        let bit = config
            .get("estop_bit")
            .and_then(|v| v.as_str())
            .unwrap_or("5");
        format!("DB{}.DBX{}.{}", db, offset, bit)
    } else {
        config
            .get("estop_input")
            .and_then(|v| v.as_str())
            .unwrap_or("I0.0")
            .to_string()
    }
}

fn read_estop_source(
    client: &mut plc::PlcClient,
    config: &serde_json::Value,
) -> Result<bool, String> {
    if estop_source(config) == "DB" {
        let db = config
            .get("estop_db")
            .and_then(|v| v.as_str())
            .unwrap_or("1001")
            .parse::<i32>()
            .map_err(|_| "estop_db inválido".to_string())?;
        let offset = config
            .get("estop_offset")
            .and_then(|v| v.as_str())
            .unwrap_or("2")
            .parse::<i32>()
            .map_err(|_| "estop_offset inválido".to_string())?;
        let bit = config
            .get("estop_bit")
            .and_then(|v| v.as_str())
            .unwrap_or("5")
            .parse::<i32>()
            .map_err(|_| "estop_bit inválido".to_string())?;
        let value = plc::read_tag(client, db, offset, "BOOL", bit)?;
        Ok(value.as_bool().unwrap_or(false))
    } else {
        let address = config
            .get("estop_input")
            .and_then(|v| v.as_str())
            .unwrap_or("I0.0");
        plc::read_i_checked(client, address)
    }
}

#[tauri::command]
fn resolve_model_by_serial(_serial: String) -> Option<String> {
    // Placeholder hasta tener acceso a la BD externa serial -> modelo.
    None
}

#[tauri::command]
fn plc_tags_list() -> serde_json::Value {
    if !db::is_ready() {
        return serde_json::json!([]);
    }
    let mut rows = db::repo::plc_tags_list();
    if !config_flag("plc_enabled") {
        return rows;
    }
    let cfg = plc_cfg();
    let mut client = plc::connect(&cfg, 1500).ok();
    if let Some(arr) = rows.as_array_mut() {
        for tag in arr {
            let db_number = tag.get("db").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let offset = tag.get("offset").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let data_type = tag.get("type").and_then(|v| v.as_str()).unwrap_or("REAL");
            let bit = tag.get("bit").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let value = if let Some(c) = client.as_mut() {
                plc::read_tag(c, db_number, offset, data_type, bit)
                    .unwrap_or(serde_json::json!(null))
            } else {
                serde_json::json!(null)
            };
            if let Some(obj) = tag.as_object_mut() {
                obj.insert("value".to_string(), value);
            }
        }
    }
    rows
}

#[tauri::command]
fn plc_tags_add(name: String, db_number: i64, offset: i64, data_type: String, bit: i64) -> i64 {
    match db::repo::plc_tags_add(&name, db_number, offset, &data_type, bit) {
        Ok(id) => id,
        Err(e) => { eprintln!("[plc_tags_add] {}", e); -1 }
    }
}

#[tauri::command]
fn plc_tags_delete(id: i64) -> String {
    match db::repo::plc_tags_delete(id) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_tag_write_by_id(tag_id: i64, value: serde_json::Value) -> String {
    let tag = match db::repo::plc_tag_get(tag_id) {
        Ok(v) => v,
        Err(e) => return format!("error: {}", e),
    };
    let db_number = tag.get("db_number").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let offset = tag.get("offset").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let data_type = tag
        .get("data_type")
        .and_then(|v| v.as_str())
        .unwrap_or("REAL");
    let bit = tag.get("bit").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::write_tag(&mut c, db_number, offset, data_type, &value, bit) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_tag_read_by_id(tag_id: i64) -> serde_json::Value {
    let tag = match db::repo::plc_tag_get(tag_id) {
        Ok(v) => v,
        Err(_) => return serde_json::json!(null),
    };
    let db_number = tag.get("db_number").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let offset = tag.get("offset").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let data_type = tag
        .get("data_type")
        .and_then(|v| v.as_str())
        .unwrap_or("REAL");
    let bit = tag.get("bit").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => plc::read_tag(&mut c, db_number, offset, data_type, bit)
            .unwrap_or(serde_json::json!(null)),
        Err(_) => serde_json::json!(null),
    }
}

#[tauri::command]
fn plc_io_status() -> serde_json::Value {
    let config = get_config();
    let cfg = plc::config_from(&config);
    let address = estop_address(&config);
    if !config_flag("plc_enabled") {
        let mut v = plc::empty_io();
        if let Some(obj) = v.as_object_mut() {
            obj.insert("connected".to_string(), serde_json::json!(false));
            obj.insert("enabled".to_string(), serde_json::json!(false));
            obj.insert("estop_address".to_string(), serde_json::json!(address));
            obj.insert("error".to_string(), serde_json::json!("PLC no habilitado. Actívelo en Configuración → PLC."));
        }
        return v;
    }
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let read_test = read_estop_source(&mut c, &config);
            let mut v = plc::read_io(&mut c);
            if let Some(obj) = v.as_object_mut() {
                obj.insert("connected".to_string(), serde_json::json!(true));
                obj.insert("read_ok".to_string(), serde_json::json!(read_test.is_ok()));
                obj.insert("estop_address".to_string(), serde_json::json!(address));
                obj.insert(
                    "estop_raw".to_string(),
                    serde_json::json!(read_test.as_ref().ok()),
                );
                if let Err(e) = &read_test {
                    obj.insert(
                        "error".to_string(),
                        serde_json::json!(format!("Conectó pero no pudo leer {}: {}", address, e)),
                    );
                }
            }
            v
        }
        Err(e) => {
            let mut v = plc::empty_io();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("connected".to_string(), serde_json::json!(false));
                obj.insert("error".to_string(), serde_json::json!(e));
            }
            v
        }
    }
}

#[tauri::command]
fn plc_io_write(q: String, value: bool) -> String {
    if !config_flag("plc_enabled") {
        return "error: PLC no habilitado".to_string();
    }
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::write_q(&mut c, &q, value) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_send_recipe(voltage: f64, components: serde_json::Value) -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::send_recipe(&mut c, voltage, &components, 10.0) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_start() -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::start(&mut c) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_stop() -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::stop(&mut c) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_read_results() -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => plc::read_results(&mut c),
        Err(e) => {
            let mut v = plc::empty_results();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("error".to_string(), serde_json::json!(e));
            }
            v
        }
    }
}

#[tauri::command]
fn plc_set_mode(mode: i64) -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let ok = plc::set_mode(&mut c, mode as i32).is_ok();
            let current = plc::get_mode(&mut c).unwrap_or(1);
            serde_json::json!({ "mode": current, "ok": ok })
        }
        Err(e) => serde_json::json!({ "mode": 1, "ok": false, "error": e }),
    }
}

#[tauri::command]
fn plc_get_mode() -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => serde_json::json!({ "mode": plc::get_mode(&mut c).unwrap_or(1) }),
        Err(e) => serde_json::json!({ "mode": 1, "error": e }),
    }
}

#[tauri::command]
fn plc_test_component(component: String, voltage: f64) -> serde_json::Value {
    let name = component.trim().to_uppercase();
    let cfg = plc_cfg();
    match plc::connect(&cfg, 2000) {
        Ok(mut c) => match plc::test_component(&mut c, &name, voltage) {
            Ok(v) => v,
            Err(e) => serde_json::json!({ "component": name, "value": 0.0, "error": e }),
        },
        Err(e) => serde_json::json!({ "component": name, "value": 0.0, "error": e }),
    }
}

#[tauri::command]
fn plc_abort() -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let _ = plc::stop(&mut c);
            let _ = plc::set_mode(&mut c, 1);
            "OK".to_string()
        }
        Err(e) => format!("error: {}", e),
    }
}

// --- Hipot 3865 (Rust) ---

#[tauri::command]
fn hipot_list_ports() -> Vec<String> {
    hipot::list_ports()
}

#[tauri::command]
fn hipot_config_get() -> serde_json::Value {
    let cfg = hipot::config_from(&get_config());
    hipot::config_to_json(&cfg)
}

#[tauri::command]
fn hipot_config_set(port: String, baud: u32) -> String {
    let result = set_config(serde_json::json!({
        "hipot_port": port,
        "hipot_baud": baud.to_string(),
    }));
    let _ = db::repo::log_add(
        "INFO", "HIPOT", "config",
        Some(&format!("Puerto {} @ {} baudios", port, baud)),
        None, None, None, None, result.starts_with("saved"), None,
    );
    result
}

#[tauri::command]
fn hipot_status() -> serde_json::Value {
    let cfg = hipot::config_from(&get_config());
    let ports = hipot::list_ports();
    if !config_flag("hipot_enabled") {
        return serde_json::json!({
            "connected": false, "enabled": false, "port": cfg.port, "baud": cfg.baud,
            "ports": ports, "error": "Hipot no habilitado. Actívelo en Configuración → Hipot."
        });
    }
    match hipot::check_connection(&cfg) {
        Ok(_) => serde_json::json!({
            "connected": true, "port": cfg.port, "baud": cfg.baud, "ports": ports
        }),
        Err(e) => serde_json::json!({
            "connected": false, "port": cfg.port, "baud": cfg.baud,
            "ports": ports, "error": e
        }),
    }
}

#[tauri::command]
fn hipot_probe() -> serde_json::Value {
    let cfg = hipot::config_from(&get_config());
    let ports = hipot::list_ports();
    match hipot::command(&cfg, "*IDN?") {
        Ok(value) => {
            let response = value
                .get("response")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            serde_json::json!({
                "connected": true,
                "responding": !response.is_empty() && response != "ACK",
                "port": cfg.port,
                "baud": cfg.baud,
                "ports": ports,
                "idn": response,
                "parsed": value.get("parsed").cloned().unwrap_or_else(|| serde_json::json!({})),
            })
        }
        Err(e) => serde_json::json!({
            "connected": false,
            "responding": false,
            "port": cfg.port,
            "baud": cfg.baud,
            "ports": ports,
            "error": e,
        }),
    }
}

#[tauri::command]
fn hipot_command(command: String) -> serde_json::Value {
    let cfg = hipot::config_from(&get_config());
    match hipot::command(&cfg, &command) {
        Ok(mut value) => {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("command".to_string(), serde_json::json!(command));
            }
            value
        }
        Err(e) => serde_json::json!({
            "response": "", "parsed": {}, "error": e, "command": command
        }),
    }
}

#[tauri::command]
fn hipot_prepare(voltage: f64, current_max: f64, time: f64) -> serde_json::Value {
    let cfg = hipot::config_from(&get_config());
    match hipot::prepare_test(&cfg, voltage, current_max, time) {
        Ok(value) => serde_json::json!({ "ok": true, "detail": value }),
        Err(e) => serde_json::json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
fn hipot_test(action: String) -> serde_json::Value {
    let cfg = hipot::config_from(&get_config());
    match hipot::test_action(&cfg, &action) {
        Ok(value) => value,
        Err(e) => serde_json::json!({
            "response": "", "parsed": {}, "error": e, "action": action
        }),
    }
}

#[tauri::command]
fn hipot_abort() -> serde_json::Value {
    hipot_test("RESET".to_string())
}

#[tauri::command]
fn hipot_close() -> String {
    hipot::close();
    let _ = db::repo::log_add(
        "INFO", "HIPOT", "close", Some("Puerto Hipot cerrado"),
        None, None, None, None, true, None,
    );
    "closed".to_string()
}

/// Ciclo real de producción: PLC -> Hipot -> guardar resultados.
#[tauri::command]
fn run_test_cycle(
    app: tauri::AppHandle,
    execution_id: i64,
    recipe_id: i64,
    voltage: f64,
    components: serde_json::Value,
) -> serde_json::Value {
    let started = std::time::Instant::now();
    let mut errors: Vec<String> = Vec::new();
    let mut hipot_parsed = serde_json::json!({});
    let mut hipot_pass = false;
    let mut plc_pass = false;
    let mut plc_values = serde_json::json!({});
    let mut plc_component_values = serde_json::json!({});

    let _ = db::repo::log_add(
        "INFO", "HIPOT", "cycle_start",
        Some(&format!("Inicio Hipot · exec #{} · receta #{}", execution_id, recipe_id)),
        Some(execution_id), None, None, None, true, None,
    );

    // --- Hipot primero: detecta cortos antes de energizar la prueba eléctrica ---
    let _ = app.emit("test-phase", "HIPOT_RUNNING");
    let hipot_cfg = hipot::config_from(&get_config());
    let (hipot_voltage, hipot_current, hipot_time) =
        db::repo::recipe_hipot_params(recipe_id).unwrap_or((0.0, 0.0, 0.0));
    let _ = db::repo::log_add(
        "INFO", "HIPOT", "recipe_params",
        Some(&format!(
            "Receta #{} · V={} · I={} · T={}",
            recipe_id, hipot_voltage, hipot_current, hipot_time
        )),
        Some(execution_id), None, None, None, true, None,
    );

    if config_flag("hipot_enabled") {
    match hipot::run_test(&hipot_cfg, hipot_voltage, hipot_current, hipot_time) {
        Ok(v) => {
            hipot_parsed = v
                .get("result")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            hipot_pass = hipot_parsed
                .get("pass")
                .and_then(|p| p.as_bool())
                .unwrap_or(false);
        }
        Err(e) => {
            errors.push(format!("Hipot: {}", e));
            hipot_pass = false;
        }
    }
    } else {
        errors.push("Hipot no habilitado. Actívelo en Configuración → Hipot.".to_string());
        hipot_pass = false;
    }

    let hipot_duration = hipot_parsed
        .get("duration")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| started.elapsed().as_secs_f64());
    let _ = save_hipot_result(
        execution_id,
        serde_json::json!({
            "pass": hipot_pass,
            "voltage": hipot_parsed.get("voltage"),
            "current": hipot_parsed.get("current"),
            "resistance": hipot_parsed.get("resistance"),
            "insulation": hipot_parsed.get("insulation"),
            "duration": hipot_duration,
        }),
    );

    let parsed_text = hipot_parsed.to_string();
    let _ = db::repo::log_add(
        if hipot_pass { "INFO" } else { "ERROR" },
        "HIPOT",
        "result",
        Some(&format!(
            "Hipot {} · V={} I={} R={}",
            if hipot_pass { "PASS" } else { "FAIL" },
            hipot_parsed.get("voltage").and_then(|v| v.as_f64()).unwrap_or(0.0),
            hipot_parsed.get("current").and_then(|v| v.as_f64()).unwrap_or(0.0),
            hipot_parsed.get("resistance").and_then(|v| v.as_f64()).unwrap_or(0.0),
        )),
        Some(execution_id),
        None,
        Some(parsed_text.as_str()),
        Some(hipot_duration as i64 * 1000),
        hipot_pass,
        None,
    );

    if !hipot_pass {
        let _ = db::repo::set_final_status(execution_id, "BAD");
        let duration = started.elapsed().as_secs_f64();
        return serde_json::json!({
            "execution_id": execution_id,
            "recipe_id": recipe_id,
            "voltage": voltage,
            "plc": false,
            "hipot": false,
            "final": "BAD",
            "values": {},
            "plc_results": {},
            "hipot_data": hipot_parsed,
            "duration": duration,
            "errors": errors,
            "skipped_electrical": true,
        });
    }

    // --- Prueba eléctrica (solo si Hipot pasó) ---
    let _ = app.emit("test-phase", "PLC_RUNNING");
    if config_flag("plc_enabled") {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 2000) {
        Ok(mut client) => {
            if let Err(e) = plc::send_recipe(&mut client, voltage, &components, hipot_time) {
                errors.push(format!("PLC receta: {}", e));
            }
            let _ = plc::set_mode(&mut client, 1);
            if let Err(e) = plc::start(&mut client) {
                errors.push(format!("PLC start: {}", e));
            }

            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
            loop {
                let results = plc::read_results(&mut client);
                let mut snapshot = serde_json::Map::new();
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
                        if let Some(value) = results.get(&name) {
                            snapshot.insert(name, value.clone());
                        }
                    }
                }
                if !snapshot.is_empty() {
                    let _ = app.emit("test-values", serde_json::Value::Object(snapshot));
                }
                let complete = results
                    .get("complete")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if complete {
                    plc_values = results;
                    break;
                }
                if std::time::Instant::now() >= deadline {
                    errors.push("PLC: timeout esperando Test_Complete".to_string());
                    plc_values = results;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            let mut readings = Vec::new();
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
                    if !comp.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true) {
                        continue;
                    }
                    let min = comp.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let max = comp.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let value = plc_values
                        .get(&name)
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let ok = value >= min && value <= max;
                    readings.push(serde_json::json!({
                        "component": name,
                        "voltage": voltage.to_string(),
                        "tag": name,
                        "value": value,
                        "min": min,
                        "max": max,
                        "pass": ok,
                    }));
                }
            }

            // El PLC real no tiene un bit Test_Pass: se evalúa con los
            // resultados de corriente contra los límites de la receta.
            let finished = plc_values
                .get("complete")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let plc_error = plc_values
                .get("error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let all_ok = !readings.is_empty()
                && readings
                    .iter()
                    .all(|r| r.get("pass").and_then(|v| v.as_bool()).unwrap_or(false));
            plc_pass = finished && !plc_error && all_ok;
            if let Some(obj) = plc_values.as_object_mut() {
                obj.insert("pass".to_string(), serde_json::json!(plc_pass));
            }

            let mut component_values = serde_json::Map::new();
            for reading in &readings {
                if let Some(name) = reading.get("component").and_then(|v| v.as_str()) {
                    let value = reading
                        .get("value")
                        .cloned()
                        .unwrap_or(serde_json::json!(0.0));
                    component_values.insert(name.to_string(), value);
                }
            }
            plc_component_values = serde_json::Value::Object(component_values);
            let _ = save_plc_result(execution_id, plc_pass, serde_json::json!(readings));
        }
        Err(e) => {
            errors.push(format!("PLC no conectado: {}", e));
            let _ = save_plc_result(execution_id, false, serde_json::json!([]));
        }
    }
    } else {
        errors.push("PLC no habilitado. Actívelo en Configuración → PLC.".to_string());
        let _ = save_plc_result(execution_id, false, serde_json::json!([]));
    }

    let final_status = if plc_pass && hipot_pass { "GOOD" } else { "BAD" };
    let _ = db::repo::set_final_status(execution_id, final_status);
    let duration = started.elapsed().as_secs_f64();
    let _ = db::repo::set_total_duration(execution_id, duration);
    let _ = db::repo::log_add(
        "INFO", "HIPOT", "cycle_end",
        Some(&format!("Resultado final {}", final_status)),
        Some(execution_id), None, None, Some(duration as i64 * 1000),
        final_status == "GOOD", None,
    );

    serde_json::json!({
        "execution_id": execution_id,
        "recipe_id": recipe_id,
        "voltage": voltage,
        "plc": plc_pass,
        "hipot": hipot_pass,
        "final": final_status,
        "values": plc_component_values,
        "plc_results": plc_values,
        "hipot_data": hipot_parsed,
        "duration": duration,
        "errors": errors,
        "skipped_electrical": false,
    })
}

#[tauri::command]
fn template_list() -> serde_json::Value {
    db::repo::templates_list()
}

#[tauri::command]
fn template_get(tid: i64) -> Result<serde_json::Value, String> {
    db::repo::template_get(tid)
}

#[tauri::command]
fn template_create(name: String, width_mm: i64, height_mm: i64, canvas_data: String, is_default: bool) -> Result<i64, String> {
    db::repo::template_create(&name, width_mm, height_mm, &canvas_data, is_default)
}

#[tauri::command]
fn template_update(tid: i64, name: String, width_mm: i64, height_mm: i64, canvas_data: String, is_default: bool) -> Result<String, String> {
    db::repo::template_update(tid, &name, width_mm, height_mm, &canvas_data, is_default)?;
    Ok("saved".to_string())
}

#[tauri::command]
fn template_delete(tid: i64) -> Result<String, String> {
    db::repo::template_delete(tid)?;
    Ok("deleted".to_string())
}

#[tauri::command]
fn config_export() -> Result<String, String> {
    config_io::export()
}

#[tauri::command]
fn config_import(data: String, mode: String) -> Result<serde_json::Value, String> {
    config_io::import(&data, &mode)
}

#[tauri::command]
fn import_export_log_list(limit: i64) -> serde_json::Value {
    db::repo::import_export_log_list(limit)
}

#[tauri::command]
fn history_list(model: Option<String>, serial: Option<String>, start: Option<String>, end: Option<String>, limit: Option<i64>) -> serde_json::Value {
    db::repo::history_list(
        model.as_deref(), serial.as_deref(), start.as_deref(), end.as_deref(),
        limit.unwrap_or(200),
    )
}

#[tauri::command]
fn sync_log_list(limit: Option<i64>) -> serde_json::Value {
    db::repo::sync_log_list(limit.unwrap_or(20))
}

#[tauri::command]
fn local_db_scan_servers() -> serde_json::Value {
    let mut servers: Vec<String> = Vec::new();
    for key in [
        r"HKLM\SOFTWARE\Microsoft\Microsoft SQL Server\Instance Names\SQL",
        r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Microsoft SQL Server\Instance Names\SQL",
    ] {
        if let Ok(output) = std::process::Command::new("reg")
            .args(["query", key])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("HKEY") {
                    continue;
                }
                if let Some(name) = line.split_whitespace().next() {
                    let server = if name.eq_ignore_ascii_case("MSSQLSERVER") {
                        ".".to_string()
                    } else {
                        format!(".\\{}", name)
                    };
                    if !servers.iter().any(|s| s.eq_ignore_ascii_case(&server)) {
                        servers.push(server);
                    }
                }
            }
        }
    }
    if !servers.iter().any(|s| s.eq_ignore_ascii_case(".\\SQLEXPRESS")) {
        servers.push(".\\SQLEXPRESS".to_string());
    }
    servers.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    serde_json::json!({
        "servers": servers,
        "default": ".\\SQLEXPRESS",
    })
}

#[tauri::command]
fn local_db_config_get() -> serde_json::Value {
    let path = db::paths::db_config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("use_sqlite".to_string(), serde_json::json!("0"));
            }
            return value;
        }
    }
    serde_json::json!({
        "use_sqlite": "0",
        "server": db::odbc_local::DEFAULT_DB_SERVER,
        "name": db::odbc_local::DEFAULT_DB_NAME,
        "user": db::odbc_local::DEFAULT_DB_USER,
        "password": db::odbc_local::DEFAULT_DB_PASSWORD,
        "auth_defaults_version": "2",
    })
}

#[tauri::command]
fn local_db_config_set(config: serde_json::Value) -> Result<String, String> {
    let path = db::paths::db_config_path();
    let mut config = config;
    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "auth_defaults_version".to_string(),
            serde_json::json!("2"),
        );
    }
    let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok("saved".to_string())
}

#[tauri::command]
fn local_db_status() -> serde_json::Value {
    let cfg = db::odbc_local::config();
    serde_json::json!({
        "use_sqlite": cfg.use_sqlite,
        "server": cfg.server,
        "name": cfg.name,
    })
}

#[tauri::command]
fn local_db_test_connection() -> Result<String, String> {
    db::odbc_local::test_connection()
}

#[tauri::command]
fn local_db_test_config(config: serde_json::Value) -> Result<String, String> {
    db::odbc_local::test_config(&config)
}

#[tauri::command]
fn local_db_setup(config: serde_json::Value, create_database: bool) -> Result<String, String> {
    let mut config = config;
    if let Some(obj) = config.as_object_mut() {
        obj.insert(
            "auth_defaults_version".to_string(),
            serde_json::json!("2"),
        );
    }
    if create_database {
        db::odbc_local::ensure_database(&config)?;
    }
    db::odbc_local::test_target_config(&config)?;
    let path = db::paths::db_config_path();
    let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok("saved".to_string())
}

#[tauri::command]
fn local_db_ensure_tables() -> Result<serde_json::Value, String> {
    let created = db::odbc_local::ensure_schema()?;
    Ok(serde_json::json!({ "status": "ok", "created": created }))
}

#[tauri::command]
fn local_db_migrate() -> Result<serde_json::Value, String> {
    db::odbc_local::migrate_from_sqlite()
}

#[tauri::command]
fn audit_table_names() -> serde_json::Value {
    let rows = db::odbc_local::with_connection(|conn| {
        db::odbc_local::query_json(
            conn,
            "SELECT name FROM sys.tables WHERE name <> 'sysdiagrams' ORDER BY name",
        )
    })
    .unwrap_or_default();
    let tables = rows
        .iter()
        .filter_map(|row| row.get("name").and_then(|v| v.as_str()).map(str::to_string))
        .collect::<Vec<_>>();
    serde_json::json!({ "tables": tables })
}

#[tauri::command]
fn audit_tables_status(conn_str: String) -> serde_json::Value {
    use odbc_api::Environment;
    let env = match Environment::new() {
        Ok(e) => e,
        Err(e) => {
            return serde_json::json!({
                "connected": false, "table_exists": false, "error": e.to_string()
            })
        }
    };
    let conn = match env.connect_with_connection_string(
        &conn_str,
        odbc_api::ConnectionOptions::default(),
    ) {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({
                "connected": false, "table_exists": false, "error": e.to_string()
            })
        }
    };
    let table_exists = conn.execute("SELECT 1 FROM executions", (), None).is_ok();
    serde_json::json!({ "connected": true, "table_exists": table_exists })
}

#[tauri::command]
fn audit_ensure_tables(conn_str: String) -> Result<serde_json::Value, String> {
    let result = ensure_remote_tables(conn_str);
    if result.starts_with("OK") {
        Ok(serde_json::json!({ "exists": true, "detail": result }))
    } else {
        Err(result)
    }
}

#[tauri::command]
fn barcode_scan(code: String, barcode_type: String, scan_order: i64, execution_id: Option<i64>) -> Result<serde_json::Value, String> {
    db::repo::barcode_scan(&code, &barcode_type, scan_order, execution_id)
}

// --- Recipes (SQLite) ---
#[tauri::command]
fn list_recipes() -> serde_json::Value {
    db::repo::recipes_list()
}

#[tauri::command]
fn get_recipe_by_model(model: String) -> serde_json::Value {
    db::repo::recipe_by_model(&model)
}

#[tauri::command]
fn create_recipe(name: String, model: String, type_: String, product: Option<String>, voltage_levels: String, components: serde_json::Value, totals: serde_json::Value, modified_by: Option<String>, hipot_voltage: Option<f64>, hipot_current_max: Option<f64>, hipot_time: Option<f64>) -> i64 {
    match db::repo::recipe_create(
        &name, &model, &type_, product.as_deref(), &voltage_levels,
        &components, &totals, modified_by.as_deref(),
        hipot_voltage, hipot_current_max, hipot_time,
    ) {
        Ok(id) => id,
        Err(e) => { eprintln!("[recipe_create] {}", e); -1 }
    }
}

#[tauri::command]
fn recipes_restore_defaults() -> serde_json::Value {
    match db::defaults::restore_default_recipes() {
        Ok(v) => serde_json::json!({
            "ok": true,
            "inserted": v.get("inserted").cloned().unwrap_or(serde_json::json!(0)),
            "updated": v.get("updated").cloned().unwrap_or(serde_json::json!(0)),
        }),
        Err(e) => serde_json::json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
fn delete_recipe(id: i64) -> String {
    match db::repo::recipe_delete(id) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

// --- Operators & Users (SQLite) ---
#[tauri::command]
fn list_operators() -> serde_json::Value {
    db::repo::operators_list()
}

#[tauri::command]
fn add_operator(name: String, employee_id: String) -> i64 {
    match db::repo::operator_add(&name, &employee_id) {
        Ok(id) => id,
        Err(e) => { eprintln!("[add_operator] {}", e); -1 }
    }
}

#[tauri::command]
fn list_users() -> serde_json::Value {
    db::repo::users_list()
}

#[tauri::command]
fn add_user(username: String, password: String, role: String) -> i64 {
    match db::repo::user_add(&username, &password, &role) {
        Ok(id) => id,
        Err(e) => { eprintln!("[add_user] {}", e); -1 }
    }
}

#[tauri::command]
fn delete_user(id: i64) -> String {
    match db::repo::user_delete(id) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn auth_login(username: String) -> Result<serde_json::Value, String> {
    db::repo::auth_login(&username)
}

#[tauri::command]
fn user_get(uid: i64) -> Result<serde_json::Value, String> {
    db::repo::user_get(uid)
}

#[tauri::command]
fn user_update(uid: i64, username: Option<String>, role: Option<String>, is_active: Option<bool>) -> Result<String, String> {
    db::repo::user_update(uid, username.as_deref(), role.as_deref(), is_active)?;
    Ok("updated".to_string())
}

// --- Executions (SQLite) ---
#[tauri::command]
fn start_execution(recipe_id: i64, operator_id: Option<i64>, batch: Option<String>, serial_number: Option<String>) -> i64 {
    match db::repo::execution_start(recipe_id, operator_id, batch.as_deref(), serial_number.as_deref()) {
        Ok(id) => id,
        Err(e) => { eprintln!("[start_execution] {}", e); -1 }
    }
}

#[tauri::command]
fn save_plc_result(eid: i64, passed: bool, readings: serde_json::Value) -> String {
    match db::repo::save_plc_result(eid, passed, &readings) {
        Ok(_) => "saved".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn save_hipot_result(eid: i64, body: serde_json::Value) -> String {
    match db::repo::save_hipot_result(eid, &body) {
        Ok(status) => status,
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn list_executions(limit: i64) -> serde_json::Value {
    db::repo::executions_list(limit)
}

#[tauri::command]
fn update_recipe(id: i64, name: String, model: String, type_: String, product: Option<String>, voltage_levels: String, components: serde_json::Value, totals: serde_json::Value, modified_by: Option<String>, hipot_voltage: Option<f64>, hipot_current_max: Option<f64>, hipot_time: Option<f64>) -> String {
    match db::repo::recipe_update(
        id, &name, &model, &type_, product.as_deref(), &voltage_levels,
        &components, &totals, modified_by.as_deref(),
        hipot_voltage, hipot_current_max, hipot_time,
    ) {
        Ok(_) => "updated".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn test_odbc_connection(conn_str: String) -> String {
    use odbc_api::Environment;
    let result = (|| -> Result<String, String> {
        let env = Environment::new().map_err(|e| format!("Error creando entorno ODBC: {}", e))?;
        let conn = env.connect_with_connection_string(&conn_str, odbc_api::ConnectionOptions::default())
            .map_err(|e| format!("Error de conexion: {}", e))?;
        drop(conn);
        drop(env);
        Ok("OK".to_string())
    })();
    result.unwrap_or_else(|e| e)
}

#[tauri::command]
fn execute_sql(conn_str: String, sql: String) -> String {
    use odbc_api::Environment;
    let result = (|| -> Result<String, String> {
        let env = Environment::new().map_err(|e| format!("Error creando entorno ODBC: {}", e))?;
        let conn = env.connect_with_connection_string(&conn_str, odbc_api::ConnectionOptions::default())
            .map_err(|e| format!("Error de conexion: {}", e))?;
        let _ = conn.execute(&sql, (), None).map_err(|e| format!("Error ejecutando SQL: {}", e))?;
        drop(conn);
        drop(env);
        Ok("OK".to_string())
    })();
    result.unwrap_or_else(|e| e)
}

#[tauri::command]
fn ensure_remote_tables(conn_str: String) -> String {
    use odbc_api::Environment;
    let result = (|| -> Result<String, String> {
        let env = Environment::new().map_err(|e| format!("Error creando entorno ODBC: {}", e))?;
        let conn = env.connect_with_connection_string(&conn_str, odbc_api::ConnectionOptions::default())
            .map_err(|e| format!("Error de conexion: {}", e))?;
        let schema = include_str!("../schema.sql");
        let mut count = 0;
        for stmt in schema.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() { continue; }
            conn.execute(stmt, (), None).map_err(|e| format!("Error en statement #{}: {}", count + 1, e))?;
            count += 1;
        }
        drop(conn);
        drop(env);
        Ok(format!("OK: {} statements ejecutados", count))
    })();
    result.unwrap_or_else(|e| e)
}

#[tauri::command]
fn sync_pending(remote_conn_str: String) -> String {
    match db::repo::sync_pending(&remote_conn_str) {
        Ok(report) => format!("OK: {}", report),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn print_zpl_ethernet(ip: String, port: u16, zpl: String) -> String {
    use std::net::TcpStream;
    use std::io::Write;
    match TcpStream::connect((ip.as_str(), port)) {
        Ok(mut stream) => {
            stream.set_write_timeout(Some(std::time::Duration::from_secs(10))).ok();
            match stream.write_all(zpl.as_bytes()) {
                Ok(_) => "OK".to_string(),
                Err(e) => format!("Error enviando: {}", e),
            }
        }
        Err(e) => format!("Error conectando: {}", e),
    }
}

#[tauri::command]
fn print_zpl_windows(printer_name: String, zpl: String) -> String {
    use std::fs;
    use std::process::Command;
    // Write ZPL to temp file
    let temp = std::env::temp_dir().join("hipot_print.zpl");
    if fs::write(&temp, &zpl).is_err() {
        return "Error escribiendo archivo temporal".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        let path_str = temp.to_string_lossy().to_string();
        let result = Command::new("cmd")
            .args(["/C", "print", &format!("/d:{}", printer_name), &path_str])
            .output();
        let _ = fs::remove_file(&temp);
        match result {
            Ok(o) if o.status.success() => "OK".to_string(),
            Ok(o) => format!("Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("Error: {}", e),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = temp;
        "No soportado en esta plataforma".to_string()
    }
}

// --- Logs (SQLite) ---
#[tauri::command]
fn list_logs(category: Option<String>, level: Option<String>, q: Option<String>, limit: i64) -> serde_json::Value {
    db::repo::logs_list(category.as_deref(), level.as_deref(), q.as_deref(), limit)
}

#[tauri::command]
fn log_action(level: String, category: String, action: String, detail: Option<String>, execution_id: Option<i64>, value_sent: Option<String>, value_received: Option<String>, duration_ms: Option<i64>, success: Option<bool>, error_message: Option<String>) -> i64 {
    match db::repo::log_add(
        &level, &category, &action, detail.as_deref(), execution_id,
        value_sent.as_deref(), value_received.as_deref(), duration_ms,
        success.unwrap_or(true), error_message.as_deref(),
    ) {
        Ok(id) => id,
        Err(e) => { eprintln!("[log_action] {}", e); -1 }
    }
}

#[tauri::command]
fn logs_prune(days: Option<i64>, keep: Option<i64>) -> Result<serde_json::Value, String> {
    db::repo::logs_prune(days.unwrap_or(30), keep.unwrap_or(5000))
}

#[tauri::command]
fn canvas_to_zpl(template_json: String, data: serde_json::Value) -> String {
    let t: serde_json::Value = match serde_json::from_str(&template_json) {
        Ok(v) => v,
        Err(_) => return "^XA^XZ".to_string(),
    };
    let mm_to_dots = 203.0 / 25.4;
    let w = (t["label"]["width_mm"].as_f64().unwrap_or(100.0) * mm_to_dots) as i32;
    let h = (t["label"]["height_mm"].as_f64().unwrap_or(60.0) * mm_to_dots) as i32;
    let mut zpl = format!("^XA\n^PW{}\n^LL{}\n\n", w, h);

    let replace = |text: &str| -> String {
        let mut s = text.to_string();
        let reps = [
            ("{modelo}", data.get("model").and_then(|v| v.as_str()).unwrap_or("")),
            ("{serie}", data.get("serial_number").and_then(|v| v.as_str()).unwrap_or("")),
            ("{batch}", data.get("batch_number").and_then(|v| v.as_str()).unwrap_or("")),
            ("{operador}", data.get("operator_name").and_then(|v| v.as_str()).unwrap_or("")),
            ("{fecha}", data.get("date").and_then(|v| v.as_str()).unwrap_or("")),
            ("{hora}", data.get("time").and_then(|v| v.as_str()).unwrap_or("")),
            ("{duracion}", data.get("duration").and_then(|v| v.as_str()).unwrap_or("")),
            ("{estado}", data.get("final_status").and_then(|v| v.as_str()).unwrap_or("")),
            ("{voltaje}", data.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
            ("{fan}", data.get("fan_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{door}", data.get("door_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{frame}", data.get("frame_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{drain}", data.get("drain_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{defrost}", data.get("defrost_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{light}", data.get("light_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{total}", data.get("total_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{plc_status}", data.get("plc_pass").and_then(|v| v.as_str()).unwrap_or("")),
            ("{hipot_status}", data.get("hipot_pass").and_then(|v| v.as_str()).unwrap_or("")),
        ];
        for (k, v) in reps {
            s = s.replace(k, v);
        }
        s
    };

    for obj in t.get("objects").and_then(|v| v.as_array()).unwrap_or(&vec![]) {
        let x = (obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) * mm_to_dots / 10.0) as i32;
        let y = (obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) * mm_to_dots / 10.0) as i32;
        let typ = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match typ {
            "text" => {
                let txt = replace(obj.get("content").and_then(|v| v.as_str()).unwrap_or(""));
                let size = obj.get("font_size").and_then(|v| v.as_i64()).unwrap_or(24) as i32;
                zpl.push_str(&format!("^FO{},{}^A0N,{},{}^FD{}^FS\n", x, y, size, size, txt));
            }
            "barcode" => {
                let data = replace(obj.get("data_source").and_then(|v| v.as_str()).unwrap_or(""));
                let hh = obj.get("height").and_then(|v| v.as_i64()).unwrap_or(80) as i32;
                zpl.push_str(&format!("^FO{},{}^BY3^BCN,{},Y,N,N^FD{}^FS\n", x, y, hh, data));
            }
            "qrcode" => {
                let data = replace(obj.get("data_source").and_then(|v| v.as_str()).unwrap_or(""));
                zpl.push_str(&format!("^FO{},{}^BQ2,3,N,N^FD{}^FS\n", x, y, data));
            }
            "line" => {
                let x2 = (obj.get("x2").and_then(|v| v.as_f64()).unwrap_or(obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) + 100.0) * mm_to_dots / 10.0) as i32;
                let th = obj.get("thickness").and_then(|v| v.as_i64()).unwrap_or(2) as i32;
                zpl.push_str(&format!("^FO{},{}^GB{},{},{}^FS\n", x, y, (x2 - x).max(1), th, th));
            }
            "rect" => {
                let w = (obj.get("w").and_then(|v| v.as_f64()).unwrap_or(100.0) * mm_to_dots / 10.0) as i32;
                let h = (obj.get("h").and_then(|v| v.as_f64()).unwrap_or(40.0) * mm_to_dots / 10.0) as i32;
                let th = obj.get("thickness").and_then(|v| v.as_i64()).unwrap_or(2) as i32;
                zpl.push_str(&format!("^FO{},{}^GB{},{},{}^FS\n", x, y, w.max(1), h.max(1), th));
            }
            _ => {}
        }
    }
    format!("{}\n^XZ", zpl)
}


#[tauri::command]
fn print_test_zebra(printer_ip: String, printer_port: u16, printer_name: String) -> String {
    let data = serde_json::json!({
        "model": "TEST", "serial_number": "12345", "batch_number": "B001",
        "final_status": "GOOD", "date": "", "time": ""
    });
    let zpl = canvas_to_zpl(db::seed::DEFAULT_TEMPLATE_JSON.to_string(), data);
    if !printer_ip.is_empty() {
        print_zpl_ethernet(printer_ip, printer_port, zpl)
    } else {
        print_zpl_windows(printer_name, zpl)
    }
}

#[tauri::command]
fn reprint_zebra(eid: i64, printer_ip: String, printer_port: u16, printer_name: String) -> String {
    let row = db::repo::label_data(eid);
    if row.is_null() {
        return "error: execution not found".to_string();
    }
    let serial = row
        .get("serial_number")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let status = row
        .get("final_status")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let model = row.get("model").and_then(|v| v.as_str()).unwrap_or("");
    let batch = row.get("batch").and_then(|v| v.as_str()).unwrap_or("");
    let created = row.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let data = serde_json::json!({
        "model": model, "serial_number": serial,
        "batch_number": batch, "final_status": status,
        "date": created, "time": ""
    });
    let zpl = canvas_to_zpl(db::seed::DEFAULT_TEMPLATE_JSON.to_string(), data);
    if !printer_ip.is_empty() {
        print_zpl_ethernet(printer_ip, printer_port, zpl)
    } else {
        print_zpl_windows(printer_name, zpl)
    }
}

#[tauri::command]
fn duplicate_recipe(id: i64, new_model: String) -> i64 {
    match db::repo::recipe_duplicate(id, &new_model) {
        Ok(id) => id,
        Err(e) => { eprintln!("[duplicate_recipe] {}", e); -1 }
    }
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            open_external,
            get_config,
            set_config,
            app_health,
            db_retry,
            show_touch_keyboard,
            plc_status,
            plc_status_info,
            plc_probe,
            plc_emergency_stop,
            resolve_model_by_serial,
            plc_config_get,
            plc_config_set,
            plc_tags_list,
            plc_tags_add,
            plc_tags_delete,
            plc_tag_write_by_id,
            plc_tag_read_by_id,
            plc_io_status,
            plc_io_write,
            plc_send_recipe,
            plc_start,
            plc_stop,
            plc_read_results,
            plc_set_mode,
            plc_get_mode,
            plc_test_component,
            plc_abort,
            hipot_list_ports,
            hipot_config_get,
            hipot_config_set,
            hipot_status,
            hipot_probe,
            hipot_prepare,
            hipot_command,
            hipot_test,
            hipot_abort,
            hipot_close,
            plc_read_tag,
            plc_write_tag,
            list_recipes,
            get_recipe_by_model,
            create_recipe,
            update_recipe,
            delete_recipe,
            duplicate_recipe,
            recipes_restore_defaults,
            list_operators,
            add_operator,
            list_users,
            add_user,
            delete_user,
            auth_login,
            user_get,
            user_update,
            start_execution,
            run_test_cycle,
            save_plc_result,
            save_hipot_result,
            list_executions,
            list_logs,
            log_action,
            logs_prune,
            template_list,
            template_get,
            template_create,
            template_update,
            template_delete,
            config_export,
            config_import,
            import_export_log_list,
            barcode_scan,
            history_list,
            sync_log_list,
            local_db_config_get,
            local_db_scan_servers,
            local_db_config_set,
            local_db_status,
            local_db_test_connection,
            local_db_test_config,
            local_db_setup,
            local_db_ensure_tables,
            local_db_migrate,
            audit_table_names,
            audit_tables_status,
            audit_ensure_tables,
            test_odbc_connection,
            execute_sql,
            ensure_remote_tables,
            sync_pending,
            print_zpl_ethernet,
            print_zpl_windows,
            print_test_zebra,
            reprint_zebra,
            canvas_to_zpl
        ])
        .setup(|app| {
            // Credenciales/instancia local por defecto y migración de Windows auth vacía.
            db::odbc_local::ensure_default_config_file();
            // La BD se inicializa en segundo plano. Nunca bloquea el hilo principal
            // ni el splash: si SQL Express tarda o no está, la UI sigue respondiendo.
            initialize_db_in_background();
            // Auto-sync loop: intervalo configurable (sync_interval_minutes)
            std::thread::spawn(move || {
                loop {
                    let cfg = get_config();
                    let minutes = cfg
                        .get("sync_interval_minutes")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(5)
                        .clamp(1, 1440);
                    std::thread::sleep(std::time::Duration::from_secs(minutes * 60));
                    let server = cfg.get("sql_server").and_then(|v| v.as_str()).unwrap_or("");
                    if !server.is_empty() {
                        let db_name = cfg.get("sql_database").and_then(|v| v.as_str()).unwrap_or("HiPotDB_Central");
                        let user = cfg.get("sql_user").and_then(|v| v.as_str()).unwrap_or("");
                        let pwd = cfg.get("sql_password").and_then(|v| v.as_str()).unwrap_or("");
                        let driver = "ODBC Driver 17 for SQL Server";
                        let conn_str = if user.is_empty() {
                            format!("Driver={{{}}};Server={};Database={};Trusted_Connection=yes;", driver, server, db_name)
                        } else {
                            format!("Driver={{{}}};Server={};Database={};Uid={};Pwd={};", driver, server, db_name, user, pwd)
                        };
                        let _ = sync_pending(conn_str);
                    }
                }
            });

            // Splash -> main: temporizado en Rust para que nunca se atore.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut shown = false;
                for _ in 0..12 {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    if let Some(main) = handle.get_webview_window("main") {
                        let _ = main.show();
                        let _ = main.set_focus();
                        shown = true;
                        break;
                    }
                }
                if !shown {
                    eprintln!("[hipot] no se encontró la ventana principal tras 3s");
                }
                if let Some(splash) = handle.get_webview_window("splash") {
                    let _ = splash.close();
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running HiPotApp");
}
