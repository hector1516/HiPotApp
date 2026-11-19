# Flujo automático v2 — HiPotApp

## Pasos

| # | Paso | Acción |
|---|---|---|
| 1 | Modelo | Se elige de la lista de recetas. Se recuerda el último modelo usado. |
| 2 | Serie | Se escanea. |
| 3 | Batch | Se escanea y crea la ejecución. |
| 4 | Hipot | Primero. Detecta cortos/aislamiento. |
| 5 | Eléctrica | Solo si Hipot pasó. PLC mide consumo. |
| 6 | Resultado | GOOD solo si Hipot y Eléctrica pasan. |

## Reglas

- El modelo **no se escanea**. Se selecciona de las recetas dadas de alta.
- El último modelo se guarda en `localStorage` (`last_model`) y se restaura al abrir la app.
- Cuando exista la BD externa serial → modelo, el comando `resolve_model_by_serial` la consultará y podrá sobrescribir la selección manual.
- Si el **Hipot falla**, no se ejecuta la prueba eléctrica y el resultado es `BAD`.
- Si el Hipot pasa, se ejecuta la eléctrica y el resultado final es `GOOD` solo si ambas pasan.
- La etiqueta Zebra se imprime solo con `GOOD`.

## Parámetros Hipot por receta

Cada receta guarda:

| Campo | Uso |
|---|---|
| `hipot_voltage` | Voltaje de prueba que se envía con `VOLTAGE:<valor>` |
| `hipot_current_max` | Límite de corriente que se envía con `CURRENT_MAX:<valor>` |
| `hipot_time` | Tiempo que se envía con `TIME:<valor>` |

Antes de cada `START`, `run_test_cycle` lee estos valores de la receta y los envía al Hipot. Si un valor está vacío o en 0, no se envía ese comando.

## Log de comandos Hipot

Cada TX/RX del Hipot se guarda en `app_logs` con:

- Categoría `HIPOT`
- Acción `tx_rx`
- `value_sent`: comando enviado
- `value_received`: respuesta cruda
- `duration_ms`: tiempo de respuesta
- `error_message`: error si falló

Se puede ver en:

- **Logs → categoría HIPOT**
- **Configuración → Hipot → Últimos comandos Hipot**

## Paro de emergencia

- Entrada PLC configurable (`estop_input`, por defecto `I0.0`).
- Tipo de contacto configurable (`estop_nc`: `1` = NC, `0` = NO).
- Se consulta cada 800 ms.
- Al activarse:
  - Se aborta PLC y Hipot.
  - Overlay rojo a pantalla completa, animado, con flecha inferior apuntando al botón físico.
  - Bloquea la interacción hasta que la entrada se libere.
- Si el PLC no responde, no se muestra la alarma roja; se indica “Paro de emergencia no disponible”.

## Modo manual

- Requiere el mismo gafete de admin que Configuración.
- Si el gafete es de operador, se rechaza.
- El acceso queda registrado en `app_logs`.
