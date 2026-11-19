from pathlib import Path
import re

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

new_fn = r'''#[tauri::command]
fn run_test_cycle(
    app: tauri::AppHandle,
    execution_id: i64,
    recipe_id: i64,
    voltage: f64,
    components: serde_json::Value,
) -> serde_json::Value {
    let started = std::time::Instant::now();
    let mut errors: Vec<String> = Vec::new();
    let mut hipot_parsed = serde_json::json!({});
    let mut hipot_pass = false;
    let mut plc_pass = false;
    let mut plc_values = serde_json::json!({});
    let mut plc_component_values = serde_json::json!({});

    // --- Hipot primero: detecta cortos antes de energizar la prueba eléctrica ---
    let _ = app.emit("test-phase", "HIPOT_RUNNING");
    let hipot_cfg = hipot::config_from(&get_config());

    match hipot::test_action(&hipot_cfg, "START") {
        Ok(v) => {
            hipot_parsed = v
                .get("parsed")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            if hipot_parsed
                .get("pass")
                .and_then(|p| p.as_bool())
                .is_none()
            {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    match hipot::command(&hipot_cfg, "STATUS?") {
                        Ok(s) => {
                            let parsed = s
                                .get("parsed")
                                .cloned()
                                .unwrap_or_else(|| serde_json::json!({}));
                            if parsed.get("pass").and_then(|p| p.as_bool()).is_some() {
                                hipot_parsed = parsed;
                                break;
                            }
                            hipot_parsed = parsed;
                        }
                        Err(e) => errors.push(format!("Hipot status: {}", e)),
                    }
                    if std::time::Instant::now() >= deadline {
                        errors.push("Hipot: timeout esperando PASS/FAIL".to_string());
                        break;
                    }
                }
            }
            hipot_pass = hipot_parsed
                .get("pass")
                .and_then(|p| p.as_bool())
                .unwrap_or(false);
        }
        Err(e) => errors.push(format!("Hipot start: {}", e)),
    }

    let hipot_duration = started.elapsed().as_secs_f64();
    let _ = save_hipot_result(
        execution_id,
        serde_json::json!({
            "pass": hipot_pass,
            "voltage": hipot_parsed.get("voltage"),
            "current": hipot_parsed.get("current"),
            "resistance": hipot_parsed.get("resistance"),
            "insulation": hipot_parsed.get("insulation"),
            "duration": hipot_duration,
        }),
    );

    if !hipot_pass {
        let _ = db::repo::set_final_status(execution_id, "BAD");
        let duration = started.elapsed().as_secs_f64();
        return serde_json::json!({
            "execution_id": execution_id,
            "recipe_id": recipe_id,
            "voltage": voltage,
            "plc": false,
            "hipot": false,
            "final": "BAD",
            "values": {},
            "plc_results": {},
            "hipot_data": hipot_parsed,
            "duration": duration,
            "errors": errors,
            "skipped_electrical": true,
        });
    }

    // --- Prueba eléctrica (solo si Hipot pasó) ---
    let _ = app.emit("test-phase", "PLC_RUNNING");
    let cfg = plc_cfg();
    match plc::connect(&cfg, 2000) {
        Ok(mut client) => {
            if let Err(e) = plc::send_recipe(&mut client, voltage, &components) {
                errors.push(format!("PLC receta: {}", e));
            }
            let _ = plc::set_mode(&mut client, 1);
            if let Err(e) = plc::start(&mut client) {
                errors.push(format!("PLC start: {}", e));
            }

            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
            loop {
                let results = plc::read_results(&mut client);
                let mut snapshot = serde_json::Map::new();
                if let Some(arr) = components.as_array() {
                    for comp in arr {
                        let name = comp
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_uppercase();
                        if name.is_empty() {
                            continue;
                        }
                        if let Some(value) = results.get(&name) {
                            snapshot.insert(name, value.clone());
                        }
                    }
                }
                if !snapshot.is_empty() {
                    let _ = app.emit("test-values", serde_json::Value::Object(snapshot));
                }
                let complete = results
                    .get("complete")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if complete {
                    plc_values = results;
                    break;
                }
                if std::time::Instant::now() >= deadline {
                    errors.push("PLC: timeout esperando Test_Complete".to_string());
                    plc_values = results;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            plc_pass = plc_values
                .get("pass")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let mut readings = Vec::new();
            if let Some(arr) = components.as_array() {
                for comp in arr {
                    let name = comp
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_uppercase();
                    if name.is_empty() {
                        continue;
                    }
                    let min = comp.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let max = comp.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let value = plc_values
                        .get(&name)
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let ok = value >= min && value <= max;
                    readings.push(serde_json::json!({
                        "component": name,
                        "voltage": voltage.to_string(),
                        "tag": name,
                        "value": value,
                        "min": min,
                        "max": max,
                        "pass": ok,
                    }));
                }
            }
            let mut component_values = serde_json::Map::new();
            for reading in &readings {
                if let Some(name) = reading.get("component").and_then(|v| v.as_str()) {
                    let value = reading
                        .get("value")
                        .cloned()
                        .unwrap_or(serde_json::json!(0.0));
                    component_values.insert(name.to_string(), value);
                }
            }
            plc_component_values = serde_json::Value::Object(component_values);
            let _ = save_plc_result(execution_id, plc_pass, serde_json::json!(readings));
        }
        Err(e) => {
            errors.push(format!("PLC no conectado: {}", e));
            let _ = save_plc_result(execution_id, false, serde_json::json!([]));
        }
    }

    let final_status = if plc_pass && hipot_pass { "GOOD" } else { "BAD" };
    let _ = db::repo::set_final_status(execution_id, final_status);
    let duration = started.elapsed().as_secs_f64();

    serde_json::json!({
        "execution_id": execution_id,
        "recipe_id": recipe_id,
        "voltage": voltage,
        "plc": plc_pass,
        "hipot": hipot_pass,
        "final": final_status,
        "values": plc_component_values,
        "plc_results": plc_values,
        "hipot_data": hipot_parsed,
        "duration": duration,
        "errors": errors,
        "skipped_electrical": false,
    })
}

'''

pattern = r'(?ms)^#\[tauri::command\]\nfn run_test_cycle\(.*?\n\}\n\n#\[tauri::command\]\nfn template_list'
s2, n = re.subn(pattern, new_fn + '#[tauri::command]\nfn template_list', s, count=1)
if n == 0:
    print('NOT FOUND')
else:
    p.write_text(s2, encoding='utf-8')
    print('run_test_cycle reordered: Hipot first')
