// HiPotApp: ventana Tauri + arranque/parada automática del backend Python
// empaquetado como recurso (todo-en-uno, offline-first).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{Manager, State, WindowEvent};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const BACKEND_BIN: &str = "hipot-backend-x86_64-pc-windows-msvc.exe";

fn db_path() -> std::path::PathBuf {
    let base = std::env::var("APPDATA")
        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let _ = std::fs::create_dir_all(&base);
    base.join("hipot_local.db")
}

#[tauri::command]
fn get_config() -> serde_json::Value {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!({}),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS configuration (config_key TEXT PRIMARY KEY, config_value TEXT)", []);
    let mut map = serde_json::Map::new();
    if let Ok(mut stmt) = conn.prepare("SELECT config_key, config_value FROM configuration") {
        if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
            for r in rows.flatten() {
                map.insert(r.0, serde_json::Value::String(r.1));
            }
        }
    }
    serde_json::Value::Object(map)
}

#[tauri::command]
fn set_config(values: serde_json::Value) -> String {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return "error: cannot open db".to_string(),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS configuration (config_key TEXT PRIMARY KEY, config_value TEXT)", []);
    if let Some(obj) = values.as_object() {
        for (k, v) in obj {
            let val = v.as_str().unwrap_or(&v.to_string()).to_string();
            let _ = conn.execute(
                "INSERT INTO configuration (config_key, config_value) VALUES (?1, ?2) ON CONFLICT(config_key) DO UPDATE SET config_value = excluded.config_value",
                rusqlite::params![k, val],
            );
        }
    }
    "saved".to_string()
}

#[tauri::command]
fn read_backend_log() -> String {
    let log_dir = std::env::var("APPDATA")
        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_path = log_dir.join("backend.log");
    std::fs::read_to_string(&log_path).unwrap_or_else(|_| "No hay log disponible.".to_string())
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
        use std::fs::{OpenOptions, create_dir_all};
        use std::io::Write;
        let log_dir = std::env::var("APPDATA")
            .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let _ = create_dir_all(&log_dir);
        let log_path = log_dir.join("backend.log");
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
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = ip.parse().map_err(|_| "invalid ip".to_string())?;
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);
    let transport = s7::tcp::Transport::connect(opts).map_err(|e| e.to_string())?;
    let mut client = s7::client::Client::new(transport).map_err(|e| e.to_string())?;

    let dt = data_type.to_uppercase();
    match dt.as_str() {
        "BOOL" => {
            let mut buf = vec![0u8; 1];
            client.ab_read(offset, 1, &mut buf).map_err(|e| e.to_string())?;
            let val = (buf[0] >> bit) & 1 == 1;
            Ok(serde_json::json!(val))
        }
        "WORD" | "INT" => {
            let mut buf = vec![0u8; 2];
            client.ab_read(offset, 2, &mut buf).map_err(|e| e.to_string())?;
            let val = u16::from_be_bytes([buf[0], buf[1]]);
            Ok(serde_json::json!(val))
        }
        "REAL" => {
            let mut buf = vec![0u8; 4];
            client.ag_read(db, offset, 4, &mut buf).map_err(|e| e.to_string())?;
            // Parse float from bytes (big-endian)
            let val = f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            Ok(serde_json::json!(val))
        }
        _ => Err("unsupported type".to_string()),
    }
}

