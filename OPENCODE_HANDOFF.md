# HiPotApp — Estado de Desarrollo y Handoff para Agentes (OpenCode / Antigravity)

**Última actualización:** 10 de Septiembre, 2026  
**Rama principal:** `main`  
**Últimos commits:**  
- `ac554a1` — `feat: implementacion soporte plc siemens s7-1200 red y lectura/escritura por renglon`  
- `9f1d4b9` — `feat: update recipe table layout and remove manual plc and print reprint tabs`

---

## 📌 Resumen del Estado del Proyecto

El proyecto **HiPotApp** cuenta con:
1. **Frontend React + Tauri v2:** 
   - Matriz de recetas actualizada (primera columna es voltaje entero; min/max y checkbox activado para fan, door, frame, drain, defrost, length, total; fecha/hora y usuario de guardado/modificación).
   - Pestaña PLC en Configuración con IP/Rack/Slot del PLC, lectura en tiempo real y **escritura interactiva por renglón con campo de valor y botón "Escribir"**.
2. **Backend Python FastAPI:** 
   - Soporte completo para Siemens S7-1200 a través de Snap7 (lectura/escritura `REAL`, `WORD`/`INT`, `BOOL`).
   - Liberación automática de puerto 8000 en inicio para prevenir errores `[Errno 10048]`.
   - Empaquetado como ejecutable único (`hipot-backend-x86_64-pc-windows-msvc.exe`) integrado como sidecar en Tauri.
3. **Ejecutable Único:**
   - La aplicación compila a un archivo `hipotapp.exe` en `tauri/src-tauri/target/release/` que al ejecutarse inicia automáticamente la ventana Tauri y su backend Python sidecar.

---

## 🏗️ Estructura del Código Implementado

### 1. Backend Python (`App/python/`)
* **Entry point para PyInstaller:** `backend_entry.py` (puerto 8000)
* **Entry point app:** `App/python/main.py` (lifespan, CORS, inicialización de semillas en BD)
* **Modelos ORM (SQLAlchemy):** `App/python/models.py` (17 tablas según `docs/04-base-datos.md`)
* **Gestión de PLC Siemens:** `App/python/plc_manager.py` (Snap7 para Siemens S7-1200 con `read_tag`, `write_tag` y `update_config`)
* **Rutas API PLC:** `App/python/routes/api.py` (`/admin/plc-config`, `/admin/plc-tags`, `/admin/plc-write`)

### 2. Frontend React + Tauri v2 (`tauri/`)
* **Pestaña PLC (`AdminPanel.tsx`):**
  * Configuración de Red (IP, Rack, Slot).
  * Tabla de variables en tiempo real con refresco automático (2.5s).
  * Entrada de valor y botón **"Escribir"** en cada fila.
* **Recetas (`AdminPanel.tsx`):**
  * Matriz de pruebas completa con límites min/max y checkboxes de activación.
  * Audit trail con fecha, hora y usuario que modificó la receta.

---

## 🧪 Comandos para Verificación y Desarrollo

| Acción | Comando | Notas |
|---|---|---|
| **Ejecutar Tests Backend** | `python -m pytest App/tests` | Debe pasar 6/6 tests sin errores. |
| **Compilar Backend `.exe`** | `python -m PyInstaller --onefile --noconfirm --name hipot-backend-x86_64-pc-windows-msvc backend_entry.py` | Genera binario sidecar en `dist/`. |
| **Copiar Sidecar a Tauri** | `Copy-Item 'dist/hipot-backend-x86_64-pc-windows-msvc.exe' 'tauri/binaries/hipot-backend-x86_64-pc-windows-msvc.exe'` | Requerido antes de `cargo build`. |
| **Compilar Aplicación Release** | `cd tauri && npm run build && cargo build --manifest-path src-tauri/Cargo.toml --release` | Genera `hipotapp.exe` independiente. |
| **Lanzar Aplicación** | `cmd.exe /c start "" "D:\Antigravity\HiPot\tauri\src-tauri\target\release\hipotapp.exe"` | Inicia el frontend y backend juntos. |

---

## 📋 Tareas Pendientes / Roadmap Futuro

1. **Pruebas en Hardware Real Siemens S7-1200:**
   - Asegurar que en TIA Portal esté activado **"Permitir acceso vía PUT/GET"** en las propiedades de la CPU.
   - Asegurar que el Bloque de Datos (DB) no tenga **"Optimized Block Access"** activado.
2. **Build de Instalador NSIS (Cuando el usuario lo solicite):**
   - Ejecutar `npm run tauri build` para empaquetar el instalador `.exe` final.
