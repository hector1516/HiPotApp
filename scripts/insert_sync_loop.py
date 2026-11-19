p = 'tauri/src-tauri/src/main.rs'
s = open(p, encoding='utf-8').read()
marker = '            // Splash -> main: temporizado en Rust para que nunca se atore.'
loop_code = '''            // Auto-sync loop: cada 5 minutos sincronizar pendientes a SQL Server remoto
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(300));
                    let cfg = get_config();
                    let server = cfg.get("sql_server").and_then(|v| v.as_str()).unwrap_or("");
                    if !server.is_empty() {
                        let db = cfg.get("sql_database").and_then(|v| v.as_str()).unwrap_or("HiPotDB_Central");
                        let user = cfg.get("sql_user").and_then(|v| v.as_str()).unwrap_or("");
                        let pwd = cfg.get("sql_password").and_then(|v| v.as_str()).unwrap_or("");
                        let driver = "ODBC Driver 17 for SQL Server";
                        let conn_str = if user.is_empty() {
                            format!("Driver={{{}}};Server={};Database={};Trusted_Connection=yes;", driver, server, db)
                        } else {
                            format!("Driver={{{}}};Server={};Database={};Uid={};Pwd={};", driver, server, db, user, pwd)
                        };
                        let _ = sync_pending(conn_str);
                    }
                }
            });

'''
s = s.replace(marker, loop_code + marker, 1)
open(p, 'w', encoding='utf-8').write(s)
print("done")