# HiPotApp v1.1.1 — Ajustes de Live y lector gigante

> **IMPORTANTE:** release de banco. El watchdog del PLC sigue pendiente
> (`docs/PLC-WATCHDOG.md`). No usar en producción hasta completar las pruebas HIL.

## Cambios

- **Live sin solapes**
  - Corregido el traslape de los gauges analógicos con el estado de entradas,
    el bloque I/O y la tabla DB100 en 1366×768 y 1280×720.
  - La tabla DB100 ahora tiene altura limitada y scroll propio; ya no empuja
    los gauges ni se encima con el texto inferior.
  - Los gauges analógicos cambian de tamaño según la altura de la pantalla:
    compactos en 720p/768p y grandes en 1080p.
  - La columna de componentes aprovecha más ancho/alto en 1080p.
  - Los Hipot pasan a 3 columnas también en 1366×768; ya no queda un gauge
    huérfano en una segunda fila.
- **Hipot**
  - Corregida la detección de lectura: los gauges de Hipot ya muestran voltaje,
    corriente y duración cuando el equipo responde.
  - Nuevo panel grande de **último ensayo** (PASS/FAIL/EN PRUEBA/SIN LECTURA)
    que llena el espacio libre del bloque.
- **Gauge ampliado**
  - Ahora ocupa **toda la pantalla** con fondo translúcido y desenfoque.
  - Número gigante con decimales adaptativos (hasta 4 decimales en valores
    pequeños).
  - Animación de conteo, entrada con resorte, anillos pulsantes, aurora,
    cuadrícula en movimiento y brillo animado en el número.
  - Se abre en un portal sobre toda la app: cubre barra superior y pie.
  - Cierre con botón, clic fuera o `ESC`.

## Validación

- `npm run build`: correcto.
- Revisión visual en 1920×1080, 1366×768 y 1280×720.
- `cargo test`: 47/47.

## Assets

- `HiPotApp_1.1.1_x64-setup.exe`
- `HiPotApp_1.1.1_x64-setup.exe.sig`
- `latest.json`
