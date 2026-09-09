# Base de Datos — HiPotApp

## 1. Arquitectura Dual (Local + Remoto)

```
┌─────────────────────────────────────┐     ┌─────────────────────────────────────┐
│         PC LOCAL (Panel 10.1")      │     │      SERVIDOR REMOTO (Otra PC)      │
│                                     │     │                                     │
│  ┌───────────────────────────────┐  │     │  ┌───────────────────────────────┐  │
│  │   SQL Express (Local)        │  │     │  │   SQL Server (Remoto)         │  │
│  │   - Recetas                  │  │  ──▶│  │   - Recetas (copia)           │  │
│  │   - Resultados del día       │  │ Sync│  │   - Todos los resultados       │  │
│  │   - Configuración            │  │     │  │   - Configuración              │  │
│  │   - Usuarios                 │  │     │  │   - Usuarios                   │  │
│  │   - Logs de ejecución        │  │     │  │   - Logs de ejecución          │  │
│  └───────────────────────────────┘  │     │  └───────────────────────────────┘  │
│                                     │     │                                     │
│  App funciona SIN red (offline)     │     │  Servidor central de reportes      │
└─────────────────────────────────────┘     └─────────────────────────────────────┘
         │                                                   │
         │              App Opera Local                      │
         │              (sin depender de red)                │
         │                                                   │
         └─────────── Sync cuando hay red ──────────────────┘
```

## 2. Por qué Dual

| Problema | Solución |
|----------|----------|
| Red falla durante prueba | Local guarda todo, sync después |
| Servidor remoto cae | App sigue funcionando |
| Múltiples líneas producen | Cada una tiene su SQL Express local |
| Reportes centrales | Remoto consolida todo |
| Velocidad de consulta | Local es rápido (mismo disco) |
| Backup automático | Remoto tiene copia de seguridad |

## 3. Tablas (mismas en ambos servidores)

