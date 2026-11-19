from pathlib import Path

p = Path('tauri/src/components/ConfigGatePopup.tsx')
s = p.read_text(encoding='utf-8')

s = s.replace(
    "export function ConfigGatePopup({ open, onClose, onAdmin }: { open: boolean; onClose: () => void; onAdmin: (batch:string)=>void }) {",
    "export function ConfigGatePopup({ open, onClose, onAdmin, target = 'config' }: { open: boolean; onClose: () => void; onAdmin: (batch:string)=>void; target?: 'config' | 'manual' }) {",
)

lines = []
for line in s.splitlines():
    if '<h3' in line and 'escanea tu gafete' in line:
        lines.append("            <h3 style={{ margin: '8px 0 4px 0' }}>{target === 'manual' ? 'Modo manual · escanea tu gafete' : 'Configuración · escanea tu gafete'}</h3>")
    elif 'Solo <b>admin</b> entra' in line:
        lines.append("            <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 10 }}>{target === 'manual' ? 'Solo admin puede entrar al modo manual. Si es operador, se denegará.' : 'Solo admin entra a configuración. Si es operador, se denegará.'}</div>")
    else:
        lines.append(line)

s = '\n'.join(lines) + '\n'
s = s.replace("setMsg(' Solo admin", "setMsg('Solo admin")
s = s.replace('` Gafete no encontrado', '`Gafete no encontrado')

p.write_text(s, encoding='utf-8')
print('gate target added')
