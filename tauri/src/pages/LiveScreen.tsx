import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Gauge } from '../components/Gauge';
import { GaugeZoomModal, playGaugeTone, type ZoomGauge } from '../components/GaugeZoomModal';
import { useHipot } from '../hooks/useHipot';

type IoData = {
  connected?: boolean;
  enabled?: boolean;
  read_ok?: boolean;
  error?: string;
  outputs?: Record<string, boolean>;
  inputs?: Record<string, boolean>;
  analog?: Record<string, number>;
};

type ResultData = {
  complete?: boolean;
  running?: boolean;
  error?: boolean;
  [key: string]: any;
};

const COMPONENTS = ['FAN', 'DOOR', 'FRAME', 'DRAIN', 'DEFROST', 'LIGHT'];

const COMPONENT_ADDRESS: Record<string, string> = {
  FAN: 'DB1001.DBD4',
  DOOR: 'DB1001.DBD8',
  FRAME: 'DB1001.DBD12',
  DRAIN: 'DB1001.DBD16',
  DEFROST: 'DB1001.DBD86',
  LIGHT: 'DB1001.DBD90',
};

function fmtValue(value: any, decimals = 2): string {
  if (value === null || value === undefined) return '—';
  const n = Number(value);
  return Number.isFinite(n) ? n.toFixed(decimals) : '—';
}

type ZoomTarget = {
  kind: 'analog' | 'component' | 'hipot';
  key: string;
  label: string;
  min: number;
  max: number;
  unit: string;
};

const ANALOG_GAUGES = [
  { key: 'IA1', label: 'Amplificador 1', min: 0, max: 1, unit: 'A' },
  { key: 'IA2', label: 'Amplificador 2', min: 0, max: 10, unit: 'A' },
  { key: 'IA3', label: 'Amplificador 3', min: 0, max: 20, unit: 'A' },
  { key: 'IA4', label: 'Voltaje de línea', min: 0, max: 250, unit: 'V' },
];

function statusClass(on: boolean | null | undefined) {
  return on === true ? 'ok' : on === false ? 'bad' : '';
}

function Metric({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="live-metric">
      <span>{label}</span>
      <b>{value}</b>
    </div>
  );
}

function RawRow({ label, value }: { label: string; value: any }) {
  const text = value === null || value === undefined
    ? '—'
    : typeof value === 'boolean'
      ? (value ? '1' : '0')
      : typeof value === 'number'
        ? (Number.isInteger(value) ? String(value) : value.toFixed(2))
        : String(value);
  return (
    <tr>
      <td>{label}</td>
      <td className="mono">{text}</td>
    </tr>
  );
}

