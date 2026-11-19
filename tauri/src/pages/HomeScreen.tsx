import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { BarcodeInput } from '../components/BarcodeInput';
import { SplitLayout } from '../components/SplitLayout';
import { PageHeader } from '../components/PageHeader';
import { usePLC } from '../hooks/usePLC';
import { useHipot } from '../hooks/useHipot';
import { AUTO_STEPS, stepAt } from '../config/autoSteps';

function RailStep({ n, label, state }: { n: number; label: string; state: 'done' | 'now' | 'todo' }) {
  return (
    <div className={`rail-step ${state}`}>
      <span className="num">{state === 'done' ? '✓' : n}</span>
      <span>{label}</span>
    </div>
  );
}

function EquipmentRow({ label, on, detail }: { label: string; on: boolean | null; detail?: string }) {
  return (
    <div className="rail-status">
      <span className={`dot ${on ? 'ok' : on === false ? 'bad' : ''}`} />
      <span style={{ flex: 1, minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis' }}>{label}</span>
      {detail && <span style={{ color: 'var(--on-rail-muted)', fontSize: 11 }}>{detail}</span>}
      <span className="pill dark" style={{ fontSize: 10 }}>{on ? 'OK' : on === false ? 'OFF' : '—'}</span>
    </div>
  );
}

export function HomeScreen({
  step, recipe, models, selectedModel, serial, batch, error,
  onSelectModel, onChangeModel, onScanSerial, onScanBatch,
}: {
  step: number;
  recipe: any;
  models: any[];
  selectedModel: string;
  serial: string;
  batch: string;
  error: string;
  onSelectModel: (model: string) => void;
  onChangeModel: () => void;
  onScanSerial: (code: string) => void;
  onScanBatch: (code: string) => void;
}) {
  const plc = usePLC();
  const hipot = useHipot();
  const [modelChoice, setModelChoice] = useState(selectedModel || models[0]?.model || '');
  const prompt = stepAt(step - 1).prompt;

  useEffect(() => {
    if (selectedModel && models.some((m: any) => m.model === selectedModel)) setModelChoice(selectedModel);
  }, [selectedModel, models]);

  useEffect(() => {
    if ((!modelChoice || !models.some((m: any) => m.model === modelChoice)) && models[0]) {
      setModelChoice(models[0].model);
    }
  }, [models]);

  const aside = (
    <>
      <div>
        <div className="rail-eyebrow">Estación de pruebas</div>
        <h2 className="rail-title">Nueva unidad</h2>
        <p className="rail-sub">
          Selecciona el modelo, escanea serie y batch. La prueba inicia automáticamente con el batch.
        </p>
      </div>

      <div className="rail-divider" />

      <div className="rail-steps">
        {AUTO_STEPS.map((s, i) => {
          const n = i + 1;
          const state = n < step ? 'done' : n === step ? 'now' : 'todo';
          return <RailStep key={s.id} n={n} label={s.label} state={state} />;
        })}
      </div>

      <div className="rail-divider" />

      <div className="rail-kpi">
        <span className="label">Modelo</span>
        <span className="value">{recipe?.model ?? selectedModel ?? '—'}</span>
      </div>
      {step > 1 && (
        <button className="btn sm" style={{ width: '100%' }} onClick={onChangeModel}>
          Cambiar modelo
        </button>
      )}
      <div className="rail-kpi">
        <span className="label">Serie</span>
        <span className="value">{serial || '—'}</span>
      </div>
      <div className="rail-kpi">
        <span className="label">Batch</span>
        <span className="value">{batch || '—'}</span>
      </div>

      <div className="rail-divider" />

      <div className="stack" style={{ gap: 8 }}>
        <div className="rail-eyebrow">Equipos</div>
        <EquipmentRow label="PLC S7-1200" on={plc.connected} detail={plc.detail} />
        <EquipmentRow label="Hipot 3865" on={hipot.connected} />
        <EquipmentRow label="Receta" on={recipe ? true : null} />
      </div>

      <div className="image-card">
        <img src="/IMG_0854.png" alt="Estación de pruebas HiPot" />
      </div>

      <div className="rail-actions">
        <div className="rail-status" style={{ fontSize: 11, color: 'var(--on-rail-muted)' }}>
          <span className="dot ok" />
          <span>Los datos se guardan localmente</span>
        </div>
      </div>
    </>
  );

  return (
    <SplitLayout aside={aside}>
      <PageHeader
        eyebrow={`Paso ${Math.min(step, AUTO_STEPS.length)} de ${AUTO_STEPS.length}`}
        title={prompt}
        subtitle={
          step === 1
            ? 'Elige el modelo de la lista de recetas. Se recordará el último usado.'
            : 'El lector es keyboard-wedge: escanea o usa el teclado en pantalla.'
        }
      />

      {error && <div className="alert">{error}</div>}

      {step === 1 ? (
        <motion.div className="card" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
          <div className="field">
            <label>Modelo / receta</label>
            <select
              className="input"
              value={modelChoice}
              onChange={e => setModelChoice(e.target.value)}
              disabled={!models.length}
            >
              {models.map((m: any) => (
                <option key={m.id} value={m.model}>{m.model} — {m.name}</option>
              ))}
              {!models.length && <option value="">Sin recetas</option>}
            </select>
          </div>
          <button
            className="btn primary lg"
            style={{ marginTop: 14 }}
            disabled={!modelChoice}
            onClick={() => onSelectModel(modelChoice)}
          >
            Continuar con {modelChoice || 'modelo'}
          </button>
          {!models.length && (
            <div className="alert warn" style={{ marginTop: 14 }}>
              No hay recetas dadas de alta. Cree una en Configuración → Recetas.
            </div>
          )}
        </motion.div>
      ) : step <= 3 ? (
        <motion.div className="card" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
          <div className="scan-zone">
            <BarcodeInput
              label=""
              onScan={step === 2 ? onScanSerial : onScanBatch}
              autoFocus
            />
          </div>
          <div className="row" style={{ marginTop: 14, color: 'var(--muted)', fontSize: 12 }}>
            <span className="pill">Enter para confirmar</span>
            <span className="pill">El código se normaliza a MAYÚSCULAS</span>
          </div>
        </motion.div>
      ) : (
        <div className="alert warn">
          <span>Iniciando prueba…</span>
        </div>
      )}

      {recipe && (
        <motion.div
          className="card"
          style={{ marginTop: 16 }}
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.35, delay: 0.05 }}
        >
          <div className="row" style={{ justifyContent: 'space-between', marginBottom: 10 }}>
            <h3 style={{ margin: 0 }}>Receta · {recipe.model}</h3>
            <div className="row">
              <span className="pill accent">{recipe.model}</span>
              {serial && <span className="pill">Serie: {serial}</span>}
              {batch && <span className="pill">Batch: {batch}</span>}
            </div>
          </div>
          <div className="row" style={{ marginBottom: 10 }}>
            {(recipe.hipot_voltage ?? 0) > 0 && (recipe.hipot_current_max ?? 0) > 0 ? (
              <>
                <span className="pill warn">Hipot: {recipe.hipot_voltage} V</span>
                <span className="pill">I máx: {recipe.hipot_current_max} mA</span>
                <span className="pill">Tiempo: {recipe.hipot_time ?? 10} s</span>
              </>
            ) : (
              <span className="pill warn">Hipot sin configurar</span>
            )}
          </div>
          <div className="table-wrap">
            <table className="grid">
              <thead>
                <tr><th>Componente</th><th>Voltaje</th><th>Mín</th><th>Máx</th><th>En prueba</th></tr>
              </thead>
              <tbody>
                {(recipe.components ?? []).map((c: any) => (
                  <tr key={c.name} style={{ opacity: c.enabled ? 1 : 0.5 }}>
                    <td><b>{c.name}</b></td>
                    <td>{c.voltage}</td>
                    <td className="mono">{c.min}</td>
                    <td className="mono">{c.max}</td>
                    <td>{c.enabled ? <span className="pill on">Sí</span> : <span className="pill">—</span>}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </motion.div>
      )}
    </SplitLayout>
  );
}
