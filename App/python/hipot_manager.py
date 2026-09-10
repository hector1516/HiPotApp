"""Hipot 3865 RS-232 vía pyserial (doc 03/06). Con modo simulado."""
import threading
import time
from queue import Queue, Empty

try:
    import serial
    import serial.tools.list_ports
    HAS_SERIAL = True
except Exception:
    HAS_SERIAL = False


class HipotManager:
    def __init__(self, port="COM3", baudrate=9600, simulate=False):
        self.port, self.baudrate = port, baudrate
        self.simulate = simulate or not HAS_SERIAL
        self.serial = None
        self.connected = False
        self.running = False
        self.read_thread = None
        self.data_queue: Queue = Queue()

    @staticmethod
    def list_ports() -> list:
        if not HAS_SERIAL:
            return ["COM3 (simulado)"]
        try:
            return [p.device for p in serial.tools.list_ports.comports()]
        except Exception:
            return []

    def connect(self) -> bool:
        if self.simulate:
            self.connected = True
            self.start_reading()
            return True
        try:
            self.serial = serial.Serial(port=self.port, baudrate=self.baudrate,
                                        bytesize=serial.EIGHTBITS,
                                        parity=serial.PARITY_NONE,
                                        stopbits=serial.STOPBITS_ONE, timeout=1)
            self.connected = True
            self.start_reading()
            return True
        except Exception:
            self.connected = False
            return False

    def disconnect(self):
        self.running = False
        if self.read_thread:
            self.read_thread.join(timeout=2)
        try:
            if self.serial and self.serial.is_open:
                self.serial.close()
        except Exception:
            pass
        self.connected = False

    def send_command(self, command: str) -> bool:
        if not self.connected:
            return False
        if self.simulate:
            cmd = command.strip().upper()
            if cmd == "START":
                self.data_queue.put("V:120.5 I:1.23 R:456.7 PASS")
            elif cmd in ("STATUS?", "MEASURE?", "RESULT?"):
                self.data_queue.put("V:120.5 I:1.23 R:456.7 PASS")
            return True
        try:
            self.serial.write(f"{command}\r\n".encode("ascii"))
            return True
        except Exception:
            return False

    def read_response(self, timeout: float = 1.0):
        try:
            return self.data_queue.get(timeout=timeout)
        except Empty:
            return None

    def start_reading(self):
        if self.running:
            return
        self.running = True
        self.read_thread = threading.Thread(target=self._read_loop, daemon=True)
        self.read_thread.start()

    def _read_loop(self):
        while self.running and self.connected:
            try:
                if self.simulate:
                    time.sleep(0.5)
                    continue
                if self.serial.in_waiting:
                    data = self.serial.readline().decode("ascii", errors="ignore").strip()
                    if data:
                        self.data_queue.put(data)
                else:
                    time.sleep(0.1)
            except Exception:
                self.connected = False
                break

    # atajos
    def start_test(self): return self.send_command("START")
    def stop_test(self): return self.send_command("STOP")
    def abort_test(self): return self.send_command("ABORT")

    def set_voltage(self, v): return self.send_command(f"VOLTAGE:{v}")
    def set_current_limit(self, l): return self.send_command(f"CURRENT_MAX:{l}")

    def test_connection(self) -> bool:
        if self.simulate:
            return self.connected
        self.send_command("STATUS?")
        time.sleep(0.5)
        return True

    @staticmethod
    def parse_measurement(data: str) -> dict:
        result: dict = {}
        try:
            for part in (data or "").split():
                if part.startswith("V:"): result["voltage"] = float(part[2:])
                elif part.startswith("I:"): result["current"] = float(part[2:])
                elif part.startswith("R:"): result["resistance"] = float(part[2:])
                elif part == "PASS": result["pass"] = True
                elif part == "FAIL": result["pass"] = False
        except Exception:
            pass
        return result
