# Ventana Administrador — HiPotApp

## 1. Acceso

```
Botón "⚙️ CONFIG" en pantalla principal
→ Solicita password de admin
→ Abre vista admin en misma ventana (1280x800)
→ Botón "🔙 VOLVER" regresa a pantalla operador
```

## 2. Pestanas del Admin

### 2.1 Usuarios

```
┌─────────────────────────────────────────────────────────────┐
│  👤 USUARIOS                                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [+ NUEVO USUARIO]                                         │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Nombre          ID        Rol       Estado         │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  Juan Pérez      JP001     operator  ✅ Activo      │   │
│  │  María López     ML002     admin     ✅ Activo      │   │
│  │  Carlos Ruiz     CR003     operator  ❌ Inactivo    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Roles:                                                     │
│  - operator: Solo puede ejecutar pruebas                   │
│  - admin: Acceso completo a configuración                  │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Recetas

```
┌─────────────────────────────────────────────────────────────┐
│  📋 RECETAS                                  [📥 Importar] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [+ NUEVA RECETA]  [📋 DUPLICAR]  [🗑️ ELIMINAR]          │
│                                                             │
│  Buscar: [________________] [🔍]                           │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Modelo      Nombre         Voltajes    Componentes │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  REF-1234    Consumos RL    120,208,    6 (FAN,     │   │
│  │                           220,240      DOOR,FRAME,  │   │
│  │                                        DRAIN,DEFROST│   │
│  │                                        ,LIGHT)      │   │
│  │  REF-5678    Consumos RT    120,240     4 (FAN,      │   │
│  │                                        DOOR,FRAME,  │   │
│  │                                        LIGHT)       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  AL EDITAR RECETA:                                         │
│  - Nombre libre                                            │
│  - Modelo único (barcode)                                  │
│  - Voltajes disponibles: 120, 208, 220, 240               │
│  - Por cada voltaje, activar/desactivar componentes        │
│  - Límites MIN/MAX por componente                          │
│  - Checkbox "USAR" por componente                          │
│  - Totales calculados o manuales                           │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 PLC

