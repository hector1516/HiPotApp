from pathlib import Path

p = Path('tauri/src/components/LogPopup.tsx')
s = p.read_text(encoding='utf-8')
lines = []
for line in s.splitlines():
    if '!logs.length' in line:
        lines.append("                  {!logs.length && <tr><td colSpan={5} style={{ textAlign:'center', color:'var(--muted)' }}>{loading ? 'Cargando…' : 'Sin registros'}</td></tr>}")
    else:
        lines.append(line)
p.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print('log empty state fixed')
