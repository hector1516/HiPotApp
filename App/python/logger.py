"""Logger a app_logs + consola (doc 08)."""
import time
from datetime import datetime


class AppLogger:
    def __init__(self, session_factory=None):
        self.session_factory = session_factory

    def _persist(self, entry: dict):
        if not self.session_factory:
            return
        try:
            from .models import AppLog
            s = self.session_factory()
            try:
                s.add(AppLog(**entry))
                s.commit()
            finally:
                s.close()
        except Exception as e:
            print(f"[LOGGER-ERROR] no se pudo persistir log: {e}")

    def log(self, level, category, action, detail="", execution_id=None,
            user_id=None, value_sent=None, value_received=None,
            duration_ms=None, success=True, error_message=None):
        entry = dict(
            timestamp=datetime.now(), level=level, category=category,
            action=action, detail=detail, execution_id=execution_id,
            user_id=user_id, value_sent=value_sent,
            value_received=value_received, duration_ms=duration_ms,
            success=1 if success else 0, error_message=error_message,
        )
        print(f"[{level}] [{category}] {action}: {detail}")
        self._persist(entry)

    def debug(self, cat, action, detail="", **kw):
        self.log("DEBUG", cat, action, detail, **kw)

    def info(self, cat, action, detail="", **kw):
        self.log("INFO", cat, action, detail, **kw)

    def warn(self, cat, action, detail="", **kw):
        self.log("WARN", cat, action, detail, **kw)

    def error(self, cat, action, detail="", error_message=None, **kw):
        self.log("ERROR", cat, action, detail, success=False,
                 error_message=error_message, **kw)


logger = AppLogger()


def timed(logger_: AppLogger, category: str, action: str):
    """Context manager que mide duración y loguea."""
    class _T:
        def __enter__(self):
            self.t0 = time.time()
            return self

        def __exit__(self, exc_type, exc, tb):
            ms = int((time.time() - self.t0) * 1000)
            if exc_type:
                logger_.error(category, action, f"{action} falló",
                              error_message=str(exc), duration_ms=ms)
            else:
                logger_.debug(category, action, f"{action} OK", duration_ms=ms)
            return False
    return _T()
