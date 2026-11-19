p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = 'struct Backend(Mutex<Option<Child>>);'
cmds = '''#[tauri::command]
fn print_zpl_ethernet(ip: String, port: u16, zpl: String) -> String {
    use std::net::TcpStream;
    use std::io::Write;
    match TcpStream::connect((ip.as_str(), port)) {
        Ok(mut stream) => {
            stream.set_write_timeout(Some(std::time::Duration::from_secs(10))).ok();
            match stream.write_all(zpl.as_bytes()) {
                Ok(_) => "OK".to_string(),
                Err(e) => format!("Error enviando: {}", e),
            }
        }
        Err(e) => format!("Error conectando: {}", e),
    }
}

#[tauri::command]
fn print_zpl_windows(printer_name: String, zpl: String) -> String {
    use std::fs;
    use std::process::Command;
    // Write ZPL to temp file
    let temp = std::env::temp_dir().join("hipot_print.zpl");
    if fs::write(&temp, &zpl).is_err() {
        return "Error escribiendo archivo temporal".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        let path_str = temp.to_string_lossy().to_string();
        let result = Command::new("cmd")
            .args(["/C", "print", &format!("/d:{}", printer_name), &path_str])
            .output();
        let _ = fs::remove_file(&temp);
        match result {
            Ok(o) if o.status.success() => "OK".to_string(),
            Ok(o) => format!("Error: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("Error: {}", e),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = temp;
        "No soportado en esta plataforma".to_string()
    }
}

'''
s = s.replace(marker, cmds + marker, 1)
s = s.replace(
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status])",
    ".invoke_handler(tauri::generate_handler![open_external, read_backend_log, get_config, set_config, plc_status, print_zpl_ethernet, print_zpl_windows])"
)
open(p, 'w', encoding='utf-8').write(s)
print("done")