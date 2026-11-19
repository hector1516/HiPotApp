# Plan de migración completa a Rust — HiPotApp en un solo `.exe`

> Fecha: 2026-11-18
> Estado de partida: `main` @ `6bec4c5` + corrección de registro de comandos.
> Objetivo: **un solo `HiPotApp.exe`**, sin Python, sin sidecar, sin puertos HTTP, con BD local SQL Express y sync a SQL Server remoto.

---

## 1. Resumen ejecutivo

La migración está **aproximadamente al 55%**. La UI ya usa `invoke` para una parte importante (recetas, usuarios parciales, ejecuciones, logs, impresión, sync manual), pero todavía quedan **8 archivos del frontend con ~58 llamadas HTTP** y **~25 endpoints de Python sin equivalente en Rust**.

Además se detectaron **3 problemas estructurales** que hoy rompen funcionalidades aunque el código "exista":

1. **Comandos Rust no registrados**: `list_recipes`, `get_recipe_by_model`, `create_recipe`, `update_recipe`, `delete_recipe`, `duplicate_recipe`, `print_test_zebra`, `reprint_zebra`, `sync_pending`, `test_odbc_connection`, etc. no estaban en `generate_handler!`. Al no estar registrados, `invoke()` falla con "command not found". **Ya corregido en esta sesión** (ver §4).
2. **Dos esquemas de BD distintos**: Rust usa la tabla `configuration` (singular) y Python usa `configurations` (plural). Rust además crea tablas con DDL repetido en cada comando. Si mañana se apunta a SQL Express, los datos no se verán entre sí.
3. **Dos rutas de BD distintas**: Rust usa `%APPDATA%\HiPotApp\hipot_local.db`; Python usa el directorio del `.exe` / `App/python`. La app puede estar leyendo una BD y el sidecar otra.

Mientras esos tres puntos no se resuelvan, cualquier migración adicional se construye sobre arena.

---

## 2. Arquitectura actual

```
┌──────────────────────────────────────────────────────────────┐
│ HiPotApp.exe (Tauri 2)                                       │
│                                                              │
│  React UI ── invoke() ──► 37 comandos Rust                   │
│      │                        ├─ rusqlite (SQLite local)     │
│      │                        ├─ s7 (PLC S7-1200)            │
│      │                        ├─ odbc-api (SQL remoto)       │
│      │                        └─ TCP 9100 / print (Zebra)    │
│      │                                                       │
│      └── fetch() ──► http://127.0.0.1:8765 ──► sidecar .exe  │
│                        FastAPI + SQLAlchemy                  │
│                        ├─ SQL Express / SQLite               │
│                        ├─ snap7 (PLC)                        │
│                        ├─ pyserial (Hipot 3865)              │
│                        ├─ pyodbc (SQL Server remoto)         │
│                        └─ bcrypt / JWT                       │
└──────────────────────────────────────────────────────────────┘
```

**Problema:** el sidecar es un segundo `.exe` de ~40–60 MB, abre el puerto 8765, duplica lógica y es la causa del indicador **"Sin conexión"** cuando no arranca.

---

## 3. Inventario real (verificado en código)

### 3.1 Comandos Rust existentes (37)

| Área | Comandos |
|---|---|
| Config / sistema | `get_config`, `set_config`, `read_backend_log`, `restart_backend`, `open_external`, `show_touch_keyboard` |
| PLC | `plc_status`, `plc_read_tag`, `plc_write_tag` |
| Recetas | `list_recipes`, `get_recipe_by_model`, `create_recipe`, `update_recipe`, `delete_recipe`, `duplicate_recipe` |
| Operadores / usuarios | `list_operators`, `add_operator`, `list_users`, `add_user`, `delete_user` |
| Ejecuciones | `start_execution`, `save_plc_result`, `save_hipot_result`, `list_executions` |
| BD remota / sync | `test_odbc_connection`, `execute_sql`, `ensure_remote_tables`, `sync_pending` |
| Impresión | `canvas_to_zpl`, `print_zpl_ethernet`, `print_zpl_windows`, `print_test_zebra`, `reprint_zebra` |
| Logs | `list_logs`, `log_action` |

### 3.2 Endpoints Python (56) y su estado

