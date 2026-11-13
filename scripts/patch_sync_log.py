p = 'App/python/routes/api.py'
lines = open(p, encoding='utf-8').read().splitlines()
for i, l in enumerate(lines):
    if 'def sync_now' in l:
        for j in range(i, min(i+15, len(lines))):
            if 'return {"status": "ok", "synced": n}' in lines[j]:
                lines.insert(j, '        from ..logger import logger')
                lines.insert(j+1, '        logger.info("SYNC", "sync_now", f"Sincronizados {n} registros")')
                break
        break
open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print("done")