import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Gauge } from '../components/Gauge';
import { api } from '../api';

// Modo manual: sin guardar, sin modelo, 1-a-1 por hardware. Conectado a variables de PLC mapeadas.
export function ManualScreen({ onBack }: { onBack: () => void }) {
  const [recipes, setRecipes] = useState<any[]>([]);
  const [rid, setRid] = useState<number | null>(null);
  const [voltage, setVoltage] = useState(120);
  const [active, setActive] = useState<string | null>(null);
  const [values, setValues] = useState<Record<string, number>>({});
  const [mappedTags, setMappedTags] = useState<Record<string, any>>({});
  const [hipot, setHipot] = useState<{ raw: string; parsed: any } | null>(null);
  const [err, setErr] = useState('');

  const loadMapped = () => {
    api.get('/manual/live-mapped')
      .then((data: any) => { if (data) setMappedTags(data); })
      .catch(() => {});
  };

  useEffect(() => {
    api.post('/manual/mode', { mode: 2 }).catch(() => {});
    api.get('/recipes').then(list => {
      setRecipes(list);
      if (list[0]) setRid(list[0].id);
    }).catch(() => {});

    loadMapped();
    const interval = setInterval(loadMapped, 2000);

    return () => {
      clearInterval(interval);
      api.post('/manual/mode', { mode: 1 }).catch(() => {});
      api.post('/manual/abort').catch(() => {});
    };
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
      loadMapped();
    } catch (e: any) { setErr(String(e?.message ?? e)); }
    finally { setActive(null); }
  };

  const hipotCmd = async (action: 'START' | 'STOP' | 'STATUS') => {
    try { const r = await api.post('/manual/hipot', { action }); setHipot({ raw: r.response ?? '', parsed: r.parsed ?? {} }); }
    catch (e: any) { setErr(String(e?.message ?? e)); }
  };

  const voltReading = mappedTags['VOLTAGE_READING'];

  return (
    <div style={{ padding: 12, position: 'relative', zIndex: 1, minWidth: 0 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10, flexWrap: 'wrap', minWidth: 0 }}>
        <button className="btn" onClick={onBack}>← Volver</button>
        <h2 style={{ margin: 0, flex: 1, minWidth: 140, fontSize: 16 }}>Modo Manual — Sincronización en Vivo de Variables PLC</h2>
        <span className="pill accent" style={{ flexShrink: 0 }}>Modo PLC Manual (DB1.DBW110=2)</span>
      </div>

      {/* Barra de Voltaje y Selección de Receta */}
      <div className="card" style={{ display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap', marginBottom: 10, minWidth: 0 }}>
        <select className="input" style={{ maxWidth: 280, flex: '1 1 200px' }} value={rid ?? ''} onChange={e => setRid(+e.target.value)}>
          {recipes.map(r => <option key={r.id} value={r.id}>{r.model} — {r.name}</option>)}
          {!recipes.length && <option value="">Sin recetas — demo</option>}
        </select>
        <select className="input" style={{ maxWidth: 120, flex: '0 1 120px' }} value={voltage} onChange={e => setVoltage(+e.target.value)}>
          {[120, 208, 220, 240].map(v => <option key={v} value={v}>{v} V</option>)}
        </select>

        {/* Monitoreo en vivo de Voltaje Medido */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, background: '#f8fafc', border: '1px solid #cbd5e1', padding: '6px 12px', borderRadius: 8, marginLeft: 'auto' }}>
          <span style={{ fontSize: 12, fontWeight: 700, color: '#475569' }}>Voltaje Medido PLC:</span>
          <span style={{ fontFamily: 'monospace', fontSize: 14, fontWeight: 800, color: '#1e40af' }}>
            {voltReading?.value !== undefined && voltReading?.value !== null ? `${Number(voltReading.value).toFixed(1)} V` : '— V'}
          </span>
          {voltReading?.address && (
            <span className="pill" style={{ fontSize: 10, background: '#e2e8f0' }}>{voltReading.address} ({voltReading.name})</span>
          )}
        </div>
      </div>

      {err && <div className="toast-err">{err}</div>}

      {/* Grid de Componentes con Enlace a Variables de PLC */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 10, alignItems: 'start' }}>
        {items.map(c => {
          const resTagKey = `RES_${c.name}`;
          const tagInfo = mappedTags[resTagKey];
          const liveValFromTag = tagInfo?.value;
          const v = liveValFromTag !== undefined && liveValFromTag !== null ? Number(liveValFromTag) : values[c.name];
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
                  {measuring ? 'Midiendo' : `${c.min}–${c.max}`}
                </span>
              </div>

              {/* Dirección Física de PLC asignada en /admin */}
              <div style={{ fontSize: 11, color: tagInfo ? '#1e40af' : '#64748b', marginTop: 4, display: 'flex', justifyContent: 'space-between' }}>
                <span>PLC Tag:</span>
                <span style={{ fontWeight: 700, fontFamily: 'monospace' }}>{tagInfo ? `${tagInfo.address} (${tagInfo.name})` : 'Sin mapear'}</span>
              </div>

              <div style={{ display: 'flex', justifyContent: 'center', padding: '6px 0' }}>
                <Gauge label={c.name} value={v ?? 0} min={c.min} max={c.max} size={124} />
              </div>

              <button className={`btn ${measuring ? '' : 'primary'}`} style={{ width: '100%' }}
                disabled={busy || !c.enabled} onClick={() => testOne(c.name)}>
                {measuring ? 'Midiendo…' : 'Probar Componente'}
              </button>
            </div>
          );
        })}
      </div>

      {/* Hipot Manual */}
      <div className="card" style={{ marginTop: 10 }}>
        <h3 style={{ margin: '0 0 8px 0', fontSize: 14 }}>Prueba de Rigidez Dieléctrica (Hipot) — Manual</h3>
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
      </div>
    </div>
  );
}
