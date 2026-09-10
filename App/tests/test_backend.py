"""Tests backend HiPotApp: barcode, ZPL, auth, config, hipot-sim."""
import os
os.environ["USE_SQLITE"] = "1"

from App.python.barcode_manager import normalize_code, BarcodeManager
from App.python.label_generator import canvas_to_zpl, DEFAULT_TEMPLATE_JSON, replace_placeholder
from App.python.auth import hash_password, verify_password
from App.python.hipot_manager import HipotManager


def test_normalize():
    assert normalize_code("  ref-1234 ") == "REF-1234"
    assert normalize_code("abc123") == "ABC123"


def test_barcode_process():
    r = BarcodeManager.process("ref-1", "model", 1)
    assert r["ok"] and r["code"] == "REF-1"
    assert not BarcodeManager.process("   ")["ok"]


def test_zpl_placeholders():
    z = canvas_to_zpl(DEFAULT_TEMPLATE_JSON, {"model": "REF-1", "serial_number": "S1",
                                              "final_status": "GOOD", "batch_number": "B1"})
    assert "^XA" in z and "^XZ" in z and "REF-1" in z
    assert replace_placeholder("{modelo}-{serie}", {"model": "A", "serial_number": "B"}) == "A-B"


def test_auth():
    h = hash_password("admin123")
    assert h != "admin123" and verify_password("admin123", h)
    assert not verify_password("wrong", h)


def test_hipot_sim():
    h = HipotManager(simulate=True)
    assert h.connect()
    assert h.send_command("STATUS?")
    resp = h.read_response(timeout=1)
    assert resp and "PASS" in resp
    assert HipotManager.parse_measurement(resp)["pass"] is True
    h.disconnect()


def test_config_export_import(tmp_path):
    from sqlalchemy import create_engine
    from sqlalchemy.orm import sessionmaker
    from App.python.models import Base
    from App.python.config_manager import ConfigManager
    url = f"sqlite:///{tmp_path}/t.db"
    eng = create_engine(url, connect_args={"check_same_thread": False})
    Base.metadata.create_all(eng)
    sf = sessionmaker(bind=eng)
    mgr = ConfigManager(sf)
    data = mgr.export_config(user_id="test")
    assert "Predeterminada" in data or "metadata" in data
    res = mgr.import_config(data, mode="add")
    assert res["status"] == "success"