| Grupo | Endpoints | Estado Rust |
|---|---|---|
| Barcode | `POST /barcode/scan` | ❌ Falta |
| Recetas | `GET/POST/PUT/DELETE /recipes`, `/recipes/model/{m}`, `/recipes/{id}/duplicate` | ✅ Completo |
| Operadores | `GET/POST /operators` | ✅ Completo |
| Auth | `POST /auth/login`, `POST /auth/register` | ⚠️ Registro sí, login no |
| Usuarios | `GET/PUT/DELETE /users`, `GET /users/{id}` | ⚠️ Falta update y get permisos |
| Ejecuciones | start, plc-result, hipot-result, list | ✅ Completo |
| Estado | `GET /status/plc`, `/status/hipot`, `/health` | ⚠️ PLC sí; Hipot y health no |
| Config | `GET/POST /admin/config`, `POST /admin/sync-now` | ⚠️ Config sí; sync-now no |
| Logs | `GET /admin/logs` | ✅ Completo |
| Impresión | `POST /admin/print-test`, `POST /executions/{id}/reprint` | ✅ Completo |
| Plantillas | `GET/POST/PUT/DELETE /admin/templates`, `GET /admin/templates/{id}` | ❌ Falta |
| Import/Export | `GET /admin/export`, `POST /admin/import`, `GET /admin/import-export-log` | ❌ Falta |
| PLC admin | `GET/POST /admin/plc-config`, `GET/POST/DELETE /admin/plc-tags`, `POST /admin/plc-read`, `POST /admin/plc-write` | ❌ Falta |
| Hipot admin | `POST /admin/hipot-cmd` | ❌ Falta |
| Sync | `GET /admin/sync-log` | ❌ Falta |
| Historial | `GET /history` | ❌ Falta |
| Manual | `GET/POST /manual/mode`, `POST /manual/plc-test`, `POST /manual/hipot`, `POST /manual/abort` | ❌ Falta |
| PLC I/O | `GET /admin/plc-io-status`, `POST /admin/plc-io-write` | ❌ Falta |
| BD local | `GET/POST /admin/local-db-config` | ❌ Falta |
| Auditoría | `GET /audit/tables-status`, `POST /audit/ensure-table`, `GET /audit/table-names` | ❌ Falta |

### 3.3 Llamadas HTTP restantes en el frontend (~58)

| Archivo | Llamadas |
|---|---|
| `tauri/src/pages/AdminPanel.tsx` | `/users` (fallback), `PUT /users/{id}`, `GET /users/{id}`, `/recipes` (fallback), `DELETE /recipes/{id}`, `/admin/plc-config`, `/admin/plc-tags`, `/admin/plc-write`, `/admin/plc-tags/{id}`, `/status/plc`, `/admin/plc-tags` (POST), `/status/hipot`, `/admin/hipot-cmd`, `/admin/config`, `/admin/sync-log`, `/health`, `/admin/templates*`, `/admin/import-export-log`, `/admin/local-db-config`, `/audit/*`, `/admin/export`, `/admin/import`, `/admin/plc-io-status`, `/admin/plc-io-write`, `/manual/hipot` |
| `tauri/src/pages/ManualScreen.tsx` | `/manual/mode` (x2), `/recipes/model/{m}`, `/manual/plc-test` (x2), `/manual/hipot`, `/manual/abort` |
| `tauri/src/App.tsx` | `/admin/config` (x2), `/health`, `/executions/{id}/reprint`, `/admin/print-test`, `/barcode/scan` (x3) |
| `tauri/src/components/ConfigGatePopup.tsx` | `/auth/login` |
| `tauri/src/components/HistoryPopup.tsx` | `/history` |
| `tauri/src/components/LogPopup.tsx` | `/admin/logs` |
| `tauri/src/components/ReprintPopup.tsx` | `/history` |
| `tauri/src/hooks/useHipot.ts` | `/status/hipot` |
| `tauri/src/api.ts` | `/barcode/scan`, `/recipes/model/{m}` |

### 3.4 Tablas (18)

`recipes`, `recipe_components`, `recipe_totals`, `operators`, `batches`, `executions`, `plc_readings`, `hipot_results`, `barcode_logs`, `configurations`, `users`, `sync_log`, `app_logs`, `ui_state_log`, `label_templates`, `label_prints`, `plc_tags`, `import_export_logs`.

El esquema válido para SQL Server está en `tauri/src-tauri/schema.sql`. `docs/schema_mssql.sql` es solo un stub de 1.1 KB y debe regenerarse o eliminarse.

