## HiPotApp v1.0.22 — Release estable

### Cambios funcionales
- **Puerto del backend cambiado a 8765**: se evitan conflictos con otros servicios que usan el puerto 8000. Se actualizaron main.py, backend_entry.py, sidecar.py, api.ts, vite.config.ts y el texto de debug.
- **Teclado alfanumerico con signos**: fila adicional de simbolos (@#$%&*()_-+=).
- **App maximizada al iniciar**: la ventana arranca maximizada (maximized: true).
- **Indicador de backend**: muestra "Backend OK" (verde) o "Sin conexion" (rojo) en el pie de pagina. Se actualiza cada 3 segundos.
- **Mejor mensaje de error**: si falla al guardar la IP del PLC por falta de conexion al backend, se muestra un mensaje claro indicando que el backend no esta disponible.

### Assets
- HiPotApp_1.0.22_x64-setup.exe
- HiPotApp_1.0.22_x64-setup.exe.sig
- latest.json

### Notas
- Si el indicador dice "Sin conexion", verifica que no haya otro proceso en el puerto 8765 y que el binario del backend exista en tauri/binaries/.