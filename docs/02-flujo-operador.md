# Flujo del Operador — HiPotApp

## 1. Flujo Completo Paso a Paso

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                         FLUJO DE PRUEBA COMPLETO                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                             ║
║  PASO 1: ESCANEAR MODELO                                                   ║
║  ├─ Operador escanea código de barrera del modelo                          ║
║  ├─ App busca receta en DB por modelo                                      ║
║  ├─ Si no existe → mostrar error "Receta no encontrada"                    ║
║  └─ Si existe → mostrar receta en UI                                       ║
║                                                                             ║
║  PASO 2: ESCANEAR SERIE                                                    ║
║  ├─ Operador escanea número de serie de la unidad                          ║
║  ├─ App valida formato del código                                          ║
║  └─ UI muestra confirmación                                                ║
║                                                                             ║
║  PASO 3: MOSTRAR RECETA                                                    ║
║  ├─ UI muestra: Modelo, Serie, Voltajes, Componentes                       ║
║  ├─ Colores: Verde = habilitado, Gris = deshabilitado                      ║
║  └─ Botón "LISTO" habilitado                                               ║
║                                                                             ║
║  PASO 4: AVISO "TODO LISTO"                                                ║
║  ├─ App muestra: "TODO LISTO - ESCANEE BATCH PARA INICIAR"                 ║
║  ├─ Fondo cambia a amarillo (preparado)                                    ║
║  └─ Esperando escaneo de batch                                             ║
║                                                                             ║
║  PASO 5: ESCANEAR BATCH                                                    ║
║  ├─ Operador escanea su código de batch/usuario                            ║
║  ├─ App crea registro de ejecución (execution_id)                          ║
║  ├─ Registra: operador, batch, fecha/hora inicio                           ║
║  └─ Auto-START: inicia prueba PLC                                          ║
║                                                                             ║
║  PASO 6: PRUEBA PLC (S7-1200)                                              ║
║  ├─ App envía receta al PLC (voltajes, componentes, límites)               ║
║  ├─ PLC ejecuta prueba de consumo                                          ║
║  ├─ App lee tags PLC en tiempo real (cada 200-500ms)                       ║
║  ├─ UI muestra: valores, gráficas, animaciones                             ║
║  ├─ Fondo cambia a azul (PLC corriendo)                                    ║
║  └─ PLC termina → App lee variable de pass/fail                            ║
║                                                                             ║
║  PASO 7: GUARDAR RESULTADOS PLC                                            ║
║  ├─ App guarda en DB: cada lectura, timestamps, pass/fail                  ║
║  ├─ execution.plc_pass_fail = TRUE/FALSE                                   ║
║  └─ UI muestra resumen PLC                                                 ║
║                                                                             ║
║  PASO 8: PRUEBA HIPOT (3865)                                               ║
║  ├─ App envía receta al Hipot (parámetros de prueba)                       ║
║  ├─ App verifica que receta llegó (confirmación)                           ║
║  ├─ App envía comando RUN al Hipot                                         ║
║  ├─ Hipot ejecuta prueba de aislamiento                                    ║
║  ├─ App lee mediciones por serial (voltaje, corriente, resistencia)        ║
║  ├─ UI muestra: valores en tiempo real, gráficas                           ║
║  ├─ Fondo cambia a morado (Hipot corriendo)                                ║
║  └─ Hipot termina → App lee resultado final                                ║
║                                                                             ║
║  PASO 9: GUARDAR RESULTADOS HIPOT                                          ║
║  ├─ App guarda en DB: mediciones, timestamps, pass/fail                    ║
║  ├─ execution.hipot_pass_fail = TRUE/FALSE                                 ║
║  └─ UI muestra resumen Hipot                                               ║
║                                                                             ║
║  PASO 10: EVALUACIÓN FINAL                                                 ║
║  ├─ App calcula: PLC pass + Hipot pass = ?                                 ║
║  ├─ Ambos pasan → final_status = "GOOD"                                    ║
║  ├─ Al menos uno falla → final_status = "BAD"                              ║
║  ├─ Calcula duración total del proceso                                     ║
║  └─ UI muestra: resultado final con animación                              ║
║                                                                             ║
║  PASO 11: GUARDAR COMPLETO                                                 ║
║  ├─ App guarda execution completa en DB                                    ║
║  ├─ Datos: serial, batch, operador, fechas, duraciones, status             ║
║  ├─ Si red disponible → sync a SQL Server remoto                           ║
║  └─ UI muestra: "PRUEBA COMPLETADA - BUENO/MALO"                          ║
║                                                                             ║
║  PASO 12: RESET                                                            ║
║  ├─ UI vuelve a estado IDLE                                                ║
║  ├─ Fondo vuelve a gris                                                    ║
║  └─ Esperando siguiente unidad                                             ║
║                                                                             ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

## 2. Estados de la App y Colores

| Estado | Fondo | Descripción |
|--------|-------|-------------|
| **IDLE** | Gris claro | Esperando escaneo |
| **ESCANEANDO** | Gris oscuro | Procesando código |
| **RECETA CARGADA** | Azul claro | Mostrando receta |
| **LISTO** | Amarillo | Esperando batch para iniciar |
| **PLC CORRIENDO** | Azul | Prueba PLC en progreso |
| **HIPOT CORRIENDO** | Morado | Prueba Hipot en progreso |
| **COMPLETADO BUENO** | Verde | Prueba exitosa |
| **COMPLETADO MALO** | Rojo | Prueba falló |
| **ERROR** | Rojo oscuro | Error en comunicación |

## 3. Secuencia de Escaneos

```
╔══════════════════════════════════════════════════════════════════╗
║                    SECUENCIA DE ESCANEOS                        ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                  ║
║  1. MODELO ──────────▶ 2. SERIE ──────────▶ 3. BATCH           ║
║     │                    │                    │                   ║
║     ▼                    ▼                    ▼                   ║
║  Buscar receta       Validar formato      Crear execution       ║
║  en DB               Guardar serie        Registrar operador    ║
║  Mostrar receta      Mostrar confirmación Auto-START            ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
```

## 4. Manejo de Errores

| Error | Acción | UI |
|-------|--------|-----|
| Receta no encontrada | Mostrar mensaje, pedir re-escaneo | Modal error |
| Barcode inválido | Vibrar (si touch), mostrar error | Toast rojo |
| PLC no responde | Reintentar 3 veces, mostrar estado | Indicador rojo |
| Hipot no responde | Reintentar 3 veces, mostrar estado | Indicador rojo |
| DB local falla | Alerta crítica, parar proceso | Pantalla roja |
| Red cae durante sync | Continuar local, reintentar después | Badge "offline" |
| PLC fail | Guardar resultado, continuar a Hipot | Badge rojo PLC |
| Hipot fail | Guardar resultado, calcular final | Badge rojo Hipot |

## 5. Tiempos Estimados por Paso

| Paso | Tiempo estimado |
|------|-----------------|
| Escaneo modelo | 2-3 segundos |
| Escaneo serie | 2-3 segundos |
| Mostrar receta | 1 segundo |
| Escaneo batch | 2-3 segundos |
| Prueba PLC | 30-120 segundos |
| Prueba Hipot | 15-60 segundos |
| Guardado | 2-3 segundos |
| **Total típico** | **60-180 segundos** |