---

## 4. Hallazgos críticos (P0)

### P0-1 — Comandos no registrados (CORREGIDO)
`main.rs` registraba solo 18 comandos. Faltaban 19, incluidos todos los de recetas, impresión y sync. Se amplió `generate_handler!` a los 37 comandos y `cargo build` compila.

### P0-2 — Nombre de tabla de configuración inconsistente
- Rust: `configuration` (`main.rs` líneas 27, 45).
- Python/SQLAlchemy: `configurations` (`models.py` línea 130).
- `schema.sql`: `configurations`.

**Acción:** unificar en `configurations` y crear un módulo único de acceso a datos.

### P0-3 — Ruta de BD local inconsistente
- Rust: `%APPDATA%\HiPotApp\hipot_local.db`.
- Python: junto al `.exe` o `App/python/hipot_local.db`.
- `db_config.json` solo lo lee Python.

**Acción:** una sola ruta administrada por Rust (`%APPDATA%\HiPotApp\hipot_local.db`), y `db_config.json` leído por Rust.

### P0-4 — DDL duplicado
Cada comando Rust hace `CREATE TABLE IF NOT EXISTS` de sus tablas. Hay ~20 copias del mismo DDL. Si se cambia una columna, hay que cambiarla en 20 sitios.

**Acción:** un módulo `db/schema.rs` con una sola función `ensure_schema(conn)`.

### P0-5 — `delete_recipe` migrado pero no usado
`AdminPanel.tsx` línea 169 sigue usando `api.del('/recipes/{id}')`.

### P0-6 — `add_user` guarda password vacío
`AdminPanel.tsx` llama `add_user` con `password: ''`. Hoy el login no usa password, pero si se activa bcrypt hay que corregir el flujo.

### P0-7 — Sync incompleto
`sync_pending` solo copia `executions` y no escribe `sync_log` ni `synced_at`. Python sincroniza 9 tablas (`SyncManager.SYNC_TABLES`).

### P0-8 — Health basado en HTTP
`App.tsx` marca "Sin conexión" si `GET /health` falla. Al eliminar el sidecar, el indicador debe pasar a un comando Rust `app_health`.

---

## 5. Arquitectura objetivo

```
┌──────────────────────────────────────────────────────────────┐
│ HiPotApp.exe (Tauri 2, un solo binario)                      │
│                                                              │
│  React UI (dist embebido)                                    │
│      │ invoke()                                              │
│      ▼                                                       │
│  Rust core                                                   │
│   ├─ db/          → ODBC → SQL Express local                 │
│   ├─ plc/         → s7    → S7-1200 (Q / I / IA, DB1)        │
│   ├─ hipot/       → serialport → Hipot 3865 RS-232           │
│   ├─ printer/     → TCP 9100 / spooler Windows → ZT231       │
│   ├─ sync/        → ODBC → SQL Server remoto                 │
│   ├─ logging/     → archivo rotativo + tabla app_logs        │
│   └─ config/      → tabla configurations + db_config.json     │
│                                                              │
│  Updater (GitHub Releases)                                   │
└──────────────────────────────────────────────────────────────┘
```

**Sin Python. Sin `127.0.0.1:8765`. Sin sidecar. Sin puertos.**

### Decisión de BD local: SQL Express

Requisito del usuario: **solo SQL Express**, no SQLite.

Implicaciones que deben quedar explícitas:

- SQL Express debe estar instalado en cada estación (servicio `MSSQL$SQLEXPRESS`).
- Se requiere **ODBC Driver 17/18 for SQL Server**.
- El instalador NSIS debe: detectar SQL Express, avisar si falta y, opcionalmente, instalarlo (requiere admin y ~250 MB).
- Alternativa recomendada si se quiere evitar el servicio: **SQL Server Express LocalDB**, que es parte de Express pero arranca bajo demanda. Mismo motor, menos fricción para una estación única.
- La cadena de conexión por defecto: `Server=.\SQLEXPRESS;Database=HiPotDB;Trusted_Connection=yes;`.
- Se necesita una **migración única** de `hipot_local.db` (SQLite) a SQL Express en el primer arranque.

**Recomendación de ingeniería:** implementar una capa `Repository` con dos backends (`SqlExpressRepo` para producción y `SqliteRepo` solo para tests/dev). Así se puede migrar sin romper la app y se pueden correr tests sin SQL Express. En producción el binario usa solo `SqlExpressRepo`.

