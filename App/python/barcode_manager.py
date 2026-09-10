"""Barcode USB HID: normalización a MAYÚSCULAS (doc 06 §4, regla crítica)."""

def normalize_code(raw: str) -> str:
    """Todo código escaneado -> .strip().upper()."""
    return (raw or "").strip().upper()


def classify_scan(step: int) -> str:
    return {1: "model", 2: "serial", 3: "batch"}.get(step, "model")


class BarcodeManager:
    """Backend helper: valida + normaliza. El lector es keyboard-wedge;
    el frontend captura el input y POSTea aquí."""

    @staticmethod
    def process(raw: str, barcode_type: str = "model", scan_order: int = 1) -> dict:
        normalized = normalize_code(raw)
        if not normalized:
            return {"ok": False, "error": "Código vacío"}
        return {"ok": True, "raw": raw, "code": normalized,
                "barcode_type": barcode_type, "scan_order": scan_order}
