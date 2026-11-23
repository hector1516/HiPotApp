import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

type LogicStatus = 'idle' | 'running' | 'ok' | 'fail';

type LogicEvent = {
  step: string;
  label: string;
  status: LogicStatus;
  duration_ms?: number;
  component?: string;
  range?: string;
  channel?: string;
  value?: number;
  raw?: number;
  min?: number;
  max?: number;
  pass?: boolean;
  voltage?: number;
  expected?: number;
  voltage_ok?: boolean;
  outputs?: string[];
  missing?: string[];
  message?: string;
};

type HistoryItem = LogicEvent & { at: string };

const FLOW_STEPS: { id: string; label: string; group?: string }[] = [
  { id: 'INICIO', label: 'Inicio de ciclo' },
  { id: 'HIPOT', label: 'Prueba Hipot', group: 'Hipot' },
  { id: 'HIPOT_RESULT', label: 'Resultado Hipot', group: 'Hipot' },
  { id: 'PREPARACION', label: 'Preparar salidas', group: 'Eléctrica' },
  { id: 'PARO', label: 'Verificar paro', group: 'Eléctrica' },
  { id: 'CALIBRACION', label: 'Verificar calibración', group: 'Eléctrica' },
  { id: 'VOLTAJE', label: 'Seleccionar voltaje', group: 'Eléctrica' },
  { id: 'ESTABILIZAR', label: 'Estabilizar voltaje', group: 'Eléctrica' },
  { id: 'MEDIR_VOLTAJE', label: 'Medir voltaje', group: 'Eléctrica' },
  { id: 'COMPONENTE', label: 'Seleccionar componente', group: 'Componentes' },
  { id: 'ENERGIZAR', label: 'Energizar componente', group: 'Componentes' },
  { id: 'MIDIENDO', label: 'Midiendo corriente', group: 'Componentes' },
  { id: 'EVALUAR', label: 'Evaluar corriente', group: 'Componentes' },
  { id: 'APAGAR', label: 'Apagar salidas', group: 'Cierre' },
  { id: 'RESULTADO', label: 'Resultado de la secuencia', group: 'Cierre' },
  { id: 'FIN', label: 'Ciclo terminado', group: 'Cierre' },
];

const STATUS_TEXT: Record<LogicStatus, string> = {
  idle: 'Pendiente',
  running: 'En curso',
  ok: 'OK',
  fail: 'Fallo',
};

function fmt(value: number | undefined, digits = 2): string {
  return typeof value === 'number' && Number.isFinite(value) ? value.toFixed(digits) : '—';
}

function StatusPill({ status }: { status: LogicStatus }) {
  return <span className={`logic-pill logic-${status}`}>{STATUS_TEXT[status]}</span>;
}