```
┌─────────────────────────────────────────────────────────────┐
│  📡 PLC (Siemens S7-1200)                                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Configuración:                                            │
│  IP:        [192.168.1.10]                                 │
│  Rack:      [0]                                            │
│  Slot:      [1]                                            │
│                                                             │
│  [🔌 PROBAR CONEXIÓN]                                      │
│  Estado: ✅ Conectado (último: 14:30:25)                   │
│                                                             │
│  ─── VARIABLES EN TIEMPO REAL ───                          │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Tag               Valor    Última lectura          │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  DB1.Test_Running   TRUE     14:30:45.100           │   │
│  │  DB1.Test_Pass      TRUE     14:30:45.100           │   │
│  │  DB1.Result_FAN     0.45     14:30:45.050           │   │
│  │  DB1.Result_DOOR    2.10     14:30:45.050           │   │
│  │  DB1.Result_FRAME   0.85     14:30:45.050           │   │
│  │  DB1.Result_DRAIN   2.02     14:30:45.050           │   │
│  │  DB1.Result_LIGHT   0.32     14:30:45.050           │   │
│  │  DB1.Result_TOTAL   5.74     14:30:45.050           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  [🔄 Refrescar cada: [500] ms]                            │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 Hipot

```
┌─────────────────────────────────────────────────────────────┐
│  ⚡ Hipot 3865 (RS-232)                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Configuración:                                            │
│  Puerto:   [COM3 ▼]  [🔍 Detectar puertos]                │
│  Baud:     [9600 ▼]                                       │
│  Parity:   [None ▼]                                       │
│  Data Bits:[8 ▼]                                          │
│  Stop Bits:[1 ▼]                                          │
│                                                             │
│  [🔌 PROBAR CONEXIÓN]                                      │
│  Estado: ✅ Conectado                                      │
│                                                             │
│  ─── PRUEBA MANUAL ───                                     │
│                                                             │
│  Comando: [________________] [📤 ENVIAR]                   │
│  Última respuesta: V:120.5 I:1.23 R:456.7 PASS            │
│                                                             │
│  [▶️ START manual]  [⏹️ STOP manual]                       │
└─────────────────────────────────────────────────────────────┘
```

### 2.5 Base de Datos

```
┌─────────────────────────────────────────────────────────────┐
│  💾 BASE DE DATOS                                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [Ver doc 04-base-datos.md — sección 4]                    │
│                                                             │
│  - Config Local (SQL Express)                              │
│  - Config Remoto (SQL Server)                              │
│  - Probar conexión local/remoto                            │
│  - Crear tablas local/remoto                               │
│  - Sync automático                                         │
│  - Log de sync                                             │
└─────────────────────────────────────────────────────────────┘
```

### 2.6 Manual (Prueba Manual)

```
┌─────────────────────────────────────────────────────────────┐
│  🔧 MODO MANUAL                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ⚠️ Solo para pruebas y diagnóstico                       │
│                                                             │
│  ─── PLC ───                                               │
│  Leer DB: [___] Offset: [___] Tamaño: [___] [📤 LEER]     │
│  Escribir DB: [___] Offset: [___] Valor: [___] [📤 ESCRIBIR]│
│                                                             │
│  ─── HIPOT ───                                             │
│  Enviar comando: [________________] [📤 ENVIAR]            │
│  Leer respuesta: [🔄 LEER]                                 │
│                                                             │
│  ─── RESULTADO ───                                         │
│  Última lectura PLC: 0.45A                                 │
│  Última lectura Hipot: V:120.5 I:1.23                     │
└─────────────────────────────────────────────────────────────┘
```

### 2.7 Logs (Debug)

```
┌─────────────────────────────────────────────────────────────┐
│  📋 LOG DE APLICACIÓN                                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [Ver doc 08-logging-debug.md]                             │
│                                                             │
│  - Tabla completa de logs                                  │
│  - Filtros por categoría, nivel, fecha                     │
│  - Búsqueda de texto libre                                 │
│  - Exportar a CSV                                          │
│  - Limpiar logs antiguos                                   │
└─────────────────────────────────────────────────────────────┘
```

## 3. Layout Admin (1280x800)

```
┌─────────────────────────────────────────────────────────────────┐
│ [80px] HEADER                                                   │
│  ⚙️ ADMINISTRACIÓN                       [🔙 VOLVER]          │
├──────────┬──────────────────────────────────────────────────────┤
│          │                                                      │
│ [200px]  │  CONTENIDO (1080px)                                 │
│ SIDEBAR  │                                                      │
│          │  ┌────────────────────────────────────────────────┐ │
│ ┌──────┐ │  │                                                │ │
│ │Users │ │  │  Contenido de la pestaña seleccionada         │ │
│ ├──────┤ │  │                                                │ │
│ │Receta│ │  │                                                │ │
│ ├──────┤ │  │                                                │ │
│ │ PLC  │ │  │                                                │ │
│ ├──────┤ │  │                                                │ │
│ │Hipot │ │  │                                                │ │
│ ├──────┤ │  │                                                │ │
│ │  BD  │ │  │                                                │ │
│ ├──────┤ │  │                                                │ │
│ │Manual│ │  │                                                │ │
│ ├──────┤ │  │                                                │ │
│ │ Logs │ │  │                                                │ │
│ └──────┘ │  └────────────────────────────────────────────────┘ │
│          │                                                      │
├──────────┴──────────────────────────────────────────────────────┤
│ [60px] FOOTER                                                   │
│  Admin: hector1516  │  Sesión: 45 min  │  Logs: 1,234 entries  │
└─────────────────────────────────────────────────────────────────┘
```

## 4. Import / Export de Configuración

### ¿Qué se exporta?

```
config_export_YYYYMMDD_HHMMSS.json
│
├── metadata
│   ├── version: "1.0.0"
│   ├── exported_at: "2025-01-15T14:30:00Z"
│   ├── exported_by: "admin_user"
│   └── station_id: "ESTACION-01"
│
├── recipes                    ← Todas las recetas
│   ├── name, model, description
│   ├── recipe_components      ← Voltajes y componentes
│   └── recipe_totals          ← Totales por voltaje
│
├── configurations             ← Config general del sistema
│   ├── plc_ip, plc_rack, plc_slot
│   ├── hipot_port, hipot_baud
│   ├── label_width, label_height
│   ├── auto_print, print_only_good
│   └── ... (todas las configs)
│
├── label_templates            ← Plantillas de etiquetas
│   ├── name, width_mm, height_mm
│   └── canvas_data (JSON editor visual)
│
├── users                      ← Usuarios (passwords hasheados)
│   ├── name, employee_id, role, active
│   └── password_hash          ← Se exporta (bcrypt hash, no texto plano)
│
└── plc_tags                   ← Tags PLC personalizados
    ├── db_number, offset, data_type
    └── description, unit
