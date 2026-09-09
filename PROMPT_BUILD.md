# Prompt para Construir HiPotApp

Eres un desarrollador senior. Tu tarea es construir la aplicación HiPotApp completa.

## Paso 1: Leer documentación

Lee TODOS estos archivos antes de escribir código:

```
docs/00-README.md
docs/01-arquitectura.md
docs/02-flujo-operador.md
docs/03-hardware.md
docs/04-base-datos.md
docs/05-ui-touch.md
docs/06-comunicaciones.md
docs/07-admin.md
docs/08-logging-debug.md
docs/09-impresora-zebra.md
docs/10-auto-update.md
```

## Paso 2: Construir Backend Python

Crea todo en `App/python/`:

- `main.py` — FastAPI app
- `models.py` — SQLAlchemy models (17 tablas del doc 04)
- `database.py` — DB dual (SQL Express local + SQL Server remoto)
- `plc_manager.py` — Conexión snap7 al PLC S7-1200
- `hipot_manager.py` — Conexión serial RS-232 al Hipot 3865
- `barcode_manager.py` — Lector USB HID
- `sync_manager.py` — Sync local→remoto
- `label_printer.py` — Impresora Zebra ZT231 USB
- `label_generator.py` — Generador ZPL desde plantilla
- `config_manager.py` — Import/Export configuración
- `logger.py` — Logger a app_logs
- `auth.py` — Auth bcrypt
- `routes/` — Endpoints API
- `requirements.txt`

## Paso 3: Construir Frontend React

Crea todo en `tauri/src/`:

- `App.tsx` — Router principal
- `main.tsx` — Entry point
- `hooks/useUpdater.ts` — Auto-update (doc 10)
- `hooks/usePLC.ts` — Conexión PLC
- `hooks/useHipot.ts` — Conexión Hipot
- `components/UpdateNotification.tsx` — Banner de update
- `components/UpdateChangelog.tsx` — Modal changelog
- `components/BarcodeInput.tsx` — Input de barcode
- `components/LabelEditor.tsx` — Editor visual drag & drop (react-konva)
- `pages/HomeScreen.tsx` — Pantalla principal
- `pages/TestScreen.tsx` — Pantalla de prueba
- `pages/ResultScreen.tsx` — Resultado BUENO/MALO
- `pages/AdminPanel.tsx` — Panel admin (8 tabs)
- `styles/theme.ts` — Colores dinámicos por estado

## Paso 4: Configurar Tauri

- `tauri/tauri.conf.json` — NSIS + updater
- `.github/workflows/build.yml` — Build automático

## Reglas Importantes

1. **Touch-first**: Botones mínimo 80x80px
2. **Offline-first**: App funciona sin red
3. **Windows-only**: Solo Windows 11 Pro
4. **Passwords hasheados**: bcrypt, nunca texto plano
5. **Transacciones SQL**: Todo o nada
6. **Barcodes en MAYÚSCULAS**: `.strip().upper()`
7. **Colores dinámicos**: Cambian según estado del test
8. **No commitear secrets**

## Orden de Construcción

1. Backend base (FastAPI + models + DB)
2. PLC (snap7)
3. Hipot (serial)
4. Frontend base (Tauri + React + layout)
5. Flujo operador completo
6. Admin (8 tabs)
7. Impresora + editor etiquetas
8. Sync local→remoto
9. Auto-update
10. Import/Export
11. Logging
12. Pulido y animaciones

Empieza leyendo los 10 docs, luego construye fase por fase.
