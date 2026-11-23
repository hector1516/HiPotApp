# Plan: lógica en PC, PLC como I/O, calibración analógica y visor de lógica

> **Estado:** borrador para revisión. No implementar hasta aprobación.

## Decisiones de la reunión (2026-11-23)

1. **Paro de emergencia**
   - El paro es eléctrico: un relevador corta la energía directamente.
   - El PLC solo recibe el estado del paro en una entrada.
   - La entrada puede moverse y será configurable desde la app.
   - El paro también detiene el Hipot:
     - La app envía `RESET` al 3865 al detectar el paro.
     - Si el relevador corta la alimentación del Hipot, mejor; el `RESET` es la segunda barrera.
   - La app usa esa entrada para detener la secuencia, marcar el ciclo como fallido y exigir un reset manual.
   - La seguridad no depende de la PC ni del programa del PLC.

2. **Programa del PLC**
   - La lógica de referencia es la del PDF actual.
   - Para implementar se solicitará el proyecto TIA actualizado y los bloques exportados como SCL/AWL.

3. **Pantalla de lógica**
   - Tendrá **modo vista** y **modo edición**.
   - Modo vista: flujo animado en tiempo real.
   - Modo edición: permitirá modificar pasos, transiciones, condiciones, temporizadores y acciones.
   - La edición solo se aplica con la secuencia detenida y con validación previa.

4. **Ciclo de control**
   - **100 ms confirmado** (10 Hz).
   - El watchdog del PLC será independiente de la UI y del ciclo de la PC.

5. **Salidas**
   - Todas las salidas serán configurables desde la app.
   - Se precargan las salidas actuales (KM1, KM2, K1–K9, etc.).
   - Se agregará un mapa de salidas para asociar salida lógica ↔ bit del PLC.

6. **Lógica de proceso**
   - Se mantiene la lógica del PDF como base; los cambios posteriores se harán en el editor de la PC.

7. **Hipot**
   - La secuencia Hipot sigue igual; la PC la orquesta y el PLC solo aporta E/S.

## Objetivo

Mover la secuencia de prueba y la lógica de control del PLC a la aplicación de PC, dejando el S7-1200 como módulo de E/S. Agregar calibración de canales analógicos y una pantalla para revisar la lógica en ejecución.

## Decisión de arquitectura

El S7-1200 no permite leer/escribir `I/Q` directamente por PUT/GET; ya se comprobó con `I0.0`.

Por eso el PLC no puede quedar 100% sin programa. La opción recomendada es:

**PLC = puente de E/S con watchdog**
- Copia `I` y `IW` a una DB no optimizada.
- Copia los bits de salida de esa DB a `Q`.
- Refleja el estado del paro y mantiene el watchdog.
- No contiene receta, tiempos, secuencia ni lógica de componentes.

### Comparación de opciones

1. **DB espejo S7 + PUT/GET (recomendada)**
   - Reutiliza la comunicación actual.
   - Requiere un programa mínimo en el PLC.
   - Menor riesgo de integración.

2. **Modbus TCP**
   - También requiere programa de mapeo en el PLC.
   - Agrega configuración y otro protocolo.
   - No elimina la necesidad del watchdog.

3. **Quitar el PLC y usar E/S industriales en la PC**
   - Elimina el PLC, pero requiere hardware nuevo y una tarjeta de seguridad.
   - No es compatible con el hardware actual.

## Instrumentación analógica confirmada

Módulos Weidmüller ACT20P instalados:

| Equipo | Modelo | Entrada | Salida | Uso probable |
|---|---|---|---|---|
| A1 | ACT20P-VM-AO-S (7760054306) | Voltaje AC/DC | Analógica 0–10 V | Medición de voltaje de línea |
| A2 | ACT20P-CML-10-AO-RC-S (2044850000) | Corriente 0–1/5/10 A | Analógica 0(4)–20 mA + relé | Canal de corriente 0–1 A o 0–10 A |
| A3 | ACT20P-CML-10-AO-RC-S (2044850000) | Corriente 0–1/5/10 A | Analógica 0(4)–20 mA + relé | Canal de corriente 0–1 A o 0–10 A |
| A4 | ACT20P-CMT-30-AO-RC-P (1510320000) | Corriente 0–20/25/30 A | Analógica 0(4)–20 mA + relé | Canal de corriente 0–20 A |

