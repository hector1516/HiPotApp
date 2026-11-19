// HiPotApp: ventana Tauri + arranque/parada automática del backend Python
// empaquetado como recurso (todo-en-uno, offline-first).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod plc;

use db::paths::db_path;
use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{Manager, State, WindowEvent};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const BACKEND_BIN: &str = "hipot-backend-x86_64-pc-windows-msvc.exe";

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
        "legacy_sidecar": false,
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
fn read_backend_log() -> String {
    std::fs::read_to_string(db::paths::backend_log_path())
        .unwrap_or_else(|_| "No hay log disponible.".to_string())
}

#[tauri::command]
fn restart_backend(app: tauri::AppHandle) -> String {
    // Kill existing
    if let Some(mut child) = app.state::<Backend>().0.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    // Find binary
    if let Some(path) = backend_path(&app) {
        use std::fs::OpenOptions;
        use std::io::Write;
        let log_path = db::paths::backend_log_path();
        let stderr_file = OpenOptions::new().create(true).append(true).open(&log_path).ok();
        let mut cmd = Command::new(&path);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);
        if let Some(f) = stderr_file {
            cmd.stderr(std::process::Stdio::from(f));
        }
        match cmd.spawn() {
            Ok(child) => {
                *app.state::<Backend>().0.lock().unwrap() = Some(child);
                let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
                if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_path) {
                    let _ = writeln!(f, "[{}] backend reiniciado manualmente: {}", ts, path.display());
                }
                "OK: backend reiniciado".to_string()
            }
            Err(e) => {
                let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
                if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_path) {
                    let _ = writeln!(f, "[{}] error reiniciando backend: {}", ts, e);
                }
                format!("Error: {}", e)
            }
        }
    } else {
        "Error: binario del backend no encontrado".to_string()
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
    let cfg = plc::PlcConfig { ip, rack, slot, simulate: false };
    let mut client = plc::connect(&cfg, 2000)?;
    plc::read_tag(&mut client, db, offset, &data_type, bit)
}

#[tauri::command]
fn plc_write_tag(ip: String, rack: u16, slot: u16, db: i32, offset: i32, data_type: String, value: serde_json::Value, bit: i32) -> Result<String, String> {
    let cfg = plc::PlcConfig { ip, rack, slot, simulate: false };
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

// --- PLC admin / I/O / manual (Rust) ---

fn plc_cfg() -> plc::PlcConfig {
    plc::config_from(&get_config())
}

#[tauri::command]
fn plc_config_get() -> serde_json::Value {
    plc::config_to_json(&plc_cfg())
}

#[tauri::command]
fn plc_config_set(ip: String, rack: u16, slot: u16, simulate: bool) -> String {
    set_config(serde_json::json!({
        "plc_ip": ip,
        "plc_rack": rack.to_string(),
        "plc_slot": slot.to_string(),
        "plc_simulate": if simulate { "true" } else { "false" },
    }))
}

#[tauri::command]
fn plc_status_info() -> serde_json::Value {
    let cfg = plc_cfg();
    if cfg.simulate {
        return serde_json::json!({
            "connected": true, "ip": cfg.ip, "rack": cfg.rack,
            "slot": cfg.slot, "simulate": true
        });
    }
    match plc::connect(&cfg, 1500) {
        Ok(_) => serde_json::json!({
            "connected": true, "ip": cfg.ip, "rack": cfg.rack,
            "slot": cfg.slot, "simulate": false
        }),
        Err(e) => serde_json::json!({
            "connected": false, "ip": cfg.ip, "rack": cfg.rack,
            "slot": cfg.slot, "simulate": false, "error": e
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
    let mut client = if cfg.simulate {
        None
    } else {
        plc::connect(&cfg, 1500).ok()
    };
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
                let value = if cfg.simulate {
                    let v = plc::sim::tag(id);
                    match data_type.to_uppercase().as_str() {
                        "BOOL" => serde_json::json!(v != 0.0),
                        "WORD" | "INT" => serde_json::json!(v as i64),
                        _ => serde_json::json!(v),
                    }
                } else if let Some(c) = client.as_mut() {
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
    if cfg.simulate {
        let v = match data_type.to_uppercase().as_str() {
            "BOOL" => {
                if plc::value_as_bool(&value) {
                    1.0
                } else {
                    0.0
                }
            }
            "WORD" | "INT" => plc::value_as_i64(&value) as f64,
            _ => plc::value_as_f64(&value),
        };
        plc::sim::set_tag(tag_id, v);
        return "OK".to_string();
    }
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
    if cfg.simulate {
        return serde_json::json!(plc::sim::tag(tag_id));
    }
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
    if cfg.simulate {
        let mut v = plc::sim_io();
        if let Some(obj) = v.as_object_mut() {
            obj.insert("connected".to_string(), serde_json::json!(true));
            obj.insert("simulate".to_string(), serde_json::json!(true));
        }
        return v;
    }
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let mut v = plc::read_io(&mut c);
            if let Some(obj) = v.as_object_mut() {
                obj.insert("connected".to_string(), serde_json::json!(true));
                obj.insert("simulate".to_string(), serde_json::json!(false));
            }
            v
        }
        Err(e) => {
            let mut v = plc::empty_io();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("connected".to_string(), serde_json::json!(false));
                obj.insert("simulate".to_string(), serde_json::json!(false));
                obj.insert("error".to_string(), serde_json::json!(e));
            }
            v
        }
    }
}

#[tauri::command]
fn plc_io_write(q: String, value: bool) -> String {
    let cfg = plc_cfg();
    if cfg.simulate {
        plc::sim::set_q(&q, value);
        return "OK".to_string();
    }
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
    if cfg.simulate {
        plc::sim::set_recipe(voltage, &components);
        return "OK".to_string();
    }
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
    if cfg.simulate {
        plc::sim::set("result:running", 1.0);
        plc::sim::set("result:complete", 0.0);
        return "OK".to_string();
    }
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
    if cfg.simulate {
        plc::sim::set("result:running", 0.0);
        return "OK".to_string();
    }
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
    if cfg.simulate {
        return plc::sim_results();
    }
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => plc::read_results(&mut c),
        Err(_) => plc::empty_results(),
    }
}

#[tauri::command]
fn plc_set_mode(mode: i64) -> serde_json::Value {
    let cfg = plc_cfg();
    if cfg.simulate {
        plc::sim::set_mode(mode as i32);
        return serde_json::json!({ "mode": plc::sim::mode(), "ok": true });
    }
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let ok = plc::set_mode(&mut c, mode as i32).is_ok();
            let current = plc::get_mode(&mut c).unwrap_or(1);
            serde_json::json!({ "mode": current, "ok": ok })
        }
        Err(_) => serde_json::json!({ "mode": 1, "ok": false }),
    }
}

