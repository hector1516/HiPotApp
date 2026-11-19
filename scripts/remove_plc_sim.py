from pathlib import Path

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

start_marker = '// --- PLC admin / I/O / manual (Rust) ---'
end_marker = '// --- Hipot 3865 (Rust) ---'

start = s.index(start_marker)
end = s.index(end_marker)

new_block = r'''// --- PLC admin / I/O / manual (Rust, producción) ---

fn plc_cfg() -> plc::PlcConfig {
    plc::config_from(&get_config())
}

#[tauri::command]
fn plc_config_get() -> serde_json::Value {
    plc::config_to_json(&plc_cfg())
}

#[tauri::command]
fn plc_config_set(ip: String, rack: u16, slot: u16) -> String {
    set_config(serde_json::json!({
        "plc_ip": ip,
        "plc_rack": rack.to_string(),
        "plc_slot": slot.to_string(),
    }))
}

#[tauri::command]
fn plc_status_info() -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(_) => serde_json::json!({
            "connected": true, "ip": cfg.ip, "rack": cfg.rack, "slot": cfg.slot
        }),
        Err(e) => serde_json::json!({
            "connected": false, "ip": cfg.ip, "rack": cfg.rack,
            "slot": cfg.slot, "error": e
        }),
    }
}

#[tauri::command]
fn plc_tags_list() -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!([]),
    };
    let cfg = plc_cfg();
    let mut client = plc::connect(&cfg, 1500).ok();
    let mut arr = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, name, db_number, offset, data_type, bit, description, unit \
         FROM plc_tags ORDER BY id",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        }) {
            for (id, name, db_number, offset, data_type, bit, description, unit) in rows.flatten() {
                let value = if let Some(c) = client.as_mut() {
                    plc::read_tag(c, db_number as i32, offset as i32, &data_type, bit as i32)
                        .unwrap_or(serde_json::json!(null))
                } else {
                    serde_json::json!(null)
                };
                arr.push(serde_json::json!({
                    "id": id,
                    "name": name,
                    "db": db_number,
                    "offset": offset,
                    "type": data_type,
                    "bit": bit,
                    "description": description,
                    "unit": unit,
                    "value": value,
                }));
            }
        }
    }
    serde_json::Value::Array(arr)
}

#[tauri::command]
fn plc_tags_add(
    name: String,
    db_number: i64,
    offset: i64,
    data_type: String,
    bit: i64,
) -> i64 {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return -1,
    };
    conn.execute(
        "INSERT INTO plc_tags (name, db_number, offset, data_type, bit) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            name.trim().to_uppercase(),
            db_number,
            offset,
            data_type.trim().to_uppercase(),
            bit
        ],
    )
    .ok();
    conn.last_insert_rowid()
}

#[tauri::command]
fn plc_tags_delete(id: i64) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error".to_string(),
    };
    match conn.execute("DELETE FROM plc_tags WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => "deleted".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_tag_write_by_id(tag_id: i64, value: serde_json::Value) -> String {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return "error: no db".to_string(),
    };
    let tag = conn.query_row(
        "SELECT db_number, offset, data_type, bit FROM plc_tags WHERE id = ?1",
        rusqlite::params![tag_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        },
    );
    let (db_number, offset, data_type, bit) = match tag {
        Ok(v) => v,
        Err(e) => return format!("error: {}", e),
    };
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::write_tag(
            &mut c,
            db_number as i32,
            offset as i32,
            &data_type,
            &value,
            bit as i32,
        ) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_tag_read_by_id(tag_id: i64) -> serde_json::Value {
    let conn = match db::open() {
        Ok(c) => c,
        Err(_) => return serde_json::json!(null),
    };
    let tag = conn.query_row(
        "SELECT db_number, offset, data_type, bit FROM plc_tags WHERE id = ?1",
        rusqlite::params![tag_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        },
    );
    let (db_number, offset, data_type, bit) = match tag {
        Ok(v) => v,
        Err(_) => return serde_json::json!(null),
    };
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            plc::read_tag(&mut c, db_number as i32, offset as i32, &data_type, bit as i32)
                .unwrap_or(serde_json::json!(null))
        }
        Err(_) => serde_json::json!(null),
    }
}

#[tauri::command]
fn plc_io_status() -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let mut v = plc::read_io(&mut c);
            if let Some(obj) = v.as_object_mut() {
                obj.insert("connected".to_string(), serde_json::json!(true));
            }
            v
        }
        Err(e) => {
            let mut v = plc::empty_io();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("connected".to_string(), serde_json::json!(false));
                obj.insert("error".to_string(), serde_json::json!(e));
            }
            v
        }
    }
}

#[tauri::command]
fn plc_io_write(q: String, value: bool) -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::write_q(&mut c, &q, value) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_send_recipe(voltage: f64, components: serde_json::Value) -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::send_recipe(&mut c, voltage, &components) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_start() -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::start(&mut c) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_stop() -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => match plc::stop(&mut c) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("error: {}", e),
        },
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
fn plc_read_results() -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => plc::read_results(&mut c),
        Err(e) => {
            let mut v = plc::empty_results();
            if let Some(obj) = v.as_object_mut() {
                obj.insert("error".to_string(), serde_json::json!(e));
            }
            v
        }
    }
}

#[tauri::command]
fn plc_set_mode(mode: i64) -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let ok = plc::set_mode(&mut c, mode as i32).is_ok();
            let current = plc::get_mode(&mut c).unwrap_or(1);
            serde_json::json!({ "mode": current, "ok": ok })
        }
        Err(e) => serde_json::json!({ "mode": 1, "ok": false, "error": e }),
    }
}

#[tauri::command]
fn plc_get_mode() -> serde_json::Value {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => serde_json::json!({ "mode": plc::get_mode(&mut c).unwrap_or(1) }),
        Err(e) => serde_json::json!({ "mode": 1, "error": e }),
    }
}

#[tauri::command]
fn plc_test_component(component: String, voltage: f64) -> serde_json::Value {
    let name = component.trim().to_uppercase();
    let cfg = plc_cfg();
    match plc::connect(&cfg, 2000) {
        Ok(mut c) => match plc::test_component(&mut c, &name, voltage) {
            Ok(v) => v,
            Err(e) => serde_json::json!({ "component": name, "value": 0.0, "error": e }),
        },
        Err(e) => serde_json::json!({ "component": name, "value": 0.0, "error": e }),
    }
}

#[tauri::command]
fn plc_abort() -> String {
    let cfg = plc_cfg();
    match plc::connect(&cfg, 1500) {
        Ok(mut c) => {
            let _ = plc::stop(&mut c);
            let _ = plc::set_mode(&mut c, 1);
            "OK".to_string()
        }
        Err(e) => format!("error: {}", e),
    }
}

'''

s = s[:start] + new_block + s[end:]
p.write_text(s, encoding='utf-8')
print('plc commands rewritten')
