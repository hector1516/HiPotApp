from pathlib import Path

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

# 1. Import headers
s = s.replace(
    "// HiPotApp: ventana Tauri + arranque/parada automática del backend Python\n"
    "// empaquetado como recurso (todo-en-uno, offline-first).\n",
    "// HiPotApp: aplicación Tauri 2 + núcleo Rust (PLC, Hipot, BD, impresión).\n"
    "// Un solo ejecutable, sin sidecar Python ni puertos HTTP.\n",
)
s = s.replace("use std::process::{Child, Command};\n", "")
s = s.replace("use std::sync::Mutex;\n", "")
s = s.replace("use tauri::{Emitter, Manager, State, WindowEvent};\n", "use tauri::{Emitter, Manager};\n")
s = s.replace("#[cfg(target_os = \"windows\")]\nuse std::os::windows::process::CommandExt;\n\n", "")
s = s.replace('const BACKEND_BIN: &str = "hipot-backend-x86_64-pc-windows-msvc.exe";\n\n', "")

# 2. Remove read_backend_log + restart_backend
start = s.index("#[tauri::command]\nfn read_backend_log()")
end = s.index("#[tauri::command]\nfn plc_status(")
s = s[:start] + s[end:]

# 3. Remove Backend struct + backend_path + kill_backend
start = s.index("struct Backend(Mutex<Option<Child>>);")
end = s.index("fn main() {")
s = s[:start] + s[end:]

# 4. Remove manage(Backend)
s = s.replace("        .manage(Backend(Mutex::new(None)))\n", "")

# 5. Remove handler entries
s = s.replace("            read_backend_log,\n", "")
s = s.replace("            restart_backend,\n", "")

# 6. Remove backend spawn block inside setup
start = s.index("            match backend_path(app.handle()) {")
end = s.index("            // Auto-sync loop:")
s = s[:start] + s[end:]

# 7. Remove on_window_event block
start = s.index("        .on_window_event(|window, event| {")
end = s.index("        .run(tauri::generate_context!())")
s = s[:start] + s[end:]

p.write_text(s, encoding='utf-8')
print('main.rs sidecar removed')
