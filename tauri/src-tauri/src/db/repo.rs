//! Repositorios ODBC (SQL Express) para las entidades principales.
//!
//! La UI y los comandos Tauri llaman estas funciones. Toda la persistencia
//! pasa por `db::odbc_local`; `rusqlite` queda solo para la migración.

use crate::db::odbc_local::{
    execute, insert_id, query_json, query_one, sql_bool, sql_num, sql_opt, sql_text,
    with_connection, with_transaction,
};
use serde_json::{json, Map, Value};

// ---------------------------------------------------------------- configuración

pub fn config_get_all() -> Value {
    let rows = with_connection(|conn| {
        query_json(conn, "SELECT config_key, config_value FROM configurations")
    })
    .unwrap_or_default();
    let mut map = Map::new();
    for row in rows {
        if let (Some(key), Some(value)) = (
            row.get("config_key").and_then(|v| v.as_str()),
            row.get("config_value").and_then(|v| v.as_str()),
        ) {
            map.insert(key.to_string(), Value::String(value.to_string()));
        }
    }
    Value::Object(map)
}

pub fn config_set(values: &Value) -> Result<(), String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        if let Some(obj) = values.as_object() {
            for (key, value) in obj {
                let text = value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string());
                let k = sql_text(key);
                let v = sql_text(&text);
                execute(
                    conn,
                    &format!(
                        "IF EXISTS (SELECT 1 FROM configurations WHERE config_key = {k}) \
                         UPDATE configurations SET config_value = {v}, updated_at = {now} WHERE config_key = {k} \
                         ELSE INSERT INTO configurations (config_key, config_value, updated_at) VALUES ({k}, {v}, {now})"
                    ),
                )?;
            }
        }
        Ok(())
    })
}

// ---------------------------------------------------------------- recetas

pub fn recipes_list() -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            "SELECT id, name, model, [type], product, voltage_levels, is_active, \
             created_at, updated_at, modified_by FROM recipes ORDER BY id DESC",
        )
    })
    .unwrap_or_default();
    Value::Array(rows)
}

pub fn recipe_by_model(model: &str) -> Value {
    let result = with_connection(|conn| {
        let recipe = query_one(
            conn,
            &format!(
                "SELECT id, name, model, [type], product, voltage_levels, is_active, \
                 created_at, updated_at, modified_by FROM recipes WHERE model = {}",
                sql_text(model)
            ),
        )?;
        let mut recipe = match recipe {
            Some(r) => r,
            None => return Ok(Value::Null),
        };
        let rid = recipe.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

        let comps = query_json(
            conn,
            &format!(
                "SELECT name, voltage, enabled, min_value, max_value, unit \
                 FROM recipe_components WHERE recipe_id = {} ORDER BY id",
                rid
            ),
        )?
        .into_iter()
        .map(|row| {
            json!({
                "name": row.get("name").cloned().unwrap_or(Value::Null),
                "voltage": row.get("voltage").cloned().unwrap_or(Value::Null),
                "enabled": row.get("enabled").cloned().unwrap_or(json!(true)),
                "min": row.get("min_value").cloned().unwrap_or(json!(0.0)),
                "max": row.get("max_value").cloned().unwrap_or(json!(0.0)),
                "unit": row.get("unit").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();

        let totals = query_json(
            conn,
            &format!(
                "SELECT voltage, min_total, max_total FROM recipe_totals \
                 WHERE recipe_id = {} ORDER BY id",
                rid
            ),
        )?
        .into_iter()
        .map(|row| {
            json!({
                "voltage": row.get("voltage").cloned().unwrap_or(Value::Null),
                "min": row.get("min_total").cloned().unwrap_or(json!(0.0)),
                "max": row.get("max_total").cloned().unwrap_or(json!(0.0)),
            })
        })
        .collect::<Vec<_>>();

        if let Some(obj) = recipe.as_object_mut() {
            obj.insert("components".to_string(), Value::Array(comps));
            obj.insert("totals".to_string(), Value::Array(totals));
        }
        Ok(recipe)
    });
    result.unwrap_or(Value::Null)
}

pub fn recipe_create(
    name: &str,
    model: &str,
    type_: &str,
    product: Option<&str>,
    voltage_levels: &str,
    components: &Value,
    totals: &Value,
    modified_by: Option<&str>,
) -> Result<i64, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        let id = insert_id(
            conn,
            &format!(
                "INSERT INTO recipes (name, model, [type], product, voltage_levels, is_active, \
                 created_at, updated_at, modified_by) VALUES ({}, {}, {}, {}, {}, 1, {}, {}, {})",
                sql_text(name),
                sql_text(model),
                sql_text(type_),
                sql_opt(product),
                sql_text(voltage_levels),
                now,
                now,
                sql_opt(modified_by)
            ),
        )?;
        insert_components(conn, id, components)?;
        insert_totals(conn, id, totals)?;
        Ok(id)
    })
}

pub fn recipe_update(
    id: i64,
    name: &str,
    model: &str,
    type_: &str,
    product: Option<&str>,
    voltage_levels: &str,
    components: &Value,
    totals: &Value,
    modified_by: Option<&str>,
) -> Result<(), String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        let exists = query_one(
            conn,
            &format!("SELECT COUNT(*) AS n FROM recipes WHERE id = {}", id),
        )?
        .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
        .unwrap_or(0);
        if exists == 0 {
            return Err("Receta no encontrada".to_string());
        }
        execute(
            conn,
            &format!(
                "UPDATE recipes SET name = {}, model = {}, [type] = {}, product = {}, \
                 voltage_levels = {}, updated_at = {}, modified_by = {} WHERE id = {}",
                sql_text(name),
                sql_text(model),
                sql_text(type_),
                sql_opt(product),
                sql_text(voltage_levels),
                now,
                sql_opt(modified_by),
                id
            ),
        )?;
        execute(
            conn,
            &format!("DELETE FROM recipe_components WHERE recipe_id = {}", id),
        )?;
        execute(
            conn,
            &format!("DELETE FROM recipe_totals WHERE recipe_id = {}", id),
        )?;
        insert_components(conn, id, components)?;
        insert_totals(conn, id, totals)?;
        Ok(())
    })
}

