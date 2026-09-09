# UI Touch (10.1" 1280x800) — HiPotApp

## 1. Especificaciones de Pantalla

| Parámetro | Valor |
|-----------|-------|
| **Tamaño** | 10.1" |
| **Resolución** | 1280 x 800 px |
| **Ratio** | 16:10 |
| **Touch** | Capacitivo multitouch |
| **Entrada** | Dedo directo O guantes |

## 2. Reglas de Diseño Touch

### Tamaños mínimos de elementos

| Elemento | Tamaño mínimo | Recomendado |
|----------|---------------|-------------|
| **Botón acción** | 64 x 64 px | 80 x 80 px |
| **START/STOP** | 120 x 60 px | 140 x 70 px |
| **Campo de texto** | 48 px alto | 56 px alto |
| **Texto etiqueta** | 14 px | 16-18 px |
| **Texto valor** | 24 px | 32-48 px |
| **Icono** | 24 x 24 px | 32 x 32 px |
| **Espaciado entre botones** | 8 px | 12-16 px |

### Contraste y legibilidad

```
- Fondo oscuro (#1a1a2e) con texto claro (#ffffff)
- Contraste mínimo 4.5:1 (WCAG AA)
- Sin gradientes sutiles en fondos
- Bordes gruesos (2-3px) en botones
- Estados claros: hover/press con cambio de color visible
```

### Guantes

```
- Botones más grandes (80x80 mínimo)
- Mayor espacio entre elementos (16px mínimo)
- Sin gestos finos (swipe, pinch para acciones críticas)
- Solo single-tap para todo
- Feedback visual claro al tocar (cambio de color inmediato)
- Sin doble-tap requerido
```

## 3. Layout Principal (Operador)

### Resolución: 1280 x 800

```
┌─────────────────────────────────────────────────────────────────┐
│ [80px] HEADER                                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  PASO: [1] Modelo > [2] Serie > [3] Batch > [4] Prueba  │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │  │
│  │  │  1      │ │  2      │ │  3      │ │  4      │       │  │
│  │  │ MODELO  │ │ SERIE   │ │ BATCH   │ │ PRUEBA  │       │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│ [620px] CONTENIDO PRINCIPAL                                     │
│  ┌─────────────────────────────┬─────────────────────────────┐  │
│  │  COLUMNA IZQUIERDA (640px)  │  COLUMNA DERECHA (640px)    │  │
│  │                             │                              │  │
│  │  ┌───────────────────────┐  │  ┌───────────────────────┐  │  │
│  │  │  VALORES EN TIEMPO    │  │  │  RECETA CARGADA       │  │  │
│  │  │  REAL                 │  │  │                        │  │  │
│  │  │  ┌─────────────────┐  │  │  │  Modelo: REF-1234     │  │  │
│  │  │  │  FAN: 0.45 A ✅ │  │  │  │  Serie: ABC123        │  │  │
│  │  │  │  DOOR: 2.1 A ✅ │  │  │  │  Voltaje: 120VAC      │  │  │
│  │  │  │  FRAME: 0.8 A ✅│  │  │  │  Componentes: 5       │  │  │
│  │  │  │  DRAIN: 2.0 A ✅│  │  │  │  Batch: OPER-001      │  │  │
│  │  │  │  LIGHT: 0.3 A ✅│  │  │  └───────────────────────┘  │  │
│  │  │  └─────────────────┘  │  │                              │  │
│  │  │                       │  │  ┌───────────────────────┐  │  │
│  │  │  ┌─────────────────┐  │  │  │  CONTROLES            │  │  │
│  │  │  │  GRÁFICA        │  │  │  │                        │  │  │
│  │  │  │  EN TIEMPO REAL │  │  │  │  [▶️ START] [⏹️ STOP] │  │  │
│  │  │  │                 │  │  │  │  [📊 GRÁF] [📋 HIST]  │  │  │
│  │  │  │  (barras o      │  │  │  │  [⚙️ CONFIG]          │  │  │
│  │  │  │   líneas)       │  │  │  └───────────────────────┘  │  │
│  │  │  └─────────────────┘  │  │                              │  │
│  │  └───────────────────────┘  │  ┌───────────────────────┐  │  │
│  │                             │  │  ESTADO                │  │  │
│  │                             │  │  PLC: 🟢 Conectado     │  │  │
│  │                             │  │  HIPOT: 🟢 Conectado   │  │  │
│  │                             │  │  DB: 🟢 Local          │  │  │
│  │                             │  │  RED: 🟢 Online        │  │  │
│  │                             │  └───────────────────────┘  │  │
│  └─────────────────────────────┴─────────────────────────────┘  │
│                                                                 │
│ [100px] FOOTER                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  HORA: 14:30:25  │ OPERADOR: Juan Pérez │ STATUS: GOOD  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 4. Estados Visuales

### Fondo por Estado

```
IDLE (Esperando)         → Gris claro #e0e0e0
ESCANEANDO               → Gris oscuro #b0b0b0
RECETA CARGADA           → Azul claro #e3f2fd
LISTO (Esperando batch)  → Amarillo #fff3e0
PLC CORRIENDO            → Azul #bbdefb
HIPOT CORRIENDO          → Morado #e1bee7
COMPLETADO BUENO         → Verde #c8e6c9
COMPLETADO MALO          → Rojo #ffcdd2
ERROR                    → Rojo oscuro #ef9a9a
```

### Indicadores de Conexión

```
🟢 Verde pulsante  = Conectado y activo
🟡 Amarillo fijo   = Conectando
🔴 Rojo fijo       = Desconectado
⚫ Gris            = No configurado
```

### Animaciones (Framer Motion)

```
- Transición de fondo: 0.5s ease-in-out
- Aparición de valores: 0.3s fade-in
- Conteo de valores: count-up/down 0.5s
- Barra de progreso: 0.3s width transition
- Toast notifications: 0.3s slide-in desde derecha
- Botones: scale 0.95 al presionar
- Pulse de conexión: 1s infinite loop
```

## 5. Componentes de UI

### Botón START (140 x 70 px)

```
┌───────────────────────┐
│                       │
│   ▶️  INICIAR PRUEBA  │
│                       │
└───────────────────────┘

