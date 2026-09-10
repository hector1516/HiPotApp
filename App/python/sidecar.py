"""Entry point del backend empaquetado (PyInstaller onefile).

Uso dev:  python -m App.python.sidecar
Binario:  hipot-backend-x86_64-pc-windows-msvc.exe (lo arranca el .exe Tauri)
"""
import uvicorn

from .main import app

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8000, workers=1, reload=False)
