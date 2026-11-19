# SQL Express: error 18456 "Login failed"

## Síntoma

La app muestra:

```text
No se pudo conectar a SQL Express:
ODBC ... State: 28000, Native error: 18456,
Login failed for user 'DESKTOP-XXXX\Usuario'.
```

Significa que SQL Server está encendido, pero **no acepta el usuario de Windows** con el que corre la app. No es un problema de la app ni del puerto.

## Solución rápida desde la app

1. En la pantalla principal, pulsa **Configurar SQL**.
2. Servidor: `.\SQLEXPRESS`
3. Base: `HiPotDB`
4. Autenticación: **SQL Server (usuario/contraseña)**.
5. Usuario: `sa` y la contraseña del SQL Express.
6. Marca **Crear la base de datos si no existe**.
7. Pulsa **Probar conexión** y luego **Guardar y reintentar**.

Si `sa` no tiene permiso para crear la base, usa una cuenta con `dbcreator`/`sysadmin` o crea la base manualmente (ver abajo).

## Cuenta SQL por defecto (`eccsa`)

A partir de v1.0.35 la app usa por defecto:

| Campo | Valor |
|---|---|
| Servidor | `.\SQLEXPRESS` |
| Base | `HiPotDB` |
| Autenticación | SQL Server |
| Usuario | `eccsa` |
| Contraseña | `Qwe123456?` |

La app también detecta las instancias locales y las muestra en una lista.

### Requisitos en SQL Server

1. SQL Server debe estar en **modo mixto** (SQL Server y Windows Authentication). Si está solo en Windows, el login `eccsa` no podrá entrar.
2. El login `eccsa` debe estar habilitado.
3. Para que la app cree `HiPotDB` automáticamente, `eccsa` necesita `dbcreator` o `sysadmin`:

```sql
ALTER SERVER ROLE dbcreator ADD MEMBER [eccsa];
```

O crea la base manualmente y dale `db_owner`:

```sql
IF DB_ID(N'HiPotDB') IS NULL CREATE DATABASE [HiPotDB];
GO
USE [HiPotDB];
IF NOT EXISTS (SELECT 1 FROM sys.database_principals WHERE name = N'eccsa')
    CREATE USER [eccsa] FOR LOGIN [eccsa];
ALTER ROLE db_owner ADD MEMBER [eccsa];
GO
```

### Seguridad

La contraseña `Qwe123456?` queda en la configuración local y en el instalador. Si el repositorio o el instalador son públicos, cámbiala después de instalar y actualiza la app en **Configurar SQL**.

## Script automático (recomendado)

En la PC donde está SQL Express, abre **PowerShell** (no necesita estar elevado) y pega:

```powershell
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$dir = "$env:TEMP\hipot-sql-fix"
New-Item -ItemType Directory -Force $dir | Out-Null
Invoke-WebRequest "https://raw.githubusercontent.com/hector1516/HiPotApp/main/scripts/fix-sql-login.ps1" -OutFile "$dir\fix-sql-login.ps1"
Start-Process powershell -Verb RunAs -ArgumentList "-NoExit -NoProfile -ExecutionPolicy Bypass -File `"$dir\fix-sql-login.ps1`""
```

Acepta el UAC. El script:

1. Prueba la conexión con el usuario de Windows actual.
2. Si no tiene permisos, inicia SQL Server en modo single-user.
3. Crea el login `DESKTOP-GNJRVIS\Administrator` y lo agrega a `sysadmin`.
4. Crea `HiPotDB` si no existe.
5. Reinicia el servicio normal.
6. Muestra el resultado y la ruta del log.

Al terminar, en la app usa **Windows**, servidor `.\SQLEXPRESS`, base `HiPotDB`, sin usuario ni contraseña.

También puedes descargar el `.cmd` que abre el script elevado:

```text
https://raw.githubusercontent.com/hector1516/HiPotApp/main/scripts/fix-sql-login.cmd
```

## Solución manual con SSMS / sqlcmd

Abre SQL Server Management Studio conectándote como un usuario que sí tenga permisos (por ejemplo `sa`) y ejecuta:

```sql
-- Agregar el usuario de Windows que usa la app
CREATE LOGIN [DESKTOP-XXXX\Usuario] FROM WINDOWS;
ALTER SERVER ROLE sysadmin ADD MEMBER [DESKTOP-XXXX\Usuario];
GO

-- Crear la base
IF DB_ID(N'HiPotDB') IS NULL
    CREATE DATABASE [HiPotDB];
GO
```

Después, en la app usa autenticación **Windows** y reintenta.

## Si `sa` está deshabilitado

En SSMS:

1. Clic derecho en el servidor → **Properties** → **Security**.
2. Selecciona **SQL Server and Windows Authentication mode**.
3. Reinicia el servicio `MSSQL$SQLEXPRESS` (Services → SQL Server (SQLEXPRESS) → Restart).
4. Habilita `sa` en **Security → Logins → sa → Status** y define contraseña.

## Verificaciones

- Servicio: `SQL Server (SQLEXPRESS)` debe estar **Running**.
- Instancia: `.\SQLEXPRESS` (o `localhost\SQLEXPRESS`).
- Base: `HiPotDB`.
- Archivo de configuración de la app:
  `%APPDATA%\HiPotApp\db_config.json`

Ejemplo usando autenticación SQL:

```json
{
  "use_sqlite": "0",
  "server": ".\\SQLEXPRESS",
  "name": "HiPotDB",
  "user": "sa",
  "password": "TU_PASSWORD"
}
```

## Nota

La app ahora **no se congela** si SQL Express falla: inicia en segundo plano, muestra el error y permite configurar la conexión desde **Configurar SQL**.