pub fn recipe_delete(id: i64) -> Result<(), String> {
    with_transaction(|conn| {
        execute(
            conn,
            &format!("DELETE FROM recipe_components WHERE recipe_id = {}", id),
        )?;
        execute(
            conn,
            &format!("DELETE FROM recipe_totals WHERE recipe_id = {}", id),
        )?;
        execute(conn, &format!("DELETE FROM recipes WHERE id = {}", id))?;
        Ok(())
    })
}

pub fn recipe_duplicate(id: i64, new_model: &str) -> Result<i64, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        let original = query_one(
            conn,
            &format!(
                "SELECT name, [type], product, voltage_levels, modified_by \
                 FROM recipes WHERE id = {}",
                id
            ),
        )?
        .ok_or_else(|| "Receta no encontrada".to_string())?;

        let name = original
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Receta");
        let new_name = format!("{} (copia)", name);
        let type_ = original.get("type").and_then(|v| v.as_str()).unwrap_or("current_consumption");
        let product = original.get("product").and_then(|v| v.as_str());
        let voltage = original.get("voltage_levels").and_then(|v| v.as_str()).unwrap_or("");
        let modified = original.get("modified_by").and_then(|v| v.as_str());

        let new_id = insert_id(
            conn,
            &format!(
                "INSERT INTO recipes (name, model, [type], product, voltage_levels, is_active, \
                 created_at, updated_at, modified_by) VALUES ({}, {}, {}, {}, {}, 1, {}, {}, {})",
                sql_text(&new_name),
                sql_text(new_model),
                sql_text(type_),
                sql_opt(product),
                sql_text(voltage),
                now,
                now,
                sql_opt(modified)
            ),
        )?;

        let comps = query_json(
            conn,
            &format!(
                "SELECT name, voltage, enabled, min_value, max_value, unit \
                 FROM recipe_components WHERE recipe_id = {}",
                id
            ),
        )?;
        for c in comps {
            execute(
                conn,
                &format!(
                    "INSERT INTO recipe_components (recipe_id, name, voltage, enabled, min_value, max_value, unit) \
                     VALUES ({}, {}, {}, {}, {}, {}, {})",
                    new_id,
                    sql_text(c.get("name").and_then(|v| v.as_str()).unwrap_or("")),
                    sql_text(c.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
                    sql_bool(c.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)),
                    sql_num(c.get("min_value").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                    sql_num(c.get("max_value").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                    sql_text(c.get("unit").and_then(|v| v.as_str()).unwrap_or("A"))
                ),
            )?;
        }

        let totals = query_json(
            conn,
            &format!(
                "SELECT voltage, min_total, max_total FROM recipe_totals WHERE recipe_id = {}",
                id
            ),
        )?;
        for t in totals {
            execute(
                conn,
                &format!(
                    "INSERT INTO recipe_totals (recipe_id, voltage, min_total, max_total) \
                     VALUES ({}, {}, {}, {})",
                    new_id,
                    sql_text(t.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
                    sql_num(t.get("min_total").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                    sql_num(t.get("max_total").and_then(|v| v.as_f64()).unwrap_or(0.0))
                ),
            )?;
        }
        Ok(new_id)
    })
}

fn insert_components(conn: &mut odbc_api::Connection<'_>, recipe_id: i64, components: &Value) -> Result<(), String> {
    if let Some(arr) = components.as_array() {
        for c in arr {
            execute(
                conn,
                &format!(
                    "INSERT INTO recipe_components (recipe_id, name, voltage, enabled, min_value, max_value, unit) \
                     VALUES ({}, {}, {}, {}, {}, {}, {})",
                    recipe_id,
                    sql_text(c.get("name").and_then(|v| v.as_str()).unwrap_or("")),
                    sql_text(c.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
                    sql_bool(c.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true)),
                    sql_num(c.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                    sql_num(c.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                    sql_text(c.get("unit").and_then(|v| v.as_str()).unwrap_or("A"))
                ),
            )?;
        }
    }
    Ok(())
}

fn insert_totals(conn: &mut odbc_api::Connection<'_>, recipe_id: i64, totals: &Value) -> Result<(), String> {
    if let Some(arr) = totals.as_array() {
        for t in arr {
            execute(
                conn,
                &format!(
                    "INSERT INTO recipe_totals (recipe_id, voltage, min_total, max_total) \
                     VALUES ({}, {}, {}, {})",
                    recipe_id,
                    sql_text(t.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
                    sql_num(t.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                    sql_num(t.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0))
                ),
            )?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- operadores

pub fn operators_list() -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            "SELECT id, name, employee_id, is_active, created_at FROM operators ORDER BY id DESC",
        )
    })
    .unwrap_or_default();
    Value::Array(rows)
}

pub fn operator_add(name: &str, employee_id: &str) -> Result<i64, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_connection(|conn| {
        insert_id(
            conn,
            &format!(
                "INSERT INTO operators (name, employee_id, is_active, created_at) \
                 VALUES ({}, {}, 1, {})",
                sql_text(name),
                sql_text(employee_id),
                now
            ),
        )
    })
}

// ---------------------------------------------------------------- usuarios

pub fn users_list() -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            "SELECT id, username, role, is_active, created_at FROM users ORDER BY id DESC",
        )
    })
    .unwrap_or_default();
    Value::Array(rows)
}

