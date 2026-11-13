# Hardware — HiPotApp

## 1. Panel PC (Estación de Trabajo)
... (rest of original content would be here, but we are appending the new PLC addressing info)

---

## 7. Tabla de Direccionamiento PLC (Siemens S7-1200) — Actualizado 13 Nov 2026

### Salidas Digitales (Q)
| Dirección | Etiqueta | Función |
|---|---|---|
| Q0.0 | KM2 | Contactor de 120VAC |
| Q0.1 | KM1 | Contactor de 220VAC |
| Q0.2 | K2 | SSR que suministra tensión a la herramienta con amplificador A4 (0-20A) |
| Q0.3 | K8 | Relevador para suministrar tensión a la herramienta con amplificador A2 (0-1A) |
| Q0.4 | K9 | Relevador para suministrar tensión a la herramienta con amplificador A3 (0-10A) |
| Q0.5 | K3 | L de FAN |
| Q0.6 | K4 | L de LIGHT |
| Q0.7 | K5 | L de FRAME ANTI SWEAT |
| Q1.0 | K6 | L de DOOR ANTI SWEAT |
| Q1.1 | K7 | L de DRAIN HEATERS |
| Q2.0 | K1 | L de HEATERS (DEFROST) |
| Q2.1 | Libre | Sin asignar |
| Q2.2 | Libre | Sin asignar |

### Entradas Digitales (I)
| Dirección | Etiqueta | Función |
|---|---|---|
| I0.0 | — | Paro de Emergencia |
| I0.1 | NA KM1 | Normalmente abierto de contactor 220VAC |
| I0.2 | NA KM2 | Normalmente abierto de contactor 110VAC |

### Entradas Analógicas (IA)
| Dirección | Función |
|---|---|
| IA1 | Amplificador de corriente de 0-1A (A2) |
| IA2 | Amplificador de corriente de 0-10A (A3) |
| IA3 | Amplificador de corriente de 0-20A (A4) |
| IA4 | Amplificador de tensión de 0-220VAC |

### Amplificadores de Corriente
| Amplificador | Rango | Interruptor |
|---|---|---|
| A2 | 0-1A | K8 (Q0.3) |
| A3 | 0-10A | K9 (Q0.4) |
| A4 | 0-20A | K2 (Q0.2) |

### Cargas y Salidas
| Salida | Carga |
|---|---|
| K3 (Q0.5) | FAN (Ventilador) |
| K6 (Q1.0) | DOOR (Calefacción de puerta / Door Anti-Sweat) |
| K5 (Q0.7) | FRAME (Calefacción de marco / Frame Anti-Sweat) |
| K7 (Q1.1) | DRAIN (Calefacción de desagüe / Drain Heaters) |
| K1 (Q2.0) | DEFROST (Descongelación / Heaters) |
| K4 (Q0.6) | LIGHT (Iluminación) |

### Advertencia
> Amplificador de tensión: corroborar con AMPLIFICADOR Y CONTACTOS NA de contactores que sea el voltaje correcto seleccionado ANTES DE ACCIONAR K8, K9 o K2.

### Mapeo Componente → Salida PLC
| Componente | Salida (Q) | Amplificador | Rango |
|---|---|---|---|
| FAN | Q0.5 (K3) | A2 (0-1A) | 0.3-1.5 A |
| DOOR | Q1.0 (K6) | A3 (0-10A) | 1.5-3.0 A |
| FRAME | Q0.7 (K5) | A3 (0-10A) | 0.5-1.2 A |
| DRAIN | Q1.1 (K7) | A3 (0-10A) | 1.5-2.8 A |
| DEFROST | Q2.0 (K1) | A4 (0-20A) | 0.4-1.0 A |
| LIGHT | Q0.6 (K4) | A2 (0-1A) | 0.1-0.6 A |

### Voltaje de alimentación
- 110VAC controlado por KM2 (Q0.0)
- 220VAC controlado por KM1 (Q0.1)