Estados:
- Normal:    Fondo #4caf50, Texto #ffffff
- Hover:     Fondo #66bb6a
- Press:     Fondo #388e3c, Scale 0.95
- Disabled:  Fondo #9e9e9e, Texto #757575
```

### Botón STOP (140 x 70 px)

```
┌───────────────────────┐
│                       │
│   ⏹️  PARAR PRUEBA   │
│                       │
└───────────────────────┘

Estados:
- Normal:    Fondo #f44336, Texto #ffffff
- Hover:     Fondo #ef5350
- Press:     Fondo #d32f2f, Scale 0.95
- Disabled:  Fondo #9e9e9e, Texto #757575
```

### Campo de Escaneo (ancho completo, 56 px alto)

```
┌───────────────────────────────────────────────────────────────┐
│  📷 ESCANEE CÓDIGO DE BARRAS AQUÍ                           │
└───────────────────────────────────────────────────────────────┘

- Fondo: #ffffff
- Borde: 3px #2196f3
- Texto: 18px, centrado
- Placeholder: gris #9e9e9e
- Focus: Borde azul #1976d2, shadow
```

### Tarjeta de Valor (组件)

```
┌─────────────────────────────┐
│  FAN                        │
│  ┌───────────────────────┐  │
│  │      0.45 A           │  │
│  └───────────────────────┘  │
│  ✅ OK  (0.3 - 1.5 A)      │
└─────────────────────────────┘

Estados:
- OK:     Borde verde #4caf50, Icono ✅
- FAIL:   Borde rojo #f44336, Icono ❌, Fondo rojo claro
- ACTIVE: Borde azul #2196f3, Pulse animation
- IDLE:   Borde gris #9e9e9e
```

## 6. Pantalla de Administrador

### Acceso

```
Botón "⚙️ CONFIG" en pantalla principal
→ Solicita password de admin
→ Abre ventana admin (misma pantalla, 1280x800)
```

### Layout Admin

```
┌─────────────────────────────────────────────────────────────────┐
│ [80px] HEADER                                                   │
│  ⚙️ ADMINISTRACIÓN    [🔙 VOLVER]                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ [620px] CONTENIDO                                               │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  PESTAÑAS (tabs grandes para touch)                       │  │
│  │  [Usuarios] [Recetas] [PLC] [Hipot] [BD] [Manual]       │  │
│  ├───────────────────────────────────────────────────────────┤  │
│  │                                                           │  │
│  │  CONTENIDO DE PESTAÑA SELECCIONADA                        │  │
│  │                                                           │  │
│  │  Ejemplo (Pestaña BD):                                    │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │  LOCAL: .\SQLEXPRESS - HiPotDB     [🔌] [🔄]      │  │  │
│  │  │  REMOTO: 192.168.1.200 - HiPotDB_C [🔌] [🔄]      │  │  │
│  │  │                                                     │  │  │
│  │  │  SYNC: Local → Remoto cada 5 min  [🔄 SYNC AHORA] │  │  │
│  │  │  Última sync: 14:30:00                           │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
│ [100px] FOOTER                                                  │
│  Admin: hector1516 │ Sesión activa: 45 min                     │
└─────────────────────────────────────────────────────────────────┘
```

## 7. Responsive para 1280x800

```
// Breakpoints (solo para esta resolución)
- 1280x800: Layout completo (2 columnas)
- 1024x768: Layout comprimido (2 columnas, menos spacing)
- 800x600:  Layout apilado (1 columna)

// Nota: La app siempre arranca en 1280x800
// No necesita ser responsive multi-device
// Solo necesita ajustar si la ventana se redimensiona
```

## 8. Tema de Colores

```css
:root {
  /* Primarios */
  --primary: #1976d2;
  --primary-dark: #1565c0;
  --primary-light: #42a5f5;

  /* Secundarios */
  --secondary: #9c27b0;
  --secondary-dark: #7b1fa2;
  --secondary-light: #ce93d8;

  /* Estado */
  --success: #4caf50;
  --warning: #ff9800;
  --error: #f44336;
  --info: #2196f3;

  /* Fondos por estado */
  --bg-idle: #e0e0e0;
  --bg-scanning: #b0b0b0;
  --bg-loaded: #e3f2fd;
  --bg-ready: #fff3e0;
  --bg-plc-running: #bbdefb;
  --bg-hipot-running: #e1bee7;
  --bg-pass: #c8e6c9;
  --bg-fail: #ffcdd2;

  /* Texto */
  --text-primary: #ffffff;
  --text-secondary: #b0b0b0;
  --text-disabled: #757575;

  /* Touch */
  --touch-min-size: 64px;
  --touch-spacing: 12px;
  --touch-border-radius: 8px;
}
```
