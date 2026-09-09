# Auto-Update y Distribución — HiPotApp

## 1. Visión General

```
┌─────────────────────────────────────────────────────────────┐
│                    FLUJO DE ACTUALIZACIÓN                    │
│                                                             │
│  Desarrollador                                              │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Push a   │───▶│ GitHub       │───▶│ Build + Release  │  │
│  │ main     │    │ Actions      │    │ (.msi / .exe)    │  │
│  └──────────┘    └──────────────┘    └──────────────────┘  │
│                                           │                  │
│                                           ▼                  │
│                                    ┌──────────────┐         │
│                                    │ GitHub       │         │
│                                    │ Releases     │         │
│                                    └──────────────┘         │
│                                           │                  │
│                                           ▼                  │
│  Panel PC (estación)                                        │
│  ┌──────────────────────────────────────────────────┐      │
│  │  HiPotApp.exe                                     │      │
│  │  ┌──────────────────────────────────────────┐    │      │
│  │  │  1. App detecta nueva versión             │    │      │
│  │  │  2. Muestra notificación + changelog      │    │      │
│  │  │  3. Usuario acepta → descarga .msi       │    │      │
│  │  │  4. Instala → reinicia                   │    │      │
│  │  └──────────────────────────────────────────┘    │      │
│  └──────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## 2. Stack de Auto-Update

| Componente | Tecnología | Función |
|------------|-----------|---------|
| **Framework** | Tauri 2.x Updater Plugin | Sistema built-in de updates |
| **Distribución** | GitHub Releases | Almacenar .msi + signatures |
| **Build** | GitHub Actions | Compilar .msi automáticamente |
| **Installer** | NSIS (Windows) | Generar instalador .exe |
| **Updater** | `@tauri-apps/plugin-updater` | Detectar + descargar updates |
| **Changelog** | `@tauri-apps/plugin-process` | Mostrar cambios en la app |
| **Notificación** | Custom React component | Toast/banner en la UI |

## 3. Instalador: NSIS (.exe)

### Por qué NSIS en vez de MSI

```
MSI (Windows Installer):
  ✅ Estándar empresarial
  ✅ Group Policy compatible
  ❌ Requiere WiX Toolset
  ❌ Más complejo de configurar
  ❌ No incluye auto-update nativo

NSIS (Nullsoft Scriptable Install System):
  ✅ Más ligero
  ✅ Soporte nativo en Tauri
  ✅ Incluido en Tauri (no instalar aparte)
  ✅ Funciona con auto-update
  ✅ Desinstalador incluido
  ✅ Personalización fácil

DECISIÓN: NSIS (.exe) — Es el estándar de Tauri 2.x
```

### Configuración del instalador

```json
// tauri.conf.json
{
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "nsis": {
        "installMode": "both",
        "installerIcon": "icons/icon.ico",
        "languages": ["Spanish", "English"],
        "displayLanguageSelector": true,
        "headerImage": "icons/header.bmp",
        "sidebarImage": "icons/sidebar.bmp",
        "startMenuShortcut": true,
        "desktopShortcut": true
      }
    }
  }
}
```

### Resultado

```
HiPotApp_1.0.0_x64-setup.exe    ← Instalador (aprox. 15-25 MB)
HiPotApp_1.0.0_x64-setup.nsis   ← Script NSIS
HiPotApp_1.0.0_x64.msi          ← MSI alternativo (si se necesita)
```

## 4. GitHub Releases

### Estructura de una Release

```
https://github.com/hector1516/HiPotApp/releases

Tag: v1.0.0
│
├── HiPotApp_1.0.0_x64-setup.exe       ← Instalador
├── HiPotApp_1.0.0_x64-setup.exe.sig   ← Firma digital
├── HiPotApp_1.0.0_x64.msi             ← MSI (opcional)
├── latest.json                          ← Metadata para updater
└── RELEASE_NOTES.md                     ← Changelog
```

### Archivo `latest.json` (auto-update)

```json
{
  "version": "1.0.1",
  "notes": "Corrección de bug en cálculo de corriente",
  "pub_date": "2025-01-20T14:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dGVzdGlnbmF0dXJlX3ZhbHVl",
      "url": "https://github.com/hector1516/HiPotApp/releases/download/v1.0.1/HiPotApp_1.0.1_x64-setup.exe"
    }
  }
}
```

## 5. Configuración del Updater en Tauri

### `tauri.conf.json`

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/hector1516/HiPotApp/releases/latest/download/latest.json"
      ],
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQgZm9yIGdlbmVyYXRlZSBLZXlint0=",
      "windows": {
        "installMode": "both"
      }
    }
  }
}
```

