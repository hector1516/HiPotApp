import React from 'react';
import { motion } from 'framer-motion';
import { BarcodeInput } from '../components/BarcodeInput';
import { usePLC } from '../hooks/usePLC';
import { useHipot } from '../hooks/useHipot';

const STEPS = ['MODELO', 'SERIE', 'BATCH', 'ELÉCTRICA', 'HIPOT', 'RESULTADO'];
const PROMPTS = ['', 'Escanee el MODELO', 'Escanee la SERIE', 'Escanee su BATCH para iniciar', '', '', ''];

function Stepper({ step }: { step: number }) {
  const done = step > 6;
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 12, minWidth: 0, flexWrap: 'wrap' }}>
      {STEPS.map((s, i) => {
        const n = i + 1;
        const cls = done || n < step ? 'done' : n === step ? 'now' : '';
        return (
          <React.Fragment key={s}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <div className={`stepdot ${cls}`} style={{ width: 38, height: 38, fontSize: 14 }}>{done || n < step ? '✓' : n}</div>
              <b style={{ fontSize: 11, letterSpacing: '.03em', color: '#334155' }}>{s}</b>
            </div>
            {i < 5 && <div style={{ flex: 1, minWidth: 8, height: 2, background: done || n < step ? '#166534' : '#cbd5e1', borderRadius: 2 }} />}
          </React.Fragment>
        );
      })}
    </div>
  );
}

function ConnRow({ label, on, detail }: { label: string; on: boolean | null; detail?: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 14, fontWeight: 600, minWidth: 0 }}>
      <span style={{ width: 8, height: 8, borderRadius: '50%', background: on ? '#166534' : on === false ? '#991b1b' : '#94a3b8', flexShrink: 0 }} />
      <span style={{ minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis' }}>{label}</span>
      {detail && <span style={{ color: '#64748b', fontWeight: 500, fontSize: 12, marginLeft: 'auto' }}>{detail}</span>}
      <span className={`pill ${on ? 'on' : on === false ? 'off' : ''}`} style={{ fontSize: 11 }}>{on ? 'OK' : on === false ? 'OFF' : '—'}</span>
    </div>
  );
}

export function HomeScreen({ step, recipe, serial, batch, error, onScanModel, onScanSerial, onScanBatch }: {
  step: number; recipe: any; serial: string; batch: string; error: string;
  onScanModel: (c: string) => void; onScanSerial: (c: string) => void; onScanBatch: (c: string) => void;
}) {
  const plc = usePLC();
  const hipot = useHipot();
  const handler = step === 1 ? onScanModel : step === 2 ? onScanSerial : onScanBatch;

  return (
    <div style={{ display: 'flex', gap: 12, padding: 12, alignItems: 'flex-start', position: 'relative', zIndex: 1, minWidth: 0 }}>
      <div style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <div className="card">
          <Stepper step={step} />
          {error && <div className="toast-err">{error}</div>}
          {step <= 3 ? (
            <>
              <div className="hero-title">{PROMPTS[step]}</div>
              <div className="scan-zone" style={{ marginTop: 6 }}>
                <BarcodeInput label="" onScan={handler} autoFocus />
              </div>
            </>
          ) : (
            <div style={{ background: '#fffbeb', border: '1px solid #fde68a', padding: 16, borderRadius: 8, textAlign: 'center' }}>
              <div style={{ fontSize: 18, fontWeight: 800, color: '#92400e' }}>Iniciando prueba…</div>
            </div>
          )}
        </div>

        {recipe && (
          <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} className="card" style={{ overflow: 'hidden' }}>
            <h3>Receta · {recipe.model}</h3>
            <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 8 }}>
              <span className="pill" style={{ background: '#1e293b', color: '#fff', borderColor: '#1e293b' }}>{recipe.model}</span>
              {serial && <span className="pill">Serie: {serial}</span>}
              {batch && <span className="pill accent">Batch: {batch}</span>}
            </div>
            <div style={{ overflowX: 'auto' }}>
              <table className="grid">
                <thead><tr><th>Componente</th><th>Voltaje</th><th>Mín</th><th>Máx</th><th>En prueba</th></tr></thead>
                <tbody>
                  {(recipe.components ?? []).map((c: any) => (
                    <tr key={c.name} style={{ opacity: c.enabled ? 1 : 0.5 }}>
                      <td><b>{c.name}</b></td><td>{c.voltage}</td>
                      <td>{c.min}</td><td>{c.max}</td>
                      <td>{c.enabled ? 'Sí' : '—'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </motion.div>
        )}
      </div>

      <div style={{ width: 280, flexShrink: 0, display: 'flex', flexDirection: 'column', gap: 12, minWidth: 0 }}>
        <div className="card">
          <h3>Equipos</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <ConnRow label="PLC S7-1200" on={plc.connected} detail={plc.detail} />
            <ConnRow label="Hipot 3865" on={hipot.connected} />
            <ConnRow label="Receta" on={recipe ? true : null} />
          </div>
        </div>
        <div className="card" style={{ fontSize: 13, color: '#475569', lineHeight: 1.5 }}>
          <b style={{ color: '#1e293b' }}>Secuencia</b>
          <div>1. Modelo → busca receta</div>
          <div>2. Serie → valida unidad</div>
          <div>3. Batch → arranca automáticamente</div>
        </div>
      </div>
    </div>
  );
}