```sql
-- ============================================================
-- RECETAS
-- ============================================================
CREATE TABLE recipes (
  id INT IDENTITY(1,1) PRIMARY KEY,
  name NVARCHAR(100) NOT NULL,
  model NVARCHAR(50) UNIQUE NOT NULL,
  type NVARCHAR(50) DEFAULT 'current_consumption',
  product NVARCHAR(100),
  voltage_levels NVARCHAR(MAX), -- JSON: ["120VAC","208VAC","220VAC","240VAC"]
  is_active BIT DEFAULT 1,
  created_at DATETIME2 DEFAULT GETDATE(),
  updated_at DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- COMPONENTES DE RECETA
-- ============================================================
CREATE TABLE recipe_components (
  id INT IDENTITY(1,1) PRIMARY KEY,
  recipe_id INT FOREIGN KEY REFERENCES recipes(id),
  name NVARCHAR(50) NOT NULL,      -- FAN, DOOR, FRAME, etc.
  voltage NVARCHAR(20) NOT NULL,   -- 120VAC, 208VAC, etc.
  enabled BIT DEFAULT 1,           -- 0 = no probar (cero al PLC)
  min_value FLOAT DEFAULT 0,
  max_value FLOAT DEFAULT 0,
  unit NVARCHAR(10) DEFAULT 'A'
);

-- ============================================================
-- TOTALES POR VOLTAJE
-- ============================================================
CREATE TABLE recipe_totals (
  id INT IDENTITY(1,1) PRIMARY KEY,
  recipe_id INT FOREIGN KEY REFERENCES recipes(id),
  voltage NVARCHAR(20) NOT NULL,
  min_total FLOAT NOT NULL,
  max_total FLOAT NOT NULL
);

-- ============================================================
-- OPERADORES
-- ============================================================
CREATE TABLE operators (
  id INT IDENTITY(1,1) PRIMARY KEY,
  name NVARCHAR(100) NOT NULL,
  employee_id NVARCHAR(20) UNIQUE NOT NULL,
  is_active BIT DEFAULT 1,
  created_at DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- BATCHES
-- ============================================================
CREATE TABLE batches (
  id INT IDENTITY(1,1) PRIMARY KEY,
  batch_number NVARCHAR(50) UNIQUE NOT NULL,
  operator_id INT FOREIGN KEY REFERENCES operators(id),
  scanned_at DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- RESULTADOS PLC (lecturas individuales)
-- ============================================================
CREATE TABLE plc_readings (
  id INT IDENTITY(1,1) PRIMARY KEY,
  execution_id INT FOREIGN KEY REFERENCES executions(id),
  recipe_id INT FOREIGN KEY REFERENCES recipes(id),
  component_name NVARCHAR(50),    -- FAN, DOOR, FRAME, etc.
  voltage NVARCHAR(20),           -- 120VAC, 208VAC, etc.
  tag_name NVARCHAR(50),          -- DB1.DBW0, etc.
  value FLOAT,
  min_limit FLOAT,
  max_limit FLOAT,
  is_pass BIT,
  timestamp DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- RESULTADOS HIPOT (mediciones)
-- ============================================================
CREATE TABLE hipot_results (
  id INT IDENTITY(1,1) PRIMARY KEY,
  execution_id INT FOREIGN KEY REFERENCES executions(id),
  recipe_id INT FOREIGN KEY REFERENCES recipes(id),
  voltage FLOAT,
  current FLOAT,
  resistance FLOAT,
  insulation_resistance FLOAT,
  test_duration FLOAT,            -- segundos
  is_pass BIT,
  timestamp DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- EJECUCIONES (flujo principal)
-- ============================================================
CREATE TABLE executions (
  id INT IDENTITY(1,1) PRIMARY KEY,
  recipe_id INT FOREIGN KEY REFERENCES recipes(id),
  operator_id INT FOREIGN KEY REFERENCES operators(id),
  batch_id INT FOREIGN KEY REFERENCES batches(id),
  serial_number NVARCHAR(100),    -- escaneado

  -- PLC
  plc_start_time DATETIME2,
  plc_end_time DATETIME2,
  plc_duration FLOAT,             -- segundos
  plc_pass_fail BIT,

  -- HIPOT
  hipot_start_time DATETIME2,
  hipot_end_time DATETIME2,
  hipot_duration FLOAT,
  hipot_pass_fail BIT,

  -- Estado final
  final_status NVARCHAR(10),      -- GOOD / BAD
  total_duration FLOAT,           -- segundos totales

  -- Sync
  synced_to_remote BIT DEFAULT 0, -- 0 = pendiente, 1 = sincronizado
  synced_at DATETIME2,

  created_at DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- LOGS DE CÓDIGO DE BARRAS
-- ============================================================
CREATE TABLE barcode_logs (
  id INT IDENTITY(1,1) PRIMARY KEY,
  code NVARCHAR(100) NOT NULL,
  barcode_type NVARCHAR(20),      -- model, serial, batch
  scan_order INT,                 -- 1=model, 2=serial, 3=batch
  execution_id INT FOREIGN KEY REFERENCES executions(id),
  scanned_at DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- CONFIGURACIÓN (key-value)
-- ============================================================
CREATE TABLE configurations (
  id INT IDENTITY(1,1) PRIMARY KEY,
  config_key NVARCHAR(50) UNIQUE NOT NULL,
  config_value NVARCHAR(500) NOT NULL,
  updated_at DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- USUARIOS / ADMIN
-- ============================================================
CREATE TABLE users (
  id INT IDENTITY(1,1) PRIMARY KEY,
  username NVARCHAR(50) UNIQUE NOT NULL,
  password_hash NVARCHAR(255) NOT NULL,
  role NVARCHAR(20) DEFAULT 'operator', -- operator, admin
  is_active BIT DEFAULT 1,
  created_at DATETIME2 DEFAULT GETDATE()
);

-- ============================================================
-- SINCRONIZACIÓN (log de sync)
-- ============================================================
CREATE TABLE sync_log (
  id INT IDENTITY(1,1) PRIMARY KEY,
  direction NVARCHAR(10),         -- local_to_remote, remote_to_local
  table_name NVARCHAR(50),
  records_synced INT,
  status NVARCHAR(20),            -- success, error
  error_message NVARCHAR(500),
  started_at DATETIME2 DEFAULT GETDATE(),
  completed_at DATETIME2
);

-- ============================================================
-- APP LOGS (trazabilidad completa para debug)
-- Registra CADA acción de la app: botones, escaneos,
-- conexiones, errores, valores enviados/recibidos.
-- ============================================================
CREATE TABLE app_logs (
  id INT IDENTITY(1,1) PRIMARY KEY,
  timestamp DATETIME2 DEFAULT GETDATE(),
  level NVARCHAR(10) NOT NULL,       -- INFO, WARN, ERROR, DEBUG
  category NVARCHAR(50) NOT NULL,    -- UI, PLC, HIPOT, DB, SYNC, BARCODE, SYSTEM
  action NVARCHAR(100) NOT NULL,     -- button_press, scan, plc_read, plc_write, etc.
  detail NVARCHAR(MAX),              -- descripción legible del evento
  execution_id INT,                  -- NULL si no hay ejecución activa
  user_id INT,                       -- NULL si es sistema
  value_sent NVARCHAR(200),          -- dato enviado (ej: "DB1, tag=FAN_MIN, val=0.3")
  value_received NVARCHAR(200),      -- dato recibido (ej: "V:120.5 I:1.23 PASS")
  duration_ms INT,                   -- duración de la operación en milisegundos
  success BIT DEFAULT 1,             -- 1=éxito, 0=fallo
  error_message NVARCHAR(500)        -- mensaje de error si success=0
);

-- Índices para consultas rápidas de debug
CREATE INDEX idx_app_logs_timestamp ON app_logs(timestamp);
CREATE INDEX idx_app_logs_category ON app_logs(category);
CREATE INDEX idx_app_logs_level ON app_logs(level);
CREATE INDEX idx_app_logs_execution ON app_logs(execution_id);

-- ============================================================
-- UI STATE LOG (estado visual en cada momento)
-- ============================================================
CREATE TABLE ui_state_log (
  id INT IDENTITY(1,1) PRIMARY KEY,
  timestamp DATETIME2 DEFAULT GETDATE(),
  execution_id INT,
  current_step INT,                   -- 1=modelo, 2=serie, 3=batch, 4=prueba
  current_phase NVARCHAR(20),         -- IDLE, SCANNING, LOADED, READY, PLC_RUNNING, HIPOT_RUNNING, COMPLETE
  background_color NVARCHAR(20),      -- #e0e0e0, #bbdefb, etc.
  message_displayed NVARCHAR(200)     -- texto visible en pantalla
);

-- ============================================================
-- IMPRESIÓN DE ETIQUETAS (Zebra ZT231)
-- ============================================================
CREATE TABLE label_prints (
  id INT IDENTITY(1,1) PRIMARY KEY,
  execution_id INT FOREIGN KEY REFERENCES executions(id),
  printed_by NVARCHAR(50) DEFAULT 'auto',  -- 'auto' o 'admin_reprint'
  printed_at DATETIME2 DEFAULT GETDATE(),
  printer_model NVARCHAR(50) DEFAULT 'Zebra ZT231',
  connection_type NVARCHAR(10),       -- usb, ethernet
  zpl_data NVARCHAR(MAX),            -- ZPL enviado (para reimpresión)
  status NVARCHAR(20),               -- success, error
  error_message NVARCHAR(200),
  copies INT DEFAULT 1
);
```

