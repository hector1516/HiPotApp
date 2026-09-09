# Logging y Debug — HiPotApp

## 1. Propósito

Cada acción de la app queda registrada con timestamp para:
- Debuggear problemas en producción
- Auditar qué hizo cada operador
- Rastrear errores de comunicación PLC/Hipot
- Medir tiempos de proceso
- Reproducir exactamente qué pasó en cada prueba

## 2. Niveles de Log

| Nivel | Cuándo se usa | Ejemplo |
|-------|---------------|---------|
| **DEBUG** | Detalles internos, valores enviados/recibidos | `PLC write DB1.FAN_MIN = 0.3` |
| **INFO** | Acciones normales del usuario y sistema | `Operador escaneó modelo REF-1234` |
| **WARN** | Advertencias que no paran el proceso | `Reconexión automática al PLC` |
| **ERROR** | Fallos que afectan el proceso | `No se pudo conectar al Hipot` |

## 3. Categorías

| Categoría | Qué registra |
|-----------|-------------|
| **UI** | Botones presionados, pantallas, escaneos de barcode |
| **PLC** | Conexión, reads, writes, errores de snap7 |
| **HIPOT** | Conexión, comandos enviados, respuestas recibidas |
| **DB** | Queries, inserts, sync, errores de conexión |
| **SYNC** | Sincronización local→remoto, reintentos |
| **BARCODE** | Código crudo + normalizado (mayúsculas) |
| **SYSTEM** | Arranque de app, cierre, memoria, CPU |

## 4. Ejemplo de Registro

```
┌─────┬──────────────────┬────────┬────────┬──────────────────────────────────────────────────┬──────┐
│ ID  │ Timestamp        │ Level  │ Cat.   │ Detail                                          │ Dur  │
├─────┼──────────────────┼────────┼────────┼──────────────────────────────────────────────────┼──────┤
│ 1   │ 14:30:00.000     │ INFO   │ SYSTEM │ App iniciada - Windows 11 Pro                   │ -    │
│ 2   │ 14:30:01.200     │ INFO   │ PLC    │ Conectando a PLC 192.168.1.10...               │ 150ms │
│ 3   │ 14:30:01.350     │ INFO   │ PLC    │ PLC conectado OK (rack=0, slot=1)               │ -    │
│ 4   │ 14:30:05.100     │ INFO   │ BARCODE│ Scan crudo: "ref-1234" → Normalizado: "REF-1234"│ -    │
│ 5   │ 14:30:05.150     │ INFO   │ DB     │ Receta encontrada: REF-1234 (recipe_id=1)      │ 15ms │
│ 6   │ 14:30:08.200     │ INFO   │ BARCODE│ Scan crudo: "SN-ABC123" → Normalizado: "SN-ABC123"│ -   │
│ 7   │ 14:30:15.000     │ INFO   │ UI     │ Botón START presionado                          │ -    │
│ 8   │ 14:30:15.050     │ DEBUG  │ PLC    │ write DB1.Recipe_Voltage = 120.0                │ 30ms │
│ 9   │ 14:30:15.080     │ DEBUG  │ PLC    │ write DB1.Component_FAN_MIN = 0.3               │ 25ms │
│ 10  │ 14:30:15.110     │ DEBUG  │ PLC    │ write DB1.Component_FAN_MAX = 1.5               │ 28ms │
│ 11  │ 14:30:15.140     │ DEBUG  │ PLC    │ write DB1.Component_FAN_ENABLE = TRUE           │ 22ms │
│ 12  │ 14:30:15.200     │ DEBUG  │ PLC    │ write DB1.Test_Start = TRUE                     │ 30ms │
│ 13  │ 14:30:15.500     │ DEBUG  │ PLC    │ read DB1.Test_Running = TRUE                    │ 25ms │
│ 14  │ 14:30:16.000     │ DEBUG  │ PLC    │ read DB1.Result_FAN = 0.45                      │ 25ms │
│ 15  │ 14:30:16.500     │ DEBUG  │ PLC    │ read DB1.Result_FAN = 0.47                      │ 25ms │
│ ... │ ...              │ ...    │ ...    │ ...                                             │ ...  │
│ 50  │ 14:30:45.000     │ INFO   │ PLC    │ PLC terminó - PASS (duración: 30.0s)            │ -    │
│ 51  │ 14:30:45.050     │ INFO   │ DB     │ Resultados PLC guardados (execution_id=1)      │ 45ms │
│ 52  │ 14:30:45.100     │ INFO   │ HIPOT  │ Conectando a Hipot COM3 9600 baud...           │ 200ms │
│ 53  │ 14:30:45.300     │ INFO   │ HIPOT  │ Hipot conectado OK                              │ -    │
│ 54  │ 14:30:45.350     │ DEBUG  │ HIPOT  │ send: START\r\n                                 │ 15ms │
│ 55  │ 14:30:45.800     │ DEBUG  │ HIPOT  │ recv: V:120.5 I:1.23 R:456.7 PASS              │ 50ms │
│ ... │ ...              │ ...    │ ...    │ ...                                             │ ...  │
│ 80  │ 14:30:55.000     │ INFO   │ HIPOT  │ Hipot terminó - PASS                            │ -    │
│ 81  │ 14:30:55.050     │ INFO   │ DB     │ Resultados Hipot guardados                      │ 35ms │
│ 82  │ 14:30:55.100     │ INFO   │ DB     │ Execution completa: GOOD (total: 55.0s)         │ 20ms │
│ 83  │ 14:30:55.200     │ INFO   │ SYNC   │ Sync a SQL Server remoto: 1 registro enviado    │ 150ms│
└─────┴──────────────────┴────────┴────────┴──────────────────────────────────────────────────┴──────┘
```

