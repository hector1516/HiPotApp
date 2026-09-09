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

### Mapeo de Tags (DB1)

```
// Direcciones de memoria DB1
Offset  Tamaño   Tag                      Tipo
------  ------   ---                      ----
0       4        Recipe_Voltage           REAL
4       4        Component_FAN_MIN        REAL
8       4        Component_FAN_MAX        REAL
12      1        Component_FAN_ENABLE     BOOL
16      4        Component_DOOR_MIN       REAL
20      4        Component_DOOR_MAX       REAL
24      1        Component_DOOR_ENABLE    BOOL
...     ...      (repetir para FRAME, DRAIN, DEFROST, LIGHT)
100     1        Test_Start               BOOL
101     1        Test_Stop                BOOL
102     1        Test_Complete            BOOL
103     1        Test_Pass                BOOL
104     1        Test_Running             BOOL
// Resultados (lectura)
200     4        Result_FAN               REAL
204     4        Result_DOOR              REAL
208     4        Result_FRAME             REAL
212     4        Result_DRAIN             REAL
216     4        Result_DEFROST           REAL
220     4        Result_LIGHT             REAL
224     4        Result_TOTAL             REAL
```

## 3. Hipot 3865 (pyserial)

### Configuración

```python
import serial
import serial.tools.list_ports
import threading
import time
from queue import Queue

class HipotManager:
    def __init__(self, port='COM3', baudrate=9600):
        self.port = port
        self.baudrate = baudrate
        self.serial = None
        self.connected = False
        self.read_thread = None
        self.data_queue = Queue()
        self.running = False
    
    def list_ports(self):
        """Lista puertos COM disponibles"""
        ports = serial.tools.list_ports.comports()
        return [p.device for p in ports]
    
    def connect(self):
        try:
            self.serial = serial.Serial(
                port=self.port,
                baudrate=self.baudrate,
                bytesize=serial.EIGHTBITS,
                parity=serial.PARITY_NONE,
                stopbits=serial.STOPBITS_ONE,
                timeout=1
            )
            self.connected = True
            self.start_reading()
            return True
        except Exception as e:
            self.connected = False
            return False
    
    def disconnect(self):
        self.running = False
        if self.read_thread:
            self.read_thread.join(timeout=2)
        if self.serial and self.serial.is_open:
            self.serial.close()
        self.connected = False
    
    def send_command(self, command):
        """Envía comando al Hipot"""
        if not self.connected:
            return False
        try:
            self.serial.write(f'{command}\r\n'.encode('ascii'))
            return True
        except:
            return False
    
    def read_response(self):
        """Lee respuesta del Hipot"""
        if not self.connected:
            return None
        try:
            response = self.serial.readline().decode('ascii').strip()
            return response
        except:
            return None
    
    def start_reading(self):
        """Inicia hilo de lectura continua"""
        self.running = True
        self.read_thread = threading.Thread(target=self._read_loop)
        self.read_thread.daemon = True
        self.read_thread.start()
    
    def _read_loop(self):
        """Loop de lectura en hilo separado"""
        while self.running and self.connected:
            try:
                if self.serial.in_waiting:
                    data = self.serial.readline().decode('ascii').strip()
                    if data:
                        self.data_queue.put(data)
                time.sleep(0.1)  # 100ms
            except:
                self.connected = False
                break
    
    def get_measurement(self):
        """Obtiene medición del queue"""
        if not self.data_queue.empty():
            return self.data_queue.get()
        return None
    
    def start_test(self):
        """Inicia prueba en el Hipot"""
        return self.send_command('START')
    
    def stop_test(self):
        """Para prueba en el Hipot"""
        return self.send_command('STOP')
    
    def abort_test():
        """Paro de emergencia"""
        return self.send_command('ABORT')
    
    def set_voltage(self, voltage):
        """Configura voltaje"""
        return self.send_command(f'VOLTAGE:{voltage}')
    
    def set_current_limit(self, limit):
        """Configura límite de corriente"""
        return self.send_command(f'CURRENT_MAX:{limit}')
    
    def get_status(self):
        """Obtiene estado actual"""
        self.send_command('STATUS?')
        time.sleep(0.1)
        return self.read_response()
    
    def test_connection(self):
        """Prueba si el Hipot responde"""
        self.send_command('STATUS?')
        time.sleep(0.5)
        response = self.read_response()
        return response is not None and len(response) > 0
    
    def parse_measurement(self, data):
        """Parsea string de medición"""
        # Formato típico: V:120.5 I:1.23 R:456.7 KΩ PASS
        result = {}
        try:
            parts = data.split()
            for part in parts:
                if part.startswith('V:'):
                    result['voltage'] = float(part[2:])
                elif part.startswith('I:'):
                    result['current'] = float(part[2:])
                elif part.startswith('R:'):
                    result['resistance'] = float(part[2:])
                elif part == 'PASS':
                    result['pass'] = True
                elif part == 'FAIL':
                    result['pass'] = False
        except:
            pass
        return result
```

### Comandos RS-232

```
// Comandos de control
START\r\n          // Iniciar prueba
STOP\r\n           // Parar prueba normal
ABORT\r\n          // Paro de emergencia

// Configuración
VOLTAGE:120\r\n    // Seleccionar voltaje (120, 208, 220, 240)
CURRENT_MAX:10.5\r\n  // Límite de corriente
TIME:30\r\n        // Tiempo máximo de prueba (segundos)

// Consulta
STATUS?\r\n        // Estado del equipo
MEASURE?\r\n       // Obtener medición actual
RESULT?\r\n        // Obtener resultado final

// Respuesta típica
V:120.5 I:1.23 R:456.7 KΩ PASS\r\n
V:0.0 I:0.0 R:0.0 FAIL\r\n
```

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
