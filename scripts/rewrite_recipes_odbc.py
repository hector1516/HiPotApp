from pathlib import Path
import re

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

replacements = {
    'list_recipes': '''#[tauri::command]
fn list_recipes() -> serde_json::Value {
    db::repo::recipes_list()
}
''',
    'get_recipe_by_model': '''#[tauri::command]
fn get_recipe_by_model(model: String) -> serde_json::Value {
    db::repo::recipe_by_model(&model)
}
''',
    'create_recipe': '''#[tauri::command]
fn create_recipe(name: String, model: String, type_: String, product: Option<String>, voltage_levels: String, components: serde_json::Value, totals: serde_json::Value, modified_by: Option<String>) -> i64 {
    match db::repo::recipe_create(
        &name, &model, &type_, product.as_deref(), &voltage_levels,
        &components, &totals, modified_by.as_deref(),
    ) {
        Ok(id) => id,
        Err(e) => { eprintln!("[recipe_create] {}", e); -1 }
    }
}
''',
    'update_recipe': '''#[tauri::command]
fn update_recipe(id: i64, name: String, model: String, type_: String, product: Option<String>, voltage_levels: String, components: serde_json::Value, totals: serde_json::Value, modified_by: Option<String>) -> String {
    match db::repo::recipe_update(
        id, &name, &model, &type_, product.as_deref(), &voltage_levels,
        &components, &totals, modified_by.as_deref(),
    ) {
        Ok(_) => "updated".to_string(),
        Err(e) => format!("error: {}", e),
    }
}
''',
    'delete_recipe': '''#[tauri::command]
fn delete_recipe(id: i64) -> String {
    match db::repo::recipe_delete(id) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}
''',
    'duplicate_recipe': '''#[tauri::command]
fn duplicate_recipe(id: i64, new_model: String) -> i64 {
    match db::repo::recipe_duplicate(id, &new_model) {
        Ok(id) => id,
        Err(e) => { eprintln!("[duplicate_recipe] {}", e); -1 }
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
print('recipes replaced')
