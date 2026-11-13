p = 'tauri/src/pages/AdminPanel.tsx'
s = open(p, encoding='utf-8').read()
s = s.replace('type="number"', 'type="number" inputMode="numeric"')
s = s.replace('type="text"', 'type="text" inputMode="text"')
s = s.replace('type="password"', 'type="password" inputMode="text"')
open(p, 'w', encoding='utf-8').write(s)
print("done")