"""PLC Siemens S7-1200 vía snap7 (doc 03/06). Con modo simulado para dev."""
import time

try:
    import snap7
    from snap7.util import get_real, set_real, get_bool, set_bool
    HAS_SNAP7 = True
except Exception:
    HAS_SNAP7 = False

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
            return True
        try:
            self.client.connect(self.ip, self.rack, self.slot)
            self.connected = True
            return True
        except Exception:
            self.connected = False
            return False

    def disconnect(self):
        if self.client and self.connected and not self.simulate:
            try:
                self.client.disconnect()
            except Exception:
                pass
        self.connected = False

    # --- primitivas ---
    def read_word(self, db: int, start: int) -> int:
        if self.simulate:
            return int(self._sim.get((db, start, "W"), 1))
        data = self.client.db_read(db, start, 2)
        return int.from_bytes(data, "big", signed=False)

    def write_word(self, db: int, start: int, value: int):
        if self.simulate:
            self._sim[(db, start, "W")] = int(value) & 0xFFFF
            return
        self.client.db_write(db, start, int(value).to_bytes(2, "big"))

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
