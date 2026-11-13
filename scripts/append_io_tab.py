p = 'tauri/src/pages/AdminPanel.tsx'
content = open(p, encoding='utf-8').read()
io_tab = '''

function PlcIoTab() {
  const [status, setStatus] = useState<any>(null);
  const [err, setErr] = useState('');

  const load = () => {
    api.get('/admin/plc-io-status').then(setStatus).catch(e => setErr(String(e?.message ?? e)));
  };

  useEffect(() => {
    load();
    const id = setInterval(load, 2000);
    return () => clearInterval(id);
  }, []);

  const toggle = async (q: string, val: boolean) => {
    try {
      await api.post('/admin/plc-io-write', { q, value: val });
      load();
    } catch (e: any) { setErr(String(e?.message ?? e)); }
  };

  const Q_LABELS: Record<string, string> = {
    "Q0.0": "KM2 (120VAC)", "Q0.1": "KM1 (220VAC)", "Q0.2": "K2 (SSR 0-20A)",
    "Q0.3": "K8 (Relay 0-1A)", "Q0.4": "K9 (Relay 0-10A)", "Q0.5": "K3 (FAN)",
    "Q0.6": "K4 (LIGHT)", "Q0.7": "K5 (FRAME)", "Q1.0": "K6 (DOOR)",
    "Q1.1": "K7 (DRAIN)", "Q2.0": "K1 (DEFROST)", "Q2.1": "Libre", "Q2.2": "Libre",
  };
  const I_LABELS: Record<string, string> = {
    "I0.0": "Paro de Emergencia", "I0.1": "NA KM1 (220VAC)", "I0.2": "NA KM2 (110VAC)",
  };
  const IA_LABELS: Record<string, string> = {
    "IA1": "Amplificador 0-1A", "IA2": "Amplificador 0-10A",
    "IA3": "Amplificador 0-20A", "IA4": "Amplificador 0-220VAC",
  };

  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>Monitor PLC (IO) - Solo lectura</h2>
      {err && <div className="toast-err">{err}</div>}
      <div style={{ fontSize: 12, color: '#64748b', marginBottom: 10 }}>
        Actualizacion automatica cada 2 segundos. Los indicadores muestran el estado en tiempo real.
      </div>

      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Salidas digitales (Q)</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 6 }}>
          {Object.entries(status?.outputs ?? {}).map(([q, v]) => (
            <div key={q} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 8px', background: '#fff', borderRadius: 6, border: '1px solid #d8e0ea' }}>
              <span style={{ width: 10, height: 10, borderRadius: '50%', background: v ? '#166534' : '#94a3b8', flexShrink: 0 }} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <b style={{ fontSize: 12 }}>{q}</b>
                <div style={{ fontSize: 10, color: '#64748b', overflow: 'hidden', textOverflow: 'ellipsis' }}>{Q_LABELS[q] || ''}</div>
              </div>
              <span className={`pill ${v ? 'on' : ''}`} style={{ fontSize: 10 }}>{v ? 'ON' : 'OFF'}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Entradas digitales (I)</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 6 }}>
          {Object.entries(status?.inputs ?? {}).map(([i, v]) => (
            <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 8px', background: '#fff', borderRadius: 6, border: '1px solid #d8e0ea' }}>
              <span style={{ width: 10, height: 10, borderRadius: '50%', background: v ? '#b7790f' : '#94a3b8', flexShrink: 0 }} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <b style={{ fontSize: 12 }}>{i}</b>
                <div style={{ fontSize: 10, color: '#64748b', overflow: 'hidden', textOverflow: 'ellipsis' }}>{I_LABELS[i] || ''}</div>
              </div>
              <span className={`pill ${v ? 'accent' : ''}`} style={{ fontSize: 10 }}>{v ? 'ON' : 'OFF'}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="card" style={{ minWidth: 0 }}>
        <h3>Entradas analogicas (IA)</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 6 }}>
          {Object.entries(status?.analog ?? {}).map(([ia, v]) => (
            <div key={ia} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 8px', background: '#fff', borderRadius: 6, border: '1px solid #d8e0ea' }}>
              <div style={{ flex: 1, minWidth: 0 }}>
                <b style={{ fontSize: 12 }}>{ia}</b>
                <div style={{ fontSize: 10, color: '#64748b', overflow: 'hidden', textOverflow: 'ellipsis' }}>{IA_LABELS[ia] || ''}</div>
              </div>
              <span className="pill" style={{ fontSize: 11, fontVariantNumeric: 'tabular-nums' }}>{Number(v).toFixed(2)}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
'''
content += io_tab
open(p, 'w', encoding='utf-8').write(content)
print("done")