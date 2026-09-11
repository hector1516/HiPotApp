import React, { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

export function ResultScreen({ status, rows, duration, holdSeconds = 10, printStatus = null, onNext }: {
  status: 'GOOD' | 'BAD';
  rows: { label: string; value: string; ok: boolean | null }[];
  duration: string;
  holdSeconds?: number;
  printStatus?: 'sending' | 'sent' | 'error' | null;
  onNext: () => void;
}) {
  const good = status === 'GOOD';
  const [remaining, setRemaining] = useState(holdSeconds);
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

  const progress = holdSeconds > 0 ? remaining / holdSeconds : 0;
  const R = 46;
  const C = 2 * Math.PI * R;

  return (
    <div style={{ padding: 12, height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'space-between', textAlign: 'center', position: 'relative', zIndex: 1, minWidth: 0, overflow: 'hidden' }}>
      <div style={{ flexShrink: 0 }}>
        <motion.div
          initial={{ scale: 0.96, opacity: 0 }} animate={{ scale: 1, opacity: 1 }}
          transition={{ duration: 0.3 }}
          className="stamp"
          style={{ color: good ? '#166534' : '#991b1b', borderColor: good ? '#166534' : '#991b1b', marginTop: 2, padding: '4px 20px', fontSize: 40 }}
        >
          {good ? 'BUENO' : 'MALO'}
        </motion.div>
        <div style={{ fontSize: 13, color: '#64748b', marginTop: 4 }}>Duración total: <b style={{ color: '#1e293b' }}>{duration}</b></div>
      </div>

      {/* Aviso de etiqueta — solo si BUENO (ambas pruebas OK) */}
      <AnimatePresence>
        {good && printStatus && (
          <motion.div initial={{ opacity: 0, y: -8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}
            style={{
              width: '100%', maxWidth: 540, display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0,
              background: printStatus === 'error' ? '#fef2f2' : '#e6f4ea',
              border: `1px solid ${printStatus === 'error' ? '#fecaca' : '#bbd8c0'}`,
              borderLeft: `4px solid ${printStatus === 'error' ? '#991b1b' : '#166534'}`,
              borderRadius: 8, padding: '6px 12px', textAlign: 'left',
            }}>
            <motion.span animate={printStatus === 'sending' ? { y: [0, -3, 0] } : {}}
              transition={{ duration: 0.7, repeat: printStatus === 'sending' ? Infinity : 0 }}
              style={{ fontSize: 20, flexShrink: 0 }}>
              {printStatus === 'sending' ? '🖨️' : printStatus === 'sent' ? '✅' : '⚠️'}
            </motion.span>
            <div style={{ minWidth: 0, flex: 1 }}>
              <div style={{ fontSize: 13, fontWeight: 800, color: printStatus === 'error' ? '#991b1b' : '#166534' }}>
                {printStatus === 'sending' ? 'Enviando etiqueta a Zebra ZT231…' : printStatus === 'sent' ? 'Etiqueta impresa — Zebra ZT231' : 'No se pudo imprimir — reintente desde Historial'}
              </div>
              <div style={{ fontSize: 12, color: '#64748b' }}>
                {printStatus === 'sent' ? 'BUENO en eléctrica e Hipot · Etiqueta generada' : printStatus === 'sending' ? 'Impresión automática por resultado BUENO' : 'Ver Sistema → Historial → Reimprimir'}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <div className="card" style={{ width: '100%', maxWidth: 540, padding: '8px 14px', textAlign: 'left', flexShrink: 0 }}>
        {rows.map((r, i) => (
          <div key={i} style={{ display: 'flex', gap: 10, padding: '5px 0',
                                borderBottom: i < rows.length - 1 ? '1px solid #e8edf3' : 'none', fontSize: 13, minWidth: 0 }}>
            <span style={{ flex: 1, color: '#64748b' }}>{r.label}</span>
            <b style={{ color: '#1e293b' }}>{r.value}</b>
            {r.ok === true && <span className="pill on" style={{ fontSize: 11, padding: '2px 8px' }}>OK</span>}
            {r.ok === false && <span className="pill off" style={{ fontSize: 11, padding: '2px 8px' }}>FALLO</span>}
          </div>
        ))}
      </div>

      {/* Cuenta regresiva Hipot — anillo + logo */}
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 6, flexShrink: 0, paddingBottom: 4 }}>
        <div style={{ position: 'relative', width: 96, height: 96 }}>
          <svg width={96} height={96} viewBox="0 0 110 110">
            <circle cx={55} cy={55} r={R} fill="none" stroke="#e2e8f0" strokeWidth={8} />
            <motion.circle
              cx={55} cy={55} r={R} fill="none"
              stroke={good ? '#166534' : '#991b1b'} strokeWidth={8} strokeLinecap="round"
              strokeDasharray={C} strokeDashoffset={C * (1 - progress)}
              transform="rotate(-90 55 55)"
              transition={{ duration: 0.3 }}
            />
          </svg>
          <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
            <b style={{ fontSize: 24, color: '#141b26', lineHeight: 1 }}>{remaining}</b>
            <span style={{ fontSize: 10, color: '#64748b', fontWeight: 700, letterSpacing: '.08em' }}>SEG</span>
          </div>
        </div>
        <div className="progress-track" style={{ width: 260, height: 6 }}>
          <motion.div className="progress-fill" style={{ background: good ? '#166534' : '#991b1b' }}
            animate={{ width: `${progress * 100}%` }} transition={{ duration: 0.3 }} />
        </div>
        <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', justifyContent: 'center', marginTop: 4 }}>
          <button className="btn primary big" onClick={onNext} autoFocus style={{ minHeight: 52 }}>
            Siguiente unidad →
          </button>
          <button className="btn" onClick={onNext} style={{ minHeight: 52 }}>Omitir espera</button>
        </div>
      </div>
    </div>
  );
}
