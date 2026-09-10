"""Impresora Zebra ZT231 USB (doc 09)."""
import os
import subprocess
import tempfile


class ZebraPrinter:
    def __init__(self, printer_name: str = "Zebra ZT231"):
        self.printer_name = printer_name
        self.connected = False

    def connect(self) -> bool:
        if os.name != "nt":
            self.connected = True  # dev en linux: simular
            return True
        try:
            r = subprocess.run(["wmic", "printer", "get", "name"],
                               capture_output=True, text=True, timeout=10)
            self.connected = self.printer_name in r.stdout
            return self.connected
        except Exception:
            return False

    def print_zpl(self, zpl_data: str) -> bool:
        if not self.connected and not self.connect():
            return False
        try:
            with tempfile.NamedTemporaryFile(mode="w", suffix=".zpl", delete=False) as f:
                f.write(zpl_data)
                path = f.name
            if os.name == "nt":
                subprocess.run(["print", f"/d:{self.printer_name}", path],
                               check=True, shell=True, timeout=30)
            os.unlink(path)
            return True
        except Exception:
            return False

    def print_from_template(self, template_json: str, execution_data: dict) -> bool:
        from .label_generator import canvas_to_zpl
        return self.print_zpl(canvas_to_zpl(template_json, execution_data))

    def test_print(self) -> bool:
        return self.print_zpl(
            "^XA\n^FO50,50^ADN,36,20^FDHello Zebra ZT231^FS\n"
            "^FO50,100^ADN,24,12^FDHiPot Test Print^FS\n^XZ")