### Generar par de claves para updater

```bash
# Instalar tauri-cli (si no está)
npm install -g @tauri-apps/cli

# Generar par de claves
npx tauri signer generate -w ~/.tauri/hopot.key
```

**Output:**
```
Key created successfully
Public key: dW50cnVzdGVkIGNvbW1lbnQgZm9yIGdlbmVyYXRlZSBLZXlint0=
Private key saved to: ~/.tauri/hopot.key

IMPORTANTE: La clave privada NUNCA se sube a GitHub.
Solo la pública va en tauri.conf.json.
```

## 6. GitHub Actions — Build Automático

### `.github/workflows/build.yml`

```yaml
name: Build & Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: windows-latest
    
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Install dependencies (Windows)
        run: |
          choco install wixtoolset -y
          choco install nsis -y

      - name: Install frontend dependencies
        run: npm install
        working-directory: tauri

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ""
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "HiPotApp ${{ github.ref_name }}"
          releaseBody: |
            ## Cambios
            
            Ver RELEASE_NOTES.md para detalles completos.
          releaseDraft: false
          prerelease: false
          args: --target x86_64-pc-windows-msvc
```

### Variables Secrets necesarias en GitHub

```
GITHUB_TOKEN               → Se genera automáticamente
TAURI_SIGNING_PRIVATE_KEY  → La clave privada generada con tauri signer
```

### Flujo completo

```
1. Desarrollador hace tag:  git tag v1.0.1 && git push --tags
2. GitHub Actions detecta el tag
3. Actions compila la app (frontend + Rust)
4. Genera .exe (NSIS installer) + .msi
5. Genera firma digital (.sig)
6. Crea release en GitHub con los archivos
7. Updater en la app detecta la nueva versión
8. Muestra notificación al usuario
```

## 7. Updater en el Frontend (React)

### Instalar plugins

```bash
cd tauri
npm install @tauri-apps/plugin-updater
npm install @tauri-apps/plugin-process
npm install @tauri-apps/plugin-notification
```

### Hook personalizado `useUpdater.ts`

```typescript
import { useEffect, useState, useCallback } from 'react';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

interface UpdateInfo {
  version: string;
  notes: string;
  date: string;
}

interface UpdaterState {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  progress: number;
  updateInfo: UpdateInfo | null;
  error: string | null;
}

export function useUpdater() {
  const [state, setState] = useState<UpdaterState>({
    checking: false,
    available: false,
    downloading: false,
    progress: 0,
    updateInfo: null,
    error: null,
  });

  const checkForUpdates = useCallback(async () => {
    setState(prev => ({ ...prev, checking: true, error: null }));
    
    try {
      const update = await check();
      
      if (update) {
        setState(prev => ({
          ...prev,
          checking: false,
          available: true,
          updateInfo: {
            version: update.version,
            notes: update.body || '',
            date: update.date || new Date().toISOString(),
          },
        }));
      } else {
        setState(prev => ({
          ...prev,
          checking: false,
          available: false,
        }));
      }
    } catch (error) {
      setState(prev => ({
        ...prev,
        checking: false,
        error: error as string,
      }));
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    setState(prev => ({ ...prev, downloading: true, progress: 0 }));
    
    try {
      const update = await check();
      
      if (update) {
        let downloaded = 0;
        let contentLength = 0;
        
        await update.downloadAndInstall((event) => {
          switch (event.event) {
            case 'Started':
              contentLength = event.data.contentLength || 0;
              break;
            case 'Progress':
              downloaded += event.data.chunkLength;
              setState(prev => ({
                ...prev,
                progress: contentLength > 0 
                  ? Math.round((downloaded / contentLength) * 100)
                  : 0,
              }));
              break;
            case 'Finished':
              setState(prev => ({
                ...prev,
                downloading: false,
                progress: 100,
              }));
              break;
          }
        });
        
        // Reiniciar la app
        await relaunch();
      }
    } catch (error) {
      setState(prev => ({
        ...prev,
        downloading: false,
        error: error as string,
      }));
    }
  }, []);

  // Auto-check cada 4 horas
  useEffect(() => {
    checkForUpdates();
    const interval = setInterval(checkForUpdates, 4 * 60 * 60 * 1000);
    return () => clearInterval(interval);
  }, [checkForUpdates]);

  return {
    ...state,
    checkForUpdates,
    downloadAndInstall,
  };
}
```

