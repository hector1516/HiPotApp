from pathlib import Path

p = Path('tauri/src/components/ReprintPopup.tsx')
s = p.read_text(encoding='utf-8')
lines = s.splitlines()
out = []
removed = False
for line in lines:
    if "setMsg(r === 'OK'" in line:
        indent = line[:len(line) - len(line.lstrip())]
        out.append(indent + "setMsg(r === 'OK' ? 'Etiqueta enviada a Zebra ZT231' : `Error: ${r}`);")
        removed = True
        continue
    if removed and 'setMsg(' in line and "Etiqueta enviada" in line:
        removed = False
        continue
    removed = False
    out.append(line)
s = '\n'.join(out) + '\n'
s = s.replace('Reimprimir etiqueta', 'Reimprimir etiqueta')
p.write_text(s, encoding='utf-8')
print('reprint patched')