```

### ¿Qué NO se exporta?

```
❌ Ejecuciones de prueba     (datos de producción)
❌ PLC readings               (mediciones)
❌ Hipot results              (resultados)
❌ Barcode logs               (logs de escaneo)
❌ App logs                   (logs de aplicación)
❌ Label prints               (historial de impresiones)
❌ Sync logs                  (logs de sincronización)
❌ Import/export logs         (meta-historial)
```

### Pantalla de Import/Export

```
┌─────────────────────────────────────────────────────────────┐
│  📦 IMPORTAR / EXPORTAR CONFIGURACIÓN                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ─── EXPORTAR ───                                          │
│                                                             │
│  Selecciona qué exportar:                                  │
│  [✅] Recetas (23 recetas)                                  │
│  [✅] Configuración general                                 │
│  [✅] Plantillas de etiquetas (5 plantillas)                │
│    [✅] Usuarios (4 usuarios)                                   │
│  [✅] Tags PLC personalizados                               │
│                                                             │
│  [📥 EXPORTAR CONFIGURACIÓN]                               │
│  → Descarga: config_export_20250115_143000.json            │
│                                                             │
│  ─── IMPORTAR ───                                          │
│                                                             │
│  Arrastra archivo aquí o [📂 Seleccionar archivo]          │
│                                                             │
│  Archivo: config_export_20250115_143000.json               │
│  Estación origen: ESTACION-01                               │
│  Versión: 1.0.0                                             │
│  Fecha exportación: 15/01/2025 14:30                        │
│                                                             │
│  Contenido detectado:                                      │
│  📋 23 recetas                                             │
│  ⚙️ 15 configuraciones                                     │
│  🎨 5 plantillas de etiquetas                              │
│  👤 4 usuarios                                             │
│                                                             │
│  Modo de importación:                                      │
│  (○) Agregar (no sobreescribe existentes)                  │
│  (●) Sobreescribir (reemplaza existentes)                  │
│  ( ) Merge (actualiza solo diferentes)                     │
│                                                             │
│  [📤 IMPORTAR]  [❌ Cancelar]                               │
│                                                             │
│  ─── HISTORIAL DE OPERACIONES ───                          │
│                                                             │
│  Fecha           Tipo      Usuario    Archivo              │
│  15/01 14:30     Export    admin      config_v1.json       │
│  14/01 09:15     Import    admin      config_estacion2.json│
│  13/01 16:45     Export    admin      backup_pre_migracion │
└─────────────────────────────────────────────────────────────┘
```

### Flujo de Exportación

```
1. Admin hace clic en [📥 EXPORTAR]
2. App recopila datos de la BD:
   - SELECT * FROM recipes
   - SELECT * FROM recipe_components
   - SELECT * FROM recipe_totals
   - SELECT * FROM configurations
   - SELECT * FROM label_templates
   - SELECT id, name, employee_id, role, active FROM users
   - SELECT * FROM plc_tags
3. Genera JSON con metadata
4. Descarga archivo: config_export_YYYYMMDD_HHMMSS.json
5. Registra en import_export_logs
```

### Flujo de Importación

```
1. Admin selecciona archivo JSON
2. App valida:
   ✅ Formato JSON válido
   ✅ Versión compatible
   ✅ Campos requeridos presentes
   ⚠️ Conflictos detectados (modelos duplicados)
3. Muestra preview de cambios
4. Admin selecciona modo (agregar/sobreescribir/merge)
5. Admin confirma importación
6. App ejecuta en transacción SQL:
   - BEGIN TRANSACTION
   - INSERT/UPDATE datos
   - COMMIT (si todo OK)
   - ROLLBACK (si error)
7. Registra en import_export_logs
8. Muestra resumen
```

### Seguridad de Importación

```
VALIDACIONES:
  ✅ Archivo debe ser JSON válido
  ✅ Campo "version" debe existir
  ✅ Passwords se importan como hash (bcrypt)
  ✅ No se importan datos de producción
  ✅ Conflictos se muestran antes de importar
  ✅ Transacción SQL (todo o nada)

PROTECCIONES:
  🔒 Solo admins pueden importar/exportar
  🔒 Cada operación se registra en logs
  🔒 Si hay conflicto → muestra alerta y deja decidir
  🔒 Usuarios se crean o actualizan conservando el hash
```

### Código Python (Backend)

```python
import json
from datetime import datetime
from sqlalchemy import text