## 4. Configuración Dual en Admin

### **Pestana: Configuración → Base de Datos**

```
┌─────────────────────────────────────────────────────────────┐
│  CONFIGURACIÓN BASE DE DATOS                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─── LOCAL (SQL Express) ──────────────────────────────┐  │
│  │  Servidor: [.\SQLEXPRESS]                            │  │
│  │  Base:     [HiPotDB]                                 │  │
│  │  Puerto:   [1433]                                    │  │
│  │  Windows Auth: [✅]  SQL Auth: [ ]                   │  │
│  │  Usuario:  [___________]                             │  │
│  │  Password: [___________]                             │  │
│  │                                                       │  │
│  │  [🔌 PROBAR CONEXIÓN]  [🔄 CREAR TABLAS]            │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌─── REMOTO (SQL Server) ──────────────────────────────┐  │
│  │  Servidor: [192.168.1.100]                            │  │
│  │  Base:     [HiPotDB_Central]                          │  │
│  │  Puerto:   [1433]                                    │  │
│  │  Windows Auth: [ ]  SQL Auth: [✅]                   │  │
│  │  Usuario:  [hipot_user]                              │  │
│  │  Password: [••••••••]                                │  │
│  │                                                       │  │
│  │  [🔌 PROBAR CONEXIÓN]  [🔄 CREAR TABLAS]            │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌─── SINCRONIZACIÓN ───────────────────────────────────┐  │
│  │  Modo: [Solo Local ▼]                                │  │
│  │    ○ Solo Local (sin sync)                           │  │
│  │    ● Local → Remoto (push automático)                │  │
│  │    ○ Remoto → Local (pull)                           │  │
│  │    ○ Bidireccional                                   │  │
│  │                                                       │  │
│  │  Intervalo: [5] minutos                              │  │
│  │  Sync en background: [✅]                            │  │
│  │  Última sync: 2025-01-15 14:30:00                   │  │
│  │                                                       │  │
│  │  [🔄 SYNC AHORA]  [📋 VER LOG]                      │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  [💾 GUARDAR]  [✅ APLICAR]                                │
└─────────────────────────────────────────────────────────────┘
```

