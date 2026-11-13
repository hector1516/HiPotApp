p = 'tauri/src/pages/AdminPanel.tsx'
s = open(p, encoding='utf-8').read()
# remove state lines
s = s.replace("  const [autoStart, setAutoStart] = useState(false);\n  const [autoStartMsg, setAutoStartMsg] = useState('');\n", "")
# remove autostart from useEffect
s = s.replace(" api.get('/admin/autostart').then(r => setAutoStart(!!r?.enabled)).catch(() => {});", "")
# remove the auto-start card block
import re
card_start = s.find('      <div className="card" style={{ marginBottom: 10 }}>\n        <h3>Inicio automatico de Windows</h3>')
if card_start != -1:
    card_end = s.find('      </div>\n', card_start) + len('      </div>\n')
    s = s[:card_start] + s[card_end:]
open(p, 'w', encoding='utf-8').write(s)
print("done")