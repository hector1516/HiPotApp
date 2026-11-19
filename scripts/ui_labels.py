from pathlib import Path

p = Path('tauri/src/pages/AdminPanel.tsx')
s = p.read_text(encoding='utf-8')
for old, new in [
    ('Test Zebra (Rust)', 'Probar Zebra'),
    ('Sync ahora (Rust)', 'Sincronizar ahora'),
    ('Hipot Start', 'Iniciar Hipot'),
    ('>Stop<', '>Detener<'),
    ('>Estado<', '>Consultar estado<'),
]:
    s = s.replace(old, new)
p.write_text(s, encoding='utf-8')
print('labels updated')
