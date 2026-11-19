from pathlib import Path

p = Path('tauri/src/components/HistoryPopup.tsx')
s = p.read_text(encoding='utf-8')
lines = []
for line in s.splitlines():
    if 'Sin datos' in line and 'colSpan' in line:
        lines.append('                  {!rows.length && <tr><td colSpan={9} style={{ textAlign:\'center\', color:\'var(--muted)\', padding: 16 }}>{busy ? \'Consultando…\' : \'Sin datos — elige rango y pulsa Consultar\'}</td></tr>}')
    else:
        lines.append(line)
p.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print('history empty state updated')
