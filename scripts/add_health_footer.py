p = 'tauri/src/App.tsx'
lines = open(p, encoding='utf-8').read().splitlines()
# find the line with Offline-first
idx = None
for i, l in enumerate(lines):
    if 'Offline-first' in l:
        idx = i
        break
if idx is not None:
    insert = [
        "        <span style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: 12 }}>",
        "          <span style={{ width: 8, height: 8, borderRadius: '50%', background: health ? '#166534' : '#991b1b' }} />",
        "          {health ? 'Backend OK' : 'Sin conexion'}",
        "        </span>",
    ]
    lines[idx:idx] = insert
    open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
    print("done")
else:
    print("not found")