#[tauri::command]
fn plc_get_mode() -> serde_json::Value {
    let cfg = plc_cfg();
    if cfg.simulate {
        return serde_json::json!({ "mode": plc::sim::mode() });
    }
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => serde_json::json!({ "mode": plc::get_mode(&mut c).unwrap_or(1) }),
        Err(_) => serde_json::json!({ "mode": 1 }),
    }
}

#[tauri::command]
fn plc_test_component(component: String, voltage: f64) -> serde_json::Value {
    let name = component.trim().to_uppercase();
    let cfg = plc_cfg();
    if cfg.simulate {
        let value = plc::sim::test_component(&name);
        return serde_json::json!({
            "component": name,
            "value": value,
            "all": plc::sim_results(),
        });
    }
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
    if cfg.simulate {
        plc::sim::set("result:running", 0.0);
        plc::sim::set_mode(1);
        return "OK".to_string();
    }
    if let Ok(mut c) = plc::connect(&cfg, 1500) {
        let _ = plc::stop(&mut c);
        let _ = plc::set_mode(&mut c, 1);
    }
    "OK".to_string()
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
    let _ = conn.execute("DELETE FROM users WHERE id = ?1", rusqlite::params![id]);
    "deleted".to_string()
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
    if let Some(start) = plc_start {
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
struct Backend(Mutex<Option<Child>>);

fn backend_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    use tauri::path::BaseDirectory;
    // 1. Directorio de recursos (app instalada / bundle)
    if let Ok(p) = app
        .path()
        .resolve(format!("binaries/{BACKEND_BIN}"), BaseDirectory::Resource)
    {
        if p.exists() {
            return Some(p);
        }
    }
    // 2. Junto al .exe o subiendo hasta 4 niveles (cubre target/release → tauri/binaries)
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..5 {
            if let Some(d) = dir.clone() {
                let p = d.join(BACKEND_BIN);
                if p.exists() { return Some(p); }
                let p = d.join(format!("binaries/{BACKEND_BIN}"));
                if p.exists() { return Some(p); }
                dir = d.parent().map(|p| p.to_path_buf());
            }
        }
    }
    // 3. Fallback dev: cwd
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(format!("../binaries/{BACKEND_BIN}"));
        if p.exists() { return Some(p); }
        let p = cwd.join(format!("binaries/{BACKEND_BIN}"));
        if p.exists() { return Some(p); }
    }
    None
}

fn kill_backend(state: &State<Backend>) {
    let mut guard = match state.0.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            open_external,
            read_backend_log,
            get_config,
            set_config,
            app_health,
            show_touch_keyboard,
            restart_backend,
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
            start_execution,
            save_plc_result,
            save_hipot_result,
            list_executions,
            list_logs,
            log_action,
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
        .manage(Backend(Mutex::new(None)))
        .setup(|app| {
            // Esquema + seed una sola vez, antes de que la UI llame comandos.
            db::init();
            match backend_path(app.handle()) {
                Some(path) => {
                    // Redirigir stderr a un archivo de log para diagnóstico
                    use std::fs::OpenOptions;
                    use std::io::Write;
                    let log_path = db::paths::backend_log_path();
                    let stderr_file = OpenOptions::new()
                        .create(true).append(true).open(&log_path).ok();
                    let mut cmd = Command::new(&path);
                    if let Some(f) = stderr_file {
                        cmd.stderr(std::process::Stdio::from(f));
                    }
                    match cmd.spawn() {
                        Ok(child) => {
                            *app.state::<Backend>().0.lock().unwrap() = Some(child);
                            eprintln!("[hipot] backend iniciado: {}", path.display());
                            use std::io::Write;
                            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
                            let _ = writeln!(std::fs::OpenOptions::new().create(true).append(true).open(&log_path).unwrap(), "[{}] backend iniciado: {}", ts, path.display());
                        }
                        Err(e) => {
                            eprintln!("[hipot] no se pudo arrancar el backend: {e}");
                            use std::io::Write;
                            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
                            let _ = writeln!(std::fs::OpenOptions::new().create(true).append(true).open(&log_path).unwrap(), "[{}] error arrancando backend: {}", ts, e);
                        }
                    }
                }
                None => {
                    eprintln!(
                        "[hipot] backend no encontrado; arráncalo manual: uvicorn App.python.main:app"
                    );
                    // Escribir en log
                    use std::fs::OpenOptions;
                    use std::io::Write;
                    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(db::paths::backend_log_path()) {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs().to_string())
                            .unwrap_or_default();
                        let _ = writeln!(f, "[{}] backend no encontrado en ninguna ruta", ts);
                    }
                }
            }
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
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::Destroyed) {
                kill_backend(&window.state::<Backend>());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running HiPotApp");
}
