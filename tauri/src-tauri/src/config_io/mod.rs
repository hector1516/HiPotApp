//! Export / import de configuración (recetas, configuración, plantillas,
//! usuarios y tags PLC) en JSON. Replica `App/python/config_manager.py`.

use rusqlite::{params, Connection};
use serde_json::{json, Value};

const VERSION: &str = "1.0.0";

pub fn export(conn: &Connection) -> Result<String, String> {
    let now = chrono::Local::now();
    let mut out = json!({
        "metadata": {
            "version": VERSION,
            "exported_at": now.to_rfc3339(),
            "exported_by": null,
            "station_id": "ESTACION-01",
        },
        "recipes": [],
        "configurations": [],
        "label_templates": [],
        "users": [],
        "plc_tags": [],
    });

    let mut recipes = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, name, model, type, product, voltage_levels, is_active, \
                 created_at, updated_at, modified_by FROM recipes ORDER BY id",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "model": row.get::<_, String>(2)?,
                    "type": row.get::<_, Option<String>>(3)?,
                    "product": row.get::<_, Option<String>>(4)?,
                    "voltage_levels": row.get::<_, Option<String>>(5)?,
                    "is_active": row.get::<_, i64>(6)? != 0,
                    "created_at": row.get::<_, Option<String>>(7)?,
                    "updated_at": row.get::<_, Option<String>>(8)?,
                    "modified_by": row.get::<_, Option<String>>(9)?,
                }))
            })
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let rid = row.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let mut recipe = row;
            recipe["components"] = components(conn, rid)?;
            recipe["totals"] = totals(conn, rid)?;
            recipes.push(recipe);
        }
    }
    out["recipes"] = json!(recipes);

    out["configurations"] = query_rows(
        conn,
        "SELECT id, config_key, config_value, updated_at FROM configurations ORDER BY id",
        |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "config_key": row.get::<_, String>(1)?,
                "config_value": row.get::<_, String>(2)?,
                "updated_at": row.get::<_, Option<String>>(3)?,
            }))
        },
        [],
    )?;

    out["label_templates"] = query_rows(
        conn,
        "SELECT id, name, width_mm, height_mm, canvas_data, is_default, \
         created_at, updated_at FROM label_templates ORDER BY id",
        |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "width_mm": row.get::<_, i64>(2)?,
                "height_mm": row.get::<_, i64>(3)?,
                "canvas_data": row.get::<_, Option<String>>(4)?,
                "is_default": row.get::<_, i64>(5)? != 0,
                "created_at": row.get::<_, Option<String>>(6)?,
                "updated_at": row.get::<_, Option<String>>(7)?,
            }))
        },
        [],
    )?;

    out["users"] = query_rows(
        conn,
        "SELECT id, username, password_hash, role, is_active, created_at \
         FROM users ORDER BY id",
        |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "username": row.get::<_, String>(1)?,
                "password_hash": row.get::<_, Option<String>>(2)?,
                "role": row.get::<_, Option<String>>(3)?,
                "is_active": row.get::<_, i64>(4)? != 0,
                "created_at": row.get::<_, Option<String>>(5)?,
            }))
        },
        [],
    )?;

    out["plc_tags"] = query_rows(
        conn,
        "SELECT id, name, db_number, offset, data_type, bit, description, unit \
         FROM plc_tags ORDER BY id",
        |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "db_number": row.get::<_, i64>(2)?,
                "offset": row.get::<_, i64>(3)?,
                "data_type": row.get::<_, String>(4)?,
                "bit": row.get::<_, i64>(5)?,
                "description": row.get::<_, Option<String>>(6)?,
                "unit": row.get::<_, Option<String>>(7)?,
            }))
        },
        [],
    )?;

    let summary = summary_of(&out);
    let _ = conn.execute(
        "INSERT INTO import_export_logs
         (operation_type, data_summary, mode, conflicts_found, records_imported,
          status, performed_at)
         VALUES ('export', ?1, NULL, 0, ?2, 'success', ?3)",
        params![
            summary.to_string(),
            out["recipes"].as_array().map(|a| a.len()).unwrap_or(0) as i64,
            now.format("%Y-%m-%d %H:%M:%S").to_string()
        ],
    );

    serde_json::to_string_pretty(&out).map_err(|e| e.to_string())
}

