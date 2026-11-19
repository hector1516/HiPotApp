#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Repara el login de Windows para SQL Express y crea HiPotDB si falta.

.DESCRIPTION
    Error 18456 = SQL Server rechazó el usuario de Windows actual.
    Este script:
      1. Intenta conectarse con la cuenta actual.
      2. Si no tiene permisos, inicia SQL Server en modo single-user.
      3. Crea el login del usuario Windows actual como sysadmin.
      4. Crea HiPotDB si no existe.
      5. Reinicia el servicio normal.

    Ejecutar en PowerShell como Administrador, en la misma PC donde está SQL Express.
    No borra datos existentes.
#>

$ErrorActionPreference = 'Stop'
$instance = '.\SQLEXPRESS'
$serviceName = 'MSSQL$SQLEXPRESS'
$dbName = 'HiPotDB'
$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
$login = $identity
$logPath = Join-Path $env:TEMP 'hipot-fix-sql-login.log'

function Write-Log([string]$message) {
    $line = "[{0}] {1}" -f (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'), $message
    Write-Host $line
    Add-Content -Path $logPath -Value $line -Encoding UTF8
}

function Find-Sqlcmd {
    $cmd = Get-Command sqlcmd.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $candidates = Get-ChildItem 'C:\Program Files\Microsoft SQL Server' -Recurse -Filter sqlcmd.exe -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending
    if ($candidates) { return $candidates[0].FullName }
    return $null
}

$sqlcmd = Find-Sqlcmd
Write-Log "Usuario Windows: $identity"
Write-Log "Instancia: $instance"
if ($sqlcmd) {
    Write-Log "sqlcmd: $sqlcmd"
} else {
    Write-Log "sqlcmd: no encontrado; se usará .NET SqlClient"
}

function Invoke-Sql([string]$query, [bool]$singleUser) {
    if ($sqlcmd) {
        $arguments = @('-S', $instance, '-E', '-b', '-l', '5', '-d', 'master', '-Q', $query)
        if ($singleUser) { $arguments += '-A' }
        $output = & $sqlcmd @arguments 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw ($output -join "`n")
        }
        return $output
    }

    Add-Type -AssemblyName System.Data
    $connectionString = "Server=$instance;Database=master;Integrated Security=True;TrustServerCertificate=True;Connect Timeout=5;Application Name=SQLCMD;"
    $connection = New-Object System.Data.SqlClient.SqlConnection $connectionString
    try {
        $connection.Open()
        $command = $connection.CreateCommand()
        $command.CommandText = $query
        $command.CommandTimeout = 60
        return $command.ExecuteNonQuery()
    } finally {
        $connection.Close()
    }
}

function Get-ServiceSqlServerExe {
    $svc = Get-CimInstance Win32_Service -Filter "Name='$serviceName'" -ErrorAction SilentlyContinue
    if (-not $svc) {
        throw "No se encontró el servicio $serviceName. Verifica el nombre de la instancia."
    }
    $path = $svc.PathName
    if ($path -match '^"([^"]+\.exe)"') { return $Matches[1] }
    if ($path -match '^(\S+\.exe)') { return $Matches[1] }
    throw "No se pudo obtener sqlservr.exe desde: $path"
}

function Escape-SqlName([string]$name) {
    return $name.Replace(']', ']]')
}

$loginSqlName = Escape-SqlName $login
$dbSqlName = Escape-SqlName $dbName
$loginLiteral = $login.Replace("'", "''")
$dbLiteral = $dbName.Replace("'", "''")

function Set-HipotSqlAccess([bool]$singleUser) {
    $loginSql = @"
USE master;
IF NOT EXISTS (SELECT 1 FROM sys.server_principals WHERE name = N'$loginLiteral')
    CREATE LOGIN [$loginSqlName] FROM WINDOWS;
IF IS_SRVROLEMEMBER('sysadmin', N'$loginLiteral') = 0
    ALTER SERVER ROLE sysadmin ADD MEMBER [$loginSqlName];
"@
    $dbSql = @"
USE master;
IF DB_ID(N'$dbLiteral') IS NULL
    CREATE DATABASE [$dbSqlName];
"@
    Invoke-Sql $loginSql $singleUser | Out-Null
    Invoke-Sql $dbSql $singleUser | Out-Null
}

function Test-HipotLogin {
    Invoke-Sql "SELECT 1 AS ok" $false | Out-Null
}

$normalOk = $false
try {
    Test-HipotLogin
    $normalOk = $true
    Write-Log "Conexión normal con autenticación Windows: OK"
} catch {
    Write-Log "Conexión normal falló: $($_.Exception.Message)"
}

if ($normalOk) {
    try {
        Write-Log "Agregando login/creando base de datos con la sesión actual..."
        Set-HipotSqlAccess $false
        Write-Log "Acceso configurado correctamente sin modo single-user."
        Write-Host ""
        Write-Host "LISTO. Abre HiPotApp y usa:" -ForegroundColor Green
        Write-Host "  Servidor : $instance"
        Write-Host "  Base     : $dbName"
        Write-Host "  Auth     : Windows (usuario actual, sin contraseña)"
        Write-Host ""
        Write-Host "Log: $logPath"
        exit 0
    } catch {
        Write-Log "La sesión actual no tiene permisos de sysadmin: $($_.Exception.Message)"
        Write-Log "Se reintentará en modo single-user."
    }
}

# ---- Modo single-user (recuperación) ----
$proc = $null
$serviceStopped = $false
try {
    $exe = Get-ServiceSqlServerExe
    Write-Log "sqlservr.exe: $exe"
    Write-Log "Deteniendo servicio $serviceName..."
    Stop-Service -Name $serviceName -Force
    $serviceStopped = $true
    Start-Sleep -Seconds 2

    $argLine = "-s SQLEXPRESS -m`"SQLCMD`""
    Write-Log "Iniciando SQL Server en single-user: $exe $argLine"
    $proc = Start-Process -FilePath $exe -ArgumentList $argLine -PassThru -WindowStyle Hidden

    $ready = $false
    for ($i = 1; $i -le 45; $i++) {
        Start-Sleep -Seconds 1
        try {
            Invoke-Sql "SELECT 1 AS ok" $true | Out-Null
            $ready = $true
            Write-Log "SQL Server single-user listo (intento $i)."
            break
        } catch {
            if ($i -eq 1 -or $i % 5 -eq 0) {
                Write-Log "Esperando single-user... intento $i"
            }
        }
    }

    if (-not $ready) {
        throw "No se pudo conectar en modo single-user. Revisa el log: $logPath"
    }

    Write-Log "Agregando login/creando base de datos en modo single-user..."
    Set-HipotSqlAccess $true
    Write-Log "Acceso configurado correctamente."
} catch {
    Write-Log "ERROR: $($_.Exception.Message)"
    throw
} finally {
    if ($proc -and -not $proc.HasExited) {
        Write-Log "Deteniendo SQL Server single-user..."
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }
    if ($serviceStopped) {
        Write-Log "Iniciando servicio normal $serviceName..."
        Start-Service -Name $serviceName -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 3
    }
}

# Verificación final
Test-HipotLogin
Write-Log "Verificación final: OK. Login y base disponibles."

Write-Host ""
Write-Host "LISTO. Abre HiPotApp y usa:" -ForegroundColor Green
Write-Host "  Servidor : $instance"
Write-Host "  Base     : $dbName"
Write-Host "  Auth     : Windows (usuario actual, sin contraseña)"
Write-Host "  Luego    : Probar conexión -> Guardar y reintentar"
Write-Host ""
Write-Host "Log: $logPath"
