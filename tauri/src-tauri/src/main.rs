// HiPotApp: ventana Tauri + arranque/parada automática del backend Python
// empaquetado como recurso (todo-en-uno, offline-first).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{Manager, State, WindowEvent};

const BACKEND_BIN: &str = "hipot-backend-x86_64-pc-windows-msvc.exe";

#[tauri::command]
fn read_backend_log() -> String {
    let log_dir = std::env::var("APPDATA")
        .map(|d| std::path::PathBuf::from(d).join("HiPotApp"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_path = log_dir.join("backend.log");
    std::fs::read_to_string(&log_path).unwrap_or_else(|_| "No hay log disponible.".to_string())
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
        .invoke_handler(tauri::generate_handler![open_external, read_backend_log])
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
