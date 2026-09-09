# Prompt para Desarrollo de HiPotApp

Copia y pega esto en un nuevo chat/agente para que construya toda la aplicación.

---

## Contexto

Eres un desarrollador senior de software. Tu tarea es construir una aplicación de escritorio completa llamada **HiPotApp** para una estación de pruebas industriales. La app se comunica con un PLC Siemens S7-1200 y un equipo Hipot 3865, ejecuta pruebas, guarda resultados en base de datos dual (local + remoto), imprime etiquetas en Zebra ZT231, y tiene auto-update.

## Repositorio GitHub

```
https://github.com/hector1516/HiPotApp
```

Clona el repo antes de empezar. Todos los archivos de documentación están en la carpeta `docs/`.

## Archivos de Documentación (LEER TODOS)

Estos archivos contienen TODOS los requisitos, esquemas de BD, layouts, y decisiones técnicas. **Debes leerlos todos antes de escribir código.**

```
docs/00-README.md              ← Índice maestro
docs/01-arquitectura.md        ← Stack tecnológico, diagrama, decisiones
docs/02-flujo-operador.md      ← Flujo completo del operador (12 pasos)
docs/03-hardware.md            ← PLC S7-1200, Hipot 3865, Zebra ZT231, scanner
docs/04-base-datos.md          ← Esquema SQL completo (17 tablas)
docs/05-ui-touch.md            ← Layout 1280x800, colores, touch, guantes
docs/06-comunicaciones.md      ← snap7, pyserial, barcode, sync manager
docs/07-admin.md               ← Panel admin (8 pestañas + import/export)
docs/08-logging-debug.md       ← Sistema de logs (app_logs, ui_state_log)
docs/09-impresora-zebra.md     ← ZPL, editor visual drag & drop, reimpresión
docs/10-auto-update.md         ← NSIS installer, GitHub Releases, updater
```

## Stack Tecnológico

| Capa | Tecnología |
|------|-----------|
| Frontend | Tauri 2.x + React 18 + Framer Motion |
| Backend | Python FastAPI + SQLAlchemy |
| PLC | snap7 (Siemens S7-1200) |
| Hipot | pyserial (RS-232) |
| BD Local | SQL Express |
| BD Remoto | SQL Server |
| Installer | NSIS (Tauri bundle) |
| Auto-update | Tauri Updater + GitHub Releases |
| Etiquetas | ZPL II → Zebra ZT231 USB |

## Arquitectura General

```
┌─────────────────────────────────────────────────┐
│           TAURI APP (React Frontend)             │
│  ┌─────────────┐  IPC  ┌──────────────────────┐ │
│  │  UI Touch   │◄─────►│  Python FastAPI      │ │
│  │  1280x800   │       │  Backend             │ │
│  │  Framer     │       │  ┌────────┐ ┌──────┐ │ │
│  │  Motion     │       │  │ PLC    │ │ Hipot│ │ │
│  └─────────────┘       │  │ (snap7)│ │(seri)│ │ │
│                        │  └───┬────┘ └──┬───┘ │ │
│                        │      │         │      │ │
│                        │  ┌───▼─────────▼───┐  │ │
│                        │  │ SQLAlchemy      │  │ │
│                        │  └────────┬────────┘  │ │
│                        └───────────┼───────────┘ │
└────────────────────────────────────┼─────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
              ┌─────▼─────┐  ┌─────▼─────┐  ┌─────▼─────┐
              │ PLC       │  │ SQL Expr  │  │ SQL Server│
              │ S7-1200   │  │ (Local)   │  │ (Remoto)  │
              │ Ethernet  │  └─────┬─────┘  └───────────┘
              └───────────┘        │ Sync
                                   │
                           ┌───────▼───────┐
                           │ Hipot 3865    │
                           │ RS-232 USB    │
                           └───────────────┘
                                   │
                           ┌───────▼───────┐
                           │ Zebra ZT231   │
                           │ USB           │
                           └───────────────┘
                                   │
                           ┌───────▼───────┐
                           │ Barcode       │
                           │ Scanner USB   │
                           └───────────────┘
```

## Flujo del Operador (resumen)

1. Escanea MODELO → App busca receta y la muestra
2. Escanea SERIE → App valida y confirma
3. Escanea BATCH → Crea ejecución
4. App dice "TODO LISTO, ESCANEAR BATCH PARA INICIAR"
5. Operador escanea BATCH otra vez → INICIA PLC automáticamente
6. PLC ejecuta prueba de consumo → resultados en UI
7. PLC termina → App inicia Hipot automáticamente
8. Hipot ejecuta prueba de aislamiento → resultados en UI
9. Cálculo final: BUENO o MALO
10. Si BUENO → imprime etiqueta automáticamente

## Base de Datos (17 tablas)

Leer `docs/04-base-datos.md` para esquema completo. Las tablas principales:

