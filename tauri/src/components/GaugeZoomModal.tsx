import React, { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { AnimatePresence, motion } from 'framer-motion';

export type ZoomDetail = { label: string; value: string };
export type ZoomRelated = { label: string; value: string; unit?: string; active?: boolean };

export type ZoomGauge = {
  label: string;
  value: number;
  min: number;
  max: number;
  unit: string;
  unknown?: boolean;
  details?: ZoomDetail[];
  related?: ZoomRelated[];
};

export function playGaugeTone(kind: 'open' | 'close') {
  try {
    const Ctx = window.AudioContext || (window as any).webkitAudioContext;
    if (!Ctx) return;
    const ctx = new Ctx();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    const now = ctx.currentTime;
    osc.type = 'sine';
    osc.frequency.setValueAtTime(kind === 'open' ? 720 : 420, now);
    osc.frequency.exponentialRampToValueAtTime(kind === 'open' ? 980 : 320, now + 0.12);
    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.exponentialRampToValueAtTime(0.035, now + 0.012);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.2);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.22);
    osc.onended = () => { void ctx.close(); };
  } catch {
    // El sonido es opcional; nunca debe bloquear la lectura.
  }
}

function decimalsFor(value: number, unit: string): number {
  const abs = Math.abs(value);
  if (unit === 'V') return abs >= 1000 ? 2 : abs >= 100 ? 3 : 4;
  if (abs >= 100) return 2;
  if (abs >= 10) return 3;
  return 4;
}

function AnimatedValue({ value, decimals }: { value: number; decimals: number }) {
  const [display, setDisplay] = useState(0);
  const fromRef = useRef(0);

  useEffect(() => {
    const from = fromRef.current;
    const target = Number.isFinite(value) ? value : 0;
    const start = performance.now();
    const duration = 720;
    let raf = 0;
    const tick = (now: number) => {
      const t = Math.min(1, (now - start) / duration);
      const eased = 1 - Math.pow(1 - t, 3);
      setDisplay(from + (target - from) * eased);
      if (t < 1) raf = requestAnimationFrame(tick);
      else fromRef.current = target;
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [value]);

  return <>{display.toFixed(decimals)}</>;
}

export function GaugeZoomModal({
  item,
  onClose,
}: {
  item: ZoomGauge | null;
  onClose: () => void;
}) {
  useEffect(() => {
    if (!item) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    document.body.classList.add('gauge-zoom-open');
    return () => {
      window.removeEventListener('keydown', onKey);
      document.body.classList.remove('gauge-zoom-open');
    };
  }, [item, onClose]);

  const ok = !!item && !item.unknown && item.value >= item.min && item.value <= item.max;
  const status = !item ? '' : item.unknown ? 'SIN LECTURA' : ok ? 'EN RANGO' : 'FUERA DE RANGO';

  return createPortal(
    <AnimatePresence>
      {item && (
        <motion.div
          className="gauge-zoom-backdrop"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.28 }}
          onClick={onClose}
          role="presentation"
        >
          <motion.div
            className={`gauge-zoom ${item.unknown ? 'unknown' : ok ? 'ok' : 'bad'}`}
            initial={{ opacity: 0, scale: 0.86 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.92 }}
            transition={{ type: 'spring', stiffness: 190, damping: 24 }}
            onClick={e => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label={`${item.label}: ${item.unknown ? 'sin lectura' : item.value}`}
          >
            <div className="gauge-zoom-aurora" aria-hidden="true" />
            <div className="gauge-zoom-grid" aria-hidden="true" />

            <button type="button" className="gauge-zoom-close" onClick={onClose} aria-label="Cerrar lectura">
              ×
            </button>

            <div className="gauge-zoom-hero">
              <motion.div
                className="gauge-zoom-ring gauge-zoom-ring-a"
                initial={{ scale: 0.7, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                transition={{ duration: 0.9, delay: 0.1 }}
                aria-hidden="true"
              />
              <motion.div
                className="gauge-zoom-ring gauge-zoom-ring-b"
                initial={{ scale: 0.5, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                transition={{ duration: 1.1, delay: 0.2 }}
                aria-hidden="true"
              />

              <motion.div
                className="gauge-zoom-kicker"
                initial={{ opacity: 0, y: -8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.12 }}
              >
                LECTURA EN VIVO · ESC PARA CERRAR
              </motion.div>

              <motion.div
                className="gauge-zoom-label"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.18 }}
              >
                {item.label}
              </motion.div>

              <motion.div
                className="gauge-zoom-number"
                initial={{ scale: 0.62, opacity: 0, filter: 'blur(14px)' }}
                animate={{ scale: 1, opacity: 1, filter: 'blur(0px)' }}
                transition={{ type: 'spring', stiffness: 130, damping: 18, delay: 0.12 }}
              >
                {item.unknown ? '—' : <AnimatedValue value={item.value} decimals={decimalsFor(item.value, item.unit)} />}
                <span>{item.unit}</span>
              </motion.div>

              <motion.div
                className="gauge-zoom-status"
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.28 }}
              >
                {status}
              </motion.div>

              <motion.div
                className="gauge-zoom-range"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.34 }}
              >
                Rango: {item.min} – {item.max} {item.unit}
              </motion.div>
            </div>

            <div className="gauge-zoom-content">
              {!!item.details?.length && (
                <div className="gauge-zoom-details">
                  {item.details.map((d, i) => (
                    <div key={`${d.label}-${i}`} className="gauge-zoom-detail">
                      <span>{d.label}</span>
                      <b>{d.value}</b>
                    </div>
                  ))}
                </div>
              )}

              {!!item.related?.length && (
                <div className="gauge-zoom-related">
                  <div className="gauge-zoom-subtitle">Valores relacionados</div>
                  <div className="gauge-zoom-related-grid">
                    {item.related.map((r, i) => (
                      <div key={`${r.label}-${i}`} className={`gauge-zoom-related-item ${r.active ? 'active' : ''}`}>
                        <span>{r.label}</span>
                        <b>{r.value}{r.unit ? ` ${r.unit}` : ''}</b>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div className="gauge-zoom-actions">
              <button type="button" className="btn" onClick={onClose}>
                Cerrar
              </button>
              <span>o presione ESC</span>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  );
}
