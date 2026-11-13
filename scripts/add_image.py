p = 'tauri/src/pages/HomeScreen.tsx'
lines = open(p, encoding='utf-8').read().splitlines()
# find the line with '3. Batch'
idx = None
for i, l in enumerate(lines):
    if 'Batch' in l and 'arranca' in l:
        idx = i
        break
if idx is not None:
    # insert after the closing </div> of the Secuencia card (idx+2 is the </div>)
    insert_at = idx + 2
    img_lines = [
        '        <div className="card" style={{ display: \'flex\', justifyContent: \'center\', padding: 8 }}>',
        '          <img src="/IMG_0854.png" alt="HiPotApp" style={{ maxWidth: \'100%\', maxHeight: 120, objectFit: \'contain\', borderRadius: 8 }} />',
        '        </div>',
    ]
    lines[insert_at:insert_at] = img_lines
    open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
    print("done")
else:
    print("not found")