export function LiveScreen({
  onBack,
  hardwareReady = true,
  phase = 'IDLE',
}: {
  onBack: () => void;
  hardwareReady?: boolean;
  phase?: string;
}) {
  const hipot = useHipot(hardwareReady);
  const [io, setIo] = useState<IoData | null>(null);
  const [results, setResults] = useState<ResultData>({});
  const [raw, setRaw] = useState<any>(null);
  const [zoom, setZoom] = useState<ZoomTarget | null>(null);
  const [hipotLive, setHipotLive] = useState<any>({});
  const [printer, setPrinter] = useState<any>({});
  const [last, setLast] = useState('');

  useEffect(() => {
    invoke<any>('get_config').then(c => setPrinter(c || {})).catch(() => {});
  }, []);

  useEffect(() => {
    if (!hardwareReady) return;
    let stopped = false;
    let busy = false;
    const tick = async () => {
      if (busy) return;
      busy = true;
      const [ioRes, resultRes, rawRes] = await Promise.allSettled([
        invoke<IoData>('plc_io_status'),
        invoke<ResultData>('plc_read_results'),
        invoke<any>('plc_raw_monitor'),
      ]);
      if (!stopped) {
        if (ioRes.status === 'fulfilled') setIo(ioRes.value);
        if (resultRes.status === 'fulfilled') setResults(resultRes.value || {});
        if (rawRes.status === 'fulfilled') setRaw(rawRes.value || null);
        setLast(new Date().toLocaleTimeString());
      }
      busy = false;
    };
    tick();
    const id = window.setInterval(tick, 1800);
    return () => {
      stopped = true;
      window.clearInterval(id);
    };
  }, [hardwareReady]);

  // TD? es de solo lectura. No lo consultamos durante una prueba automática
  // de Hipot para no interferir con la secuencia que ya está leyendo el equipo.
  useEffect(() => {
    if (!hardwareReady || !hipot.connected || phase === 'HIPOT_RUNNING') return;
    let stopped = false;
    let busy = false;
    const tick = async () => {
      if (busy) return;
      busy = true;
      try {
        const r = await invoke<any>('hipot_test', { action: 'STATUS' });
        if (!stopped) setHipotLive(r?.parsed ?? {});
      } catch {
        // El Hipot puede no tener un último resultado; se mantiene el anterior.
      }
      busy = false;
    };
    tick();
    const id = window.setInterval(tick, 2500);
    return () => {
      stopped = true;
      window.clearInterval(id);
    };
  }, [hardwareReady, hipot.connected, phase]);

  const inputs = io?.inputs ?? {};
  const outputs = io?.outputs ?? {};
  const analog = io?.analog ?? {};
  const activeOutputs = Object.entries(outputs).filter(([, v]) => v).map(([k]) => k);
  const activeInputs = Object.entries(inputs).filter(([, v]) => v).map(([k]) => k);
  const plcOk = io?.connected === true && io?.read_ok !== false;
  const hipotOk = hipot.connected === true;
  const plcDataReady = !!io && Object.keys(io.outputs ?? {}).length > 0;
  const resultDataReady = Object.keys(results).length > 0;
  const hipotDataReady = !!hipotLive?.raw;

  const zoomValue = !zoom
    ? 0
    : zoom.kind === 'analog'
      ? Number(analog[zoom.key] ?? 0)
      : zoom.kind === 'component'
        ? Number(results[zoom.key] ?? 0)
        : Number(hipotLive[zoom.key] ?? 0);
  const zoomUnknown = !zoom
    ? true
    : zoom.kind === 'analog'
      ? (!plcOk || !plcDataReady)
      : zoom.kind === 'component'
        ? (!plcOk || !resultDataReady)
        : (!hipotOk || !hipotDataReady);

  let zoomDetails: { label: string; value: string }[] = [];
  let zoomRelated: { label: string; value: string; unit?: string; active?: boolean }[] = [];

  if (zoom) {
    if (zoom.kind === 'analog') {
      zoomDetails = [
        { label: 'Canal', value: zoom.key },
        { label: 'Origen', value: 'PLC · entrada analógica' },
        { label: 'Rango', value: `${zoom.min} – ${zoom.max} ${zoom.unit}` },
        { label: 'Estado', value: zoomUnknown ? 'Sin lectura' : 'Lectura activa' },
      ];
      zoomRelated = ANALOG_GAUGES.map(g => ({
        label: g.label,
        value: fmtValue(analog[g.key]),
        unit: g.unit,
        active: g.key === zoom.key,
      }));
    } else if (zoom.kind === 'component') {
      zoomDetails = [
        { label: 'Dirección', value: COMPONENT_ADDRESS[zoom.key] ?? 'DB1001' },
        { label: 'Medición', value: results.running ? 'Midiendo' : results.complete ? 'Terminado' : 'En espera' },
        { label: 'Error eléctrico', value: results.error ? 'SÍ' : 'NO' },
        { label: 'Valor', value: zoomUnknown ? 'Sin lectura' : `${fmtValue(zoomValue)} A` },
      ];
      zoomRelated = COMPONENTS.map(name => ({
        label: name,
        value: fmtValue(results[name]),
        unit: 'A',
        active: name === zoom.key,
      }));
    } else {
      zoomDetails = [
        { label: 'Conexión', value: hipotOk ? 'Conectado' : 'Sin conexión' },
        { label: 'Puerto', value: hipot.status?.port ?? '—' },
        { label: 'Baudios', value: String(hipot.status?.baud ?? '—') },
        { label: 'Estado', value: hipotLive.status || (phase === 'HIPOT_RUNNING' ? 'En prueba' : '—') },
        ...(hipotLive.raw ? [{ label: 'Respuesta', value: String(hipotLive.raw) }] : []),
      ];
      zoomRelated = [
        { label: 'Voltaje', value: fmtValue(hipotLive.voltage), unit: 'V', active: zoom.key === 'voltage' },
        { label: 'Corriente', value: fmtValue(hipotLive.current), unit: 'mA', active: zoom.key === 'current' },
        { label: 'Duración', value: fmtValue(hipotLive.duration), unit: 's', active: zoom.key === 'duration' },
      ];
    }
  }

  const zoomItem: ZoomGauge | null = zoom
    ? {
        label: zoom.label,
        value: zoomValue,
        min: zoom.min,
        max: zoom.max,
        unit: zoom.unit,
        unknown: zoomUnknown,
        details: zoomDetails,
        related: zoomRelated,
      }
    : null;

  const openZoom = (target: ZoomTarget) => {
    setZoom(target);
    playGaugeTone('open');
  };
  const closeZoom = () => {
    setZoom(null);
    playGaugeTone('close');
  };

  return (
    <div className="live-page">
      <div className="live-page-head">
        <div>
          <div className="rail-eyebrow">Monitoreo general</div>
          <h2>Live</h2>
          <p>Lectura directa de instrumentos. No depende de receta ni de una prueba en curso.</p>
        </div>
        <div className="live-page-status">
          <span className="pill"><span className={`dot ${statusClass(plcOk)}`} /> PLC</span>
          <span className="pill"><span className={`dot ${statusClass(hipotOk)}`} /> Hipot</span>
          <span className="pill"><span className={`dot ${statusClass(hardwareReady)}`} /> SQL</span>
          <span className="live-updated">{last || '—'}</span>
          <button className="btn" onClick={onBack}>Volver</button>
        </div>
      </div>

      <div className="live-dashboard">
        <section className="live-block live-block-plc">
          <div className="live-block-head">
            <h3>PLC S7-1200 · Analógicas</h3>
            <span className={`pill ${plcOk ? 'on' : 'off'}`}>{plcOk ? 'En línea' : 'Sin lectura'}</span>
          </div>
          <div className="live-gauge-grid">
            {ANALOG_GAUGES.map(g => (
              <Gauge
                key={g.key}
                label={g.label}
                value={Number(analog[g.key] ?? 0)}
                min={g.min}
                max={g.max}
                unit={g.unit}
                size={112}
                unknown={!plcOk || !plcDataReady}
                flashOnChange
                onClick={() => openZoom({ kind: 'analog', key: g.key, label: g.label, min: g.min, max: g.max, unit: g.unit })}
              />
            ))}
          </div>
          <div className="live-divider" />
          <div className="live-inline-status">
            <span><b>Entradas:</b> {activeInputs.length ? activeInputs.join(' · ') : 'ninguna activa'}</span>
            <span><b>Salidas:</b> {activeOutputs.length ? activeOutputs.join(' · ') : 'ninguna activa'}</span>
          </div>
          <div className="live-io-title">I/O digital</div>
          <div className="live-chips">
            {Object.entries(inputs).map(([k, v]) => (
              <span key={k} className={`live-chip ${v ? 'on' : ''}`}>{k} {v ? '1' : '0'}</span>
            ))}
            {Object.entries(outputs).map(([k, v]) => (
              <span key={k} className={`live-chip ${v ? 'accent' : ''}`}>{k} {v ? '1' : '0'}</span>
            ))}
            {!Object.keys(inputs).length && !Object.keys(outputs).length && (
              <span className="live-empty">Sin lectura</span>
            )}
          </div>
          <div className="live-raw-title">DB1000 / DB1001</div>
          <table className="live-raw-table">
            <tbody>
              <RawRow label="DB1000.DBD4 · receta" value={raw?.DB1000?.mask} />
              <RawRow label="DB1000.DBD8 · FAN" value={raw?.DB1000?.fan_time} />
              <RawRow label="DB1000.DBD12 · DOOR" value={raw?.DB1000?.door_time} />
              <RawRow label="DB1000.DBD16 · FRAME" value={raw?.DB1000?.frame_time} />
              <RawRow label="DB1000.DBD86 · DRAIN" value={raw?.DB1000?.drain_time} />
              <RawRow label="DB1000.DBD90 · DEFROST" value={raw?.DB1000?.defrost_time} />
              <RawRow label="DB1000.DBD94 · LIGHT" value={raw?.DB1000?.light_time} />
              <RawRow label="DB1000.DBD98 · tolerancia" value={raw?.DB1000?.tolerance} />
              <RawRow label="DB1000.DBX2.5 · start" value={raw?.DB1000?.start} />
              <RawRow label="DB1000.DBX2.6 · stop" value={raw?.DB1000?.stop} />
              <RawRow label="DB1001.DBX2.1 · measuring" value={raw?.DB1001?.measuring} />
              <RawRow label="DB1001.DBX2.2 · finished" value={raw?.DB1001?.finished} />
              <RawRow label="DB1001.DBX2.4 · estop" value={raw?.DB1001?.estop} />
              <RawRow label="DB1001.DBX2.5 · error" value={raw?.DB1001?.error} />
            </tbody>
          </table>
        </section>

        <section className="live-block live-block-components">
          <div className="live-block-head">
            <h3>Consumos por componente</h3>
            <span className="pill">{results.running ? 'Midiendo' : results.complete ? 'Terminado' : 'En espera'}</span>
          </div>
          <div className="live-gauge-grid live-gauge-grid-3">
            {COMPONENTS.map(name => (
              <Gauge
                key={name}
                label={name}
                value={Number(results[name] ?? 0)}
                min={0}
                max={25}
                unit="A"
                size={118}
                unknown={!plcOk || !resultDataReady}
                flashOnChange
                onClick={() => openZoom({ kind: 'component', key: name, label: name, min: 0, max: 25, unit: 'A' })}
              />
            ))}
          </div>
          <div className="live-divider" />
          <div className="live-inline-status">
            <span><b>Error eléctrico:</b> {results.error ? 'SÍ' : 'NO'}</span>
            <span><b>Q activas:</b> {activeOutputs.length}</span>
          </div>
        </section>

        <section className="live-block live-block-hipot">
          <div className="live-block-head">
            <h3>Hipot 3865</h3>
            <span className={`pill ${hipotOk ? 'on' : 'off'}`}>{hipotOk ? 'Conectado' : 'Sin conexión'}</span>
          </div>
          <div className="live-gauge-grid live-gauge-grid-1">
            <Gauge label="Voltaje" value={Number(hipotLive.voltage ?? 0)} min={0} max={5000} unit="V" size={124} unknown={!hipotOk || !hipotDataReady} flashOnChange onClick={() => openZoom({ kind: 'hipot', key: 'voltage', label: 'Voltaje Hipot', min: 0, max: 5000, unit: 'V' })} />
            <Gauge label="Corriente" value={Number(hipotLive.current ?? 0)} min={0} max={20} unit="mA" size={124} unknown={!hipotOk || !hipotDataReady} flashOnChange onClick={() => openZoom({ kind: 'hipot', key: 'current', label: 'Corriente Hipot', min: 0, max: 20, unit: 'mA' })} />
            <Gauge label="Duración" value={Number(hipotLive.duration ?? 0)} min={0} max={60} unit="s" size={124} unknown={!hipotOk || !hipotDataReady} flashOnChange onClick={() => openZoom({ kind: 'hipot', key: 'duration', label: 'Duración Hipot', min: 0, max: 60, unit: 's' })} />
          </div>
          <div className="live-divider" />
          <Metric label="Puerto" value={hipot.status?.port ?? '—'} />
          <Metric label="Baudios" value={hipot.status?.baud ?? '—'} />
          <Metric label="Estado" value={hipotLive.status || (phase === 'HIPOT_RUNNING' ? 'En prueba' : '—')} />
          {hipotLive.raw && <div className="live-raw mono">{hipotLive.raw}</div>}
        </section>

        <section className="live-block live-block-system">
          <div className="live-system-item">
            <span className="live-system-icon">▣</span>
            <div><b>SQL Express</b><span>{hardwareReady ? 'Conectado' : 'Sin conexión'}</span></div>
            <span className={`dot ${statusClass(hardwareReady)}`} />
          </div>
          <div className="live-system-item">
            <span className="live-system-icon">▤</span>
            <div><b>Zebra</b><span>{printer.printer_name || 'Sin configurar'}</span></div>
            <span className={`dot ${printer.printer_name ? 'ok' : ''}`} />
          </div>
          <div className="live-system-item">
            <span className="live-system-icon">⌨</span>
            <div><b>Lector</b><span>USB HID keyboard-wedge</span></div>
            <span className="dot ok" />
          </div>
          <div className="live-system-note">
            Solo visualización. Los controles están en Manual y Configuración.
          </div>
        </section>
      </div>

      <GaugeZoomModal item={zoomItem} onClose={closeZoom} />
    </div>
  );
}