pub fn user_add(username: &str, password: &str, role: &str) -> Result<i64, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_connection(|conn| {
        insert_id(
            conn,
            &format!(
                "INSERT INTO users (username, password_hash, role, is_active, created_at) \
                 VALUES ({}, {}, {}, 1, {})",
                sql_text(&username.trim().to_uppercase()),
                sql_text(password),
                sql_text(role),
                now
            ),
        )
    })
}

pub fn user_delete(id: i64) -> Result<(), String> {
    with_transaction(|conn| {
        let current = query_one(
            conn,
            &format!("SELECT role, is_active FROM users WHERE id = {}", id),
        )?
        .ok_or_else(|| "Usuario no encontrado".to_string())?;
        let role = current.get("role").and_then(|v| v.as_str()).unwrap_or("");
        let active = current.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
        if role == "admin" && active {
            let admins = query_one(
                conn,
                "SELECT COUNT(*) AS n FROM users WHERE role = 'admin' AND is_active = 1",
            )?
            .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
            .unwrap_or(0);
            if admins <= 1 {
                return Err("No se puede borrar el último admin activo".to_string());
            }
        }
        execute(conn, &format!("DELETE FROM users WHERE id = {}", id))?;
        Ok(())
    })
}

pub fn auth_login(username: &str) -> Result<Value, String> {
    let uname = username.trim().to_uppercase();
    if uname.is_empty() {
        return Err("Gafete/usuario requerido".to_string());
    }
    with_connection(|conn| {
        let user = query_one(
            conn,
            &format!(
                "SELECT username, role, is_active FROM users WHERE username = {}",
                sql_text(&uname)
            ),
        )?
        .ok_or_else(|| format!("Gafete no encontrado: {}", uname))?;
        if !user.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Err(format!("Usuario inactivo: {}", uname));
        }
        Ok(json!({
            "username": user.get("username").cloned().unwrap_or(Value::Null),
            "role": user.get("role").cloned().unwrap_or(Value::Null),
        }))
    })
}

pub fn user_get(uid: i64) -> Result<Value, String> {
    with_connection(|conn| {
        let user = query_one(
            conn,
            &format!(
                "SELECT id, username, role, is_active, created_at FROM users WHERE id = {}",
                uid
            ),
        )?
        .ok_or_else(|| "Usuario no encontrado".to_string())?;
        let role = user.get("role").and_then(|v| v.as_str()).unwrap_or("operator");
        let permissions = if role == "admin" {
            vec![
                "Todo lo de operator",
                "Configurar recetas (altas/bajas/cambios)",
                "Gestionar usuarios y permisos",
                "PLC/Hipot/BD, etiquetas, logs, import/export",
                "Ajustar espera de resultado e impresión",
            ]
        } else {
            vec![
                "Ejecutar pruebas (Modelo/Serie/Batch)",
                "Ver resultado BUENO/MALO",
                "Usar modo manual (sin password)",
                "Reimprimir/Log en principal",
            ]
        };
        let mut out = user;
        if let Some(obj) = out.as_object_mut() {
            obj.insert("permissions".to_string(), json!(permissions));
        }
        Ok(out)
    })
}

