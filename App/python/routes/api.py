"""Rutas: recetas, operadores, ejecuciones, barcode, hardware, admin."""
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import Optional
from datetime import datetime
import json

router = APIRouter()


# ---------- schemas ----------
class ScanIn(BaseModel):
    code: str
    barcode_type: str = "model"
    scan_order: int = 1
    execution_id: Optional[int] = None


class RecipeIn(BaseModel):
    name: str
    model: str
    type: str = "current_consumption"
    product: Optional[str] = None
    voltage_levels: Optional[str] = None
    components: list = []
    totals: list = []
    modified_by: Optional[str] = None


class OperatorIn(BaseModel):
    name: str
    employee_id: str


class UserIn(BaseModel):
    username: str
    password: Optional[str] = ""
    role: str = "operator"


class LoginIn(BaseModel):
    username: str
    password: str


def _sf():
    from ..database import local_db
    return local_db.Session


# ---------- barcode ----------
@router.post("/barcode/scan")
def scan_barcode(body: ScanIn):
    from ..barcode_manager import BarcodeManager
    from ..models import BarcodeLog
    from ..logger import logger
    res = BarcodeManager.process(body.code, body.barcode_type, body.scan_order)
    if not res["ok"]:
        raise HTTPException(400, res.get("error", "scan inválido"))
    s = _sf()()
    try:
        s.add(BarcodeLog(code=res["code"], barcode_type=body.barcode_type,
                         scan_order=body.scan_order, execution_id=body.execution_id))
        s.commit()
    finally:
        s.close()
    logger.info("BARCODE", f"scan_{body.barcode_type}",
                f"Crudo: '{body.code}' -> Normalizado: '{res['code']}'")
    return {"status": "scanned", **res}


# ---------- recipes ----------
@router.get("/recipes")
def list_recipes(q: str = ""):
    from ..models import Recipe
    s = _sf()()
    try:
        query = s.query(Recipe)
        if q:
            query = query.filter(Recipe.model.contains(q.upper()))
        return [{"id": r.id, "name": r.name, "model": r.model,
                 "type": r.type, "product": r.product,
                 "voltage_levels": r.voltage_levels, "is_active": r.is_active,
                 "modified_by": r.modified_by, "updated_at": str(r.updated_at) if r.updated_at else None,
                 "created_at": str(r.created_at) if r.created_at else None}
                for r in query.all()]
    finally:
        s.close()


@router.get("/recipes/model/{model}")
def get_recipe(model: str):
    from ..models import Recipe, RecipeComponent, RecipeTotal
    s = _sf()()
    try:
        r = s.query(Recipe).filter_by(model=model.strip().upper()).first()
        if not r:
            raise HTTPException(404, f"Receta no encontrada: {model}")
        comps = s.query(RecipeComponent).filter_by(recipe_id=r.id).all()
        tots = s.query(RecipeTotal).filter_by(recipe_id=r.id).all()
        return {"id": r.id, "name": r.name, "model": r.model,
                "voltage_levels": json.loads(r.voltage_levels) if r.voltage_levels else [],
                "components": [{"name": c.name, "voltage": c.voltage, "enabled": c.enabled,
                                "min": c.min_value, "max": c.max_value, "unit": c.unit} for c in comps],
                "totals": [{"voltage": t.voltage, "min": t.min_total, "max": t.max_total} for t in tots],
                "modified_by": r.modified_by, "updated_at": str(r.updated_at) if r.updated_at else None,
                "created_at": str(r.created_at) if r.created_at else None}
    finally:
        s.close()


@router.post("/recipes")
def create_recipe(body: RecipeIn):
    from ..models import Recipe, RecipeComponent, RecipeTotal
    from datetime import datetime
    s = _sf()()
    try:
        if s.query(Recipe).filter_by(model=body.model.strip().upper()).first():
            raise HTTPException(400, "Modelo ya existe")
        r = Recipe(name=body.name, model=body.model.strip().upper(),
                   type=body.type, product=body.product,
                   voltage_levels=body.voltage_levels or '["120VAC"]',
                   modified_by=(body.modified_by or "").strip().upper() or None,
                   updated_at=datetime.now())
        s.add(r); s.flush()
        for c in body.components:
            # voltage puede venir como entero 120 -> guardar como "120"
            v = str(c.get("voltage", "120")).strip()
            s.add(RecipeComponent(recipe_id=r.id, name=c.get("name", ""),
                                  voltage=v, enabled=c.get("enabled", True),
                                  min_value=c.get("min", 0), max_value=c.get("max", 0)))
        for t in body.totals:
            v = str(t.get("voltage", "")).strip()
            s.add(RecipeTotal(recipe_id=r.id, voltage=v, min_total=t.get("min", 0), max_total=t.get("max", 0)))
        s.commit()
        return {"status": "created", "recipe_id": r.id}
    except HTTPException:
        raise
    except Exception as e:
        s.rollback()
        raise HTTPException(500, str(e))
    finally:
        s.close()


