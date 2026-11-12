p = 'tauri/src/pages/TestScreen.tsx'
lines = open(p, encoding='utf-8').read().splitlines()
for i, l in enumerate(lines):
    if "PLC midiendo" in l or "Hipot midiendo" in l:
        print(i+1, repr(l))