pub fn user_update(
    uid: i64,
    username: Option<&str>,
    role: Option<&str>,
    is_active: Option<bool>,
) -> Result<(), String> {
    with_transaction(|conn| {
        let current = query_one(
            conn,
            &format!(
                "SELECT username, role, is_active FROM users WHERE id = {}",
                uid
            ),
        )?
        .ok_or_else(|| "Usuario no encontrado".to_string())?;
        let cur_username = current
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let cur_role = current
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("operator")
            .to_string();
        let cur_active = current
            .get("is_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let new_username = username
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| cur_username.clone());
        let new_role = role
            .filter(|r| *r == "admin" || *r == "operator")
            .unwrap_or(&cur_role)
            .to_string();
        let new_active = is_active.unwrap_or(cur_active);

        if new_username != cur_username {
            let exists = query_one(
                conn,
                &format!(
                    "SELECT COUNT(*) AS n FROM users WHERE username = {} AND id <> {}",
                    sql_text(&new_username),
                    uid
                ),
            )?
            .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
            .unwrap_or(0);
            if exists > 0 {
                return Err("Usuario ya existe".to_string());
            }
        }

        let admins = query_one(
            conn,
            "SELECT COUNT(*) AS n FROM users WHERE role = 'admin' AND is_active = 1",
        )?
        .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
        .unwrap_or(0);
        if cur_role == "admin"
            && cur_active
            && (new_role != "admin" || !new_active)
            && admins <= 1
        {
            return Err("Debe quedar al menos un admin activo".to_string());
        }

        execute(
            conn,
            &format!(
                "UPDATE users SET username = {}, role = {}, is_active = {} WHERE id = {}",
                sql_text(&new_username),
                sql_text(&new_role),
                sql_bool(new_active),
                uid
            ),
        )?;
        Ok(())
    })
}

// ---------------------------------------------------------------- ejecuciones

pub fn execution_start(
    recipe_id: i64,
    operator_id: Option<i64>,
    batch: Option<&str>,
    serial_number: Option<&str>,
) -> Result<i64, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        let mut batch_id: Option<i64> = None;
        if let Some(batch) = batch {
            let bn = batch.trim().to_uppercase();
            if !bn.is_empty() {
                let existing = query_one(
                    conn,
                    &format!("SELECT id FROM batches WHERE batch_number = {}", sql_text(&bn)),
                )?;
                batch_id = if let Some(row) = existing {
                    row.get("id").and_then(|v| v.as_i64())
                } else {
                    Some(insert_id(
                        conn,
                        &format!(
                            "INSERT INTO batches (batch_number, operator_id, scanned_at) \
                             VALUES ({}, {}, {})",
                            sql_text(&bn),
                            operator_id.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
                            now
                        ),
                    )?)
                };
            }
        }

        let serial = serial_number
            .map(|s| sql_text(&s.trim().to_uppercase()))
            .unwrap_or_else(|| "NULL".to_string());
        let batch_sql = batch_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());
        let operator_sql = operator_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string());

        insert_id(
            conn,
            &format!(
                "INSERT INTO executions \
                 (recipe_id, operator_id, batch_id, serial_number, plc_start_time, created_at) \
                 VALUES ({}, {}, {}, {}, {}, {})",
                recipe_id, operator_sql, batch_sql, serial, now, now
            ),
        )
    })
}

