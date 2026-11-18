// HiPotApp: ventana Tauri + arranque/parada automática del backend Python
// empaquetado como recurso (todo-en-uno, offline-first).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{Manager, State, WindowEvent};

const BACKEND_BIN: &str = "hipot-backend-x86_64-pc-windows-msvc.exe";

fn db_path() -> std::path::PathBuf {
    let base = std::env::var("APPDATA")
        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let _ = std::fs::create_dir_all(&base);
    base.join("hipot_local.db")
}

#[tauri::command]
fn get_config() -> serde_json::Value {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return serde_json::json!({}),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS configuration (config_key TEXT PRIMARY KEY, config_value TEXT)", []);
    let mut map = serde_json::Map::new();
    if let Ok(mut stmt) = conn.prepare("SELECT config_key, config_value FROM configuration") {
        if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
            for r in rows.flatten() {
                map.insert(r.0, serde_json::Value::String(r.1));
            }
        }
    }
    serde_json::Value::Object(map)
}

#[tauri::command]
fn set_config(values: serde_json::Value) -> String {
    let conn = match rusqlite::Connection::open(db_path()) {
        Ok(c) => c,
        Err(_) => return "error: cannot open db".to_string(),
    };
    let _ = conn.execute("CREATE TABLE IF NOT EXISTS configuration (config_key TEXT PRIMARY KEY, config_value TEXT)", []);
    if let Some(obj) = values.as_object() {
        for (k, v) in obj {
            let val = v.as_str().unwrap_or(&v.to_string()).to_string();
            let _ = conn.execute(
                "INSERT INTO configuration (config_key, config_value) VALUES (?1, ?2) ON CONFLICT(config_key) DO UPDATE SET config_value = excluded.config_value",
                rusqlite::params![k, val],
            );
        }
    }
    "saved".to_string()
}

#[tauri::command]
fn read_backend_log() -> String {
    let log_dir = std::env::var("APPDATA")
        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_path = log_dir.join("backend.log");
    std::fs::read_to_string(&log_path).unwrap_or_else(|_| "No hay log disponible.".to_string())
}

#[tauri::command]
fn plc_status(ip: String, rack: u16, slot: u16) -> bool {
    use std::net::IpAddr;
    use std::time::Duration;
    let addr: IpAddr = match ip.parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let mut opts = s7::tcp::Options::new(addr, rack, slot, s7::transport::Connection::OP);
    opts.read_timeout = Duration::from_secs(2);
    opts.write_timeout = Duration::from_secs(2);
    let transport = match s7::tcp::Transport::connect(opts) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let mut client = match s7::client::Client::new(transport) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut buf = vec![0u8; 1];
    client.ab_read(0, 1, &mut buf).is_ok()
}

#[tauri::command]
fn plc_read_tag(ip: String, rack: u16, slot: u16, db: i32, offset: i32, data_type: String, bit: i32) -> Result<serde_json::Value, String> {
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
            client.ab_read(offset, 1, &mut buf).map_err(|e| e.to_string())?;
            let val = (buf[0] >> bit) & 1 == 1;
            Ok(serde_json::json!(val))
        }
        "WORD" | "INT" => {
            let mut buf = vec![0u8; 2];
            client.ab_read(offset, 2, &mut buf).map_err(|e| e.to_string())?;
            let val = u16::from_be_bytes([buf[0], buf[1]]);
            Ok(serde_json::json!(val))
        }
        "REAL" => {
            let mut buf = vec![0u8; 4];
            client.ag_read(db, offset, 4, &mut buf).map_err(|e| e.to_string())?;
            // Parse float from bytes (big-endian)
            let val = f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            Ok(serde_json::json!(val))
        }
        _ => Err("unsupported type".to_string()),
    }
}

#[tauri::command]
fn plc_write_tag(ip: String, rack: u16, slot: u16, db: i32, offset: i32, data_type: String, value: serde_json::Value, bit: i32) -> Result<String, String> {
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
            client.ab_read(offset, 1, &mut buf).map_err(|e| e.to_string())?;
            if val {
                buf[0] |= 1 << bit;
            } else {
                buf[0] &= !(1 << bit);
            }
            client.ab_write(offset, 1, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "WORD" | "INT" => {
            let val = value.as_i64().unwrap_or(0) as u16;
            let mut buf = val.to_be_bytes().to_vec();
            client.ab_write(offset, 2, &mut buf).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        "REAL" => {
            let val = value.as_f64().unwrap_or(0.0) as f32;
            let mut bytes = val.to_be_bytes().to_vec();
            client.ag_write(db, offset, 4, &mut bytes).map_err(|e| e.to_string())?;
            Ok("written".to_string())
        }
        _ => Err("unsupported type".to_string()),
    }
}

#[tauri::command]
fn show_touch_keyboard() {
    #[cfg(target_os = "windows")]
    {
        let tabtip = "C:\\Program Files\\Common Files\\Microsoft Shared\\Ink\\TabTip.exe";
        let _ = std::process::Command::new(tabtip).spawn();
    }
}

#[tauri::command]
fn open_external(url: String) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn();
    }
}