### Componente `UpdateNotification.tsx`

```tsx
import React, { useState } from 'react';
import { useUpdater } from '../hooks/useUpdater';
import { motion, AnimatePresence } from 'framer-motion';

export function UpdateNotification() {
  const {
    checking,
    available,
    downloading,
    progress,
    updateInfo,
    error,
    checkForUpdates,
    downloadAndInstall,
  } = useUpdater();

  const [dismissed, setDismissed] = useState(false);

  if (dismissed || !available) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ y: -100, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        exit={{ y: -100, opacity: 0 }}
        className="update-notification"
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          background: 'linear-gradient(135deg, #1a237e, #283593)',
          color: 'white',
          padding: '16px 24px',
          zIndex: 9999,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
        }}
      >
        <div style={{ flex: 1 }}>
          <div style={{ 
            fontSize: '16px', 
            fontWeight: 'bold',
            marginBottom: '4px' 
          }}>
            Nueva versión disponible: v{updateInfo?.version}
          </div>
          
          {updateInfo?.notes && (
            <div style={{ 
              fontSize: '13px', 
              opacity: 0.9,
              maxHeight: '80px',
              overflow: 'auto',
              marginBottom: '8px',
              whiteSpace: 'pre-wrap',
            }}>
              {updateInfo.notes}
            </div>
          )}
          
          {downloading && (
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              gap: '12px',
              marginTop: '8px' 
            }}>
              <div style={{
                flex: 1,
                height: '6px',
                background: 'rgba(255,255,255,0.3)',
                borderRadius: '3px',
                overflow: 'hidden',
              }}>
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${progress}%` }}
                  style={{
                    height: '100%',
                    background: '#4caf50',
                    borderRadius: '3px',
                  }}
                />
              </div>
              <span style={{ fontSize: '13px' }}>{progress}%</span>
            </div>
          )}
        </div>

        <div style={{ display: 'flex', gap: '8px', marginLeft: '16px' }}>
          {!downloading && (
            <>
              <button
                onClick={downloadAndInstall}
                style={{
                  background: '#4caf50',
                  color: 'white',
                  border: 'none',
                  padding: '10px 20px',
                  borderRadius: '6px',
                  fontSize: '14px',
                  fontWeight: 'bold',
                  cursor: 'pointer',
                  minWidth: '120px',
                }}
              >
                Actualizar
              </button>
              <button
                onClick={() => setDismissed(true)}
                style={{
                  background: 'transparent',
                  color: 'white',
                  border: '1px solid rgba(255,255,255,0.3)',
                  padding: '10px 16px',
                  borderRadius: '6px',
                  fontSize: '14px',
                  cursor: 'pointer',
                }}
              >
                Después
              </button>
            </>
          )}
          
          {downloading && (
            <button
              disabled
              style={{
                background: 'rgba(255,255,255,0.2)',
                color: 'white',
                border: 'none',
                padding: '10px 20px',
                borderRadius: '6px',
                fontSize: '14px',
                cursor: 'not-allowed',
              }}
            >
              Instalando...
            </button>
          )}
        </div>

        {error && (
          <div style={{
            position: 'absolute',
            bottom: 0,
            left: 0,
            right: 0,
            background: '#f44336',
            padding: '8px 16px',
            fontSize: '13px',
          }}>
            Error: {error}
            <button
              onClick={checkForUpdates}
              style={{
                marginLeft: '12px',
                background: 'white',
                color: '#f44336',
                border: 'none',
                padding: '4px 12px',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Reintentar
            </button>
          </div>
        )}
      </motion.div>
    </AnimatePresence>
  );
}
```

### Componente `UpdateChangelog.tsx`

```tsx
import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';

