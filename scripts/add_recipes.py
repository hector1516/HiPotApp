p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''// --- Recipes (SQLite) ---
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

'''
s = s.replace(marker, cmds + marker, 1)
# register commands
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, list_recipes, get_recipe_by_model, create_recipe, delete_recipe])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")