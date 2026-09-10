"""Generador ZPL desde plantilla del editor visual (doc 09 §canvas_to_zpl)."""
import json

MM_TO_DOTS = 203 / 25.4  # 8.0


def replace_placeholder(text: str, data: dict) -> str:
    reps = {
        "{modelo}": str(data.get("model", "")),
        "{serie}": str(data.get("serial_number", "")),
        "{batch}": str(data.get("batch_number", "")),
        "{operador}": str(data.get("operator_name", "")),
        "{fecha}": str(data.get("date", "")),
        "{hora}": str(data.get("time", "")),
        "{duracion}": str(data.get("duration", "")),
        "{estado}": str(data.get("final_status", "")),
        "{voltaje}": str(data.get("voltage", "")),
        "{fan}": str(data.get("fan_value", "")),
        "{door}": str(data.get("door_value", "")),
        "{frame}": str(data.get("frame_value", "")),
        "{drain}": str(data.get("drain_value", "")),
        "{defrost}": str(data.get("defrost_value", "")),
        "{light}": str(data.get("light_value", "")),
        "{total}": str(data.get("total_value", "")),
        "{plc_status}": str(data.get("plc_pass", "")),
        "{hipot_status}": str(data.get("hipot_pass", "")),
    }
    for k, v in reps.items():
        text = text.replace(k, v)
    return text


def canvas_to_zpl(template_json: str, execution_data: dict) -> str:
    t = json.loads(template_json) if isinstance(template_json, str) else template_json
    w = int(t["label"]["width_mm"] * MM_TO_DOTS)
    h = int(t["label"]["height_mm"] * MM_TO_DOTS)
    zpl = f"^XA\n^PW{w}\n^LL{h}\n\n"
    for obj in t.get("objects", []):
        x = int(obj.get("x", 0) * MM_TO_DOTS / 10)
        y = int(obj.get("y", 0) * MM_TO_DOTS / 10)
        typ = obj.get("type")
        if typ == "text":
            txt = replace_placeholder(obj.get("content", ""), execution_data)
            size = obj.get("font_size", 24)
            zpl += f"^FO{x},{y}^A0N,{size},{size}^FD{txt}^FS\n"
        elif typ == "barcode":
            data = replace_placeholder(obj.get("data_source", ""), execution_data)
            hh = obj.get("height", 80)
            zpl += f"^FO{x},{y}^BY3^BCN,{hh},Y,N,N^FD{data}^FS\n"
        elif typ == "qrcode":
            data = replace_placeholder(obj.get("data_source", ""), execution_data)
            zpl += f"^FO{x},{y}^BQ2,3,N,N^FD{data}^FS\n"
        elif typ == "line":
            x2 = int(obj.get("x2", obj.get("x", 0) + 100) * MM_TO_DOTS / 10)
            th = obj.get("thickness", 2)
            zpl += f"^FO{x},{y}^GB{max(x2 - x, 1)},{th},{th}^FS\n"
        elif typ == "rect":
            w = int(obj.get("w", 100) * MM_TO_DOTS / 10)
            h = int(obj.get("h", 40) * MM_TO_DOTS / 10)
            th = obj.get("thickness", 2)
            zpl += f"^FO{x},{y}^GB{max(w, 1)},{max(h, 1)},{th}^FS\n"
    return zpl + "\n^XZ"


DEFAULT_TEMPLATE_JSON = json.dumps({
    "label": {"width_mm": 100, "height_mm": 60, "bg_color": "#FFFFFF"},
    "objects": [
        {"id": "t1", "type": "text", "x": 50, "y": 20, "content": "HIPOT - RESULTADO",
         "font_size": 28, "bold": True},
        {"id": "t2", "type": "text", "x": 50, "y": 60, "content": "Modelo: {modelo}", "font_size": 18},
        {"id": "t3", "type": "text", "x": 50, "y": 85, "content": "Serie: {serie}", "font_size": 18},
        {"id": "t4", "type": "text", "x": 50, "y": 115, "content": "Estado: {estado}",
         "font_size": 20, "bold": True},
        {"id": "b1", "type": "barcode", "x": 50, "y": 300,
         "data_source": "{modelo}-{serie}-{batch}", "height": 80},
        {"id": "q1", "type": "qrcode", "x": 400, "y": 280, "size": 100,
         "data_source": "{modelo}|{serie}|{batch}|{estado}"},
    ],
})
