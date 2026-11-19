from pathlib import Path

p = Path('tauri/src/pages/AdminPanel.tsx')
s = p.read_text(encoding='utf-8')

if "from '../components/Feedback'" not in s:
    s = s.replace(
        "import { LabelEditor, LObj, objsToTemplate, templateToObjs } from '../components/LabelEditor';",
        "import { LabelEditor, LObj, objsToTemplate, templateToObjs } from '../components/LabelEditor';\nimport { askConfirm, notify } from '../components/Feedback';",
    )

s = s.replace('alert(', 'notify(')
s = s.replace('confirm(', 'await askConfirm(')
s = s.replace('await await askConfirm(', 'await askConfirm(')

p.write_text(s, encoding='utf-8')
print('feedback wired')
