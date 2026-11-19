from pathlib import Path
import re

p = Path('tauri/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

for fn in ('read_plc_tag', 'write_plc_tag'):
    pattern = r'#\[tauri::command\]\nfn ' + fn + r'\(.*?\n\}\n\n'
    s2, n = re.subn(pattern, '', s, count=1, flags=re.S)
    print(fn, 'removed' if n else 'not found')
    s = s2

p.write_text(s, encoding='utf-8')