@router.put("/recipes/{rid}")
def update_recipe(rid: int, body: RecipeIn):
    from ..models import Recipe, RecipeComponent, RecipeTotal
    s = _sf()()
    try:
        r = s.query(Recipe).get(rid)
        if not r:
            raise HTTPException(404, "Receta no encontrada")
        other = s.query(Recipe).filter(Recipe.model == body.model.strip().upper(), Recipe.id != rid).first()
        if other:
            raise HTTPException(400, "Modelo ya existe en otra receta")
        from datetime import datetime
        r.name = body.name
        r.model = body.model.strip().upper()
        r.type = body.type
        r.product = body.product
        r.voltage_levels = body.voltage_levels or r.voltage_levels
        r.modified_by = (body.modified_by or "").strip().upper() or r.modified_by
        r.updated_at = datetime.now()
        s.query(RecipeComponent).filter_by(recipe_id=rid).delete()
        s.query(RecipeTotal).filter_by(recipe_id=rid).delete()
        s.flush()
        for c in body.components:
            v = str(c.get("voltage","120")).strip()
            s.add(RecipeComponent(recipe_id=rid, name=c.get("name",""), voltage=v,
                                  enabled=c.get("enabled", True), min_value=c.get("min",0), max_value=c.get("max",0)))
        for t in body.totals:
            v = str(t.get("voltage","")).strip()
            s.add(RecipeTotal(recipe_id=rid, voltage=v, min_total=t.get("min",0), max_total=t.get("max",0)))
        s.commit()
        return {"status": "updated"}
    except HTTPException:
        raise
    except Exception as e:
        s.rollback()
        raise HTTPException(500, str(e))
    finally:
        s.close()


@router.delete("/recipes/{rid}")
def delete_recipe(rid: int):
    from ..models import Recipe, RecipeComponent, RecipeTotal
    s = _sf()()
    try:
        r = s.query(Recipe).get(rid)
        if not r:
            raise HTTPException(404, "Receta no encontrada")
        s.query(RecipeComponent).filter_by(recipe_id=rid).delete()
        s.query(RecipeTotal).filter_by(recipe_id=rid).delete()
        s.delete(r); s.commit()
        return {"status": "deleted"}
    finally:
        s.close()


@router.post("/recipes/{rid}/duplicate")
def duplicate_recipe(rid: int, body: dict = {}):
    from ..models import Recipe, RecipeComponent, RecipeTotal
    s = _sf()()
    try:
        r = s.query(Recipe).get(rid)
        if not r:
            raise HTTPException(404, "Receta no encontrada")
        suffix = (body.get("suffix") or "COPIA").strip().upper()
        new_model = (body.get("model") or f"{r.model}-{suffix}").strip().upper()
        if s.query(Recipe).filter_by(model=new_model).first():
            raise HTTPException(400, "Modelo destino ya existe")
        nr = Recipe(name=f"{r.name} ({suffix})", model=new_model, type=r.type, product=r.product,
                    voltage_levels=r.voltage_levels)
        s.add(nr); s.flush()
        for c in s.query(RecipeComponent).filter_by(recipe_id=rid).all():
            s.add(RecipeComponent(recipe_id=nr.id, name=c.name, voltage=c.voltage, enabled=c.enabled,
                                  min_value=c.min_value, max_value=c.max_value, unit=c.unit))
        for t in s.query(RecipeTotal).filter_by(recipe_id=rid).all():
            s.add(RecipeTotal(recipe_id=nr.id, voltage=t.voltage, min_total=t.min_total, max_total=t.max_total))
        s.commit()
        return {"status": "duplicated", "recipe_id": nr.id, "model": new_model}
    finally:
        s.close()


# ---------- operators / users ----------
@router.get("/operators")
def list_operators():
    from ..models import Operator
    s = _sf()()
    try:
        return [{"id": o.id, "name": o.name, "employee_id": o.employee_id,
                 "is_active": o.is_active} for o in s.query(Operator).all()]
    finally:
        s.close()


