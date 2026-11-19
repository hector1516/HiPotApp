p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''// --- Logs (SQLite) ---
#[tauri::command]
fn list_logs(category: Option<String>, level: Option<String>, q: Option<String>, limit: i64) -> serde_json::Value {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS app_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp TEXT, level TEXT, category TEXT, action TEXT, detail TEXT, execution_id INTEGER, user_id INTEGER, value_sent TEXT, value_received TEXT, duration_ms INTEGER, success INTEGER, error_message TEXT)", []);
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
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS app_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp TEXT, level TEXT, category TEXT, action TEXT, detail TEXT, execution_id INTEGER, user_id INTEGER, value_sent TEXT, value_received TEXT, duration_ms INTEGER, success INTEGER, error_message TEXT)", []);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
    conn.execute(
        "INSERT INTO app_logs (timestamp, level, category, action, detail, execution_id, value_sent, value_received, duration_ms, success, error_message) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![now, level, category, action, detail, execution_id, value_sent, value_received, duration_ms, if success.unwrap_or(true) {1} else {0}, error_message],
    ).ok();
    conn.last_insert_rowid()
}

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, list_logs, log_action])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")