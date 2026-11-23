# Comunicaciones — HiPotApp

## 1. Resumen de Protocolos

| Dispositivo | Protocolo | Librería Python | Puerto | Velocidad |
|-------------|-----------|-----------------|--------|-----------|
| **PLC S7-1200** | S7/TCP | snap7 | Ethernet 102 | ~100ms/ciclo |
| **Hipot 3865** | RS-232 Serial | pyserial | COM3 (USB) | 9600-115200 baud |
| **Barcode Scanner** | USB HID | keyboard input | USB | Instantáneo |
| **SQL Express** | TCP/IP | pymssql/pyodbc | 1433 | ~10ms/query |
| **SQL Server Remoto** | TCP/IP | pymssql/pyodbc | 1433 | ~50-200ms/query |

## 2. PLC S7-1200 (snap7)

### Configuración

```python
import snap7
from snap7.util import get_real, set_real, get_bool, set_bool

class PLCManager:
    def __init__(self, ip='192.168.1.10', rack=0, slot=1):
        self.client = snap7.client.Client()
        self.ip = ip
        self.rack = rack
        self.slot = slot
        self.connected = False
    
    def connect(self):
        try:
            self.client.connect(self.ip, self.rack, self.slot)
            self.connected = True
            return True
        except Exception as e:
            self.connected = False
            return False
    
    def disconnect(self):
        if self.connected:
            self.client.disconnect()
            self.connected = False
    
    def read_tag(self, db_number, start, size):
        """Lee datos del DB del PLC"""
        data = self.client.db_read(db_number, start, size)
        return data
    
    def write_tag(self, db_number, start, data):
        """Escribe datos al DB del PLC"""
        self.client.db_write(db_number, start, data)
    
    def read_real(self, db_number, start):
        """Lee un valor REAL (4 bytes)"""
        data = self.client.db_read(db_number, start, 4)
        return get_real(data, 0)
    
    def write_real(self, db_number, start, value):
        """Escribe un valor REAL"""
        data = bytearray(4)
        set_real(data, 0, value)
        self.client.db_write(db_number, start, data)
    
    def read_bool(self, db_number, start, bit):
        """Lee un valor BOOL"""
        data = self.client.db_read(db_number, start, 1)
        return get_bool(data, 0, bit)
    
    def write_bool(self, db_number, start, bit, value):
        """Escribe un valor BOOL"""
        data = bytearray(1)
        set_bool(data, 0, bit, value)
        self.client.db_write(db_number, start, data)
    
    def test_connection(self):
        """Prueba si el PLC responde"""
        try:
            data = self.client.db_read(1, 0, 1)
            return True
        except:
            return False
```

### Mapeo real de la app (DB1000 / DB1001)

La app usa las DB no optimizadas que ya existen en el programa PLC. El detalle
completo está en `docs/PLC-S7-1200-PUT-GET.md`.

```
DB1000 = COMM01_EXT_UNIT  (receta y órdenes)
2.5     BOOL    start test
2.6     BOOL    stop test
4.0     REAL    bitmask de receta (A/B/C + 110VAC)
8.0     REAL    FAN Time (s)
12.0    REAL    DOOR Time (s)
16.0    REAL    FRAME Time (s)
86.0    REAL    DRAIN Time (s)
90.0    REAL    DEFROST Time (s)
94.0    REAL    LIGHT Time (s)
98.0    REAL    tolerancia de voltaje (%)

DB1001 = COMM01_UNIT_EXT  (resultados)
2.1     BOOL    Measuring
2.2     BOOL    Finished
2.3     BOOL    com st
2.4     BOOL    E-stop (mapear en PLC)
2.5     BOOL    Error de voltaje (mapear en PLC)
4.0     REAL    Result FAN
8.0     REAL    Result DOOR
12.0    REAL    Result FRAME
16.0    REAL    Result DRAIN
86.0    REAL    Result DEFROST
90.0    REAL    Result LIGHT
```

## 3. Hipot 3865 (USB/RS-232)

> **Corrección 2026-11-19:** el protocolo real del 3865 está en el Apéndice D del manual oficial. La versión anterior de este documento describía comandos que **no existen** en el equipo (`START`, `STOP`, `ABORT`, `VOLTAGE:`, `CURRENT_MAX:`, `TIME:`, `STATUS?`, `MEASURE?`, `RESULT?`). La implementación Rust ya usa el protocolo real. Ver `docs/manuals/Hipot3865/README.md` y `docs/manuals/Hipot3865/Hypot_Manual.pdf`.

### Configuración del puerto

| Parámetro | Valor |
|---|---|
| Baudios | **38400** |
| Bits de datos | 8 |
| Paridad | Ninguna |
| Bits de parada | 1 |
| Handshaking | Ninguno (sin XON/XOFF) |
| Terminador de comando | **NL / LF (0x0A)** |
| Respuesta a comando | **ACK (0x06)** o **NAK (0x15)** |
| Respuesta a consulta | ACK/NAK + datos terminados en NL |

