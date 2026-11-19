//! Capa local SQL Express (ODBC).
//!
//! Objetivo de la Fase 7: que la BD local deje de ser SQLite y pase a SQL
//! Express. Este módulo implementa la infraestructura y la migración; los
//! comandos se irán moviendo aquí en 7B/7C.

use odbc_api::handles::DataType;
use odbc_api::{Connection, ConnectionOptions, Cursor, Environment, ResultSetMetadata};
use serde_json::{json, Map, Value};

#[derive(Clone, Debug)]
pub struct LocalConfig {
    pub use_sqlite: bool,
    pub server: String,
    pub name: String,
    pub user: String,
    pub password: String,
}

pub fn config() -> LocalConfig {
    let path = crate::db::paths::db_config_path();
    let value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    let get = |key: &str, default: &str| {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    };
    LocalConfig {
        use_sqlite: get("use_sqlite", "0") == "1",
        server: get("server", ".\\SQLEXPRESS"),
        name: get("name", "HiPotDB"),
        user: get("user", ""),
        password: get("password", ""),
    }
}

impl LocalConfig {
    pub fn connection_string(&self) -> String {
        let driver = "ODBC Driver 17 for SQL Server";
        if self.user.is_empty() {
            format!(
                "Driver={{{}}};Server={};Database={};Trusted_Connection=yes;TrustServerCertificate=yes;",
                driver, self.server, self.name
            )
        } else {
            format!(
                "Driver={{{}}};Server={};Database={};Uid={};Pwd={};TrustServerCertificate=yes;",
                driver, self.server, self.name, self.user, self.password
            )
        }
    }
}

