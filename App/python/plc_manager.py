"""PLC Siemens S7-1200 vía snap7 (doc 03/06). Con modo simulado para dev."""
import time

try:
    import snap7
    from snap7.util import get_real, set_real, get_bool, set_bool
    HAS_SNAP7 = True
except Exception:
    HAS_SNAP7 = False

# Offsets por defecto (si no están mapeados en BD)
OFF_VOLTAGE = 0
COMP_OFFSETS = {  # name -> (min_off, max_off, enable_off)
    "FAN": (4, 8, 12), "DOOR": (16, 20, 24), "FRAME": (28, 32, 36),
    "DRAIN": (40, 44, 48), "DEFROST": (52, 56, 60), "LIGHT": (64, 68, 72),
}
OFF_START, OFF_STOP, OFF_COMPLETE, OFF_PASS, OFF_RUNNING = 100, 101, 102, 103, 104
OFF_MODE = 110  # INT: 1=auto, 2=manual
RES_OFFSETS = {"FAN": 200, "DOOR": 204, "FRAME": 208, "DRAIN": 212,
               "DEFROST": 216, "LIGHT": 220, "TOTAL": 224}


class PLCManager:
    def __init__(self, ip="10.188.141.10", rack=0, slot=1, simulate=False):
        self.ip, self.rack, self.slot = ip, rack, slot
        self.simulate = simulate or not HAS_SNAP7
        self.connected = False
        self.client = snap7.client.Client() if HAS_SNAP7 and not self.simulate else None
        self._sim = {}  # memoria simulada
        self._last_connect_attempt = 0

    def connect(self) -> bool:
        if self.simulate:
            self.connected = True
            return True
        if not HAS_SNAP7 or not self.client:
            self.connected = False
            return False
        now = time.time()
        # Evitar sobrecargar el socket si intentó conectar recientemente y falló
        if not self.connected and (now - self._last_connect_attempt < 2.0):
            return False
        self._last_connect_attempt = now
        try:
            try:
                self.client.disconnect()
            except Exception:
                pass
            self.client.connect(self.ip, self.rack, self.slot)
            self.connected = True
            return True
        except Exception:
            self.connected = False
            return False

    def ensure_connected(self):
        if not self.simulate and not self.connected:
            self.connect()

    def _handle_snap7_error(self, e: Exception):
        msg = str(e).lower()
        # Si es un error de dirección inválida (0x05) u otro error de offset del PLC, la conexión física sigue activa.
        if "invalid address" in msg or "0x05" in msg or "out of range" in msg:
            return
        # Solo desconectar si realmente se perdió el socket / TCP
        if any(term in msg for term in ["connection", "socket", "refused", "reset", "unreachable", "timeout"]):
            self.connected = False

    def disconnect(self):
        if self.client and self.connected and not self.simulate:
            try:
                self.client.disconnect()
            except Exception:
                pass
        self.connected = False

    def update_config(self, ip: str, rack: int = 0, slot: int = 1, simulate: bool = True):
        self.ip = ip
        self.rack = int(rack)
        self.slot = int(slot)
        self.simulate = simulate or not HAS_SNAP7
        self._last_connect_attempt = 0
        if self.connected:
            self.disconnect()
        self.connect()

    # --- primitivas seguras ---
    def read_word(self, db: int, start: int) -> int:
        if self.simulate:
            return int(self._sim.get((db, start, "W"), 0))
        self.ensure_connected()
        try:
            data = self.client.db_read(db, start, 2)
            return int.from_bytes(data, "big", signed=False)
        except Exception as e:
            self._handle_snap7_error(e)
            raise

    def write_word(self, db: int, start: int, value: int):
        if self.simulate:
            self._sim[(db, start, "W")] = int(value) & 0xFFFF
            return
        self.ensure_connected()
        try:
            self.client.db_write(db, start, int(value).to_bytes(2, "big"))
        except Exception as e:
            self._handle_snap7_error(e)
            raise

    def read_real(self, db: int, start: int) -> float:
        if self.simulate:
            return float(self._sim.get((db, start), 0.0))
        self.ensure_connected()
        try:
            from snap7.util import get_real as _gr
            return _gr(self.client.db_read(db, start, 4), 0)
        except Exception as e:
            self._handle_snap7_error(e)
            raise

    def write_real(self, db: int, start: int, value: float):
        if self.simulate:
            self._sim[(db, start)] = float(value)
            return
        self.ensure_connected()
        try:
            data = bytearray(4)
            set_real(data, 0, float(value))
            self.client.db_write(db, start, data)
        except Exception as e:
            self._handle_snap7_error(e)
            raise

    def read_bool(self, db: int, start: int, bit: int = 0) -> bool:
        if self.simulate:
            return bool(self._sim.get((db, start, bit), False))
        self.ensure_connected()
        try:
            return get_bool(self.client.db_read(db, start, 1), 0, bit)
        except Exception as e:
            self._handle_snap7_error(e)
            raise

    def write_bool(self, db: int, start: int, value: bool, bit: int = 0):
        if self.simulate:
            self._sim[(db, start, bit)] = bool(value)
            return
        self.ensure_connected()
        try:
            data = bytearray(self.client.db_read(db, start, 1))
            set_bool(data, 0, bit, bool(value))
            self.client.db_write(db, start, data)
        except Exception as e:
            self._handle_snap7_error(e)
            raise

    def read_tag(self, db: int, start: int, data_type: str = "REAL", bit: int = 0):
        dt = (data_type or "REAL").upper()
        if dt == "BOOL":
            return self.read_bool(db, start, bit)
        elif dt in ("WORD", "INT"):
            return self.read_word(db, start)
        else:
            return self.read_real(db, start)

    def write_tag(self, db: int, start: int, data_type: str, value, bit: int = 0):
        dt = (data_type or "REAL").upper()
        if dt == "BOOL":
            val = bool(value) if isinstance(value, bool) else str(value).lower() in ("true", "1", "yes", "on")
            self.write_bool(db, start, val, bit)
        elif dt in ("WORD", "INT"):
            self.write_word(db, start, int(value))
        else:
            self.write_real(db, start, float(value))

    def set_mode(self, mode: int, db: int = 1001) -> bool:
        """1=auto, 2=manual."""
        if mode not in (1, 2):
            return False
        try:
            self.write_word(db, OFF_MODE, int(mode))
            return True
        except Exception:
            try:
                self.write_word(1, OFF_MODE, int(mode))
                return True
            except Exception:
                return False

    def get_mode(self, db: int = 1001) -> int:
        try:
            return self.read_word(db, OFF_MODE)
        except Exception:
            try:
                return self.read_word(1, OFF_MODE)
            except Exception:
                return 1

    def test_single_component(self, name: str, voltage: float = 120.0, db: int = 1001) -> bool:
        """Habilita solo `name` para el test 1-a-1 del modo manual."""
        try:
            n = name.strip().upper()
            for comp, offs in COMP_OFFSETS.items():
                try:
                    self.write_bool(db, offs[2], comp == n)
                except Exception:
                    pass
            try:
                self.write_real(db, OFF_VOLTAGE, float(voltage))
            except Exception:
                pass
            try:
                self.write_bool(db, OFF_START, True)
            except Exception:
                pass
            if self.simulate:
                self._sim[(db, OFF_RUNNING, 0)] = True
                offs = COMP_OFFSETS.get(n)
                if offs:
                    lo = float(self._sim.get((db, offs[0]), 0.0))
                    hi = float(self._sim.get((db, offs[1]), 1.0))
                    import random
                    mid = (lo + hi) / 2 if hi > lo else 0.5
                    span = (hi - lo) if hi > lo else 0.4
                    v = mid + random.uniform(-span * 0.2, span * 0.2)
                    ro = RES_OFFSETS.get(n)
                    if ro is not None:
                        self._sim[(db, ro)] = float(v)
            return True
        except Exception:
            return False

    def test_connection(self, db: int = 1001) -> bool:
        if self.simulate:
            return self.connected
        self.ensure_connected()
        if not self.connected:
            return False
        try:
            self.client.db_read(db, 0, 1)
            return True
        except Exception:
            try:
                self.client.db_read(1, 0, 1)
                return True
            except Exception:
                return False

    # --- receta / test ---
    def send_recipe(self, voltage: float, components: list[dict], db: int = 1001) -> bool:
        """components: [{name, min, max, enabled}]"""
        try:
            try:
                self.write_real(db, OFF_VOLTAGE, float(voltage))
            except Exception:
                pass
            for c in components:
                offs = COMP_OFFSETS.get(c["name"].upper())
                if not offs:
                    continue
                try:
                    self.write_real(db, offs[0], float(c.get("min", 0)))
                    self.write_real(db, offs[1], float(c.get("max", 0)))
                    self.write_bool(db, offs[2], bool(c.get("enabled", True)))
                except Exception:
                    pass
            return True
        except Exception:
            return False

    def start_test(self, db: int = 1001) -> bool:
        try:
            self.write_bool(db, OFF_START, True)
            if self.simulate:
                self._sim[(db, OFF_RUNNING, 0)] = True
            return True
        except Exception:
            return False

    def stop_test(self, db: int = 1001) -> bool:
        try:
            self.write_bool(db, OFF_STOP, True)
            if self.simulate:
                self._sim[(db, OFF_RUNNING, 0)] = False
            return True
        except Exception:
            return False

    def read_results(self, db: int = 1001) -> dict:
        out = {}
        for name, off in RES_OFFSETS.items():
            try:
                out[name] = self.read_real(db, off)
            except Exception:
                out[name] = 0.0
        try:
            out["complete"] = self.read_bool(db, OFF_COMPLETE)
            out["pass"] = self.read_bool(db, OFF_PASS)
            out["running"] = self.read_bool(db, OFF_RUNNING)
        except Exception:
            out.update(complete=False, pass_=False, running=False)
        return out

    def read_mapped_tags(self, tags_list: list) -> dict:
        """tags_list: lista de objetos/dicts de PlcTag cargados de BD"""
        self.ensure_connected()
        out = {}
        for t in tags_list:
            app_var = getattr(t, "app_variable", None) if hasattr(t, "app_variable") else (t.get("app_variable") if isinstance(t, dict) else None)
            if not app_var or app_var == "CUSTOM":
                continue
            db = getattr(t, "db_number", 1) if hasattr(t, "db_number") else t.get("db", 1)
            off = getattr(t, "offset", 0) if hasattr(t, "offset") else t.get("offset", 0)
            dt = getattr(t, "data_type", "REAL") if hasattr(t, "data_type") else t.get("type", "REAL")
            bit = getattr(t, "bit", 0) if hasattr(t, "bit") else t.get("bit", 0)
            name = getattr(t, "name", "") if hasattr(t, "name") else t.get("name", "")
            try:
                val = self.read_tag(db, off, dt, bit)
            except Exception as e:
                self._handle_snap7_error(e)
                val = None
            out[app_var] = {
                "key": app_var,
                "name": name,
                "db": db,
                "offset": off,
                "type": dt,
                "bit": bit,
                "value": val,
                "address": f"DB{db} @ {off}" + (f".{bit}" if dt.upper() == "BOOL" else "")
            }
        return out

    def poll_until_complete(self, timeout_s: int = 180, interval: float = 0.4) -> dict:
        t0 = time.time()
        while time.time() - t0 < timeout_s:
            r = self.read_results()
            if r.get("complete"):
                return r
            time.sleep(interval)
        return self.read_results()
