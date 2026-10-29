# Auditoría de Seguridad — HiPotApp

**Fecha:** 29 de Octubre, 2026  
**Alcance:** Todo el repositorio (código, scripts, configuración, historial local).

---

## 1. Hallazgos

### 1.1 Clave JWT hardcodeada (`App/python/auth.py`)
- **Línea:** `SECRET = "hipot-dev-secret-change-me"`
- **Riesgo:** Uso de una clave secreta fija en el código fuente. Si el repositorio se comparte, cualquiera puede generar tokens JWT válidos.
- **Acción:** Se reemplazó por lectura de la variable de entorno `JWT_SECRET`. Si no está definida, se usa un valor por defecto solo para desarrollo y se emite una advertencia en consola.

### 1.2 Contraseña por defecto en script de kiosco (`scripts/setup-kiosco.ps1`)
- **Línea:** `$password = ConvertTo-SecureString "Kiosco123!" -AsPlainText -Force`
- **Riesgo:** Contraseña predecible documentada en el script. Cualquiera que lea el script conoce la contraseña inicial.
- **Acción:** Se mantuvo como valor por defecto para facilitar la instalación, pero se documentó que **debe cambiarse inmediatamente** tras la configuración. El script ahora acepta el parámetro `-Password` para personalizarla.

### 1.3 Token de GitHub en la URL del remote (`git remote`)
- **Hallazgo:** El remote local contenía el token `ghp_...` en la URL (`https://ghp_...@github.com/...`).
- **Riesgo:** El token estaba expuesto en la configuración local de Git. No estaba en el código del repo, pero sí en `.git/config`.
- **Acción:** Se actualizó la URL del remote a `https://github.com/hector1516/HiPotApp.git` (sin token). Para operaciones remotas se usará `GH_TOKEN` o un credential helper.

### 1.4 Variables de entorno con valores por defecto sensibles
- **Archivo:** `App/python/database.py`
- **Hallazgo:** Valores por defecto como `DB_REMOTE_SERVER = "192.168.1.200"` y `DB_LOCAL_SERVER = ".\\SQLEXPRESS"`.
- **Riesgo:** Bajo. Son valores de infraestructura, no credenciales. Se mantienen porque son necesarios para el funcionamiento offline-first.
- **Acción:** Ninguna. Se documenta que deben sobrescribirse en producción mediante variables de entorno.

### 1.5 `.gitignore`
- **Estado:** Ya incluye `*.token`, `*.key`, `.env`, `credentials.*`, `**/secrets.*`.
- **Acción:** Se añadió `.DS_Store` para evitar archivos basura de macOS.

---

## 2. Cambios aplicados

| Archivo | Cambio |
|---|---|
| `App/python/auth.py` | `SECRET` ahora se lee de `JWT_SECRET` (env). Fallback con advertencia. |
| `scripts/setup-kiosco.ps1` | Acepta parámetro `-Password`. Documentado. |
| `.gitignore` | Añadido `.DS_Store`. |
| `git remote` | URL sin token. |

---

## 3. Recomendaciones para producción

1. **Definir `JWT_SECRET`** en el entorno del servidor/backend antes de arrancar la app.
2. **Cambiar la contraseña del usuario `hipot`** inmediatamente después de ejecutar el script de kiosco.
3. **No commitear** archivos `.env` con credenciales reales.
4. **Rotar el token de GitHub** si se sospecha que fue expuesto (el token original ya fue usado en el remote local).

---

## 4. Verificación

- Escaneo de patrones de secretos (tokens, passwords, API keys) en todos los archivos trackeados: **sin hallazgos críticos** (solo placeholders y valores por defecto documentados).
- Escaneo de archivos no trackeados: **sin hallazgos adicionales**.
