// HiPotApp: aplicación Tauri 2 + núcleo Rust (PLC, Hipot, BD, impresión).
// Un solo ejecutable, sin sidecar Python ni puertos HTTP.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config_io;
mod db;
mod hipot;
mod plc;

use db::paths::db_path;
use tauri::{Emitter, Manager};
#[tauri::command]
fn get_config() -> serde_json::Value {
    match db::open() {
        Ok(conn) => db::config::get_all(&conn),
        Err(_) => serde_json::json!({}),
    }
}

#[tauri::command]
fn app_health() -> serde_json::Value {
    // Salud del núcleo Rust (sin HTTP). No sondea hardware para no bloquear la UI.
    let db_ok = rusqlite::Connection::open(db_path())
        .and_then(|conn| conn.query_row("SELECT 1", [], |row| row.get::<_, i64>(0)))
        .is_ok();
    let cfg = get_config();
    let plc_ip = cfg
        .get("plc_ip")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    serde_json::json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "runtime": "rust",
        "database": db_ok,
        "database_path": db_path().display().to_string(),
        "plc_configured": !plc_ip.is_empty(),
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    })
}

#[tauri::command]
fn set_config(values: serde_json::Value) -> String {
    match db::open() {
        Ok(conn) => match db::config::set_many(&conn, &values) {
            Ok(_) => "saved".to_string(),
            Err(e) => format!("error: {}", e),
        },
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
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(_) => serde_json::json!({
            "connected": true, "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot
        }),
        Err(e) => serde_json::json!({
            "connected": false, "ip": cfg.ip, "rack": cfg.rack,
            "slot": cfg.slot, "error": e
        }),
    }
}

#[tauri::command]
fn plc_tags_list() -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let cfg = plc_cfg();
    let mut client = plc::connect(&cfg, 1500).ok();
    let mut arr = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, name, db_number, offset, data_type, bit, description, unit \
         FROM plc_tags ORDER BY id",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        }) {
            for (id, name, db_number, offset, data_type, bit, description, unit) in rows.flatten() {
                let value = if let Some(c) = client.as_mut() {
                    plc::read_tag(c, db_number as i32, offset as i32, &data_type, bit as i32)
                        .unwrap_or(serde_json::json!(null))
                } else {
                    serde_json::json!(null)
                };
                arr.push(serde_json::json!({
                    "id": id,
                    "name": name,
                    "db": db_number,
                    "offset": offset,
                    "type": data_type,
                    "bit": bit,
                    "description": description,
                    "unit": unit,
                    "value": value,
                }));
            }
        }
    }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn plc_tags_add(
    name: String,
    db_number: i64,
    offset: i64,
    data_type: String,
    bit: i64,
) -> i64 {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    conn.execute(
        "INSERT INTO plc_tags (name, db_number, offset, data_type, bit) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            name.trim().to_uppercase(),
            db_number,
            offset,
            data_type.trim().to_uppercase(),
            bit
        ],
    )
    .ok();
    conn.last_insert_rowid()
}

#[tauri::command]
fn plc_tags_delete(id: i64) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    match conn.execute("DELETE FROM plc_tags WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_tag_write_by_id(tag_id: i64, value: serde_json::Value) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error: no db".to_string(),
    };
    let tag = conn.query_row(
        "SELECT db_number, offset, data_type, bit FROM plc_tags WHERE id = ?1",
        rusqlite::params![tag_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        },
    );
    let (db_number, offset, data_type, bit) = match tag {
        Ok(v) => v,
        Err(e) => return format!("error: {}", e),
    };
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::write_tag(
            &mut c,
            db_number as i32,
            offset as i32,
            &data_type,
            &value,
            bit as i32,
        ) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_tag_read_by_id(tag_id: i64) -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!(null),
    };
    let tag = conn.query_row(
        "SELECT db_number, offset, data_type, bit FROM plc_tags WHERE id = ?1",
        rusqlite::params![tag_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        },
    );
    let (db_number, offset, data_type, bit) = match tag {
        Ok(v) => v,
        Err(_) => return serde_json::json!(null),
    };
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            plc::read_tag(&mut c, db_number as i32, offset as i32, &data_type, bit as i32)
                .unwrap_or(serde_json::json!(null))
        }
        Err(_) => serde_json::json!(null),
    }
}