struct Backend(Mutex<Option<Child>>);

fn backend_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    use tauri::path::BaseDirectory;
    // 1. Directorio de recursos (app instalada / bundle)
    if let Ok(p) = app
        .path()
        .resolve(format!("binaries/{BACKEND_BIN}"), BaseDirectory::Resource)
    {
        if p.exists() {
            return Some(p);
        }
    }
    // 2. Junto al .exe o subiendo hasta 4 niveles (cubre target/release → tauri/binaries)
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..5 {
            if let Some(d) = dir.clone() {
                let p = d.join(BACKEND_BIN);
                if p.exists() { return Some(p); }
                let p = d.join(format!("binaries/{BACKEND_BIN}"));
                if p.exists() { return Some(p); }
                dir = d.parent().map(|p| p.to_path_buf());
            }
        }
    }
    // 3. Fallback dev: cwd
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(format!("../binaries/{BACKEND_BIN}"));
        if p.exists() { return Some(p); }
        let p = cwd.join(format!("binaries/{BACKEND_BIN}"));
        if p.exists() { return Some(p); }
    }
    None
}

fn kill_backend(state: &State<Backend>) {
    let mut guard = match state.0.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, plc_read_tag, plc_write_tag])
        .manage(Backend(Mutex::new(None)))
        .setup(|app| {
            match backend_path(app.handle()) {
                Some(path) => {
                    // Redirigir stderr a un archivo de log para diagnóstico
                    use std::fs::{OpenOptions, create_dir_all};
                    use std::io::Write;
                    let log_dir = std::env::var("APPDATA")
                        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
                        .unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let _ = create_dir_all(&log_dir);
                    let log_path = log_dir.join("backend.log");
                    let stderr_file = OpenOptions::new()
                        .create(true).append(true).open(&log_path).ok();
                    let mut cmd = Command::new(&path);
                    if let Some(f) = stderr_file {
                        cmd.stderr(std::process::Stdio::from(f));
                    }
                    match cmd.spawn() {
                        Ok(child) => {
                            *app.state::<Backend>().0.lock().unwrap() = Some(child);
                            eprintln!("[hipot] backend iniciado: {}", path.display());
                            use std::io::Write;
                            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
                            let _ = writeln!(std::fs::OpenOptions::new().create(true).append(true).open(&log_path).unwrap(), "[{}] backend iniciado: {}", ts, path.display());
                        }
                        Err(e) => {
                            eprintln!("[hipot] no se pudo arrancar el backend: {e}");
                            use std::io::Write;
                            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs().to_string()).unwrap_or_default();
                            let _ = writeln!(std::fs::OpenOptions::new().create(true).append(true).open(&log_path).unwrap(), "[{}] error arrancando backend: {}", ts, e);
                        }
                    }
                }
                None => {
                    eprintln!(
                        "[hipot] backend no encontrado; arráncalo manual: uvicorn App.python.main:app"
                    );
                    // Escribir en log
                    use std::fs::{OpenOptions, create_dir_all};
                    let log_dir = std::env::var("APPDATA")
                        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
                        .unwrap_or_else(|_| std::path::PathBuf::from("."));
                    let _ = create_dir_all(&log_dir);
                    use std::io::Write;
                    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_dir.join("backend.log")) {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs().to_string())
                            .unwrap_or_default();
                        let _ = writeln!(f, "[{}] backend no encontrado en ninguna ruta", ts);
                    }
                }
            }
            // Splash -> main: temporizado en Rust para que nunca se atore.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(2600));
                if let Some(main) = handle.get_webview_window("main") {
                    let _ = main.show();
                    let _ = main.set_focus();
                }
                if let Some(splash) = handle.get_webview_window("splash") {
                    let _ = splash.close();
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::Destroyed) {
                kill_backend(&window.state::<Backend>());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running HiPotApp");
}
