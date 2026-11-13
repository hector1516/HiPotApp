p = 'App/python/routes/api.py'
s = open(p, encoding='utf-8').read()
marker = '# ---------- config BD local ----------'
endpoints = '''# ---------- PLC IO (salidas/entradas) ----------
@router.get("/admin/plc-io-status")
def plc_io_status():
    plc = _plc()
    status = plc.get_io_status()
    from ..logger import logger
    logger.debug("PLC", "io_status", f"Q={status['outputs']} I={status['inputs']} IA={status['analog']}")
    return status


@router.post("/admin/plc-io-write")
def plc_io_write(body: dict):
    plc = _plc()
    q = body.get("q", "")
    val = bool(body.get("value", False))
    plc.write_digital_output(q, val)
    from ..logger import logger
    logger.info("PLC", "io_write", f"{q} = {val}", value_sent=f"{q}={val}")
    return {"status": "written", "q": q, "value": val}


'''
s = s.replace(marker, endpoints + marker, 1)
open(p, 'w', encoding='utf-8').write(s)
print("done")