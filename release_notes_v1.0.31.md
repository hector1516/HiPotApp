# HiPotApp v1.0.31 — Migración completa a Rust / SQL Express

## Cambios principales

- **Un solo ejecutable**: se eliminó el sidecar Python, el puerto 8765 y `api.ts`.
- **BD local SQL Express** (ODBC Driver 17/18 for SQL Server).
- **Migración integrada** de `hipot_local.db` (SQLite) a SQL Express.
- **Sync multitable** local → SQL Server remoto, con `sync_log` y `synced_at`.
- **PLC S7-1200** completo en Rust: configuración, tags, Q/I/IA, receta, start/stop, resultados y modo manual.
- **Hipot 3865** en Rust por RS-232: puertos, comandos, parseo de mediciones y modo manual.
- **Ciclo real de producción** PLC → Hipot → guardado de resultados. Sin `Math.random()` ni resultados simulados.
- **Sin modos de simulación** de PLC/Hipot.
- **Auth/usuarios**, plantillas de etiqueta, import/export, historial, auditoría y logs en Rust.
- **Zebra ZT231** por Ethernet TCP 9100 o spooler de Windows.
- **Hardening**: instancia única, ErrorBoundary global, rotación de logs (30 días / 5000 registros), splash con timeout.

## Requisitos de instalación

1. **SQL Server Express** instalado en la estación.
2. **ODBC Driver 17 for SQL Server** (o 18).
3. Base de datos local `HiPotDB` (la app puede crearla desde el panel).

## Primer arranque / migración

1. Instalar y arrancar SQL Express.
2. Abrir **Configuración → Sistema → Base de datos local**.
3. Verificar servidor (`.\SQLEXPRESS`), base (`HiPotDB`), usuario/contraseña.
4. Pulsar **Probar SQL Express**.
5. Pulsar **Crear tablas locales**.
6. Pulsar **Migrar SQLite → SQL Express** (si existe `%APPDATA%\HiPotApp\hipot_local.db`).
7. Configurar la BD remota y probar **Verificar conexión**.

## Validación pendiente

- Prueba física con PLC S7-1200, Hipot 3865 y Zebra ZT231.
- Instalación en VM limpia con SQL Express.
- Prueba de auto-update desde la versión anterior.