@router.post("/operators")
def create_operator(body: OperatorIn):
    from ..models import Operator
    s = _sf()()
    try:
        o = Operator(name=body.name, employee_id=body.employee_id.strip().upper())
        s.add(o); s.commit()
        return {"status": "created", "operator_id": o.id}
    except Exception as e:
        s.rollback()
        raise HTTPException(500, str(e))
    finally:
        s.close()


@router.post("/auth/register")
def register(body: UserIn):
    from ..models import User
    s = _sf()()
    try:
        if s.query(User).filter_by(username=body.username.strip().upper()).first():
            raise HTTPException(400, "Usuario ya existe")
        # Sin passwords: solo gafete/batch. Se guarda vacío.
        u = User(username=body.username.strip().upper(), password_hash="", role=body.role)
        s.add(u); s.commit()
        return {"status": "created", "user_id": u.id}
    finally:
        s.close()


@router.post("/auth/login")
def login(body: LoginIn):
    from ..models import User
    from ..auth import create_token
    s = _sf()()
    try:
        u = s.query(User).filter_by(username=body.username.strip().upper()).first()
        if not u or not u.is_active:
            raise HTTPException(401, "Usuario no existe o inactivo")
        # Sin passwords: entra solo con batch/gafete
        return {"token": create_token(u.username, u.role), "role": u.role, "username": u.username}
    finally:
        s.close()


# ---------- usuarios (solo batch/nombre + permisos) ----------
class UserUpIn(BaseModel):
    username: Optional[str] = None
    password: Optional[str] = None
    role: Optional[str] = None
    is_active: Optional[bool] = None


@router.get("/users")
def list_users():
    from ..models import User
    s = _sf()()
    try:
        return [{"id": u.id, "username": u.username, "role": u.role, "is_active": u.is_active,
                 "created_at": str(u.created_at)} for u in s.query(User).order_by(User.id).all()]
    finally:
        s.close()


@router.put("/users/{uid}")
def update_user(uid: int, body: UserUpIn):
    from ..models import User
    from ..auth import hash_password
    s = _sf()()
    try:
        u = s.query(User).get(uid)
        if not u:
            raise HTTPException(404, "Usuario no encontrado")
        if body.username and body.username.strip().upper() != u.username:
            if s.query(User).filter(User.username == body.username.strip().upper(), User.id != uid).first():
                raise HTTPException(400, "Usuario ya existe")
            u.username = body.username.strip().upper()
        if body.password:
            if len(body.password) < 4:
                raise HTTPException(400, "Password mínimo 4 caracteres")
            u.password_hash = hash_password(body.password)
        if body.role and body.role in ("operator","admin"):
            # no dejar sin admin
            if u.role == "admin" and body.role != "admin":
                admins = s.query(User).filter(User.role == "admin", User.is_active == True).count()  # noqa: E712
                if admins <= 1:
                    raise HTTPException(400, "Debe quedar al menos un admin activo")
            u.role = body.role
        if body.password is not None:
            # Passwords eliminados: se ignora, se limpia
            u.password_hash = ""
        if body.is_active is not None:
            if u.role == "admin" and body.is_active == False:
                admins = s.query(User).filter(User.role == "admin", User.is_active == True).count()  # noqa: E712
                if admins <= 1 and u.is_active:
                    raise HTTPException(400, "Debe quedar al menos un admin activo")
            u.is_active = bool(body.is_active)
        s.commit()
        return {"status": "updated"}
    finally:
        s.close()


@router.delete("/users/{uid}")
def delete_user(uid: int):
    from ..models import User
    s = _sf()()
    try:
        u = s.query(User).get(uid)
        if not u:
            raise HTTPException(404, "Usuario no encontrado")
        if u.role == "admin":
            admins = s.query(User).filter(User.role == "admin").count()
            if admins <= 1:
                raise HTTPException(400, "No se puede borrar el último admin")
        s.delete(u); s.commit()
        return {"status": "deleted"}
    finally:
        s.close()


