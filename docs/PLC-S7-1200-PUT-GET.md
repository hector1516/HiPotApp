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

1. Crea un DB global nuevo, por ejemplo `DB100` (`DB_HipotApp`).
2. Verifica que el DB sea **no optimizado** (Properties → Attributes → desactivar “Optimized block access”).
3. Agrega un Bool en:

```text
DB100.DBX0.0
```

> Importante: en el respaldo del PLC, `DB1` es `RDM INT` (optimizado, 24 bytes), no la receta.
> No uses `DB1.DBX105.0`. La receta real está en `DB2` y las mediciones en `DB3`, pero ambos
> son optimizados y no se pueden leer por PUT/GET sin cambiarlos.

4. En el programa del PLC (OB1), copia la entrada:

```pascal
"DB100".Estop_Raw := "I0.0";
```

Con el paro NC:

```text
Liberado   → I0.0 = 1 → DB100.DBX0.0 = 1
Presionado → I0.0 = 0 → DB100.DBX0.0 = 0
```

5. Descarga el programa y la configuración de hardware.

### En la app

**Configuración → Sistema → Paro de emergencia**:

- Fuente: **DB del PLC**
- DB: `100`
- Byte offset: `0`
- Bit: `0`
- Tipo de contacto: **NC**
- Guardar

La app leerá `DB100.DBX0.0` y ya no dependerá de leer `I0.0` por S7.

## Hallazgos del respaldo del PLC

El respaldo `HUSSMANN - HiPot_V20` muestra:

- `E-STOP` = `%I0.0` (no existe un DB de paro).
- `DB1` = `RDM INT`, optimizado, 24 bytes. **No es la receta.**
- `DB2` = `RECIPE` (receta real).
- `DB3` = `MEASURE_DB` (DB de instancia de `MEASURE`).
- `DB2` y `DB3` están configurados como **optimizados / solo simbólicos**.
- Por eso la app no puede usar `DB1` ni leer/escribir `DB2`/`DB3` por offset absoluto.

Antes de ejecutar pruebas automáticas hay que crear una **DB de interfaz no optimizada**
(por ejemplo `DB100`) y copiar allí receta, órdenes, resultados y paro desde el programa PLC.
El mapeo actual de la app (`DB1` offset 0/4/8/...) corresponde a otro programa y no aplica a este respaldo.

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
