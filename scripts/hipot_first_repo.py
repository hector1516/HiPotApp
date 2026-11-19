from pathlib import Path
import re

p = Path('tauri/src-tauri/src/db/repo.rs')
s = p.read_text(encoding='utf-8')

new_fn = r'''pub fn save_hipot_result(execution_id: i64, body: &Value) -> Result<String, String> {
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
        let plc_pass = execution.get("plc_pass_fail").and_then(|v| v.as_bool());
        let hipot_pass = body.get("pass").and_then(|v| v.as_bool()).unwrap_or(false);
        let duration = body.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let final_status = plc_pass.map(|p| if p && hipot_pass { "GOOD" } else { "BAD" });

        if let Some(status) = final_status {
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
                    sql_text(status),
                    sql_num(duration),
                    execution_id
                ),
            )?;
        } else {
            execute(
                conn,
                &format!(
                    "UPDATE executions SET hipot_start_time = {}, hipot_end_time = {}, \
                     hipot_pass_fail = {}, hipot_duration = {}, total_duration = {} WHERE id = {}",
                    now,
                    now,
                    sql_bool(hipot_pass),
                    sql_num(duration),
                    sql_num(duration),
                    execution_id
                ),
            )?;
        }

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

        Ok(final_status.unwrap_or("PENDING").to_string())
    })
}

pub fn set_final_status(execution_id: i64, status: &str) -> Result<(), String> {
    with_connection(|conn| {
        execute(
            conn,
            &format!(
                "UPDATE executions SET final_status = {} WHERE id = {}",
                sql_text(status),
                execution_id
            ),
        )
    })
}

'''

pattern = r'(?ms)^pub fn save_hipot_result\(.*?\n\}\n\npub fn executions_list'
s2, n = re.subn(pattern, new_fn + 'pub fn executions_list', s, count=1)
if n == 0:
    print('NOT FOUND')
else:
    p.write_text(s2, encoding='utf-8')
    print('hipot save refactored')
