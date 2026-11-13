## HiPotApp v1.0.21 — Release estable

### Cambios funcionales
- **Indicador de backend en el pie de página**: muestra "Backend OK" (verde) o "Sin conexión" (rojo) según el estado del backend Python. Se actualiza cada 3 segundos.
- **Mejor mensaje de error**: si falla al guardar la IP del PLC por falta de conexión al backend, se muestra un mensaje claro indicando que el backend no está disponible.
- **Teclado alfanumérico con signos**: fila adicional de símbolos (@#$%&*()_-+=).
- **App maximizada al iniciar**: la ventana arranca maximizada.

### Assets
- `HiPotApp_1.0.21_x64-setup.exe`
- `HiPotApp_1.0.21_x64-setup.exe.sig`
- `latest.json`

### Notas
- Si el indicador dice "Sin conexión", el backend no arrancó. Posibles causas: puerto 8000 ocupado, binario faltante, o app en modo desarrollo sin backend manual.