import React, { useEffect, useRef, useState } from 'react';
import { motion } from 'framer-motion';
import { CountUp } from './CountUp';

// Anillo sobrio: trazo fino, colores contenidos.
export function Gauge({ label, value, min, max, unit = 'A', size = 140, unknown = false, flashOnChange = false, onClick }: {
  label: string; value: number; min: number; max: number; unit?: string; size?: number; unknown?: boolean; flashOnChange?: boolean; onClick?: () => void;
}) {
  const [flash, setFlash] = useState(false);
  const previous = useRef(value);

  useEffect(() => {
    if (!flashOnChange) {
      previous.current = value;
      return;
    }
    const changed = Math.abs(previous.current - value) > 1e-9;
    previous.current = value;
    if (!changed) return;
    setFlash(true);
    const id = window.setTimeout(() => setFlash(false), 520);
    return () => window.clearTimeout(id);
  }, [value, flashOnChange]);

  const ok = !unknown && value >= min && value <= max;
  const span = Math.max(max - min, 1e-6);
  const mid = (min + max) / 2;
  const dev = Math.min(Math.abs(value - mid) / (span / 2), 1);
  const frac = unknown ? 0.02 : ok ? 1 - dev * 0.45 : 0.92;
  const R = 52;
  const C = 2 * Math.PI * R;
  const color = unknown ? '#94a3b8' : ok ? '#166534' : '#991b1b';
  const background = unknown ? '#f8fafc' : flash ? '#dbeafe' : '#fff';
  return (
    <div
      style={{
        background, borderRadius: 10, padding: 10,
        width: size + 20, textAlign: 'center',
        border: `1px solid ${unknown ? '#cbd5e1' : ok ? '#a8d0b4' : '#e8a0a0'}`,
        boxShadow: flash ? '0 0 0 3px rgba(59,130,246,.18), 0 1px 4px rgba(20,27,38,.06)' : '0 1px 4px rgba(20,27,38,.06)',
        transition: 'background-color .18s ease, box-shadow .18s ease',
        cursor: onClick ? 'pointer' : undefined,
      }}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
      title={onClick ? `Ampliar ${label}` : undefined}
      onKeyDown={onClick ? (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick();
        }
      } : undefined}
    >
      <div style={{ fontWeight: 700, fontSize: 13, color: unknown ? '#64748b' : ok ? '#166534' : '#991b1b' }}>
        {label} {unknown ? '· —' : ok ? '· OK' : '· FUERA'}
      </div>
      <svg width={size} height={size} viewBox="0 0 130 130">
        <circle cx="65" cy="65" r={R} fill="none" stroke="#e2e8f0" strokeWidth="10" />
        <motion.circle
          cx="65" cy="65" r={R} fill="none" stroke={color} strokeWidth="10"
          strokeLinecap="round" strokeDasharray={C}
          initial={false}
          animate={{ strokeDashoffset: C * (1 - frac) }}
          transition={{ duration: 0.4 }}
          transform="rotate(-90 65 65)"
          opacity={0.9}
        />
        <text x="65" y="60" textAnchor="middle" fontSize="22" fontWeight="800" fill="#141b26">
          {unknown ? '—' : <CountUp value={value} />}
        </text>
        <text x="65" y="82" textAnchor="middle" fontSize="12" fill="#64748b">{unit}</text>
      </svg>
      <div style={{ fontSize: 12, color: '#64748b', fontWeight: 600 }}>{min} – {max} {unit}</div>
    </div>
  );
}
