//! Export / import de configuración sobre SQL Express (ODBC).
//!
//! Mantiene el mismo formato JSON que la implementación anterior, pero ya no
//! usa SQLite.

use crate::db::odbc_local::{
    execute, insert_id, query_json, query_one, sql_bool, sql_num, sql_opt, sql_opt_num, sql_text,
    with_connection, with_transaction,
};
use serde_json::{json, Value};

const VERSION: &str = "1.0.0";

pub fn export() -> Result<String, String> {
    with_connection(|conn| {
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

        let mut recipes = query_json(
            conn,
            "SELECT id, name, model, [type], product, voltage_levels, is_active, \
             created_at, updated_at, modified_by, hipot_voltage, hipot_current_max, hipot_time \
             FROM recipes ORDER BY id",
        )?;
        for recipe in recipes.iter_mut() {
            let rid = recipe.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let components = query_json(
                conn,
                &format!(
                    "SELECT id, recipe_id, name, voltage, enabled, min_value, max_value, unit \
                     FROM recipe_components WHERE recipe_id = {} ORDER BY id",
                    rid
                ),
            )?;
            let totals = query_json(
                conn,
                &format!(
                    "SELECT id, recipe_id, voltage, min_total, max_total \
                     FROM recipe_totals WHERE recipe_id = {} ORDER BY id",
                    rid
                ),
            )?;
            if let Some(obj) = recipe.as_object_mut() {
                obj.insert("components".to_string(), Value::Array(components));
                obj.insert("totals".to_string(), Value::Array(totals));
            }
        }
        out["recipes"] = Value::Array(recipes);
        out["configurations"] = Value::Array(query_json(
            conn,
            "SELECT id, config_key, config_value, updated_at FROM configurations ORDER BY id",
        )?);
        out["label_templates"] = Value::Array(query_json(
            conn,
            "SELECT id, name, width_mm, height_mm, canvas_data, is_default, \
             created_at, updated_at FROM label_templates ORDER BY id",
        )?);
        out["users"] = Value::Array(query_json(
            conn,
            "SELECT id, username, password_hash, role, is_active, created_at \
             FROM users ORDER BY id",
        )?);
        out["plc_tags"] = Value::Array(query_json(
            conn,
            "SELECT id, name, db_number, [offset], data_type, bit, description, unit \
             FROM plc_tags ORDER BY id",
        )?);

        let summary = summary_of(&out).to_string();
        let now_sql = sql_text(&now.format("%Y-%m-%d %H:%M:%S").to_string());
        execute(
            conn,
            &format!(
                "INSERT INTO import_export_logs \
                 (operation_type, data_summary, mode, conflicts_found, records_imported, status, performed_at) \
                 VALUES ('export', {}, NULL, 0, {}, 'success', {})",
                sql_text(&summary),
                out["recipes"].as_array().map(|a| a.len()).unwrap_or(0),
                now_sql
            ),
        )?;

        serde_json::to_string_pretty(&out).map_err(|e| e.to_string())
    })
}

