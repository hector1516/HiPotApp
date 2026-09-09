# Hardware — HiPotApp

## 1. Panel PC (Estación de Trabajo)

| Especificación | Valor |
|----------------|-------|
| **Pantalla** | 10.1" TFT LCD |
| **Resolución** | 1280 x 800 (16:10) |
| **Touch** | Capacitivo multitouch |
| **Sistema** | Windows 11 Pro |
| **RAM** | Mínimo 8GB (recomendado 16GB) |
| **Almacenamiento** | SSD 256GB mínimo |
| **Conexiones** | USB 3.0+ (mínimo 4 puertos), Ethernet RJ45 |
| **Montaje** | Panel empotrado o brazo articulado |

### Configuración Windows 11

```
1. Desactivar gestos de borde (para que no salga de la app)
   - Configuración → Touch → Desactivar gestos desde bordes

2. Fullscreen automático para Tauri
   - Iniciar con Windows
   - Modo kiosk o fullscreen borderless

3. Desactivar screensaver y suspensión
   - Configuración → Sistema → Energía → Nunca apagar

4. Puerto COM del Hypot
   - Administrador de dispositivos → Puertos COM
   - Asignar COM fijo (COM3 recomendado)
   - Desactivar suspensión del puerto USB

5. Touch calibration
   - Configuración → Pantalla → Calibrar touch
```

## 2. PLC Siemens S7-1200

| Especificación | Valor |
|----------------|-------|
| **Modelo** | S7-1200 (CPU 1211C, 1212C, 1214C, o 1215C) |
| **Comunicación** | Ethernet TCP/IP |
| **IP típica** | 192.168.1.10 |
| **Puerto** | 102 (estándar S7) |
| **Rack** | 0 |
| **Slot** | 1 |
| **Protocolo** | S7 Communication (vía snap7) |

### Tags del PLC (requeridos)

```
// Tags de configuración (App → PLC)
DB1.Recipe_Voltage        WORD    // Voltaje seleccionado (120, 208, 220, 240)
DB1.Component_FAN_MIN     REAL    // Límite mínimo FAN
DB1.Component_FAN_MAX     REAL    // Límite máximo FAN
DB1.Component_FAN_ENABLE  BOOL    // TRUE = probar, FALSE = saltar
DB1.Component_DOOR_MIN    REAL
DB1.Component_DOOR_MAX    REAL
DB1.Component_DOOR_ENABLE BOOL
// ... (igual para FRAME, DRAIN, DEFROST, LIGHT)
DB1.Test_Start            BOOL    // TRUE = iniciar prueba
DB1.Test_Stop             BOOL    // TRUE = parar prueba

// Tags de resultados (PLC → App)
DB1.Result_FAN            REAL    // Valor medido FAN
DB1.Result_DOOR           REAL    // Valor medido DOOR
DB1.Result_FRAME          REAL
DB1.Result_DRAIN          REAL
DB1.Result_DEFROST        REAL
DB1.Result_LIGHT          REAL
DB1.Result_TOTAL          REAL    // Total medido
DB1.Test_Complete         BOOL    // TRUE = prueba terminó
DB1.Test_Pass             BOOL    // TRUE = pasó, FALSE = falló
DB1.Test_Running          BOOL    // TRUE = prueba en curso
```

### Conexión (snap7 Python)

```python
import snap7

# Conectar al PLC
plc = snap7.client.Client()
plc.connect('192.168.1.10', 0, 1)  # IP, rack, slot

# Leer tag
data = plc.db_read(1, 0, 10)  # DB1, inicio 0, 10 bytes

# Escribir tag
plc.db_write(1, 0, value_bytes)

# Desconectar
plc.disconnect()
```

## 3. Hipot 3865

| Especificación | Valor |
|----------------|-------|
| **Modelo** | Hipot 3865 (serie 38xx) |
| **Comunicación** | RS-232 serial (USB) |
| **Driver** | H4 IVI Driver (por validar) |
| **Baud rate** | 9600 / 19200 / 38400 (configurable) |
| **Data bits** | 8 |
| **Parity** | None |
| **Stop bits** | 1 |
| **Puerto típico** | COM3 (configurable) |

### Comandos RS-232 (por validar con manual)

```
// Comandos de control
START\r\n          // Iniciar prueba
STOP\r\n           // Parar prueba
ABORT\r\n          // Parar emergencia

// Configuración de receta
VOLTAGE:120\r\n    // Seleccionar voltaje
CURRENT_MAX:10.5\r\n  // Límite corriente
TIME:30\r\n        // Tiempo de prueba en segundos

// Consulta de valores
STATUS?\r\n        // Estado actual
MEASURE?\r\n       // Obtener medición
RESULT?\r\n        // Obtener resultado final

// Formato de respuesta típico
V:120.5 I:1.23 R:456.7 KΩ PASS\r\n
```

