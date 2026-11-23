import { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { Gauge } from '../components/Gauge';
import { SplitLayout } from '../components/SplitLayout';
import { PageHeader } from '../components/PageHeader';
import { SafetyNote } from '../components/SafetyNote';
import { LiveHardwarePanel } from '../components/LiveHardwarePanel';
import { useHipot } from '../hooks/useHipot';

type PendingAction = 'START' | 'STOP' | null;

export function ManualScreen({ onBack, hardwareReady = true }: { onBack: () => void; hardwareReady?: boolean }) {
  const [recipes, setRecipes] = useState<any[]>([]);
  const [rid, setRid] = useState<number | null>(null);
  const [voltage, setVoltage] = useState(120);
  const [active, setActive] = useState<string | null>(null);
  const [values, setValues] = useState<Record<string, number>>({});
  const [hipot, setHipot] = useState<{ raw: string; parsed: any } | null>(null);
  const [err, setErr] = useState('');
  const [pending, setPending] = useState<PendingAction>(null);
  const hipotStatus = useHipot(hardwareReady);

  useEffect(() => {
    invoke('plc_set_mode', { mode: 2 }).catch(() => {});
    invoke<any[]>('list_recipes').then(list => {
      setRecipes(Array.isArray(list) ? list : []);
      if (list?.[0]) setRid(list[0].id);
    }).catch(() => {});
    return () => {
      invoke('plc_set_mode', { mode: 1 }).catch(() => {});
      invoke('plc_abort').catch(() => {});
      invoke('hipot_test', { action: 'STOP' }).catch(() => {});
    };
  }, []);

  const recipe = recipes.find(r => r.id === rid) ?? null;
  const [detail, setDetail] = useState<any>(null);

  useEffect(() => {
    if (!recipe) { setDetail(null); return; }
    invoke<any>('get_recipe_by_model', { model: recipe.model }).then(setDetail).catch(() => setDetail(null));
  }, [recipe?.model]);

  const items: { name: string; min: number; max: number; enabled: boolean }[] =
    (detail?.components ?? []).map((c: any) => ({
      name: c.name, min: c.min, max: c.max, enabled: c.enabled !== false,
    }));

  const testOne = async (name: string) => {
    if (active) return;
    setActive(name); setErr('');
    try {
      // Dispara UNA vez. Después solo se consultan resultados.
      const first = await invoke<any>('plc_test_component', { component: name, voltage });
      setValues(v => ({ ...v, [name]: Number(first?.value ?? 0) }));
      for (let i = 0; i < 6; i++) {
        await new Promise(res => setTimeout(res, 400));
        try {
          const all = await invoke<any>('plc_read_results');
          const v = all?.[name] ?? all?.[name.toUpperCase()];
          if (typeof v === 'number') setValues(prev => ({ ...prev, [name]: v }));
        } catch {}
      }
    } catch (e: any) {
      setErr(String(e?.message ?? e));
    } finally {
      setActive(null);
    }
  };

  const hipotCmd = async (action: 'START' | 'STOP' | 'STATUS') => {
    try {
      const r = await invoke<any>('hipot_test', { action });
      if (r?.error) throw new Error(r.error);
      setHipot({ raw: r.response ?? '', parsed: r.parsed ?? {} });
    } catch (e: any) {
      setErr(String(e?.message ?? e));
    }
  };

  const runPending = async () => {
    const action = pending;
    setPending(null);
    if (!action) return;
    if (action === 'START') {
      const v = Number(detail?.hipot_voltage ?? 0);
      const i = Number(detail?.hipot_current_max ?? 0);
      const t = Number(detail?.hipot_time ?? 0);
      if (v <= 0 || i <= 0 || t <= 0) {
        setErr('La receta no tiene voltaje, corriente y tiempo Hipot configurados.');
        return;
      }
      try {
        const prep = await invoke<any>('hipot_prepare', { voltage: v, currentMax: i, time: t });
        if (prep?.error) throw new Error(prep.error);
      } catch (e: any) {
        setErr(String(e?.message ?? e));
        return;
      }
    }
    await hipotCmd(action);
  };

  const aside = (
    <>
      <div>
        <div className="rail-eyebrow">Modo manual</div>
        <h2 className="rail-title">Prueba 1 a 1</h2>
        <p className="rail-sub">Sin modelo, serie ni batch. No guarda en base de datos. Solo control y visualización.</p>
      </div>

      <div className="rail-divider" />

      <div className="field">
        <label>Receta</label>
        <select className="input" value={rid ?? ''} onChange={e => setRid(+e.target.value)}>
          {recipes.map(r => <option key={r.id} value={r.id}>{r.model} — {r.name}</option>)}
          {!recipes.length && <option value="">Sin recetas</option>}
        </select>
      </div>

      <div className="field">
        <label>Voltaje de prueba</label>
        <select className="input" value={voltage} onChange={e => setVoltage(+e.target.value)}>
          {[120, 208, 220, 240].map(v => <option key={v} value={v}>{v} V</option>)}
        </select>
      </div>

      <div className="rail-kpi">
        <span className="label">Componentes</span>
        <span className="value">{items.length}</span>
      </div>
      <div className="rail-kpi">
        <span className="label">Hipot</span>
        <span className="value">{hipotStatus.connected ? 'Conectado' : 'Sin conexión'}</span>
      </div>

      <div className="rail-divider" />

      <div className="stack" style={{ gap: 8 }}>
        <div className="rail-eyebrow">Hipot manual</div>
        <div style={{ fontSize: 11, color: 'var(--on-rail-muted)' }}>
          Receta: {detail?.hipot_voltage ?? '—'} V · {detail?.hipot_current_max ?? '—'} mA · {detail?.hipot_time ?? '—'} s
        </div>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn ok" style={{ flex: 1 }} onClick={() => setPending('START')}>Iniciar</button>
          <button className="btn danger" style={{ flex: 1 }} onClick={() => setPending('STOP')}>Detener</button>
        </div>
        <button className="btn" style={{ width: '100%' }} onClick={() => hipotCmd('STATUS')}>Consultar estado</button>
        {hipot && (
          <div className="rail-status" style={{ alignItems: 'flex-start', fontSize: 11, color: 'var(--on-rail-muted)' }}>
            <span className={`dot ${hipot.parsed?.pass === false ? 'bad' : hipot.parsed?.pass ? 'ok' : 'warn'}`} />
            <span className="mono" style={{ wordBreak: 'break-all' }}>{hipot.raw || '—'}</span>
          </div>
        )}
        <SafetyNote compact />
      </div>

      <div className="rail-divider" />

      <div className="image-card">
        <img src="/IMG_0854.png" alt="Estación de pruebas HiPot" style={{ maxHeight: 110 }} />
      </div>

      <div className="rail-actions">
        <button className="btn" style={{ width: '100%' }} onClick={onBack}>← Volver al inicio</button>
        <div className="rail-status" style={{ fontSize: 11, color: 'var(--on-rail-muted)' }}>
          <span>DB1.DBW110 = 2 (manual)</span>
        </div>
      </div>
    </>
  );

  return (
    <>
      <SplitLayout aside={aside}>
        <PageHeader
          eyebrow="Hardware · Manual"
          title="Prueba componente por componente"
          subtitle="Solo un componente activo a la vez. Los resultados no se registran."
          actions={recipe ? <span className="pill accent">{recipe.model}</span> : null}
        />

        {err && <div className="alert">{err}</div>}

        <LiveHardwarePanel hardwareReady={hardwareReady} hipotStatus={hipotStatus.status} />

        {!recipe ? (
          <div className="card" style={{ textAlign: 'center', padding: 40 }}>
            <div className="page-title" style={{ fontSize: 18 }}>Seleccione una receta</div>
            <p className="page-sub">No hay recetas cargadas. Cree una receta en Configuración → Recetas para probar componentes.</p>
          </div>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 12 }}>
            {items.map(c => {
              const v = values[c.name];
              const measuring = active === c.name;
              const busy = !!active;
              return (
                <motion.div
                  key={c.name}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.25 }}
                  className="card"
                  style={{
                    padding: 12,
                    opacity: c.enabled ? 1 : 0.5,
                    borderColor: measuring ? 'var(--accent)' : undefined,
                    background: measuring ? 'var(--accent-soft)' : undefined,
                  }}
                >
                  <div className="row" style={{ justifyContent: 'space-between', marginBottom: 6 }}>
                    <b style={{ fontSize: 14 }}>{c.name}</b>
                    <span className={`pill ${measuring ? 'accent' : ''}`}>
                      {measuring ? 'Midiendo' : `${c.min}–${c.max} A`}
                    </span>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'center', padding: '4px 0' }}>
                    <Gauge label={c.name} value={v ?? 0} min={c.min} max={c.max} size={124} />
                  </div>
                  <button
                    className={`btn ${measuring ? '' : 'primary'}`}
                    style={{ width: '100%', marginTop: 8 }}
                    disabled={busy || !c.enabled}
                    onClick={() => testOne(c.name)}
                  >
                    {measuring ? 'Midiendo…' : 'Probar'}
                  </button>
                </motion.div>
              );
            })}
            {!items.length && (
              <div className="card" style={{ gridColumn: '1 / -1', textAlign: 'center', padding: 32 }}>
                <div className="page-title" style={{ fontSize: 18 }}>La receta no tiene componentes</div>
                <p className="page-sub">Edite la receta y agregue componentes habilitados.</p>
              </div>
            )}
          </div>
        )}
      </SplitLayout>

      <AnimatePresence>
        {pending && (
          <motion.div
            initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
            style={{ position: 'fixed', inset: 0, background: 'rgba(15,23,42,.55)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 6000, padding: 16 }}
            onClick={() => setPending(null)}
          >
            <motion.div
              initial={{ scale: 0.96, y: 10 }} animate={{ scale: 1, y: 0 }} exit={{ scale: 0.96, opacity: 0 }}
              className="card" style={{ width: 'min(420px, 96vw)' }}
              onClick={e => e.stopPropagation()}
            >
              <h3>{pending === 'START' ? 'Iniciar Hipot' : 'Detener Hipot'}</h3>
              <p className="page-sub" style={{ marginBottom: 16 }}>
                {pending === 'START'
                  ? 'Se aplicará alta tensión a la unidad. Verifique que el área esté despejada.'
                  : 'Se detendrá la prueba de Hipot en curso.'}
              </p>
              <div className="row" style={{ justifyContent: 'flex-end' }}>
                <button className="btn" onClick={() => setPending(null)}>Cancelar</button>
                <button className={`btn ${pending === 'START' ? 'ok' : 'danger'}`} onClick={runPending}>
                  {pending === 'START' ? 'Iniciar Hipot' : 'Detener Hipot'}
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
