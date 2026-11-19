p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''#[tauri::command]
fn sync_pending(remote_conn_str: String) -> String {
    use odbc_api::Environment;
    // Read pending executions from local SQLite
    let local_conn = match rusqlite::Connection::open(db_path()) {
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

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, sync_pending])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")