#[tauri::command]
fn plc_io_status() -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let mut v = plc::read_io(&mut c);
            if let Some(obj) = v.as_object_mut() {
                obj.insert("connected".to_string(), serde_json::json!(true));
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
        Ok(mut c) => match plc::send_recipe(&mut c, voltage, &components) {
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
    set_config(serde_json::json!({
        "hipot_port": port,
        "hipot_baud": baud.to_string(),
    }))
}

#[tauri::command]
fn hipot_status() -> serde_json::Value {
    let cfg = hipot::config_from(&get_config());
    let ports = hipot::list_ports();
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
    hipot_test("STOP".to_string())
}

#[tauri::command]
fn hipot_close() -> String {
    hipot::close();
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
    let mut plc_pass = false;
    let mut plc_values = serde_json::json!({});
    let mut plc_component_values = serde_json::json!({});

    // --- PLC ---
    let cfg = plc_cfg();
    match plc::connect(&cfg, 2000) {
        Ok(mut client) => {
            if let Err(e) = plc::send_recipe(&mut client, voltage, &components) {
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

            plc_pass = plc_values
                .get("pass")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

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

    let _ = app.emit("test-phase", "HIPOT_RUNNING");

    // --- Hipot ---
    let hipot_cfg = hipot::config_from(&get_config());
    let mut hipot_parsed = serde_json::json!({});
    let mut hipot_pass = false;

    match hipot::test_action(&hipot_cfg, "START") {
        Ok(v) => {
            hipot_parsed = v
                .get("parsed")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            if hipot_parsed
                .get("pass")
                .and_then(|p| p.as_bool())
                .is_none()
            {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    match hipot::command(&hipot_cfg, "STATUS?") {
                        Ok(s) => {
                            let parsed = s
                                .get("parsed")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({}));
                            if parsed.get("pass").and_then(|p| p.as_bool()).is_some() {
                                hipot_parsed = parsed;
                                break;
                            }
                            hipot_parsed = parsed;
                        }
                        Err(e) => errors.push(format!("Hipot status: {}", e)),
                    }
                    if std::time::Instant::now() >= deadline {
                        errors.push("Hipot: timeout esperando PASS/FAIL".to_string());
                        break;
                    }
                }
            }
            hipot_pass = hipot_parsed
                .get("pass")
                .and_then(|p| p.as_bool())
                .unwrap_or(false);
        }
        Err(e) => errors.push(format!("Hipot start: {}", e)),
    }

    let duration = started.elapsed().as_secs_f64();
    let _ = save_hipot_result(
        execution_id,
        serde_json::json!({
            "pass": hipot_pass,
            "voltage": hipot_parsed.get("voltage"),
            "current": hipot_parsed.get("current"),
            "resistance": hipot_parsed.get("resistance"),
            "insulation": hipot_parsed.get("insulation"),
            "duration": duration,
        }),
    );

    let final_status = if plc_pass && hipot_pass { "GOOD" } else { "BAD" };
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
    })
}

#[tauri::command]
fn template_list() -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut arr = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, name, width_mm, height_mm, is_default FROM label_templates ORDER BY id",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        }) {
            for (id, name, width, height, is_default) in rows.flatten() {
                arr.push(serde_json::json!({
                    "id": id, "name": name, "w": width, "h": height,
                    "is_default": is_default != 0,
                }));
            }
        }
    }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn template_get(tid: i64) -> Result<serde_json::Value, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT id, name, width_mm, height_mm, canvas_data, is_default \
         FROM label_templates WHERE id = ?1",
        rusqlite::params![tid],
        |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "width_mm": row.get::<_, i64>(2)?,
                "height_mm": row.get::<_, i64>(3)?,
                "canvas_data": row.get::<_, Option<String>>(4)?,
                "is_default": row.get::<_, i64>(5)? != 0,
            }))
        },
    )
    .map_err(|_| "Plantilla no encontrada".to_string())
}

