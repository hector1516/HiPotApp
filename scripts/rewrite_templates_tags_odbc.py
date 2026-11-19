from pathlib import Path
import re

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

replacements = {
    'template_list': '''#[tauri::command]
fn template_list() -> serde_json::Value {
    db::repo::templates_list()
}
''',
    'template_get': '''#[tauri::command]
fn template_get(tid: i64) -> Result<serde_json::Value, String> {
    db::repo::template_get(tid)
}
''',
    'template_create': '''#[tauri::command]
fn template_create(name: String, width_mm: i64, height_mm: i64, canvas_data: String, is_default: bool) -> Result<i64, String> {
    db::repo::template_create(&name, width_mm, height_mm, &canvas_data, is_default)
}
''',
    'template_update': '''#[tauri::command]
fn template_update(tid: i64, name: String, width_mm: i64, height_mm: i64, canvas_data: String, is_default: bool) -> Result<String, String> {
    db::repo::template_update(tid, &name, width_mm, height_mm, &canvas_data, is_default)?;
    Ok("saved".to_string())
}
''',
    'template_delete': '''#[tauri::command]
fn template_delete(tid: i64) -> Result<String, String> {
    db::repo::template_delete(tid)?;
    Ok("deleted".to_string())
}
''',
    'plc_tags_add': '''#[tauri::command]
fn plc_tags_add(name: String, db_number: i64, offset: i64, data_type: String, bit: i64) -> i64 {
    match db::repo::plc_tags_add(&name, db_number, offset, &data_type, bit) {
        Ok(id) => id,
        Err(e) => { eprintln!("[plc_tags_add] {}", e); -1 }
    }
}
''',
    'plc_tags_delete': '''#[tauri::command]
fn plc_tags_delete(id: i64) -> String {
    match db::repo::plc_tags_delete(id) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}
''',
    'barcode_scan': '''#[tauri::command]
fn barcode_scan(code: String, barcode_type: String, scan_order: i64, execution_id: Option<i64>) -> Result<serde_json::Value, String> {
    db::repo::barcode_scan(&code, &barcode_type, scan_order, execution_id)
}
''',
    'history_list': '''#[tauri::command]
fn history_list(model: Option<String>, serial: Option<String>, start: Option<String>, end: Option<String>, limit: Option<i64>) -> serde_json::Value {
    db::repo::history_list(
        model.as_deref(), serial.as_deref(), start.as_deref(), end.as_deref(),
        limit.unwrap_or(200),
    )
}
''',
    'sync_log_list': '''#[tauri::command]
fn sync_log_list(limit: Option<i64>) -> serde_json::Value {
    db::repo::sync_log_list(limit.unwrap_or(20))
}
''',
    'import_export_log_list': '''#[tauri::command]
fn import_export_log_list(limit: i64) -> serde_json::Value {
    db::repo::import_export_log_list(limit)
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
print('templates/tags/logs replaced')
