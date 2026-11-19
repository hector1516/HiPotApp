p = 'tauri/src/pages/AdminPanel.tsx'
lines = open(p, encoding='utf-8').read().splitlines()
idx = None
for i, l in enumerate(lines):
    if 'auth/register' in l:
        idx = i
        break
if idx is not None:
    new_lines = [
        "        await invoke('add_user', { username: payload.username, password: '', role: payload.role });",
        "        setMsg('Usuario creado - gafete listo');",
    ]
    lines[idx:idx+3] = new_lines
    open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
    print("done")
else:
    print("not found")