#[tauri::command]
fn template_create(
    name: String,
    width_mm: i64,
    height_mm: i64,
    canvas_data: String,
    is_default: bool,
) -> Result<i64, String> {
    let mut conn = db::open().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if is_default {
        tx.execute("UPDATE label_templates SET is_default = 0", [])
            .map_err(|e| e.to_string())?;
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    tx.execute(
        "INSERT INTO label_templates
         (name, width_mm, height_mm, canvas_data, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        rusqlite::params![name, width_mm, height_mm, canvas_data, if is_default {1} else {0}, now],
    )
    .map_err(|e| e.to_string())?;
    let id = tx.last_insert_rowid();
    tx.commit().map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
fn template_update(
    tid: i64,
    name: String,
    width_mm: i64,
    height_mm: i64,
    canvas_data: String,
    is_default: bool,
) -> Result<String, String> {
    let mut conn = db::open().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if is_default {
        tx.execute(
            "UPDATE label_templates SET is_default = 0 WHERE id <> ?1",
            rusqlite::params![tid],
        )
        .map_err(|e| e.to_string())?;
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let changed = tx
        .execute(
            "UPDATE label_templates
             SET name = ?1, width_mm = ?2, height_mm = ?3, canvas_data = ?4,
                 is_default = ?5, updated_at = ?6
             WHERE id = ?7",
            rusqlite::params![
                name,
                width_mm,
                height_mm,
                canvas_data,
                if is_default {1} else {0},
                now,
                tid
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Plantilla no encontrada".to_string());
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok("saved".to_string())
}

#[tauri::command]
fn template_delete(tid: i64) -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let changed = conn
        .execute(
            "DELETE FROM label_templates WHERE id = ?1",
            rusqlite::params![tid],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("Plantilla no encontrada".to_string());
    }
    Ok("deleted".to_string())
}

#[tauri::command]
fn config_export() -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    config_io::export(&conn)
}

#[tauri::command]
fn config_import(data: String, mode: String) -> Result<serde_json::Value, String> {
    let mut conn = db::open().map_err(|e| e.to_string())?;
    config_io::import(&mut conn, &data, &mode)
}

#[tauri::command]
fn import_export_log_list(limit: i64) -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut arr = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, operation_type, mode, data_summary, status, performed_at \
         FROM import_export_logs ORDER BY id DESC LIMIT ?1",
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "op": row.get::<_, String>(1)?,
                "mode": row.get::<_, Option<String>>(2)?,
                "summary": row.get::<_, Option<String>>(3)?,
                "status": row.get::<_, Option<String>>(4)?,
                "at": row.get::<_, Option<String>>(5)?,
            }))
        }) {
            for row in rows.flatten() {
                arr.push(row);
            }
        }
    }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn history_list(
    model: Option<String>,
    serial: Option<String>,
    start: Option<String>,
    end: Option<String>,
    limit: Option<i64>,
) -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!({"rows": [], "total": 0, "good": 0, "bad": 0}),
    };

    let mut sql = String::from(
        "SELECT e.id, e.recipe_id, COALESCE(r.model, '-') AS recipe_model, \
         COALESCE(e.serial_number, '-') AS serial, COALESCE(b.batch_number, '-') AS batch, \
         e.plc_pass_fail, e.hipot_pass_fail, COALESCE(e.final_status, '-') AS final, \
         e.created_at, e.total_duration \
         FROM executions e \
         LEFT JOIN recipes r ON r.id = e.recipe_id \
         LEFT JOIN batches b ON b.id = e.batch_id \
         WHERE 1 = 1",
    );
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(s) = serial.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND e.serial_number LIKE ?");
        params.push(rusqlite::types::Value::Text(format!(
            "%{}%",
            s.trim().to_uppercase()
        )));
    }
    if let Some(m) = model.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND r.model LIKE ?");
        params.push(rusqlite::types::Value::Text(format!(
            "%{}%",
            m.trim().to_uppercase()
        )));
    }
    if let Some(s) = start.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND e.created_at >= ?");
        params.push(rusqlite::types::Value::Text(s));
    }
    if let Some(e) = end.filter(|s| !s.trim().is_empty()) {
        if let Ok(day) = chrono::NaiveDate::parse_from_str(&e, "%Y-%m-%d") {
            if let Some(next) = day.succ_opt() {
                sql.push_str(" AND e.created_at < ?");
                params.push(rusqlite::types::Value::Text(
                    next.format("%Y-%m-%d").to_string(),
                ));
            }
        }
    }

    sql.push_str(" ORDER BY e.id DESC LIMIT ?");
    params.push(rusqlite::types::Value::Integer(
        limit.unwrap_or(200).clamp(1, 5000),
    ));

    let mut rows = Vec::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(iter) = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "recipe_id": row.get::<_, Option<i64>>(1)?,
                "recipe_model": row.get::<_, String>(2)?,
                "serial": row.get::<_, String>(3)?,
                "batch": row.get::<_, String>(4)?,
                "plc": row.get::<_, Option<i64>>(5)?.map(|v| v != 0),
                "hipot": row.get::<_, Option<i64>>(6)?.map(|v| v != 0),
                "final": row.get::<_, String>(7)?,
                "created": row.get::<_, Option<String>>(8)?,
                "total_duration": row.get::<_, Option<f64>>(9)?,
            }))
        }) {
            for row in iter.flatten() {
                rows.push(row);
            }
        }
    }

    let good = rows
        .iter()
        .filter(|r| r.get("final").and_then(|v| v.as_str()) == Some("GOOD"))
        .count();
    let bad = rows
        .iter()
        .filter(|r| r.get("final").and_then(|v| v.as_str()) == Some("BAD"))
        .count();
    serde_json::json!({
        "rows": rows,
        "total": rows.len(),
        "good": good,
        "bad": bad,
    })
}

#[tauri::command]
fn sync_log_list(limit: Option<i64>) -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut arr = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, direction, table_name, records_synced, status, error_message \
         FROM sync_log ORDER BY id DESC LIMIT ?1",
    ) {
        if let Ok(rows) = stmt.query_map(
            rusqlite::params![limit.unwrap_or(20).clamp(1, 500)],
            |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "dir": row.get::<_, Option<String>>(1)?,
                    "table": row.get::<_, Option<String>>(2)?,
                    "n": row.get::<_, Option<i64>>(3)?,
                    "status": row.get::<_, Option<String>>(4)?,
                    "err": row.get::<_, Option<String>>(5)?,
                }))
            },
        ) {
            for row in rows.flatten() {
                arr.push(row);
            }
        }
    }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn local_db_config_get() -> serde_json::Value {
    let path = db::paths::db_config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            return value;
        }
    }
    serde_json::json!({
        "use_sqlite": "0",
        "server": ".\\SQLEXPRESS",
        "name": "HiPotDB",
        "user": "",
        "password": "",
    })
}

#[tauri::command]
fn local_db_config_set(config: serde_json::Value) -> Result<String, String> {
    let path = db::paths::db_config_path();
    let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok("saved".to_string())
}

