# Impresora Zebra ZT231 — HiPotApp

## 1. Especificaciones de la Impresora

| Parámetro | Valor |
|-----------|-------|
| **Modelo** | Zebra ZT231 |
| **Tipo** | Impresora de etiquetas térmica directa / transferencia térmica |
| **Resolución** | 203 dpi (8 dots/mm) |
| **Ancho máximo** | 108 mm (4.25") |
| **Largo máximo** | 991 mm (39") |
| **Velocidad** | hasta 152 mm/s (6"/s) |
| **Conexión** | USB (único método) |
| **Lenguaje** | ZPL II (Zebra Programming Language) |
| **Display** | LCD mono con teclado |

## 2. Conexión USB

```
USB (único método en esta estación)
- Conector: USB-B en la impresora → USB-A en el PC
- Driver: Zebra ZT231 v8
- Plug & play en Windows 11 Pro
- Windows la detecta como "Zebra ZT231"
```

## 3. Driver e Instalación

### Windows 11 Pro

```
1. Descargar driver de: https://www.zebra.com/us/en/support-downloads.html
2. Buscar "ZT231" → Driver v8
3. Instalar → Seleccionar conexión USB
4. Windows lo detecta como "Zebra ZT231"
5. Verificar en: Panel de Control → Dispositivos e Impresoras
```

### Verificar con Zebra Setup Utilities

```
1. Instalar Zebra Setup Utilities (gratis)
2. Abrir → Seleccionar ZT231
3. Abrir prueba de comunicación
4. Enviar: ^XA^FO50,50^ADN,36,20^FDHello Zebra^FS^XZ
5. Si imprime "Hello Zebra" → Conexión OK
```

## 4. Protocolo ZPL II

### Estructura básica ZPL

```
^XA          ← Inicio de etiqueta
^FO(x,y)     ← Posición (x, y) en puntos
^A           ← Fuente
^FDtexto^FS  ← Campo de datos
^BC          ← Código de barras
^XZ          ← Fin de etiqueta
```

### Comandos comunes

```
^XA                          Inicio etiqueta
^XZ                          Fin etiqueta
^POI                          Orientación inversa
^BY2                          Ancho de barra (2x)
^FO100,100                   Posición X=100, Y=100
^ADN,36,20                   Fuente escalable (alto=36, ancho=20)
^A0N,30,30                   Fuente 0 normal, alto=30
^FDHello^FS                  Texto "Hello"
^BCN,100,Y,N,N              Código de barras Code 128
^BY3                         Ancho de barra
^BQ2,3,N,N                  QR Code
^PR4                         Velocidad de impresión (4 pulgadas/seg)
^LS0                         Offset lateral
^LL400                       Largo de etiqueta (400 puntos)
^PW400                       Ancho de impresión (400 puntos)
```

## 5. Etiqueta Diseñada

### Diseño propuesto (100mm x 60mm)

```
┌─────────────────────────────────────────────────────────────┐
│                    HIPOD - RESULTADO DE PRUEBA              │
│                                                             │
│  ┌─────────────────┐  ┌─────────────────────────────────┐  │
│  │  MODELO         │  │  ESTADO:    ✅ BUENO            │  │
│  │  REF-1234       │  │  ─────────────────────────────  │  │
│  └─────────────────┘  │  SERIAL:    SN-ABC123           │  │
│                       │  BATCH:     BATCH-001           │  │
│  ┌─────────────────┐  │  OPERADOR:  Juan Pérez (JP001) │  │
│  │  ████  ████     │  │  FECHA:     2025-01-15         │  │
│  │  ████  ████     │  │  HORA:      14:30:55           │  │
│  │  ████  ████     │  │  DURACIÓN:  55 seg             │  │
│  │  [QR CODE]      │  └─────────────────────────────────┘  │
│  └─────────────────┘                                        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  RESULTADOS                                         │   │
│  │  PLC: PASS  |  HIPOT: PASS  |  TOTAL: 5.74A       │   │
│  │  Voltaje: 120VAC  |  FAN: 0.45A  |  DOOR: 2.10A   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ═══════════════════════════════════════════════════════    │
│  [████████████████████████████████████████████████]         │
│  [█████ CODE 128: REF-1234-SN-ABC123-BATCH001 █████]       │
│  [████████████████████████████████████████████████]         │
└─────────────────────────────────────────────────────────────┘
```

### Contenido de la etiqueta

| Campo | Fuente | Ejemplo |
|-------|--------|---------|
| **Título** | Fijo | "HIPOD - RESULTADO DE PRUEBA" |
| **Estado** | `executions.final_status` | "✅ BUENO" o "❌ MALO" |
| **Modelo** | `recipes.model` | "REF-1234" |
| **Serie** | `executions.serial_number` | "SN-ABC123" |
| **Batch** | `batches.batch_number` | "BATCH-001" |
| **Operador** | `operators.name + employee_id` | "Juan Pérez (JP001)" |
| **Fecha** | `executions.plc_start_time` | "2025-01-15" |
| **Hora fin** | `executions.hipot_end_time` | "14:30:55" |
| **Duración** | `executions.total_duration` | "55 seg" |
| **PLC** | `executions.plc_pass_fail` | "PASS" o "FAIL" |
| **Hipot** | `executions.hipot_pass_fail` | "PASS" o "FAIL" |
| **Total corriente** | `plc_readings` (TOTAL) | "5.74A" |
| **Voltaje** | `recipe_components` | "120VAC" |
| **Componentes** | `plc_readings` | "FAN: 0.45A, DOOR: 2.10A" |
| **QR Code** | Datos concatenados | URL o JSON con todos los datos |
| **Código barras** | Concatenado | "REF-1234-SN-ABC123-BATCH001" |

## 6. Editor Visual de Etiquetas (Drag & Drop)

### Tecnología: react-konva

```
react-konva = Canvas HTML5 + React
- Arrastrar objetos libremente en el canvas
- Redimensionar con mouse/touch (80px mínimo para touch)
- Seleccionar y mover
- Capas (fondo, texto, barcode, QR)
- Panel de propiedades dinámico
- Preview en tiempo real
- Exportar a ZPL automáticamente
- Guardar/cargar plantillas
```

### Layout del Editor (1280x800)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  🎨 EDITOR DE ETIQUETAS                              [💾 Guardar] [🖨️ Probar]│
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─── PALETA (200px) ──┐  ┌─── CANVAS (780px) ─────┐  ┌─── PROPS (300px)─┐ │
│  │                      │  │                          │  │                   │ │
│  │  📋 ARRASTRA AQUÍ:   │  │  ┌────────────────────┐  │  │  Elemento:        │ │
│  │                      │  │  │                    │  │  │  Texto Fijo       │ │
│  │  ─── TEXTO ───       │  │  │  HIPOD RESULTADO  │  │  │                   │ │
│  │  [📝 Texto fijo]    │  │  │  REF-1234          │  │  │  Contenido:       │ │
│  │  [📅 Fecha]         │  │  │  SN-ABC123         │  │  │  [HIPOD RESULTADO]│ │
│  │  [⏰ Hora]          │  │  │  ✅ BUENO          │  │  │                   │ │
│  │  [👤 Operador]      │  │  │  [BARCODE]          │  │  │  ○ Texto fijo     │ │
│  │  [📦 Modelo]        │  │  │  [QR]               │  │  │  ○ Campo dinámico │ │
│  │  [🔢 Serie]         │  │  │                    │  │  │    Modelo ↓        │ │
│  │  [🏭 Batch]         │  │  └────────────────────┘  │  │    Serie ↓         │ │
│  │  [⏱️ Duración]      │  │                          │  │    Batch ↓         │ │
│  │  [✅ Estado]        │  │  ─── CANVAS ───          │  │    Operador ↓      │ │
│  │                      │  │                          │  │    Fecha ↓         │ │
│  │  ─── RESULTADOS ──  │  │  (arrastra desde la     │  │    Hora ↓          │ │
│  │  [⚡ Voltaje]       │  │   paleta al canvas)      │  │    ...             │ │
│  │  [🔌 FAN]           │  │                          │  │                   │ │
│  │  [🚪 DOOR]          │  │  ─── CONTROLES ───       │  │  ─── ESTILO ───   │ │
│  │  [🔧 FRAME]         │  │                          │  │                   │ │
│  │  [💧 DRAIN]         │  │  Zoom: [100%] [🔍+][🔍-]│  │  Fuente: [Arial ▼]│ │
│  │  [❄️ DEFROST]       │  │  [↩️ Deshacer] [↪️ Rehacer]│ │  Tamaño: [24]    │ │
│  │  [💡 LIGHT]         │  │  [👁️ Preview] [🖨️ Probar]│  │  Negrita: [✅]    │ │
│  │  [📊 TOTAL]         │  │                          │  │  Color: [Negro ▼] │ │
│  │                      │  │                          │  │                   │ │
│  │  ─── CÓDIGOS ───    │  │                          │  │  ─── POSICIÓN ── │ │
│  │  [📊 Barcode 1D]    │  │                          │  │  X: [50]         │ │
│  │  [📱 QR Code]       │  │                          │  │  Y: [30]         │ │
│  │                      │  │                          │  │  Ancho: [auto]    │ │
│  │  ─── VISUALES ───   │  │                          │  │  Alto: [auto]     │ │
│  │  [▬ Línea]          │  │                          │  │                   │ │
│  │  [□ Rectángulo]     │  │                          │  │  [🗑️ Eliminar]   │ │
│  │  [🖼️ Logo]          │  │                          │  │                   │ │
│  │                      │  │                          │  └───────────────────┘ │
│  │  ─── PLANTILLAS ──  │  │                          │                         │
│  │  [📄 Predeterminada]│  │                          │                         │
│  │  [📄 Modelo A]      │  │                          │                         │
│  │  [📄 Modelo B]      │  │                          │                         │
│  │  [+ Nueva]          │  │                          │                         │
│  └──────────────────────┘  └──────────────────────────┘                         │
│                                                                               │
│  ─── BARRA INFERIOR ──────────────────────────────────────────────────────── │
│  Tamaño: Ancho [100] mm  Alto [60] mm   [🔄 Aplicar]   │  Plantilla: [Predeterminada ▼]  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Cómo funciona el Drag & Drop

```
PASO 1: Seleccionar elemento en la paleta
   └─ Click en "📝 Texto fijo" en la columna izquierda

PASO 2: Aparece en el canvas
   └─ Cuadro de texto aparece en posición central del canvas
   └─ Seleccionado automáticamente (borde azul, puntos de redimensión)

PASO 3: Arrastrar a posición deseada
   └─ Click + mantener + mover mouse/touch
   └─ Soltar en la posición final
   └─ Guías de alineación aparecen automáticamente

PASO 4: Editar propiedades
   └─ Panel derecho muestra propiedades del elemento seleccionado
   └─ Cambiar contenido, fuente, tamaño, color, posición

PASO 5: Redimensionar (si aplica)
   └─ Arrastrar puntos de esquina para cambiar tamaño
   └─ Para touch: puntos de al menos 40x40px

PASO 6: Guardar plantilla
   └─ [💾 Guardar] → Guarda JSON con todos los objetos
   └─ Se puede cargar después para editar
```

### Elementos de la Paleta

#### Texto

| Elemento | Placeholder | Se rellena con |
|----------|-------------|----------------|
| 📝 Texto fijo | (texto libre) | Lo que escriba el admin |
| 📦 Modelo | `{modelo}` | `recipes.model` |
| 🔢 Serie | `{serie}` | `executions.serial_number` |
| 🏭 Batch | `{batch}` | `batches.batch_number` |
| 👤 Operador | `{operador}` | `operators.name` |
| 📅 Fecha | `{fecha}` | `executions.plc_start_time` |
| ⏰ Hora | `{hora}` | `executions.hipot_end_time` |
| ⏱️ Duración | `{duracion}` | `executions.total_duration` |
| ✅ Estado | `{estado}` | `executions.final_status` |

#### Resultados

| Elemento | Placeholder | Se rellena con |
|----------|-------------|----------------|
| ⚡ Voltaje | `{voltaje}` | `plc_readings.voltage` |
| 🔌 FAN | `{fan}` | `plc_readings` (FAN) |
| 🚪 DOOR | `{door}` | `plc_readings` (DOOR) |
| 🔧 FRAME | `{frame}` | `plc_readings` (FRAME) |
| 💧 DRAIN | `{drain}` | `plc_readings` (DRAIN) |
| ❄️ DEFROST | `{defrost}` | `plc_readings` (DEFROST) |
| 💡 LIGHT | `{light}` | `plc_readings` (LIGHT) |
| 📊 TOTAL | `{total}` | `plc_readings` (TOTAL) |
| ⚙️ PLC Status | `{plc_status}` | `executions.plc_pass_fail` |
| ⚡ Hipot Status | `{hipot_status}` | `executions.hipot_pass_fail` |

#### Códigos

| Elemento | Descripción |
|----------|-------------|
| 📊 Barcode 1D | Code 128 con datos concatenados |
| 📱 QR Code | JSON con todos los datos de la prueba |

#### Visuales

| Elemento | Descripción |
|----------|-------------|
| ▬ Línea | Separador horizontal |
| □ Rectángulo | Marco o fondo |
| 🖼️ Logo | Imagen subida por el admin |

### Panel de Propiedades (cambia según elemento seleccionado)

```
┌─── PROPIEDADES ────────────────────────┐
│                                          │
│  Elemento: Texto Fijo                    │
│                                          │
│  ─── CONTENIDO ───                       │
│  ○ Texto fijo                            │
│    [HIPOD - RESULTADO]                   │
│  ○ Campo dinámico                        │
│    [Modelo ▼]                            │
│                                          │
│  ─── ESTILO ───                          │
│  Fuente:   [Arial ▼]                    │
│  Tamaño:   [24] px                      │
│  Negrita:  [✅]                          │
│  Cursiva:  [ ]                          │
│  Color:    [Negro ▼]                    │
│                                          │
│  ─── POSICIÓN ───                        │
│  X: [50]    Y: [30]                     │
│  Ancho: [auto]  Alto: [auto]            │
│  Rotación: [0]°                         │
│                                          │
│  ─── ACCIONES ───                        │
│  [📋 Duplicar]  [⬆️ Subir]  [⬇️ Bajar]  │
│  [🗑️ Eliminar]                          │
│                                          │
└──────────────────────────────────────────┘
```

### Funciones del Editor

| Función | Descripción |
|---------|-------------|
| **Drag & Drop** | Arrastrar campos desde paleta hasta canvas |
| **Mover objetos** | Click + arrastrar en el canvas |
| **Redimensionar** | Arrastrar puntos de esquina |
| **Seleccionar** | Click en el objeto |
| **Propiedades** | Panel derecho dinámico según elemento |
| **Campos dinámicos** | Placeholders que se rellenan al imprimir |
| **Preview** | Vista previa en tiempo real |
| **Guardar plantilla** | JSON con todos los objetos |
| **Cargar plantilla** | Seleccionar plantilla guardada |
| **Múltiples plantillas** | Admin puede crear varias |
| **Tamaño personalizado** | Ancho x Alto en mm |
| **Zoom** | Acercar/alejar canvas |
| **Deshacer/Rehacer** | Historial de cambios |
| **Probar impresión** | Enviar ZPL a la Zebra |
| **Exportar ZPL** | Descargar archivo ZPL |
| **Touch-friendly** | Elementos mínimos 80x80px |

### Guardar/Cargar Plantillas

```sql
CREATE TABLE label_templates (
  id INT IDENTITY(1,1) PRIMARY KEY,
  name NVARCHAR(100) NOT NULL,
  width_mm INT DEFAULT 100,
  height_mm INT DEFAULT 60,
  canvas_data NVARCHAR(MAX),  -- JSON con todos los objetos del canvas
  is_default BIT DEFAULT 0,
  created_at DATETIME2 DEFAULT GETDATE(),
  updated_at DATETIME2 DEFAULT GETDATE()
);
```

### Ejemplo de canvas_data (JSON guardado)

```json
{
  "label": {
    "width_mm": 100,
    "height_mm": 60,
    "bg_color": "#FFFFFF"
  },
  "objects": [
    {
      "id": "obj_1",
      "type": "text",
      "x": 50, "y": 20,
      "content": "HIPOD - RESULTADO",
      "is_dynamic": false,
      "font_family": "Arial",
      "font_size": 28,
      "bold": true,
      "color": "#000000"
    },
    {
      "id": "obj_2",
      "type": "text",
      "x": 50, "y": 60,
      "content": "Modelo: {modelo}",
      "is_dynamic": true,
      "dynamic_field": "modelo",
      "font_family": "Arial",
      "font_size": 18,
      "bold": false,
      "color": "#000000"
    },
    {
      "id": "obj_3",
      "type": "text",
      "x": 50, "y": 85,
      "content": "Serie: {serie}",
      "is_dynamic": true,
      "dynamic_field": "serie",
      "font_family": "Arial",
      "font_size": 18,
      "bold": false,
      "color": "#000000"
    },
    {
      "id": "obj_4",
      "type": "text",
      "x": 50, "y": 115,
      "content": "Estado: {estado}",
      "is_dynamic": true,
      "dynamic_field": "estado",
      "font_family": "Arial",
      "font_size": 20,
      "bold": true,
      "color": "#4CAF50"
    },
    {
      "id": "obj_5",
      "type": "line",
      "x": 50, "y": 145,
      "x2": 450, "y2": 145,
      "color": "#000000",
      "thickness": 2
    },
    {
      "id": "obj_6",
      "type": "barcode",
      "x": 50, "y": 300,
      "code_type": "CODE128",
      "data_source": "{modelo}-{serie}-{batch}",
      "height": 80
    },
    {
      "id": "obj_7",
      "type": "qrcode",
      "x": 400, "y": 280,
      "size": 100,
      "data_source": "json_full"
    }
  ]
}
```

### Conversión de Canvas a ZPL

```python
def canvas_to_zpl(template_json, execution_data):
    """Convierte plantilla del canvas a ZPL listo para imprimir"""
    
    # Factores de conversión (203 dpi = 8 puntos por mm)
    MM_TO_DOTS = 203 / 25.4  # 8.0
    
    template = json.loads(template_json)
    width_pts = int(template['label']['width_mm'] * MM_TO_DOTS)
    height_pts = int(template['label']['height_mm'] * MM_TO_DOTS)
    
    zpl = f"^XA\n^PW{width_pts}\n^LL{height_pts}\n\n"
    
    for obj in template['objects']:
        x = int(obj['x'] * MM_TO_DOTS)
        y = int(obj['y'] * MM_TO_DOTS)
        
        if obj['type'] == 'text':
            text = obj['content']
            if obj.get('is_dynamic'):
                text = replace_placeholder(text, execution_data)
            
            size = obj.get('font_size', 24)
            bold = "^A0N" if obj.get('bold') else "^A0N"
            
            zpl += f"^FO{x},{y}{bold},{size},{size}^FD{text}^FS\n"
        
        elif obj['type'] == 'barcode':
            data = replace_placeholder(obj['data_source'], execution_data)
            height = obj.get('height', 80)
            zpl += f"^FO{x},{y}^BY3^BCN,{height},Y,N,N^FD{data}^FS\n"
        
        elif obj['type'] == 'qrcode':
            data = replace_placeholder(obj['data_source'], execution_data)
            zpl += f"^FO{x},{y}^BQ2,3,N,N^FD{data}^FS\n"
        
        elif obj['type'] == 'line':
            x2 = int(obj.get('x2', obj['x'] + 100) * MM_TO_DOTS)
            thickness = obj.get('thickness', 2)
            zpl += f"^FO{x},{y}^GB{x2-x},{thickness},{thickness}^FS\n"
    
    zpl += "\n^XZ"
    return zpl

def replace_placeholder(text, data):
    """Reemplaza {placeholder} con datos reales"""
    replacements = {
        '{modelo}': data.get('model', ''),
        '{serie}': data.get('serial_number', ''),
        '{batch}': data.get('batch_number', ''),
        '{operador}': data.get('operator_name', ''),
        '{fecha}': data.get('date', ''),
        '{hora}': data.get('time', ''),
        '{duracion}': str(data.get('duration', 0)),
        '{estado}': data.get('final_status', ''),
        '{voltaje}': data.get('voltage', ''),
        '{fan}': str(data.get('fan_value', 0)),
        '{door}': str(data.get('door_value', 0)),
        '{frame}': str(data.get('frame_value', 0)),
        '{drain}': str(data.get('drain_value', 0)),
        '{defrost}': str(data.get('defrost_value', 0)),
        '{light}': str(data.get('light_value', 0)),
        '{total}': str(data.get('total_value', 0)),
        '{plc_status}': data.get('plc_pass', ''),
        '{hipot_status}': data.get('hipot_pass', ''),
    }
    for placeholder, value in replacements.items():
        text = text.replace(placeholder, value)
    return text
```

## 7. Configuración en Admin

### Pestana: Configuración → Impresora

```
┌─────────────────────────────────────────────────────────────┐
│  🖨️ IMPRESORA ZEBRA                                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Conexión:                                                  │
│  Método:   USB (fijo)                                      │
│  Puerto:   [Zebra ZT231] (auto-detectado por Windows)     │
│                                                             │
│  [🔌 PROBAR CONEXIÓN]  [🖨️ IMPRIMIR PRUEBA]              │
│  Estado: ✅ Conectada                                       │
│                                                             │
│  ─── CONFIGURACIÓN ETIQUETA ───                            │
│                                                             │
│  Plantilla activa: [Predeterminada ▼]                      │
│  [🎨 ABRIR EDITOR]                                         │
│                                                             │
│  ─── IMPRESIÓN AUTOMÁTICA ───                              │
│                                                             │
│  Imprimir al completar: [✅]                               │
│  Solo si pasa:         [✅] (no imprimir si FALLA)         │
│  Copias:               [1]                                │
│                                                             │
│  [💾 GUARDAR CONFIGURACIÓN]                                │
└─────────────────────────────────────────────────────────────┘
```

## 8. Reimpresión desde Admin

### Flujo de reimpresión

```
1. Admin va a pestaña "📋 Historial" o "🖨️ Reimpresión"
2. Busca ejecución por: fecha, serial, batch, modelo
3. Selecciona la ejecución
4. Preview de la etiqueta (usa la plantilla activa)
5. [🖨️ REIMPRIMIR] → Genera ZPL → Envía a la Zebra
```

### Pantalla de reimpresión

```
┌─────────────────────────────────────────────────────────────────┐
│  🖨️ REIMPRESIÓN DE ETIQUETAS                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Buscar: [________________] [🔍]                               │
│  Filtros: Fecha: [__/__/____] a [__/__/____]                   │
│           Modelo: [________] Serie: [________]                  │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │  Fecha       Modelo    Serie      Batch      Estado       │ │
│  ├───────────────────────────────────────────────────────────┤ │
│  │  15/01/2025  REF-1234  SN-ABC123  BATCH-001  ✅ GOOD     │ │
│  │  15/01/2025  REF-1234  SN-DEF456  BATCH-001  ❌ BAD      │ │
│  │  14/01/2025  REF-5678  SN-GHI789  BATCH-002  ✅ GOOD     │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  Seleccionada: REF-1234 / SN-ABC123 / BATCH-001                │
│                                                                 │
│  ┌─── PREVIEW ──────────────┐  ┌─────────────────────────┐   │
│  │  HIPOD - RESULTADO       │  │  Plantilla: [Default ▼] │   │
│  │  REF-1234                │  │  Copias: [1]            │   │
│  │  SN-ABC123               │  │                         │   │
│  │  ✅ BUENO                │  │  [🖨️ REIMPRIMIR]       │   │
│  │  [QR] [BARCODE]          │  │  [📥 EXPORTAR PDF]      │   │
│  └──────────────────────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 9. Código de Impresión

```python
import subprocess
import tempfile
import os
import json

class ZebraPrinter:
    def __init__(self):
        self.printer_name = "Zebra ZT231"
        self.connected = False
    
    def connect(self):
        """Verificar que la impresora esté disponible en Windows"""
        try:
            result = subprocess.run(
                ['wmic', 'printer', 'get', 'name'],
                capture_output=True, text=True
            )
            if self.printer_name in result.stdout:
                self.connected = True
                return True
            return False
        except:
            return False
    
    def print_zpl(self, zpl_data):
        """Enviar datos ZPL a la impresora USB"""
        if not self.connected:
            return False
        
        try:
            with tempfile.NamedTemporaryFile(
                mode='w', suffix='.zpl', delete=False
            ) as f:
                f.write(zpl_data)
                temp_path = f.name
            
            subprocess.run(
                ['print', '/d:' + self.printer_name, temp_path],
                check=True, shell=True
            )
            
            os.unlink(temp_path)
            return True
            
        except Exception as e:
            print(f"Error de impresión: {e}")
            return False
    
    def print_from_template(self, template_json, execution_data):
        """Imprimir usando plantilla del editor visual"""
        zpl = canvas_to_zpl(template_json, execution_data)
        return self.print_zpl(zpl)
    
    def test_print(self):
        """Imprimir etiqueta de prueba"""
        zpl = """
^XA
^FO50,50^ADN,36,20^FDHello Zebra ZT231^FS
^FO50,100^ADN,24,12^FDHiPod Test Print^FS
^FO50,150^ADN,24,12^FDUSB Connection OK^FS
^XZ
"""
        return self.print_zpl(zpl)
```

## 10. Tablas de BD

```sql
-- Plantillas de etiquetas (diseñadas en el editor visual)
CREATE TABLE label_templates (
  id INT IDENTITY(1,1) PRIMARY KEY,
  name NVARCHAR(100) NOT NULL,
  width_mm INT DEFAULT 100,
  height_mm INT DEFAULT 60,
  canvas_data NVARCHAR(MAX),  -- JSON con objetos del canvas
  is_default BIT DEFAULT 0,
  created_at DATETIME2 DEFAULT GETDATE(),
  updated_at DATETIME2 DEFAULT GETDATE()
);

-- Registro de impresiones
CREATE TABLE label_prints (
  id INT IDENTITY(1,1) PRIMARY KEY,
  execution_id INT FOREIGN KEY REFERENCES executions(id),
  template_id INT FOREIGN KEY REFERENCES label_templates(id),
  printed_by NVARCHAR(50) DEFAULT 'auto',
  printed_at DATETIME2 DEFAULT GETDATE(),
  printer_model NVARCHAR(50) DEFAULT 'Zebra ZT231',
  connection_type NVARCHAR(10) DEFAULT 'usb',
  zpl_data NVARCHAR(MAX),
  status NVARCHAR(20),
  error_message NVARCHAR(200),
  copies INT DEFAULT 1
);
```

## 11. Resumen

| Función | Estado |
|---------|--------|
| Impresión automática al pasar prueba | ✅ Configurable |
| Solo imprimir si GOOD (no BAD) | ✅ Opcional |
| Editor visual drag & drop | ✅ react-konva canvas |
| Mover objetos en canvas | ✅ Click + arrastrar |
| Redimensionar objetos | ✅ Arrastrar esquinas |
| Panel de propiedades | ✅ Dinámico según elemento |
| Campos dinámicos ({modelo}, {serie}, etc.) | ✅ 20+ placeholders |
| Texto fijo | ✅ Lo que escriba el admin |
| Barcode 1D (Code 128) | ✅ Datos concatenados |
| QR Code | ✅ JSON completo |
| Líneas y rectángulos | ✅ Separadores y marcos |
| Logo del cliente | ✅ Imagen subida |
| Guardar/cargar plantillas | ✅ Tabla label_templates |
| Múltiples plantillas | ✅ Admin puede crear varias |
| Vista previa | ✅ En tiempo real |
| Zoom | ✅ Acercar/alejar |
| Deshacer/Rehacer | ✅ Historial |
| Probar impresión | ✅ Botón de prueba |
| Reimpresión desde admin | ✅ Con búsqueda |
| Touch-friendly | ✅ Elementos 80x80px mínimo |
| Conversión automática a ZPL | ✅ canvas_to_zpl() |
| **Total tablas BD: 16** | (14 + label_templates + label_prints) |