/// Abre una conexión local, ejecuta la operación y la cierra.
pub fn with_connection<T>(
    f: impl FnOnce(&mut Connection<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let cfg = config();
    if cfg.use_sqlite {
        return Err(
            "La BD local está configurada como SQLite. Cambia a SQL Express en Sistema → Base de datos local."
                .to_string(),
        );
    }
    let env = Environment::new().map_err(|e| format!("Error ODBC: {}", e))?;
    let mut conn = env
        .connect_with_connection_string(&cfg.connection_string(), ConnectionOptions::default())
        .map_err(|e| format!("No se pudo conectar a SQL Express: {}", e))?;
    f(&mut conn)
}

pub fn test_connection() -> Result<String, String> {
    with_connection(|conn| {
        query_one(conn, "SELECT 1 AS ok")?;
        Ok("OK".to_string())
    })
}

pub fn execute(conn: &mut Connection<'_>, sql: &str) -> Result<(), String> {
    conn.execute(sql, (), None).map_err(|e| e.to_string())?;
    Ok(())
}

/// Ejecuta una consulta y devuelve filas como objetos JSON.
pub fn query_json(conn: &mut Connection<'_>, sql: &str) -> Result<Vec<Value>, String> {
    let mut cursor = match conn.execute(sql, (), None).map_err(|e| e.to_string())? {
        Some(cursor) => cursor,
        None => return Ok(Vec::new()),
    };

    let cols = cursor.num_result_cols().map_err(|e| e.to_string())?;
    if cols <= 0 {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    let mut types = Vec::new();
    for i in 1..=cols as u16 {
        names.push(cursor.col_name(i).map_err(|e| e.to_string())?);
        types.push(cursor.col_data_type(i).map_err(|e| e.to_string())?);
    }

    let mut rows = Vec::new();
    while let Some(mut row) = cursor.next_row().map_err(|e| e.to_string())? {
        let mut obj = Map::new();
        for i in 1..=cols as u16 {
            let mut buf: Vec<u16> = Vec::new();
            let present = row
                .get_wide_text(i, &mut buf)
                .map_err(|e| e.to_string())?;
            let value = if present {
                value_from_text(&types[(i - 1) as usize], String::from_utf16_lossy(&buf))
            } else {
                Value::Null
            };
            obj.insert(names[(i - 1) as usize].clone(), value);
        }
        rows.push(Value::Object(obj));
    }
    Ok(rows)
}

pub fn query_one(conn: &mut Connection<'_>, sql: &str) -> Result<Option<Value>, String> {
    Ok(query_json(conn, sql)?.into_iter().next())
}

pub fn last_id(conn: &mut Connection<'_>, table: &str) -> Result<i64, String> {
    let rows = query_json(conn, &format!("SELECT MAX(id) AS id FROM [{}]", table))?;
    Ok(rows
        .first()
        .and_then(|row| row.get("id"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0))
}

/// Crea las tablas que falten usando `schema.sql` como fuente.
pub fn ensure_schema() -> Result<usize, String> {
    with_connection(|conn| {
        let schema = include_str!("../../schema.sql");
        let mut created = 0usize;
        for stmt in schema.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            if let Some(table) = create_table_name(stmt) {
                let exists = query_one(
                    conn,
                    &format!(
                        "SELECT COUNT(*) AS n FROM sys.tables WHERE name = '{}'",
                        table.replace('\'', "''")
                    ),
                )?
                .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
                .unwrap_or(0)
                    > 0;
                if exists {
                    continue;
                }
                execute(conn, stmt)?;
                created += 1;
            } else {
                let _ = execute(conn, stmt);
            }
        }
        Ok(created)
    })
}

/// Migra una vez los datos de `hipot_local.db` (SQLite) a SQL Express.
pub fn migrate_from_sqlite() -> Result<Value, String> {
    let sqlite_path = crate::db::paths::db_path();
    if !sqlite_path.exists() {
        return Err(format!(
            "No existe la BD SQLite: {}",
            sqlite_path.display()
        ));
    }
    let src = rusqlite::Connection::open(&sqlite_path).map_err(|e| e.to_string())?;

    let tables = [
        "configurations",
        "recipes",
        "recipe_components",
        "recipe_totals",
        "operators",
        "batches",
        "executions",
        "plc_readings",
        "hipot_results",
        "barcode_logs",
        "users",
        "sync_log",
        "app_logs",
        "ui_state_log",
        "label_templates",
        "label_prints",
        "plc_tags",
        "import_export_logs",
    ];

    let report = with_connection(|conn| {
        let mut report = Map::new();
        for table in tables {
            let migrated = migrate_table(&src, conn, table)?;
            report.insert(table.to_string(), json!(migrated));
        }
        Ok(Value::Object(report))
    })?;

    // Si la BD antigua usaba `configuration` (singular), migrar esa tabla.
    let _ = with_connection(|conn| {
        if sqlite_table_exists(&src, "configuration")
            && !sqlite_table_exists(&src, "configurations")
        {
            migrate_table_named(&src, conn, "configuration", "configurations")?;
        }
        Ok(())
    });

    // Marcar migración en la configuración de SQL Express.
    let _ = with_connection(|conn| {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        execute(
            conn,
            &format!(
                "IF NOT EXISTS (SELECT 1 FROM configurations WHERE config_key = 'sqlite_migrated_at') \
                 INSERT INTO configurations (config_key, config_value, updated_at) \
                 VALUES ('sqlite_migrated_at', '{}', '{}')",
                now, now
            ),
        )
    });

    Ok(report)
}

fn migrate_table(
    src: &rusqlite::Connection,
    target: &mut Connection<'_>,
    table: &str,
) -> Result<i64, String> {
    migrate_table_named(src, target, table, table)
}

fn migrate_table_named(
    src: &rusqlite::Connection,
    target: &mut Connection<'_>,
    source_table: &str,
    target_table: &str,
) -> Result<i64, String> {
    if !sqlite_table_exists(src, source_table) {
        return Ok(0);
    }

    // Si ya hay datos en destino, no volver a migrar esa tabla.
    let existing = query_one(
        target,
        &format!("SELECT COUNT(*) AS n FROM [{}]", target_table),
    )?
    .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
    .unwrap_or(0);
    if existing > 0 {
        return Ok(0);
    }

    let columns = sqlite_columns(src, source_table)?;
    if columns.is_empty() {
        return Ok(0);
    }

    let mut stmt = src
        .prepare(&format!("SELECT * FROM [{}]", source_table))
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    execute(
        target,
        &format!("SET IDENTITY_INSERT [{}] ON", target_table),
    )?;

    let mut count = 0i64;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let mut values = Vec::new();
        for i in 0..columns.len() {
            let value = row.get_ref(i).map_err(|e| e.to_string())?;
            values.push(sql_literal(value));
        }
        let sql = format!(
            "INSERT INTO [{}] ({}) VALUES ({})",
            target_table,
            columns
                .iter()
                .map(|c| format!("[{}]", c))
                .collect::<Vec<_>>()
                .join(", "),
            values.join(", ")
        );
        execute(target, &sql)?;
        count += 1;
    }

    execute(
        target,
        &format!("SET IDENTITY_INSERT [{}] OFF", target_table),
    )?;
    Ok(count)
}

fn sqlite_table_exists(conn: &rusqlite::Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

fn sqlite_columns(conn: &rusqlite::Connection, table: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info([{}])", table))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;
    Ok(rows.flatten().collect())
}

fn sql_literal(value: rusqlite::types::ValueRef<'_>) -> String {
    match value {
        rusqlite::types::ValueRef::Null => "NULL".to_string(),
        rusqlite::types::ValueRef::Integer(v) => v.to_string(),
        rusqlite::types::ValueRef::Real(v) => {
            if v.is_finite() {
                v.to_string()
            } else {
                "NULL".to_string()
            }
        }
        rusqlite::types::ValueRef::Text(bytes) => {
            let s = String::from_utf8_lossy(bytes);
            format!("N'{}'", s.replace('\'', "''"))
        }
        rusqlite::types::ValueRef::Blob(bytes) => {
            let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            format!("0x{}", hex)
        }
    }
}

fn create_table_name(stmt: &str) -> Option<String> {
    let upper = stmt.to_uppercase();
    let idx = upper.find("CREATE TABLE")?;
    let rest = stmt[idx + "CREATE TABLE".len()..].trim();
    let name = rest
        .split(|c: char| c.is_whitespace() || c == '(')
        .next()?
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn value_from_text(data_type: &DataType, text: String) -> Value {
    match data_type {
        DataType::Integer
        | DataType::SmallInt
        | DataType::BigInt
        | DataType::TinyInt => text
            .parse::<i64>()
            .map(Value::from)
            .unwrap_or(Value::String(text)),
        DataType::Real
        | DataType::Double
        | DataType::Float { .. }
        | DataType::Numeric { .. }
        | DataType::Decimal { .. } => text
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .unwrap_or(Value::String(text)),
        DataType::Bit => Value::Bool(!(text == "0" || text.eq_ignore_ascii_case("false"))),
        _ => Value::String(text),
    }
}
