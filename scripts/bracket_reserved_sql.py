from pathlib import Path

for path in ['tauri/src-tauri/src/db/repo.rs', 'tauri/src-tauri/src/config_io/mod.rs']:
    p = Path(path)
    s = p.read_text(encoding='utf-8')
    replacements = [
        ('SELECT id, name, model, type, product', 'SELECT id, name, model, [type], product'),
        ('SELECT name, type, product', 'SELECT name, [type], product'),
        ('(name, model, type, product, voltage_levels', '(name, model, [type], product, voltage_levels'),
        ('model = {}, type = {}', 'model = {}, [type] = {}'),
        ('SELECT id, name, db_number, offset, data_type', 'SELECT id, name, db_number, [offset], data_type'),
        ('(name, db_number, offset, data_type, bit)', '(name, db_number, [offset], data_type, bit)'),
    ]
    for old, new in replacements:
        s = s.replace(old, new)
    p.write_text(s, encoding='utf-8')
print('reserved words bracketed')
