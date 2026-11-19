from pathlib import Path

p = Path('tauri/src/App.tsx')
s = p.read_text(encoding='utf-8')
pairs = [
    ('setStep(2)', "setStep(stepNumber('serial'))"),
    ('setStep(3)', "setStep(stepNumber('batch'))"),
    ('setStep(4)', "setStep(stepNumber('plc'))"),
    ('setStep(5)', "setStep(stepNumber('hipot'))"),
    ('setStep(6)', "setStep(stepNumber('result'))"),
    ('setStep(1)', "setStep(stepNumber('model'))"),
    ('PASO {Math.min(step, 6)}/6', 'PASO {Math.min(step, AUTO_STEPS.length)}/{AUTO_STEPS.length}'),
]
for old, new in pairs:
    s = s.replace(old, new)
p.write_text(s, encoding='utf-8')
print('step numbers centralized')
