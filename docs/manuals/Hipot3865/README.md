# Hipot 3865 — documentación oficial descargada

Fuente: <https://www.arisafety.com/model-3865.html>  
Fecha de descarga: 2026-11-19

## Archivos descargados (locales)

Los PDF, MSI y ZIP son archivos grandes y **no se suben a GitHub** (ver `.gitignore`). Quedan disponibles en esta carpeta para consulta local.

| Archivo | Contenido |
|---|---|
| `Hypot_Manual.pdf` | Manual completo del Hypot 3800/3865 (94 páginas). Incluye Apéndice D, protocolo USB/RS-232. |
| `Hypot3800_QSG.pdf` | Guía rápida de la serie 3800. |
| `AR_Hypot_Specs.pdf` | Especificaciones de la serie Hypot. |
| `EN_50191_Compliance.pdf` | Cumplimiento EN 50191. |
| `AR_Catalog.pdf` | Catálogo general Associated Research. |
| `H4_IVI_Driver_Getting_Started_Guide.pdf` | Guía de integración IVI (C#, C++, Python, LabVIEW). |
| `drivers/AR_IVI_Driver_64bit.msi` | Driver IVI 64 bits para Hypot 3800/3865. |
| `drivers/AR_IVI_Driver_32bit.msi` | Driver IVI 32 bits. |
| `drivers/AR_USB_Driver.zip` | Driver USB/RS-232 (FTDI). |
| `drivers/AR_38XX_LabVIEW_Driver.zip` | Driver LabVIEW serie 38XX. |

Textos extraídos para búsqueda: `*.txt` en esta misma carpeta.

## Protocolo real del 3865 (Apéndice D del manual)

### Puerto serie

| Parámetro | Valor |
|---|---|
| Baudios | **38400** |
| Bits de datos | 8 |
| Parada | 1 |
| Paridad | Ninguna |
| Handshaking | Ninguno (sin XON/XOFF) |
| Terminador de comando | **NL / LF (0x0A)** |
| Respuesta a comando | **ACK (0x06)** o **NAK (0x15)** |
| Respuesta a consulta | ACK/NAK + datos, terminados en NL |

El puerto USB del 3865 se presenta en Windows como un puerto COM virtual (USB-to-RS-232). La tarjeta RS-232 opcional usa el mismo protocolo.

### Comandos principales

| Comando | Función |
|---|---|
| `*IDN?` | Identificación: empresa, modelo, serie, firmware |
| `TEST` | Inicia la secuencia en el paso seleccionado |
| `RESET` | Detiene/aborta la prueba y limpia fallas |
| `TD?` | Lee la medición/estado actual o el último resultado |
| `RD <paso>?` | Lee el resultado guardado de un paso |
| `LF?` | Nombre del archivo activo en RAM |
| `FL <memoria>` | Carga un archivo de memoria a RAM |
| `FN <memoria>,<nombre>` | Crea un archivo nuevo (máx. 8 caracteres) |
| `ST?` | Total de pasos |
| `SS <paso>` | Selecciona el paso activo |
| `SD <paso>` | Borra un paso |
| `ADD <tipo>,<parámetros...>` | Agrega un paso completo |
| `EV`, `EH`, `EL`, `ERU`, `EDW`, `ERD`, etc. | Edición individual de parámetros |
| `*OPC?` | Consulta si terminó la operación pendiente |

### Formato de `TD?`

```text
<paso>, <tipo>, <estado>, <medidor1>, <medidor2>, <medidor3>
```

Ejemplos del manual:

```text
1, ACW, PASS, 0.30, 0.296, 0.5
2, ACW, HI-LMT, 0.30, 0.192, 0.2
3, ACW, Abort, 0.30, 0.296, 0.2
1, ACW, Dwell, 0.30, 0.296, 0.5
```

Interpretación:

| Tipo | Medidor 1 | Medidor 2 | Medidor 3 |
|---|---|---|---|
| ACW | Voltaje en kV | Corriente en mA | Tiempo en s |
| DCW | Voltaje en kV | Corriente en µA | Tiempo en s |
| IR | Voltaje en V | Resistencia en MΩ | Tiempo en s |

Estados de proceso: `Dwell`, `Ramp`, `Delay`.  
Estados terminales: `PASS`, `HI-LMT`, `LO-LMT`, `OTP`, `CONT-F`, `Abort`, `OUT-ERROR`, etc.

### Secuencia recomendada para la app

1. `*IDN?` para verificar comunicación.
2. Cargar un archivo de trabajo: `FL <memoria>`.
3. Borrar pasos existentes: `ST?` y `SD 1` repetidamente.
4. Agregar el paso ACW con `ADD ACW,...`.
5. Seleccionar el paso: `SS 1`.
6. Iniciar: `TEST`.
7. Consultar `TD?` y/o `*OPC?` hasta estado terminal o timeout.
8. Detener/limpiar: `RESET`.

Ejemplo de paso ACW del manual:

```text
ADD ACW,1240,0.10,0.010,0.1,1.0,0.0,5,OFF,60,OFF,1.50,0.00,0.00
```

Orden de parámetros ACW:

```text
Voltaje(V), HI-Limit(mA), LO-Limit(mA), RampUp(s), Dwell(s), RampDown(s),
ArcSense, ArcDetect, Frecuencia(Hz), Continuidad, ContHI(Ω), ContLO(Ω), ContOffset(Ω)
```

### Advertencias del manual

- La memoria no volátil tiene ciclos de escritura limitados. Evitar `FS`, `FSA`, `FN`, `FD`, `SAO`, `SSG2` en cada ciclo. Para producción, crear el archivo una vez y usar `FL` + `ADD` en RAM.
- Si la seguridad del instrumento está en un nivel restringido, los comandos de edición pueden responder NAK. Para control total por bus se requiere nivel de edición adecuado.
- El manual recomienda no ajustar parámetros individuales repetidamente; usar `ADD`/`FL` para el flujo de producción.
- Verificar la configuración **PLC Remote** (`SPR`). Si el equipo está configurado para iniciar solo por la entrada remota del panel trasero, el comando `TEST` por bus puede no arrancar la prueba. En ese caso, probar `SPR 0` una vez desde Configuración → Hipot → Comando manual, o iniciar por el modo remoto correspondiente.

### Implementación en HiPotApp

- La app usa **ACW**.
- Archivo temporal: `HIPOTAPP`, número de memoria `2000` (si está ocupado prueba 1999/1998).
- Antes de cada prueba: `RESET`, `FL`, borra pasos con `ST?`/`SD 1`, `ADD ACW,...`, `SS 1`.
- Inicio: `TEST`.
- Seguimiento: `TD?` cada 500 ms.
- Fin/aborto: `RESET`.
- Prueba de comunicación: `*IDN?`.
- Log TX/RX en `app_logs` categoría `HIPOT`.

## Corrección importante respecto a la documentación anterior

La documentación interna anterior (`docs/06-comunicaciones.md`) describía comandos que **no existen** en el 3865:

- `START`, `STOP`, `ABORT`
- `VOLTAGE:`, `CURRENT_MAX:`, `TIME:`
- `STATUS?`, `MEASURE?`, `RESULT?`
- Respuestas tipo `V:120.5 I:1.23 R:456.7 PASS`

Esos comandos eran una suposición. El protocolo real es el de este documento: `TEST`, `RESET`, `TD?`, `ADD`, ACK/NAK, 38400 8N1, terminador LF.

La implementación Rust se actualizó para usar el protocolo real. La validación final requiere el equipo físico.