#[tauri::command]
fn plc_write_tag(ip: String, rack: u16, slot: u16, db: i32, offset: i32, data_type: String, value: serde_json::Value, bit: i32) -> Result<String, String> {
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = ip.parse().map_err(|_| "invalid ip".to_string())?;
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);
    let transport = s7::tcp::Transport::connect(opts).map_err(|e| e.to_string())?;
    let mut client = s7::client::Client::new(transport).map_err(|e| e.to_string())?;

    let dt = data_type.to_uppercase();
    match dt.as_str() {
        "BOOL" => {
            let val = value.as_bool().unwrap_or(false);
            let mut buf = vec![0u8; 1];
            client.ab_read(offset, 1, &mut buf).map_err(|e| e.to_string())?;
            if val {
                buf[0] |= 1 << bit;
            } else {
                buf[0] &= !(1 << bit);
            }
            client.ab_write(offset, 1, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "WORD" | "INT" => {
            let val = value.as_i64().unwrap_or(0) as u16;
            let mut buf = val.to_be_bytes().to_vec();
            client.ab_write(offset, 2, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "REAL" => {
            let val = value.as_f64().unwrap_or(0.0) as f32;
            let mut bytes = val.to_be_bytes().to_vec();
            client.ag_write(db, offset, 4, &mut bytes).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        _ => Err("unsupported type".to_string()),
    }
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

#[tauri::command]
fn read_plc_tag(ip: String, rack: u16, slot: u16, db: i64, offset: i64, data_type: String, bit: i64) -> Result<serde_json::Value, String> {
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = ip.parse().map_err(|_| "invalid ip".to_string())?;
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);
    let transport = s7::tcp::Transport::connect(opts).map_err(|e| e.to_string())?;
    let mut client = s7::client::Client::new(transport).map_err(|e| e.to_string())?;
    let dt = data_type.to_uppercase();
    match dt.as_str() {
        "BOOL" => {
            let mut buf = vec![0u8; 1];
            client.ag_read(db as i32, offset as i32, 1, &mut buf).map_err(|e| e.to_string())?;
            let val = (buf[0] >> bit) & 1 == 1;
            Ok(serde_json::json!(val))
        }
        "WORD" | "INT" => {
            let mut buf = vec![0u8; 2];
            client.ag_read(db as i32, offset as i32, 2, &mut buf).map_err(|e| e.to_string())?;
            let val = u16::from_be_bytes([buf[0], buf[1]]);
            Ok(serde_json::json!(val))
        }
        "REAL" => {
            let mut buf = vec![0u8; 4];
            client.ag_read(db as i32, offset as i32, 4, &mut buf).map_err(|e| e.to_string())?;
            let val = f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            Ok(serde_json::json!(val))
        }
        _ => Err("unsupported type".to_string()),
    }
}

#[tauri::command]
fn write_plc_tag(ip: String, rack: u16, slot: u16, db: i64, offset: i64, data_type: String, value: serde_json::Value, bit: i64) -> Result<String, String> {
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = ip.parse().map_err(|_| "invalid ip".to_string())?;
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);
    let transport = s7::tcp::Transport::connect(opts).map_err(|e| e.to_string())?;
    let mut client = s7::client::Client::new(transport).map_err(|e| e.to_string())?;
    let dt = data_type.to_uppercase();
    match dt.as_str() {
        "BOOL" => {
            let val = value.as_bool().unwrap_or(false);
            let mut buf = vec![0u8; 1];
            client.ag_read(db as i32, offset as i32, 1, &mut buf).map_err(|e| e.to_string())?;
            if val { buf[0] |= 1 << bit; } else { buf[0] &= !(1 << bit); }
            client.ag_write(db as i32, offset as i32, 1, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "WORD" | "INT" => {
            let val = value.as_i64().unwrap_or(0) as u16;
            let mut buf = val.to_be_bytes().to_vec();
            client.ag_write(db as i32, offset as i32, 2, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "REAL" => {
            let val = value.as_f64().unwrap_or(0.0) as f32;
            let mut bytes = val.to_be_bytes().to_vec();
            client.ag_write(db as i32, offset as i32, 4, &mut bytes).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        _ => Err("unsupported type".to_string()),
    }
}

// --- Recipes (SQLite) ---
#[tauri::command]
fn list_recipes() -> serde_json::Value {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipes (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, model TEXT UNIQUE NOT NULL, type TEXT DEFAULT 'current_consumption', product TEXT, voltage_levels TEXT, is_active INTEGER DEFAULT 1, created_at TEXT, updated_at TEXT, modified_by TEXT)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!(null),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipes (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, model TEXT UNIQUE NOT NULL, type TEXT DEFAULT 'current_consumption', product TEXT, voltage_levels TEXT, is_active INTEGER DEFAULT 1, created_at TEXT, updated_at TEXT, modified_by TEXT)", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipe_components (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER NOT NULL, name TEXT NOT NULL, voltage TEXT NOT NULL, enabled INTEGER DEFAULT 1, min_value REAL DEFAULT 0, max_value REAL DEFAULT 0, unit TEXT DEFAULT 'A')", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipe_totals (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER NOT NULL, voltage TEXT NOT NULL, min_total REAL NOT NULL, max_total REAL NOT NULL)", []);

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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipes (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, model TEXT UNIQUE NOT NULL, type TEXT DEFAULT 'current_consumption', product TEXT, voltage_levels TEXT, is_active INTEGER DEFAULT 1, created_at TEXT, updated_at TEXT, modified_by TEXT)", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipe_components (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER NOT NULL, name TEXT NOT NULL, voltage TEXT NOT NULL, enabled INTEGER DEFAULT 1, min_value REAL DEFAULT 0, max_value REAL DEFAULT 0, unit TEXT DEFAULT 'A')", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipe_totals (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER NOT NULL, voltage TEXT NOT NULL, min_total REAL NOT NULL, max_total REAL NOT NULL)", []);

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
    let conn = match rusqlite::Connection::open(db_path()) {
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS operators (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, employee_id TEXT UNIQUE NOT NULL, is_active INTEGER DEFAULT 1, created_at TEXT)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS operators (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, employee_id TEXT UNIQUE NOT NULL, is_active INTEGER DEFAULT 1, created_at TEXT)", []);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    conn.execute(
        "INSERT INTO operators (name, employee_id, is_active, created_at) VALUES (?1, ?2, 1, ?3)",
        rusqlite::params![name, employee_id, now],
    ).ok();
    conn.last_insert_rowid()
}

#[tauri::command]
fn list_users() -> serde_json::Value {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, role TEXT DEFAULT 'operator', is_active INTEGER DEFAULT 1, created_at TEXT)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, role TEXT DEFAULT 'operator', is_active INTEGER DEFAULT 1, created_at TEXT)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let _ = conn.execute("DELETE FROM users WHERE id = ?1", rusqlite::params![id]);
    "deleted".to_string()
}

// --- Executions (SQLite) ---
#[tauri::command]
fn start_execution(recipe_id: i64, operator_id: Option<i64>, batch: Option<String>, serial_number: Option<String>) -> i64 {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER, operator_id INTEGER, batch_id INTEGER, serial_number TEXT, plc_start_time TEXT, plc_end_time TEXT, plc_pass_fail INTEGER, plc_duration REAL, hipot_start_time TEXT, hipot_end_time TEXT, hipot_pass_fail INTEGER, hipot_duration REAL, final_status TEXT, total_duration REAL, created_at TEXT, synced_to_remote INTEGER DEFAULT 0)", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS batches (id INTEGER PRIMARY KEY AUTOINCREMENT, batch_number TEXT UNIQUE NOT NULL, operator_id INTEGER, scanned_at TEXT)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER, operator_id INTEGER, batch_id INTEGER, serial_number TEXT, plc_start_time TEXT, plc_end_time TEXT, plc_pass_fail INTEGER, plc_duration REAL, hipot_start_time TEXT, hipot_end_time TEXT, hipot_pass_fail INTEGER, hipot_duration REAL, final_status TEXT, total_duration REAL, created_at TEXT, synced_to_remote INTEGER DEFAULT 0)", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS plc_readings (id INTEGER PRIMARY KEY AUTOINCREMENT, execution_id INTEGER, recipe_id INTEGER, component_name TEXT, voltage TEXT, tag_name TEXT, value REAL, min_limit REAL, max_limit REAL, is_pass INTEGER)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER, operator_id INTEGER, batch_id INTEGER, serial_number TEXT, plc_start_time TEXT, plc_end_time TEXT, plc_pass_fail INTEGER, plc_duration REAL, hipot_start_time TEXT, hipot_end_time TEXT, hipot_pass_fail INTEGER, hipot_duration REAL, final_status TEXT, total_duration REAL, created_at TEXT, synced_to_remote INTEGER DEFAULT 0)", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS hipot_results (id INTEGER PRIMARY KEY AUTOINCREMENT, execution_id INTEGER, recipe_id INTEGER, voltage REAL, current REAL, resistance REAL, insulation_resistance REAL, test_duration REAL, is_pass INTEGER)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER, operator_id INTEGER, batch_id INTEGER, serial_number TEXT, plc_start_time TEXT, plc_end_time TEXT, plc_pass_fail INTEGER, plc_duration REAL, hipot_start_time TEXT, hipot_end_time TEXT, hipot_pass_fail INTEGER, hipot_duration REAL, final_status TEXT, total_duration REAL, created_at TEXT, synced_to_remote INTEGER DEFAULT 0)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipes (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, model TEXT UNIQUE NOT NULL, type TEXT DEFAULT 'current_consumption', product TEXT, voltage_levels TEXT, is_active INTEGER DEFAULT 1, created_at TEXT, updated_at TEXT, modified_by TEXT)", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipe_components (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER NOT NULL, name TEXT NOT NULL, voltage TEXT NOT NULL, enabled INTEGER DEFAULT 1, min_value REAL DEFAULT 0, max_value REAL DEFAULT 0, unit TEXT DEFAULT 'A')", []);
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS recipe_totals (id INTEGER PRIMARY KEY AUTOINCREMENT, recipe_id INTEGER NOT NULL, voltage TEXT NOT NULL, min_total REAL NOT NULL, max_total REAL NOT NULL)", []);

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
        .invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, restart_backend, list_operators, add_operator, list_users, add_user, delete_user, start_execution, save_plc_result, save_hipot_result, list_executions])
        .manage(Backend(Mutex::new(None)))
        .setup(|app| {
            match backend_path(app.handle()) {
                Some(path) => {
                    // Redirigir stderr a un archivo de log para diagnóstico
                    use std::fs::{OpenOptions, create_dir_all};
                    use std::io::Write;
                    let log_dir = std::env::var("APPDATA")
                        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
                        .unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let _ = create_dir_all(&log_dir);
                    let log_path = log_dir.join("backend.log");
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
                    use std::fs::{OpenOptions, create_dir_all};
                    let log_dir = std::env::var("APPDATA")
                        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
                        .unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let _ = create_dir_all(&log_dir);
                    use std::io::Write;
                    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_dir.join("backend.log")) {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs().to_string())
                            .unwrap_or_default();
                        let _ = writeln!(f, "[{}] backend no encontrado en ninguna ruta", ts);
                    }
                }
            }
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