pub fn save_plc_result(execution_id: i64, passed: bool, readings: &Value) -> Result<(), String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        let recipe_id = query_one(
            conn,
            &format!(
                "SELECT recipe_id FROM executions WHERE id = {}",
                execution_id
            ),
        )?
        .and_then(|row| row.get("recipe_id").and_then(|v| v.as_i64()))
        .unwrap_or(0);

        execute(
            conn,
            &format!(
                "UPDATE executions SET plc_end_time = {}, plc_pass_fail = {}, plc_duration = 0 \
                 WHERE id = {}",
                now,
                sql_bool(passed),
                execution_id
            ),
        )?;

        if let Some(arr) = readings.as_array() {
            for r in arr {
                execute(
                    conn,
                    &format!(
                        "INSERT INTO plc_readings \
                         (execution_id, recipe_id, component_name, voltage, tag_name, value, \
                          min_limit, max_limit, is_pass, timestamp) \
                         VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                        execution_id,
                        recipe_id,
                        sql_text(r.get("component").and_then(|v| v.as_str()).unwrap_or("")),
                        sql_text(r.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
                        sql_text(r.get("tag").and_then(|v| v.as_str()).unwrap_or("")),
                        sql_num(r.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                        sql_num(r.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                        sql_num(r.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                        sql_bool(r.get("pass").and_then(|v| v.as_bool()).unwrap_or(false)),
                        now
                    ),
                )?;
            }
        }
        Ok(())
    })
}

pub fn save_hipot_result(execution_id: i64, body: &Value) -> Result<String, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        let execution = query_one(
            conn,
            &format!(
                "SELECT recipe_id, plc_pass_fail FROM executions WHERE id = {}",
                execution_id
            ),
        )?
        .ok_or_else(|| "Ejecución no encontrada".to_string())?;
        let recipe_id = execution
            .get("recipe_id")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let plc_pass = execution
            .get("plc_pass_fail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let hipot_pass = body.get("pass").and_then(|v| v.as_bool()).unwrap_or(false);
        let duration = body.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let final_status = if plc_pass && hipot_pass { "GOOD" } else { "BAD" };

        execute(
            conn,
            &format!(
                "UPDATE executions SET hipot_start_time = {}, hipot_end_time = {}, \
                 hipot_pass_fail = {}, hipot_duration = {}, final_status = {}, total_duration = {} \
                 WHERE id = {}",
                now,
                now,
                sql_bool(hipot_pass),
                sql_num(duration),
                sql_text(final_status),
                sql_num(duration),
                execution_id
            ),
        )?;

        execute(
            conn,
            &format!(
                "INSERT INTO hipot_results \
                 (execution_id, recipe_id, voltage, [current], resistance, insulation_resistance, \
                  test_duration, is_pass, timestamp) \
                 VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {})",
                execution_id,
                recipe_id,
                sql_opt(body.get("voltage").and_then(|v| v.as_f64()).map(|v| v.to_string()).as_deref()),
                sql_opt(body.get("current").and_then(|v| v.as_f64()).map(|v| v.to_string()).as_deref()),
                sql_opt(body.get("resistance").and_then(|v| v.as_f64()).map(|v| v.to_string()).as_deref()),
                sql_opt(body.get("insulation").and_then(|v| v.as_f64()).map(|v| v.to_string()).as_deref()),
                sql_num(duration),
                sql_bool(hipot_pass),
                now
            ),
        )?;
        Ok(final_status.to_string())
    })
}

pub fn executions_list(limit: i64) -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            &format!(
                "SELECT TOP ({}) id, recipe_id, serial_number, plc_pass_fail, hipot_pass_fail, \
                 final_status, created_at FROM executions ORDER BY id DESC",
                limit.clamp(1, 5000)
            ),
        )
    })
    .unwrap_or_default();
    let mapped = rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get("id").cloned().unwrap_or(Value::Null),
                "recipe_id": row.get("recipe_id").cloned().unwrap_or(Value::Null),
                "serial": row.get("serial_number").cloned().unwrap_or(Value::Null),
                "plc": row.get("plc_pass_fail").cloned().unwrap_or(Value::Null),
                "hipot": row.get("hipot_pass_fail").cloned().unwrap_or(Value::Null),
                "final": row.get("final_status").cloned().unwrap_or(Value::Null),
                "created": row.get("created_at").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();
    Value::Array(mapped)
}

// ---------------------------------------------------------------- logs

pub fn log_add(
    level: &str,
    category: &str,
    action: &str,
    detail: Option<&str>,
    execution_id: Option<i64>,
    value_sent: Option<&str>,
    value_received: Option<&str>,
    duration_ms: Option<i64>,
    success: bool,
    error_message: Option<&str>,
) -> Result<i64, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_connection(|conn| {
        insert_id(
            conn,
            &format!(
                "INSERT INTO app_logs \
                 (timestamp, level, category, action, detail, execution_id, value_sent, \
                  value_received, duration_ms, success, error_message) \
                 VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                now,
                sql_text(level),
                sql_text(category),
                sql_text(action),
                sql_opt(detail),
                execution_id.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
                sql_opt(value_sent),
                sql_opt(value_received),
                duration_ms.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
                sql_bool(success),
                sql_opt(error_message)
            ),
        )
    })
}

pub fn logs_list(category: Option<&str>, level: Option<&str>, q: Option<&str>, limit: i64) -> Value {
    let mut where_sql = String::from("WHERE 1 = 1");
    if let Some(c) = category.filter(|s| !s.is_empty()) {
        where_sql.push_str(&format!(" AND category = {}", sql_text(c)));
    }
    if let Some(l) = level.filter(|s| !s.is_empty()) {
        where_sql.push_str(&format!(" AND level = {}", sql_text(l)));
    }
    if let Some(qq) = q.filter(|s| !s.is_empty()) {
        where_sql.push_str(&format!(" AND detail LIKE {}", sql_text(&format!("%{}%", qq))));
    }
    let rows = with_connection(|conn| {
        query_json(
            conn,
            &format!(
                "SELECT TOP ({}) id, timestamp AS ts, level, category AS cat, action, detail, \
                 execution_id, value_sent, value_received, duration_ms, CAST(success AS INT) AS success, \
                 error_message FROM app_logs {} ORDER BY id DESC",
                limit.clamp(1, 1000),
                where_sql
            ),
        )
    })
    .unwrap_or_default();
    Value::Array(rows)
}

