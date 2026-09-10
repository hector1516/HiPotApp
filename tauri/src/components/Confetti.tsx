import React, { useMemo } from 'react';
import { motion } from 'framer-motion';

// Lluvia de confeti CSS/motion (sin librerías) para el BUENO.
const COLORS = ['#2e7d32', '#1976d2', '#ff9800', '#9c27b0', '#ffeb3b', '#00bcd4', '#e91e63'];

export function Confetti({ pieces = 90 }: { pieces?: number }) {
  const items = useMemo(() => Array.from({ length: pieces }, (_, i) => ({
    id: i,
    x: Math.random() * 100,
    w: 8 + Math.random() * 8,
    h: 10 + Math.random() * 10,
    c: COLORS[i % COLORS.length],
    d: 2.2 + Math.random() * 2.2,
    delay: Math.random() * 1.2,
    rot: Math.random() * 720 - 360,
    round: Math.random() > 0.7,
  })), [pieces]);
  return (
    <div style={{ position: 'fixed', inset: 0, overflow: 'hidden', zIndex: 2000, pointerEvents: 'none' }}>
      {items.map(p => (
        <motion.div
          key={p.id}
          initial={{ x: 0, y: '-5vh', rotate: 0, opacity: 1 }}
          animate={{ y: '108vh', x: [0, 40, -40, 20, 0], rotate: p.rot, opacity: [1, 1, 0.9, 1] }}
          transition={{ duration: p.d, delay: p.delay, ease: 'easeIn' }}
          style={{
            position: 'absolute', left: `${p.x}%`, top: 0,
            width: p.w, height: p.h, background: p.c,
            borderRadius: p.round ? '50%' : 2,
          }}
        />
      ))}
    </div>
  );
}
