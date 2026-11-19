from pathlib import Path

p = Path('tauri/src/pages/AdminPanel.tsx')
s = p.read_text(encoding='utf-8')

s = s.replace("notify('Error: ' + (e?.message || e));", "notify('Error: ' + (e?.message || e), 'error');")
s = s.replace("notify(r);", "notify(r, r === 'OK' ? 'ok' : 'error');")
s = s.replace("notify(`SQL Express: ${r}`);", "notify(`SQL Express: ${r}`, 'ok');")
s = s.replace("notify(`Tablas locales verificadas. Creadas: ${r.created ?? 0}`);", "notify(`Tablas locales verificadas. Creadas: ${r.created ?? 0}`, 'ok');")
s = s.replace("notify(`Migraci\u00f3n completada:\\n${JSON.stringify(r, null, 2)}`);", "notify(`Migraci\u00f3n completada:\\n${JSON.stringify(r, null, 2)}`, 'ok');")
s = s.replace("notify('Guardado');", "notify('Guardado', 'ok');")
s = s.replace("notify(r.exists ? 'Tablas creadas/verificadas' : 'No se pudo crear');", "notify(r.exists ? 'Tablas creadas/verificadas' : 'No se pudo crear', r.exists ? 'ok' : 'error');")

p.write_text(s, encoding='utf-8')
print('feedback variants updated')