### Conexión (pyserial Python)

```python
import serial
import threading

# Abrir puerto serie
ser = serial.Serial(
    port='COM3',
    baudrate=9600,
    bytesize=serial.EIGHTBITS,
    parity=serial.PARITY_NONE,
    stopbits=serial.STOPBITS_ONE,
    timeout=1
)

# Enviar comando
ser.write(b'START\r\n')

# Leer respuesta
response = ser.readline().decode('ascii').strip()

# Hilo de lectura continua
def read_hypot():
    while True:
        if ser.in_waiting:
            data = ser.readline().decode('ascii').strip()
            # Procesar datos...
        
# Cerrar
ser.close()
```

## 4. Impresora Zebra ZT231

| Especificación | Valor |
|----------------|-------|
| **Modelo** | Zebra ZT231 |
| **Tipo** | Térmica directa / Transferencia térmica |
| **Resolución** | 203 dpi |
| **Ancho máximo** | 108 mm |
| **Velocidad** | hasta 152 mm/s |
| **Conexión** | USB (único método) |
| **Lenguaje** | ZPL II |
| **Etiqueta típica** | 100 x 60 mm |
| **Driver** | Zebra ZT231 v8 (Windows) |

### Funcionamiento

```
1. Impresora conectada por USB-B → USB-A en PC
2. Windows la detecta como "Zebra ZT231"
3. App genera datos ZPL con resultado de prueba
4. Envía a impresora por comando Windows (print /d:)
5. Imprime etiqueta automáticamente si pasa la prueba
```

### Cuándo imprime

```
- Prueba completa (PLC + Hipot)
- execution.final_status = "GOOD" (solo si pasa)
- Datos: Modelo, Serie, Batch, Operador, Fecha, Resultados, Barcode, QR
- Admin puede reimpresión manual desde historial
```

### Verificar impresión

```python
# Enviar prueba de impresión
zpl = "^XA^FO50,50^ADN,36,20^FDHello Zebra^FS^XZ"
subprocess.run(['print', '/d:Zebra ZT231', temp_file])
```

## 5. Lector de Código de Barras

| Especificación | Valor |
|----------------|-------|
| **Tipo** | USB HID (teclado emulador) |
| **Conexión** | USB directo |
| **Protocolo** | Keyboard wedge |
| **Formatos** | Code 128, Code 39, QR, etc. |

### Funcionamiento

```
1. Lector conectado por USB
2. Windows lo reconoce como teclado
3. Al escanear, envía caracteres como si fueran digiteados
4. App recibe el código en el campo activo
5. Carácter especial Enter al final (configurable)
```

### Configuración en App

```python
# El código llega como texto del teclado
# En React/Tauri, capturar en input activo
# El escaneo es automático cuando hay foco en campo
```

## 6. Diagrama de Conexiones

```
┌─────────────────────────────────────────────────────────────────┐
│                    PANEL PC (10.1" Touch)                       │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Windows 11 Pro                         │  │
│  │                                                           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │  │
│  │  │   Tauri     │  │   Python    │  │   SQL       │      │  │
│  │  │   App       │◄─┤   FastAPI   │◄─┤   Express   │      │  │
│  │  └─────────────┘  └──────┬──────┘  └─────────────┘      │  │
│  └──────────────────────────┼───────────────────────────────┘  │
│                             │                                   │
└─────────────────────────────┼───────────────────────────────────┘
                              │
      ┌───────────┬───────────┼───────────┬───────────┐
      │           │           │           │           │
      ▼           ▼           ▼           ▼           ▼
┌───────────┐┌───────────┐┌───────────┐┌───────────┐┌───────────┐
│  Hipot    ││  PLC      ││  Barcode  ││  Zebra    ││  Red      │
│  3865     ││  S7-1200  ││  Scanner  ││  ZT231    ││  LAN      │
│  RS-232   ││  Ethernet ││  USB HID  ││  USB      ││  SQL Sync │
│  (COM3)   ││  (RJ45)   ││  (USB)    ││  (USB-B)  ││  (RJ45)   │
└───────────┘└───────────┘└───────────┘└───────────┘└───────────┘
   USB           Ethernet      USB          USB         Ethernet
```

## 6. Requisitos de Red

| Dispositivo | IP | Puerto | Protocolo |
|-------------|-----|--------|-----------|
| PC Local | 192.168.1.100 | - | - |
| PLC S7-1200 | 192.168.1.10 | 102 | S7/TCP |
| SQL Server Remoto | 192.168.1.100 | 1433 | TCP |

### Configuración de Red

```
// PC Local
IP: 192.168.1.100
Mascara: 255.255.255.0
Gateway: 192.168.1.1

// PLC
IP: 192.168.1.10
Mascara: 255.255.255.0

// SQL Server Remoto
IP: 192.168.1.200
Puerto: 1433 (TCP)
```
