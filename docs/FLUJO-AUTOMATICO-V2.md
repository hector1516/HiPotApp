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
| `hipot_voltage` | Voltaje de prueba (V) del paso ACW |
| `hipot_current_max` | Límite de corriente en **mA** (ACW) |
| `hipot_time` | Tiempo de prueba / Dwell en segundos |

Antes de cada prueba, `run_test_cycle` lee estos valores y crea un paso ACW en el 3865 con:

```text
ADD ACW,<voltaje>,<mA>,0,0.1,<dwell>,0.0,5,OFF,60,OFF,1.50,0.00,0.00
```

Después envía `SS 1` y `TEST`. Si falta voltaje, corriente o tiempo, el ciclo Hipot falla y **no se ejecuta la prueba eléctrica**.

## Ejecución y resultado Hipot

- `TEST` inicia el paso ACW seleccionado.
- La app consulta `TD?` cada 500 ms.
- `TD?` devuelve: `<paso>, <tipo>, <estado>, <voltaje>, <corriente>, <tiempo>`.
- Para ACW, el voltaje viene en kV y la corriente en mA; la app convierte el voltaje a V para guardarlo.
- `PASS` = aprobado. Cualquier otro estado terminal (`HI-LMT`, `LO-LMT`, `OTP`, `CONT-F`, `Abort`, `OUT-ERROR`, etc.) = fallado.
- `Dwell`, `Ramp` y `Delay` son estados en proceso.
- Al terminar o al vencer el timeout (dwell + 30 s), la app envía `RESET`.

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

La prueba de comunicación de **Configuración → Hipot → Probar Hipot** envía `*IDN?` y exige respuesta del equipo (no solo abre el puerto).

## Paro de emergencia

- Entrada PLC configurable (`estop_input`, por defecto `I0.0`).
- Tipo de contacto configurable (`estop_nc`: `1` = NC, `0` = NO).
- Se consulta cada 800 ms.
- Al activarse:
  - Se aborta el PLC y se envía `RESET` al Hipot.
  - Overlay rojo a pantalla completa, animado, con flecha inferior apuntando al botón físico.
  - Bloquea la interacción hasta que la entrada se libere.
- Si el PLC no responde, no se muestra la alarma roja; se indica “Paro de emergencia no disponible”.

## Modo manual

- Requiere el mismo gafete de admin que Configuración.
- Si el gafete es de operador, se rechaza.
- El acceso queda registrado en `app_logs`.