#[tauri::command]
fn audit_table_names() -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!({"tables": []}),
    };
    let mut tables = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name FROM sqlite_master \
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name <> 'app_meta' \
         ORDER BY name",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for table in rows.flatten() {
                tables.push(table);
            }
        }
    }
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
fn barcode_scan(
    code: String,
    barcode_type: String,
    scan_order: i64,
    execution_id: Option<i64>,
) -> Result<serde_json::Value, String> {
    let raw = code;
    let normalized = raw.trim().to_uppercase();
    if normalized.is_empty() {
        return Err("Código vacío".to_string());
    }
    let conn = db::open().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO barcode_logs
         (code, barcode_type, scan_order, execution_id, scanned_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![&normalized, &barcode_type, scan_order, execution_id, &now],
    )
    .map_err(|e| e.to_string())?;

    let _ = log_action(
        "INFO".to_string(),
        "BARCODE".to_string(),
        format!("scan_{}", barcode_type),
        Some(format!(
            "Crudo: '{}' -> Normalizado: '{}'",
            raw, normalized
        )),
        execution_id,
        None,
        Some(normalized.clone()),
        None,
        Some(true),
        None,
    );

    Ok(serde_json::json!({
        "status": "scanned",
        "raw": raw,
        "code": normalized,
        "barcode_type": barcode_type,
        "scan_order": scan_order,
    }))
}

// --- Recipes (SQLite) ---
#[tauri::command]
fn list_recipes() -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut stmt = match conn.prepare("SELECT id, name, model, type, product, voltage_levels, is_active, created_at, updated_at, modified_by FROM recipes ORDER BY id DESC") {
        Ok(s) => s,
        Err(_) => return serde_json::json!([]),
    };
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "model": row.get::<_, String>(2)?,
            "type": row.get::<_, Option<String>>(3)?,
            "product": row.get::<_, Option<String>>(4)?,
            "voltage_levels": row.get::<_, Option<String>>(5)?,
            "is_active": row.get::<_, i64>(6)? == 1,
            "created_at": row.get::<_, Option<String>>(7)?,
            "updated_at": row.get::<_, Option<String>>(8)?,
            "modified_by": row.get::<_, Option<String>>(9)?,
        }))
    });
    let mut arr = Vec::new();
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            arr.push(r);
        }
    }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn get_recipe_by_model(model: String) -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!(null),
    };

    // Get recipe
    let recipe = conn.query_row(
        "SELECT id, name, model, type, product, voltage_levels, is_active, created_at, updated_at, modified_by FROM recipes WHERE model = ?1",
        rusqlite::params![model],
        |row| Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "model": row.get::<_, String>(2)?,
            "type": row.get::<_, Option<String>>(3)?,
            "product": row.get::<_, Option<String>>(4)?,
            "voltage_levels": row.get::<_, Option<String>>(5)?,
            "is_active": row.get::<_, i64>(6)? == 1,
            "created_at": row.get::<_, Option<String>>(7)?,
            "updated_at": row.get::<_, Option<String>>(8)?,
            "modified_by": row.get::<_, Option<String>>(9)?,
        }))
    );
    let recipe = match recipe {
        Ok(r) => r,
        Err(_) => return serde_json::json!(null),
    };
    let rid = recipe.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

    // Get components
    let mut comps = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT name, voltage, enabled, min_value, max_value, unit FROM recipe_components WHERE recipe_id = ?1") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![rid], |row| {
            Ok(serde_json::json!({
                "name": row.get::<_, String>(0)?,
                "voltage": row.get::<_, String>(1)?,
                "enabled": row.get::<_, i64>(2)? == 1,
                "min": row.get::<_, f64>(3)?,
                "max": row.get::<_, f64>(4)?,
                "unit": row.get::<_, Option<String>>(5)?,
            }))
        }) {
            for r in rows.flatten() {
                comps.push(r);
            }
        }
    }

    // Get totals
    let mut totes = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT voltage, min_total, max_total FROM recipe_totals WHERE recipe_id = ?1") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![rid], |row| {
            Ok(serde_json::json!({
                "voltage": row.get::<_, String>(0)?,
                "min": row.get::<_, f64>(1)?,
                "max": row.get::<_, f64>(2)?,
            }))
        }) {
            for r in rows.flatten() {
                totes.push(r);
            }
        }
    }

    let mut result = recipe;
    if let Some(obj) = result.as_object_mut() {
        obj.insert("components".to_string(), serde_json::Value::Array(comps));
        obj.insert("totals".to_string(), serde_json::Value::Array(totes));
    }
    result
}

#[tauri::command]
fn create_recipe(name: String, model: String, type_: String, product: Option<String>, voltage_levels: String, components: serde_json::Value, totals: serde_json::Value, modified_by: Option<String>) -> i64 {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO recipes (name, model, type, product, voltage_levels, is_active, created_at, updated_at, modified_by) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6, ?7)",
        rusqlite::params![name, model, type_, product, voltage_levels, now, modified_by],
    ).ok();
    let rid = conn.last_insert_rowid();

    // Insert components
    if let Some(comps) = components.as_array() {
        for c in comps {
            let comp_name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let voltage = c.get("voltage").and_then(|v| v.as_str()).unwrap_or("");
            let enabled = c.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            let min = c.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max = c.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let unit = c.get("unit").and_then(|v| v.as_str()).unwrap_or("A");
            conn.execute(
                "INSERT INTO recipe_components (recipe_id, name, voltage, enabled, min_value, max_value, unit) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![rid, comp_name, voltage, if enabled {1} else {0}, min, max, unit],
            ).ok();
        }
    }

    // Insert totals
    if let Some(tots) = totals.as_array() {
        for t in tots {
            let voltage = t.get("voltage").and_then(|v| v.as_str()).unwrap_or("");
            let min = t.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max = t.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
            conn.execute(
                "INSERT INTO recipe_totals (recipe_id, voltage, min_total, max_total) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![rid, voltage, min, max],
            ).ok();
        }
    }

    rid
}