## 5. Formato de Log en Código

```python
import logging
from datetime import datetime

class AppLogger:
    def __init__(self, db_session=None):
        self.db = db_session
    
    def log(self, level, category, action, detail,
            execution_id=None, user_id=None,
            value_sent=None, value_received=None,
            duration_ms=None, success=True, error_message=None):
        
        entry = AppLog(
            timestamp=datetime.now(),
            level=level,
            category=category,
            action=action,
            detail=detail,
            execution_id=execution_id,
            user_id=user_id,
            value_sent=value_sent,
            value_received=value_received,
            duration_ms=duration_ms,
            success=1 if success else 0,
            error_message=error_message
        )
        
        if self.db:
            self.db.add(entry)
            self.db.commit()
        
        # También imprime en consola para debug
        print(f"[{level}] [{category}] {action}: {detail}")
    
    # Atajos
    def debug(self, cat, action, detail, **kw):
        self.log("DEBUG", cat, action, detail, **kw)
    
    def info(self, cat, action, detail, **kw):
        self.log("INFO", cat, action, detail, **kw)
    
    def warn(self, cat, action, detail, **kw):
        self.log("WARN", cat, action, detail, **kw)
    
    def error(self, cat, action, detail, error_message=None, **kw):
        self.log("ERROR", cat, action, detail, success=False,
                 error_message=error_message, **kw)
```

## 6. Uso en Cada Módulo

### Barcode Scanner

```python
# Escaneo de modelo
raw = "ref-1234"
normalized = raw.strip().upper()
logger.info("BARCODE", "scan_model",
    f"Crudo: '{raw}' → Normalizado: '{normalized}'")

# Escaneo de serie
raw = "sn-abc123"
normalized = raw.strip().upper()
logger.info("BARCODE", "scan_serial",
    f"Crudo: '{raw}' → Normalizado: '{normalized}'")

# Escaneo de batch
raw = "batch-001"
normalized = raw.strip().upper()
logger.info("BARCODE", "scan_batch",
    f"Crudo: '{raw}' → Normalizado: '{normalized}'")
```

### PLC Communication

```python
# Conexión
logger.info("PLC", "connecting", f"IP: {ip}, rack: {rack}, slot: {slot}")
start = time.time()
success = plc.connect()
duration = int((time.time() - start) * 1000)
if success:
    logger.info("PLC", "connected", f"PLC {ip} conectado", duration_ms=duration)
else:
    logger.error("PLC", "connect_failed", f"No se pudo conectar a {ip}",
                 error_message=str(e), duration_ms=duration)

# Read
start = time.time()
value = plc.read_real(db, offset)
duration = int((time.time() - start) * 1000)
logger.debug("PLC", "read", f"DB{db}.Offset{offset} = {value}",
             duration_ms=duration, value_received=str(value))

# Write
start = time.time()
plc.write_real(db, offset, value)
duration = int((time.time() - start) * 1000)
logger.debug("PLC", "write", f"DB{db}.Offset{offset} = {value}",
             duration_ms=duration, value_sent=str(value))
```