---

## 6. Roadmap por fases

> Orden pensado para **no reescribir dos veces**. Primero se estabiliza la base de datos, luego se migran las features sobre esa base, y al final se elimina el sidecar.

### Fase 0 — Fundaciones y correcciones P0 (1–2 sesiones)

- [x] Registrar los 37 comandos en `generate_handler!`.
- [ ] Crear módulo `db/`:
  - [ ] `db/mod.rs`: ruta única `%APPDATA%\HiPotApp\hipot_local.db` + `db_config.json`.
  - [ ] `db/schema.rs`: un solo `ensure_schema()` con el DDL de `schema.sql`.
  - [ ] `db/config.rs`: `get_config` / `set_config` sobre `configurations`.
  - [ ] `db/seed.rs`: valores por defecto (`DEFAULT_CONFIGS`), usuario `ADMIN`, plantilla por defecto.
- [ ] Eliminar los ~20 `CREATE TABLE IF NOT EXISTS` dispersos.
- [ ] Unificar `configuration` → `configurations`.
- [ ] Corregir `AdminPanel.tsx` para usar `invoke('delete_recipe')`.
- [ ] Corregir `add_user` para recibir password real.
- [ ] Regenerar `docs/schema_mssql.sql` desde `schema.sql`.

**Criterio de aceptación:** `cargo test` y la app arrancan con una BD vacía, crean las 18 tablas, siembran configuración y el panel de admin lee la misma BD que los comandos.

---

### Fase 1 — Salud, estado y fin del "Sin conexión" (1 sesión)

Comandos nuevos:

| Comando | Reemplaza |
|---|---|
| `app_health` | `GET /health` |
| `plc_status` (ya existe) | `GET /status/plc` |
| `hipot_status` | `GET /status/hipot` |
| `app_info` | versión, rutas, uptime, estado BD |

Cambios frontend:

- `App.tsx`: health check con `invoke('app_health')` cada 3 s.
- `useHipot.ts`: `invoke('hipot_status')`.
- Indicadores de PLC/Hipot/BD con estados `ok / warn / error` y detalle.

**Criterio de aceptación:** con el sidecar apagado, la app muestra "Sistema OK" (o el error real de hardware), no "Sin conexión".

---

### Fase 2 — PLC completo en Rust (2–3 sesiones)

Portar `App/python/plc_manager.py` a `plc/`:

- Offsets: `OFF_MODE=110`, `OFF_VOLTAGE`, `OFF_START`, `OFF_STOP`, `OFF_COMPLETE`, `OFF_PASS`, `OFF_RUNNING`, `RES_OFFSETS`, `COMP_OFFSETS`.
- Mapas I/O: `_Q_MAP`, `_I_MAP`, `_IA_MAP`.
- Funciones: `read_digital_output`, `read_digital_input`, `read_analog_input`, `write_digital_output`, `send_recipe`, `start_test`, `stop_test`, `read_results`.

Comandos nuevos:

| Comando | Reemplaza |
|---|---|
| `plc_config_get` / `plc_config_set` | `GET/POST /admin/plc-config` |
| `plc_tags_list` / `plc_tags_add` / `plc_tags_delete` | `/admin/plc-tags` |
| `plc_tag_read_by_id` / `plc_tag_write_by_id` | `/admin/plc-read`, `/admin/plc-write` |
| `plc_io_status` | `GET /admin/plc-io-status` |
| `plc_io_write` | `POST /admin/plc-io-write` |
| `plc_send_recipe` / `plc_start` / `plc_stop` / `plc_read_results` | lógica interna de `/executions` |

Cambios frontend:

- `AdminPanel.tsx` → `PlcTab`, `PlcIoTab`, `ManualTab`.
- `TestScreen.tsx` si usa PLC.

**Criterio de aceptación:** lectura/escritura de Q/I/IA y DB1 contra PLC real; simulación si no hay PLC.

---

### Fase 3 — Hipot 3865 en Rust (2 sesiones)

Portar `hipot_manager.py` con `serialport`:

- Puerto, baudios, paridad, databits, stopbits desde configuración.
- Comandos: `STATUS?`, `START`, `STOP`, `ABORT`, `VOLTAGE:`, `CURRENT_MAX:`.
- Parseo de mediciones a `{voltage, current, resistance, insulation_resistance, pass}`.

