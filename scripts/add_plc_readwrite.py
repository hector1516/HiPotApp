p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''#[tauri::command]
fn read_plc_tag(ip: String, rack: u16, slot: u16, db: i64, offset: i64, data_type: String, bit: i64) -> Result<serde_json::Value, String> {
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = ip.parse().map_err(|_| "invalid ip".to_string())?;
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);
    let transport = s7::tcp::Transport::connect(opts).map_err(|e| e.to_string())?;
    let mut client = s7::client::Client::new(transport).map_err(|e| e.to_string())?;
    let dt = data_type.to_uppercase();
    match dt.as_str() {
        "BOOL" => {
            let mut buf = vec![0u8; 1];
            client.ag_read(db as i32, offset as i32, 1, &mut buf).map_err(|e| e.to_string())?;
            let val = (buf[0] >> bit) & 1 == 1;
            Ok(serde_json::json!(val))
        }
        "WORD" | "INT" => {
            let mut buf = vec![0u8; 2];
            client.ag_read(db as i32, offset as i32, 2, &mut buf).map_err(|e| e.to_string())?;
            let val = u16::from_be_bytes([buf[0], buf[1]]);
            Ok(serde_json::json!(val))
        }
        "REAL" => {
            let mut buf = vec![0u8; 4];
            client.ag_read(db as i32, offset as i32, 4, &mut buf).map_err(|e| e.to_string())?;
            let val = f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            Ok(serde_json::json!(val))
        }
        _ => Err("unsupported type".to_string()),
    }
}

#[tauri::command]
fn write_plc_tag(ip: String, rack: u16, slot: u16, db: i64, offset: i64, data_type: String, value: serde_json::Value, bit: i64) -> Result<String, String> {
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = ip.parse().map_err(|_| "invalid ip".to_string())?;
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);
    let transport = s7::tcp::Transport::connect(opts).map_err(|e| e.to_string())?;
    let mut client = s7::client::Client::new(transport).map_err(|e| e.to_string())?;
    let dt = data_type.to_uppercase();
    match dt.as_str() {
        "BOOL" => {
            let val = value.as_bool().unwrap_or(false);
            let mut buf = vec![0u8; 1];
            client.ag_read(db as i32, offset as i32, 1, &mut buf).map_err(|e| e.to_string())?;
            if val { buf[0] |= 1 << bit; } else { buf[0] &= !(1 << bit); }
            client.ag_write(db as i32, offset as i32, 1, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "WORD" | "INT" => {
            let val = value.as_i64().unwrap_or(0) as u16;
            let mut buf = val.to_be_bytes().to_vec();
            client.ag_write(db as i32, offset as i32, 2, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "REAL" => {
            let val = value.as_f64().unwrap_or(0.0) as f32;
            let mut bytes = val.to_be_bytes().to_vec();
            client.ag_write(db as i32, offset as i32, 4, &mut bytes).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        _ => Err("unsupported type".to_string()),
    }
}

'''
s = s.replace(marker, cmds + marker, 1)
# register commands
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, list_plc_tags, add_plc_tag, delete_plc_tag, read_plc_tag, write_plc_tag])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")