from pathlib import Path
import re

files = [
    'tauri/src/pages/AdminPanel.tsx',
    'tauri/src/components/HistoryPopup.tsx',
    'tauri/src/components/ReprintPopup.tsx',
    'tauri/src/components/LogPopup.tsx',
    'tauri/src/components/ConfigGatePopup.tsx',
    'tauri/src/components/UpdateNotification.tsx',
]
emoji = re.compile(r'[\U0001F300-\U0001FAFF\u2600-\u26FF\u2700-\u27BF\uFE0F]')
for file in files:
    p = Path(file)
    s = p.read_text(encoding='utf-8')
    s2 = emoji.sub('', s)
    if s2 != s:
        p.write_text(s2, encoding='utf-8')
        print('cleaned', file)