#[tauri::command]
fn delete_recipe(id: i64) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let _ = conn.execute("DELETE FROM recipe_components WHERE recipe_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM recipe_totals WHERE recipe_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM recipes WHERE id = ?1", rusqlite::params![id]);
    "deleted".to_string()
}

// --- Operators & Users (SQLite) ---
#[tauri::command]
fn list_operators() -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut stmt = match conn.prepare("SELECT id, name, employee_id, is_active, created_at FROM operators ORDER BY id DESC") {
        Ok(s) => s,
        Err(_) => return serde_json::json!([]),
    };
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "employee_id": row.get::<_, String>(2)?,
            "is_active": row.get::<_, i64>(3)? == 1,
            "created_at": row.get::<_, Option<String>>(4)?,
        }))
    });
    let mut arr = Vec::new();
    if let Ok(rows) = rows { for r in rows.flatten() { arr.push(r); } }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn add_operator(name: String, employee_id: String) -> i64 {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    conn.execute(
        "INSERT INTO operators (name, employee_id, is_active, created_at) VALUES (?1, ?2, 1, ?3)",
        rusqlite::params![name, employee_id, now],
    ).ok();
    conn.last_insert_rowid()
}

#[tauri::command]
fn list_users() -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut stmt = match conn.prepare("SELECT id, username, role, is_active, created_at FROM users ORDER BY id DESC") {
        Ok(s) => s,
        Err(_) => return serde_json::json!([]),
    };
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "username": row.get::<_, String>(1)?,
            "role": row.get::<_, String>(2)?,
            "is_active": row.get::<_, i64>(3)? == 1,
            "created_at": row.get::<_, Option<String>>(4)?,
        }))
    });
    let mut arr = Vec::new();
    if let Ok(rows) = rows { for r in rows.flatten() { arr.push(r); } }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn add_user(username: String, password: String, role: String) -> i64 {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    // password hashing skipped for now (use bcrypt crate later)
    let hash = password.clone();
    conn.execute(
        "INSERT INTO users (username, password_hash, role, is_active, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
        rusqlite::params![username, hash, role, now],
    ).ok();
    conn.last_insert_rowid()
}

#[tauri::command]
fn delete_user(id: i64) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let current = conn.query_row(
        "SELECT role, is_active FROM users WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
    );
    if let Ok((role, active)) = current {
        if role == "admin" && active != 0 {
            let admins: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM users WHERE role = 'admin' AND is_active = 1",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if admins <= 1 {
                return "error: no se puede borrar el último admin activo".to_string();
            }
        }
    }
    match conn.execute("DELETE FROM users WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn auth_login(username: String) -> Result<serde_json::Value, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let uname = username.trim().to_uppercase();
    if uname.is_empty() {
        return Err("Gafete/usuario requerido".to_string());
    }
    let row = conn
        .query_row(
            "SELECT username, role, is_active FROM users WHERE username = ?1",
            rusqlite::params![uname],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .map_err(|_| format!("Gafete no encontrado: {}", uname))?;
    if row.2 != 1 {
        return Err(format!("Usuario inactivo: {}", uname));
    }
    Ok(serde_json::json!({
        "username": row.0,
        "role": row.1,
    }))
}

#[tauri::command]
fn user_get(uid: i64) -> Result<serde_json::Value, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let row = conn
        .query_row(
            "SELECT id, username, role, is_active, created_at FROM users WHERE id = ?1",
            rusqlite::params![uid],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .map_err(|_| "Usuario no encontrado".to_string())?;

    let permissions: Vec<&str> = if row.2 == "admin" {
        vec![
            "Todo lo de operator",
            "Configurar recetas (altas/bajas/cambios)",
            "Gestionar usuarios y permisos",
            "PLC/Hipot/BD, etiquetas, logs, import/export",
            "Ajustar espera de resultado e impresión",
        ]
    } else {
        vec![
            "Ejecutar pruebas (Modelo/Serie/Batch)",
            "Ver resultado BUENO/MALO",
            "Usar modo manual (sin password)",
            "Reimprimir/Log en principal",
        ]
    };

    Ok(serde_json::json!({
        "id": row.0,
        "username": row.1,
        "role": row.2,
        "is_active": row.3 != 0,
        "permissions": permissions,
        "created_at": row.4,
    }))
}

#[tauri::command]
fn user_update(
    uid: i64,
    username: Option<String>,
    role: Option<String>,
    is_active: Option<bool>,
) -> Result<String, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let current = conn
        .query_row(
            "SELECT username, role, is_active FROM users WHERE id = ?1",
            rusqlite::params![uid],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .map_err(|_| "Usuario no encontrado".to_string())?;

    let new_username = username
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| current.0.clone());
    let new_role = role
        .filter(|r| r == "admin" || r == "operator")
        .unwrap_or_else(|| current.1.clone());
    let new_active = is_active.unwrap_or(current.2 != 0);

    if new_username != current.0 {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE username = ?1 AND id <> ?2",
                rusqlite::params![new_username, uid],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if exists > 0 {
            return Err("Usuario ya existe".to_string());
        }
    }

    let active_admins: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE role = 'admin' AND is_active = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if current.1 == "admin"
        && current.2 != 0
        && (new_role != "admin" || !new_active)
        && active_admins <= 1
    {
        return Err("Debe quedar al menos un admin activo".to_string());
    }

    conn.execute(
        "UPDATE users SET username = ?1, role = ?2, is_active = ?3 WHERE id = ?4",
        rusqlite::params![new_username, new_role, if new_active { 1 } else { 0 }, uid],
    )
    .map_err(|e| e.to_string())?;

    Ok("updated".to_string())
}

