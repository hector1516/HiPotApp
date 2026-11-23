# Mapa de E/S confirmado — PLC puente DB100

> Fuente: tabla de I/O del tablero y proyecto `HUSSMANN - HiPot_20261123_1614.zip`.
> La app usará `DB100` como interfaz única. No se usarán DB1000/DB1001.

## DB100

| Offset | Campo | Tipo | Contenido |
|---|---|---|---|
| 0.0 | `IB0` | Byte | Entradas digitales 0 |
| 1.0 | `IB1` | Byte | Entradas digitales 1 |
| 2.0 | `IB2` | Byte | Entradas digitales 2 |
| 3.0 | `IB3` | Byte | Entradas digitales 3 |
| 4.0 | `QB0` | Byte | Salidas digitales 0 |
| 5.0 | `QB1` | Byte | Salidas digitales 1 |
| 6.0 | `QB2` | Byte | Salidas digitales 2 |
| 7.0 | `QB3` | Byte | Salidas digitales 3 |
| 8.0 | `IW100` | Word | Analógica 1 |
| 10.0 | `IW102` | Word | Analógica 2 |
| 12.0 | `IW104` | Word | Analógica 3 |
| 14.0 | `IW106` | Word | Analógica 4 |

La DB es no optimizada y permite direccionamiento por offset.

## Salidas digitales

### `QB0` — byte 4.0

| Bit | Dirección | Símbolo | Función |
|---|---|---|---|
| 0 | Q0.0 | KM2 | Contactor 120 VAC |
| 1 | Q0.1 | KM1 | Contactor 220 VAC |
| 2 | Q0.2 | K2 | SSR A4, herramienta 0–20 A |
| 3 | Q0.3 | K8 | Relé A2, herramienta 0–1 A |
| 4 | Q0.4 | K9 | Relé A3, herramienta 0–10 A |
| 5 | Q0.5 | K3 | L de FAN |
| 6 | Q0.6 | K4 | L de LIGHT |
| 7 | Q0.7 | K5 | L de FRAME anti-sweat |

### `QB1` — byte 5.0

| Bit | Dirección | Símbolo | Función |
|---|---|---|---|
| 0 | Q1.0 | K6 | L de DOOR anti-sweat |
| 1 | Q1.1 | K7 | L de DRAIN HEATERS |
| 2-7 | Q1.2-Q1.7 | — | Libres |

### `QB2` — byte 6.0

| Bit | Dirección | Símbolo | Función |
|---|---|---|---|
| 0 | Q2.0 | K1 | L de HEATERS |
| 1 | Q2.1 | — | Libre |
| 2 | Q2.2 | — | Libre |
| 3-7 | Q2.3-Q2.7 | — | Libres |

### `QB3` — byte 7.0

Sin salidas asignadas actualmente.

## Entradas digitales

### `IB0` — byte 0.0

| Bit | Dirección | Función |
|---|---|---|
| 0 | I0.0 | Paro de emergencia |
| 1 | I0.1 | NA KM1 (contactor 220 VAC) |
| 2 | I0.2 | NA KM2 (contactor 110/120 VAC) |
| 3-7 | I0.3-I0.7 | Libres |

### `IB1` a `IB3`

Sin entradas asignadas actualmente.

## Entradas analógicas

| Canal | Rango | Módulo | Campo DB100 |
|---|---|---|---|
| IA1 | 0–1 A | A2 `CML-10` | `IW100` (confirmar) |
| IA2 | 0–10 A | A3 `CML-10` | `IW102` (confirmar) |
| IA3 | 0–20 A | A4 `CMT-30` | `IW104` (confirmar) |
| IA4 | 0–220 VAC | A1 `VM-AO-S` | `IW106` (confirmar) |

Pendiente: confirmar con `FC1` que los campos `IW100/102/104/106` correspondan en ese orden a IA1/IA2/IA3/IA4.

## Lógica de software

- La app lee `IB0..IB3` y `IW100..IW106`.
- La app escribe `QB0..QB3`.
- El paro es eléctrico; la app solo lee el bit configurado de `IB`.
- La secuencia y las recetas se ejecutan en la PC.
- El PLC solo copia E/S y aplica watchdog.

## Pendiente crítico

Agregar a `DB100`:

- `Heartbeat` (DInt o Byte) en un offset libre.
- `PcOnline` / `WatchdogOk`.
- Lógica en el PLC que ponga `QB0..QB3 = 0` si el heartbeat no cambia en 500 ms.