@router.get("/users/{uid}")
def get_user(uid: int):
    from ..models import User
    s = _sf()()
    try:
        u = s.query(User).get(uid)
        if not u:
            raise HTTPException(404, "Usuario no encontrado")
        perms = {
            "operator": ["Ejecutar pruebas (Modelo/Serie/Batch)", "Ver resultado BUENO/MALO", "Usar modo manual (sin password)", "Reimprimir/Log en principal"],
            "admin": ["Todo lo de operator", "Configurar recetas (altas/bajas/cambios)", "Gestionar usuarios y permisos", "PLC/Hipot/BD, etiquetas, logs, import/export", "Ajustar espera de resultado e impresión"],
        }
        return {"id": u.id, "username": u.username, "role": u.role, "is_active": u.is_active,
                "permissions": perms.get(u.role, []), "created_at": str(u.created_at)}
    finally:
        s.close()


# ---------- executions (flujo operador doc 02) ----------
class ExecStart(BaseModel):
    recipe_id: int
    operator_id: Optional[int] = None
    batch: Optional[str] = None
    serial_number: Optional[str] = None


@router.post("/executions/start")
def exec_start(body: ExecStart):
    from ..models import Execution, Batch
    s = _sf()()
    try:
        batch_id = None
        if body.batch:
            b = s.query(Batch).filter_by(batch_number=body.batch.strip().upper()).first()
            if not b:
                b = Batch(batch_number=body.batch.strip().upper(),
                          operator_id=body.operator_id)
                s.add(b); s.flush()
            batch_id = b.id
        ex = Execution(recipe_id=body.recipe_id, operator_id=body.operator_id,
                       batch_id=batch_id,
                       serial_number=(body.serial_number or "").strip().upper(),
                       plc_start_time=datetime.now())
        s.add(ex); s.commit()
        return {"execution_id": ex.id, "status": "started"}
    finally:
        s.close()


@router.post("/executions/{eid}/plc-result")
def exec_plc(eid: int, passed: bool, readings: list = []):
    from ..models import Execution, PlcReading
    s = _sf()()
    try:
        ex = s.query(Execution).get(eid)
        if not ex:
            raise HTTPException(404, "Execution no encontrada")
        ex.plc_end_time = datetime.now()
        ex.plc_pass_fail = passed
        if ex.plc_start_time:
            ex.plc_duration = (ex.plc_end_time - ex.plc_start_time).total_seconds()
        for r in readings:
            s.add(PlcReading(execution_id=eid, recipe_id=ex.recipe_id,
                             component_name=r.get("component"), voltage=r.get("voltage"),
                             tag_name=r.get("tag"), value=r.get("value"),
                             min_limit=r.get("min"), max_limit=r.get("max"),
                             is_pass=r.get("pass")))
        s.commit()
        return {"status": "saved", "plc_pass": passed}
    finally:
        s.close()


@router.post("/executions/{eid}/hipot-result")
def exec_hipot(eid: int, body: dict):
    from ..models import Execution, HipotResult
    s = _sf()()
    try:
        ex = s.query(Execution).get(eid)
        if not ex:
            raise HTTPException(404, "Execution no encontrada")
        ex.hipot_start_time = datetime.fromisoformat(body["start"]) if body.get("start") else datetime.now()
        ex.hipot_end_time = datetime.now()
        ex.hipot_pass_fail = bool(body.get("pass", False))
        if ex.hipot_start_time and ex.hipot_end_time:
            ex.hipot_duration = (ex.hipot_end_time - ex.hipot_start_time).total_seconds()
        s.add(HipotResult(execution_id=eid, recipe_id=ex.recipe_id,
                          voltage=body.get("voltage"), current=body.get("current"),
                          resistance=body.get("resistance"),
                          insulation_resistance=body.get("insulation"),
                          test_duration=body.get("duration"),
                          is_pass=bool(body.get("pass", False))))
        # final
        ex.final_status = "GOOD" if (ex.plc_pass_fail and ex.hipot_pass_fail) else "BAD"
        if ex.plc_start_time:
            ex.total_duration = (ex.hipot_end_time - ex.plc_start_time).total_seconds()
        s.commit()
        return {"status": "completed", "final_status": ex.final_status}
    finally:
        s.close()


@router.get("/executions")
def list_executions(limit: int = 50):
    from ..models import Execution
    s = _sf()()
    try:
        rows = s.query(Execution).order_by(Execution.id.desc()).limit(limit).all()
        return [{"id": e.id, "recipe_id": e.recipe_id, "serial": e.serial_number,
                 "plc": e.plc_pass_fail, "hipot": e.hipot_pass_fail,
                 "final": e.final_status, "created": str(e.created_at)} for e in rows]
    finally:
        s.close()


