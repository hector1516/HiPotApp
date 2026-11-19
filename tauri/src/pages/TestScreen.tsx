import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Gauge } from '../components/Gauge';
import { SplitLayout } from '../components/SplitLayout';
import { PageHeader } from '../components/PageHeader';
import { SafetyNote } from '../components/SafetyNote';
import { usePLC } from '../hooks/usePLC';
import { useHipot } from '../hooks/useHipot';

export function TestScreen({ phase, recipe, values, limits, onStop }: {
  phase: 'PLC_RUNNING' | 'HIPOT_RUNNING';
  recipe: any;
  values: Record<string, number>;
  limits: Record<string, { min: number; max: number }>;
  onStop: () => void;
}) {
  const [secs, setSecs] = useState(0);
  const plc = usePLC();
  const hipot = useHipot();

  useEffect(() => {
    const id = setInterval(() => setSecs(s => s + 1), 1000);
    return () => clearInterval(id);
  }, []);

  const isPlc = phase === 'PLC_RUNNING';
  const mm = String(Math.floor(secs / 60)).padStart(2, '0');
  const ss = String(secs % 60).padStart(2, '0');
  const gauges = Object.entries(values);
  const title = isPlc ? 'Prueba eléctrica en curso' : 'Prueba Hipot en curso';

  const aside = (
    <>
      <div>
        <div className="rail-eyebrow">Prueba en curso</div>
        <h2 className="rail-title">{title}</h2>
        <p className="rail-sub">
          {isPlc
            ? 'El PLC está midiendo consumo por componente. No abra el gabinete.'
            : 'El Hipot está midiendo aislamiento. No toque la unidad bajo prueba.'}
        </p>
      </div>

      <div className="rail-divider" />

      <div className="rail-kpi">
        <span className="label">Tiempo</span>
        <span className="value mono">{mm}:{ss}</span>
      </div>
      <div className="rail-kpi">
        <span className="label">Receta</span>
        <span className="value">{recipe?.model ?? '—'}</span>
      </div>
      <div className="rail-kpi">
        <span className="label">Fase</span>
        <span className="value">{isPlc ? 'Eléctrica' : 'Hipot'}</span>
      </div>

      <div className="rail-divider" />

      <div className="stack" style={{ gap: 8 }}>
        <div className="rail-eyebrow">Estado</div>
        <div className="rail-status">
          <span className={`dot ${isPlc ? 'warn' : ''}`} />
          <span>Eléctrica {isPlc ? 'midiendo' : 'pendiente'}</span>
        </div>
        <div className="rail-status">
          <span className={`dot ${!isPlc ? 'warn' : 'ok'}`} />
          <span>Hipot {!isPlc ? 'midiendo' : 'completada'}</span>
        </div>
        <div className="rail-status">
          <span className={`dot ${plc.connected ? 'ok' : 'bad'}`} />
          <span>PLC {plc.connected ? 'conectado' : 'sin conexión'}</span>
        </div>
        <div className="rail-status">
          <span className={`dot ${hipot.connected ? 'ok' : 'bad'}`} />
          <span>Hipot {hipot.connected ? 'conectado' : 'sin conexión'}</span>
        </div>
      </div>

      <div className="rail-actions">
        <button className="btn danger" style={{ width: '100%' }} onClick={onStop}>
          Parar prueba
        </button>
        <div className="rail-status" style={{ fontSize: 11, color: 'var(--on-rail-muted)' }}>
          <span>El hardware mide un componente a la vez.</span>
        </div>
      </div>
    </>
  );

  return (
    <SplitLayout aside={aside}>
      <PageHeader
        eyebrow={isPlc ? 'Fase 2 de 2' : 'Fase 1 de 2'}
        title={title}
        subtitle="Lecturas en vivo. Los valores fuera de límite se marcan en rojo."
        actions={<span className="pill accent mono">{mm}:{ss}</span>}
      />

      {!isPlc && <SafetyNote />}

      <div className="card" style={{ marginBottom: 16 }}>
        <div className="row" style={{ justifyContent: 'space-between', marginBottom: 8 }}>
          <b style={{ fontSize: 13 }}>Progreso de la prueba</b>
          <span className="pill">{isPlc ? 'Eléctrica' : 'Hipot'}</span>
        </div>
        <div className="progress-track" style={{ height: 12 }}>
          <motion.div
            className="progress-fill"
            style={{ width: '34%', background: isPlc ? 'var(--accent)' : '#6d28d9' }}
            animate={{ x: ['-110%', '320%'] }}
            transition={{ duration: 1.8, repeat: Infinity, ease: 'easeInOut' }}
          />
        </div>
        <div className="row" style={{ marginTop: 10, gap: 8 }}>
          <span className={`pill ${isPlc ? 'warn' : ''}`}>Eléctrica {isPlc ? 'midiendo' : 'pendiente'}</span>
          <span className={`pill ${!isPlc ? 'warn' : 'on'}`}>Hipot {!isPlc ? 'midiendo' : 'completada'}</span>
        </div>
      </div>

      {gauges.length ? (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(190px, 1fr))', gap: 12 }}>
          {gauges.map(([k, v]) => {
            const lim = limits[k];
            return (
              <motion.div key={k} initial={{ opacity: 0, scale: 0.97 }} animate={{ opacity: 1, scale: 1 }} transition={{ duration: 0.25 }}>
                <Gauge label={k} value={Number(v)} min={lim?.min ?? 0} max={lim?.max ?? Math.max(Number(v) * 1.2, 1)} size={128} />
              </motion.div>
            );
          })}
        </div>
      ) : (
        <div className="card" style={{ textAlign: 'center', padding: 32 }}>
          <div className="page-title" style={{ fontSize: 18 }}>Esperando lecturas del PLC…</div>
          <p className="page-sub">Los indicadores aparecerán cuando el PLC publique los valores de cada componente.</p>
        </div>
      )}
    </SplitLayout>
  );
}
