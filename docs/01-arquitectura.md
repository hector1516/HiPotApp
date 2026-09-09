# Arquitectura — HiPotApp

## 1. Stack Tecnológico

| Capa | Tecnología | Versión | Justificación |
|------|-----------|---------|---------------|
| **Frontend** | Tauri + React | 2.x + 18+ | App desktop ligera, rendimiento nativo |
| **UI Library** | Tailwind CSS + Framer Motion | 3.x + 11.x | Estilos rápidos, animaciones fluidas |
| **Backend** | FastAPI + Python | 3.14+ | API moderna, async, auto-docs |
| **PLC** | snap7 | latest | Driver Siemens S7-1200/1500 más maduro |
| **Hypot** | pyserial | latest | Comunicación RS-232 USB |
| **Database Local** | SQL Express | 2019+ | Gratuito, suficiente para local |
| **Database Remoto** | SQL Server | 2019+ | Central de reportes |
| **ORM** | SQLAlchemy | 2.x | Mapeo flexible, soporte SQL Server |
| **Migraciones** | Alembic | latest | Control de versión del esquema |

## 2. Diagrama de Arquitectura

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        TAURI APP (React Frontend)                       │
│                                                                         │
│  ┌─────────────┐  IPC   ┌───────────────────────────────────────────┐  │
│  │  UI Touch   │◄──────►│        Python FastAPI Backend             │  │
│  │  10.1" 1280x800│      │                                           │  │
│  │  Framer Motion│      │  ┌─────────────┐  ┌────────────────────┐  │  │
│  └─────────────┘        │  │ PLC Module  │  │ Hypot Module       │  │  │
│         │               │  │ (snap7)     │  │ (pyserial)         │  │  │
│         │               │  └──────┬──────┘  └─────────┬──────────┘  │  │
│         │               │         │                    │             │  │
│         │               │  ┌──────▼──────────────────▼──────────┐   │  │
│         │               │  │        Database Module             │   │  │
│         │               │  │   (SQLAlchemy + Alembic)           │   │  │
│         │               │  └───────────────┬────────────────────┘   │  │
│         │               └──────────────────┼────────────────────────┘  │
│         │                                  │                           │
└─────────┼──────────────────────────────────┼───────────────────────────┘
          │                                  │
          │                                  │
    ┌─────▼─────┐                    ┌───────▼───────┐
    │ PLC       │                    │ SQL Express   │
    │ S7-1200   │                    │ (Local)       │
    │ Ethernet  │                    └───────┬───────┘
    └───────────┘                            │ Sync
                                             │
                                     ┌───────▼───────┐
                                     │ SQL Server    │
                                     │ (Remoto)      │
                                     └───────────────┘

    ┌───────────┐                    ┌───────────┐
    │ Hypot     │                    │ Barcode   │
    │ 3865      │                    │ Scanner   │
    │ RS-232    │                    │ (USB HID) │
    └───────────┘                    └───────────┘
```

## 3. Flujo de Datos

### **Prueba Normal (Online)**

```
1. Escaneo MODELO → DB busca receta → UI muestra receta
2. Escaneo SERIE → DB valida → UI confirma
3. Escaneo BATCH → DB crea execution → UI muestra "LISTO"
4. START → Python envía receta PLC → PLC ejecuta
5. PLC mide corrientes → Python lee tags → UI muestra valores
6. PLC termina → Python lee pass/fail → DB guarda resultado
7. Python envía receta Hypot → Hypot ejecuta
8. Hypot mide → Python lee serial → UI muestra valores
9. Hypot termina → Python lee resultado → DB guarda
10. Cálculo final → BUENO/MALO → DB guarda execution completa
```

### **Prueba Offline (sin red)**

```
1. App guarda todo en SQL Express (local)
2. sync_to_remote = 0 en cada registro
3. Cuando red vuelve → sync automático cada N minutos
4. Log de sync muestra estado
```

## 4. Seguridad

| Aspecto | Implementación |
|---------|---------------|
| **Token GitHub** | Variable de entorno, no en código |
| **Passwords DB** | Encriptados con bcrypt |
| **Conexión SQL** | Named pipes o encriptación SSL |
| **Roles** | operator (prueba) / admin (config) |
| **Auditoría** | Log de cada escaneo y ejecución |

## 5. Auto-Update y Distribución

| Componente | Tecnología | Función |
|------------|-----------|---------|
| **Instalador** | NSIS (.exe) | Instalador ligero, incluido en Tauri |
| **Build auto** | GitHub Actions | Compilar en cada tag `v*` |
| **Distribución** | GitHub Releases | Almacenar .exe + firmas |
| **Updater** | Tauri Updater Plugin | Detectar + descargar + instalar |
| **Changelog** | RELEASE_NOTES.md | Notas por versión |
| **Seguridad** | Firma digital (.sig) | Verificar integridad |

```
Desarrollador → git tag v1.0.1 → GitHub Actions → Release → Panel PC detecta → Actualiza
```

Documentación completa: `docs/10-auto-update.md`

## 6. Requisitos del Sistema

### **PC Local (Panel 10.1")**
- Windows 11 Pro
- Touch 10.1" 1280x800 multitouch
- SQL Express 2019+
- Python 3.14+
- Node.js 18+ (para Tauri build)
- USB para Hypot (RS-232)
- Ethernet para PLC (S7-1200)
- Lector de código de barras (USB HID)

### **Servidor Remoto**
- Windows Server 2019+ o Windows 10/11 Pro
- SQL Server 2019+
- Acceso de red desde la PC local
- Puerto TCP 1433 abierto
