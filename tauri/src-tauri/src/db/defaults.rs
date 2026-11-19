//! Datos de fábrica: recetas RL y tags PLC DB1.
//!
//! Las recetas se cargan una vez en SQL Express (marcador en `configurations`)
//! y también se pueden restaurar manualmente desde Configuración → Recetas.

use serde_json::{json, Value};

use crate::db::odbc_local::{
    execute, insert_id, query_one, sql_num, sql_text, with_transaction,
};

struct RecipeSeed {
    model: &'static str,
    name: &'static str,
    product: &'static str,
    voltage: &'static str,
    components: &'static [(&'static str, f64, f64)],
    total: (f64, f64),
}

/// Cada fila de la hoja CONSUMOS RL es una receta independiente.
/// El límite de corriente del Hipot queda NULL: debe capturarse en la app.
const DEFAULT_RECIPES: &[RecipeSeed] = &[
    RecipeSeed {
        model: "RL-120",
        name: "CONSUMOS RL 120VAC",
        product: "RL",
        voltage: "120",
        components: &[
            ("FAN", 0.3, 1.5),
            ("DOOR", 0.43, 3.8),
            ("FRAME", 0.39, 1.97),
            ("DRAIN", 1.67, 2.57),
            ("LIGHT", 0.18, 0.84),
        ],
        total: (2.97, 10.68),
    },
    RecipeSeed {
        model: "RL-208",
        name: "CONSUMOS RL 208VAC",
        product: "RL",
        voltage: "208",
        components: &[("DEFROST", 2.88, 16.82)],
        total: (2.88, 16.82),
    },
    RecipeSeed {
        model: "RL-220",
        name: "CONSUMOS RL 220VAC",
        product: "RL",
        voltage: "220",
        components: &[
            ("DRAIN", 0.34, 1.53),
            ("DEFROST", 7.11, 17.79),
            ("LIGHT", 0.17, 0.46),
        ],
        total: (7.62, 19.78),
    },
    RecipeSeed {
        model: "RL-240",
        name: "CONSUMOS RL 240VAC",
        product: "RL",
        voltage: "240",
        components: &[
            ("FAN", 0.3, 0.75),
            ("DOOR", 0.8, 1.9),
            ("FRAME", 0.45, 1.12),
            ("DRAIN", 0.41, 1.67),
            ("DEFROST", 7.76, 19.42),
        ],
        total: (9.72, 24.86),
    },
];

struct PlcTagSeed {
    name: &'static str,
    offset: i64,
    data_type: &'static str,
    bit: i64,
    description: &'static str,
    unit: &'static str,
}