```sql
recipes                    -- Recetas por modelo
recipe_components          -- Componentes por voltaje (FAN, DOOR, etc.)
recipe_totals              -- Totales por voltaje
operators                  -- Operadores
batches                    -- Lotes
executions                 -- Ejecuciones de prueba
plc_readings               -- Lecturas PLC por componente
hipot_results              -- Resultados Hipot
barcode_logs               -- Logs de escaneo
configurations             -- Config del sistema
users                      -- Usuarios con roles
sync_log                   -- Log de sincronización
app_logs                   -- Logs de aplicación
ui_state_log               -- Logs de estado UI
label_prints               -- Historial de impresiones
label_templates            -- Plantillas editor visual
import_export_logs         -- Historial import/export config
```

## UI Design

- **Resolución**: 1280x800 (panel PC 10.1")
- **Touch**: Botones mínimo 80x80px para guantes
- **Colores dinámicos por estado**:
  - Idle: `#F5F5F5` (gris claro)
  - Escaneando: `#E3F2FD` (azul claro)
  - PLC ejecutando: `#E8F5E9` (verde claro)
  - Hipot ejecutando: `#F3E5F5` (púrpura claro)
  - BUENO: `#C8E6C9` (verde)
  - MALO: `#FFCDD2` (rojo)
- **Animaciones**: Framer Motion para transiciones de pantalla
- **Sidebar admin**: 200px, 8 pestañas

## Comunicaciones PLC (snap7)

Leer `docs/06-comunicaciones.md` y `docs/03-hardware.md`.

```python
# Ejemplo de conexión
import snap7
client = snap7.client.Client()
client.connect('192.168.1.10', 0, 1)  # IP, Rack, Slot

# Leer tag
data = client.db_read(1, 0, 10)  # DB1, offset 0, 10 bytes

# Escribir tag
client.db_write(1, 0, data)
```

**Tags PLC en DB1** (confirmar con hardware real):
- `Test_Command` (BOOL) - Comando START/STOP
- `Test_Running` (BOOL) - PLC ejecutando
- `Test_Pass` (BOOL) - Resultado PASS/FAIL
- `FAN_Min`, `FAN_Max`, `FAN_Enabled`, `FAN_Result`
- `DOOR_Min`, `DOOR_Max`, `DOOR_Enabled`, `DOOR_Result`
- `FRAME_Min`, `FRAME_Max`, `FRAME_Enabled`, `FRAME_Result`
- `DRAIN_Min`, `DRAIN_Max`, `DRAIN_Enabled`, `DRAIN_Result`
- `DEFROST_Min`, `DEFROST_Max`, `DEFROST_Enabled`, `DEFROST_Result`
- `LIGHT_Min`, `LIGHT_Max`, `LIGHT_Enabled`, `LIGHT_Result`
- `Total_Result`

## Comunicaciones Hipot (RS-232)

Leer `docs/06-comunicaciones.md` y `docs/03-hardware.md`.

```python
import serial
ser = serial.Serial('COM3', 9600, parity='N', stopbits=1, bytesize=8)
ser.write(b'START\r\n')
response = ser.readline().decode()
```

**Nota**: Los comandos exactos del Hipot 3865 se deben validar con el dispositivo físico. El manual confirma RS-232 pero no los comandos exactos.

## Editor Visual de Etiquetas

Leer `docs/09-impresora-zebra.md` sección 6.

- **Librería**: react-konva (Canvas HTML5)
- **Funciones**: Drag & drop, resize, propiedades, preview
- **Placeholders**: `{modelo}`, `{serie}`, `{batch}`, `{operador}`, `{fecha}`, `{hora}`, `{duracion}`, `{estado}`, `{fan}`, `{door}`, `{frame}`, `{drain}`, `{defrost}`, `{light}`, `{total}`, `{voltaje}`, `{plc_status}`, `{hipot_status}`
- **Exportación**: Convierte canvas a ZPL II
- **Guardado**: JSON en tabla `label_templates`

## Auto-Update

Leer `docs/10-auto-update.md`.

- **Installer**: NSIS (.exe) via Tauri
- **Build**: GitHub Actions en cada tag `v*`
- **Updater**: Tauri Updater Plugin, check cada 4 horas
- **Changelog**: RELEASE_NOTES.md + modal en la app
- **Seguridad**: Firma digital (.sig)

## Import/Export Configuración

Leer `docs/07-admin.md` sección 4.

- **Exportar**: Recetas, configs, plantillas, usuarios (con passwords hash), tags PLC
- **Importar**: Modo agregar/sobreescribir/merge, con validación y preview
- **Archivo**: JSON con metadata (versión, fecha, estación origen)
- **Historial**: Tabla `import_export_logs`

## Lo que Debes Construir

### Backend Python (`App/python/`)

```
App/python/
├── main.py              ← FastAPI app
├── models.py            ← SQLAlchemy models (17 tablas)
├── database.py          ← Config DB dual (local + remoto)
├── plc_manager.py       ← Clase PLCManager (snap7)
├── hipot_manager.py     ← Clase HipotManager (pyserial)
├── barcode_manager.py   ← Clase BarcodeManager
├── sync_manager.py      ← Clase SyncManager (local→remoto)
├── label_printer.py     ← Clase ZebraPrinter (USB)
├── label_generator.py   ← Generador ZPL desde template
├── config_manager.py    ← Import/Export configuración
├── logger.py            ← Clase Logger (app_logs)
├── auth.py              ← Autenticación (bcrypt)
├── routes/
│   ├── plc.py           ← Endpoints PLC
│   ├── hipot.py         ← Endpoints Hipot
│   ├── recipes.py       ← Endpoints recetas
│   ├── executions.py    ← Endpoints ejecuciones
│   ├── admin.py         ← Endpoints admin
│   ├── printer.py       ← Endpoints impresora
│   └── config.py        ← Endpoints import/export
└── requirements.txt
```

### Frontend React (`tauri/src/`)

```
tauri/src/
├── App.tsx              ← Router principal
├── main.tsx             ← Entry point
├── hooks/
│   ├── useUpdater.ts    ← Auto-update hook
│   ├── usePLC.ts        ← Conexión PLC
│   └── useHipot.ts      ← Conexión Hipot
├── components/
│   ├── UpdateNotification.tsx
│   ├── UpdateChangelog.tsx
│   ├── BarcodeInput.tsx
│   ├── StatusIndicator.tsx
│   └── ...
├── pages/
│   ├── HomeScreen.tsx    ← Pantalla principal operador
│   ├── TestScreen.tsx    ← Pantalla de prueba activa
│   ├── ResultScreen.tsx  ← Resultado BUENO/MALO
│   ├── AdminPanel.tsx    ← Panel admin (8 tabs)
│   └── ...
├── styles/
│   └── theme.ts          ← Colores dinámicos por estado
└── utils/
    ├── api.ts            ← Llamadas al backend
    └── labelEditor.ts    ← Editor visual react-konva
```

### Tauri Config

```
tauri/
├── tauri.conf.json      ← Config NSIS + updater
├── package.json         ← Dependencias npm
└── Cargo.toml           ← Dependencias Rust (auto)
```

## Orden de Desarrollo Recomendado

1. **Fase 1: Backend base** — FastAPI + SQLAlchemy + 17 tablas + DB dual
2. **Fase 2: PLC** — Conexión snap7, lectura/escritura tags, test manual
3. **Fase 3: Hipot** — Conexión serial, comandos básicos, test manual
4. **Fase 4: Frontend base** — Tauri + React + layout 1280x800 + routing
5. **Fase 5: Flujo operador** — Escaneo → receta → PLC → Hipot → resultado
6. **Fase 6: Admin** — 8 pestañas, CRUD completo
7. **Fase 7: Impresora** — ZPL, editor visual drag & drop
8. **Fase 8: Sync** — Local → Remoto automático
9. **Fase 9: Auto-update** — NSIS, GitHub Actions, updater
10. **Fase 10: Import/Export** — Config completa entre estaciones
11. **Fase 11: Logging** — app_logs, ui_state_log, trazabilidad
12. **Fase 12: Pulido** — Animaciones, touch, edge cases

## Restricciones Importantes

- **NO commitear secrets** — El token de GitHub NUNCA va en el código
- **Touch-first** — Todo debe funcionar con guantes (botones 80px+)
- **Offline-first** — La app debe funcionar sin red (sync después)
- **Windows-only** — Solo Windows 11 Pro (panel PC)
- **USB para Zebra** — Solo conexión USB, no Ethernet
- **RS-232 para Hipot** — No Modbus, RS-232 directo
- **Passwords hasheados** — bcrypt, nunca texto plano
- **Transacciones SQL** — Todo o nada en operaciones críticas

## Comandos Útiles

```bash
# Backend
cd App/python
pip install -r requirements.txt
uvicorn main:app --reload --host 0.0.0.0 --port 8000

# Frontend
cd tauri
npm install
npm run tauri dev

# Build installer
npm run tauri build

# Generar claves updater
npx tauri signer generate -w ~/.tauri/hopot.key
```

## Cuando Termines

1. Verificar que `npm run tauri build` genera el .exe
2. Probar en una PC real con panel touch
3. Conectar PLC real y probar comunicación
4. Conectar Hipot real y validar comandos
5. Conectar Zebra ZT231 y probar impresión
6. Subir release a GitHub
7. Probar auto-update desde otra PC

---

**IMPORTANTE**: Lee TODOS los archivos en `docs/` antes de empezar. Cada documento contiene decisiones técnicas específicas que debes respetar. Si algo no está claro, revisa el documento correspondiente o pregunta.