Pendiente de confirmar con el tablero:
- Configuración de DIP switches de cada módulo.
- Rango real seleccionado en A2 y A3.
- Tipo de salida analógica configurada: 0–20 mA o 4–20 mA.
- Canal del PLC al que llega cada módulo.
- Uso de los relés de A2, A3 y A4: ¿van a entradas digitales del PLC o quedan libres?
- Correspondencia con los canales actuales de la app: IA1, IA2, IA3 e IA4.

La calibración de la app debe hacerse por canal físico, no por nombre de componente.

## Límite de seguridad

El paro eléctrico es la protección principal: el relevador corta la energía sin depender de la PC ni del PLC.

El PLC solo refleja el estado del paro en una entrada. Esa entrada puede ser configurable desde la app.

La PC debe:
- Detectar el paro.
- Detener la secuencia.
- Enviar `RESET` al Hipot de inmediato, sin esperar al ciclo normal.
- Marcar el ciclo como fallido.
- Exigir un reset manual antes de permitir otro ciclo.
- No reanudar automáticamente al liberar el paro.

El Hipot debe quedar detenido por dos vías:
- Corte eléctrico del relevador, si aplica.
- Comando `RESET` de la app al 3865.

Si la PC no puede enviar `RESET` porque el puerto está ocupado o la PC falló, el corte eléctrico debe dejar el Hipot sin alta tensión.

El watchdog sigue siendo necesario para otro riesgo distinto: si la PC se cuelga o pierde comunicación, el PLC debe apagar las salidas que controla. El paro eléctrico no cubre una caída de la PC.

El PLC debe conservar:
- Watchdog de la PC.
- Corte de salidas si la PC deja de enviar heartbeat.
- Interlocks críticos definidos por el especialista Siemens.

La lógica de secuencia sí se mueve a la PC.

## Fases

### Fase 0 — Levantamiento y contrato de E/S

Entregables:
- Tabla completa de entradas, salidas, analógicas y módulos.
- Direcciones reales y rangos eléctricos.
- Mapa de salidas lógicas ↔ bits del PLC, con las salidas actuales precargadas.
- Máquina de estados actual extraída del PLC (PDF/SCL).
- Lista de interlocks y condiciones de seguridad.
- Tiempos, tolerancias y condiciones de PASS/FAIL.
- Decisión de watchdog: tiempo de heartbeat y tiempo de corte.
- Dirección configurable del paro eléctrico.

Criterio de aceptación:
- El especialista Siemens firma el mapa de E/S, el mapa de salidas y el límite PLC/PC.

### Fase 1 — PLC puente de E/S

Entregables:
- DB no optimizada `DB_PC_IO` con:
  - Entradas digitales.
  - Salidas digitales.
  - Analógicas crudas.
  - Heartbeat.
  - Estado de paro y watchdog.
  - Dirección/estado del paro configurable.
- OB1 mínimo:
  - `I` → DB.
  - DB → `Q`.
  - Watchdog.
  - Estado del paro.
- Sin lógica de receta ni de secuencia.
- Documento de offsets y bits.

Criterio de aceptación:
- Prueba de lectura/escritura de todas las E/S.
- Al desconectar la PC o detener el heartbeat, las salidas caen.
- Al presionar el paro, la energía se corta por el relevador, la app detecta el estado y envía `RESET` al Hipot.

### Fase 2 — Motor de lógica en la PC

Entregables:
- Hilo de control separado de React, con ciclo de **100 ms** confirmado.
- Heartbeat cada 100 ms; timeout de watchdog propuesto: 500 ms, a validar con el especialista Siemens.
- Máquina de estados versionada.
- Temporizadores, transiciones, interlocks y evaluación de resultados.
- Escritura de salidas a `DB_PC_IO`.
- Heartbeat.
- Log de cada transición y comando.
- Modo **shadow**: calcula salidas y estados sin escribirlas, para comparar contra la lógica anterior del PLC.

Criterio de aceptación:
- Pruebas unitarias de estados, transiciones, timeout y fallos.
- En shadow mode, la secuencia calculada coincide con la del PLC actual.
- Si la PC se congela, el watchdog del PLC apaga salidas.

### Fase 3 — Integración de receta, Hipot y resultados

Entregables:
- La receta selecciona componentes, tiempos, tolerancia y parámetros Hipot.
- La PC inicia/detiene Hipot y PLC.
- La PC evalúa corriente por componente y voltaje.
- Resultado final `GOOD/BAD` calculado en PC.
- Persistencia y logs actuales se mantienen.

Criterio de aceptación:
- Ciclo completo GOOD.
- Ciclo BAD por Hipot.
- Ciclo BAD por corriente.
- Ciclo BAD por voltaje.
- Paro en cualquier paso deja salidas seguras.

