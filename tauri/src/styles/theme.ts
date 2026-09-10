// Paleta estandarizada HiPot — profesional, contenida. El naranja solo como acento fino.
export type Phase =
  | 'IDLE' | 'SCANNING' | 'LOADED' | 'READY'
  | 'PLC_RUNNING' | 'HIPOT_RUNNING' | 'COMPLETE_GOOD' | 'COMPLETE_BAD' | 'ERROR';

// Fondos desaturados por fase: mismo gris base con tinte muy leve — nada que fatigue.
export const BG: Record<Phase, string> = {
  IDLE: '#eef2f6',
  SCANNING: '#e8eef4',
  LOADED: '#eef2f6',
  READY: '#fdf6e3',
  PLC_RUNNING: '#eaf0f7',
  HIPOT_RUNNING: '#f0ecf5',
  COMPLETE_GOOD: '#e8f1e8',
  COMPLETE_BAD: '#f2e6e6',
  ERROR: '#f2e0e0',
};

export const C = {
  navy: '#141b26',
  navy2: '#1e2a3a',
  accent: '#b7790f',      // naranja Hipot desaturado — solo líneas y badges
  accentSoft: '#fdf1d0',
  steel: '#475569',
  line: '#d8e0ea',
  success: '#166534',
  warning: '#92400e',
  error: '#991b1b',
  info: '#1e40af',
  dark: '#141b26',
};

export const touch = {
  btn: { minWidth: 80, minHeight: 56, borderRadius: 8, fontSize: 16, fontWeight: 700 as const },
  big: { minWidth: 160, minHeight: 56, borderRadius: 8, fontSize: 17, fontWeight: 700 as const },
};

export function bgFor(phase: Phase): string { return BG[phase] ?? BG.IDLE; }
