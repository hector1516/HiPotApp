import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Gauge } from '../components/Gauge';
import { api } from '../api';

// Modo manual: sin guardar, sin modelo, 1-a-1 por hardware. Todo en una pantalla.
export function ManualScreen({ onBack }: { onBack: () => void }) {
  const [recipes, setRecipes] = useState<any[]>([]);
  const [rid, setRid] = useState<number | null>(null);
  const [voltage, setVoltage] = useState(120);
  const [active, setActive] = useState<string | null>(null);
  const [values, setValues] = useState<Record<string, number>>({});
  const [hipot, setHipot] = useState<{ raw: string; parsed: any } | null>(null);
  const [err, setErr] = useState('');

  useEffect(() => {
    api.post('/manual/mode', { mode: 2 }).catch(() => {});
    api.get('/recipes').then(list => {
      setRecipes(list);
      if (list[0]) setRid(list[0].id);
    }).catch(() => {});
    return () => { api.post('/manual/mode', { mode: 1 }).catch(() => {}); api.post('/manual/abort').catch(() => {}); };
  }, []);

  const recipe = recipes.find(r => r.id === rid) ?? null;
  const [detail, setDetail] = useState<any>(null);
  useEffect(() => {
    if (!recipe) return;
    api.recipeByModel(recipe.model).then(setDetail).catch(() => setDetail(null));
  }, [recipe?.model]);

  const items: { name: string; min: number; max: number; enabled: boolean }[] =
    (detail?.components ?? [
      { name: 'FAN', min: 0.3, max: 1.5, enabled: true },
      { name: 'DOOR', min: 1.5, max: 3.0, enabled: true },
      { name: 'FRAME', min: 0.5, max: 1.2, enabled: true },
      { name: 'DRAIN', min: 1.5, max: 2.8, enabled: true },
      { name: 'DEFROST', min: 0.4, max: 1.0, enabled: true },
      { name: 'LIGHT', min: 0.1, max: 0.6, enabled: true },
    ]).map((c: any) => ({ name: c.name, min: c.min, max: c.max, enabled: c.enabled !== false }));

  const testOne = async (name: string) => {
    if (active) return;
    setActive(name); setErr('');
    try {
      const r = await api.post('/manual/plc-test', { component: name, voltage });
      setValues(v => ({ ...v, [name]: r.value }));
      for (let i = 0; i < 5; i++) {
        await new Promise(res => setTimeout(res, 350));
        try {
          const s = await api.post('/manual/plc-test', { component: name, voltage });
          setValues(v => ({ ...v, [name]: s.value }));
        } catch {}
      }
    } catch (e: any) { setErr(String(e?.message ?? e)); }
    finally { setActive(null); }
  };

  const hipotCmd = async (action: 'START' | 'STOP' | 'STATUS') => {
    try { const r = await api.post('/manual/hipot', { action }); setHipot({ raw: r.response ?? '', parsed: r.parsed ?? {} }); }
    catch (e: any) { setErr(String(e?.message ?? e)); }
  };

  return (
    <div style={{ padding: 12, position: 'relative', zIndex: 1, minWidth: 0 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10, flexWrap: 'wrap', minWidth: 0 }}>
        <button className="btn" onClick={onBack}>← Volver</button>
        <h2 style={{ margin: 0, flex: 1, minWidth: 140, fontSize: 16 }}>Modo manual — un componente a la vez</h2>
        <span className="pill accent" style={{ flexShrink: 0 }}>Manual · No guarda · DB1.DBW110=2</span>
      </div>

      <div className="card" style={{ display: 'flex', gap: 10, alignItems: 'center', flexWrap: 'wrap', marginBottom: 10, minWidth: 0 }}>
        <select className="input" style={{ maxWidth: 280, flex: '1 1 220px' }} value={rid ?? ''} onChange={e => setRid(+e.target.value)}>
          {recipes.map(r => <option key={r.id} value={r.id}>{r.model} — {r.name}</option>)}
          {!recipes.length && <option value="">Sin recetas — demo</option>}
        </select>
        <select className="input" style={{ maxWidth: 130, flex: '0 1 130px' }} value={voltage} onChange={e => setVoltage(+e.target.value)}>
          {[120, 208, 220, 240].map(v => <option key={v} value={v}>{v} V</option>)}
        </select>
        <span style={{ fontSize: 12, color: '#64748b', marginLeft: 'auto' }}>Solo 1 botón activo a la vez por hardware.</span>
      </div>

      {err && <div className="toast-err">{err}</div>}

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))', gap: 10, alignItems: 'start' }}>
        {items.map(c => {
          const v = values[c.name];
          const measuring = active === c.name;
          const busy = !!active;
          return (
            <div key={c.name}
              style={{
                background: measuring ? '#fdf6e3' : '#fff',
                borderRadius: 10, padding: 10,
                border: `1px solid ${measuring ? '#f0d9a0' : '#d8e0ea'}`,
                opacity: c.enabled ? 1 : 0.5, minWidth: 0, overflow: 'hidden',
              }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, minWidth: 0 }}>
                <b style={{ fontSize: 13, minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis' }}>{c.name}</b>
                <span className={`pill ${measuring ? 'accent' : ''}`} style={{ marginLeft: 'auto', fontSize: 11, flexShrink: 0 }}>
                  {measuring ? 'Midiendo' : `${c.min}–${c.max} A`}
                </span>
              </div>
              <div style={{ display: 'flex', justifyContent: 'center', padding: '6px 0' }}>
                <Gauge label={c.name} value={v ?? 0} min={c.min} max={c.max} size={124} />
              </div>
              <button className={`btn ${measuring ? '' : 'primary'}`} style={{ width: '100%' }}
                disabled={busy || !c.enabled} onClick={() => testOne(c.name)}>
                {measuring ? 'Midiendo…' : 'Probar'}
              </button>
            </div>
          );
        })}
      </div>

      <div className="card" style={{ marginTop: 10 }}>
        <h3 style={{ margin: '0 0 8px 0', fontSize: 14 }}>Hipot — manual</h3>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center', minWidth: 0 }}>
          <button className="btn ok" disabled={!!active} onClick={() => hipotCmd('START')}>Hipot Start</button>
          <button className="btn danger" onClick={() => hipotCmd('STOP')}>Stop</button>
          <button className="btn" onClick={() => hipotCmd('STATUS')}>Estado</button>
          {hipot && (
            <span className={`pill ${hipot.parsed?.pass === false ? 'off' : hipot.parsed?.pass ? 'on' : ''}`} style={{ maxWidth: 520, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {hipot.raw || '—'} {hipot.parsed?.voltage != null ? `· ${hipot.parsed.voltage}V` : ''} {hipot.parsed?.pass ? 'PASS' : hipot.parsed?.pass === false ? 'FAIL' : ''}
            </span>
          )}
        </div>
        <div style={{ fontSize: 12, color: '#64748b', marginTop: 6 }}>No pide modelo/serie/batch. No guarda en BD. Solo visualización.</div>
      </div>
    </div>
  );
}
