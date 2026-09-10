"""Launcher del backend para PyInstaller onefile (ver App/python/sidecar.py)."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import uvicorn
from App.python.main import app

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8000, workers=1, reload=False)