El puerto USB del 3865 aparece en Windows como un puerto COM virtual (USB-to-RS-232). La tarjeta RS-232 opcional usa el mismo protocolo. El driver USB/RS-232 está en `docs/manuals/Hipot3865/drivers/AR_USB_Driver.zip`.

### Comandos principales

| Comando | Función |
|---|---|
| `*IDN?` | Identificación: empresa, modelo, serie, firmware |
| `TEST` | Inicia la secuencia en el paso seleccionado |
| `RESET` | Detiene/aborta la prueba y limpia fallas |
| `TD?` | Lee la medición/estado actual o el último resultado |
| `RD <paso>?` | Lee el resultado guardado de un paso |
| `LF?` | Nombre del archivo activo en RAM |
| `LF <memoria>?` | Nombre de un archivo por número de memoria |
| `FL <memoria>` | Carga un archivo de memoria a RAM |
| `FN <memoria>,<nombre>` | Crea un archivo nuevo (nombre máx. 8 caracteres) |
| `ST?` | Total de pasos del archivo activo |
| `SS <paso>` | Selecciona el paso activo |
| `SD <paso>` | Borra un paso |
| `ADD <tipo>,<parámetros...>` | Agrega un paso completo |
| `EV`, `EH`, `EL`, `ERU`, `EDW`, `ERD`, etc. | Edición individual de parámetros |
| `*OPC?` | Consulta si terminó la operación pendiente |

### Formato de `TD?` / `RD n?`

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

| Tipo | Medidor 1 | Medidor 2 | Medidor 3 |
|---|---|---|---|
| ACW | Voltaje en kV | Corriente en mA | Tiempo en s |
| DCW | Voltaje en kV | Corriente en µA | Tiempo en s |
| IR | Voltaje en V | Resistencia en MΩ | Tiempo en s |

Estados de proceso: `Dwell`, `Ramp`, `Delay`.  
Estados terminales: `PASS`, `HI-LMT`, `LO-LMT`, `OTP`, `CONT-F`, `Abort`, `OUT-ERROR`, etc.

### Secuencia usada por la app

1. `*IDN?` para verificar comunicación.
2. Cargar el archivo temporal `HIPOTAPP` con `FL` (se crea una vez con `FN` si no existe).
3. Borrar los pasos anteriores con `ST?` + `SD 1`.
4. Crear el paso ACW con `ADD ACW,<voltaje>,<mA>,0,0.1,<dwell>,0.0,5,OFF,60,OFF,1.50,0.00,0.00`.
5. Seleccionar el paso con `SS 1`.
6. Iniciar con `TEST`.
7. Consultar `TD?` cada 500 ms hasta `PASS` o estado terminal; timeout = dwell + 30 s.
8. Enviar `RESET` al terminar o al abortar.

### Paso ACW: orden de parámetros

```text
ADD ACW,Voltaje(V),HI-Limit(mA),LO-Limit(mA),RampUp(s),Dwell(s),RampDown(s),
ArcSense,ArcDetect,Frecuencia(Hz),Continuidad,ContHI(Ω),ContLO(Ω),ContOffset(Ω)
```

Ejemplo del manual:

```text
ADD ACW,1240,0.10,0.010,0.1,1.0,0.0,5,OFF,60,OFF,1.50,0.00,0.00
```

### Notas de integración

- La memoria no volátil tiene ciclos de escritura limitados. Crear el archivo temporal una vez y usar `FL` + `ADD` en RAM; evitar `FS`, `FSA`, `FN`, `FD`, `SAO`, `SSG2` en cada ciclo.
- Si la seguridad del instrumento está restringida, los comandos de edición pueden responder NAK. Para control por bus se requiere el nivel de acceso adecuado.
- El manual recomienda no ajustar parámetros individuales repetidamente; para producción usar `ADD`/`FL`.
- Verificar **PLC Remote** (`SPR`). Si el 3865 está configurado para iniciar solo por la entrada remota del panel trasero, `TEST` por bus puede no arrancar. Probar `SPR 0` una vez desde Configuración → Hipot → Comando manual.
- La app usa **ACW** por defecto. Si se requiere DCW o IR, se debe extender la receta y el comando `ADD` correspondiente.

## 4. Barcode Scanner (USB HID)

### Configuración

```python
import keyboard  # pip install keyboard
import threading

class BarcodeScanner:
    def __init__(self):
        self.buffer = ''
        self.scanning = False
        self.on_scan = None  # Callback
        self.thread = None
    
    def start(self):
        """Inicia escucha de código de barras"""
        self.scanning = True
        self.thread = threading.Thread(target=self._listen)
        self.thread.daemon = True
        self.thread.start()
    
    def stop(self):
        """Detiene escucha"""
        self.scanning = False
    
    def _listen(self):
        """Escucha teclas del lector"""
        def on_key(event):
            if not self.scanning:
                return
            
            if event.name == 'enter':
                # Fin de escaneo
                if self.buffer and self.on_scan:
                    self.on_scan(self.buffer)
                self.buffer = ''
            elif len(event.name) == 1:
                # Carácter normal
                self.buffer += event.name
        
        keyboard.on_press(on_key)
    
    def set_callback(self, callback):
        """Establece callback para escaneo"""
        self.on_scan = callback
```