# ---------- hardware ----------
@router.get("/status/plc")
def plc_status():
    from ..plc_manager import PLCManager
    from ..models import Configuration
    s = _sf()()
    try:
        ip = (s.query(Configuration).filter_by(config_key="plc_ip").first().config_value
              if s.query(Configuration).filter_by(config_key="plc_ip").first() else "192.168.1.10")
    finally:
        s.close()
    plc = PLCManager(ip=ip, simulate=True)
    plc.connect()
    return {"connected": plc.connected, "ip": ip, "simulate": plc.simulate}


@router.get("/status/hipot")
def hipot_status():
    from ..hipot_manager import HipotManager
    h = HipotManager(simulate=True)
    h.connect()
    return {"connected": h.connected, "ports": HipotManager.list_ports()}


@router.get("/health")
def health():
    from ..database import local_db
    return {"status": "healthy",
            "services": {"database": "connected" if local_db.test_connection() else "disconnected",
                         "plc": "ready", "hipot": "ready"}}


# ---------- admin: config / sync / logs / print ----------
@router.get("/admin/config")
def get_config():
    from ..models import Configuration
    s = _sf()()
    try:
        return {c.config_key: c.config_value for c in s.query(Configuration).all()}
    finally:
        s.close()


@router.post("/admin/config")
def set_config(body: dict):
    from ..models import Configuration
    s = _sf()()
    try:
        for k, v in body.items():
            ex = s.query(Configuration).filter_by(config_key=k).first()
            if ex:
                ex.config_value = str(v)
            else:
                s.add(Configuration(config_key=k, config_value=str(v)))
        s.commit()
        return {"status": "saved"}
    finally:
        s.close()


@router.post("/admin/sync-now")
def sync_now():
    from ..database import local_db, get_remote_db
    from ..sync_manager import SyncManager
    try:
        mgr = SyncManager(local_db.Session, get_remote_db().Session)
        n = mgr.sync_pending()
        return {"status": "ok", "synced": n}
    except Exception as e:
        return {"status": "error", "error": str(e), "synced": 0}


@router.get("/admin/logs")
def get_logs(category: str = "", level: str = "", q: str = "", limit: int = 200):
    from ..models import AppLog
    s = _sf()()
    try:
        query = s.query(AppLog).order_by(AppLog.id.desc())
        if category: query = query.filter(AppLog.category == category)
        if level: query = query.filter(AppLog.level == level)
        if q: query = query.filter(AppLog.detail.contains(q))
        rows = query.limit(limit).all()
        return [{"id": l.id, "ts": str(l.timestamp), "level": l.level,
                 "cat": l.category, "action": l.action, "detail": l.detail,
                 "sent": l.value_sent, "recv": l.value_received,
                 "ms": l.duration_ms, "ok": l.success} for l in rows]
    finally:
        s.close()


@router.post("/admin/print-test")
def print_test():
    from ..label_printer import ZebraPrinter
    p = ZebraPrinter()
    ok = p.test_print()
    return {"status": "ok" if ok else "error"}


@router.post("/executions/{eid}/reprint")
def reprint(eid: int, copies: int = 1):
    from ..models import Execution, LabelTemplate, LabelPrint
    from ..label_printer import ZebraPrinter
    from ..label_generator import DEFAULT_TEMPLATE_JSON
    s = _sf()()
    try:
        ex = s.query(Execution).get(eid)
        if not ex:
            raise HTTPException(404, "Execution no encontrada")
        tpl = s.query(LabelTemplate).filter_by(is_default=True).first()
        zpl_json = tpl.canvas_data if tpl else DEFAULT_TEMPLATE_JSON
        from ..label_generator import canvas_to_zpl
        data = {"model": "", "serial_number": ex.serial_number,
                "final_status": ex.final_status or ""}
        zpl = canvas_to_zpl(zpl_json, data)
        ok = ZebraPrinter().print_zpl(zpl)
        s.add(LabelPrint(execution_id=eid, printed_by="admin_reprint",
                         zpl_data=zpl, status="success" if ok else "error",
                         copies=copies))
        s.commit()
        return {"status": "ok" if ok else "error"}
    finally:
        s.close()


# ---------- label templates CRUD ----------
class TemplateIn(BaseModel):
    name: str
    width_mm: int = 100
    height_mm: int = 60
    canvas_data: str = ""
    is_default: bool = False


