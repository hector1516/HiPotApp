import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

function Dot({ on }: { on: boolean | null }) {
  return <span className={`dot ${on === true ? 'ok' : on === false ? 'bad' : ''}`} />;
}

function KV({ label, value, mono = false }: { label: string; value: React.ReactNode; mono?: boolean }) {
  return (
    <div className="live-kv">
      <span>{label}</span>
      <b className={mono ? 'mono' : undefined}>{value}</b>
    </div>
  );
}

/**
 * Panel de lectura rápida para modo manual.
 * No escribe nada: solo consulta PLC (IO + resultados) y el estado del Hipot.
 */
export function LiveHardwarePanel({
  hardwareReady = true,
  hipotStatus,
}: {
  hardwareReady?: boolean;
  hipotStatus?: any;
}) {
  const [io, setIo] = useState<any>(null);
  const [results, setResults] = useState<any>(null);
  const [error, setError] = useState('');
  const [last, setLast] = useState('');

  useEffect(() => {
    if (!hardwareReady) {
      setIo(null);
      setResults(null);
      return;
    }
    let stopped = false;
    let busy = false;
    const tick = async () => {
      if (busy) return;
      busy = true;
      const [ioRes, resultRes] = await Promise.allSettled([
        invoke<any>('plc_io_status'),
        invoke<any>('plc_read_results'),
      ]);
      if (!stopped) {
        if (ioRes.status === 'fulfilled') setIo(ioRes.value);
        if (resultRes.status === 'fulfilled') setResults(resultRes.value);
        const msg = [
          ioRes.status === 'fulfilled' ? ioRes.value?.error : '',
          resultRes.status === 'fulfilled' ? resultRes.value?.error : '',
        ].filter(Boolean).join(' · ');
        setError(msg);
        setLast(new Date().toLocaleTimeString());
      }
      busy = false;
    };
    tick();
    const id = window.setInterval(tick, 1500);
    return () => {
      stopped = true;
      window.clearInterval(id);
    };
  }, [hardwareReady]);

  const outputs = (io?.outputs ?? {}) as Record<string, boolean>;
  const inputs = (io?.inputs ?? {}) as Record<string, boolean>;
  const analog = (io?.analog ?? {}) as Record<string, number>;
  const plcConnected = io?.connected ?? null;

  const hipotConnected = hipotStatus?.connected ?? null;
  const hipotEnabled = hipotStatus?.enabled;

  return (
    <div className="card live-panel">
      <div className="live-head">
        <div>
          <div className="rail-eyebrow" style={{ margin: 0 }}>Lectura directa</div>
          <h3 style={{ margin: '2px 0 0 0' }}>Parámetros en vivo</h3>
        </div>
        <div className="live-head-status">
          <span className="pill dark"><Dot on={plcConnected} /> PLC</span>
          <span className="pill dark"><Dot on={hipotConnected} /> Hipot</span>
          <span className="live-updated">{last || '—'}</span>
        </div>
      </div>

      {!hardwareReady && (
        <div className="alert warn" style={{ marginTop: 8 }}>
          La base de datos todavía no está lista; las lecturas de hardware se activan al conectar.
        </div>
      )}
      {error && <div className="alert" style={{ marginTop: 8, padding: '6px 10px' }}>{error}</div>}

      <div className="live-grid">
        <section className="live-section">
          <h4>PLC · Entradas / analógicas</h4>
          <div className="live-chips">
            {Object.entries(inputs).map(([k, v]) => (
              <span key={k} className={`live-chip ${v ? 'on' : ''}`}>{k} {v ? '1' : '0'}</span>
            ))}
            {Object.entries(analog).map(([k, v]) => (
              <span key={k} className="live-chip mono">{k} {Number(v).toFixed(2)}</span>
            ))}
            {!Object.keys(inputs).length && !Object.keys(analog).length && (
              <span className="live-empty">Sin lectura</span>
            )}
          </div>
          <h4 style={{ marginTop: 8 }}>Salidas Q</h4>
          <div className="live-chips">
            {Object.entries(outputs).map(([k, v]) => (
              <span key={k} className={`live-chip ${v ? 'accent' : ''}`}>{k} {v ? '1' : '0'}</span>
            ))}
            {!Object.keys(outputs).length && <span className="live-empty">Sin lectura</span>}
          </div>
        </section>

        <section className="live-section">
          <h4>PLC · Resultados</h4>
          <KV label="Estado" value={
            results?.running ? 'Midiendo' : results?.complete ? 'Terminado' : 'En espera'
          } />
          <KV label="Error" value={results?.error ? 'SÍ' : 'NO'} />
          {['FAN', 'DOOR', 'FRAME', 'DRAIN', 'DEFROST', 'LIGHT'].map(name => (
            <KV key={name} label={name} value={Number(results?.[name] ?? 0).toFixed(2)} mono />
          ))}
        </section>

        <section className="live-section">
          <h4>Hipot 3865</h4>
          <KV label="Conexión" value={hipotConnected === true ? 'Conectado' : hipotConnected === false ? 'Sin conexión' : '—'} />
          <KV label="Habilitado" value={hipotEnabled === false ? 'NO' : hipotEnabled === true ? 'SÍ' : '—'} />
          <KV label="Puerto" value={hipotStatus?.port ?? '—'} mono />
          <KV label="Baudios" value={hipotStatus?.baud ?? '—'} mono />
          {hipotStatus?.error && (
            <div className="live-error">{hipotStatus.error}</div>
          )}
          {!hipotStatus?.error && !hipotConnected && (
            <div className="live-empty">Hipot no habilitado o sin lectura.</div>
          )}
        </section>
      </div>
    </div>
  );
}
