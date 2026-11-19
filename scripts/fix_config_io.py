from pathlib import Path

p = Path('tauri/src-tauri/src/config_io/mod.rs')
s = p.read_text(encoding='utf-8')

for marker in [
    'FROM configurations ORDER BY id',
    'created_at, updated_at FROM label_templates ORDER BY id',
    'FROM users ORDER BY id',
    'FROM plc_tags ORDER BY id',
]:
    idx = s.index(marker)
    end = s.index('    )?;', idx)
    s = s[:end] + '        [],\n    )?;' + s[end + len('    )?;'):]

p.write_text(s, encoding='utf-8')
print('patched')