@router.get("/admin/templates")
def list_templates():
    from ..models import LabelTemplate
    s = _sf()()
    try:
        return [{"id": t.id, "name": t.name, "w": t.width_mm, "h": t.height_mm,
                 "is_default": t.is_default} for t in s.query(LabelTemplate).all()]
    finally:
        s.close()


@router.get("/admin/templates/{tid}")
def get_template(tid: int):
    from ..models import LabelTemplate
    s = _sf()()
    try:
        t = s.query(LabelTemplate).get(tid)
        if not t:
            raise HTTPException(404, "Plantilla no encontrada")
        return {"id": t.id, "name": t.name, "width_mm": t.width_mm,
                "height_mm": t.height_mm, "canvas_data": t.canvas_data,
                "is_default": t.is_default}
    finally:
        s.close()


@router.post("/admin/templates")
def save_template(body: TemplateIn):
    from ..models import LabelTemplate
    s = _sf()()
    try:
        if body.is_default:
            for t in s.query(LabelTemplate).filter_by(is_default=True).all():
                t.is_default = False
        t = LabelTemplate(name=body.name, width_mm=body.width_mm,
                          height_mm=body.height_mm, canvas_data=body.canvas_data,
                          is_default=body.is_default)
        s.add(t); s.commit()
        return {"status": "saved", "id": t.id}
    finally:
        s.close()


@router.put("/admin/templates/{tid}")
def update_template(tid: int, body: TemplateIn):
    from ..models import LabelTemplate
    s = _sf()()
    try:
        t = s.query(LabelTemplate).get(tid)
        if not t:
            raise HTTPException(404, "Plantilla no encontrada")
        if body.is_default:
            for x in s.query(LabelTemplate).filter_by(is_default=True).all():
                x.is_default = False
        t.name, t.width_mm, t.height_mm = body.name, body.width_mm, body.height_mm
        t.canvas_data, t.is_default = body.canvas_data, body.is_default
        s.commit()
        return {"status": "saved"}
    finally:
        s.close()


@router.delete("/admin/templates/{tid}")
def delete_template(tid: int):
    from ..models import LabelTemplate
    s = _sf()()
    try:
        t = s.query(LabelTemplate).get(tid)
        if not t:
            raise HTTPException(404, "Plantilla no encontrada")
        s.delete(t); s.commit()
        return {"status": "deleted"}
    finally:
        s.close()


# ---------- import / export ----------
@router.get("/admin/export")
def export_config():
    from ..config_manager import ConfigManager
    mgr = ConfigManager(_sf())
    return {"data": mgr.export_config()}


@router.post("/admin/import")
def import_config(body: dict):
    from ..config_manager import ConfigManager
    mgr = ConfigManager(_sf())
    try:
        res = mgr.import_config(body.get("data", "{}"),
                                mode=body.get("mode", "overwrite"))
        return res
    except Exception as e:
        raise HTTPException(400, str(e))


@router.get("/admin/import-export-log")
def import_export_log(limit: int = 20):
    from ..models import ImportExportLog
    s = _sf()()
    try:
        rows = s.query(ImportExportLog).order_by(ImportExportLog.id.desc()).limit(limit).all()
        return [{"id": r.id, "op": r.operation_type, "mode": r.mode,
                 "summary": r.data_summary, "status": r.status,
                 "at": str(r.performed_at)} for r in rows]
    finally:
        s.close()


# ---------- plc config & tags ----------
@router.get("/admin/plc-config")
def get_plc_config():
    from ..models import Configuration
    s = _sf()()
    try:
        ip = s.query(Configuration).filter_by(config_key="plc_ip").first()
        rack = s.query(Configuration).filter_by(config_key="plc_rack").first()
        slot = s.query(Configuration).filter_by(config_key="plc_slot").first()
        sim = s.query(Configuration).filter_by(config_key="plc_simulate").first()
        return {
            "ip": ip.config_value if ip else "192.168.1.10",
            "rack": int(rack.config_value) if rack else 0,
            "slot": int(slot.config_value) if slot else 1,
            "simulate": (sim.config_value.lower() == "true") if sim else True,
        }
    finally:
        s.close()


