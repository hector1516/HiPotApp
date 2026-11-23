# PLC S7-1200: conecta pero no lee variables

## Síntoma

En la app:

- El PLC aparece como conectado por TCP.
- El **Monitor PLC (IO)** no muestra entradas verdes.
- El paro de emergencia se queda activo o no se puede leer.
- En v1.0.38 aparece un error como:

```text
Conectó al PLC pero no pudo leer I0.0: ...
```

Eso significa que el puerto 102 responde y el handshake S7 funciona, pero el PLC **rechaza las lecturas**.

## Causa más común en S7-1200: PUT/GET deshabilitado

En TIA Portal:

1. Abre el dispositivo PLC.
2. **Properties** → **Protection & Security**.
3. En **Connection mechanisms**, activa:
   - **Permit access with PUT/GET communication from remote partner**
4. Compila y descarga la configuración de hardware al PLC.
5. Reinicia la app y prueba de nuevo.

Sin esa opción, el cliente S7 puede conectar pero las lecturas/escrituras fallan.

## S7-1200: leer el paro desde un DB

En muchos S7-1200/1500, PUT/GET permite leer **DBs**, pero no las áreas `I/Q`. Si al leer `I0.0` aparece:

```text
CPU : Item not available
```

la solución es copiar la entrada a un bit de DB y leer ese DB.

### En TIA Portal

No hace falta crear un DB nuevo. Usa `DB1001`, que ya es no optimizada:

1. En `COMM` (FC1), red 6 `UNIT_EXT`, agrega la copia directa del paro:

```pascal
"COMM01_UNIT_EXT".UNIT_0025_EXT := "E-STOP";
```

2. Confirma que `DB1001` sea **no optimizada**.
3. Descarga el programa.

Con el paro NC:

```text
Liberado   → I0.0 = 1 → DB1001.DBX2.5 = 1
Presionado → I0.0 = 0 → DB1001.DBX2.5 = 0
```

> Si en el PLC se usa `NOT "E-STOP"`, el bit queda invertido:
> liberado = 0, presionado = 1. En ese caso la app debe usar **NO**, no NC.
> Lo recomendado es dejar la copia directa y usar NC en la app.

### En la app

**Configuración → Sistema → Paro de emergencia**:

- Fuente: **DB del PLC**
- DB: `1001`
- Byte offset: `2`
- Bit: `5`
- Tipo de contacto: **NC**
- Guardar

La app leerá `DB1001.DBX2.5` y ya no dependerá de leer `I0.0` por S7.

## Interfaz real de la app: DB1000 y DB1001

El programa del PLC ya tiene dos DB **no optimizadas** para comunicación con la app:

### DB1000 — `COMM01_EXT_UNIT` (receta y órdenes)

| Dirección | Tipo | Uso |
|---|---|---|
| `DB1000.DBX2.5` | Bool | `start test` |
| `DB1000.DBX2.6` | Bool | `stop test` |
| `DB1000.DBD4` | Real | Bitmask de receta (18 bits + 110VAC) |
| `DB1000.DBD8` | Real | FAN Time (s) |
| `DB1000.DBD12` | Real | DOOR Time (s) |
| `DB1000.DBD16` | Real | FRAME Time (s) |
| `DB1000.DBD86` | Real | DRAIN Time (s) |
| `DB1000.DBD90` | Real | DEFROST Time (s) |
| `DB1000.DBD94` | Real | LIGHT Time (s) |
| `DB1000.DBD98` | Real | Tolerancia de voltaje (%) |

Bitmask de receta (`DB1000.DBD4`):

| Bit | Componente/canal |
|---|---|
| 0/1/2 | FAN A/B/C |
| 3/4/5 | DOOR A/B/C |
| 6/7/8 | FRAME A/B/C |
| 9/10/11 | DRAIN A/B/C |
| 12/13/14 | DEFROST A/B/C |
| 15/16/17 | LIGHT A/B/C |
| 18 | 110VAC (0 = 220VAC) |

La app elige un solo canal por componente según el máximo de corriente de la receta:

- `max <= 1 A` → A
- `max <= 10 A` → B
- `max > 10 A` → C

### DB1001 — `COMM01_UNIT_EXT` (resultados)

| Dirección | Tipo | Uso |
|---|---|---|
| `DB1001.DBX2.1` | Bool | `Measuring` |
| `DB1001.DBX2.2` | Bool | `Finished` |
| `DB1001.DBX2.4` | Bool | `Error` (mapeado desde `MEASURE_DB.Error`) |
| `DB1001.DBX2.5` | Bool | E-stop (mapeado desde `"E-STOP"`) |
| `DB1001.DBD4` | Real | Resultado FAN |
| `DB1001.DBD8` | Real | Resultado DOOR |
| `DB1001.DBD12` | Real | Resultado FRAME |
| `DB1001.DBD16` | Real | Resultado DRAIN |
| `DB1001.DBD86` | Real | Resultado DEFROST |
| `DB1001.DBD90` | Real | Resultado LIGHT |
| `DB1001.DBD94` | Real | Voltaje medido (requiere mapeo en PLC) |

Para que la app reciba el error de voltaje, agrega en `COMM` (FC1), red 6:

```pascal
"COMM01_UNIT_EXT".UNIT_0024_EXT := "MEASURE_DB".Error;
"COMM01_UNIT_EXT".UNIT_0940_EXT := "MEASURE_DB".Result.VOLTAGE;
```

Si no agregas esas líneas, la app solo evalúa la corriente contra los límites de la receta.

## Hallazgos del respaldo del PLC

El respaldo `HUSSMANN - HiPot_V20` muestra:

- `E-STOP` = `%I0.0` (no existe un DB de paro).
- `DB1` = `RDM INT`, optimizado, 24 bytes. **No es la receta.**
- `DB2` = `RECIPE` (receta real).
- `DB3` = `MEASURE_DB` (DB de instancia de `MEASURE`).
- `DB2` y `DB3` están configurados como **optimizados / solo simbólicos**.
- Por eso la app no puede usar `DB1` ni leer/escribir `DB2`/`DB3` por offset absoluto.

La app usa **DB1000/DB1001** para receta, órdenes, resultados y paro. No usa DB1/DB2/DB3.
El mapeo anterior de la app (`DB1` offset 0/4/8/...) correspondía a otro programa y ya fue reemplazado.

## Otras verificaciones

| Punto | Valor correcto |
|---|---|
| IP del PLC | `10.188.141.10` |
| Rack | `0` |
| Slot | `1` (S7-1200) |
| Puerto | `102` |
| Ping | Debe responder |
| Firewall | Puerto 102 permitido |

## Direcciones que lee la app

La app lee entradas físicas:

```text
I0.0  → Paro de emergencia
I0.1  → NA KM1 (220VAC)
I0.2  → NA KM2 (110VAC)
Q0.0 ... Q2.2 → salidas
IA1 ... IA4 → analógicas
```

Si el paro de emergencia está en un **DB** o en otra entrada, cambia `estop_input` en la app y/o ajusta el programa del PLC.

## Cómo probar

1. En la app: **Configuración → PLC → Habilitar PLC → Guardar Red PLC**.
2. Pulsa **Probar Conexión**.
3. Debe decir **PLC conectado** y mostrar el valor de `I0.0`.
4. Ve a **Monitor PLC (IO)**; las entradas deben cambiar en verde al activar/desactivar.
5. Si sigue el error, revisa el texto de “Detalles técnicos”; allí aparece el error S7 real.

## Nota

La app no programa el PLC. Solo lee/escribe direcciones S7. El programa del PLC y la configuración de seguridad del CPU son responsabilidad de TIA Portal.
