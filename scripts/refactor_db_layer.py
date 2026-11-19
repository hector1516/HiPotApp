from pathlib import Path

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

# 1. Todas las conexiones pasan por la capa db (esquema garantizado).
s = s.replace('rusqlite::Connection::open(db_path())', 'db::open()')

# 2. Eliminar DDL disperso: ya lo hace db::schema::ensure_schema().
lines = []
for line in s.splitlines():
    if line.strip().startswith('let _ = conn.execute("CREATE TABLE IF NOT EXISTS'):
        continue
    lines.append(line)
s = '\n'.join(lines) + '\n'

# 3. La plantilla por defecto vive en db::seed.
lines = []
for line in s.splitlines():
    if line.strip().startswith('const DEFAULT_TEMPLATE_JSON'):
        continue
    lines.append(line)
s = '\n'.join(lines) + '\n'
s = s.replace('DEFAULT_TEMPLATE_JSON', 'db::seed::DEFAULT_TEMPLATE_JSON')

p.write_text(s, encoding='utf-8')
print('patched')
