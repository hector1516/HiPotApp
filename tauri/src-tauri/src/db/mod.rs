//! Capa de acceso a la base de datos local.
//!
//! Punto único de entrada: `db::open()`. Abre SQLite, aplica PRAGMAs,
//! garantiza el esquema completo y deja la conexión lista para usar.
//!
//! Cuando se migre a SQL Express (Fase 7) solo cambia la implementación
//! interna de este módulo; los comandos no se tocan.

pub mod config;
pub mod paths;
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

/// Inicialización de arranque: esquema + valores por defecto.
pub fn init() {
    if let Ok(conn) = open() {
        seed::seed_defaults(&conn);
    }
}
