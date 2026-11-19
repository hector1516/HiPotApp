p = 'tauri/src/components/ReprintPopup.tsx'
s = open(p, encoding='utf-8').read()
old = """    try {
      await api.post(`/executions/${sel}/reprint`);
      setMsg('✓ Etiqueta enviada a Zebra ZT231');
    } catch (e: any) { setMsg(`✗ ${e?.message ?? e}`); }"""
new = """    try {
      const cfg = await invoke<any>('get_config').catch(() => ({}));
      const ip = cfg?.printer_ip || '';
      const port = parseInt(cfg?.printer_port || '9100', 10);
      const name = cfg?.printer_name || 'Zebra ZT231';
      const r = await invoke<string>('reprint_zebra', { eid: Number(sel), printerIp: ip, printerPort: port, printerName: name });
      setMsg(r === 'OK' ? '✓ Etiqueta enviada a Zebra ZT231' : `✗ ${r}`);
    } catch (e: any) { setMsg(`✗ ${e?.message ?? e}`); }"""
# The actual text may have different chars, so do line-based replacement
import re
lines = s.splitlines()
for i, l in enumerate(lines):
    if "api.post(`/executions/${sel}/reprint`)" in l:
        lines[i] = "      const cfg = await invoke<any>('get_config').catch(() => ({}));"
        lines.insert(i+1, "      const ip = cfg?.printer_ip || '';")
        lines.insert(i+2, "      const port = parseInt(cfg?.printer_port || '9100', 10);")
        lines.insert(i+3, "      const name = cfg?.printer_name || 'Zebra ZT231';")
        lines.insert(i+4, "      const r = await invoke<string>('reprint_zebra', { eid: Number(sel), printerIp: ip, printerPort: port, printerName: name });")
        lines.insert(i+5, "      setMsg(r === 'OK' ? '✓ Etiqueta enviada a Zebra ZT231' : `✗ ${r}`);")
        # remove old setMsg line
        # find and remove the line with setMsg('✓ Etiqueta enviada
        for j in range(i+6, min(i+10, len(lines))):
            if "setMsg('✓ Etiqueta enviada a Zebra ZT231')" in lines[j] or "setMsg('✓" in lines[j]:
                del lines[j]
                break
        break
open(p, 'w', encoding='utf-8').write('\n'.join(lines) + '\n')
print("done")