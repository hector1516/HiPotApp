import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { api } from '../api';

export function LogPopup({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [logs, setLogs] = useState<any[]>([]);
  const [q, setQ] = useState('');
  const [cat, setCat] = useState('');
  const [level, setLevel] = useState('');
  const load = async () => {
    try {
      const r = await api.get(`/admin/logs?category=${encodeURIComponent(cat)}&level=${encodeURIComponent(level)}&q=${encodeURIComponent(q)}&limit=120`);
      setLogs(Array.isArray(r) ? r : []);
    } catch { setLogs([]); }
  };
  useEffect(() => { if (open) load(); }, [open]);

  return (
    <AnimatePresence>
      {open && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
          onClick={onClose}
          style={{ position: 'fixed', inset: 0, background: 'rgba(20,27,38,.55)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 5000, padding: 12 }}>
          <motion.div initial={{ scale: 0.96, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} exit={{ scale: 0.96, opacity: 0 }}
            onClick={e => e.stopPropagation()}
            className="card" style={{ width: 'min(860px,96vw)', maxHeight: '88vh', display: 'flex', flexDirection: 'column', minWidth: 0, overflow: 'hidden' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <h3 style={{ margin: 0, flex: 1 }}>📋 Log de la app</h3>
              <button className="btn sm" onClick={onClose}>Cerrar</button>
            </div>
            <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 8, minWidth: 0 }}>
              <select className="input" style={{ maxWidth: 140, flex: '1 1 120px' }} value={cat} onChange={e => setCat(e.target.value)}>
                <option value="">Todas CAT</option>{['UI','PLC','HIPOT','DB','SYNC','BARCODE','SYSTEM'].map(c=><option key={c} value={c}>{c}</option>)}
              </select>
              <select className="input" style={{ maxWidth: 120, flex: '1 1 100px' }} value={level} onChange={e => setLevel(e.target.value)}>
                <option value="">Todos</option>{['DEBUG','INFO','WARN','ERROR'].map(c=><option key={c} value={c}>{c}</option>)}
              </select>
              <input className="input" placeholder="Buscar…" value={q} onChange={e => setQ(e.target.value)} style={{ flex: '1 1 160px', minWidth: 0 }} />
              <button className="btn primary" style={{ flexShrink: 0 }} onClick={load}>Actualizar</button>
            </div>
            <div style={{ flex: 1, overflow: 'auto', border: '1px solid var(--line)', borderRadius: 8, minWidth: 0 }}>
              <table className="grid" style={{ fontSize: 12 }}>
                <thead><tr><th>#</th><th>Fecha</th><th>Nivel</th><th>Cat</th><th>Detalle</th></tr></thead>
                <tbody>
                  {logs.map((l: any) => (
                    <tr key={l.id}><td>{l.id}</td><td style={{ whiteSpace: 'nowrap' }}>{(l.ts || '').slice(11,19)}</td>
                      <td><span className={`pill ${l.level==='ERROR'?'off':l.level==='WARN'?'warn':''}`} style={{ fontSize: 10 }}>{l.level}</span></td>
                      <td>{l.cat}</td><td style={{ wordBreak: 'break-word' }}>{l.detail}</td></tr>
                  ))}
                  {!logs.length && <tr><td colSpan={5} style={{ textAlign:'center', color:'#64748b' }}>Sin registros</td></tr>}
                </tbody>
              </table>
            </div>
            <div style={{ fontSize: 11, color: '#64748b', marginTop: 6 }}>Vista rápida — el histórico completo está en CONFIG → Logs.</div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
