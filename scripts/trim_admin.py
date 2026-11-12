p = 'tauri/src/pages/AdminPanel.tsx'
lines = open(p, encoding='utf-8').read().splitlines()
# find line with 'function ManualPlc'
idx = None
for i, l in enumerate(lines):
    if l.strip() == 'function ManualPlc() {':
        idx = i
        break
if idx is not None:
    # keep everything up to idx-1 (the closing brace of previous function is at idx-1)
    # Actually previous function ends at idx-1 with '}', keep up to idx-1 inclusive
    lines = lines[:idx]
    # remove trailing empty lines
    while lines and lines[-1].strip() == '':
        lines.pop()
    open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
    print('trimmed')
else:
    print('not found')