/// Rota logs: elimina los más antiguos por fecha y conserva solo los últimos N.
pub fn logs_prune(days: i64, keep: i64) -> Result<Value, String> {
    with_connection(|conn| {
        let cutoff = (chrono::Local::now() - chrono::Duration::days(days.max(1)))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        execute(
            conn,
            &format!("DELETE FROM app_logs WHERE timestamp < {}", sql_text(&cutoff)),
        )?;
        execute(
            conn,
            &format!(
                "DELETE FROM app_logs WHERE id NOT IN \
                 (SELECT TOP ({}) id FROM app_logs ORDER BY id DESC)",
                keep.clamp(100, 1_000_000)
            ),
        )?;
        let remaining = query_one(conn, "SELECT COUNT(*) AS n FROM app_logs")?
            .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        Ok(json!({ "status": "ok", "remaining": remaining }))
    })
}

// ---------------------------------------------------------------- plantillas

pub fn templates_list() -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            "SELECT id, name, width_mm AS w, height_mm AS h, is_default \
             FROM label_templates ORDER BY id",
        )
    })
    .unwrap_or_default();
    Value::Array(rows)
}

pub fn template_get(tid: i64) -> Result<Value, String> {
    with_connection(|conn| {
        query_one(
            conn,
            &format!(
                "SELECT id, name, width_mm, height_mm, canvas_data, is_default \
                 FROM label_templates WHERE id = {}",
                tid
            ),
        )?
        .ok_or_else(|| "Plantilla no encontrada".to_string())
    })
}

pub fn template_create(
    name: &str,
    width_mm: i64,
    height_mm: i64,
    canvas_data: &str,
    is_default: bool,
) -> Result<i64, String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        if is_default {
            execute(conn, "UPDATE label_templates SET is_default = 0")?;
        }
        insert_id(
            conn,
            &format!(
                "INSERT INTO label_templates \
                 (name, width_mm, height_mm, canvas_data, is_default, created_at, updated_at) \
                 VALUES ({}, {}, {}, {}, {}, {}, {})",
                sql_text(name),
                width_mm,
                height_mm,
                sql_text(canvas_data),
                sql_bool(is_default),
                now,
                now
            ),
        )
    })
}

pub fn template_update(
    tid: i64,
    name: &str,
    width_mm: i64,
    height_mm: i64,
    canvas_data: &str,
    is_default: bool,
) -> Result<(), String> {
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_transaction(|conn| {
        if is_default {
            execute(
                conn,
                &format!(
                    "UPDATE label_templates SET is_default = 0 WHERE id <> {}",
                    tid
                ),
            )?;
        }
        execute(
            conn,
            &format!(
                "UPDATE label_templates SET name = {}, width_mm = {}, height_mm = {}, \
                 canvas_data = {}, is_default = {}, updated_at = {} WHERE id = {}",
                sql_text(name),
                width_mm,
                height_mm,
                sql_text(canvas_data),
                sql_bool(is_default),
                now,
                tid
            ),
        )?;
        Ok(())
    })
}

pub fn template_delete(tid: i64) -> Result<(), String> {
    with_connection(|conn| {
        execute(
            conn,
            &format!("DELETE FROM label_templates WHERE id = {}", tid),
        )
    })
}

// ---------------------------------------------------------------- tags PLC

pub fn plc_tags_list() -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            "SELECT id, name, db_number, [offset], data_type, bit, description, unit \
             FROM plc_tags ORDER BY id",
        )
    })
    .unwrap_or_default();
    let mapped = rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get("id").cloned().unwrap_or(Value::Null),
                "name": row.get("name").cloned().unwrap_or(Value::Null),
                "db": row.get("db_number").cloned().unwrap_or(Value::Null),
                "offset": row.get("offset").cloned().unwrap_or(Value::Null),
                "type": row.get("data_type").cloned().unwrap_or(Value::Null),
                "bit": row.get("bit").cloned().unwrap_or(Value::Null),
                "description": row.get("description").cloned().unwrap_or(Value::Null),
                "unit": row.get("unit").cloned().unwrap_or(Value::Null),
                "value": Value::Null,
            })
        })
        .collect::<Vec<_>>();
    Value::Array(mapped)
}

pub fn plc_tags_add(
    name: &str,
    db_number: i64,
    offset: i64,
    data_type: &str,
    bit: i64,
) -> Result<i64, String> {
    with_connection(|conn| {
        insert_id(
            conn,
            &format!(
                "INSERT INTO plc_tags (name, db_number, [offset], data_type, bit) \
                 VALUES ({}, {}, {}, {}, {})",
                sql_text(&name.trim().to_uppercase()),
                db_number,
                offset,
                sql_text(&data_type.trim().to_uppercase()),
                bit
            ),
        )
    })
}

pub fn plc_tags_delete(id: i64) -> Result<(), String> {
    with_connection(|conn| execute(conn, &format!("DELETE FROM plc_tags WHERE id = {}", id)))
}

