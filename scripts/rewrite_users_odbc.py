from pathlib import Path
import re

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

replacements = {
    'list_operators': '''#[tauri::command]
fn list_operators() -> serde_json::Value {
    db::repo::operators_list()
}
''',
    'add_operator': '''#[tauri::command]
fn add_operator(name: String, employee_id: String) -> i64 {
    match db::repo::operator_add(&name, &employee_id) {
        Ok(id) => id,
        Err(e) => { eprintln!("[add_operator] {}", e); -1 }
    }
}
''',
    'list_users': '''#[tauri::command]
fn list_users() -> serde_json::Value {
    db::repo::users_list()
}
''',
    'add_user': '''#[tauri::command]
fn add_user(username: String, password: String, role: String) -> i64 {
    match db::repo::user_add(&username, &password, &role) {
        Ok(id) => id,
        Err(e) => { eprintln!("[add_user] {}", e); -1 }
    }
}
''',
    'delete_user': '''#[tauri::command]
fn delete_user(id: i64) -> String {
    match db::repo::user_delete(id) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}
''',
    'auth_login': '''#[tauri::command]
fn auth_login(username: String) -> Result<serde_json::Value, String> {
    db::repo::auth_login(&username)
}
''',
    'user_get': '''#[tauri::command]
fn user_get(uid: i64) -> Result<serde_json::Value, String> {
    db::repo::user_get(uid)
}
''',
    'user_update': '''#[tauri::command]
fn user_update(uid: i64, username: Option<String>, role: Option<String>, is_active: Option<bool>) -> Result<String, String> {
    db::repo::user_update(uid, username.as_deref(), role.as_deref(), is_active)?;
    Ok("updated".to_string())
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
print('operators/users replaced')
