//! Esquema local SQLite.
//!
//! Fuente única de verdad para la BD local. Los comandos ya no crean tablas:
//! llaman a `db::open()`, que ejecuta `ensure_schema()` una vez.
//!
//! `tauri/src-tauri/schema.sql` sigue siendo el esquema de SQL Server para el
//! remoto. Cuando la BD local pase a SQL Express (Fase 7), este módulo se
//! sustituirá por el equivalente ODBC.

use rusqlite::Connection;

const LOCAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS configurations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_key TEXT UNIQUE NOT NULL,
    config_value TEXT NOT NULL,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS recipes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    model TEXT UNIQUE NOT NULL,
    type TEXT DEFAULT 'current_consumption',
    product TEXT,
    voltage_levels TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT,
    updated_at TEXT,
    modified_by TEXT,
    hipot_voltage REAL,
    hipot_current_max REAL,
    hipot_time REAL
);

CREATE TABLE IF NOT EXISTS recipe_components (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    voltage TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    min_value REAL DEFAULT 0,
    max_value REAL DEFAULT 0,
    unit TEXT DEFAULT 'A'
);

CREATE TABLE IF NOT EXISTS recipe_totals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id INTEGER NOT NULL,
    voltage TEXT NOT NULL,
    min_total REAL NOT NULL,
    max_total REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS operators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    employee_id TEXT UNIQUE NOT NULL,
    is_active INTEGER DEFAULT 1,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS batches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_number TEXT UNIQUE NOT NULL,
    operator_id INTEGER,
    scanned_at TEXT
);

CREATE TABLE IF NOT EXISTS executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id INTEGER,
    operator_id INTEGER,
    batch_id INTEGER,
    serial_number TEXT,
    plc_start_time TEXT,
    plc_end_time TEXT,
    plc_pass_fail INTEGER,
    plc_duration REAL,
    hipot_start_time TEXT,
    hipot_end_time TEXT,
    hipot_pass_fail INTEGER,
    hipot_duration REAL,
    final_status TEXT,
    total_duration REAL,
    created_at TEXT,
    synced_to_remote INTEGER DEFAULT 0,
    synced_at TEXT
);

CREATE TABLE IF NOT EXISTS plc_readings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER,
    recipe_id INTEGER,
    component_name TEXT,
    voltage TEXT,
    tag_name TEXT,
    value REAL,
    min_limit REAL,
    max_limit REAL,
    is_pass INTEGER,
    timestamp TEXT
);

CREATE TABLE IF NOT EXISTS hipot_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER,
    recipe_id INTEGER,
    voltage REAL,
    current REAL,
    resistance REAL,
    insulation_resistance REAL,
    test_duration REAL,
    is_pass INTEGER,
    timestamp TEXT
);

CREATE TABLE IF NOT EXISTS barcode_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL,
    barcode_type TEXT,
    scan_order INTEGER,
    execution_id INTEGER,
    scanned_at TEXT
);

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT DEFAULT 'operator',
    is_active INTEGER DEFAULT 1,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    direction TEXT,
    table_name TEXT,
    records_synced INTEGER DEFAULT 0,
    status TEXT,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS app_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT,
    level TEXT NOT NULL,
    category TEXT NOT NULL,
    action TEXT NOT NULL,
    detail TEXT,
    execution_id INTEGER,
    user_id INTEGER,
    value_sent TEXT,
    value_received TEXT,
    duration_ms INTEGER,
    success INTEGER DEFAULT 1,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS ui_state_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT,
    execution_id INTEGER,
    current_step INTEGER,
    current_phase TEXT,
    background_color TEXT,
    message_displayed TEXT
);

