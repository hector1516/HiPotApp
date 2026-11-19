p = 'tauri/src-tauri/src/main.rs'
lines = open(p, encoding='utf-8').read().splitlines()
first = None
second = None
for i, l in enumerate(lines):
    if 'fn sync_pending' in l:
        if first is None:
            first = i
        else:
            second = i
            break
if second is not None:
    end = None
    for i in range(second, len(lines)):
        if lines[i].strip() == 'struct Backend(Mutex<Option<Child>>);':
            end = i
            break
    if end is not None:
        del lines[second-2:end]
        open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
        print("removed")
    else:
        print("struct Backend not found")
else:
    print("no duplicate")