Comandos nuevos:

| Comando | Reemplaza |
|---|---|
| `serial_list_ports` | `HipotManager.list_ports()` |
| `hipot_status` | `GET /status/hipot` |
| `hipot_command` | `POST /admin/hipot-cmd` |
| `hipot_test` | `POST /manual/hipot` |
| `hipot_abort` | `POST /manual/abort` |

Cambios frontend:

- `useHipot.ts`, `ManualScreen.tsx`, `HipotTab` de `AdminPanel.tsx`.

**Criterio de aceptación:** handshake RS-232 con el 3865 real y modo simulado.

---

### Fase 4 — Auth y usuarios completos (1 sesión)

Comandos:

| Comando | Reemplaza |
|---|---|
| `auth_login` | `POST /auth/login` |
| `user_get` | `GET /users/{id}` |
| `user_update` | `PUT /users/{id}` |
| `auth_verify` (opcional) | validación de token/rol |

Notas:

- Hoy el login es **sin contraseña** (solo gafete/batch). Mantener ese comportamiento; si se activa bcrypt, usar el crate `bcrypt` y migrar `password_hash`.
- `ConfigGatePopup.tsx` y `UsersTab` deben usar `invoke`.

**Criterio de aceptación:** entrar al panel con `ADMIN`/gafete sin HTTP.

---

### Fase 5 — Plantillas de etiqueta e import/export (2 sesiones)

Comandos:

| Comando | Reemplaza |
|---|---|
| `template_list` | `GET /admin/templates` |
| `template_get` | `GET /admin/templates/{id}` |
| `template_create` | `POST /admin/templates` |
| `template_update` | `PUT /admin/templates/{id}` |
| `template_delete` | `DELETE /admin/templates/{id}` |
| `config_export` | `GET /admin/export` |
| `config_import` | `POST /admin/import` |
| `import_export_log_list` | `GET /admin/import-export-log` |

Cambios frontend:

- `AdminPanel.tsx` → `LabelsTab`, `SystemTab` (import/export).
- `LabelEditor.tsx` si aplica.

**Criterio de aceptación:** CRUD de plantillas, exportar/importar configuración y registrar la operación.

---

### Fase 6 — Historial, logs y auditoría (1–2 sesiones)

Comandos:

| Comando | Reemplaza |
|---|---|
| `history_list` (filtros `model`, `serial`, `start`, `end`, `limit`) | `GET /history` |
| `log_list` (ya existe) | `GET /admin/logs` |
| `local_db_config_get` / `local_db_config_set` | `GET/POST /admin/local-db-config` |
| `audit_tables_status` | `GET /audit/tables-status` |
| `audit_ensure_tables` | `POST /audit/ensure-table` |
| `audit_table_names` | `GET /audit/table-names` |
| `sync_log_list` | `GET /admin/sync-log` |

Cambios frontend:

- `HistoryPopup.tsx`, `LogPopup.tsx`, `ReprintPopup.tsx`, `SystemTab`.

**Criterio de aceptación:** historial con filtros y reimpresión sin HTTP.

---

### Fase 7 — BD local SQL Express + sync multitable (3–4 sesiones)

Esta es la fase más grande y la que cumple el requisito "solo SQL Express".

- [ ] Implementar `db/odbc.rs` con `odbc-api`:
  - [ ] Conexión con `Trusted_Connection` o usuario/password.
  - [ ] Pool simple (Mutex<Connection>) o reconexión por comando.
  - [ ] Timeouts y mensajes de error claros.
- [ ] Implementar repositorios para las 18 tablas.
- [ ] Migrar todos los comandos CRUD de `rusqlite` a ODBC.
- [ ] `ensure_local_schema()` al arranque.
- [ ] Migración única SQLite → SQL Express (leer `hipot_local.db`, insertar, marcar flag).
- [ ] `db_config.json` administrado por Rust.
- [ ] Sync multitable: `executions`, `plc_readings`, `hipot_results`, `barcode_logs`, `recipes`, `operators`, `batches`, `users`, `label_prints`.
- [ ] Escribir `sync_log` (dirección, tabla, registros, estado, error).
- [ ] Actualizar `synced_at`.
- [ ] Reintentos y modo offline (si remoto no responde, no bloquear la UI).
- [ ] `sync_now` manual y loop cada `sync_interval_minutes`.

