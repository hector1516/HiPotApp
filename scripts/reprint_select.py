from pathlib import Path

p = Path('tauri/src/components/ReprintPopup.tsx')
s = p.read_text(encoding='utf-8')
lines = []
for line in s.splitlines():
    if '!list.length' in line and '<option' in line:
        lines.append('              {!list.length && <option value="">{loading ? \'Cargando…\' : \'Sin datos\'}</option>}')
    else:
        lines.append(line)
p.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print('reprint select loading updated')
