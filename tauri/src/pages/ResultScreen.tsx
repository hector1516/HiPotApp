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
    <div style={{ padding: 14, minHeight: 520, textAlign: 'center', position: 'relative', zIndex: 1, minWidth: 0 }}>
      <motion.div
        initial={{ scale: 0.96, opacity: 0 }} animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.3 }}
        className="stamp"
        style={{ color: good ? '#166534' : '#991b1b', borderColor: good ? '#166534' : '#991b1b', marginTop: 4 }}
      >
        {good ? 'BUENO' : 'MALO'}
      </motion.div>
      <div style={{ fontSize: 13, color: '#64748b', margin: '8px 0 10px 0' }}>Duración total: <b style={{ color: '#1e293b' }}>{duration}</b></div>

      {/* Aviso de etiqueta — solo si BUENO (ambas pruebas OK) */}
      <AnimatePresence>
        {good && printStatus && (
          <motion.div initial={{ opacity: 0, y: -8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }}
            style={{
              maxWidth: 560, margin: '0 auto 10px auto', display: 'flex', alignItems: 'center', gap: 10,
              background: printStatus === 'error' ? '#fef2f2' : '#e6f4ea',
              border: `1px solid ${printStatus === 'error' ? '#fecaca' : '#bbd8c0'}`,
              borderLeft: `4px solid ${printStatus === 'error' ? '#991b1b' : '#166534'}`,
              borderRadius: 8, padding: '8px 12px', textAlign: 'left',
            }}>
            <motion.span animate={printStatus === 'sending' ? { y: [0, -3, 0] } : {}}
              transition={{ duration: 0.7, repeat: printStatus === 'sending' ? Infinity : 0 }}
              style={{ fontSize: 22, flexShrink: 0 }}>
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
            {printStatus === 'sending' && (
              <motion.span animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
                style={{ fontSize: 18, flexShrink: 0 }}>◷</motion.span>
            )}
          </motion.div>
        )}
      </AnimatePresence>

      <div className="card" style={{ maxWidth: 560, margin: '0 auto 12px auto', textAlign: 'left' }}>
        {rows.map((r, i) => (
          <div key={i} style={{ display: 'flex', gap: 10, padding: '8px 0',
                                borderBottom: i < rows.length - 1 ? '1px solid #e8edf3' : 'none', fontSize: 14, minWidth: 0 }}>
            <span style={{ flex: 1, color: '#64748b' }}>{r.label}</span>
            <b style={{ color: '#1e293b' }}>{r.value}</b>
            {r.ok === true && <span className="pill on" style={{ fontSize: 11 }}>OK</span>}
            {r.ok === false && <span className="pill off" style={{ fontSize: 11 }}>FALLO</span>}
          </div>
        ))}
      </div>

      {/* Cuenta regresiva Hipot — anillo + logo */}
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8, marginTop: 4 }}>
        <div style={{ position: 'relative', width: 110, height: 110 }}>
          <svg width={110} height={110} viewBox="0 0 110 110">
            <circle cx={55} cy={55} r={R} fill="none" stroke="#e2e8f0" strokeWidth={8} />
            <motion.circle
              cx={55} cy={55} r={R} fill="none"
              stroke={good ? '#166534' : '#991b1b'} strokeWidth={8} strokeLinecap="round"
              strokeDasharray={C} strokeDashoffset={C * (1 - progress)}
              transform="rotate(-90 55 55)"
              transition={{ duration: 0.3 }}
            />
            {[0, 90, 180, 270].map((a, i) => (
              <motion.circle key={a} cx={55 + Math.cos((a * Math.PI) / 180) * (R - 6)} cy={55 + Math.sin((a * Math.PI) / 180) * (R - 6)} r={3}
                fill={progress > (i + 1) / 4 ? (good ? '#166534' : '#991b1b') : '#cbd5e1'}
                animate={{ scale: progress > (i + 1) / 4 ? [1, 1.4, 1] : 1 }}
                transition={{ duration: 0.6, delay: i * 0.08 }}
              />
            ))}
          </svg>
          <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
            <img src="logo.png" alt="" style={{ width: 36, opacity: 0.9 }} />
            <b style={{ fontSize: 22, color: '#141b26', lineHeight: 1 }}>{remaining}</b>
            <span style={{ fontSize: 10, color: '#64748b', fontWeight: 700, letterSpacing: '.08em' }}>SEG</span>
          </div>
        </div>
        <div className="progress-track" style={{ width: 280, height: 6, maxWidth: '90vw' }}>
          <motion.div className="progress-fill" style={{ background: good ? '#166534' : '#991b1b' }}
            animate={{ width: `${progress * 100}%` }} transition={{ duration: 0.3 }} />
        </div>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', justifyContent: 'center' }}>
          <button className="btn primary big" onClick={onNext} autoFocus>
            Siguiente unidad →
          </button>
          <button className="btn" onClick={onNext}>Omitir espera</button>
        </div>
      </div>
    </div>
  );
}
