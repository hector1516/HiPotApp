import { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { SplitLayout } from '../components/SplitLayout';
import { PageHeader } from '../components/PageHeader';

export function ResultScreen({
  status, rows, duration, holdSeconds = 10, printStatus = null, onNext, onReprint,
}: {
  status: 'GOOD' | 'BAD';
  rows: { label: string; value: string; ok: boolean | null }[];
  duration: string;
  holdSeconds?: number;
  printStatus?: 'sending' | 'sent' | 'error' | null;
  onNext: () => void;
  onReprint?: () => void;
}) {
  const good = status === 'GOOD';
  const [remaining, setRemaining] = useState(holdSeconds);
  const [held, setHeld] = useState(false);
  const rafRef = useRef<number | null>(null);
  const startRef = useRef<number>(Date.now());
  const onNextRef = useRef(onNext);
  useEffect(() => { onNextRef.current = onNext; }, [onNext]);

  useEffect(() => {
    setRemaining(holdSeconds);
    startRef.current = Date.now();
    const tick = () => {
      const elapsed = (Date.now() - startRef.current) / 1000;
      const left = Math.max(0, Math.ceil(holdSeconds - elapsed));
      setRemaining(left);
      if (elapsed >= holdSeconds) onNextRef.current();
      else rafRef.current = window.setTimeout(tick, 200) as unknown as number;
    };
    rafRef.current = window.setTimeout(tick, 200) as unknown as number;
    return () => { if (rafRef.current) clearTimeout(rafRef.current); };
  }, [holdSeconds]);

  const progress = holdSeconds > 0 && !held ? remaining / holdSeconds : 0;
  const R = 52;
  const C = 2 * Math.PI * R;
  const color = good ? 'var(--ok)' : 'var(--bad)';

  const aside = (
    <>
      <div>
        <div className="rail-eyebrow">Resultado final</div>
        <h2 className="rail-title">{good ? 'Unidad aprobada' : 'Unidad rechazada'}</h2>
        <p className="rail-sub">
          {good ? 'Ambas pruebas pasaron. Puede continuar con la siguiente unidad.' : 'Una o más pruebas fallaron. Revise la unidad y el historial.'}
        </p>
      </div>

      <div className="rail-divider" />

      <motion.div
        initial={{ scale: 0.94, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: 'spring', stiffness: 220, damping: 18 }}
        style={{
          alignSelf: 'center', color, border: `4px solid ${color}`, borderRadius: 14,
          padding: '8px 26px', fontSize: 44, fontWeight: 900, letterSpacing: 3,
          background: 'rgba(255,255,255,.94)', transform: 'rotate(-2deg)',
        }}
      >
        {good ? 'BUENO' : 'MALO'}
      </motion.div>

      <div className="rail-kpi">
        <span className="label">Duración total</span>
        <span className="value mono">{duration}</span>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8 }}>
        <div style={{ position: 'relative', width: 116, height: 116 }}>
          <svg width={116} height={116} viewBox="0 0 116 116">
            <circle cx={58} cy={58} r={R} fill="none" stroke="rgba(255,255,255,.12)" strokeWidth={8} />
            <motion.circle
              cx={58} cy={58} r={R} fill="none" stroke={color} strokeWidth={8} strokeLinecap="round"
              strokeDasharray={C} strokeDashoffset={C * (1 - progress)}
              transform="rotate(-90 58 58)" transition={{ duration: 0.3 }}
            />
          </svg>
          <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
            <img src="logo.png" alt="" style={{ width: 30, opacity: .9, marginBottom: 2 }} />
            <b style={{ fontSize: 26, color: '#fff', lineHeight: 1 }}>{held ? '—' : remaining}</b>
            <span style={{ fontSize: 10, color: 'var(--on-rail-muted)', fontWeight: 700, letterSpacing: '.08em' }}>SEG</span>
          </div>
        </div>
        <div className="rail-status" style={{ fontSize: 11, color: 'var(--on-rail-muted)' }}>
          {held ? 'Espera omitida' : 'Volviendo al inicio automáticamente'}
        </div>
      </div>

      <div className="rail-actions">
        <button className="btn primary" style={{ width: '100%' }} onClick={onNext}>
          Siguiente unidad →
        </button>
        <button className="btn" style={{ width: '100%' }} onClick={() => setHeld(true)} disabled={held}>
          Omitir espera
        </button>
      </div>
    </>
  );

  return (
    <SplitLayout aside={aside}>
      <PageHeader
        eyebrow={good ? 'Resultado: BUENO' : 'Resultado: MALO'}
        title={good ? 'Prueba completada correctamente' : 'Prueba con fallo'}
        subtitle="Resumen de la ejecución y estado de impresión de la etiqueta."
        actions={
          onReprint ? (
            <button className="btn" onClick={onReprint}>Reimprimir etiqueta</button>
          ) : null
        }
      />

      <AnimatePresence>
        {good && printStatus && (
          <motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            className={`alert ${printStatus === 'error' ? '' : 'ok'}`}
            style={{ marginBottom: 16 }}
          >
            <div style={{ flex: 1 }}>
              <b>
                {printStatus === 'sending' ? 'Enviando etiqueta a Zebra ZT231…' : printStatus === 'sent' ? 'Etiqueta impresa · Zebra ZT231' : 'No se pudo imprimir la etiqueta'}
              </b>
              <div style={{ fontWeight: 500, fontSize: 12, marginTop: 2 }}>
                {printStatus === 'sent' ? 'BUENO en eléctrica e Hipot · Etiqueta generada' : printStatus === 'sending' ? 'Impresión automática por resultado BUENO' : 'Use Reimprimir etiqueta o Historial → Reimprimir.'}
              </div>
            </div>
            {printStatus === 'sending' && (
              <motion.span animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>◌</motion.span>
            )}
          </motion.div>
        )}
      </AnimatePresence>

      <div className="card">
        <h3>Resumen</h3>
        <div className="table-wrap">
          <table className="grid">
            <thead><tr><th>Concepto</th><th>Valor</th><th>Estado</th></tr></thead>
            <tbody>
              {rows.map((r, i) => (
                <tr key={i}>
                  <td className="muted">{r.label}</td>
                  <td><b>{r.value}</b></td>
                  <td>
                    {r.ok === true ? <span className="pill on">OK</span>
                      : r.ok === false ? <span className="pill off">FALLO</span>
                      : <span className="pill">—</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3>¿Qué sigue?</h3>
        <div className="stack" style={{ gap: 8 }}>
          <div className="rail-status" style={{ color: 'var(--text-2)' }}>
            <span className="dot ok" />
            <span>Retire la unidad y coloque la siguiente.</span>
          </div>
          <div className="rail-status" style={{ color: 'var(--text-2)' }}>
            <span className="dot ok" />
            <span>Escanee el nuevo modelo para iniciar otro ciclo.</span>
          </div>
          <div className="rail-status" style={{ color: 'var(--text-2)' }}>
            <span className="dot warn" />
            <span>Si la etiqueta no salió, use Reimprimir etiqueta.</span>
          </div>
        </div>
      </div>
    </SplitLayout>
  );
}