@router.post("/admin/plc-config")
def set_plc_config(body: dict):
    from ..models import Configuration
    s = _sf()()
    try:
        keys = {
            "plc_ip": str(body.get("ip", "192.168.1.10")),
            "plc_rack": str(body.get("rack", 0)),
            "plc_slot": str(body.get("slot", 1)),
            "plc_simulate": str(body.get("simulate", True)),
        }
        for k, v in keys.items():
            ex = s.query(Configuration).filter_by(config_key=k).first()
            if ex:
                ex.config_value = v
            else:
                s.add(Configuration(config_key=k, config_value=v))
        s.commit()
        p = _plc()
        p.update_config(ip=keys["plc_ip"], rack=int(keys["plc_rack"]),
                         slot=int(keys["plc_slot"]), simulate=(keys["plc_simulate"].lower() == "true"))
        return {"status": "saved", "config": get_plc_config()}
    finally:
        s.close()


@router.get("/admin/plc-tags")
def list_plc_tags():
    from ..models import PlcTag
    s = _sf()()
    try:
        tags = s.query(PlcTag).all()
        plc = _plc()
        out = []
        for t in tags:
            try:
                val = plc.read_tag(t.db_number, t.offset, t.data_type, t.bit)
            except Exception:
                val = None
            out.append({
                "id": t.id, "name": t.name, "db": t.db_number, "offset": t.offset,
                "type": t.data_type, "bit": t.bit, "value": val,
                "description": t.description, "unit": t.unit
            })
        return out
    finally:
        s.close()


@router.post("/admin/plc-tags")
def create_plc_tag(body: dict):
    from ..models import PlcTag
    s = _sf()()
    try:
        t = PlcTag(name=body.get("name", "").strip().upper(),
                   db_number=int(body.get("db", 1)),
                   offset=int(body.get("offset", 0)),
                   data_type=body.get("type", "REAL").strip().upper(),
                   bit=int(body.get("bit", 0)),
                   description=body.get("description"),
                   unit=body.get("unit"))
        s.add(t); s.commit()
        return {"status": "created", "id": t.id}
    finally:
        s.close()


@router.delete("/admin/plc-tags/{tid}")
def delete_plc_tag(tid: int):
    from ..models import PlcTag
    s = _sf()()
    try:
        t = s.query(PlcTag).get(tid)
        if not t:
            raise HTTPException(404, "Tag no encontrado")
        s.delete(t); s.commit()
        return {"status": "deleted"}
    finally:
        s.close()


@router.post("/admin/plc-read")
def plc_read(body: dict):
    plc = _plc()
    db, off = int(body.get("db", 1)), int(body.get("offset", 0))
    typ = body.get("type", "REAL")
    bit = int(body.get("bit", 0))
    try:
        val = plc.read_tag(db, off, typ, bit)
        return {"value": val}
    except Exception as e:
        raise HTTPException(500, str(e))


@router.post("/admin/plc-write")
def plc_write(body: dict):
    plc = _plc()
    tag_id = body.get("tag_id")
    if tag_id:
        from ..models import PlcTag
        s = _sf()()
        try:
            tag = s.query(PlcTag).get(tag_id)
            if not tag:
                raise HTTPException(404, "Tag no encontrado")
            db, off, typ, bit = tag.db_number, tag.offset, tag.data_type, tag.bit
        finally:
            s.close()
    else:
        db = int(body.get("db", 1))
        off = int(body.get("offset", 0))
        typ = body.get("type", "REAL")
        bit = int(body.get("bit", 0))
    val = body.get("value")
    if val is None:
        raise HTTPException(400, "value es requerido")
    try:
        plc.write_tag(db, off, typ, val, bit)
        return {"status": "written", "value": val}
    except Exception as e:
        raise HTTPException(500, str(e))


@router.post("/admin/hipot-cmd")
def hipot_cmd(body: dict):
    from ..hipot_manager import HipotManager
    h = HipotManager(simulate=True)
    h.connect()
    ok = h.send_command(body.get("command", "STATUS?"))
    resp = h.read_response(timeout=0.5)
    parsed = HipotManager.parse_measurement(resp or "")
    return {"sent": ok, "response": resp, "parsed": parsed}


@router.get("/admin/sync-log")
def sync_log(limit: int = 20):
    from ..models import SyncLog
    s = _sf()()
    try:
        rows = s.query(SyncLog).order_by(SyncLog.id.desc()).limit(limit).all()
        return [{"id": r.id, "dir": r.direction, "table": r.table_name,
                 "n": r.records_synced, "status": r.status,
                 "err": r.error_message} for r in rows]
    finally:
        s.close()


