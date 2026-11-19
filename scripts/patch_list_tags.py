p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
old = '''#[tauri::command]
fn list_plc_tags() -> serde_json::Value {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS plc_tags (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, db_number INTEGER DEFAULT 1, offset INTEGER DEFAULT 0, data_type TEXT DEFAULT 'REAL', bit INTEGER DEFAULT 0, description TEXT, unit TEXT)", []);
    let mut stmt = match conn.prepare("SELECT id, name, db_number, offset, data_type, bit, description, unit FROM plc_tags") {
        Ok(s) => s,
        Err(_) => return serde_json::json!([]),
    };
    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "db": row.get::<_, i64>(2)?,
            "offset": row.get::<_, i64>(3)?,
            "type": row.get::<_, String>(4)?,
            "bit": row.get::<_, i64>(5)?,
            "description": row.get::<_, Option<String>>(6)?,
            "unit": row.get::<_, Option<String>>(7)?,
        }))
    });
    let mut arr = Vec::new();
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            arr.push(r);
        }
    }
    serde_json::Value::Array(arr)
}'''
new = '''#[tauri::command]
fn list_plc_tags() -> serde_json::Value {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS plc_tags (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, db_number INTEGER DEFAULT 1, offset INTEGER DEFAULT 0, data_type TEXT DEFAULT 'REAL', bit INTEGER DEFAULT 0, description TEXT, unit TEXT)", []);
    let mut stmt = match conn.prepare("SELECT id, name, db_number, offset, data_type, bit, description, unit FROM plc_tags") {
        Ok(s) => s,
        Err(_) => return serde_json::json!([]),
    };
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?, row.get::<_, String>(4)?, row.get::<_, i64>(5)?, row.get::<_, Option<String>>(6)?, row.get::<_, Option<String>>(7)?))
    });
    let mut arr = Vec::new();
    // Read config for IP/rack/slot
    let cfg = get_config();
    let ip = cfg.get("plc_ip").and_then(|v| v.as_str()).unwrap_or("192.168.1.10").to_string();
    let rack = cfg.get("plc_rack").and_then(|v| v.as_str()).unwrap_or("0").parse::<u16>().unwrap_or(0);
    let slot = cfg.get("plc_slot").and_then(|v| v.as_str()).unwrap_or("1").parse::<u16>().unwrap_or(1);
    if let Ok(rows) = rows {
        for r in rows.flatten() {
            let (id, name, db, offset, data_type, bit, description, unit) = r;
            // Read value from PLC
            let mut val = serde_json::Value::Null;
            if let Ok(v) = read_plc_tag(ip.clone(), rack, slot, db, offset, data_type.clone(), bit) {
                val = v;
            }
            arr.push(serde_json::json!({
                "id": id,
                "name": name,
                "db": db,
                "offset": offset,
                "type": data_type,
                "bit": bit,
                "description": description,
                "unit": unit,
                "value": val,
            }));
        }
    }
    serde_json::Value::Array(arr)
}'''
if old in s:
    s = s.replace(old, new)
    open(p, 'w', encoding='utf-8').write(s)
    print("done")
else:
    print("not found")