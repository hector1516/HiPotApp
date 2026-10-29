# setup-kiosco.ps1
# Configura modo kiosco (Acceso asignado) en Windows 11 para HiPotApp.
# Ejecutar como Administrador.

param(
    [string]$UserName = "hipot",
    [string]$AppPath = "C:\Program Files\HiPotApp\hipotapp.exe"
)

Write-Host "=== Configurando kiosco para usuario '$UserName' ===" -ForegroundColor Cyan

# 1. Crear usuario si no existe
if (-not (Get-LocalUser -Name $UserName -ErrorAction SilentlyContinue)) {
    Write-Host "Creando usuario '$UserName'..." -ForegroundColor Yellow
    $password = ConvertTo-SecureString "Kiosco123!" -AsPlainText -Force
    New-LocalUser -Name $UserName -Password $password -FullName "Usuario Kiosco" -Description "Cuenta para kiosco HiPotApp" | Out-Null
    Add-LocalGroupMember -Group "Users" -Member $UserName
    Write-Host "Usuario creado. Password temporal: Kiosco123! (cambiarla despues)" -ForegroundColor Green
} else {
    Write-Host "Usuario '$UserName' ya existe." -ForegroundColor Green
}

# 2. Verificar que la app exista
if (-not (Test-Path $AppPath)) {
    Write-Host "ERROR: No se encontro la app en $AppPath" -ForegroundColor Red
    exit 1
}
Write-Host "App encontrada: $AppPath" -ForegroundColor Green

# 3. Configurar Assigned Access (kiosco single-app)
try {
    Set-AssignedAccess -UserName $UserName -AppPath $AppPath -ErrorAction Stop
    Write-Host "Kiosco configurado correctamente para '$UserName'." -ForegroundColor Green
} catch {
    Write-Host "Set-AssignedAccess fallo: $_" -ForegroundColor Red
    Write-Host "Intentando metodo alternativo (XML)..." -ForegroundColor Yellow

    # Generar XML de configuracion multi-app kiosco
    $xmlPath = "$env:TEMP\kiosco-config.xml"
    $xml = @"
<?xml version="1.0" encoding="utf-8"?>
<AssignedAccessConfiguration xmlns="http://schemas.microsoft.com/AssignedAccess/2017/config">
  <Profiles>
    <Profile Id="{12345678-1234-1234-1234-123456789012}">
      <AllAppsList>
        <AllowedApps>
          <App DesktopAppPath="$AppPath" />
        </AllowedApps>
      </AllAppsList>
      <StartLayout>
        <![CDATA[
        <LayoutModificationTemplate xmlns:defaultlayout="http://schemas.microsoft.com/Start/2014/FullDefaultLayout" xmlns:start="http://schemas.microsoft.com/Start/2014/StartLayout" Version="1">
          <LayoutOptions StartTileGroupCellWidth="6" />
          <DefaultLayoutOverride>
            <StartLayoutCollection>
              <defaultlayout:StartLayout GroupCellWidth="6">
                <start:DesktopApplicationTile Size="2x2" Column="0" Row="0" DesktopApplicationID="HiPotApp" />
              </defaultlayout:StartLayout>
            </StartLayoutCollection>
          </DefaultLayoutOverride>
        </LayoutModificationTemplate>
        ]]>
      </StartLayout>
      <Taskbar ShowTaskbar="false" />
    </Profile>
  </Profiles>
  <Configs>
    <Config>
      <Account>$UserName</Account>
      <DefaultProfile Id="{12345678-1234-1234-1234-123456789012}" />
    </Config>
  </Configs>
</AssignedAccessConfiguration>
"@
    $xml | Out-File -FilePath $xmlPath -Encoding utf8
    Write-Host "XML generado en: $xmlPath" -ForegroundColor Yellow
    Write-Host "Aplicalo con: Set-AssignedAccess -ConfigurationFile $xmlPath" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Instrucciones finales ===" -ForegroundColor Cyan
Write-Host "1. Cierra sesion del usuario '$UserName'."
Write-Host "2. Inicia sesion con '$UserName'."
Write-Host "3. La app deberia arrancar en modo kiosco."
Write-Host "4. Para salir del kiosco, inicia sesion como administrador y ejecuta:"
Write-Host "   Remove-AssignedAccess -UserName '$UserName'" -ForegroundColor Yellow