### Fase 4 — Pantalla de calibración analógica

Entregables:
- Nueva pestaña en Configuración: **Calibración**.
- Lista de canales analógicos.
- Para cada canal:
  - Valor crudo en vivo.
  - Valor de ingeniería.
  - Unidad.
  - Punto bajo y punto alto.
  - Ganancia y offset calculados.
  - Botón “Tomar punto” para calibrar con una señal conocida.
  - Validación y guardado versionado.
- La calibración se aplica en la lógica PC y en Live.

Criterio de aceptación:
- Dos puntos conocidos producen el valor esperado dentro de la tolerancia definida.
- Reiniciar la app conserva la calibración.
- Un canal sin calibrar se muestra como “sin calibrar”, no como cero válido.

Pendiente de definir:
- Módulos, canales, rangos (0–10 V, 4–20 mA, etc.).
- Valores crudos mínimos/máximos.
- Precisión aceptable.

### Fase 4.5 — Mapa de salidas configurable

Entregables:
- Pantalla o pestaña para asociar cada salida lógica con un bit del PLC.
- Precarga de las salidas actuales: KM1, KM2, K1–K9 y las que ya usa la app.
- Validación de duplicados, bits inválidos y salidas sin mapear.
- Solo se puede editar con la secuencia detenida.
- La lógica de la PC usa el mapa, no direcciones fijas.

Criterio de aceptación:
- Cambiar una salida de bit no requiere recompilar la app.
- Una salida sin mapear no puede activarse.
- El cambio queda versionado.

### Fase 5 — Pantalla de lógica: vista y edición

La pantalla tendrá dos modos: **Vista** y **Edición**.

#### Modo Vista

- Diagrama de flujo de pasos y transiciones.
- Animación en tiempo real:
  - Paso activo resaltado.
  - Transición recorrida.
  - Temporizador en curso.
  - Valores de condiciones.
  - Salidas activas.
- Panel lateral con:
  - Estado actual.
  - Paso actual.
  - Temporizador.
  - Condiciones verdaderas/falsas.
  - Última transición y último error.
  - Historial reciente.

#### Modo Edición

- Botón **Modo edición** visible solo para administrador.
- Solo se puede editar con la secuencia detenida.
- Permite:
  - Agregar, borrar y reordenar pasos.
  - Editar transiciones y condiciones.
  - Elegir señales de entrada, resultados, temporizadores y estados.
  - Asignar salidas y acciones.
  - Duplicar pasos.
- Validación antes de guardar:
  - Sin pasos huérfanos.
  - Sin transiciones sin destino.
  - Sin salidas no mapeadas.
  - Sin condiciones incompletas.
  - Sin bucles sin timeout.
- Guardar como borrador, **publicar** y **rollback**.
- Versionado y auditoría de quién cambió qué.
- Simulación antes de publicar.

No será un editor de código libre: será un editor de flujo restringido a las señales y acciones permitidas. Esto evita lógica inválida o insegura.

Criterio de aceptación:
- En modo Vista se entiende por qué la secuencia está detenida o esperando.
- En modo Edición se puede cambiar un paso y publicarlo sin editar código.
- Una lógica inválida no se puede publicar.
- Los cambios quedan versionados y se pueden revertir.

Pendiente de definir:
- ¿La edición incluye cualquier condición lógica o solo las condiciones predefinidas?
- ¿Quién puede publicar además del administrador?
- ¿Se importa automáticamente la secuencia actual del PLC como primer flujo editable?

### Fase 6 — Validación y migración

Entregables:
- Pruebas en banco con PLC, Hipot y señales analógicas.
- Pruebas de:
  - Paro de emergencia.
  - Pérdida de red.
  - Cierre de la app.
  - Watchdog.
  - Reinicio de la PC.
  - Salidas atascadas.
- Comparación shadow contra lógica anterior.
- Manual de operación y recuperación.

Criterio de aceptación:
- El especialista Siemens y el responsable de seguridad aprueban.
- La PC no puede dejar salidas activas si pierde comunicación.

### Fase 7 — Release

- Versión mayor `2.0.0`.
- Migración de configuración y recetas.
- Release solo después de aprobación y pruebas.

## Archivos previstos

### Backend Rust

