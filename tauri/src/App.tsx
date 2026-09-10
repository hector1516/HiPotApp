import React, { useRef, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { HomeScreen } from './pages/HomeScreen';
import { TestScreen } from './pages/TestScreen';
import { ResultScreen } from './pages/ResultScreen';
import { AdminPanel } from './pages/AdminPanel';
import { ManualScreen } from './pages/ManualScreen';
import { UpdateNotification } from './components/UpdateNotification';
import { Backdrop } from './components/Backdrop';
import { ReprintPopup } from './components/ReprintPopup';
import { LogPopup } from './components/LogPopup';
import { HistoryPopup } from './components/HistoryPopup';
import { ConfigGatePopup } from './components/ConfigGatePopup';
import { Phase } from './styles/theme';
import { api } from './api';

// Router principal: HOME -> TEST -> RESULT + ADMIN (doc 02 flujo operador).
type View = 'home' | 'test' | 'result' | 'admin' | 'manual';

export default function App() {
  const [view, setView] = useState<View>('home');
  const [phase, setPhase] = useState<Phase>('IDLE');
  const [step, setStep] = useState(1);
  const [recipe, setRecipe] = useState<any>(null);
  const [serial, setSerial] = useState('');
  const [batch, setBatch] = useState('');
  const [error, setError] = useState('');
  const [execId, setExecId] = useState<number | null>(null);
  const [values, setValues] = useState<Record<string, number>>({ FAN: 0.45, DOOR: 2.1 });
  const [final, setFinal] = useState<'GOOD' | 'BAD'>('GOOD');
  const [t0, setT0] = useState<number>(0);
  const [elapsed, setElapsed] = useState('0:00');
  const [holdSeconds, setHoldSeconds] = useState(10);
  const [printStatus, setPrintStatus] = useState<'sending' | 'sent' | 'error' | null>(null);
  const [showReprint, setShowReprint] = useState(false);
  const [showLog, setShowLog] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [showConfigGate, setShowConfigGate] = useState(false);
  const [godActive, setGodActive] = useState(false);
  const [adminBatch, setAdminBatch] = useState<string>('');
  const [clock, setClock] = useState('');
  const timers = useRef<number[]>([]);

  React.useEffect(() => {
    const id = setInterval(() => setClock(new Date().toLocaleTimeString()), 1000);
    api.get('/admin/config').then(c => {
      const v = parseInt(c?.result_hold_seconds ?? '10', 10);
      if (!Number.isNaN(v) && v >= 3 && v <= 60) setHoldSeconds(v);
    }).catch(() => {});
    return () => clearInterval(id);
  }, []);

  const triggerGodMode = React.useCallback(() => {
    if (godActive) return;
    setGodActive(true);
    setAdminBatch('GODMODE');
    window.setTimeout(() => {
      setGodActive(false);
      setView('admin');
    }, 2300);
  }, [godActive]);

  // Solo Ctrl+Shift+K abre GodMode
  React.useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        triggerGodMode();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [triggerGodMode]);

  const later = (fn: () => void, ms: number) => {
    timers.current.push(window.setTimeout(fn, ms));
  };
  const clearTimers = () => { timers.current.forEach(clearTimeout); timers.current = []; };

  const scanModel = async (code: string) => {
    setError(''); setPhase('SCANNING');
    try {
      await api.scan(code, 'model', 1);
      const r = await api.recipeByModel(code);
      setRecipe(r); setStep(2); setPhase('LOADED');
    } catch { setError(`Receta no encontrada: ${code}`); setPhase('ERROR'); later(() => setPhase('IDLE'), 1500); }
  };
  const scanSerial = async (code: string) => {
    setSerial(code);
    await api.scan(code, 'serial', 2).catch(() => {});
    setStep(3); setPhase('READY');
  };
  const scanBatch = async (code: string) => {
    setBatch(code);
    await api.scan(code, 'batch', 3).catch(() => {});
    try {
      const ex = await api.post('/executions/start', {
        recipe_id: recipe?.id ?? 1, batch: code, serial_number: serial });
      setExecId(ex.execution_id);
    } catch { /* offline: seguir simulado */ }
    const start = Date.now();
    setT0(start);
    setStep(4); setPhase('PLC_RUNNING'); setView('test');
    later(() => { setStep(5); setPhase('HIPOT_RUNNING'); }, 4000);
    later(() => {
      const good = Math.random() > 0.2;
      const s = Math.round((Date.now() - start) / 1000);
      setElapsed(`${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`);
      setFinal(good ? 'GOOD' : 'BAD');
      setPhase(good ? 'COMPLETE_GOOD' : 'COMPLETE_BAD');
      setStep(6); setView('result');
      if (good) {
        setPrintStatus('sending');
        const eid = execId;
        const p = eid
          ? api.post(`/executions/${eid}/reprint`).catch(() => api.post('/admin/print-test'))
          : api.post('/admin/print-test').catch(() => null);
        p.then(() => setPrintStatus('sent')).catch(() => setPrintStatus('error'));
      } else setPrintStatus(null);
    }, 8000);
  };

  const reset = () => {
    clearTimers();
    setPrintStatus(null);
    api.get('/admin/config').then(c => {
      const v = parseInt(c?.result_hold_seconds ?? '10', 10);
      if (!Number.isNaN(v) && v >= 3 && v <= 60) setHoldSeconds(v);
    }).catch(() => {});
    setView('home'); setStep(1); setRecipe(null); setSerial(''); setBatch('');
    setError(''); setPhase('IDLE'); setExecId(null);
    setValues({ FAN: 0.45, DOOR: 2.1 });
  };

  const limits: Record<string, { min: number; max: number }> = {};
  (recipe?.components ?? []).forEach((c: any) => {
    if (c.enabled) limits[c.name] = { min: c.min, max: c.max };
  });

  return (
    <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column', position: 'relative' }}>
      <Backdrop phase={phase} />
      <img src="logo.png" alt="" aria-hidden
        style={{ position: 'fixed', right: 16, bottom: 12, width: 320,
                 opacity: 0.06, zIndex: 0, pointerEvents: 'none' }} />
      <UpdateNotification />
      <header style={{ height: 64, display: 'flex', alignItems: 'center', gap: 10, padding: '0 12px',
                       background: '#141b26', color: '#fff',
                       position: 'relative', zIndex: 1, borderBottom: '3px solid #b7790f' }}>
        <img src="logo.png" alt="HiPot Tester" style={{ height: 42, objectFit: 'contain' }} />
        <b style={{ fontSize: 18, letterSpacing: '-.02em' }}>HiPot<span style={{ color: '#d9a441', fontWeight: 800 }}>App</span></b>
        <span className="pill" style={{ background: '#1e2a3a', borderColor: '#2a3a52', color: '#cbd5e1' }}>PASO {Math.min(step, 6)}/6</span>
        {recipe && <span className="pill" style={{ background: '#1e2a3a', borderColor: '#2a3a52', color: '#e2e8f0' }}>{recipe.model}</span>}
        <span style={{ marginLeft: 'auto', fontSize: 13, color: '#cbd5e1', fontVariantNumeric: 'tabular-nums' }}>{clock}</span>
        <button className="btn" style={{ background: '#fff', color: '#141b26', borderColor: '#d8e0ea' }}
          onClick={() => setShowHistory(true)}>📊 HISTORIAL</button>
        <button className="btn" style={{ background: '#fff', color: '#141b26', borderColor: '#d8e0ea' }}
          onClick={() => setShowReprint(true)}>🖨️ REIMPRIMIR</button>
        <button className="btn" style={{ background: '#fff', color: '#141b26', borderColor: '#d8e0ea' }}
          onClick={() => setShowLog(true)}>📋 LOG</button>
        <button className="btn" style={{ background: '#b7790f', color: '#fff', borderColor: '#b7790f' }}
          onClick={() => setView('manual')}>🧪 MANUAL</button>
        <button className="btn ghost" style={{ background: '#1e2a3a', color: '#fff', borderColor: '#334155' }}
          onClick={() => setShowConfigGate(true)} title="Escanea gafete o Ctrl+Shift+K">CONFIG</button>
      </header>

      <main style={{ flex: 1, position: 'relative', zIndex: 1 }}>
        <AnimatePresence mode="wait">
          <motion.div
            key={view + phase}
            initial={{ opacity: 0, x: 60 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -60 }}
            transition={{ duration: 0.25 }}
          >
        {view === 'home' && (
          <HomeScreen step={step} recipe={recipe} serial={serial} batch={batch} error={error}
            onScanModel={scanModel} onScanSerial={scanSerial} onScanBatch={scanBatch} />
        )}
        {view === 'test' && (
          <TestScreen phase={phase === 'HIPOT_RUNNING' ? 'HIPOT_RUNNING' : 'PLC_RUNNING'}
            recipe={recipe} values={values} limits={limits} onStop={reset} />
        )}
        {view === 'result' && (
          <ResultScreen status={final} duration={elapsed} holdSeconds={holdSeconds} printStatus={printStatus}
            rows={[
              { label: 'Modelo', value: recipe?.model ?? '-', ok: null },
              { label: 'Serie', value: serial || '-', ok: null },
              { label: 'Batch', value: batch || '-', ok: null },
              { label: `Exec #${execId ?? '-'} · PLC`, value: 'PASS', ok: final === 'GOOD' ? true : null },
              { label: 'Hipot', value: 'PASS', ok: final === 'GOOD' ? true : null },
            ]}
            onNext={reset} />
        )}
        {view === 'admin' && <AdminPanel onBack={reset} adminBatch={adminBatch} />}
        {view === 'manual' && <ManualScreen onBack={reset} />}
          </motion.div>
        </AnimatePresence>
      </main>

      <footer style={{ minHeight: 48, display: 'flex', alignItems: 'center', gap: 10,
                       padding: '6px 14px', background: '#141b26', color: '#94a3b8',
                       position: 'relative', zIndex: 1, borderTop: '1px solid #2a3a52', fontSize: 13 }}>
        <span className={`pill ${phase.startsWith('COMPLETE_GOOD') ? 'on' : phase.startsWith('COMPLETE_BAD') ? 'off' : ''}`} style={{ fontSize: 12 }}>
          {phase.replace(/_/g, ' ')}
        </span>
        <span style={{ marginLeft: 'auto' }}>Offline-first · Windows 11 · v1.0.0 · Ctrl+Shift+K → GodMode</span>
      </footer>

      <ReprintPopup open={showReprint} onClose={() => setShowReprint(false)} />
      <LogPopup open={showLog} onClose={() => setShowLog(false)} />
      <HistoryPopup open={showHistory} onClose={() => setShowHistory(false)} />
      <ConfigGatePopup open={showConfigGate} onClose={() => setShowConfigGate(false)} onAdmin={(b)=>{ setAdminBatch(b); setView('admin'); }} />

      <AnimatePresence>
        {godActive && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
            style={{ position: 'fixed', inset: 0, background: 'radial-gradient(ellipse at center, #1e2a3a 0%, #0a0f1a 70%)', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', zIndex: 6000 }}>
            <motion.div
              initial={{ scale: 0.7, opacity: 0 }} animate={{ scale: 1, opacity: 1 }}
              transition={{ type: 'spring', stiffness: 200, damping: 18 }}
              style={{ position: 'relative' }}>
              <motion.img src="logo.png" alt="" style={{ width: 280, filter: 'drop-shadow(0 0 24px rgba(183,121,15,.9))' }}
                animate={{ scale: [1, 1.04, 1], rotate: [0, 1, -1, 0] }} transition={{ duration: 0.9, repeat: Infinity }} />
              <motion.div
                style={{ position: 'absolute', inset: -16, border: '2px solid #b7790f', borderRadius: 20 }}
                animate={{ scale: [1, 1.08], opacity: [0.7, 0] }} transition={{ duration: 0.9, repeat: Infinity }} />
              <motion.div
                style={{ position: 'absolute', inset: -28, border: '1px solid rgba(183,121,15,.4)', borderRadius: 24 }}
                animate={{ scale: [1, 1.12], opacity: [0.4, 0] }} transition={{ duration: 0.9, repeat: Infinity, delay: 0.3 }} />
            </motion.div>
            <motion.div initial={{ y: 20, opacity: 0 }} animate={{ y: 0, opacity: 1 }} transition={{ delay: 0.4 }}
              style={{ marginTop: 22, textAlign: 'center' }}>
              <div style={{ fontSize: 36, fontWeight: 900, color: '#fbbf24', letterSpacing: 4, textShadow: '0 0 18px rgba(183,121,15,.8)' }}>GOD MODE</div>
              <div style={{ fontSize: 13, color: '#94a3b8', letterSpacing: 3, marginTop: 4 }}>ACCESO DE CONFIGURACIÓN</div>
              <motion.div style={{ width: 180, height: 3, background: '#1e2a3a', borderRadius: 999, margin: '14px auto 0 auto', overflow: 'hidden' }}>
                <motion.div initial={{ x: '-100%' }} animate={{ x: '0%' }} transition={{ duration: 2.1, ease: 'easeInOut' }}
                  style={{ width: '100%', height: '100%', background: 'linear-gradient(90deg,#b7790f,#fbbf24)' }} />
              </motion.div>
            </motion.div>
            <motion.div animate={{ opacity: [0.3, 1, 0.3] }} transition={{ duration: 1, repeat: Infinity }}
              style={{ position: 'absolute', bottom: 22, fontSize: 11, color: '#64748b', letterSpacing: 2 }}>HIPOT TESTER · HP-500R · HIGH VOLTAGE</motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
