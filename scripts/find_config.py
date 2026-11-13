lines = open('App/python/routes/api.py', encoding='utf-8').read().splitlines()
for i, l in enumerate(lines):
    if '/admin/config' in l and '@router' in l:
        print(i+1, l)
        for j in range(i, min(i+25, len(lines))):
            print(j+1, lines[j])
        print()
"