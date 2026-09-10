import React from 'react';
import { motion } from 'framer-motion';

// Backdrop muy sutil — solo un velo desaturado por fase, nada que sature.
const PHASE_BG: Record<string, string> = {
  IDLE: '#eef2f6',
  SCANNING: '#eef2f6',
  LOADED: '#eef2f6',
  READY: '#fdf6e3',
  PLC_RUNNING: '#eaf0f7',
  HIPOT_RUNNING: '#f0ecf5',
  COMPLETE_GOOD: '#e8f1e8',
  COMPLETE_BAD: '#f2e6e6',
  ERROR: '#f2e0e0',
};

export function Backdrop({ phase }: { phase: string }) {
  return (
    <motion.div
      animate={{ backgroundColor: PHASE_BG[phase] ?? PHASE_BG.IDLE }}
      transition={{ duration: 0.6 }}
      style={{ position: 'fixed', inset: 0, zIndex: 0, pointerEvents: 'none' }}
    />
  );
}
