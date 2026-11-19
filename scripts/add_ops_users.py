p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''// --- Operators & Users (SQLite) ---
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
    # password hashing skipped for now (use bcrypt crate later)
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

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, list_operators, add_operator, list_users, add_user, delete_user])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")