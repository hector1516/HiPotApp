"""HiPotApp backend — FastAPI + DB dual + PLC + Hipot (docs 01-10)."""
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .database import local_db
from .logger import AppLogger, logger
from .routes.api import router


@asynccontextmanager
async def lifespan(app: FastAPI):
    local_db.create_tables()
    s = local_db.Session()
    try:
        from .models import User, LabelTemplate
        from .auth import hash_password
        from .label_generator import DEFAULT_TEMPLATE_JSON
        # Sin passwords: solo gafete
        if not s.query(User).filter_by(username="ADMIN").first():
            s.add(User(username="ADMIN", password_hash="", role="admin"))
        # Corrige viejo admin en minúsculas si existe
        old = s.query(User).filter_by(username="admin").first()
        if old:
            old.username = "ADMIN"
            old.password_hash = ""
        if not s.query(LabelTemplate).first():
            s.add(LabelTemplate(name="Predeterminada 100x60", width_mm=100,
                                height_mm=60, canvas_data=DEFAULT_TEMPLATE_JSON,
                                is_default=True))
        s.commit()
    except Exception as e:
        print(f"[startup-seed] {e}")
        s.rollback()
    finally:
        s.close()

    # Auto-conectar PLC en el arranque con IP/datos guardados en BD
    try:
        from .routes.api import _plc
        p = _plc()
        print(f"[plc-startup] Conectado al PLC ({p.ip}): {p.connected}")
    except Exception as e:
        print(f"[plc-startup-err] {e}")

    logger.info("SYSTEM", "startup", "HiPotApp backend iniciado")
    yield


app = FastAPI(title="HiPotApp API",
              description="PLC S7-1200 + Hipot 3865 + Zebra ZT231",
              version="1.0.0", lifespan=lifespan)
app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_credentials=True,
                   allow_methods=["*"], allow_headers=["*"])

# logger con persistencia
try:
    logger.session_factory = local_db.Session
except Exception:
    pass

app.include_router(router)


@app.get("/")
def root():
    return {"message": "HiPotApp API running", "version": "1.0.0"}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run("App.python.main:app", host="0.0.0.0", port=8000, reload=True)
