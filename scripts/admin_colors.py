from pathlib import Path

p = Path('tauri/src/pages/AdminPanel.tsx')
s = p.read_text(encoding='utf-8')
replacements = {
    '#64748b': 'var(--muted)',
    '#475569': 'var(--text-2)',
    '#166534': 'var(--ok)',
    '#991b1b': 'var(--bad)',
    '#eef2f6': 'var(--bg)',
    '#d8e0ea': 'var(--line)',
    '#f1f5f9': 'var(--surface-2)',
    '#b7790f': 'var(--accent)',
    '#1e293b': 'var(--text)',
    '#334155': 'var(--text-2)',
}
for old, new in replacements.items():
    s = s.replace(old, new)
p.write_text(s, encoding='utf-8')
print('admin colors tokenized')