pub fn import(data: &str, mode: &str) -> Result<Value, String> {
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
        let _ = with_connection(|conn| {
            for recipe in recipes {
                let model = recipe
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                if model.is_empty() {
                    continue;
                }
                if let Some(existing) = query_one(
                    conn,
                    &format!(
                        "SELECT id FROM recipes WHERE model = {}",
                        sql_text(model)
                    ),
                )? {
                    conflicts.push(json!({
                        "type": "recipe",
                        "field": "model",
                        "value": model,
                        "existing_id": existing.get("id").cloned().unwrap_or(Value::Null),
                    }));
                }
            }
            Ok(())
        });
    }

    let imported = with_transaction(|conn| {
        let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
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
                let existing = query_one(
                    conn,
                    &format!("SELECT id FROM recipes WHERE model = {}", sql_text(model)),
                )?;

                let rid = if let Some(row) = existing {
                    if mode == "add" {
                        continue;
                    }
                    let id = row.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                    execute(
                        conn,
                        &format!(
                            "UPDATE recipes SET name = {}, [type] = {}, product = {}, \
                             voltage_levels = {}, is_active = {}, updated_at = {}, \
                             hipot_voltage = {}, hipot_current_max = {}, hipot_time = {} WHERE id = {}",
                            sql_text(recipe.get("name").and_then(|v| v.as_str()).unwrap_or("")),
                            sql_text(recipe.get("type").and_then(|v| v.as_str()).unwrap_or("current_consumption")),
                            sql_opt(recipe.get("product").and_then(|v| v.as_str())),
                            sql_opt(recipe.get("voltage_levels").and_then(|v| v.as_str())),
                            sql_bool(recipe.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true)),
                            now,
                            sql_opt_num(recipe.get("hipot_voltage").and_then(|v| v.as_f64())),
                            sql_opt_num(recipe.get("hipot_current_max").and_then(|v| v.as_f64())),
                            sql_opt_num(recipe.get("hipot_time").and_then(|v| v.as_f64())),
                            id
                        ),
                    )?;
                    execute(conn, &format!("DELETE FROM recipe_components WHERE recipe_id = {}", id))?;
                    execute(conn, &format!("DELETE FROM recipe_totals WHERE recipe_id = {}", id))?;
                    id
                } else {
                    insert_id(
                        conn,
                        &format!(
                            "INSERT INTO recipes \
                             (name, model, [type], product, voltage_levels, is_active, created_at, updated_at, \
                              hipot_voltage, hipot_current_max, hipot_time) \
                             VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                            sql_text(recipe.get("name").and_then(|v| v.as_str()).unwrap_or("")),
                            sql_text(model),
                            sql_text(recipe.get("type").and_then(|v| v.as_str()).unwrap_or("current_consumption")),
                            sql_opt(recipe.get("product").and_then(|v| v.as_str())),
                            sql_opt(recipe.get("voltage_levels").and_then(|v| v.as_str())),
                            sql_bool(recipe.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true)),
                            now,
                            now,
                            sql_opt_num(recipe.get("hipot_voltage").and_then(|v| v.as_f64())),
                            sql_opt_num(recipe.get("hipot_current_max").and_then(|v| v.as_f64())),
                            sql_opt_num(recipe.get("hipot_time").and_then(|v| v.as_f64()))
                        ),
                    )?
                };

                if let Some(components) = recipe.get("components").and_then(|v| v.as_array()) {
                    for c in components {
                        execute(
                            conn,
                            &format!(
                                "INSERT INTO recipe_components \
                                 (recipe_id, name, voltage, enabled, min_value, max_value, unit) \
                                 VALUES ({}, {}, {}, {}, {}, {}, {})",
                                rid,
                                sql_text(c.get("name").and_then(|v| v.as_str()).unwrap_or("")),
                                sql_text(c.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
                                sql_bool(c.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)),
                                sql_num(c.get("min_value").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                                sql_num(c.get("max_value").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                                sql_text(c.get("unit").and_then(|v| v.as_str()).unwrap_or("A"))
                            ),
                        )?;
                    }
                }
                if let Some(totals) = recipe.get("totals").and_then(|v| v.as_array()) {
                    for t in totals {
                        execute(
                            conn,
                            &format!(
                                "INSERT INTO recipe_totals (recipe_id, voltage, min_total, max_total) \
                                 VALUES ({}, {}, {}, {})",
                                rid,
                                sql_text(t.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
                                sql_num(t.get("min_total").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                                sql_num(t.get("max_total").and_then(|v| v.as_f64()).unwrap_or(0.0))
                            ),
                        )?;
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
                let value = c.get("config_value").and_then(|v| v.as_str()).unwrap_or("");
                execute(
                    conn,
                    &format!(
                        "IF EXISTS (SELECT 1 FROM configurations WHERE config_key = {k}) \
                         UPDATE configurations SET config_value = {v}, updated_at = {now} WHERE config_key = {k} \
                         ELSE INSERT INTO configurations (config_key, config_value, updated_at) \
                         VALUES ({k}, {v}, {now})",
                        k = sql_text(key),
                        v = sql_text(value),
                        now = now
                    ),
                )?;
            }
        }

        if let Some(templates) = config.get("label_templates").and_then(|v| v.as_array()) {
            for t in templates {
                let is_default = t.get("is_default").and_then(|v| v.as_bool()).unwrap_or(false);
                if is_default {
                    execute(conn, "UPDATE label_templates SET is_default = 0")?;
                }
                execute(
                    conn,
                    &format!(
                        "INSERT INTO label_templates \
                         (name, width_mm, height_mm, canvas_data, is_default, created_at, updated_at) \
                         VALUES ({}, {}, {}, {}, {}, {}, {})",
                        sql_text(t.get("name").and_then(|v| v.as_str()).unwrap_or("importada")),
                        t.get("width_mm").and_then(|v| v.as_i64()).unwrap_or(100),
                        t.get("height_mm").and_then(|v| v.as_i64()).unwrap_or(60),
                        sql_opt(t.get("canvas_data").and_then(|v| v.as_str())),
                        sql_bool(is_default),
                        now,
                        now
                    ),
                )?;
            }
        }

        if let Some(users) = config.get("users").and_then(|v| v.as_array()) {
            for u in users {
                let username = u.get("username").and_then(|v| v.as_str()).unwrap_or("");
                if username.is_empty() {
                    continue;
                }
                execute(
                    conn,
                    &format!(
                        "IF NOT EXISTS (SELECT 1 FROM users WHERE username = {u}) \
                         INSERT INTO users (username, password_hash, role, is_active, created_at) \
                         VALUES ({u}, {p}, {r}, {a}, {now})",
                        u = sql_text(username),
                        p = sql_text(u.get("password_hash").and_then(|v| v.as_str()).unwrap_or("")),
                        r = sql_text(u.get("role").and_then(|v| v.as_str()).unwrap_or("operator")),
                        a = sql_bool(u.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true)),
                        now = now
                    ),
                )?;
            }
        }

        if let Some(tags) = config.get("plc_tags").and_then(|v| v.as_array()) {
            for t in tags {
                let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() {
                    continue;
                }
                execute(
                    conn,
                    &format!(
                        "IF NOT EXISTS (SELECT 1 FROM plc_tags WHERE name = {n}) \
                         INSERT INTO plc_tags (name, db_number, [offset], data_type, bit, description, unit) \
                         VALUES ({n}, {d}, {o}, {dt}, {b}, {des}, {u})",
                        n = sql_text(name),
                        d = t.get("db_number").and_then(|v| v.as_i64()).unwrap_or(1),
                        o = t.get("offset").and_then(|v| v.as_i64()).unwrap_or(0),
                        dt = sql_text(t.get("data_type").and_then(|v| v.as_str()).unwrap_or("REAL")),
                        b = t.get("bit").and_then(|v| v.as_i64()).unwrap_or(0),
                        des = sql_opt(t.get("description").and_then(|v| v.as_str())),
                        u = sql_opt(t.get("unit").and_then(|v| v.as_str()))
                    ),
                )?;
            }
        }

        let summary = summary_of(&config).to_string();
        execute(
            conn,
            &format!(
                "INSERT INTO import_export_logs \
                 (operation_type, data_summary, mode, conflicts_found, records_imported, status, performed_at) \
                 VALUES ('import', {}, {}, {}, {}, 'success', {})",
                sql_text(&summary),
                sql_text(mode),
                conflicts.len() as i64,
                imported,
                now
            ),
        )?;

        Ok(imported)
    })?;

    Ok(json!({
        "status": "success",
        "imported": imported,
        "conflicts": conflicts,
    }))
}

fn summary_of(config: &Value) -> Value {
    json!({
        "recipes": config.get("recipes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
        "templates": config.get("label_templates").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
        "users": config.get("users").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
    })
}
