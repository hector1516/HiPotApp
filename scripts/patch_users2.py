p = 'tauri/src/pages/AdminPanel.tsx'
lines = open(p, encoding='utf-8').read().splitlines()
# find the line with auth/register
idx = None
for i, l in enumerate(lines):
    if 'auth/register' in l:
        idx = i
        break
if idx is not None:
    # replace idx, idx+1, idx+2
    lines[idx] = "        await invoke('add_user', { username: payload.username, password: '', role: payload.role });"
    lines[idx+1] = "        setMsg('Usuario creado - gafete listo');"
    del lines[idx+2]  # remove the old setMsg line? Actually idx+2 is setMsg, idx+1 is the if line
    # Wait: idx=auth/register, idx+1=if line, idx+2=setMsg
    # We replaced idx and idx+1, need remove idx+2
    # But we already deleted? Let's just rebuild
    open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
    print("done")
else:
    print("not found")