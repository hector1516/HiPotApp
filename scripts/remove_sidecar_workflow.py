from pathlib import Path

p = Path('.github/workflows/build.yml')
s = p.read_text(encoding='utf-16')
start = s.index('      - name: Setup Python')
end = s.index('      - name: Setup Node.js')
s = s[:start] + s[end:]
p.write_text(s, encoding='utf-8')
print('workflow updated')
