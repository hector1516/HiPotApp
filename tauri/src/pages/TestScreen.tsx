import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Gauge } from '../components/Gauge';

export function TestScreen({ phase, recipe, values, limits, onStop }: {
  phase: 'PLC_RUNNING' | 'HIPOT_RUNNING';
  recipe: any;
  values: Record<string, number>;
  limits: Record<string, { min: number; max: number }>;
  onStop: () => void;
}) {
  const [secs, setSecs] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setSecs(s => s + 1), 1000);
    return () => clearInterval(id);
  }, []);
  const isPlc = phase === 'PLC_RUNNING';
  const mm = String(Math.floor(secs / 60)).padStart(2, '0');
  const ss = String(secs % 60).padStart(2, '0');

  return (
    <div style={{ padding: 14, minHeight: 520, position: 'relative', zIndex: 1, minWidth: 0 }}>
      <div style={{
        display: 'flex', alignItems: 'center', gap: 12, marginBottom: 10,
        background: '#fff', border: '1px solid var(--line)', borderRadius: 10, padding: '10px 14px',
      }}>
        <span style={{ width: 10, height: 10, borderRadius: '50%', background: isPlc ? '#b7790f' : '#7c3aed', flexShrink: 0 }} />
        <h2 style={{ margin: 0, flex: 1, fontSize: 16, color: '#141b26' }}>
          {isPlc ? 'Prueba PLC en curso' : 'Prueba Hipot en curso'}
        </h2>
        <span className="pill" style={{ fontSize: 13, fontVariantNumeric: 'tabular-nums' }}>{mm}:{ss}</span>
        {recipe && <span className="pill">{recipe.model}</span>}
      </div>

      <div className="progress-track" style={{ marginBottom: 10 }}>
        <motion.div className="progress-fill" initial={false}
          animate={{ width: isPlc ? '48%' : '84%' }} transition={{ duration: 0.6 }} />
      </div>

      <div style={{ display: 'flex', gap: 8, marginBottom: 10, flexWrap: 'wrap' }}>
        <span className={`pill ${isPlc ? 'accent' : 'on'}`}>{isPlc ? '● PLC midiendo' : '✓ PLC completado'}</span>
        <span className={`pill ${!isPlc ? 'accent' : ''}`}>{!isPlc ? '● Hipot midiendo' : '○ Hipot pendiente'}</span>
      </div>

      <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', alignItems: 'flex-start' }}>
        {Object.entries(values).map(([k, v]) => {
          const lim = limits[k];
          return <Gauge key={k} label={k} value={Number(v)} min={lim?.min ?? 0} max={lim?.max ?? Math.max(Number(v) * 1.2, 1)} size={132} />;
        })}
      </div>

      <div style={{ marginTop: 14, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        <button className="btn danger" onClick={onStop}>Parar prueba</button>
        <span style={{ fontSize: 12, color: '#64748b', alignSelf: 'center' }}>El hardware mide un componente a la vez.</span>
      </div>
    </div>
  );
}