/// Mapa DB1 documentado en docs/06-comunicaciones.md.
const DEFAULT_PLC_TAGS: &[PlcTagSeed] = &[
    PlcTagSeed { name: "Recipe_Voltage", offset: 0, data_type: "REAL", bit: 0, description: "Voltaje de receta", unit: "V" },
    PlcTagSeed { name: "Component_FAN_MIN", offset: 4, data_type: "REAL", bit: 0, description: "FAN mínimo", unit: "A" },
    PlcTagSeed { name: "Component_FAN_MAX", offset: 8, data_type: "REAL", bit: 0, description: "FAN máximo", unit: "A" },
    PlcTagSeed { name: "Component_FAN_ENABLE", offset: 12, data_type: "BOOL", bit: 0, description: "FAN habilitado", unit: "" },
    PlcTagSeed { name: "Component_DOOR_MIN", offset: 16, data_type: "REAL", bit: 0, description: "DOOR mínimo", unit: "A" },
    PlcTagSeed { name: "Component_DOOR_MAX", offset: 20, data_type: "REAL", bit: 0, description: "DOOR máximo", unit: "A" },
    PlcTagSeed { name: "Component_DOOR_ENABLE", offset: 24, data_type: "BOOL", bit: 0, description: "DOOR habilitado", unit: "" },
    PlcTagSeed { name: "Component_FRAME_MIN", offset: 28, data_type: "REAL", bit: 0, description: "FRAME mínimo", unit: "A" },
    PlcTagSeed { name: "Component_FRAME_MAX", offset: 32, data_type: "REAL", bit: 0, description: "FRAME máximo", unit: "A" },
    PlcTagSeed { name: "Component_FRAME_ENABLE", offset: 36, data_type: "BOOL", bit: 0, description: "FRAME habilitado", unit: "" },
    PlcTagSeed { name: "Component_DRAIN_MIN", offset: 40, data_type: "REAL", bit: 0, description: "DRAIN mínimo", unit: "A" },
    PlcTagSeed { name: "Component_DRAIN_MAX", offset: 44, data_type: "REAL", bit: 0, description: "DRAIN máximo", unit: "A" },
    PlcTagSeed { name: "Component_DRAIN_ENABLE", offset: 48, data_type: "BOOL", bit: 0, description: "DRAIN habilitado", unit: "" },
    PlcTagSeed { name: "Component_DEFROST_MIN", offset: 52, data_type: "REAL", bit: 0, description: "DEFROST mínimo", unit: "A" },
    PlcTagSeed { name: "Component_DEFROST_MAX", offset: 56, data_type: "REAL", bit: 0, description: "DEFROST máximo", unit: "A" },
    PlcTagSeed { name: "Component_DEFROST_ENABLE", offset: 60, data_type: "BOOL", bit: 0, description: "DEFROST habilitado", unit: "" },
    PlcTagSeed { name: "Component_LIGHT_MIN", offset: 64, data_type: "REAL", bit: 0, description: "LIGHT mínimo", unit: "A" },
    PlcTagSeed { name: "Component_LIGHT_MAX", offset: 68, data_type: "REAL", bit: 0, description: "LIGHT máximo", unit: "A" },
    PlcTagSeed { name: "Component_LIGHT_ENABLE", offset: 72, data_type: "BOOL", bit: 0, description: "LIGHT habilitado", unit: "" },
    PlcTagSeed { name: "Test_Start", offset: 100, data_type: "BOOL", bit: 0, description: "Inicio de prueba", unit: "" },
    PlcTagSeed { name: "Test_Stop", offset: 101, data_type: "BOOL", bit: 0, description: "Paro de prueba", unit: "" },
    PlcTagSeed { name: "Test_Complete", offset: 102, data_type: "BOOL", bit: 0, description: "Prueba completa", unit: "" },
    PlcTagSeed { name: "Test_Pass", offset: 103, data_type: "BOOL", bit: 0, description: "Resultado PASS", unit: "" },
    PlcTagSeed { name: "Test_Running", offset: 104, data_type: "BOOL", bit: 0, description: "Prueba en proceso", unit: "" },
    PlcTagSeed { name: "Test_Mode", offset: 110, data_type: "WORD", bit: 0, description: "Modo de prueba", unit: "" },
    PlcTagSeed { name: "Result_FAN", offset: 200, data_type: "REAL", bit: 0, description: "Resultado FAN", unit: "A" },
    PlcTagSeed { name: "Result_DOOR", offset: 204, data_type: "REAL", bit: 0, description: "Resultado DOOR", unit: "A" },
    PlcTagSeed { name: "Result_FRAME", offset: 208, data_type: "REAL", bit: 0, description: "Resultado FRAME", unit: "A" },
    PlcTagSeed { name: "Result_DRAIN", offset: 212, data_type: "REAL", bit: 0, description: "Resultado DRAIN", unit: "A" },
    PlcTagSeed { name: "Result_DEFROST", offset: 216, data_type: "REAL", bit: 0, description: "Resultado DEFROST", unit: "A" },
    PlcTagSeed { name: "Result_LIGHT", offset: 220, data_type: "REAL", bit: 0, description: "Resultado LIGHT", unit: "A" },
    PlcTagSeed { name: "Result_TOTAL", offset: 224, data_type: "REAL", bit: 0, description: "Resultado TOTAL", unit: "A" },
];

fn now_text() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn marker_exists(conn: &mut odbc_api::Connection<'_>, key: &str) -> Result<bool, String> {
    let count = query_one(
        conn,
        &format!(
            "SELECT COUNT(*) AS n FROM configurations WHERE config_key = {}",
            sql_text(key)
        ),
    )?
    .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
    .unwrap_or(0);
    Ok(count > 0)
}

fn set_marker(conn: &mut odbc_api::Connection<'_>, key: &str, now: &str) -> Result<(), String> {
    execute(
        conn,
        &format!(
            "IF NOT EXISTS (SELECT 1 FROM configurations WHERE config_key = {k}) \
             INSERT INTO configurations (config_key, config_value, updated_at) \
             VALUES ({k}, '1', {now})",
            k = sql_text(key),
            now = sql_text(now)
        ),
    )
}

fn recipe_id(conn: &mut odbc_api::Connection<'_>, model: &str) -> Result<Option<i64>, String> {
    Ok(query_one(
        conn,
        &format!("SELECT id FROM recipes WHERE model = {}", sql_text(model)),
    )?
    .and_then(|row| row.get("id").and_then(|v| v.as_i64())))
}

fn insert_recipe(
    conn: &mut odbc_api::Connection<'_>,
    seed: &RecipeSeed,
    now: &str,
) -> Result<i64, String> {
    let voltage_levels = format!("[\"{}\"]", seed.voltage);
    let id = insert_id(
        conn,
        &format!(
            "INSERT INTO recipes \
             (name, model, [type], product, voltage_levels, is_active, created_at, updated_at, \
              modified_by, hipot_voltage, hipot_current_max, hipot_time) \
             VALUES ({}, {}, {}, {}, {}, 1, {}, {}, 'SEED', NULL, NULL, 10)",
            sql_text(seed.name),
            sql_text(seed.model),
            sql_text("current_consumption"),
            sql_text(seed.product),
            sql_text(&voltage_levels),
            sql_text(now),
            sql_text(now)
        ),
    )?;
    insert_children(conn, id, seed)?;
    Ok(id)
}

