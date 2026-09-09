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

## 4. Seguridad del Admin

```
1. Login obligatorio para acceder
2. Password hasheado con bcrypt en DB
3. Sesión expira después de 30 min inactividad
4. Solo usuarios con rol "admin" pueden acceder
5. Cada acción queda registrada en app_logs
6. Timeout de sesión visible en footer
```
