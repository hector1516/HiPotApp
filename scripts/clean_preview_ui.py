from pathlib import Path
import re

p = Path('tauri/src/pages/AdminPanel.tsx')
s = p.read_text(encoding='utf-8')
s = re.sub(
    r'<button className="btn" title="Previsualizar OK".*?>.*?</button>\s*'
    r'<button className="btn" title="Previsualizar NO OK".*?>.*?</button>',
    '<button className="btn ok" onClick={() => setPreviewStatus(\'GOOD\')}>Previsualizar OK</button>\n'
    '          <button className="btn danger" onClick={() => setPreviewStatus(\'BAD\')}>Previsualizar MALO</button>',
    s,
    flags=re.S,
)
s = s.replace('🔄 Actualizar Valores', 'Actualizar valores')
p.write_text(s, encoding='utf-8')
print('preview buttons cleaned')