export function LogicScreen({
  onBack,
  adminBatch,
}: {
  onBack: () => void;
  adminBatch?: string;
}) {
  const [current, setCurrent] = useState<LogicEvent | null>(null);
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [running, setRunning] = useState(false);
  const [io, setIo] = useState<any>(null);
  const [nowTick, setNowTick] = useState(0);
  const startedAt = useRef(0);

  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | undefined;
    listen<LogicEvent>('logic-state', event => {
      if (stopped) return;
      const payload = event.payload;
      startedAt.current = performance.now();
      setNowTick(t => t + 1);
      setCurrent(payload);
      setRunning(payload.step !== 'FIN' && payload.status !== 'fail');
      const item = { ...payload, at: new Date().toLocaleTimeString() };
      setHistory(prev => payload.step === 'INICIO' ? [item] : [item, ...prev].slice(0, 14));
    })
      .then(fn => {
        if (stopped) fn();
        else unlisten = fn;
      })
      .catch(() => {});
    return () => {
      stopped = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let busy = false;
    const tick = async () => {
      if (busy) return;
      busy = true;
      try {
        const [ioRes, runRes] = await Promise.allSettled([
          invoke<any>('plc_io_status'),
          invoke<boolean>('test_running'),
        ]);
        if (ioRes.status === 'fulfilled') setIo(ioRes.value);
        if (runRes.status === 'fulfilled') setRunning(runRes.value);
      } catch {
        // Se conserva el último estado visible.
      }
      busy = false;
    };
    tick();
    const id = window.setInterval(tick, 1400);
    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    const id = window.setInterval(() => setNowTick(t => t + 1), 100);
    return () => window.clearInterval(id);
  }, []);

  const duration = Math.max(0, Number(current?.duration_ms ?? 0));
  const elapsed = current && startedAt.current ? performance.now() - startedAt.current : 0;
  const remaining = duration > 0 ? Math.max(0, duration - elapsed) / 1000 : null;
  const progress = duration > 0 ? Math.min(1, elapsed / duration) : 0;

  const activeOutputs = Array.isArray(current?.outputs)
    ? current!.outputs
    : Object.entries(io?.outputs ?? {}).filter(([, v]) => v).map(([k]) => k);

  const stateFor = (id: string): LogicStatus => {
    if (current?.step === id) return current.status;
    const last = history.find(h => h.step === id);
    if (last) return last.status;
    return 'idle';
  };

  const stepCount = FLOW_STEPS.length;
  const activeIndex = current ? FLOW_STEPS.findIndex(s => s.id === current.step) : -1;

  return (
    <div className="logic-page" data-tick={nowTick}>
      <div className="live-page-head">
        <div>
          <div className="rail-eyebrow">Secuencia en la PC</div>
          <h2>Lógica</h2>
        </div>
        <div className="live-page-head-meta">
          <span className={`pill ${running ? 'on' : 'off'}`}>{running ? 'En ejecución' : 'Detenida'}</span>
          <span className="pill">{current ? `${Math.max(activeIndex + 1, 1)}/${stepCount}` : `0/${stepCount}`}</span>
          {adminBatch && <span className="pill">{adminBatch.toUpperCase() === 'GODMODE' ? 'GodMode' : `Admin: ${adminBatch}`}</span>}
          <span className="live-updated">{current ? `Último evento: ${current.label}` : 'Sin eventos'}</span>
          <button className="btn" onClick={onBack}>Volver</button>
        </div>
      </div>

      <div className="logic-layout">
        <section className="live-block logic-flow-panel">
          <div className="live-block-head">
            <h3>Flujo de la secuencia</h3>
            <StatusPill status={(current?.status ?? 'idle') as LogicStatus} />
          </div>
          {!current && (
            <div className="logic-empty">
              Sin ciclo en curso. Inicie una prueba para ver el flujo animado en tiempo real.
            </div>
          )}
          <div className="logic-flow">
            {FLOW_STEPS.map((step, index) => {
              const state = stateFor(step.id);
              const groupStart = index === 0 || FLOW_STEPS[index - 1].group !== step.group;
              return (
                <React.Fragment key={step.id}>
                  {groupStart && step.group && (
                    <div className="logic-group">{step.group}</div>
                  )}
                  <div className={`logic-step logic-${state} ${current?.step === step.id ? 'logic-active' : ''}`}>
                    <div className="logic-node" />
                    <div className="logic-step-body">
                      <div className="logic-step-top">
                        <b>{step.label}</b>
                        <StatusPill status={state} />
                      </div>
                      {current?.step === step.id && (
                        <div className="logic-step-detail">
                          {current.component && <span>Componente: <b>{current.component}</b></span>}
                          {current.range && <span>Rango: <b>{current.range}</b></span>}
                          {current.channel && <span>Canal: <b>{current.channel}</b></span>}
                          {typeof current.value === 'number' && <span>Valor: <b>{fmt(current.value, 3)}</b></span>}
                          {typeof current.min === 'number' && typeof current.max === 'number' && (
                            <span>Límites: <b>{fmt(current.min, 3)} – {fmt(current.max, 3)}</b></span>
                          )}
                          {typeof current.voltage === 'number' && (
                            <span>Voltaje: <b>{fmt(current.voltage, 1)} V</b></span>
                          )}
                          {current.missing?.length ? <span>Faltan: <b>{current.missing.join(', ')}</b></span> : null}
                          {current.message && <span className="logic-message">{current.message}</span>}
                          {duration > 0 && current.status === 'running' && (
                            <div className="logic-progress">
                              <div className="logic-progress-bar" style={{ width: `${progress * 100}%` }} />
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  </div>
                  {index < FLOW_STEPS.length - 1 && <div className={`logic-connector ${state === 'ok' ? 'logic-connector-ok' : ''}`} />}
                  {step.id === 'EVALUAR' && (
                    <div className="logic-loop">↺ repite para el siguiente componente habilitado</div>
                  )}
                </React.Fragment>
              );
            })}
          </div>
        </section>

        <aside className="logic-side">
          <section className="live-block">
            <div className="live-block-head">
              <h3>Estado actual</h3>
              <span className={`dot ${running ? 'ok' : 'bad'}`} />
            </div>
            <div className="live-metric"><span>Paso</span><b>{current?.label ?? 'Sin ciclo'}</b></div>
            <div className="live-metric"><span>Estado</span><b>{STATUS_TEXT[(current?.status ?? 'idle') as LogicStatus]}</b></div>
            <div className="live-metric"><span>Componente</span><b>{current?.component ?? '—'}</b></div>
            <div className="live-metric"><span>Temporizador</span><b>{remaining === null ? '—' : `${remaining.toFixed(1)} s`}</b></div>
            <div className="live-metric"><span>Voltaje</span><b>{typeof current?.voltage === 'number' ? `${fmt(current.voltage, 2)} V` : '—'}</b></div>
            <div className="live-metric"><span>Corriente</span><b>{current?.component ? `${fmt(current.value, 3)} A` : '—'}</b></div>
            <div className="live-metric"><span>Paro (crudo)</span><b>{io?.estop === true ? '1' : io?.estop === false ? '0' : '—'}</b></div>
            <div className="live-metric"><span>Calibración</span><b>{io?.analog_calibrated ? `${Object.values(io.analog_calibrated).filter(Boolean).length}/4` : '—'}</b></div>
            <div className="live-divider" />
            <div className="logic-output-title">Salidas activas</div>
            <div className="live-chips">
              {activeOutputs.length ? activeOutputs.map(q => <span key={q} className="live-chip accent">{q}</span>) : <span className="live-empty">ninguna</span>}
            </div>
          </section>

          <section className="live-block">
            <div className="live-block-head"><h3>Historial reciente</h3></div>
            <div className="logic-history">
              {history.length === 0 && <div className="live-empty">Sin transiciones registradas.</div>}
              {history.map((h, i) => (
                <div key={`${h.step}-${h.at}-${i}`} className="logic-history-row">
                  <span className="mono">{h.at}</span>
                  <span className={`logic-history-dot logic-${h.status}`} />
                  <span className="logic-history-label">{h.label}</span>
                  <span className={`logic-history-status logic-${h.status}`}>{STATUS_TEXT[h.status]}</span>
                </div>
              ))}
            </div>
            {current?.message && <div className="logic-last-error">{current.message}</div>}
          </section>
        </aside>
      </div>
    </div>
  );
}
