//! Configuración clave/valor sobre la tabla `configurations`.

use rusqlite::{params, Connection};

/// Devuelve toda la configuración como objeto JSON.
pub fn get_all(conn: &Connection) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Ok(mut stmt) = conn.prepare("SELECT config_key, config_value FROM configurations") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            for (key, value) in rows.flatten() {
                map.insert(key, serde_json::Value::String(value));
            }
        }
    }
    serde_json::Value::Object(map)
}

/// Devuelve un valor concreto o `None`.
pub fn get(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT config_value FROM configurations WHERE config_key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

/// Guarda un objeto JSON de pares clave/valor.
pub fn set_many(conn: &Connection, values: &serde_json::Value) -> Result<(), rusqlite::Error> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    if let Some(obj) = values.as_object() {
        for (key, value) in obj {
            let text = value.as_str().map(str::to_string).unwrap_or_else(|| value.to_string());
            conn.execute(
                "INSERT INTO configurations (config_key, config_value, updated_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(config_key) DO UPDATE SET
                   config_value = excluded.config_value,
                   updated_at = excluded.updated_at",
                params![key, text, now],
            )?;
        }
    }
    Ok(())
}
