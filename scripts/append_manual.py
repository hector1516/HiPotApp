p = 'tauri/src/pages/AdminPanel.tsx'
content = open(p, encoding='utf-8').read()
manual_tab = '''

function ManualTab() {
  const [tags, setTags] = useState<any[]>([]);
  const [msg, setMsg] = useState('');
  const [hipot, setHipot] = useState<{ raw: string; parsed: any } | null>(null);
  const [voltage, setVoltage] = useState(120);
  const [writing, setWriting] = useState<number | null>(null);

  const load = () => api.get('/admin/plc-tags').then(setTags).catch(() => setTags([]));
  useEffect(() => { load(); }, []);

  const writeTag = async (tag: any, value: any) => {
    setWriting(tag.id);
    try {
      await api.post('/admin/plc-write', { tag_id: tag.id, value });
      setMsg(`Escrito ${tag.name} = ${value}`);
      load();
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
    finally { setWriting(null); }
  };

  const hipotCmd = async (action: 'START' | 'STOP' | 'STATUS') => {
    try {
      const r = await api.post('/manual/hipot', { action });
      setHipot({ raw: r.response ?? '', parsed: r.parsed ?? {} });
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };

  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>Manual avanzado (sin registro)</h2>
      {msg && <div style={{ marginBottom: 8, fontSize: 12, color: '#475569' }}>{msg}</div>}

      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Salidas del PLC (tags)</h3>
        <div style={{ fontSize: 12, color: '#64748b', marginBottom: 8 }}>
          Escribe valores directamente en los tags del PLC. No se guarda en BD.
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6, maxHeight: 240, overflow: 'auto' }}>
          {tags.map((t: any) => (
            <div key={t.id} style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', minWidth: 0 }}>
              <span style={{ flex: '1 1 120px', fontSize: 13, fontWeight: 600 }}>{t.name}</span>
              <span style={{ fontSize: 11, color: '#64748b', flex: '0 1 80px' }}>{t.data_type}</span>
              <input className="input" style={{ maxWidth: 100, flex: '0 1 100px' }}
                defaultValue={t.value ?? ''} id={`tag-${t.id}`}
                placeholder="valor" />
              <button className="btn primary" style={{ fontSize: 12 }}
                disabled={writing === t.id}
                onClick={() => {
                  const inp = document.getElementById(`tag-${t.id}`) as HTMLInputElement;
                  writeTag(t, inp?.value ?? '');
                }}>
                {writing === t.id ? '...' : 'Escribir'}
              </button>
            </div>
          ))}
          {!tags.length && <div style={{ fontSize: 12, color: '#64748b' }}>No hay tags configurados.</div>}
        </div>
      </div>

      <div className="card" style={{ minWidth: 0 }}>
        <h3>Hipot manual (sin registro)</h3>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', marginBottom: 8 }}>
          <span style={{ fontSize: 13, color: '#475569' }}>Voltaje:</span>
          <select className="input" style={{ maxWidth: 120 }} value={voltage} onChange={e => setVoltage(+e.target.value)}>
            {[120, 208, 220, 240].map(v => <option key={v} value={v}>{v} V</option>)}
          </select>
        </div>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
          <button className="btn ok" onClick={() => hipotCmd('START')}>Hipot Start</button>
          <button className="btn danger" onClick={() => hipotCmd('STOP')}>Stop</button>
          <button className="btn" onClick={() => hipotCmd('STATUS')}>Estado</button>
          {hipot && (
            <span style={{ fontSize: 12, color: '#475569', marginLeft: 8 }}>
              {hipot.raw || '-'} {hipot.parsed?.voltage != null ? `→ ${hipot.parsed.voltage}V` : ''} {hipot.parsed?.pass ? 'PASS' : hipot.parsed?.pass === false ? 'FAIL' : ''}
            </span>
          )}
        </div>
        <div style={{ fontSize: 12, color: '#64748b', marginTop: 6 }}>
          No guarda en BD. Solo visualización y control directo.
        </div>
      </div>
    </div>
  );
}
'''
content += manual_tab
open(p, 'w', encoding='utf-8').write(content)
print("done")