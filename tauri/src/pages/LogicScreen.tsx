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

type FlowComponent = { name: string; enabled: boolean; test_time_ms: number };
type FlowData = {
  stabilize_ms: number;
  default_test_time_ms: number;
  voltage_tolerance_pct: number;
  stop_on_fail: boolean;
  components: FlowComponent[];
};
type FlowResponse = {
  draft: FlowData;
  published: FlowData;
  defaults: FlowData;
  draft_version?: string;
  published_version?: string;
  draft_at?: string;
  draft_by?: string;
  published_at?: string;
  published_by?: string;
  test_running?: boolean;
};

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
  const [flowData, setFlowData] = useState<FlowResponse | null>(null);
  const [draft, setDraft] = useState<FlowData | null>(null);
  const [editMode, setEditMode] = useState(false);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState('');
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

  const cloneFlow = (flow: FlowData): FlowData => JSON.parse(JSON.stringify(flow));

  const loadFlow = async (syncDraft = true) => {
    try {
      const r = await invoke<FlowResponse>('logic_flow_get');
      setFlowData(r);
      if (r.test_running) setRunning(true);
      if (syncDraft) setDraft(cloneFlow(r.draft));
    } catch (e: any) {
      setMsg(`Error al cargar la lógica: ${String(e?.message ?? e)}`);
    }
  };

  useEffect(() => {
    loadFlow();
  }, []);

  const startEdit = () => {
    if (!adminBatch) {
      setMsg('El modo edición requiere acceso de administrador.');
      return;
    }
    if (running) {
      setMsg('Detenga la secuencia antes de editar la lógica.');
      return;
    }
    setDraft(cloneFlow(flowData?.draft ?? flowData?.published ?? flowData?.defaults ?? {
      stabilize_ms: 2400,
      default_test_time_ms: 0,
      voltage_tolerance_pct: 5,
      stop_on_fail: true,
      components: [],
    }));
    setMsg('');
    setEditMode(true);
  };

  const exitEdit = () => {
    setEditMode(false);
    setMsg('');
    if (flowData) setDraft(cloneFlow(flowData.draft));
  };

  const setField = (field: keyof FlowData, value: any) => {
    setDraft(prev => prev ? { ...prev, [field]: value } : prev);
  };

  const setComponent = (index: number, patch: Partial<FlowComponent>) => {
    setDraft(prev => {
      if (!prev) return prev;
      const components = prev.components.map((step, i) => i === index ? { ...step, ...patch } : step);
      return { ...prev, components };
    });
  };

  const moveComponent = (index: number, delta: number) => {
    setDraft(prev => {
      if (!prev) return prev;
      const target = index + delta;
      if (target < 0 || target >= prev.components.length) return prev;
      const components = [...prev.components];
      [components[index], components[target]] = [components[target], components[index]];
      return { ...prev, components };
    });
  };

  const validateDraft = (flow: FlowData): string | null => {
    if (!flow.components.some(step => step.enabled)) return 'Debe quedar al menos un componente habilitado.';
    if (flow.stabilize_ms < 100 || flow.stabilize_ms > 60000) return 'La estabilización debe estar entre 100 y 60000 ms.';
    if (flow.voltage_tolerance_pct < 0.5 || flow.voltage_tolerance_pct > 20) return 'La tolerancia debe estar entre 0.5 y 20 %.';
    if (flow.components.some(step => step.test_time_ms < 0 || step.test_time_ms > 600000)) return 'Los tiempos por componente deben estar entre 0 y 600000 ms.';
    return null;
  };

  const saveDraft = async (): Promise<boolean> => {
    if (!draft) return false;
    const invalid = validateDraft(draft);
    if (invalid) {
      setMsg(invalid);
      return false;
    }
    setBusy(true);
    setMsg('');
    try {
      const r = await invoke<string>('logic_flow_save_draft', {
        payload: draft,
        user: adminBatch || 'ADMIN',
      });
      if (r !== 'saved') throw new Error(r);
      await loadFlow(true);
      setMsg('Borrador guardado y versionado.');
      return true;
    } catch (e: any) {
      setMsg(`No se guardó: ${String(e?.message ?? e)}`);
      return false;
    } finally {
      setBusy(false);
    }
  };

  const publish = async () => {
    if (!draft) return;
    const invalid = validateDraft(draft);
    if (invalid) {
      setMsg(invalid);
      return;
    }
    setBusy(true);
    setMsg('');
    try {
      const saved = await invoke<string>('logic_flow_save_draft', {
        payload: draft,
        user: adminBatch || 'ADMIN',
      });
      if (saved !== 'saved') throw new Error(saved);
      const published = await invoke<string>('logic_flow_publish', {
        user: adminBatch || 'ADMIN',
      });
      if (published !== 'saved') throw new Error(published);
      await loadFlow(true);
      setMsg('Lógica publicada. Se aplicará al próximo ciclo.');
    } catch (e: any) {
      setMsg(`No se publicó: ${String(e?.message ?? e)}`);
    } finally {
      setBusy(false);
    }
  };

  const restoreDefaults = () => {
    if (!flowData) return;
    setDraft(cloneFlow(flowData.defaults));
    setMsg('Valores predeterminados cargados. Guarde o publique para aplicarlos.');
  };

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
          {adminBatch && !editMode && (
            <button className="btn" onClick={startEdit} disabled={running} title="Editor restringido de la secuencia">
              Modo edición
            </button>
          )}
          {editMode && <button className="btn" onClick={exitEdit}>Salir de edición</button>}
          <button className="btn" onClick={onBack}>Volver</button>
        </div>
      </div>

      {msg && <div className="toast-err" style={{ marginBottom: 10 }}>{msg}</div>}

      {editMode && draft && (
        <section className="live-block" style={{ marginBottom: 12 }}>
          <div className="live-block-head">
            <h3>Modo edición restringido</h3>
            <span className="pill">Borrador v{flowData?.draft_version || '0'}</span>
            <span className="pill">Publicado v{flowData?.published_version || '0'}</span>
            {flowData?.published_by && <span className="live-updated">Publicado por {flowData.published_by} · {flowData.published_at}</span>}
          </div>
          {running && (
            <div className="logic-last-error" style={{ marginBottom: 10 }}>
              Hay una secuencia en curso. No se puede guardar ni publicar mientras se ejecuta.
            </div>
          )}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 14 }}>
            <div>
              <div className="logic-editor-title">Tiempos y condiciones</div>
              <label className="logic-editor-field">
                <span>Estabilización de voltaje (ms)</span>
                <input className="input" type="number" min={100} max={60000}
                  value={draft.stabilize_ms}
                  onChange={e => setField('stabilize_ms', Number(e.target.value) || 0)} />
              </label>
              <label className="logic-editor-field">
                <span>Tiempo por defecto por componente (ms, 0 = tiempo de receta)</span>
                <input className="input" type="number" min={0} max={600000}
                  value={draft.default_test_time_ms}
                  onChange={e => setField('default_test_time_ms', Number(e.target.value) || 0)} />
              </label>
              <label className="logic-editor-field">
                <span>Tolerancia de voltaje (%)</span>
                <input className="input" type="number" min={0.5} max={20} step={0.1}
                  value={draft.voltage_tolerance_pct}
                  onChange={e => setField('voltage_tolerance_pct', Number(e.target.value) || 0)} />
              </label>
              <label style={{ display: 'flex', gap: 8, alignItems: 'center', marginTop: 10, fontWeight: 700 }}>
                <input type="checkbox" checked={draft.stop_on_fail}
                  onChange={e => setField('stop_on_fail', e.target.checked)} />
                Detener la secuencia al primer componente fuera de rango
              </label>
              <div style={{ marginTop: 10, fontSize: 11, color: 'var(--muted)' }}>
                Las condiciones de paro, calibración y voltaje son señales predefinidas.
                Las acciones de salida se configuran en <b>Configuración → Salidas</b>.
              </div>
            </div>

            <div>
              <div className="logic-editor-title">Orden y tiempo de componentes</div>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ fontSize: 11, color: 'var(--muted)', textAlign: 'left' }}>
                    <th style={{ padding: '4px 6px' }}>Orden</th>
                    <th style={{ padding: '4px 6px' }}>Activo</th>
                    <th style={{ padding: '4px 6px' }}>Componente</th>
                    <th style={{ padding: '4px 6px' }}>Tiempo (ms)</th>
                  </tr>
                </thead>
                <tbody>
                  {draft.components.map((step, index) => (
                    <tr key={step.name} style={{ borderTop: '1px solid var(--line)' }}>
                      <td style={{ padding: '4px 6px', whiteSpace: 'nowrap' }}>
                        <button className="btn" style={{ minHeight: 28, padding: '0 7px' }} disabled={index === 0} onClick={() => moveComponent(index, -1)}>↑</button>
                        <button className="btn" style={{ minHeight: 28, padding: '0 7px', marginLeft: 3 }} disabled={index === draft.components.length - 1} onClick={() => moveComponent(index, 1)}>↓</button>
                      </td>
                      <td style={{ padding: '4px 6px' }}>
                        <input type="checkbox" checked={step.enabled}
                          onChange={e => setComponent(index, { enabled: e.target.checked })} />
                      </td>
                      <td className="mono" style={{ padding: '4px 6px', fontWeight: 700 }}>{step.name}</td>
                      <td style={{ padding: '4px 6px' }}>
                        <input className="input" type="number" min={0} max={600000} style={{ width: 100 }}
                          value={step.test_time_ms}
                          onChange={e => setComponent(index, { test_time_ms: Number(e.target.value) || 0 })} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <div style={{ marginTop: 6, fontSize: 11, color: 'var(--muted)' }}>
                0 ms significa usar el tiempo de receta. El orden se aplica al próximo ciclo publicado.
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', gap: 8, marginTop: 14, flexWrap: 'wrap' }}>
            <button className="btn primary" onClick={saveDraft} disabled={busy || running}>
              {busy ? 'Guardando…' : 'Guardar borrador'}
            </button>
            <button className="btn accent" onClick={publish} disabled={busy || running}>
              {busy ? 'Publicando…' : 'Publicar lógica'}
            </button>
            <button className="btn" onClick={restoreDefaults} disabled={busy || running}>Restaurar predeterminados</button>
            <button className="btn" onClick={exitEdit} disabled={busy}>Salir de edición</button>
          </div>
        </section>
      )}

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