pub fn import(conn: &mut Connection, data: &str, mode: &str) -> Result<Value, String> {
    let config: Value = serde_json::from_str(data).map_err(|e| format!("JSON inválido: {}", e))?;
    if config.get("metadata").is_none() {
        return Err("Archivo inválido: falta metadata".to_string());
    }
    let mode = match mode {
        "add" | "merge" | "overwrite" => mode,
        _ => "overwrite",
    };

    let mut conflicts = Vec::new();
    if let Some(recipes) = config.get("recipes").and_then(|v| v.as_array()) {
        for recipe in recipes {
            let model = recipe
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if model.is_empty() {
                continue;
            }
            if let Ok(existing_id) = conn.query_row(
                "SELECT id FROM recipes WHERE model = ?1",
                params![model],
                |row| row.get::<_, i64>(0),
            ) {
                conflicts.push(json!({
                    "type": "recipe",
                    "field": "model",
                    "value": model,
                    "existing_id": existing_id,
                }));
            }
        }
    }

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut imported = 0i64;

    if let Some(recipes) = config.get("recipes").and_then(|v| v.as_array()) {
        for recipe in recipes {
            let model = recipe
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if model.is_empty() {
                continue;
            }
            let existing: Option<i64> = tx
                .query_row(
                    "SELECT id FROM recipes WHERE model = ?1",
                    params![model],
                    |row| row.get(0),
                )
                .ok();

            let rid = if let Some(id) = existing {
                if mode == "add" {
                    continue;
                }
                tx.execute(
                    "UPDATE recipes SET name = ?1, type = ?2, product = ?3, \
                     voltage_levels = ?4, is_active = ?5, updated_at = ?6 WHERE id = ?7",
                    params![
                        recipe.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        recipe
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("current_consumption"),
                        recipe.get("product").and_then(|v| v.as_str()),
                        recipe.get("voltage_levels").and_then(|v| v.as_str()),
                        bool_i64(recipe.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true)),
                        now,
                        id
                    ],
                )
                .map_err(|e| e.to_string())?;
                let _ = tx.execute(
                    "DELETE FROM recipe_components WHERE recipe_id = ?1",
                    params![id],
                );
                let _ = tx.execute("DELETE FROM recipe_totals WHERE recipe_id = ?1", params![id]);
                id
            } else {
                tx.execute(
                    "INSERT INTO recipes
                     (name, model, type, product, voltage_levels, is_active,
                      created_at, updated_at, modified_by)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, NULL)",
                    params![
                        recipe.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        model,
                        recipe
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("current_consumption"),
                        recipe.get("product").and_then(|v| v.as_str()),
                        recipe.get("voltage_levels").and_then(|v| v.as_str()),
                        bool_i64(recipe.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true)),
                        now
                    ],
                )
                .map_err(|e| e.to_string())?;
                tx.last_insert_rowid()
            };

            if let Some(components) = recipe.get("components").and_then(|v| v.as_array()) {
                for c in components {
                    tx.execute(
                        "INSERT INTO recipe_components
                         (recipe_id, name, voltage, enabled, min_value, max_value, unit)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            rid,
                            c.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            c.get("voltage").and_then(|v| v.as_str()).unwrap_or(""),
                            bool_i64(c.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)),
                            c.get("min_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            c.get("max_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            c.get("unit").and_then(|v| v.as_str()).unwrap_or("A"),
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }

            if let Some(totals) = recipe.get("totals").and_then(|v| v.as_array()) {
                for t in totals {
                    tx.execute(
                        "INSERT INTO recipe_totals
                         (recipe_id, voltage, min_total, max_total)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![
                            rid,
                            t.get("voltage").and_then(|v| v.as_str()).unwrap_or(""),
                            t.get("min_total").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            t.get("max_total").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
            imported += 1;
        }
    }

    if let Some(configs) = config.get("configurations").and_then(|v| v.as_array()) {
        for c in configs {
            let key = c.get("config_key").and_then(|v| v.as_str()).unwrap_or("");
            if key.is_empty() {
                continue;
            }
            let value = c
                .get("config_value")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            tx.execute(
                "INSERT INTO configurations (config_key, config_value, updated_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(config_key) DO UPDATE SET
                   config_value = excluded.config_value,
                   updated_at = excluded.updated_at",
                params![key, value, now],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    if let Some(templates) = config.get("label_templates").and_then(|v| v.as_array()) {
        for t in templates {
            let is_default = t
                .get("is_default")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_default {
                let _ = tx.execute("UPDATE label_templates SET is_default = 0", []);
            }
            tx.execute(
                "INSERT INTO label_templates
                 (name, width_mm, height_mm, canvas_data, is_default, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![
                    t.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("importada"),
                    t.get("width_mm").and_then(|v| v.as_i64()).unwrap_or(100),
                    t.get("height_mm").and_then(|v| v.as_i64()).unwrap_or(60),
                    t.get("canvas_data").and_then(|v| v.as_str()),
                    bool_i64(is_default),
                    now
                ],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    if let Some(users) = config.get("users").and_then(|v| v.as_array()) {
        for u in users {
            let username = u.get("username").and_then(|v| v.as_str()).unwrap_or("");
            if username.is_empty() {
                continue;
            }
            tx.execute(
                "INSERT OR IGNORE INTO users
                 (username, password_hash, role, is_active, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    username,
                    u.get("password_hash").and_then(|v| v.as_str()).unwrap_or(""),
                    u.get("role").and_then(|v| v.as_str()).unwrap_or("operator"),
                    bool_i64(u.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true)),
                    now
                ],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    if let Some(tags) = config.get("plc_tags").and_then(|v| v.as_array()) {
        for t in tags {
            let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }
            let exists: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM plc_tags WHERE name = ?1",
                    params![name],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if exists == 0 {
                tx.execute(
                    "INSERT INTO plc_tags
                     (name, db_number, offset, data_type, bit, description, unit)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        name,
                        t.get("db_number").and_then(|v| v.as_i64()).unwrap_or(1),
                        t.get("offset").and_then(|v| v.as_i64()).unwrap_or(0),
                        t.get("data_type").and_then(|v| v.as_str()).unwrap_or("REAL"),
                        t.get("bit").and_then(|v| v.as_i64()).unwrap_or(0),
                        t.get("description").and_then(|v| v.as_str()),
                        t.get("unit").and_then(|v| v.as_str()),
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    let summary = summary_of(&config);
    tx.execute(
        "INSERT INTO import_export_logs
         (operation_type, data_summary, mode, conflicts_found, records_imported,
          status, performed_at)
         VALUES ('import', ?1, ?2, ?3, ?4, 'success', ?5)",
        params![
            summary.to_string(),
            mode,
            conflicts.len() as i64,
            imported,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "success",
        "imported": imported,
        "conflicts": conflicts,
    }))
}

fn components(conn: &Connection, recipe_id: i64) -> Result<Value, String> {
    query_rows(
        conn,
        "SELECT id, recipe_id, name, voltage, enabled, min_value, max_value, unit \
         FROM recipe_components WHERE recipe_id = ?1 ORDER BY id",
        move |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "recipe_id": row.get::<_, i64>(1)?,
                "name": row.get::<_, String>(2)?,
                "voltage": row.get::<_, String>(3)?,
                "enabled": row.get::<_, i64>(4)? != 0,
                "min_value": row.get::<_, f64>(5)?,
                "max_value": row.get::<_, f64>(6)?,
                "unit": row.get::<_, Option<String>>(7)?,
            }))
        },
        params![recipe_id],
    )
}

fn totals(conn: &Connection, recipe_id: i64) -> Result<Value, String> {
    query_rows(
        conn,
        "SELECT id, recipe_id, voltage, min_total, max_total \
         FROM recipe_totals WHERE recipe_id = ?1 ORDER BY id",
        move |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "recipe_id": row.get::<_, i64>(1)?,
                "voltage": row.get::<_, String>(2)?,
                "min_total": row.get::<_, f64>(3)?,
                "max_total": row.get::<_, f64>(4)?,
            }))
        },
        params![recipe_id],
    )
}

fn query_rows<P, F>(conn: &Connection, sql: &str, mapper: F, params: P) -> Result<Value, String>
where
    P: rusqlite::Params,
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Value>,
{
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params, mapper).map_err(|e| e.to_string())?;
    Ok(Value::Array(rows.flatten().collect()))
}

fn bool_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn summary_of(config: &Value) -> Value {
    json!({
        "recipes": config.get("recipes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
        "templates": config.get("label_templates").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
        "users": config.get("users").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_import_roundtrip() {
        let source = Connection::open_in_memory().unwrap();
        crate::db::schema::ensure_schema(&source);
        source
            .execute(
                "INSERT INTO recipes (name, model, type, is_active, created_at, updated_at) \
                 VALUES ('Receta', 'M1', 'current_consumption', 1, '2026', '2026')",
                [],
            )
            .unwrap();
        let rid = source.last_insert_rowid();
        source
            .execute(
                "INSERT INTO recipe_components
                 (recipe_id, name, voltage, enabled, min_value, max_value, unit)
                 VALUES (?1, 'FAN', '120V', 1, 0.1, 1.5, 'A')",
                params![rid],
            )
            .unwrap();

        let data = export(&source).unwrap();
        assert!(data.contains("\"model\": \"M1\""));

        let mut target = Connection::open_in_memory().unwrap();
        crate::db::schema::ensure_schema(&target);
        let result = import(&mut target, &data, "overwrite").unwrap();
        assert_eq!(result["imported"], json!(1));

        let recipes: i64 = target
            .query_row("SELECT COUNT(*) FROM recipes WHERE model = 'M1'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(recipes, 1);
        let components: i64 = target
            .query_row("SELECT COUNT(*) FROM recipe_components", [], |row| row.get(0))
            .unwrap();
        assert_eq!(components, 1);
    }
}