@router.get("/history")
def history(model: str = "", serial: str = "", limit: int = 200, start: str = "", end: str = ""):
    from ..models import Execution, Recipe, Batch
    from datetime import datetime, timedelta
    s = _sf()()
    try:
        q = s.query(Execution).order_by(Execution.id.desc())
        if serial:
            q = q.filter(Execution.serial_number.contains(serial.strip().upper()))
        if start:
            try:
                dt_start = datetime.fromisoformat(start)
                q = q.filter(Execution.created_at >= dt_start)
            except: pass
        if end:
            try:
                dt_end = datetime.fromisoformat(end) + timedelta(days=1)
                q = q.filter(Execution.created_at < dt_end)
            except: pass
        rows = q.limit(limit).all()
        # mapa recipe -> model
        recipe_map = {r.id: r.model for r in s.query(Recipe).all()}
        batch_map = {b.id: b.batch_number for b in s.query(Batch).all()}
        out = []
        for e in rows:
            out.append({
                "id": e.id, "recipe_id": e.recipe_id,
                "recipe_model": recipe_map.get(e.recipe_id, "-"),
                "serial": e.serial_number or "-",
                "batch": batch_map.get(e.batch_id, "-") if e.batch_id else "-",
                "plc": e.plc_pass_fail, "hipot": e.hipot_pass_fail,
                "final": e.final_status or "-", "created": str(e.created_at) if e.created_at else "-",
                "total_duration": e.total_duration,
            })
        if model:
            out = [x for x in out if model.strip().upper() in (x["recipe_model"] or "")]
        total = len(out)
        good = sum(1 for x in out if x["final"] == "GOOD")
        bad = sum(1 for x in out if x["final"] == "BAD")
        return {"rows": out, "total": total, "good": good, "bad": bad}
    finally:
        s.close()


# ---------- modo manual (sin guardar, sin modelo, 1-a-1) ----------
_shared_plc = None


def _plc():
    global _shared_plc
    if _shared_plc is None:
        from ..plc_manager import PLCManager
        _shared_plc = PLCManager(simulate=True)
        _shared_plc.connect()
    return _shared_plc


_shared_hipot = None


def _hipot():
    global _shared_hipot
    if _shared_hipot is None:
        from ..hipot_manager import HipotManager
        _shared_hipot = HipotManager(simulate=True)
        _shared_hipot.connect()
    return _shared_hipot


@router.post("/manual/mode")
def manual_mode(body: dict):
    """Activa modo PLC: 1=auto, 2=manual. DB1.DBW110."""
    mode = int(body.get("mode", 1))
    if mode not in (1, 2):
        raise HTTPException(400, "mode debe ser 1 o 2")
    ok = _plc().set_mode(mode)
    return {"mode": _plc().get_mode(), "ok": ok}


@router.get("/manual/mode")
def manual_mode_get():
    return {"mode": _plc().get_mode()}


@router.post("/manual/plc-test")
def manual_plc_test(body: dict):
    """Prueba 1 componente (hardware solo 1 a la vez). No guarda en BD."""
    comp = (body.get("component") or "").strip().upper()
    if not comp:
        raise HTTPException(400, "component requerido")
    _plc().set_mode(2)
    if not _plc().test_single_component(comp, float(body.get("voltage", 120))):
        raise HTTPException(500, "No se pudo disparar el test")
    import time
    time.sleep(0.6)
    try:
        vals = _plc().read_results()
    except Exception as e:
        raise HTTPException(500, str(e))
    v = vals.get(comp, 0.0)
    return {"component": comp, "value": v, "all": vals}


@router.post("/manual/hipot")
def manual_hipot(body: dict):
    """Hipot manual: {action:'START'|'STOP'|'STATUS'} — no guarda."""
    action = (body.get("action") or body.get("command") or "STATUS").strip().upper()
    h = _hipot()
    cmd = {"START": "START", "STOP": "STOP"}.get(action, "STATUS?")
    if action in ("START", "STOP"):
        h.send_command(cmd)
    else:
        h.send_command("STATUS?")
    import time
    time.sleep(0.45)
    from ..hipot_manager import HipotManager
    resp = h.read_response(timeout=0.6)
    return {"action": action, "response": resp, "parsed": HipotManager.parse_measurement(resp or "")}


@router.post("/manual/abort")
def manual_abort():
    try:
        _plc().stop_test()
        _hipot().send_command("STOP")
    except Exception:
        pass
    _plc().set_mode(1)
    return {"status": "aborted"}
