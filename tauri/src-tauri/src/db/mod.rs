//! Capa de datos local.
//!
//! `init()` prepara **SQL Express** (esquema + seed) y es lo que usa la app en
//! producción. `open()` (SQLite) se conserva únicamente para la migración
//! `SQLite -> SQL Express` y para tests.

pub mod config;
pub mod defaults;
pub mod odbc_local;
pub mod paths;
pub mod repo;
pub mod schema;
pub mod seed;

use rusqlite::Connection;

/// Abre la BD local con el esquema garantizado.
pub fn open() -> rusqlite::Result<Connection> {
    let conn = Connection::open(paths::db_path())?;
    // WAL mejora la concurrencia entre el hilo de UI y el de sync.
    let _ = conn.execute_batch(
        "PRAGMA journal_mode=WAL;\
         PRAGMA synchronous=NORMAL;\
         PRAGMA foreign_keys=ON;\
         PRAGMA busy_timeout=5000;",
    );
    schema::ensure_schema(&conn);
    Ok(conn)
}

/// Inicialización de arranque: esquema SQL Express + valores por defecto.
pub fn init() {
    if let Err(e) = odbc_local::ensure_schema() {
        eprintln!("[db] SQL Express no disponible: {}", e);
        return;
    }
    if let Err(e) = seed_express() {
        eprintln!("[db] seed SQL Express: {}", e);
    }
    if let Err(e) = defaults::seed_express() {
        eprintln!("[db] seed recetas/tags de fábrica: {}", e);
    }
    let _ = repo::logs_prune(30, 5000);
}

fn seed_express() -> Result<(), String> {
    use crate::db::seed::{DEFAULT_CONFIGS, DEFAULT_TEMPLATE_JSON};
    use odbc_local::{execute, query_one, sql_bool, sql_text, with_transaction};

    with_transaction(|conn| {
        let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

        // Migración única: el 3865 usa 38400 baud (manual, Apéndice D).
        let baud_migrated = query_one(
            conn,
            "SELECT COUNT(*) AS n FROM configurations WHERE config_key = 'migration_hipot_baud_38400'",
        )?
        .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
        .unwrap_or(0);
        if baud_migrated == 0 {
            execute(
                conn,
                &format!(
                    "UPDATE configurations SET config_value = '38400', updated_at = {now} \
                     WHERE config_key = 'hipot_baud' AND config_value = '9600'",
                    now = now
                ),
            )?;
            execute(
                conn,
                &format!(
                    "INSERT INTO configurations (config_key, config_value, updated_at) \
                     VALUES ('migration_hipot_baud_38400', '1', {now})",
                    now = now
                ),
            )?;
        }

        for (key, value) in DEFAULT_CONFIGS {
            execute(
                conn,
                &format!(
                    "IF NOT EXISTS (SELECT 1 FROM configurations WHERE config_key = {k}) \
                     INSERT INTO configurations (config_key, config_value, updated_at) \
                     VALUES ({k}, {v}, {now})",
                    k = sql_text(key),
                    v = sql_text(value),
                    now = now
                ),
            )?;
        }

        execute(
            conn,
            &format!(
                "IF NOT EXISTS (SELECT 1 FROM users WHERE username = 'ADMIN') \
                 INSERT INTO users (username, password_hash, role, is_active, created_at) \
                 VALUES ('ADMIN', '', 'admin', 1, {})",
                now
            ),
        )?;

        let templates = query_one(conn, "SELECT COUNT(*) AS n FROM label_templates")?
            .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        if templates == 0 {
            execute(
                conn,
                &format!(
                    "INSERT INTO label_templates \
                     (name, width_mm, height_mm, canvas_data, is_default, created_at, updated_at) \
                     VALUES ('Predeterminada 100x60', 100, 60, {}, {}, {}, {})",
                    sql_text(DEFAULT_TEMPLATE_JSON),
                    sql_bool(true),
                    now,
                    now
                ),
            )?;
        }
        Ok(())
    })
}
