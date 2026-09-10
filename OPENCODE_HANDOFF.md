# HiPotApp — Estado de Desarrollo y Handoff para Agentes (OpenCode / Antigravity)

**Última actualización:** 10 de Septiembre, 2026  
**Rama principal:** `main`  
**Último commit:** `01cf8da` — `feat: implementacion completa backend FastAPI y frontend React Tauri v2`

---

## 📌 Resumen del Estado del Proyecto

El proyecto **HiPotApp** ha pasado de la fase de diseño a la **fase de implementación completa**. Tanto el backend en Python FastAPI como el frontend en React con Tauri v2 están construidos, probados y verificados.

---

## 🏗️ Estructura del Código Implementado

### 1. Backend Python (`App/python/`)
* **Entry point para PyInstaller:** `backend_entry.py` (puerto 8000)
* **Entry point app:** `App/python/main.py` (lifespan, CORS, inicialización de semillas en BD)
* **Modelos ORM (SQLAlchemy):** `App/python/models.py` (17 tablas según `docs/04-base-datos.md`)
* **Base de Datos Dual:** `App/python/database.py` (SQLite/SQL Express local + conexión remota SQL Server)
* **Gestión de PLC:** `App/python/plc_manager.py` (Snap7 para Siemens S7-1200)
* **Gestión de Hipot:** `App/python/hipot_manager.py` (pyserial RS-232 para Hipot 3865 con modo simulación integrado)
* **Scanner / Barcode:** `App/python/barcode_manager.py` (Lector USB HID con normalización en mayúsculas)
* **Impresión Zebra:** `App/python/label_generator.py` y `label_printer.py` (ZPL II)
* **Configuración:** `App/python/config_manager.py` (Import/Export JSON con contraseñas cifradas)
* **Autenticación:** `App/python/auth.py` (Bcrypt)
* **Logging:** `App/python/logger.py` (Persistencia en `AppLogs` y consola)
* **Sincronización:** `App/python/sync_manager.py` (Local -> Remoto)
* **Rutas API:** `App/python/routes/api.py`

### 2. Frontend React + Tauri v2 (`tauri/`)
* **Tauri Config:** `tauri/src-tauri/tauri.conf.json` (Ventana principal 1280x800, Splash Screen 520x560, NSIS Bundle, Updater)
* **Router principal:** `tauri/src/App.tsx`
* **Vistas principales (`tauri/src/pages/`):**
  * `HomeScreen.tsx`: Pantalla principal del operador
  * `TestScreen.tsx`: Monitoreo en vivo de prueba Hipot/PLC
  * `ResultScreen.tsx`: Pantalla de resultado BUENO / MALO
  * `AdminPanel.tsx`: Panel de administración (8 pestañas + import/export)
  * `ManualScreen.tsx`: Operación manual / diagnósticos
* **Componentes clave (`tauri/src/components/`):**
  * `LabelEditor.tsx`: Editor drag & drop ZPL (react-konva)
  * `Gauge.tsx`, `CountUp.tsx`, `BarcodeInput.tsx`, etc.
* **Hooks de comunicación:** `usePLC.ts`, `useHipot.ts`, `useUpdater.ts`

### 3. Pruebas Unitarias (`App/tests/`)
* `test_backend.py`: Cubre normalización de barcode, procesamiento de escaneo, sustitución e impresión ZPL, hashing/verificación bcrypt, simulación Hipot y exportación/importación de configuración.

---

## 🧪 Comandos para Verificación y Desarrollo

| Acción | Comando | Notas |
|---|---|---|
| **Ejecutar Tests Backend** | `python -m pytest App/tests` | Debe pasar 6/6 tests sin errores. |
| **Iniciar Backend Dev** | `python backend_entry.py` | Corre servidor FastAPI en `http://127.0.0.1:8000/`. |
| **Verificar Health API** | `curl http://127.0.0.1:8000/` | Retorna `{"message":"HiPotApp API running","version":"1.0.0"}`. |
| **Compilar Frontend React** | `cd tauri && npm run build` | Ejecuta `tsc --noEmit && vite build`. |
| **Modo Dev Tauri** | `cd tauri && npm run tauri dev` | Inicia la app Tauri de escritorio. |

---

## 📋 Tareas Pendientes / Roadmap Futuro

1. **Pruebas en Hardware Real:**
   * Probar comunicación Snap7 con PLC Siemens S7-1200 en red IP real.
   * Probar comunicación RS-232 con Hipot 3865 real vía puerto COM.
   * Probar impresión en impresora Zebra ZT231 por USB.
2. **Auto-Updater Clave Pública:**
   * Generar par de llaves Tauri (`tauri signer generate`) y colocar la clave pública en `tauri/src-tauri/tauri.conf.json` -> `plugins.updater.pubkey`.
3. **Build de Producción NSIS:**
   * Ejecutar `npm run tauri build` para generar el instalador `.exe` en `tauri/src-tauri/target/release/bundle/nsis/`.