fn update_recipe(
    conn: &mut odbc_api::Connection<'_>,
    id: i64,
    seed: &RecipeSeed,
    now: &str,
) -> Result<(), String> {
    let voltage_levels = format!("[\"{}\"]", seed.voltage);
    execute(
        conn,
        &format!(
            "UPDATE recipes SET name = {}, product = {}, voltage_levels = {}, \
             updated_at = {}, modified_by = 'SEED', hipot_time = COALESCE(hipot_time, 10) WHERE id = {}",
            sql_text(seed.name),
            sql_text(seed.product),
            sql_text(&voltage_levels),
            sql_text(now),
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
    insert_children(conn, id, seed)?;
    Ok(())
}

fn insert_children(
    conn: &mut odbc_api::Connection<'_>,
    id: i64,
    seed: &RecipeSeed,
) -> Result<(), String> {
    for (name, min, max) in seed.components {
        execute(
            conn,
            &format!(
                "INSERT INTO recipe_components \
                 (recipe_id, name, voltage, enabled, min_value, max_value, unit) \
                 VALUES ({}, {}, {}, 1, {}, {}, 'A')",
                id,
                sql_text(name),
                sql_text(seed.voltage),
                sql_num(*min),
                sql_num(*max)
            ),
        )?;
    }
    execute(
        conn,
        &format!(
            "INSERT INTO recipe_totals (recipe_id, voltage, min_total, max_total) \
             VALUES ({}, {}, {}, {})",
            id,
            sql_text(seed.voltage),
            sql_num(seed.total.0),
            sql_num(seed.total.1)
        ),
    )?;
    Ok(())
}

fn insert_tag(conn: &mut odbc_api::Connection<'_>, tag: &PlcTagSeed) -> Result<(), String> {
    execute(
        conn,
        &format!(
            "IF NOT EXISTS (SELECT 1 FROM plc_tags WHERE name = {name}) \
             INSERT INTO plc_tags (name, db_number, [offset], data_type, bit, description, unit) \
             VALUES ({name}, 1, {offset}, {data_type}, {bit}, {description}, {unit})",
            name = sql_text(tag.name),
            offset = tag.offset,
            data_type = sql_text(tag.data_type),
            bit = tag.bit,
            description = sql_text(tag.description),
            unit = sql_text(tag.unit)
        ),
    )
}

/// La versión anterior guardaba RL como una sola receta con 4 voltajes.
/// Si no tiene historial, se elimina para dejar las 4 recetas por fila.
fn delete_legacy_rl(conn: &mut odbc_api::Connection<'_>) -> Result<(), String> {
    let Some(id) = recipe_id(conn, "RL")? else {
        return Ok(());
    };
    let used = query_one(
        conn,
        &format!(
            "SELECT COUNT(*) AS n FROM executions WHERE recipe_id = {}",
            id
        ),
    )?
    .and_then(|row| row.get("n").and_then(|v| v.as_i64()))
    .unwrap_or(0);
    if used == 0 {
        execute(
            conn,
            &format!("DELETE FROM recipe_components WHERE recipe_id = {}", id),
        )?;
        execute(
            conn,
            &format!("DELETE FROM recipe_totals WHERE recipe_id = {}", id),
        )?;
        execute(conn, &format!("DELETE FROM recipes WHERE id = {}", id))?;
    }
    Ok(())
}

/// Inserta recetas/tags de fábrica una sola vez (marcadores en `configurations`).
pub fn seed_express() -> Result<(), String> {
    with_transaction(|conn| {
        let now = now_text();
        if !marker_exists(conn, "seed_recipes_v1")? {
            delete_legacy_rl(conn)?;
            for seed in DEFAULT_RECIPES {
                if recipe_id(conn, seed.model)?.is_none() {
                    insert_recipe(conn, seed, &now)?;
                }
            }
            set_marker(conn, "seed_recipes_v1", &now)?;
        }
        if !marker_exists(conn, "seed_plc_tags_v1")? {
            for tag in DEFAULT_PLC_TAGS {
                insert_tag(conn, tag)?;
            }
            set_marker(conn, "seed_plc_tags_v1", &now)?;
        }
        Ok(())
    })
}

/// Restaura/actualiza las 4 recetas de fábrica sin borrar otras recetas.
pub fn restore_default_recipes() -> Result<Value, String> {
    with_transaction(|conn| {
        let now = now_text();
        delete_legacy_rl(conn)?;
        let mut inserted = 0;
        let mut updated = 0;
        for seed in DEFAULT_RECIPES {
            match recipe_id(conn, seed.model)? {
                Some(id) => {
                    update_recipe(conn, id, seed, &now)?;
                    updated += 1;
                }
                None => {
                    insert_recipe(conn, seed, &now)?;
                    inserted += 1;
                }
            }
        }
        Ok(json!({ "inserted": inserted, "updated": updated }))
    })
}
