"""Launcher del backend para PyInstaller onefile (ver App/python/sidecar.py)."""
import os
import sys
import subprocess

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Liberar puerto 8765 si estuviera ocupado
try:
    if sys.platform == "win32":
        out = subprocess.check_output(
            ["netstat", "-ano"], text=True, stderr=subprocess.DEVNULL
        )
        for line in out.splitlines():
            if ":8765" in line and "LISTENING" in line:
                parts = line.split()
                pid = parts[-1]
                if pid.isdigit() and int(pid) > 0:
                    subprocess.run(["taskkill", "/F", "/PID", pid],
                                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
except Exception:
    pass

import uvicorn
from App.python.main import app

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8765, workers=1, reload=False)
