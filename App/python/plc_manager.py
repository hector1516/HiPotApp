"""PLC Siemens S7-1200 vía snap7 (doc 03/06). Con modo simulado para dev."""
import time

try:
    import snap7
    from snap7.util import get_real, set_real, get_bool, set_bool
    from snap7.types import Areas
    HAS_SNAP7 = True
except Exception:
    HAS_SNAP7 = False
    Areas = None

# Offsets DB1 (doc 06)
OFF_VOLTAGE = 0
COMP_OFFSETS = {  # name -> (min_off, max_off, enable_off)
    "FAN": (4, 8, 12), "DOOR": (16, 20, 24), "FRAME": (28, 32, 36),
    "DRAIN": (40, 44, 48), "DEFROST": (52, 56, 60), "LIGHT": (64, 68, 72),
}
OFF_START, OFF_STOP, OFF_COMPLETE, OFF_PASS, OFF_RUNNING = 100, 101, 102, 103, 104
OFF_MODE = 110  # INT: 1=auto, 2=manual (nueva variable solicitada)
RES_OFFSETS = {"FAN": 200, "DOOR": 204, "FRAME": 208, "DRAIN": 212,
               "DEFROST": 216, "LIGHT": 220, "TOTAL": 224}


class PLCManager:
    def __init__(self, ip="192.168.1.10", rack=0, slot=1, simulate=False):
        self.ip, self.rack, self.slot = ip, rack, slot
        self.simulate = simulate or not HAS_SNAP7
        self.connected = False
        self.client = snap7.client.Client() if HAS_SNAP7 and not self.simulate else None
        self._sim = {}  # memoriasimulada

    def connect(self) -> bool:
        if self.simulate:
            self.connected = True
            from .logger import logger
            logger.info("PLC", "connect", f"Simulado conectado a {self.ip}")
            return True
        try:
            self.client.connect(self.ip, self.rack, self.slot)
            self.connected = True
            from .logger import logger
            logger.info("PLC", "connect", f"Conectado a {self.ip} rack={self.rack} slot={self.slot}")
            return True
        except Exception as e:
            self.connected = False
            from .logger import logger
            logger.error("PLC", "connect", f"Error conectando a {self.ip}: {e}", error_message=str(e))
            return False

    def disconnect(self):
        if self.client and self.connected and not self.simulate:
            try:
                self.client.disconnect()
            except Exception:
                pass
        self.connected = False
        from .logger import logger
        logger.info("PLC", "disconnect", f"Desconectado de {self.ip}")

    def update_config(self, ip: str, rack: int = 0, slot: int = 1, simulate: bool = True):
        self.ip = ip
        self.rack = int(rack)
        self.slot = int(slot)
        self.simulate = simulate or not HAS_SNAP7
        if self.connected:
            self.disconnect()
        self.connect()

    # --- primitivas ---
    def read_word(self, db: int, start: int) -> int:
        if self.simulate:
            return int(self._sim.get((db, start, "W"), 0))
        data = self.client.db_read(db, start, 2)
        return int.from_bytes(data, "big", signed=False)

    def write_word(self, db: int, start: int, value: int):
        if self.simulate:
            self._sim[(db, start, "W")] = int(value) & 0xFFFF
            return
        self.client.db_write(db, start, int(value).to_bytes(2, "big"))

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

    def set_mode(self, mode: int) -> bool:
        """1=auto, 2=manual. DB1.DBW110."""
        if mode not in (1, 2):
            return False
        try:
            self.write_word(1, OFF_MODE, int(mode))
            return True
        except Exception:
            return False

    def get_mode(self) -> int:
        try:
            return self.read_word(1, OFF_MODE)
        except Exception:
            return 1

    def test_single_component(self, name: str, voltage: float = 120.0) -> bool:
        """Habilita solo `name` para el test 1-a-1 del modo manual."""
        try:
            n = name.strip().upper()
            for comp, offs in COMP_OFFSETS.items():
                self.write_bool(1, offs[2], comp == n)
            self.write_real(1, OFF_VOLTAGE, float(voltage))
            self.write_bool(1, OFF_START, True)
            if self.simulate:
                self._sim[(1, OFF_RUNNING, 0)] = True
                # valor simulado cercano al centro del rango
                offs = COMP_OFFSETS.get(n)
                if offs:
                    lo = float(self._sim.get((1, offs[0]), 0.0))
                    hi = float(self._sim.get((1, offs[1]), 1.0))
                    import random
                    mid = (lo + hi) / 2 if hi > lo else 0.5
                    span = (hi - lo) if hi > lo else 0.4
                    v = mid + random.uniform(-span * 0.2, span * 0.2)
                    ro = RES_OFFSETS.get(n)
                    if ro is not None:
                        self._sim[(1, ro)] = float(v)
            return True
        except Exception:
            return False

    def read_real(self, db: int, start: int) -> float:
        if self.simulate:
            return float(self._sim.get((db, start), 0.0))
        from snap7.util import get_real as _gr
        return _gr(self.client.db_read(db, start, 4), 0)

    def write_real(self, db: int, start: int, value: float):
        if self.simulate:
            self._sim[(db, start)] = float(value)
            return
        data = bytearray(4)
        set_real(data, 0, float(value))
        self.client.db_write(db, start, data)

    def read_bool(self, db: int, start: int, bit: int = 0) -> bool:
        if self.simulate:
            return bool(self._sim.get((db, start, bit), False))
        return get_bool(self.client.db_read(db, start, 1), 0, bit)

    def write_bool(self, db: int, start: int, value: bool, bit: int = 0):
        if self.simulate:
            self._sim[(db, start, bit)] = bool(value)
            return
        data = bytearray(self.client.db_read(db, start, 1))
        set_bool(data, 0, bit, bool(value))
        self.client.db_write(db, start, data)

    # --- IO directo (Q, I, IA) ---
    _Q_MAP = {
        "Q0.0": (0, 0), "Q0.1": (0, 1), "Q0.2": (0, 2), "Q0.3": (0, 3),
        "Q0.4": (0, 4), "Q0.5": (0, 5), "Q0.6": (0, 6), "Q0.7": (0, 7),
        "Q1.0": (1, 0), "Q1.1": (1, 1), "Q2.0": (2, 0), "Q2.1": (2, 1),
        "Q2.2": (2, 2),
    }
    _I_MAP = {
        "I0.0": (0, 0), "I0.1": (0, 1), "I0.2": (0, 2),
    }
    _IA_MAP = {"IA1": 0, "IA2": 2, "IA3": 4, "IA4": 6}

    def _parse_q(self, addr: str):
        addr = (addr or "").upper().strip()
        if addr in self._Q_MAP:
            return self._Q_MAP[addr]
        # fallback parse Qx.y
        parts = addr.replace("Q", "").split(".")
        off = int(parts[0])
        bit = int(parts[1]) if len(parts) > 1 else 0
        return (off, bit)

    def _parse_i(self, addr: str):
        addr = (addr or "").upper().strip()
        if addr in self._I_MAP:
            return self._I_MAP[addr]
        parts = addr.replace("I", "").split(".")
        off = int(parts[0])
        bit = int(parts[1]) if len(parts) > 1 else 0
        return (off, bit)

    def read_digital_output(self, q: str) -> bool:
        off, bit = self._parse_q(q)
        if self.simulate:
            return bool(self._sim.get(("Q", off, bit), False))
        if Areas is None:
            return False
        try:
            data = self.client.read_area(Areas.PA, 0, off, 1)
            return get_bool(data, 0, bit)
        except Exception:
            return False

    def write_digital_output(self, q: str, value: bool):
        off, bit = self._parse_q(q)
        if self.simulate:
            self._sim[("Q", off, bit)] = bool(value)
            from .logger import logger
            logger.info("PLC", "write_Q", f"{q} = {value}")
            return
        if Areas is None:
            return
        try:
            data = bytearray(self.client.read_area(Areas.PA, 0, off, 1))
            set_bool(data, 0, bit, bool(value))
            self.client.write_area(Areas.PA, 0, off, data)
            from .logger import logger
            logger.info("PLC", "write_Q", f"{q} = {value}")
        except Exception as e:
            from .logger import logger
            logger.error("PLC", "write_Q", f"Error escribiendo {q}: {e}", error_message=str(e))

    def read_digital_input(self, i: str) -> bool:
        off, bit = self._parse_i(i)
        if self.simulate:
            return bool(self._sim.get(("I", off, bit), False))
        if Areas is None:
            return False
        try:
            data = self.client.read_area(Areas.PE, 0, off, 1)
            return get_bool(data, 0, bit)
        except Exception:
            return False

    def read_analog_input(self, ia: str) -> float:
        ia = (ia or "").upper().strip()
        off = self._IA_MAP.get(ia, 0)
        if self.simulate:
            return float(self._sim.get(("IA", off), 0.0))
        if Areas is None:
            return 0.0
        try:
            data = self.client.read_area(Areas.PE, 0, off, 2)
            return int.from_bytes(data, "big", signed=False) / 10.0
        except Exception:
            return 0.0

    def get_io_status(self) -> dict:
        q_status = {q: self.read_digital_output(q) for q in self._Q_MAP}
        i_status = {i: self.read_digital_input(i) for i in self._I_MAP}
        ia_status = {ia: self.read_analog_input(ia) for ia in self._IA_MAP}
        return {"outputs": q_status, "inputs": i_status, "analog": ia_status}

    def test_connection(self) -> bool:
        if self.simulate:
            return self.connected
        try:
            self.client.db_read(1, 0, 1)
            return True
        except Exception:
            return False

    # --- receta / test ---
    def send_recipe(self, voltage: float, components: list[dict]) -> bool:
        """components: [{name, min, max, enabled}]"""
        try:
            self.write_real(1, OFF_VOLTAGE, float(voltage))
            for c in components:
                offs = COMP_OFFSETS.get(c["name"].upper())
                if not offs:
                    continue
                self.write_real(1, offs[0], float(c.get("min", 0)))
                self.write_real(1, offs[1], float(c.get("max", 0)))
                self.write_bool(1, offs[2], bool(c.get("enabled", True)))
            return True
        except Exception:
            return False

    def start_test(self) -> bool:
        try:
            self.write_bool(1, OFF_START, True)
            if self.simulate:
                self._sim[(1, OFF_RUNNING, 0)] = True
            return True
        except Exception:
            return False

    def stop_test(self) -> bool:
        try:
            self.write_bool(1, OFF_STOP, True)
            if self.simulate:
                self._sim[(1, OFF_RUNNING, 0)] = False
            return True
        except Exception:
            return False

    def read_results(self) -> dict:
        out = {}
        for name, off in RES_OFFSETS.items():
            try:
                out[name] = self.read_real(1, off)
            except Exception:
                out[name] = 0.0
        try:
            out["complete"] = self.read_bool(1, OFF_COMPLETE)
            out["pass"] = self.read_bool(1, OFF_PASS)
            out["running"] = self.read_bool(1, OFF_RUNNING)
        except Exception:
            out.update(complete=False, pass_=False, running=False)
        return out

    def poll_until_complete(self, timeout_s: int = 180, interval: float = 0.4) -> dict:
        t0 = time.time()
        while time.time() - t0 < timeout_s:
            r = self.read_results()
            if r.get("complete"):
                return r
            time.sleep(interval)
        return self.read_results()
