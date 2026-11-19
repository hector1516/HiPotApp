p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''#[tauri::command]
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

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, list_recipes, get_recipe_by_model, create_recipe, update_recipe, delete_recipe, list_operators, add_operator, list_users, add_user, delete_user, start_execution, save_plc_result, save_hipot_result, list_executions])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")