**Criterio de aceptación:** con SQL Express local vacío, la app crea el esquema, opera 100% offline y sincroniza a SQL Server remoto.

---

### Fase 8 — Barcode y flujo manual (1 sesión)

Comandos:

| Comando | Reemplaza |
|---|---|
| `barcode_scan` | `POST /barcode/scan` |
| `manual_mode_set` | `POST /manual/mode` |
| `manual_plc_test` | `POST /manual/plc-test` |
| `manual_hipot` | `POST /manual/hipot` |
| `manual_abort` | `POST /manual/abort` |

Cambios frontend:

- `App.tsx`: reemplazar `api.scan(...)` por `invoke('barcode_scan', ...)`.
- `ManualScreen.tsx`: todos los endpoints.

**Criterio de aceptación:** flujo model → serial → batch → prueba → resultado sin HTTP.

---

### Fase 9 — Eliminar el sidecar Python (1 sesión)

- [ ] `main.rs`: eliminar `Backend`, `backend_path`, `kill_backend`, arranque en `setup`, `restart_backend`, `read_backend_log`.
- [ ] Crear `app_log_read` para el log propio en `%APPDATA%\HiPotApp\app.log`.
- [ ] `tauri.conf.json`: quitar `resources` del backend.
- [ ] `.github/workflows/build.yml`: quitar pasos de Python/PyInstaller y `Copy backend sidecar`.
- [ ] Borrar `backend_entry.py`, `App/python/sidecar.py` y el árbol `App/python` (mover a rama `legacy-python` o zip en `docs/legacy/`).
- [ ] Borrar `tauri/src/api.ts` y todas las referencias.
- [ ] Quitar `VITE_API_URL` y el proxy de `vite.config.ts`.
- [ ] Verificar que `dist` no contiene `8765` ni `fetch(` al backend.

**Criterio de aceptación:** `npm run tauri build` genera un instalador que funciona en una VM **sin Python instalado**.

---

### Fase 10 — Hardening y release (1–2 sesiones)

- [ ] Logging de UI: visor con fecha/hora, categoría, parámetros, valores enviados/recibidos, duración y error.
- [ ] Rotación de logs (tamaño/fecha).
- [ ] Manejo de errores global (error boundary + toast).
- [ ] Instancia única (evitar dos apps contra el mismo PLC).
- [ ] Splash con timeout y mensaje si falla el arranque.
- [ ] Confirmar maximizado, auto-inicio y teclado virtual.
- [ ] Pruebas en VM limpia: instalación, SQL Express, ODBC, PLC, Hipot, Zebra.
- [ ] Prueba de auto-update desde GitHub Releases.
- [ ] Actualizar versión y `latest.json`.
- [ ] Release con notas.

**Criterio de aceptación:** instalación desde cero en estación limpia + actualización desde la versión anterior.

---

## 7. Orden recomendado y esfuerzo

| Fase | Depende de | Esfuerzo | Prioridad |
|---|---|---|---|
| 0. Fundaciones | — | 1–2 sesiones | P0 |
| 1. Salud/estado | 0 | 1 sesión | P0 |
| 2. PLC completo | 0 | 2–3 sesiones | P1 |
| 3. Hipot | 0 | 2 sesiones | P1 |
| 4. Auth/usuarios | 0 | 1 sesión | P1 |
| 5. Plantillas/import | 0 | 2 sesiones | P1 |
| 6. Historial/auditoría | 0 | 1–2 sesiones | P1 |
| 7. SQL Express + sync | 0, 2–6 | 3–4 sesiones | P0 |
| 8. Barcode/manual | 0, 2, 3 | 1 sesión | P1 |
| 9. Quitar sidecar | 1–8 | 1 sesión | P0 |
| 10. Hardening/release | 9 | 1–2 sesiones | P0 |

**Total estimado: 16–24 sesiones de trabajo.** No es un cambio de una tarde; es una migración de producto.

**Atajo posible:** si se acepta mantener SQLite local (no cumpliría "solo SQL Express"), las fases 0–6 y 8–10 se reducen a ~10–12 sesiones. Pero el requisito actual es SQL Express.

---

## 8. Mapa de archivos a tocar