### Hipot Communication

```python
# Conexión
logger.info("HIPOT", "connecting", f"Puerto: {port}, Baud: {baud}")
success = hipot.connect()
if success:
    logger.info("HIPOT", "connected", f"Hipot {port} conectado")
else:
    logger.error("HIPOT", "connect_failed", f"No se pudo conectar a {port}")

# Comando
logger.debug("HIPOT", "send_command", f"Comando: {command}")
hipot.send_command(command)

# Respuesta
response = hipot.read_response()
logger.debug("HIPOT", "receive", f"Respuesta: {response}",
             value_received=response)
```

### UI Actions

```python
# Botón presionado
logger.info("UI", "button_press", "Botón START presionado")

# Pantalla cambiada
logger.info("UI", "screen_change", f"De {old_screen} a {new_screen}")

# Valor actualizado en UI
logger.debug("UI", "value_update",
    f"FAN: {old_value}A → {new_value}A ({status})")
```

## 7. Pantalla de Logs en Admin

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  📋 LOG DE APLICACIÓN                                    [🔄 Actualizar]  │
├─────────────────────────────────────────────────────────────────────────────┤
│  Filtrar: [Categoría: TODAS ▼] [Nivel: TODOS ▼] [Fecha: HOY ▼]          │
│  Buscar: [________________________________] [🔍]                          │
├────┬──────────────┬───────┬────────┬────────────────────────────────────────┤
│ #  │ Timestamp    │ Level │ Cat.   │ Detail                                │
├────┼──────────────┼───────┼────────┼────────────────────────────────────────┤
│ 83 │ 14:30:55.200 │ INFO  │ SYNC   │ Sync a SQL Server: 1 reg enviado      │
│ 82 │ 14:30:55.100 │ INFO  │ DB     │ Execution GOOD (total: 55.0s)         │
│ 81 │ 14:30:55.050 │ INFO  │ DB     │ Resultados Hipot guardados            │
│ 80 │ 14:30:55.000 │ INFO  │ HIPOT  │ Hipot terminó - PASS                  │
│ 55 │ 14:30:45.800 │ DEBUG │ HIPOT  │ recv: V:120.5 I:1.23 PASS            │
│ 54 │ 14:30:45.350 │ DEBUG │ HIPOT  │ send: START\r\n                       │
│ 53 │ 14:30:45.300 │ INFO  │ HIPOT  │ Hipot conectado OK                    │
│ 52 │ 14:30:45.100 │ INFO  │ HIPOT  │ Conectando a COM3...                 │
│ 51 │ 14:30:45.050 │ INFO  │ DB     │ Resultados PLC guardados             │
│ 50 │ 14:30:45.000 │ INFO  │ PLC    │ PLC terminó - PASS (30.0s)           │
│ ...│ ...          │ ...   │ ...    │ ...                                   │
│ 1  │ 14:30:00.000 │ INFO  │ SYSTEM │ App iniciada                         │
├────┴──────────────┴───────┴────────┴────────────────────────────────────────┤
│  Mostrando 83 de 83 registros                [◀] Página 1 de 1 [▶]        │
│  [📥 Exportar CSV]  [🗑️ Limpiar logs antiguos (>30 días)]                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 8. Resumen de la Tabla

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | INT PK | ID auto-incremental |
| `timestamp` | DATETIME2 | Fecha/hora del evento |
| `level` | NVARCHAR(10) | INFO, WARN, ERROR, DEBUG |
| `category` | NVARCHAR(50) | UI, PLC, HIPOT, DB, SYNC, BARCODE, SYSTEM |
| `action` | NVARCHAR(100) | Acción específica (button_press, scan, read, write) |
| `detail` | NVARCHAR(MAX) | Descripción legible |
| `execution_id` | INT FK | Vincula a la ejecución activa (NULL si no aplica) |
| `user_id` | INT FK | Operador (NULL si es sistema) |
| `value_sent` | NVARCHAR(200) | Dato enviado |
| `value_received` | NVARCHAR(200) | Dato recibido |
| `duration_ms` | INT | Duración de la operación |
| `success` | BIT | 1=éxito, 0=fallo |
| `error_message` | NVARCHAR(500) | Descripción del error |

**Total de tablas: 14** (12 originales + app_logs + ui_state_log)
