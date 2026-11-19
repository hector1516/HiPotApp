p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''#[tauri::command]
fn duplicate_recipe(id: i64, new_model: String) -> i64 {
    let conn = match rusqlite::Connection::open(db_path()) {
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

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, list_recipes, get_recipe_by_model, create_recipe, update_recipe, delete_recipe, duplicate_recipe])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")