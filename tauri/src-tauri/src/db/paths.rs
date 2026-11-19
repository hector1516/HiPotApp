//! Rutas únicas de la aplicación.
//!
//! Toda la app (UI, comandos, sync, logs) usa estas rutas. El sidecar Python
//! antiguo usaba otra ubicación; la migración de datos se hace en `schema`.

use std::path::PathBuf;

/// Directorio de datos: `%APPDATA%\HiPotApp`.
pub fn app_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("HiPotApp");
    let _ = std::fs::create_dir_all(&base);
    base
}

/// Base de datos local.
pub fn db_path() -> PathBuf {
    app_dir().join("hipot_local.db")
}

/// Configuración de la BD local/remota (compatible con el sidecar anterior).
pub fn db_config_path() -> PathBuf {
    app_dir().join("db_config.json")
}

/// Log del sidecar anterior (se conserva para diagnóstico durante la transición).
pub fn backend_log_path() -> PathBuf {
    app_dir().join("backend.log")
}

/// Log propio del núcleo Rust.
pub fn app_log_path() -> PathBuf {
    app_dir().join("app.log")
}