interface ChangelogProps {
  version: string;
  notes: string;
  date: string;
  onClose: () => void;
}

export function UpdateChangelog({ version, notes, date, onClose }: ChangelogProps) {
  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(0,0,0,0.7)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 10000,
        }}
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          onClick={(e) => e.stopPropagation()}
          style={{
            background: 'white',
            borderRadius: '16px',
            padding: '32px',
            maxWidth: '500px',
            width: '90%',
            maxHeight: '80vh',
            overflow: 'auto',
          }}
        >
          <div style={{ textAlign: 'center', marginBottom: '24px' }}>
            <div style={{ 
              fontSize: '48px', 
              marginBottom: '8px' 
            }}>
              🎉
            </div>
            <h2 style={{ 
              margin: 0, 
              color: '#1a237e',
              fontSize: '24px' 
            }}>
              HiPotApp v{version}
            </h2>
            <div style={{ 
              color: '#666', 
              fontSize: '14px',
              marginTop: '4px' 
            }}>
              {new Date(date).toLocaleDateString('es-MX', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
              })}
            </div>
          </div>

          <div style={{ 
            background: '#f5f5f5', 
            borderRadius: '8px',
            padding: '16px',
            marginBottom: '24px',
          }}>
            <h3 style={{ 
              margin: '0 0 12px 0', 
              fontSize: '16px',
              color: '#333' 
            }}>
              Cambios en esta versión
            </h3>
            <div style={{ 
              fontSize: '14px',
              lineHeight: '1.6',
              color: '#555',
              whiteSpace: 'pre-wrap',
            }}>
              {notes || 'Sin notas de cambios disponibles.'}
            </div>
          </div>

          <div style={{ textAlign: 'center' }}>
            <button
              onClick={onClose}
              style={{
                background: '#1a237e',
                color: 'white',
                border: 'none',
                padding: '12px 32px',
                borderRadius: '8px',
                fontSize: '16px',
                fontWeight: 'bold',
                cursor: 'pointer',
              }}
            >
              Entendido
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
```

### Uso en `App.tsx`

```tsx
import { UpdateNotification } from './components/UpdateNotification';
import { useUpdater } from './hooks/useUpdater';

function App() {
  const { available, updateInfo } = useUpdater();

  return (
    <div className="app">
      {/* Notificación de update (banner superior) */}
      <UpdateNotification />

      {/* Pantalla principal */}
      <MainScreen />

      {/* Changelog modal (después de actualizar) */}
      {showChangelog && updateInfo && (
        <UpdateChangelog
          version={updateInfo.version}
          notes={updateInfo.notes}
          date={updateInfo.date}
          onClose={() => setShowChangelog(false)}
        />
      )}
    </div>
  );
}
```

## 8. Changelog en Cada Release

### Plantilla `RELEASE_NOTES.md`

```markdown
# HiPotApp v1.1.0

**Fecha:** 20 Enero 2025

## ✨ Nuevas Funcionalidades

- Editor visual de etiquetas con drag & drop
- Reimpresión desde el panel de admin
- Soporte para múltiples plantillas de etiqueta

## 🔧 Mejoras

- Velocidad de conexión PLC mejorada en 40%
- Interfaz más responsiva en pantallas pequeñas
- Mensajes de error más descriptivos

## 🐛 Correcciones

- Corregido bug donde el batch no se guardaba correctamente
- Corregido cálculo de corriente total con voltaje 208VAC
- Corregido problema de conexión con Hipot al iniciar la app

## 📦 Actualizaciones

- Tauri 2.0.6 → 2.1.0
- React 18.2.0 → 18.3.0
- snap7 1.4.0 → 1.5.0

## ⚠️ Notas Importantes