CREATE TABLE IF NOT EXISTS label_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    width_mm INTEGER DEFAULT 100,
    height_mm INTEGER DEFAULT 60,
    canvas_data TEXT,
    is_default INTEGER DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS label_prints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER,
    template_id INTEGER,
    printed_by TEXT DEFAULT 'auto',
    printed_at TEXT,
    printer_model TEXT DEFAULT 'Zebra ZT231',
    connection_type TEXT DEFAULT 'usb',
    zpl_data TEXT,
    status TEXT,
    error_message TEXT,
    copies INTEGER DEFAULT 1
);

CREATE TABLE IF NOT EXISTS plc_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    db_number INTEGER DEFAULT 1,
    offset INTEGER DEFAULT 0,
    data_type TEXT DEFAULT 'REAL',
    bit INTEGER DEFAULT 0,
    description TEXT,
    unit TEXT
);

CREATE TABLE IF NOT EXISTS import_export_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL,
    user_id INTEGER,
    file_name TEXT,
    file_size_bytes INTEGER,
    data_summary TEXT,
    mode TEXT,
    conflicts_found INTEGER DEFAULT 0,
    records_imported INTEGER DEFAULT 0,
    status TEXT DEFAULT 'success',
    error_message TEXT,
    performed_at TEXT
);

CREATE TABLE IF NOT EXISTS app_meta (
    meta_key TEXT PRIMARY KEY,
    meta_value TEXT
);

CREATE INDEX IF NOT EXISTS idx_app_logs_timestamp ON app_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_app_logs_category ON app_logs(category);
CREATE INDEX IF NOT EXISTS idx_app_logs_level ON app_logs(level);
CREATE INDEX IF NOT EXISTS idx_app_logs_execution ON app_logs(execution_id);
CREATE INDEX IF NOT EXISTS idx_executions_synced ON executions(synced_to_remote);
CREATE INDEX IF NOT EXISTS idx_readings_execution ON plc_readings(execution_id);
CREATE INDEX IF NOT EXISTS idx_hipot_execution ON hipot_results(execution_id);
CREATE INDEX IF NOT EXISTS idx_barcode_execution ON barcode_logs(execution_id);
CREATE INDEX IF NOT EXISTS idx_label_prints_execution ON label_prints(execution_id);
"#;

/// Garantiza que la BD tenga todas las tablas, columnas e índices.
pub fn ensure_schema(conn: &Connection) {
    let _ = conn.execute_batch(LOCAL_SCHEMA);

    // Migración desde el nombre antiguo `configuration` (singular).
    if table_exists(conn, "configuration") {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO configurations (config_key, config_value)
             SELECT config_key, config_value FROM configuration",
            [],
        );
    }

    // Columnas añadidas después de la primera versión del esquema.
    ensure_column(conn, "recipes", "modified_by", "TEXT");
    ensure_column(conn, "recipes", "hipot_voltage", "REAL");
    ensure_column(conn, "recipes", "hipot_current_max", "REAL");
    ensure_column(conn, "recipes", "hipot_time", "REAL");
    ensure_column(conn, "executions", "synced_at", "TEXT");
    ensure_column(conn, "plc_readings", "timestamp", "TEXT");
    ensure_column(conn, "hipot_results", "timestamp", "TEXT");
    ensure_column(conn, "configurations", "updated_at", "TEXT");
    ensure_column(conn, "app_logs", "user_id", "INTEGER");
}

fn table_exists(conn: &Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) {
    let mut names = Vec::new();
    if let Ok(mut stmt) = conn.prepare(&format!("PRAGMA table_info({})", table)) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) {
            for name in rows.flatten() {
                names.push(name);
            }
        }
    }
    if !names.iter().any(|name| name == column) {
        let _ = conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, ddl),
            [],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_full_schema_and_config_roundtrip() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        ensure_schema(&conn);

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(tables >= 18, "tablas creadas: {}", tables);

        crate::db::config::set_many(
            &conn,
            &serde_json::json!({ "plc_ip": "10.0.0.5" }),
        )
        .unwrap();
        assert_eq!(
            crate::db::config::get(&conn, "plc_ip").as_deref(),
            Some("10.0.0.5")
        );
    }
}
