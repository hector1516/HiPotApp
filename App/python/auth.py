"""Auth bcrypt + JWT simple (doc 01/07). Nunca texto plano."""
import bcrypt
from datetime import datetime, timedelta
from jose import jwt

import os

SECRET = os.getenv("JWT_SECRET", "hipot-dev-secret-change-me")
if SECRET == "hipot-dev-secret-change-me":
    import warnings
    warnings.warn(
        "JWT_SECRET no está definido. Se usa una clave por defecto insegura. "
        "Define JWT_SECRET en producción.",
        stacklevel=2,
    )
ALGO = "HS256"


def hash_password(plain: str) -> str:
    return bcrypt.hashpw(plain.encode(), bcrypt.gensalt()).decode()


def verify_password(plain: str, hashed: str) -> bool:
    try:
        return bcrypt.checkpw(plain.encode(), hashed.encode())
    except Exception:
        return False


def create_token(username: str, role: str, minutes: int = 30 * 24 * 60) -> str:
    payload = {"sub": username, "role": role,
               "exp": datetime.utcnow() + timedelta(minutes=minutes)}
    return jwt.encode(payload, SECRET, algorithm=ALGO)


def decode_token(token: str) -> dict | None:
    try:
        return jwt.decode(token, SECRET, algorithms=[ALGO])
    except Exception:
        return None