pub fn plc_tag_get(id: i64) -> Result<Value, String> {
    with_connection(|conn| {
        query_one(
            conn,
            &format!(
                "SELECT id, name, db_number, [offset], data_type, bit FROM plc_tags WHERE id = {}",
                id
            ),
        )?
        .ok_or_else(|| "Tag no encontrado".to_string())
    })
}

// ---------------------------------------------------------------- barcode

pub fn barcode_scan(
    code: &str,
    barcode_type: &str,
    scan_order: i64,
    execution_id: Option<i64>,
) -> Result<Value, String> {
    let normalized = code.trim().to_uppercase();
    if normalized.is_empty() {
        return Err("Código vacío".to_string());
    }
    let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    with_connection(|conn| {
        execute(
            conn,
            &format!(
                "INSERT INTO barcode_logs (code, barcode_type, scan_order, execution_id, scanned_at) \
                 VALUES ({}, {}, {}, {}, {})",
                sql_text(&normalized),
                sql_text(barcode_type),
                scan_order,
                execution_id.map(|v| v.to_string()).unwrap_or_else(|| "NULL".to_string()),
                now
            ),
        )
    })?;
    let _ = log_add(
        "INFO",
        "BARCODE",
        &format!("scan_{}", barcode_type),
        Some(&format!("Crudo: '{}' -> Normalizado: '{}'", code, normalized)),
        execution_id,
        None,
        Some(&normalized),
        None,
        true,
        None,
    );
    Ok(json!({
        "status": "scanned",
        "raw": code,
        "code": normalized,
        "barcode_type": barcode_type,
        "scan_order": scan_order,
    }))
}

// ---------------------------------------------------------------- historial

pub fn history_list(
    model: Option<&str>,
    serial: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
    limit: i64,
) -> Value {
    let mut where_sql = String::from("WHERE 1 = 1");
    if let Some(s) = serial.filter(|s| !s.trim().is_empty()) {
        where_sql.push_str(&format!(
            " AND e.serial_number LIKE {}",
            sql_text(&format!("%{}%", s.trim().to_uppercase()))
        ));
    }
    if let Some(m) = model.filter(|s| !s.trim().is_empty()) {
        where_sql.push_str(&format!(
            " AND r.model LIKE {}",
            sql_text(&format!("%{}%", m.trim().to_uppercase()))
        ));
    }
    if let Some(s) = start.filter(|s| !s.trim().is_empty()) {
        where_sql.push_str(&format!(" AND e.created_at >= {}", sql_text(s)));
    }
    if let Some(e) = end.filter(|s| !s.trim().is_empty()) {
        if let Ok(day) = chrono::NaiveDate::parse_from_str(e, "%Y-%m-%d") {
            if let Some(next) = day.succ_opt() {
                where_sql.push_str(&format!(
                    " AND e.created_at < {}",
                    sql_text(&next.format("%Y-%m-%d").to_string())
                ));
            }
        }
    }
    let rows = with_connection(|conn| {
        query_json(
            conn,
            &format!(
                "SELECT TOP ({}) e.id, e.recipe_id, COALESCE(r.model, '-') AS recipe_model, \
                 COALESCE(e.serial_number, '-') AS serial, COALESCE(b.batch_number, '-') AS batch, \
                 e.plc_pass_fail AS plc, e.hipot_pass_fail AS hipot, \
                 COALESCE(e.final_status, '-') AS final, e.created_at AS created, \
                 e.total_duration \
                 FROM executions e \
                 LEFT JOIN recipes r ON r.id = e.recipe_id \
                 LEFT JOIN batches b ON b.id = e.batch_id \
                 {} ORDER BY e.id DESC",
                limit.clamp(1, 5000),
                where_sql
            ),
        )
    })
    .unwrap_or_default();
    let good = rows
        .iter()
        .filter(|r| r.get("final").and_then(|v| v.as_str()) == Some("GOOD"))
        .count();
    let bad = rows
        .iter()
        .filter(|r| r.get("final").and_then(|v| v.as_str()) == Some("BAD"))
        .count();
    json!({ "rows": rows, "total": rows.len(), "good": good, "bad": bad })
}

// ---------------------------------------------------------------- sync / import-export

pub fn sync_log_list(limit: i64) -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            &format!(
                "SELECT TOP ({}) id, direction AS dir, table_name AS [table], \
                 records_synced AS n, status, error_message AS err \
                 FROM sync_log ORDER BY id DESC",
                limit.clamp(1, 500)
            ),
        )
    })
    .unwrap_or_default();
    Value::Array(rows)
}

pub fn import_export_log_list(limit: i64) -> Value {
    let rows = with_connection(|conn| {
        query_json(
            conn,
            &format!(
                "SELECT TOP ({}) id, operation_type AS op, mode, data_summary AS summary, \
                 status, performed_at AS at FROM import_export_logs ORDER BY id DESC",
                limit.clamp(1, 500)
            ),
        )
    })
    .unwrap_or_default();
    Value::Array(rows)
}