- `tauri/src-tauri/src/engine/mod.rs`: motor de secuencia.
- `tauri/src-tauri/src/engine/state.rs`: estados, pasos y transiciones.
- `tauri/src-tauri/src/engine/logic.rs`: esquema de flujo editable, validación y versionado.
- `tauri/src-tauri/src/engine/io.rs`: DB espejo, heartbeat y salidas.
- `tauri/src-tauri/src/engine/outputs.rs`: mapa de salidas lógicas ↔ bits del PLC.
- `tauri/src-tauri/src/engine/calibration.rs`: calibración de analógicas.
- `tauri/src-tauri/src/engine/safety.rs`: paro, watchdog e interlocks.
- `tauri/src-tauri/src/plc/mod.rs`: bajar a primitivas S7; quitar la secuencia DB1000/1001.
- `tauri/src-tauri/src/main.rs`: comandos Tauri; `run_test_cycle` se delega al motor.

### Frontend

- `tauri/src/pages/CalibrationScreen.tsx` o pestaña en `AdminPanel.tsx`.
- `tauri/src/pages/LogicScreen.tsx`: modo vista/edición con diagrama de flujo.
- `tauri/src/pages/OutputMapScreen.tsx` o pestaña en `AdminPanel.tsx`.
- `tauri/src/pages/LiveScreen.tsx`: consumir la nueva capa de E/S.
- `tauri/src/hooks/useEngine.ts`: estado/eventos del motor.

### Documentación

- `docs/PLC-PC-IO-MAP.md`: mapa de DB, bits y offsets.
- `docs/PC-LOGIC-SPEC.md`: estados, transiciones y condiciones.
- `docs/SAFETY.md`: watchdog, paro e interlocks.

## Riesgos

1. **Seguridad:** PC no determinista. Mitigación: paro eléctrico independiente y watchdog en PLC.
2. **Latencia:** red S7 y ciclo de PC. Mitigación: ciclo 100 ms, timeouts y estados de fallo.
3. **Analógicas:** valores crudos y escalado desconocidos. Mitigación: calibración de dos puntos y validación en banco.
4. **Migración:** la lógica actual no está 100% documentada. Mitigación: extraerla del PDF/SCL y validar en shadow mode.
5. **Editor de lógica:** puede introducir lógica inválida o insegura. Mitigación: esquema restringido, validación, simulación, versionado y publicación solo con la secuencia detenida.
6. **Pérdida de comunicación:** debe apagar salidas. Mitigación: heartbeat con timeout en PLC.

## Qué necesito del PLC

Para mover la lógica con precisión necesito el proyecto actualizado y, si es posible, los bloques exportados como fuente:

1. **Proyecto TIA Portal archivado** (`.zap20`/`.ap20` o el `.zip` actualizado).
2. **Bloques exportados como SCL/AWL**:
   - `OB1`
   - `FC1`, `FC2`, `FC3`, `FC4`
   - `FB1`
   - `DB2`, `DB3`, `DB5`, `DB1000`, `DB1001`
   - UDTs utilizados
3. **Tabla de tags PLC** (Excel o captura) con direcciones y comentarios.
4. **Configuración de hardware**:
   - Orden de los módulos.
   - Canales analógicos, tipo de señal y rango.
   - Direcciones de E/S.
5. **Plano eléctrico del paro** y de las salidas.
6. Si no puedes exportar SCL/AWL: PDF completo de todos los bloques y DBs, legible y con offsets.

El PDF que ya tengo sirve como referencia, pero para implementar el motor en la PC necesito el proyecto actualizado y las fuentes de los bloques.

## Decisiones confirmadas

1. El paro es eléctrico: el relevador corta la energía, el PLC recibe el estado en una entrada configurable y la app manda `RESET` al Hipot.
2. La lógica del PDF es la base.
3. La pantalla de lógica tendrá Vista y Edición con flujo animado.
4. Ciclo de control por defecto: **100 ms**.
5. Todas las salidas serán configurables desde la app; se precargan las actuales.
6. El Hipot sigue igual.

## Pendientes

1. Módulos y canales analógicos: tipo de señal, rango y valores crudos.
2. ¿La edición de lógica permite cualquier condición o solo señales/acciones predefinidas?
3. ¿Quién puede publicar cambios de lógica además del administrador?
4. ¿La secuencia actual del PLC se importa automáticamente como primer flujo editable?
5. ¿Qué salidas requieren interlock físico además del PLC?

## Orden recomendado

1. Aprobar mapa de E/S y seguridad.
2. Implementar y probar PLC puente.
3. Motor PC en shadow mode.
4. Integración completa.
5. Calibración.
6. Mapa de salidas configurable.
7. Pantalla de lógica (Vista y Edición).
8. Validación y release 2.0.0.
