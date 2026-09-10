import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { api } from '../api';

function toCSV(rows: any[]) {
  const headers = ['ID','Fecha','Modelo','Serie','Batch','PLC','Hipot','Final','Duración(s)'];
  const lines = [headers.join(';')];
  for (const r of rows) {
    lines.push([
      r.id, `"${(r.created||'').slice(0,19).replace('T',' ')}"`, r.recipe_model, r.serial, r.batch,
      r.plc ? 'PASS' : r.plc===false?'FAIL':'-', r.hipot ? 'PASS' : r.hipot===false?'FAIL':'-',
      r.final, r.total_duration ?? ''
    ].join(';'));
  }
  lines.push('');
  lines.push(`Totales;Buenas;${rows.filter(r=>r.final==='GOOD').length};Malas;${rows.filter(r=>r.final==='BAD').length};Total;${rows.length}`);
  return lines.join('\n');
}

export function HistoryPopup({ open, onClose }: { open: boolean; onClose: () => void }) {
  const today = new Date().toISOString().slice(0,10);
  const weekAgo = new Date(Date.now() - 6*86400000).toISOString().slice(0,10);
  const [start, setStart] = useState(weekAgo);
  const [end, setEnd] = useState(today);
  const [rows, setRows] = useState<any[]>([]);
  const [tot, setTot] = useState({ total:0, good:0, bad:0 });
  const [busy, setBusy] = useState(false);
  const [played, setPlayed] = useState(false);
  const [msg, setMsg] = useState('');

  const query = async () => {
    setBusy(true); setPlayed(true); setMsg('');
    setTimeout(()=>setPlayed(false), 1400);
    try {
      const r:any = await api.get(`/history?start=${start}&end=${end}&limit=500`);
      const list = Array.isArray(r) ? r : (r.rows || []);
      const t = Array.isArray(r) ? { total: list.length, good: list.filter((x:any)=>x.final==='GOOD').length, bad: list.filter((x:any)=>x.final==='BAD').length } : { total: r.total, good: r.good, bad: r.bad };
      setRows(list); setTot(t);
      if (!list.length) setMsg('Sin registros en ese rango');
    } catch(e:any){ setMsg(String(e?.message??e)); setRows([]); }
    finally{ setBusy(false); }
  };

  const exportExcel = () => {
    if (!rows.length) return;
    const csv = toCSV(rows);
    const blob = new Blob(["\uFEFF"+csv], { type: 'application/vnd.ms-excel;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `historial_${start}_a_${end}.xls`;
    document.body.appendChild(a);
    a.click();
    setTimeout(()=>{ URL.revokeObjectURL(url); a.remove(); }, 800);
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
          onClick={onClose}
          style={{ position: 'fixed', inset: 0, background: 'rgba(20,27,38,.55)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 5000, padding: 10 }}>
          <motion.div initial={{ scale: 0.97, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} exit={{ scale: 0.97, opacity: 0 }}
            onClick={e=>e.stopPropagation()}
            className="card" style={{ width: 'min(980px,98vw)', maxHeight: '90vh', display: 'flex', flexDirection: 'column', minWidth: 0, overflow: 'hidden', padding: 12 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <h3 style={{ margin: 0, flex: 1, fontSize: 16 }}>📊 Historial de pruebas</h3>
              <button className="btn sm" onClick={onClose}>Cerrar</button>
            </div>

            <div className="card" style={{ background: '#f8fafc', marginBottom: 8, padding: 10 }}>
              <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'flex-end', minWidth: 0 }}>
                <div style={{ flex: '1 1 160px' }}>
                  <label style={{ fontSize: 11, fontWeight: 700, color: '#475569' }}>Desde</label>
                  <input type="date" className="input" value={start} onChange={e=>setStart(e.target.value)} />
                </div>
                <div style={{ flex: '1 1 160px' }}>
                  <label style={{ fontSize: 11, fontWeight: 700, color: '#475569' }}>Hasta</label>
                  <input type="date" className="input" value={end} onChange={e=>setEnd(e.target.value)} />
                </div>
                <motion.button
                  animate={played ? { scale: [1, 1.08, 0.96, 1.06, 1], rotate: [0, -2, 2, -1, 0] } : {}}
                  transition={{ duration: 0.9, ease: 'easeInOut' }}
                  className="btn primary" style={{ flexShrink: 0, minWidth: 140 }} disabled={busy} onClick={query}>
                  {busy ? 'Consultando…' : '🔍 Consultar'}
                </motion.button>
                <button className="btn" style={{ flexShrink: 0 }} disabled={!rows.length} onClick={exportExcel}>📥 Excel</button>
              </div>
              <AnimatePresence>
                {played && (
                  <motion.div initial={{ opacity: 0, y: 6, scale: 0.8 }} animate={{ opacity: 1, y: 0, scale: 1 }} exit={{ opacity: 0 }}
                    style={{ marginTop: 8, display: 'flex', gap: 6, alignItems: 'center', fontSize: 12, color: '#1e293b' }}>
                    <motion.span animate={{ y: [0, -6, 0], rotate: [0, 8, -8, 0] }} transition={{ duration: 0.7, repeat: 1 }} style={{ fontSize: 20 }}>⚡</motion.span>
                    <span><b>Hipot</b> buscando en <b>{start}</b> → <b>{end}</b></span>
                    <motion.span animate={{ scale: [1, 1.3, 1] }} transition={{ duration: 0.5, repeat: 1 }} style={{ fontSize: 16 }}>🔎</motion.span>
                    <motion.span animate={{ x: [0, 6, 0] }} transition={{ duration: 0.6, repeat: 1 }}>🧪</motion.span>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            <div style={{ display: 'flex', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
              <span className="pill" style={{ background: '#eef2f6' }}>Total: <b>{tot.total}</b></span>
              <span className="pill on">Buenas: <b>{tot.good}</b></span>
              <span className="pill off">Malas: <b>{tot.bad}</b></span>
              <span className="pill accent">Tasa OK: <b>{tot.total ? Math.round(tot.good/tot.total*100) : 0}%</b></span>
              {msg && <span style={{ fontSize: 12, color: '#991b1b' }}>{msg}</span>}
            </div>

            <div style={{ flex: 1, overflow: 'auto', border: '1px solid var(--line)', borderRadius: 8, minWidth: 0 }}>
              <table className="grid" style={{ fontSize: 12, minWidth: 720 }}>
                <thead><tr><th>#</th><th>Fecha</th><th>Modelo</th><th>Serie</th><th>Batch</th><th>PLC</th><th>Hipot</th><th>Final</th><th>s</th></tr></thead>
                <tbody>
                  {rows.map((r:any)=>(
                    <tr key={r.id}>
                      <td>{r.id}</td>
                      <td style={{ whiteSpace:'nowrap' }}>{(r.created||'').slice(0,19).replace('T',' ')}</td>
                      <td><b>{r.recipe_model}</b></td>
                      <td>{r.serial}</td>
                      <td>{r.batch}</td>
                      <td>{r.plc ? 'PASS' : r.plc===false?'FAIL':'-'}</td>
                      <td>{r.hipot ? 'PASS' : r.hipot===false?'FAIL':'-'}</td>
                      <td><span className={`pill ${r.final==='GOOD'?'on':r.final==='BAD'?'off':''}`} style={{ fontSize:10 }}>{r.final}</span></td>
                      <td>{r.total_duration ? Number(r.total_duration).toFixed(1) : '-'}</td>
                    </tr>
                  ))}
                  {!rows.length && <tr><td colSpan={9} style={{ textAlign:'center', color:'#64748b', padding: 16 }}>Sin datos — elige rango y pulsa Consultar</td></tr>}
                </tbody>
              </table>
            </div>
            <div style={{ fontSize: 11, color: '#64748b', marginTop: 6 }}>El Excel incluye la tabla + totales. Rango máximo 500 registros.</div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