// --- Executions (SQLite) ---
#[tauri::command]
fn start_execution(recipe_id: i64, operator_id: Option<i64>, batch: Option<String>, serial_number: Option<String>) -> i64 {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    let mut batch_id: Option<i64> = None;
    if let Some(b) = batch {
        let bn = b.trim().to_uppercase();
        if !bn.is_empty() {
            // find or create batch
            let existing: Option<i64> = conn.query_row("SELECT id FROM batches WHERE batch_number = ?1", rusqlite::params![bn], |row| row.get(0)).ok();
            if let Some(id) = existing {
                batch_id = Some(id);
            } else {
                conn.execute("INSERT INTO batches (batch_number, operator_id, scanned_at) VALUES (?1, ?2, ?3)", rusqlite::params![bn, operator_id, now]).ok();
                batch_id = Some(conn.last_insert_rowid());
            }
        }
    }
    let serial = serial_number.map(|s| s.trim().to_uppercase());
    conn.execute(
        "INSERT INTO executions (recipe_id, operator_id, batch_id, serial_number, plc_start_time, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        rusqlite::params![recipe_id, operator_id, batch_id, serial, now],
    ).ok();
    conn.last_insert_rowid()
}

#[tauri::command]
fn save_plc_result(eid: i64, passed: bool, readings: serde_json::Value) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    // update execution
    let ex = conn.query_row("SELECT plc_start_time FROM executions WHERE id = ?1", rusqlite::params![eid], |row| row.get::<_, Option<String>>(0));
    let plc_start = ex.ok().flatten();
    let mut duration = 0.0;
    if let Some(_start) = plc_start {
        // parse not exact, skip duration calc for simplicity
        duration = 0.0;
    }
    conn.execute(
        "UPDATE executions SET plc_end_time = ?1, plc_pass_fail = ?2, plc_duration = ?3 WHERE id = ?4",
        rusqlite::params![now, if passed {1} else {0}, duration, eid],
    ).ok();
    // insert readings
    if let Some(arr) = readings.as_array() {
        for r in arr {
            let comp = r.get("component").and_then(|v| v.as_str()).unwrap_or("");
            let voltage = r.get("voltage").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let tag = r.get("tag").and_then(|v| v.as_str()).unwrap_or("");
            let value = r.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let min = r.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max = r.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let is_pass = r.get("pass").and_then(|v| v.as_bool()).unwrap_or(false);
            let recipe_id: i64 = conn.query_row("SELECT recipe_id FROM executions WHERE id = ?1", rusqlite::params![eid], |row| row.get(0)).unwrap_or(0);
            conn.execute(
                "INSERT INTO plc_readings (execution_id, recipe_id, component_name, voltage, tag_name, value, min_limit, max_limit, is_pass) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![eid, recipe_id, comp, voltage.to_string(), tag, value, min, max, if is_pass {1} else {0}],
            ).ok();
        }
    }
    "saved".to_string()
}

#[tauri::command]
fn save_hipot_result(eid: i64, body: serde_json::Value) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    let hipot_pass = body.get("pass").and_then(|v| v.as_bool()).unwrap_or(false);
    let voltage = body.get("voltage").and_then(|v| v.as_f64());
    let current = body.get("current").and_then(|v| v.as_f64());
    let resistance = body.get("resistance").and_then(|v| v.as_f64());
    let insulation = body.get("insulation").and_then(|v| v.as_f64());
    let duration = body.get("duration").and_then(|v| v.as_f64());
    let recipe_id: i64 = conn.query_row("SELECT recipe_id FROM executions WHERE id = ?1", rusqlite::params![eid], |row| row.get(0)).unwrap_or(0);
    let plc_pass: Option<i64> = conn.query_row("SELECT plc_pass_fail FROM executions WHERE id = ?1", rusqlite::params![eid], |row| row.get(0)).ok();
    conn.execute(
        "UPDATE executions SET hipot_start_time = ?1, hipot_end_time = ?2, hipot_pass_fail = ?3, hipot_duration = ?4, final_status = ?5, total_duration = ?6 WHERE id = ?7",
        rusqlite::params![now, now, if hipot_pass {1} else {0}, duration, if plc_pass.unwrap_or(0) == 1 && hipot_pass {"GOOD"} else {"BAD"}, 0.0, eid],
    ).ok();
    conn.execute(
        "INSERT INTO hipot_results (execution_id, recipe_id, voltage, current, resistance, insulation_resistance, test_duration, is_pass) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![eid, recipe_id, voltage, current, resistance, insulation, duration, if hipot_pass {1} else {0}],
    ).ok();
    let final_status = if plc_pass.unwrap_or(0) == 1 && hipot_pass { "GOOD" } else { "BAD" };
    final_status.to_string()
}

