p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''#[tauri::command]
fn test_odbc_connection(conn_str: String) -> String {
    use odbc_api::Environment;
    let env = match Environment::new() {
        Ok(e) => e,
        Err(e) => return format!("Error creando entorno ODBC: {}", e),
    };
    match env.connect_with_connection_string(&conn_str, odbc_api::ConnectionOptions::default()) {
        Ok(_) => "OK".to_string(),
        Err(e) => format!("Error de conexion: {}", e),
    }
}

#[tauri::command]
fn execute_sql(conn_str: String, sql: String) -> String {
    use odbc_api::Environment;
    let env = match Environment::new() {
        Ok(e) => e,
        Err(e) => return format!("Error creando entorno ODBC: {}", e),
    };
    let conn = match env.connect_with_connection_string(&conn_str, odbc_api::ConnectionOptions::default()) {
        Ok(c) => c,
        Err(e) => return format!("Error de conexion: {}", e),
    };
    match conn.execute(&sql, (), None) {
        Ok(_) => "OK".to_string(),
        Err(e) => format!("Error ejecutando SQL: {}", e),
    }
}

#[tauri::command]
fn ensure_remote_tables(conn_str: String) -> String {
    use odbc_api::Environment;
    let env = match Environment::new() {
        Ok(e) => e,
        Err(e) => return format!("Error creando entorno ODBC: {}", e),
    };
    let conn = match env.connect_with_connection_string(&conn_str, odbc_api::ConnectionOptions::default()) {
        Ok(c) => c,
        Err(e) => return format!("Error de conexion: {}", e),
    };
    let schema = include_str!("../schema.sql");
    let mut count = 0;
    for stmt in schema.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() { continue; }
        match conn.execute(stmt, (), None) {
            Ok(_) => count += 1,
            Err(e) => return format!("Error en statement #{}: {}", count + 1, e),
        }
    }
    format!("OK: {} statements ejecutados", count)
}

'''
s = s.replace(marker, cmds + marker, 1)
# register commands
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, test_odbc_connection, execute_sql, ensure_remote_tables])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")