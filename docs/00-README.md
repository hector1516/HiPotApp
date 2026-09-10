# HiPotApp — Documento maestro

Repo: https://github.com/hector1516/HiPotApp
Estado: Implementación base completada (Backend FastAPI + Frontend React Tauri v2). Ver [OPENCODE_HANDOFF.md](../OPENCODE_HANDOFF.md) para el estado actual de sincronización.

## 1. Qué es
App de escritorio para estación de pruebas:
operador escanea modelo + serie + batch, la app busca la receta,
ejecuta prueba en PLC Siemens S7-1200, luego prueba en Hipot 3865,
guarda todo con timestamps y califica BUENO / MALO.

## 2. Documentos
- `01-arquitectura.md` — stack Tauri + Python, diagrama, decisiones
- `02-flujo-operador.md` — flujo real acordado paso a paso
- `03-hardware.md` — panel PC, PLC, Hipot, barcode
- `04-base-datos.md` — esquema PostgreSQL + SQLAlchemy
- `05-ui-touch.md` — UI 10.1" 1280x800 con guantes
- `06-comunicaciones.md` — snap7, serial RS-232, IVI, barcode
- `07-admin.md` — ventana administrador
- `08-logging-debug.md` — logs de debug y trazabilidad (app_logs, ui_state_log)
- `09-impresora-zebra.md` — Zebra ZT231, ZPL, editor de etiquetas, reimpresión
- `10-auto-update.md` — instalador NSIS, GitHub Releases, auto-update, changelog

## 3. Fuentes en carpeta local (no subir a GitHub, ver .gitignore)
- `Hypot_Manual.pdf` — App. D Remote BUS RS-232, sec. Remote I/O y PLC Control
- `H4_IVI_Driver_Getting_Started_Guide.pdf` — driver IVI H4, por validar comandos
- `ari38xx-x64.msi` — instalador driver
- `Gemini_Generated_Image_*.jpg` — mockups referencia 2816x1536 (muy densos para 10.1", solo inspiración)

## 4. Nota de seguridad
El token GitHub (PAT 90 días) NO se guarda en el repo.
Va en variable de entorno local (`GIT_TOKEN`) y se rota antes de vencer.
Nunca en commits, logs ni archivos.