#[tauri::command]
fn list_executions(limit: i64) -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut stmt = match conn.prepare("SELECT id, recipe_id, serial_number, plc_pass_fail, hipot_pass_fail, final_status, created_at FROM executions ORDER BY id DESC LIMIT ?1") {
        Ok(s) => s,
        Err(_) => return serde_json::json!([]),
    };
    let rows = stmt.query_map(rusqlite::params![limit], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "recipe_id": row.get::<_, Option<i64>>(1)?,
            "serial": row.get::<_, Option<String>>(2)?,
            "plc": row.get::<_, Option<i64>>(3)?,
            "hipot": row.get::<_, Option<i64>>(4)?,
            "final": row.get::<_, Option<String>>(5)?,
            "created": row.get::<_, Option<String>>(6)?,
        }))
    });
    let mut arr = Vec::new();
    if let Ok(rows) = rows { for r in rows.flatten() { arr.push(r); } }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn update_recipe(id: i64, name: String, model: String, type_: String, product: Option<String>, voltage_levels: String, components: serde_json::Value, totals: serde_json::Value, modified_by: Option<String>) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    conn.execute(
        "UPDATE recipes SET name = ?1, model = ?2, type = ?3, product = ?4, voltage_levels = ?5, updated_at = ?6, modified_by = ?7 WHERE id = ?8",
        rusqlite::params![name, model, type_, product, voltage_levels, now, modified_by, id],
    ).ok();

    // Delete old components and totals
    let _ = conn.execute("DELETE FROM recipe_components WHERE recipe_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM recipe_totals WHERE recipe_id = ?1", rusqlite::params![id]);

    // Insert components
    if let Some(comps) = components.as_array() {
        for c in comps {
            let comp_name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let voltage = c.get("voltage").and_then(|v| v.as_str()).unwrap_or("");
            let enabled = c.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            let min = c.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max = c.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let unit = c.get("unit").and_then(|v| v.as_str()).unwrap_or("A");
            conn.execute(
                "INSERT INTO recipe_components (recipe_id, name, voltage, enabled, min_value, max_value, unit) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![id, comp_name, voltage, if enabled {1} else {0}, min, max, unit],
            ).ok();
        }
    }

    // Insert totals
    if let Some(tots) = totals.as_array() {
        for t in tots {
            let voltage = t.get("voltage").and_then(|v| v.as_str()).unwrap_or("");
            let min = t.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let max = t.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
            conn.execute(
                "INSERT INTO recipe_totals (recipe_id, voltage, min_total, max_total) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, voltage, min, max],
            ).ok();
        }
    }

    "updated".to_string()
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
    use odbc_api::Environment;
    // Read pending executions from local SQLite
    let local_conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error: cannot open local db".to_string(),
    };
    let mut stmt = match local_conn.prepare("SELECT id, recipe_id, serial_number, final_status FROM executions WHERE synced_to_remote = 0 OR synced_to_remote IS NULL") {
        Ok(s) => s,
        Err(_) => return "error: cannot query local".to_string(),
    };
    let rows: Vec<(i64, Option<i64>, Option<String>, Option<String>)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }).map(|r| r.flatten().collect()).unwrap_or_default();

    if rows.is_empty() {
        return "OK: 0 pendientes".to_string();
    }

    // Connect to remote
    let env = match Environment::new() {
        Ok(e) => e,
        Err(e) => return format!("Error ODBC env: {}", e),
    };
    let remote = match env.connect_with_connection_string(&remote_conn_str, odbc_api::ConnectionOptions::default()) {
        Ok(c) => c,
        Err(e) => return format!("Error conexion remota: {}", e),
    };

    let mut synced = 0;
    for (id, recipe_id, serial, final_status) in rows {
        let sql = format!(
            "INSERT INTO executions (recipe_id, serial_number, final_status, synced_to_remote) VALUES ({}, '{}', '{}', 1)",
            recipe_id.unwrap_or(0),
            serial.unwrap_or_default().replace("'", "''"),
            final_status.unwrap_or_default().replace("'", "''"),
        );
        if remote.execute(&sql, (), None).is_ok() {
            let _ = local_conn.execute("UPDATE executions SET synced_to_remote = 1 WHERE id = ?1", rusqlite::params![id]);
            synced += 1;
        }
    }
    format!("OK: {} sincronizados", synced)
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
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let mut sql = "SELECT id, timestamp, level, category, action, detail, execution_id, value_sent, value_received, duration_ms, success, error_message FROM app_logs WHERE 1=1".to_string();
    if let Some(c) = &category { if !c.is_empty() { sql.push_str(&format!(" AND category = '{}'", c.replace("'", "''"))); } }
    if let Some(l) = &level { if !l.is_empty() { sql.push_str(&format!(" AND level = '{}'", l.replace("'", "''"))); } }
    if let Some(qq) = &q { if !qq.is_empty() { sql.push_str(&format!(" AND detail LIKE '%{}%'", qq.replace("'", "''"))); } }
    sql.push_str(&format!(" ORDER BY id DESC LIMIT {}", limit));
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return serde_json::json!([]),
    };
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "ts": row.get::<_, Option<String>>(1)?,
            "level": row.get::<_, String>(2)?,
            "cat": row.get::<_, String>(3)?,
            "action": row.get::<_, String>(4)?,
            "detail": row.get::<_, Option<String>>(5)?,
            "execution_id": row.get::<_, Option<i64>>(6)?,
            "value_sent": row.get::<_, Option<String>>(7)?,
            "value_received": row.get::<_, Option<String>>(8)?,
            "duration_ms": row.get::<_, Option<i64>>(9)?,
            "success": row.get::<_, Option<i64>>(10)?,
            "error_message": row.get::<_, Option<String>>(11)?,
        }))
    });
    let mut arr = Vec::new();
    if let Ok(rows) = rows { for r in rows.flatten() { arr.push(r); } }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn log_action(level: String, category: String, action: String, detail: Option<String>, execution_id: Option<i64>, value_sent: Option<String>, value_received: Option<String>, duration_ms: Option<i64>, success: Option<bool>, error_message: Option<String>) -> i64 {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    conn.execute(
        "INSERT INTO app_logs (timestamp, level, category, action, detail, execution_id, value_sent, value_received, duration_ms, success, error_message) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![now, level, category, action, detail, execution_id, value_sent, value_received, duration_ms, if success.unwrap_or(true) {1} else {0}, error_message],
    ).ok();
    conn.last_insert_rowid()
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
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error: cannot open db".to_string(),
    };
    let row = conn.query_row(
        "SELECT serial_number, final_status FROM executions WHERE id = ?1",
        rusqlite::params![eid],
        |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?)),
    );
    let (serial, status) = match row {
        Ok(v) => v,
        Err(_) => return "error: execution not found".to_string(),
    };
    let data = serde_json::json!({
        "model": "", "serial_number": serial.unwrap_or_default(),
        "batch_number": "", "final_status": status.unwrap_or_default(),
        "date": "", "time": ""
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
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    // Get original recipe
    let orig = conn.query_row(
        "SELECT name, model, type, product, voltage_levels, modified_by FROM recipes WHERE id = ?1",
        rusqlite::params![id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, Option<String>>(4)?, row.get::<_, Option<String>>(5)?)),
    );
    let (name, _model, rtype, product, voltage_levels, modified_by) = match orig {
        Ok(v) => v,
        Err(_) => return -1,
    };
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    let new_name = format!("{} (copia)", name);
    conn.execute(
        "INSERT INTO recipes (name, model, type, product, voltage_levels, is_active, created_at, updated_at, modified_by) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6, ?7)",
        rusqlite::params![new_name, new_model, rtype, product, voltage_levels, now, modified_by],
    ).ok();
    let new_id = conn.last_insert_rowid();

    // Copy components
    let mut stmt = conn.prepare("SELECT name, voltage, enabled, min_value, max_value, unit FROM recipe_components WHERE recipe_id = ?1").ok();
    if let Some(ref mut s) = stmt {
        let rows: Vec<(String, String, i64, f64, f64, Option<String>)> = s.query_map(rusqlite::params![id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        }).map(|r| r.flatten().collect()).unwrap_or_default();
        for (name, voltage, enabled, min, max, unit) in rows {
            conn.execute(
                "INSERT INTO recipe_components (recipe_id, name, voltage, enabled, min_value, max_value, unit) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![new_id, name, voltage, enabled, min, max, unit],
            ).ok();
        }
    }

    // Copy totals
    let mut stmt = conn.prepare("SELECT voltage, min_total, max_total FROM recipe_totals WHERE recipe_id = ?1").ok();
    if let Some(ref mut s) = stmt {
        let rows: Vec<(String, f64, f64)> = s.query_map(rusqlite::params![id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map(|r| r.flatten().collect()).unwrap_or_default();
        for (voltage, min, max) in rows {
            conn.execute(
                "INSERT INTO recipe_totals (recipe_id, voltage, min_total, max_total) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![new_id, voltage, min, max],
            ).ok();
        }
    }

    new_id
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            open_external,
            get_config,
            set_config,
            app_health,
            show_touch_keyboard,
            plc_status,
            plc_status_info,
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
            local_db_config_set,
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
            // Esquema + seed una sola vez, antes de que la UI llame comandos.
            db::init();
            // Auto-sync loop: cada 5 minutos sincronizar pendientes a SQL Server remoto
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(300));
                    let cfg = get_config();
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
                std::thread::sleep(std::time::Duration::from_millis(2600));
                if let Some(main) = handle.get_webview_window("main") {
                    let _ = main.show();
                    let _ = main.set_focus();
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
