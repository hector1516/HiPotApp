p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''#[tauri::command]
fn canvas_to_zpl(template_json: String, data: serde_json::Value) -> String {
    let t: serde_json::Value = match serde_json::from_str(&template_json) {
        Ok(v) => v,
        Err(_) => return "^XA^XZ".to_string(),
    };
    let mm_to_dots = 203.0 / 25.4;
    let w = (t["label"]["width_mm"].as_f64().unwrap_or(100.0) * mm_to_dots) as i32;
    let h = (t["label"]["height_mm"].as_f64().unwrap_or(60.0) * mm_to_dots) as i32;
    let mut zpl = format!("^XA\\n^PW{}\\n^LL{}\\n\\n", w, h);

    let replace = |text: &str| -> String {
        let mut s = text.to_string();
        let reps = [
            ("{modelo}", data.get("model").and_then(|v| v.as_str()).unwrap_or("")),
            ("{serie}", data.get("serial_number").and_then(|v| v.as_str()).unwrap_or("")),
            ("{batch}", data.get("batch_number").and_then(|v| v.as_str()).unwrap_or("")),
            ("{operador}", data.get("operator_name").and_then(|v| v.as_str()).unwrap_or("")),
            ("{fecha}", data.get("date").and_then(|v| v.as_str()).unwrap_or("")),
            ("{hora}", data.get("time").and_then(|v| v.as_str()).unwrap_or("")),
            ("{duracion}", data.get("duration").and_then(|v| v.as_str()).unwrap_or("")),
            ("{estado}", data.get("final_status").and_then(|v| v.as_str()).unwrap_or("")),
            ("{voltaje}", data.get("voltage").and_then(|v| v.as_str()).unwrap_or("")),
            ("{fan}", data.get("fan_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{door}", data.get("door_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{frame}", data.get("frame_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{drain}", data.get("drain_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{defrost}", data.get("defrost_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{light}", data.get("light_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{total}", data.get("total_value").and_then(|v| v.as_str()).unwrap_or("")),
            ("{plc_status}", data.get("plc_pass").and_then(|v| v.as_str()).unwrap_or("")),
            ("{hipot_status}", data.get("hipot_pass").and_then(|v| v.as_str()).unwrap_or("")),
        ];
        for (k, v) in reps {
            s = s.replace(k, v);
        }
        s
    };

    for obj in t.get("objects").and_then(|v| v.as_array()).unwrap_or(&vec![]) {
        let x = (obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) * mm_to_dots / 10.0) as i32;
        let y = (obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) * mm_to_dots / 10.0) as i32;
        let typ = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match typ {
            "text" => {
                let txt = replace(obj.get("content").and_then(|v| v.as_str()).unwrap_or(""));
                let size = obj.get("font_size").and_then(|v| v.as_i64()).unwrap_or(24) as i32;
                zpl.push_str(&format!("^FO{},{}^A0N,{},{}^FD{}^FS\\n", x, y, size, size, txt));
            }
            "barcode" => {
                let data = replace(obj.get("data_source").and_then(|v| v.as_str()).unwrap_or(""));
                let hh = obj.get("height").and_then(|v| v.as_i64()).unwrap_or(80) as i32;
                zpl.push_str(&format!("^FO{},{}^BY3^BCN,{},Y,N,N^FD{}^FS\\n", x, y, hh, data));
            }
            "qrcode" => {
                let data = replace(obj.get("data_source").and_then(|v| v.as_str()).unwrap_or(""));
                zpl.push_str(&format!("^FO{},{}^BQ2,3,N,N^FD{}^FS\\n", x, y, data));
            }
            "line" => {
                let x2 = (obj.get("x2").and_then(|v| v.as_f64()).unwrap_or(obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) + 100.0) * mm_to_dots / 10.0) as i32;
                let th = obj.get("thickness").and_then(|v| v.as_i64()).unwrap_or(2) as i32;
                zpl.push_str(&format!("^FO{},{}^GB{},{},{}^FS\\n", x, y, (x2 - x).max(1), th, th));
            }
            "rect" => {
                let w = (obj.get("w").and_then(|v| v.as_f64()).unwrap_or(100.0) * mm_to_dots / 10.0) as i32;
                let h = (obj.get("h").and_then(|v| v.as_f64()).unwrap_or(40.0) * mm_to_dots / 10.0) as i32;
                let th = obj.get("thickness").and_then(|v| v.as_i64()).unwrap_or(2) as i32;
                zpl.push_str(&format!("^FO{},{}^GB{},{},{}^FS\\n", x, y, w.max(1), h.max(1), th));
            }
            _ => {}
        }
    }
    format!("{}\\n^XZ", zpl)
}

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, canvas_to_zpl])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")