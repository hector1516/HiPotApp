# HiPotApp v1.1.0 — Release de banco

> **IMPORTANTE:** esta versión es para pruebas de banco. Todavía no debe usarse
> como release de producción: el PLC puente aún no tiene watchdog de heartbeat.
> Si la PC se cuelga durante un ciclo, las salidas pueden quedar energizadas.
> No liberar a producción hasta implementar `docs/PLC-WATCHDOG.md` y completar
> las pruebas HIL.

## Cambios principales

- **Motor de secuencia en la PC** sobre el PLC puente `DB100`.
  - Hipot primero.
  - Selección de voltaje 120/220.
  - Estabilización y medición de voltaje.
  - Selección de rango por componente.
  - Temporización, medición y evaluación GOOD/BAD en la PC.
- **Paro de emergencia**
  - Se lee de `DB100.IB0.0` (configurable).
  - Detiene la secuencia y envía `RESET` al Hipot.
  - Si el voltaje falla, aborta antes de energizar componentes.
- **Calibración analógica IA1–IA4**
  - Dos puntos por canal.
  - Un canal sin calibrar se muestra como “sin calibrar” y bloquea la secuencia.
  - Guardado versionado y auditoría.
- **Mapa configurable de salidas lógicas ↔ bits Q**
  - Precarga de KM1, KM2, rangos, FAN, LIGHT, FRAME, DOOR, DRAIN y HEATERS.
  - Validación de duplicados, bits inválidos y salidas faltantes.
  - Edición bloqueada durante una secuencia.
- **Pantalla Lógica**
  - Vista animada en tiempo real: paso, temporizador, componente, voltaje,
    corriente, salidas activas e historial.
  - Editor restringido para administrador: tiempos, orden de componentes,
    tolerancia de voltaje y parada por fallo.
  - Borrador, publicación, versionado, auditoría y reversión.
  - La receta decide qué componentes se prueban; el flujo no puede omitirlos.
- **Live**
  - Valores analógicos calibrados.
  - Estado de calibración por canal.
  - Consumos por componente desde los eventos del motor PC.
- **Ciclo de supervisión de 100 ms** para paro y condiciones durante las esperas.

## Requisitos antes de probar

1. Configurar IP/rack/slot del PLC y habilitarlo.
2. Verificar que `DB100` tenga el mapa de E/S actual.
3. Calibrar IA1–IA4 en **Configuración → Calibración**. Sin esto, la secuencia
   eléctrica no inicia.
4. Revisar **Configuración → Salidas** contra el tablero real.
5. Confirmar el orden real de `IW100/102/104/106` (IA1–IA4).

## Limitaciones conocidas

- **Sin watchdog de PLC todavía.** Ver `docs/PLC-WATCHDOG.md`.
- **Sin pruebas HIL** con PLC, Hipot y señales analógicas reales.
- El editor de lógica es restringido: no crea transiciones arbitrarias.
- La compilación del instalador se realiza en CI; esta PC bloquea los build
  scripts de Rust por la política Application Control.

## Assets

- `HiPotApp_1.1.0_x64-setup.exe`
- `HiPotApp_1.1.0_x64-setup.exe.sig`
- `latest.json`

## Validación local

- `cargo test`: 47/47 pruebas.
- `npm run build`: correcto.
