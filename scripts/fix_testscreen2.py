p = 'tauri/src/pages/TestScreen.tsx'
lines = open(p, encoding='utf-8').read().splitlines()
# Replace lines 41 and 42 (1-based) -> indices 40, 41
new41 = "        <span className={`pill ${!isPlc ? 'accent' : 'on'}`}>{!isPlc ? 'Hipot midiendo' : 'Hipot completado'}</span>"
new42 = "        <span className={`pill ${isPlc ? 'accent' : ''}`}>{isPlc ? 'Electrica midiendo' : 'Electrica pendiente'}</span>"
lines[40] = new41
lines[41] = new42
open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print("done")