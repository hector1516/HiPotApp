# HiPotApp v1.0.0

**Fecha:** 9 Sep 2026

## Incluye
- Backend FastAPI + DB dual (SQLite dev / SQL Express+Server) — 18 tablas, 36 endpoints
- PLC S7-1200 (snap7) + Hipot 3865 (serial) con modo simulado
- Flujo operador Modelo → Serie → Batch → PLC → Hipot → BUENO/MALO
- Admin 8 tabs: Usuarios, Recetas, PLC, Hipot, BD, Etiquetas, Logs, Sistema
- Editor de etiquetas drag & drop con plantillas + ZPL para Zebra ZT231
- Import/Export de configuración JSON, reimpresión, modo manual
- Todo-en-uno: `hipotapp.exe` arranca el backend automáticamente

## Instalación
1. Copiar `hipotapp.exe` + `hipot-backend-x86_64-pc-windows-msvc.exe` en la misma carpeta
2. Abrir `hipotapp.exe` (crea `hipot_local.db` junto al exe)
3. Login admin inicial: `admin` / `admin123` (cambiar en producción)

## Notas
- Hardware real: configurar IP PLC y puerto COM en Admin → BD/PLC/Hipot
- Sin red la app sigue operando local (offline-first); sync al volver la red
