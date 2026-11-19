from pathlib import Path

p = Path('tauri/src/pages/AdminPanel.tsx')
s = p.read_text(encoding='utf-8')
s = s.replace(
    "<h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>",
    '<h2 className="page-title" style={{ marginBottom: 14 }}>',
)
p.write_text(s, encoding='utf-8')
print('admin page titles normalized')