| Archivo | Acción |
|---|---|
| `tauri/src-tauri/src/main.rs` | Dividir en módulos: `db/`, `plc/`, `hipot/`, `printer/`, `sync/`, `auth/`, `commands/`. Hoy son ~1,260 líneas. |
| `tauri/src-tauri/schema.sql` | Fuente única del esquema. |
| `tauri/src-tauri/Cargo.toml` | Agregar `bcrypt`, `once_cell`, `parking_lot` (opcional), `log`/`tracing` (opcional). |
| `tauri/src-tauri/tauri.conf.json` | Quitar resources del backend; revisar bundle. |
| `tauri/src/api.ts` | Eliminar al final de Fase 9. |
| `tauri/src/App.tsx` | Health, scans, reprint, config. |
| `tauri/src/pages/AdminPanel.tsx` | PLC, Hipot, BD, plantillas, auditoría, usuarios. |
| `tauri/src/pages/ManualScreen.tsx` | Modo manual completo. |
| `tauri/src/components/ConfigGatePopup.tsx` | Login. |
| `tauri/src/components/HistoryPopup.tsx` | Historial. |
| `tauri/src/components/LogPopup.tsx` | Logs. |
| `tauri/src/components/ReprintPopup.tsx` | Historial. |
| `tauri/src/hooks/useHipot.ts` | Estado Hipot. |
| `.github/workflows/build.yml` | Quitar build de Python. |
| `backend_entry.py`, `App/python/**` | Eliminar/mover a legacy. |

---

## 9. Estrategia de pruebas

1. **Unitarias (Rust)**: parseo de ZPL, parseo de mediciones Hipot, mapeo de offsets PLC, repositorios con SQLite de test.
2. **Integración (Rust)**: comandos Tauri contra una BD temporal; `ensure_schema` + CRUD + sync simulado.
3. **Hardware-in-the-loop**:
   - PLC S7-1200: leer Q/I/IA, escribir DB1, receta completa.
   - Hipot 3865: `STATUS?`, `START`, `STOP`, `ABORT`.
   - Zebra ZT231: Ethernet 9100 y spooler Windows.
4. **UI**: flujo completo con teclado virtual, maximizado, auto-inicio.
5. **VM limpia**: sin Python, sin Node, solo el instalador.
6. **Actualización**: instalar versión anterior y actualizar a la nueva.

---

## 10. Riesgos

| Riesgo | Impacto | Mitigación |
|---|---|---|
| SQL Express no instalado en estaciones | App no arranca | Detección en instalador + mensaje claro + guía; opción LocalDB. |
| Driver ODBC ausente/versión distinta | Sin conexión BD | Detectar drivers; documentar; incluir en instalador si es posible. |
| Migración SQLite → SQL Express | Pérdida de datos | Script idempotente + backup previo + flag de migración. |
| `s7` crate con timeouts | Bloqueo de UI | Comandos `async`/hilos + timeouts; nunca bloquear el hilo principal. |
| Serial RS-232 timing | Lecturas incompletas | Buffer + timeout + reintentos + modo simulado. |
| Spooler Windows | Etiquetas duplicadas | Cola idempotente + estado por `label_prints`. |
| Eliminar sidecar antes de tiempo | Regresión funcional | Fase 9 solo con matriz de aceptación completa. |
| Tamaño del instalador | Distribución | Medir; SQL Express se instala aparte, no dentro del `.exe`. |

---

## 11. Definición de "terminado"

- [ ] `HiPotApp.exe` no contiene Python ni lanza procesos externos.
- [ ] No existe `http://127.0.0.1:8765` en el bundle.
- [ ] La app funciona en una VM sin Python.
- [ ] BD local = SQL Express; sync = SQL Server remoto.
- [ ] PLC, Hipot y Zebra operan desde Rust.
- [ ] Auto-inicio, maximizado, teclado virtual y auto-update funcionan.
- [ ] Logs con fecha/hora, parámetros, valores y duración visibles en la UI.
- [ ] Instalación limpia + actualización probadas.

---

## 12. Primer paso inmediato

1. Cerrar Fase 0 (módulo `db/`, esquema único, ruta única, seed).
2. Implementar Fase 1 (`app_health`, `hipot_status`) para eliminar el "Sin conexión".
3. Continuar con PLC/Hipot según disponibilidad de hardware.

Con Fase 0 y 1 hechas, la app deja de depender del health HTTP y se puede empezar a probar sin sidecar en modo lectura.
