import React, { useEffect } from 'react';
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
    return () => window.removeEventListener('keydown', onKey);
  }, [item, onClose]);

  const ok = !!item && !item.unknown && item.value >= item.min && item.value <= item.max;
  const status = !item ? '' : item.unknown ? 'SIN LECTURA' : ok ? 'EN RANGO' : 'FUERA DE RANGO';

  return (
    <AnimatePresence>
      {item && (
        <motion.div
          className="gauge-zoom-backdrop"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          role="presentation"
        >
          <motion.div
            className={`gauge-zoom ${item.unknown ? 'unknown' : ok ? 'ok' : 'bad'}`}
            initial={{ opacity: 0, scale: 0.84, y: 18 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: 10 }}
            transition={{ type: 'spring', stiffness: 240, damping: 22 }}
            onClick={onClose}
            role="dialog"
            aria-modal="true"
            aria-label={`${item.label}: ${item.unknown ? 'sin lectura' : item.value}`}
          >
            <div className="gauge-zoom-kicker">LECTURA EN VIVO · CLICK PARA CERRAR</div>
            <div className="gauge-zoom-label">{item.label}</div>

            <div className="gauge-zoom-number">
              {item.unknown ? '—' : item.value.toFixed(2)}
              <span>{item.unit}</span>
            </div>

            <div className="gauge-zoom-status">{status}</div>

            <div className="gauge-zoom-range">
              Rango: {item.min} – {item.max} {item.unit}
            </div>

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

            <div className="gauge-zoom-actions">
              <button
                type="button"
                className="btn"
                onClick={(e) => { e.stopPropagation(); onClose(); }}
              >
                Cerrar
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
