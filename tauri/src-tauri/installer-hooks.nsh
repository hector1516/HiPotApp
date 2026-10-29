; Hooks NSIS para HiPotApp: auto-inicio en Windows

!macro NSIS_HOOK_POSTINSTALL
  ; Agregar entrada al registro de inicio (HKCU) para el usuario actual
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "HiPotApp" "$INSTDIR\hipotapp.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Eliminar entrada del registro de inicio
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "HiPotApp"
!macroend