pub fn label_data(eid: i64) -> Value {
    let row = with_connection(|conn| {
        query_one(
            conn,
            &format!(
                "SELECT e.serial_number, e.final_status, COALESCE(r.model, '') AS model, \
                 COALESCE(b.batch_number, '') AS batch, e.created_at \
                 FROM executions e \
                 LEFT JOIN recipes r ON r.id = e.recipe_id \
                 LEFT JOIN batches b ON b.id = e.batch_id \
                 WHERE e.id = {}",
                eid
            ),
        )
    })
    .unwrap_or(None)
    .unwrap_or(Value::Null);
    row
}

// ---------------------------------------------------------------- sync local -> remoto

const SYNC_TABLES: &[(&str, bool)] = &[
    ("recipes", false),
    ("recipe_components", false),
    ("recipe_totals", false),
    ("operators", false),
    ("batches", false),
    ("users", false),
    ("label_templates", false),
    ("executions", true),
    ("plc_readings", false),
    ("hipot_results", false),
    ("barcode_logs", false),
    ("label_prints", false),
    ("plc_tags", false),
    ("configurations", false),
];

pub fn sync_pending(remote_conn_str: &str) -> Result<Value, String> {
    use odbc_api::{ConnectionOptions, Environment};

    let local_cfg = crate::db::odbc_local::config();
    if local_cfg.use_sqlite {
        return Err("La BD local está en SQLite; configura SQL Express para sincronizar.".to_string());
    }

    let env = Environment::new().map_err(|e| format!("ODBC: {}", e))?;
    let mut local = env
        .connect_with_connection_string(&local_cfg.connection_string(), ConnectionOptions::default())
        .map_err(|e| format!("Conexión local SQL Express: {}", e))?;
    let mut remote = env
        .connect_with_connection_string(remote_conn_str, ConnectionOptions::default())
        .map_err(|e| format!("Conexión remota: {}", e))?;

    let mut report = Map::new();
    for (table, pending_only) in SYNC_TABLES {
        let started = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        match sync_table(&mut local, &mut remote, table, *pending_only) {
            Ok(count) => {
                report.insert((*table).to_string(), json!(count));
                let _ = log_sync(&mut local, table, count, "success", None, &started);
            }
            Err(e) => {
                report.insert((*table).to_string(), json!({ "error": e }));
                let _ = log_sync(&mut local, table, 0, "error", Some(&e), &started);
            }
        }
    }

    Ok(json!({ "status": "ok", "tables": report }))
}

fn sync_table(
    local: &mut odbc_api::Connection<'_>,
    remote: &mut odbc_api::Connection<'_>,
    table: &str,
    pending_only: bool,
) -> Result<i64, String> {
    let filter = if pending_only {
        " WHERE synced_to_remote = 0 OR synced_to_remote IS NULL"
    } else {
        ""
    };
    let rows = query_json(local, &format!("SELECT * FROM [{}]{}", table, filter))?;
    if rows.is_empty() {
        return Ok(0);
    }

    execute(remote, &format!("SET IDENTITY_INSERT [{}] ON", table))?;
    let mut count = 0i64;
    for row in &rows {
        let id = row.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let columns = row
            .as_object()
            .map(|obj| {
                obj.keys()
                    .map(|k| format!("[{}]", k))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let values = row
            .as_object()
            .map(|obj| {
                obj.values()
                    .map(json_to_sql)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        if columns.is_empty() {
            continue;
        }
        execute(
            remote,
            &format!(
                "IF NOT EXISTS (SELECT 1 FROM [{}] WHERE id = {}) \
                 INSERT INTO [{}] ({}) VALUES ({})",
                table, id, table, columns, values
            ),
        )?;
        count += 1;
    }
    execute(remote, &format!("SET IDENTITY_INSERT [{}] OFF", table))?;

    if pending_only {
        let now = sql_text(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        execute(
            local,
            &format!(
                "UPDATE [{}] SET synced_to_remote = 1, synced_at = {} \
                 WHERE synced_to_remote = 0 OR synced_to_remote IS NULL",
                table, now
            ),
        )?;
    }
    Ok(count)
}

fn log_sync(
    local: &mut odbc_api::Connection<'_>,
    table: &str,
    count: i64,
    status: &str,
    error: Option<&str>,
    started: &str,
) -> Result<(), String> {
    let completed = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    execute(
        local,
        &format!(
            "INSERT INTO sync_log \
             (direction, table_name, records_synced, status, error_message, started_at, completed_at) \
             VALUES ('local_to_remote', {}, {}, {}, {}, {}, {})",
            sql_text(table),
            count,
            sql_text(status),
            sql_opt(error),
            sql_text(started),
            sql_text(&completed)
        ),
    )
}

fn json_to_sql(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => sql_bool(*b).to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => sql_text(s),
        other => sql_text(&other.to_string()),
    }
}
