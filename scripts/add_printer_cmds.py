p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''const DEFAULT_TEMPLATE_JSON: &str = r#"{"label":{"width_mm":100,"height_mm":60,"bg_color":"#FFFFFF"},"objects":[{"id":"t1","type":"text","x":50,"y":20,"content":"HIPOT - RESULTADO","font_size":28,"bold":true},{"id":"t2","type":"text","x":50,"y":60,"content":"Modelo: {modelo}","font_size":18},{"id":"t3","type":"text","x":50,"y":85,"content":"Serie: {serie}","font_size":18},{"id":"t4","type":"text","x":50,"y":115,"content":"Estado: {estado}","font_size":20,"bold":true},{"id":"b1","type":"barcode","x":50,"y":300,"data_source":"{modelo}-{serie}-{batch}","height":80},{"id":"q1","type":"qrcode","x":400,"y":280,"size":100,"data_source":"{modelo}|{serie}|{batch}|{estado}"}]}"#;

#[tauri::command]
fn print_test_zebra(printer_ip: String, printer_port: u16, printer_name: String) -> String {
    let data = serde_json::json!({
        "model": "TEST", "serial_number": "12345", "batch_number": "B001",
        "final_status": "GOOD", "date": "", "time": ""
    });
    let zpl = canvas_to_zpl(DEFAULT_TEMPLATE_JSON.to_string(), data);
    if !printer_ip.is_empty() {
        print_zpl_ethernet(printer_ip, printer_port, zpl)
    } else {
        print_zpl_windows(printer_name, zpl)
    }
}

#[tauri::command]
fn reprint_zebra(eid: i64, printer_ip: String, printer_port: u16, printer_name: String) -> String {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return "error: cannot open db".to_string(),
    };
    let row = conn.query_row(
        "SELECT serial_number, final_status FROM executions WHERE id = ?1",
        rusqlite::params![eid],
        |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?)),
    );
    let (serial, status) = match row {
        Ok(v) => v,
        Err(_) => return "error: execution not found".to_string(),
    };
    let data = serde_json::json!({
        "model": "", "serial_number": serial.unwrap_or_default(),
        "batch_number": "", "final_status": status.unwrap_or_default(),
        "date": "", "time": ""
    });
    let zpl = canvas_to_zpl(DEFAULT_TEMPLATE_JSON.to_string(), data);
    if !printer_ip.is_empty() {
        print_zpl_ethernet(printer_ip, printer_port, zpl)
    } else {
        print_zpl_windows(printer_name, zpl)
    }
}

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, restart_backend])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, restart_backend, print_test_zebra, reprint_zebra, canvas_to_zpl, print_zpl_ethernet, print_zpl_windows])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")