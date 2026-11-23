# Watchdog del PLC puente — propuesta para implementar

> **Estado:** pendiente. La app v1.1.0 todavía no envía heartbeat y el `DB100`
> actual no lo usa. No liberar a producción hasta implementar y probar esto.

## Riesgo

Si la PC se cuelga, se cierra o pierde la red durante un ciclo, las salidas
`QB0..QB3` pueden quedar energizadas. El paro eléctrico no cubre una caída de la
PC; se necesita un watchdog independiente en el PLC.

## Extensión propuesta de DB100

Mantener los offsets actuales y extender la DB a 24 bytes:

| Offset | Campo | Tipo | Uso |
|---|---|---|---|
| 0 | `IB0..IB3` | 4 bytes | Entradas digitales |
| 4 | `QB0..QB3` | 4 bytes | Salidas digitales |
| 8 | `IW100..IW106` | 4 words | Analógicas crudas |
| 16 | `Heartbeat` | DInt | La PC incrementa cada 100 ms |
| 20.0 | `PcOnline` | Bool | PC viva |
| 20.1 | `WatchdogOk` | Bool | Salidas permitidas |
| 20.2 | `Estop` | Bool | Copia del paro (opcional) |

La app ya escribe `QB0..QB3`; solo falta escribir `Heartbeat` y leer
`WatchdogOk`/`PcOnline` para diagnóstico.

## Lógica mínima en OB1/FC

```scl
// Variables estáticas del FB/FC:
//   lastHeartbeat : DInt
//   lastChange    : Time
//   tNow          : Time

tNow := Time_TCK();

IF "DB100".Heartbeat <> #lastHeartbeat THEN
    #lastHeartbeat := "DB100".Heartbeat;
    #lastChange := #tNow;
    "DB100".PcOnline := TRUE;
    "DB100".WatchdogOk := TRUE;
END_IF;

IF (#tNow - #lastChange) > T#500ms THEN
    "DB100".PcOnline := FALSE;
    "DB100".WatchdogOk := FALSE;
    "DB100".QB0 := 0;
    "DB100".QB1 := 0;
    "DB100".QB2 := 0;
    "DB100".QB3 := 0;
END_IF;

// Copia normal:
"DB100".IB0 := "IB0";
...
"QB0" := "DB100".QB0;
...
```

Ajustar `T#500ms` con el especialista Siemens. El heartbeat debe ser
independiente de la UI y del ciclo de la PC.

## Lado PC (pendiente)

Agregar un hilo de heartbeat cada 100 ms que:

- Escriba un contador en `DB100.DBD16` si el PLC está habilitado.
- Lea `PcOnline` y `WatchdogOk` para mostrarlos en Live/Admin.
- No escriba salidas desde ese hilo.
- Se detenga limpiamente al cerrar la app.

No implementarlo hasta confirmar el offset y el tamaño de DB100 en el proyecto
TIA actualizado.

## Criterios de aceptación

1. Desconectar la PC o matar el proceso: `QB0..QB3` caen a 0 en ≤ 500 ms.
2. Detener el heartbeat sin cerrar la app: mismas salidas seguras.
3. El paro eléctrico sigue cortando energía de forma independiente.
4. Rearrancar la PC no reactiva salidas automáticamente.
5. Prueba en banco firmada por el especialista Siemens.
