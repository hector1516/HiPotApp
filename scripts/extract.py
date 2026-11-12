lines = open('App/python/routes/api.py', encoding='utf-8').read().splitlines()
for i, l in enumerate(lines):
    if '/executions' in l and '@router' in l:
        print('===', l, '===')
        for j in range(i, min(i+28, len(lines))):
            print(j+1, lines[j])
        print()