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
import { FeedbackHost } from './components/Feedback';
import VirtualKeyboard from './components/VirtualKeyboard';
import { Phase } from './styles/theme';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Router principal: HOME -> TEST -> RESULT + ADMIN (doc 02 flujo operador).
type View = 'home' | 'test' | 'result' | 'admin' | 'manual';

export default function App() {
  const [view, setView] = useState<View>('home');
  const [phase, setPhase] = useState<Phase>('IDLE');
  const [step, setStep] = useState(1);
  const [kbVisible, setKbVisible] = useState(false);
  const [kbLayout, setKbLayout] = useState<'numeric' | 'text'>('text');

  // Manejador global de errores para evitar congelamiento
  React.useEffect(() => {
    const handler = (e: ErrorEvent) => {
      console.error('Error global:', e.error);
    };
    const rejectionHandler = (e: PromiseRejectionEvent) => {
      console.error('Promise rejection:', e.reason);
    };
    window.addEventListener('error', handler);
    window.addEventListener('unhandledrejection', rejectionHandler);
    document.body.classList.toggle('high-contrast', localStorage.getItem('contrast_mode') === '1');
    return () => {
      window.removeEventListener('error', handler);
      window.removeEventListener('unhandledrejection', rejectionHandler);
    };
  }, []);
  const [recipe, setRecipe] = useState<any>(null);
  const [serial, setSerial] = useState('');
  const [batch, setBatch] = useState('');
  const [error, setError] = useState('');
  const [execId, setExecId] = useState<number | null>(null);
  const [values, setValues] = useState<Record<string, number>>({});
  const [final, setFinal] = useState<'GOOD' | 'BAD'>('GOOD');
  const [t0, setT0] = useState<number>(0);
  const [health, setHealth] = useState<boolean>(false);
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
    invoke<any>('get_config').then(c => {
      const v = parseInt(c?.result_hold_seconds ?? '10', 10);
      if (!Number.isNaN(v) && v >= 3 && v <= 60) setHoldSeconds(v);
    }).catch(() => {});
    return () => clearInterval(id);
  }, []);

  // Fase del ciclo real emitida por Rust (PLC -> Hipot)
  React.useEffect(() => {
    let unlistenPhase: (() => void) | undefined;
    let unlistenValues: (() => void) | undefined;
    listen<string>('test-phase', event => {
      if (event.payload === 'HIPOT_RUNNING') setPhase('HIPOT_RUNNING');
    }).then(fn => { unlistenPhase = fn; }).catch(() => {});
    listen<Record<string, number>>('test-values', event => {
      if (event.payload) setValues(event.payload);
    }).then(fn => { unlistenValues = fn; }).catch(() => {});
    return () => { unlistenPhase?.(); unlistenValues?.(); };
  }, []);

  // Health check del núcleo Rust (sin backend HTTP)
  React.useEffect(() => {
    const check = async () => {
      try {
        const r = await invoke<any>('app_health');
        setHealth(!!r && (r.status === 'healthy' || r.database === true));
      } catch { setHealth(false); }
    };
    check();
    const id = setInterval(check, 3000);
    return () => clearInterval(id);
  }, []);

  // Abrir teclado tactil de Windows al enfocar un input
  React.useEffect(() => {
    const onFocus = (e: FocusEvent) => {
      const t = e.target as HTMLInputElement;
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA')) {
        // Solo mostrar teclado automaticamente en el panel de admin (configuracion)
        const inAdmin = !!t.closest('.admin-panel');
        if (!inAdmin) return;
        const mode = t.inputMode || (t.type === 'number' ? 'numeric' : 'text');
        const kbPref = localStorage.getItem('keyboard_mode') || 'integrated';
        if (kbPref === 'windows') {
          (window as any).__TAURI__?.core?.invoke?.('show_touch_keyboard')?.catch?.(() => {});
        } else {
          setKbLayout(mode === 'numeric' ? 'numeric' : 'text');
          setKbVisible(true);
        }
      }
    };

    // Listener para mostrar teclado manualmente (boton en HomeScreen)
    const onShowKb = (e: any) => {
      const layout = e?.detail?.layout || 'text';
      const kbPref = localStorage.getItem('keyboard_mode') || 'integrated';
      if (kbPref === 'windows') {
        (window as any).__TAURI__?.core?.invoke?.('show_touch_keyboard')?.catch?.(() => {});
      } else {
        setKbLayout(layout === 'numeric' ? 'numeric' : 'text');
        setKbVisible(true);
      }
    };
    window.addEventListener('show-virtual-keyboard', onShowKb);
    const onBlur = () => {
      setTimeout(() => {
        const ae = document.activeElement;
        if (!ae || (ae.tagName !== 'INPUT' && ae.tagName !== 'TEXTAREA')) {
          setKbVisible(false);
        }
      }, 100);
    };
    document.addEventListener('focusin', onFocus);
    document.addEventListener('focusout', onBlur);
    window.addEventListener('show-virtual-keyboard', onShowKb);
    return () => {
      document.removeEventListener('focusin', onFocus);
      document.removeEventListener('focusout', onBlur);
      window.removeEventListener('show-virtual-keyboard', onShowKb);
    };
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
      await invoke('barcode_scan', { code, barcodeType: 'model', scanOrder: 1 });
      const r = await invoke<any>('get_recipe_by_model', { model: code });
      setRecipe(r); setStep(2); setPhase('LOADED');
    } catch { setError(`Receta no encontrada: ${code}`); setPhase('ERROR'); later(() => setPhase('IDLE'), 1500); }
  };
  const scanSerial = async (code: string) => {
    setSerial(code);
    await invoke('barcode_scan', { code, barcodeType: 'serial', scanOrder: 2 }).catch(() => {});
    setStep(3); setPhase('READY');
  };
  const scanBatch = async (code: string) => {
    if (!recipe?.id) {
      setError('No hay receta cargada. Escanea el modelo primero.');
      setPhase('ERROR');
      return;
    }
    setBatch(code);
    setError('');
    await invoke('barcode_scan', { code, barcodeType: 'batch', scanOrder: 3 }).catch(() => {});
    const start = Date.now();
    setT0(start);

    let ex = 0;
    try {
      ex = await invoke<number>('start_execution', {
        recipeId: recipe.id, operatorId: null, batch: code, serialNumber: serial });
      setExecId(ex);
    } catch (e: any) {
      setError(`No se pudo registrar la ejecución: ${String(e?.message ?? e)}`);
      setPhase('ERROR');
      return;
    }

    setStep(4); setPhase('PLC_RUNNING'); setView('test');

    // Voltaje de prueba: primer nivel de la receta (o 120 V).
    const voltage = (() => {
      const raw = recipe?.voltage_levels;
      let candidate: any = raw;
      if (typeof raw === 'string') {
        try { candidate = JSON.parse(raw); } catch { candidate = raw; }
      }
      if (Array.isArray(candidate) && candidate.length) {
        const n = Number(candidate[0]);
        if (Number.isFinite(n) && n > 0) return n;
      }
      if (typeof candidate === 'string') {
        const m = candidate.match(/\d+(?:\.\d+)?/);
        if (m) return Number(m[0]);
      }
      return 120;
    })();

    try {
      const r = await invoke<any>('run_test_cycle', {
        executionId: ex,
        recipeId: recipe.id,
        voltage,
        components: recipe?.components ?? [],
      });
      if (r?.error) throw new Error(r.error);

      setValues(r?.values ?? {});
      const s = Math.round((Date.now() - start) / 1000);
      setElapsed(`${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`);
      setFinal(r?.final === 'GOOD' ? 'GOOD' : 'BAD');
      setPhase(r?.final === 'GOOD' ? 'COMPLETE_GOOD' : 'COMPLETE_BAD');
      setStep(6); setView('result');

      if (r?.final === 'GOOD') {
        setPrintStatus('sending');
        try {
          const cfg = await invoke<any>('get_config').catch(() => ({}));
          const ip = cfg?.printer_ip || '';
          const port = parseInt(cfg?.printer_port || '9100', 10);
          const name = cfg?.printer_name || 'Zebra ZT231';
          const p = await invoke<string>('reprint_zebra', {
            eid: ex, printerIp: ip, printerPort: port, printerName: name,
          });
          setPrintStatus(p === 'OK' ? 'sent' : 'error');
        } catch { setPrintStatus('error'); }
      } else {
        setPrintStatus(null);
      }

      if (Array.isArray(r?.errors) && r.errors.length) setError(r.errors.join(' | '));
    } catch (e: any) {
      const s = Math.round((Date.now() - start) / 1000);
      setElapsed(`${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`);
      setError(`Error en prueba: ${String(e?.message ?? e)}`);
      setFinal('BAD');
      setPhase('COMPLETE_BAD');
      setStep(6); setView('result');
      setPrintStatus(null);
    }
  };

  const reset = () => {
    clearTimers();
    setPrintStatus(null);
    invoke<any>('get_config').then(c => {
      const v = parseInt(c?.result_hold_seconds ?? '10', 10);
      if (!Number.isNaN(v) && v >= 3 && v <= 60) setHoldSeconds(v);
    }).catch(() => {});
    setView('home'); setStep(1); setRecipe(null); setSerial(''); setBatch('');
    setError(''); setPhase('IDLE'); setExecId(null);
    setValues({});
  };

  const limits: Record<string, { min: number; max: number }> = {};
  (recipe?.components ?? []).forEach((c: any) => {
    if (c.enabled) limits[c.name] = { min: c.min, max: c.max };
  });

  const reprintCurrent = async () => {
    if (!execId) return;
    setPrintStatus('sending');
    try {
      const cfg = await invoke<any>('get_config').catch(() => ({}));
      const p = await invoke<string>('reprint_zebra', {
        eid: execId,
        printerIp: cfg?.printer_ip || '',
        printerPort: parseInt(cfg?.printer_port || '9100', 10),
        printerName: cfg?.printer_name || 'Zebra ZT231',
      });
      setPrintStatus(p === 'OK' ? 'sent' : 'error');
    } catch {
      setPrintStatus('error');
    }
  };

  const retryHealth = async () => {
    try {
      const r = await invoke<any>('app_health');
      setHealth(!!r && (r.status === 'healthy' || r.database === true));
    } catch {
      setHealth(false);
    }
  };

  return (
    <div className="app-shell">
      <Backdrop phase={phase} />
      <UpdateNotification />
      <FeedbackHost />
      <VirtualKeyboard visible={kbVisible} layout={kbLayout} onClose={() => setKbVisible(false)} />

      <header className="app-topbar">
        <div className="app-brand">
          <img src="logo.png" alt="HiPot Tester" />
          <span style={{ color: '#fff' }}>HiPot<span style={{ color: 'var(--accent-2)' }}>App</span></span>
        </div>
        <span className="pill dark" style={{ fontSize: 11 }}>PASO {Math.min(step, 6)}/6</span>
        {recipe && <span className="pill dark" style={{ fontSize: 11 }}>{recipe.model}</span>}
        {adminBatch && <span className="pill dark" style={{ fontSize: 11 }}>Admin: {adminBatch}</span>}
        <span className="spacer" />
        <span className="app-clock">{clock}</span>
        <span className="rail-status" style={{ fontSize: 12, color: health ? '#86efac' : '#fca5a5' }}>
          <span className={`dot ${health ? 'ok' : 'bad'}`} />
          {health ? 'Sistema OK' : 'BD sin conexión'}
        </span>
        <button className="btn" onClick={() => setShowHistory(true)}>Historial</button>
        <button className="btn" onClick={() => setShowReprint(true)}>Reimprimir</button>
        <button className="btn" onClick={() => setShowLog(true)}>Log</button>
        <button className="btn" onClick={() => setView('manual')}>Manual</button>
        <button className="btn accent" onClick={() => setShowConfigGate(true)} title="Escanea gafete o Ctrl+Shift+K">Configurar</button>
      </header>

      <AnimatePresence>
        {!health && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            style={{ overflow: 'hidden' }}
          >
            <div className="alert" style={{ margin: 0, borderRadius: 0, borderLeftWidth: 4, alignItems: 'center' }}>
              <b>SQL Express no está disponible.</b>
              <span style={{ fontWeight: 500 }}>Revise el servicio o la configuración en Sistema → Base de datos local.</span>
              <span className="spacer" />
              <button className="btn sm" onClick={retryHealth}>Reintentar</button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <main className="app-body" style={{ position: 'relative', zIndex: 1 }}>
        <AnimatePresence mode="wait">
          <motion.div
            key={view + phase}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.24, ease: [0.22, 1, 0.36, 1] }}
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
            onNext={reset} onReprint={reprintCurrent} />
        )}
        {view === 'admin' && <AdminPanel onBack={reset} adminBatch={adminBatch} />}
        {view === 'manual' && <ManualScreen onBack={reset} />}
          </motion.div>
        </AnimatePresence>
      </main>

      <footer className="app-footer">
        <span className={`pill ${phase.startsWith('COMPLETE_GOOD') ? 'on' : phase.startsWith('COMPLETE_BAD') ? 'off' : ''}`} style={{ fontSize: 11 }}>
          {phase.replace(/_/g, ' ')}
        </span>
        <span className="rail-status" style={{ fontSize: 12, color: 'var(--on-rail-muted)' }}>
          <span className={`dot ${health ? 'ok' : 'bad'}`} />
          {health ? 'SQL Express conectado' : 'SQL Express sin conexión'}
        </span>
        <span className="spacer" />
        <span style={{ color: 'var(--muted-2)', fontSize: 11 }}>Offline-first · Windows 11 · v1.0.31 · Ctrl+Shift+K → GodMode</span>
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
