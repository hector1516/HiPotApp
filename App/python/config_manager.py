"""Import/Export configuración JSON con transacción (doc 07 §4)."""
import json
from datetime import datetime


class ConfigManager:
    VERSION = "1.0.0"

    def __init__(self, session_factory, station_id: str = "ESTACION-01"):
        self.sf = session_factory
        self.station_id = station_id

    # ---- export ----
    def export_config(self, user_id=None, include=("recipes", "configurations",
                                                   "label_templates", "users", "plc_tags")) -> str:
        from .models import (Recipe, RecipeComponent, RecipeTotal, Configuration,
                             LabelTemplate, User, PlcTag)
        s = self.sf()
        try:
            out = {"metadata": {"version": self.VERSION,
                                "exported_at": datetime.now().isoformat(),
                                "exported_by": user_id, "station_id": self.station_id},
                   "recipes": [], "configurations": [], "label_templates": [],
                   "users": [], "plc_tags": []}
            if "recipes" in include:
                for r in s.query(Recipe).all():
                    d = {c.name: getattr(r, c.name) for c in Recipe.__table__.columns}
                    d["components"] = [{c.name: getattr(x, c.name)
                                        for c in RecipeComponent.__table__.columns}
                                       for x in s.query(RecipeComponent).filter_by(recipe_id=r.id).all()]
                    d["totals"] = [{c.name: getattr(x, c.name)
                                    for c in RecipeTotal.__table__.columns}
                                   for x in s.query(RecipeTotal).filter_by(recipe_id=r.id).all()]
                    out["recipes"].append(d)
            if "configurations" in include:
                out["configurations"] = [{c.name: getattr(x, c.name)
                                          for c in Configuration.__table__.columns}
                                         for x in s.query(Configuration).all()]
            if "label_templates" in include:
                out["label_templates"] = [{c.name: getattr(x, c.name)
                                           for c in LabelTemplate.__table__.columns}
                                          for x in s.query(LabelTemplate).all()]
            if "users" in include:
                out["users"] = [{c.name: getattr(x, c.name)
                                 for c in User.__table__.columns}
                                for x in s.query(User).all()]
            if "plc_tags" in include:
                out["plc_tags"] = [{c.name: getattr(x, c.name)
                                    for c in PlcTag.__table__.columns}
                                   for x in s.query(PlcTag).all()]
            self._log(s, "export", user_id, out, status="success")
            s.commit()
            return json.dumps(out, indent=2, default=str)
        finally:
            s.close()

    # ---- import ----
    def detect_conflicts(self, config: dict) -> list:
        from .models import Recipe
        s = self.sf()
        try:
            out = []
            for r in config.get("recipes", []):
                ex = s.query(Recipe).filter_by(model=r.get("model")).first()
                if ex:
                    out.append({"type": "recipe", "field": "model",
                                "value": r.get("model"), "existing_id": ex.id})
            return out
        finally:
            s.close()

    def import_config(self, json_data: str, user_id=None, mode: str = "overwrite") -> dict:
        from .models import (Recipe, RecipeComponent, RecipeTotal, Configuration,
                             LabelTemplate, User, PlcTag)
        config = json.loads(json_data)
        if "metadata" not in config:
            raise ValueError("Archivo inválido: falta metadata")
        conflicts = self.detect_conflicts(config)
        s = self.sf()
        try:
            n = 0
            for r in config.get("recipes", []):
                ex = s.query(Recipe).filter_by(model=r.get("model")).first()
                if ex and mode == "add":
                    continue
                if ex and mode in ("overwrite", "merge"):
                    for k in ("name", "type", "product", "voltage_levels", "is_active"):
                        if k in r:
                            setattr(ex, k, r[k])
                    rid = ex.id
                else:
                    obj = Recipe(name=r.get("name", ""), model=r.get("model", ""),
                                 type=r.get("type", "current_consumption"),
                                 product=r.get("product"),
                                 voltage_levels=r.get("voltage_levels"),
                                 is_active=r.get("is_active", True))
                    s.add(obj); s.flush()
                    rid = obj.id
                for c in r.get("components", []):
                    s.add(RecipeComponent(recipe_id=rid, name=c.get("name", ""),
                                          voltage=c.get("voltage", ""),
                                          enabled=c.get("enabled", True),
                                          min_value=c.get("min_value", 0),
                                          max_value=c.get("max_value", 0),
                                          unit=c.get("unit", "A")))
                for t in r.get("totals", []):
                    s.add(RecipeTotal(recipe_id=rid, voltage=t.get("voltage", ""),
                                      min_total=t.get("min_total", 0),
                                      max_total=t.get("max_total", 0)))
                n += 1
            for c in config.get("configurations", []):
                ex = s.query(Configuration).filter_by(config_key=c.get("config_key")).first()
                if ex:
                    ex.config_value = c.get("config_value", "")
                else:
                    s.add(Configuration(config_key=c.get("config_key", ""),
                                        config_value=c.get("config_value", "")))
            for t in config.get("label_templates", []):
                s.add(LabelTemplate(name=t.get("name", "importada"),
                                    width_mm=t.get("width_mm", 100),
                                    height_mm=t.get("height_mm", 60),
                                    canvas_data=t.get("canvas_data"),
                                    is_default=False))
            for u in config.get("users", []):
                if not s.query(User).filter_by(username=u.get("username")).first():
                    s.add(User(username=u.get("username", ""),
                               password_hash=u.get("password_hash", ""),
                               role=u.get("role", "operator"),
                               is_active=u.get("is_active", True)))
            self._log(s, "import", user_id, config, mode=mode,
                      conflicts=len(conflicts), imported=n, status="success")
            s.commit()
            return {"status": "success", "imported": n, "conflicts": conflicts}
        except Exception as e:
            s.rollback()
            raise e
        finally:
            s.close()

    def _log(self, s, op, user_id, config, mode=None, conflicts=0,
             imported=0, status="success", err=None):
        from .models import ImportExportLog
        s.add(ImportExportLog(
            operation_type=op, user_id=user_id, mode=mode,
            data_summary=json.dumps({"recipes": len(config.get("recipes", [])),
                                     "templates": len(config.get("label_templates", [])),
                                     "users": len(config.get("users", []))}, default=str),
            conflicts_found=conflicts, records_imported=imported,
            status=status, error_message=err))
