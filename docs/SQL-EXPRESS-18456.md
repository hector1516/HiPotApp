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

## Solución con SSMS / sqlcmd

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
