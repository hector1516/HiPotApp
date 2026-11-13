p = 'App/python/routes/api.py'
lines = open(p, encoding='utf-8').read().splitlines()
# Insert logging after line 542 (s.commit()) in set_config
# Find the set_config function
for i, l in enumerate(lines):
    if 'def set_config' in l:
        # find s.commit() after it
        for j in range(i, min(i+20, len(lines))):
            if lines[j].strip() == 's.commit()' and 'return {"status": "saved"}' in lines[j+1]:
                lines.insert(j+1, '        from ..logger import logger')
                lines.insert(j+2, '        logger.info("CONFIG", "save", f"Config guardada: {list(body.keys())}", value_sent=str(body)[:200])')
                break
        break
open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print("done")