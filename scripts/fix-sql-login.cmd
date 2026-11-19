@echo off
setlocal
set "SCRIPT=%~dp0fix-sql-login.ps1"
echo Se abrirá una ventana de PowerShell como Administrador.
echo Acepta el aviso de UAC de Windows.
powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath powershell.exe -Verb RunAs -ArgumentList '-NoExit','-NoProfile','-ExecutionPolicy','Bypass','-File','\"%SCRIPT%\"'"
endlocal