## 5. Lógica de Sincronización

### **Flujo: Local → Remoto (push)**

```
1. App guarda resultado en SQL Express (local)
2. Marca registro: synced_to_remote = 0
3. Cada N minutos (o al final de prueba):
   a. Lee registros con synced_to_remote = 0
   b. Intenta INSERT en SQL Server remoto
   c. Si éxito → marca synced_to_remote = 1
   d. Si error → guarda en sync_log, reintenta después
4. Si red cae → sigue guardando local
5. Cuando red vuelve → reintentar sync pendientes
```

### **Código Python (pseudocódigo)**

```python
async def sync_to_remote():
    """Sincronizar registros pendientes al servidor remoto"""
    
    # 1. Conectar al remoto
    remote_conn = connect_to_remote_sql()
    
    # 2. Obtener registros no sincronizados
    pending = local_db.query(
        "SELECT * FROM executions WHERE synced_to_remote = 0"
    )
    
    for record in pending:
        try:
            # 3. Intentar INSERT en remoto
            remote_db.execute(
                "INSERT INTO executions (...) VALUES (...)",
                record
            )
            
            # 4. Marcar como sincronizado
            local_db.execute(
                "UPDATE executions SET synced_to_remote = 1, synced_at = GETDATE() WHERE id = ?",
                record.id
            )
            
        except Exception as e:
            # 5. Log de error
            log_sync_error(record.id, str(e))
    
    remote_conn.close()
```

### **Flujo: Crear Tablas (botón admin)**

```
1. Admin presiona "CREAR TABLAS" (local o remoto)
2. App ejecuta script SQL CREATE TABLE IF NOT EXISTS
3. Verifica cada tabla:
   - Si existe → skip
   - Si no existe → crea
4. Reporta: "5 tablas creadas, 3 ya existían"
```

## 6. Tabla de Configuración Inicial

```sql
-- Insertar configuración por defecto
INSERT INTO configurations (config_key, config_value) VALUES
('plc_ip', '192.168.1.10'),
('plc_rack', '0'),
('plc_slot', '1'),
('hipot_port', 'COM3'),
('hipot_baud', '9600'),
('hipot_parity', 'None'),
('hipot_databits', '8'),
('hipot_stopbits', '1'),
('db_local_server', '.\SQLEXPRESS'),
('db_local_name', 'HiPotDB'),
('db_remote_server', '192.168.1.100'),
('db_remote_name', 'HiPotDB_Central'),
('sync_interval_minutes', '5'),
('sync_mode', 'local_to_remote');
```

## 7. Resumen de Tablas

| Tabla | Propósito | Sync |
|-------|-----------|------|
| `recipes` | Recetas de prueba | Sí |
| `recipe_components` | Componentes por receta | Sí |
| `recipe_totals` | Totales por voltaje | Sí |
| `operators` | Operadores registrados | Sí |
| `batches` | Lotes escaneados | Sí |
| `executions` | Flujo principal PLC+Hipot | Sí |
| `plc_readings` | Lecturas individuales PLC | Sí |
| `hipot_results` | Resultados Hipot | Sí |
| `barcode_logs` | Logs de escaneo | Sí |
| `configurations` | Config del sistema | No (solo local) |
| `users` | Usuarios admin | Sí |
| `sync_log` | Log de sincronización | No (solo local) |
| `app_logs` | Trazabilidad completa de la app | No (solo local) |
| `ui_state_log` | Estado visual de la UI en cada momento | No (solo local) |
| `label_prints` | Registro de impresiones de etiquetas Zebra | Sí |
| `label_templates` | Plantillas de etiquetas (editor visual) | No (solo local) |

**Total: 16 tablas**

Ver `08-logging-debug.md` para detalles de `app_logs` y `ui_state_log`.
