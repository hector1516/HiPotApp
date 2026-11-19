from pathlib import Path
import re

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

replacements = {
    'start_execution': '''#[tauri::command]
fn start_execution(recipe_id: i64, operator_id: Option<i64>, batch: Option<String>, serial_number: Option<String>) -> i64 {
    match db::repo::execution_start(recipe_id, operator_id, batch.as_deref(), serial_number.as_deref()) {
        Ok(id) => id,
        Err(e) => { eprintln!("[start_execution] {}", e); -1 }
    }
}
''',
    'save_plc_result': '''#[tauri::command]
fn save_plc_result(eid: i64, passed: bool, readings: serde_json::Value) -> String {
    match db::repo::save_plc_result(eid, passed, &readings) {
        Ok(_) => "saved".to_string(),
        Err(e) => format!("error: {}", e),
    }
}
''',
    'save_hipot_result': '''#[tauri::command]
fn save_hipot_result(eid: i64, body: serde_json::Value) -> String {
    match db::repo::save_hipot_result(eid, &body) {
        Ok(status) => status,
        Err(e) => format!("error: {}", e),
    }
}
''',
    'list_executions': '''#[tauri::command]
fn list_executions(limit: i64) -> serde_json::Value {
    db::repo::executions_list(limit)
}
''',
    'list_logs': '''#[tauri::command]
fn list_logs(category: Option<String>, level: Option<String>, q: Option<String>, limit: i64) -> serde_json::Value {
    db::repo::logs_list(category.as_deref(), level.as_deref(), q.as_deref(), limit)
}
''',
    'log_action': '''#[tauri::command]
fn log_action(level: String, category: String, action: String, detail: Option<String>, execution_id: Option<i64>, value_sent: Option<String>, value_received: Option<String>, duration_ms: Option<i64>, success: Option<bool>, error_message: Option<String>) -> i64 {
    match db::repo::log_add(
        &level, &category, &action, detail.as_deref(), execution_id,
        value_sent.as_deref(), value_received.as_deref(), duration_ms,
        success.unwrap_or(true), error_message.as_deref(),
    ) {
        Ok(id) => id,
        Err(e) => { eprintln!("[log_action] {}", e); -1 }
    }
}
''',
}

for name, new in replacements.items():
    pattern = r'(?ms)^#\[tauri::command\]\nfn ' + re.escape(name) + r'\(.*?\n\}\n'
    s2, n = re.subn(pattern, new, s, count=1)
    if n == 0:
        print('NOT FOUND', name)
    s = s2

p.write_text(s, encoding='utf-8')
print('executions/logs replaced')