class ConfigManager:
    def __init__(self, db):
        self.db = db
    
    def export_config(self, user_id, include_recipes=True, 
                      include_config=True, include_templates=True,
                      include_users=True, include_plc_tags=True):
        """Exportar configuración a JSON"""
        
        config = {
            "metadata": {
                "version": "1.0.0",
                "exported_at": datetime.now().isoformat(),
                "exported_by": user_id,
                "station_id": self.get_station_id()
            },
            "recipes": [],
            "configurations": [],
            "label_templates": [],
            "users": [],
            "plc_tags": []
        }
        
        if include_recipes:
            recipes = self.db.execute(
                text("SELECT * FROM recipes")
            ).fetchall()
            for recipe in recipes:
                recipe_data = dict(recipe)
                recipe_data['components'] = self.db.execute(
                    text("SELECT * FROM recipe_components WHERE recipe_id = :id"),
                    {"id": recipe.id}
                ).fetchall()
                recipe_data['totals'] = self.db.execute(
                    text("SELECT * FROM recipe_totals WHERE recipe_id = :id"),
                    {"id": recipe.id}
                ).fetchall()
                config["recipes"].append(recipe_data)
        
        if include_config:
            configs = self.db.execute(
                text("SELECT * FROM configurations")
            ).fetchall()
            config["configurations"] = [dict(c) for c in configs]
        
        if include_templates:
            templates = self.db.execute(
                text("SELECT * FROM label_templates")
            ).fetchall()
            config["label_templates"] = [dict(t) for t in templates]
        
        if include_users:
            users = self.db.execute(
                text("SELECT id, name, employee_id, role, active, password_hash FROM users")
            ).fetchall()
            config["users"] = [dict(u) for u in users]
        
        if include_plc_tags:
            tags = self.db.execute(
                text("SELECT * FROM plc_tags")
            ).fetchall()
            config["plc_tags"] = [dict(t) for t in tags]
        
        # Registrar operación
        self.log_operation("export", user_id, config)
        
        return json.dumps(config, indent=2, default=str)
    
    def import_config(self, json_data, user_id, mode="overwrite"):
        """Importar configuración desde JSON"""
        
        config = json.loads(json_data)
        
        # Validaciones
        if "metadata" not in config:
            raise ValueError("Archivo inválido: falta metadata")
        
        conflicts = self.detect_conflicts(config)
        
        if conflicts and mode == "overwrite":
            # Mostrar conflictos al usuario
            return {
                "status": "conflicts",
                "conflicts": conflicts,
                "preview": self.generate_preview(config, mode)
            }
        
        # Ejecutar importación en transacción
        try:
            self.db.begin()
            
            if mode == "overwrite":
                self.import_overwrite(config)
            elif mode == "add":
                self.import_add(config)
            elif mode == "merge":
                self.import_merge(config)
            
            self.db.commit()
            
            # Registrar operación
            self.log_operation("import", user_id, config)
            
            return {"status": "success", "imported": self.count_imported(config)}
            
        except Exception as e:
            self.db.rollback()
            raise e
    
    def detect_conflicts(self, config):
        """Detectar conflictos antes de importar"""
        conflicts = []
        
        for recipe in config.get("recipes", []):
            existing = self.db.execute(
                text("SELECT id FROM recipes WHERE model = :model"),
                {"model": recipe.get("model")}
            ).fetchone()
            
            if existing:
                conflicts.append({
                    "type": "recipe",
                    "field": "model",
                    "value": recipe.get("model"),
                    "existing_id": existing.id
                })
        
        return conflicts
    
    def log_operation(self, operation, user_id, config):
        """Registrar operación en logs"""
        self.db.execute(
            text("""
                INSERT INTO import_export_logs 
                (operation_type, user_id, data_summary, performed_at)
                VALUES (:type, :user, :summary, GETDATE())
            """),
            {
                "type": operation,
                "user": user_id,
                "summary": json.dumps({
                    "recipes_count": len(config.get("recipes", [])),
                    "templates_count": len(config.get("label_templates", [])),
                    "users_count": len(config.get("users", []))
                })
            }
        )
```

### Tabla de Registro

```sql
CREATE TABLE import_export_logs (
  id INT IDENTITY(1,1) PRIMARY KEY,
  operation_type NVARCHAR(10) NOT NULL,  -- 'import' o 'export'
  user_id INT FOREIGN KEY REFERENCES users(id),
  file_name NVARCHAR(200),
  file_size_bytes BIGINT,
  data_summary NVARCHAR(MAX),           -- JSON con conteos
  mode NVARCHAR(20),                    -- 'overwrite', 'add', 'merge'
  conflicts_found INT DEFAULT 0,
  records_imported INT DEFAULT 0,
  status NVARCHAR(20) DEFAULT 'success', -- 'success', 'error', 'cancelled'
  error_message NVARCHAR(500),
  performed_at DATETIME2 DEFAULT GETDATE()
);
```

## 5. Seguridad del Admin

```
1. Login obligatorio para acceder
2. Password hasheado con bcrypt en DB
3. Sesión expira después de 30 min inactividad
4. Solo usuarios con rol "admin" pueden acceder
5. Cada acción queda registrada en app_logs
6. Timeout de sesión visible en footer
```
