//! Valores por defecto de la aplicación.

use rusqlite::{params, Connection};

/// Plantilla de etiqueta predeterminada (100 x 60 mm).
pub const DEFAULT_TEMPLATE_JSON: &str = r##"{"label":{"width_mm":100,"height_mm":60,"bg_color":"#FFFFFF"},"objects":[{"id":"t1","type":"text","x":50,"y":20,"content":"HIPOT - RESULTADO","font_size":28,"bold":true},{"id":"t2","type":"text","x":50,"y":60,"content":"Modelo: {modelo}","font_size":18},{"id":"t3","type":"text","x":50,"y":85,"content":"Serie: {serie}","font_size":18},{"id":"t4","type":"text","x":50,"y":115,"content":"Estado: {estado}","font_size":20,"bold":true},{"id":"b1","type":"barcode","x":50,"y":300,"data_source":"{modelo}-{serie}-{batch}","height":80},{"id":"q1","type":"qrcode","x":400,"y":280,"size":100,"data_source":"{modelo}|{serie}|{batch}|{estado}"}]}"##;

/// Configuración inicial. Incluye las claves del sidecar anterior y las nuevas.
pub const DEFAULT_CONFIGS: &[(&str, &str)] = &[
    // PLC
    ("plc_ip", "192.168.1.10"),
    ("plc_rack", "0"),
    ("plc_slot", "1"),
    ("plc_enabled", "0"),
    ("estop_input", "I0.0"),
    ("estop_nc", "1"),
    ("estop_source", "DB"),
    ("estop_db", "1001"),
    ("estop_offset", "2"),
    ("estop_bit", "5"),
    // Hipot
    ("hipot_port", "COM3"),
    ("hipot_baud", "38400"),
    ("hipot_enabled", "0"),
    ("hipot_parity", "None"),
    ("hipot_databits", "8"),
    ("hipot_stopbits", "1"),
    // BD local (SQL Express)
    ("db_local_server", ".\\SQLEXPRESS"),
    ("db_local_name", "HiPotDB"),
    ("db_local_user", ""),
    ("db_local_password", ""),
    // BD remota
    ("db_remote_server", "192.168.1.200"),
    ("db_remote_name", "HiPotDB_Central"),
    ("sql_server", "192.168.1.200"),
    ("sql_database", "HiPotDB_Central"),
    ("sql_user", ""),
    ("sql_password", ""),
    // Sync
    ("sync_interval_minutes", "5"),
    ("sync_mode", "local_to_remote"),
    // Impresora Zebra ZT231
    ("printer_model", "Zebra ZT231"),
    ("printer_name", "Zebra ZT231"),
    ("printer_ip", ""),
    ("printer_port", "9100"),
    ("label_width_mm", "100"),
    ("label_height_mm", "60"),
    ("auto_print", "1"),
    ("print_only_good", "1"),
    ("result_hold_seconds", "10"),
];

/// Siembra configuración, usuario ADMIN y plantilla por defecto.
/// Es idempotente: se puede llamar en cada arranque.
pub fn seed_defaults(conn: &Connection) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for (key, value) in DEFAULT_CONFIGS {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO configurations (config_key, config_value, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key, value, now],
        );
    }

    // Usuario administrador. Sin contraseña: el acceso es por gafete/batch.
    let _ = conn.execute(
        "INSERT OR IGNORE INTO users (username, password_hash, role, is_active, created_at)
         VALUES ('ADMIN', '', 'admin', 1, ?1)",
        params![now],
    );

    // Plantilla por defecto.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM label_templates", [], |row| row.get(0))
        .unwrap_or(0);
    if count == 0 {
        let _ = conn.execute(
            "INSERT INTO label_templates
             (name, width_mm, height_mm, canvas_data, is_default, created_at, updated_at)
             VALUES (?1, 100, 60, ?2, 1, ?3, ?3)",
            params!["Predeterminada 100x60", DEFAULT_TEMPLATE_JSON, now],
        );
    }

    let _ = conn.execute(
        "INSERT OR IGNORE INTO app_meta (meta_key, meta_value) VALUES ('schema_version', '1')",
        [],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_defaults_idempotently() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::schema::ensure_schema(&conn);
        seed_defaults(&conn);

        assert_eq!(
            crate::db::config::get(&conn, "plc_ip").as_deref(),
            Some("192.168.1.10")
        );
        let admins: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE username='ADMIN'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(admins, 1);

        let templates: i64 = conn
            .query_row("SELECT COUNT(*) FROM label_templates", [], |row| row.get(0))
            .unwrap();
        assert_eq!(templates, 1);

        // Llamar de nuevo no duplica nada.
        seed_defaults(&conn);
        let templates2: i64 = conn
            .query_row("SELECT COUNT(*) FROM label_templates", [], |row| row.get(0))
            .unwrap();
        assert_eq!(templates2, 1);
    }
}