- Requiere reinstalación del driver de impresora Zebra
- Los datos de la BD se migran automáticamente
```

### Integración con GitHub Releases

```yaml
# En GitHub Actions
- name: Create Release
  uses: softprops/action-gh-release@v1
  with:
    body_path: RELEASE_NOTES.md
    files: |
      src-tauri/target/release/bundle/nsis/*.exe
      src-tauri/target/release/bundle/nsis/*.sig
```

## 9. Flujo Completo del Usuario

```
PANEL PC (Estación de Producción)
═══════════════════════════════════════

1. App inicia normalmente
   │
   ▼
2. Background check cada 4 horas
   │
   ▼
3. Si hay nueva versión:
   ┌─────────────────────────────────────────┐
   │  🔔 Nueva versión disponible: v1.1.0   │
   │                                         │
   │  Cambios:                               │
   │  - Editor de etiquetas                  │
   │  - Corrección de bugs                   │
   │                                         │
   │  [Actualizar]  [Después]                │
   └─────────────────────────────────────────┘
   │
   ▼ (si usuario hace clic en "Actualizar")
4. Descarga en background
   ┌─────────────────────────────────────────┐
   │  Descargando... ████████░░░░ 65%        │
   └─────────────────────────────────────────┘
   │
   ▼
5. Descarga completa → Notificación
   ┌─────────────────────────────────────────┐
   │  ✅ Descarga completa                   │
   │  La app se reiniciará para aplicar      │
   │  los cambios.                           │
   │                                         │
   │  [Reiniciar Ahora]  [Reiniciar Después] │
   └─────────────────────────────────────────┘
   │
   ▼ (si usuario hace clic en "Reiniciar Ahora")
6. App se cierra → Instalador NSIS ejecuta
   │
   ▼
7. App se abre con nueva versión
   ┌─────────────────────────────────────────┐
   │  🎉 HiPotApp v1.1.0                    │
   │                                         │
   │  Cambios en esta versión:              │
   │  ✨ Editor de etiquetas                │
   │  🐛 Corrección de bugs                 │
   │                                         │
   │  [Entendido]                            │
   └─────────────────────────────────────────┘
   │
   ▼
8. App lista para usar con nueva versión
```

## 10. Seguridad

### Firma digital de actualizaciones

```
1. Cada build genera un .sig (firma digital)
2. El updater de Tauri verifica la firma ANTES de instalar
3. Si la firma no coincide → NO instala → Muestra error
4. La clave privada NUNCA se sube a GitHub
5. Solo la pública va en tauri.conf.json
```

### Protección contra ataques

```
MITM (Man-in-the-Middle):
  ✅ HTTPS obligatorio para endpoints
  ✅ Firma digital en cada release
  ✅ Verificación de hash

Tampering (alteración):
  ✅ Firma digital impide modificación
  ✅ Hash verification en descarga

Rollback (versión anterior):
  ✅ Tauri no permite instalar versión anterior
  ✅ Versión se incrementa siempre
```

## 11. Script de Build Local

### `build.bat` (Windows)

```batch
@echo off
echo ========================================
echo   HiPotApp - Build Installer
echo ========================================
echo.

echo [1/4] Instalando dependencias del frontend...
cd tauri
call npm install
if errorlevel 1 goto error

echo [2/4] Compilando la app...
call npx tauri build
if errorlevel 1 goto error

echo [3/4] Generando installer NSIS...
echo Installer generado en: tauri/target/release/bundle/nsis/

echo [4/4] Listo!
echo.
echo Archivos generados:
dir /b tauri\target\release\bundle\nsis\*.exe
echo.
pause
goto end

:error
echo.
echo ERROR: Build fallido
pause
exit /b 1

:end
```

## 12. Resumen

| Componente | Estado |
|------------|--------|
| Instalador NSIS (.exe) | ✅ Configurado |
| GitHub Actions (build auto) | ✅ Configurado |
| Auto-update (Tauri updater) | ✅ Configurado |
| Firma digital (.sig) | ✅ Seguridad |
| Notificación en-app | ✅ Componente React |
| Changelog por versión | ✅ RELEASE_NOTES.md |
| Download con progreso | ✅ Barra animada |
| Reinicio automático | ✅ relaunch() |
| Check cada 4 horas | ✅ Background |
| Rolling update (sin downtime) | ✅ Tag-based |

### Flujo de releases

```
git tag v1.0.1
git push --tags
    │
    ▼
GitHub Actions compila
    │
    ▼
Release creada con .exe + .sig
    │
    ▼
Panel PC detecta → Notifica → Instala → Reinicia
```