### Funcionamiento

```
1. Lector USB conectado → Windows lo reconoce como teclado
2. Al escanear → envía caracteres como si fueran digiteados
3. App captura en campo activo
4. Enter al final (configurable en lector)
5. App recibe código completo
```

### NORMALIZACIÓN A MAYÚSCULAS (CRÍTICO)

```
REGLA: Todo código escaneado se convierte a MAYÚSCULAS antes de procesar.

Por qué:
- El mismo modelo puede escanear como "REF-1234" o "ref-1234"
- El barcode scanner envía lo que lee sin importar caso
- Si la DB tiene "REF-1234" y escaneas "ref-1234" → NO encuentra
- Con .upper() siempre coincide

Implementación:
  codigo_escaneado = raw_input.strip().upper()
```

```python
class BarcodeScanner:
    def _process_code(self, raw_code):
        # SIEMPRE convertir a mayúsculas
        normalized = raw_code.strip().upper()
        
        # Log del escaneo
        log_action("BARCODE", "scan", f"Raw: {raw_code} → Normalizado: {normalized}")
        
        return normalized
```

```
Ejemplos:
  "ref-1234"     → "REF-1234"     ✅
  "abc123"       → "ABC123"       ✅
  "Ref-1234"     → "REF-1234"     ✅
  "  ref-1234 "  → "REF-1234"     ✅ (trim + upper)
```

## 5. SQL (SQLAlchemy)

### Configuración

```python
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()

class DatabaseManager:
    def __init__(self, connection_string):
        self.engine = create_engine(connection_string)
        self.Session = sessionmaker(bind=self.engine)
        self.session = None
    
    def connect(self):
        try:
            self.session = self.Session()
            return True
        except:
            return False
    
    def disconnect(self):
        if self.session:
            self.session.close()
    
    def create_tables(self):
        """Crea todas las tablas si no existen"""
        Base.metadata.create_all(self.engine)
    
    def test_connection(self):
        """Prueba conexión a la base de datos"""
        try:
            self.session.execute('SELECT 1')
            return True
        except:
            return False

# Conexión Local (SQL Express)
local_db = DatabaseManager(
    'mssql+pyodbc://./SQLEXPRESS/HiPotDB?driver=ODBC+Driver+17+for+SQL+Server'
)

# Conexión Remota (SQL Server)
remote_db = DatabaseManager(
    'mssql+pyodbc://hipot_user:password@192.168.1.200/HiPotDB_Central?driver=ODBC+Driver+17+for+SQL+Server'
)
```

## 6. Sincronización Local → Remoto

```python
import asyncio
from datetime import datetime

class SyncManager:
    def __init__(self, local_db, remote_db, interval_minutes=5):
        self.local_db = local_db
        self.remote_db = remote_db
        self.interval = interval_minutes * 60  # a segundos
        self.running = False
    
    async def start_sync(self):
        """Inicia sincronización periódica"""
        self.running = True
        while self.running:
            await self.sync_pending()
            await asyncio.sleep(self.interval)
    
    async def sync_pending(self):
        """Sincroniza registros pendientes"""
        try:
            # 1. Obtener registros no sincronizados
            pending = self.local_db.session.query(
                Execution
            ).filter(
                Execution.synced_to_remote == False
            ).all()
            
            for record in pending:
                try:
                    # 2. Insertar en remoto
                    self.remote_db.session.add(record)
                    self.remote_db.session.commit()
                    
                    # 3. Marcar como sincronizado
                    record.synced_to_remote = True
                    record.synced_at = datetime.now()
                    self.local_db.session.commit()
                    
                except Exception as e:
                    # 4. Log de error
                    self.log_sync_error(record.id, str(e))
                    self.remote_db.session.rollback()
            
            return len(pending)
            
        except Exception as e:
            self.log_sync_error('general', str(e))
            return 0
    
    def log_sync_error(self, record_id, error):
        """Guarda log de error de sincronización"""
        log = SyncLog(
            table_name='executions',
            record_id=record_id,
            error_message=error,
            timestamp=datetime.now()
        )
        self.local_db.session.add(log)
        self.local_db.session.commit()
```

## 7. Tiempos de Comunicación

| Operación | Tiempo esperado | Timeout |
|-----------|-----------------|---------|
| PLC read tag | 10-50ms | 500ms |
| PLC write tag | 10-50ms | 500ms |
| Hipot send command | 10-50ms | 200ms |
| Hipot read response | 100-500ms | 2000ms |
| Barcode scan | 50-200ms | 5000ms |
| SQL local query | 5-20ms | 1000ms |
| SQL remote query | 50-200ms | 5000ms |
| Sync push | 100-500ms/reg | 10000ms |
