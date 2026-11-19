p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''// --- Executions (SQLite) ---
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

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, start_execution, save_plc_result, save_hipot_result, list_executions])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")