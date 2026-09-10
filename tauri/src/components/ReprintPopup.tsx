import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { api } from '../api';

export function ReprintPopup({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [list, setList] = useState<any[]>([]);
  const [sel, setSel] = useState<string>('');
  const [msg, setMsg] = useState('');
  const [busy, setBusy] = useState(false);

  const load = async () => {
    setMsg('');
    try {
      const r:any = await api.get('/history?limit=50');
      const arr = Array.isArray(r) ? r : (r.rows || []);
      const goods = arr.filter((x: any) => x.final === 'GOOD').slice(0, 20);
      setList(goods);
      if (goods[0] && !sel) setSel(String(goods[0].id));
      if (!goods.length) setMsg('Sin registros BUENOS aún');
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };
  useEffect(() => { if (open) load(); }, [open]);

  const chosen = list.find(x => String(x.id) === sel);

  const doPrint = async () => {
    if (!sel) return;
    setBusy(true); setMsg('');
    try {
      await api.post(`/executions/${sel}/reprint`);
      setMsg('✅ Etiqueta enviada a Zebra ZT231');
    } catch (e: any) { setMsg(`❌ ${e?.message ?? e}`); }
    finally { setBusy(false); }
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
          onClick={onClose}
          style={{ position: 'fixed', inset: 0, background: 'rgba(20,27,38,.55)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 5000, padding: 12 }}>
          <motion.div initial={{ scale: 0.96, opacity: 0 }} animate={{ scale: 1, opacity: 1 }} exit={{ scale: 0.96, opacity: 0 }}
            onClick={e => e.stopPropagation()}
            className="card" style={{ width: 'min(560px,96vw)', maxHeight: '88vh', overflow: 'auto', minWidth: 0 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10 }}>
              <h3 style={{ margin: 0, flex: 1 }}>🖨️ Reimprimir etiqueta</h3>
              <button className="btn sm" onClick={onClose}>Cerrar</button>
            </div>
            <div style={{ fontSize: 12, color: '#64748b', marginBottom: 8 }}>Últimos 20 registros con resultado <b style={{ color: '#166534' }}>BUENO</b> — seleccione y reimprima.</div>
            <select className="input" value={sel} onChange={e => setSel(e.target.value)} style={{ marginBottom: 10 }}>
              {list.map((x: any) => (
                <option key={x.id} value={String(x.id)}>#{x.id} · {x.serial || '—'} · {x.final} · {x.created?.slice(0, 19).replace('T',' ')}</option>
              ))}
              {!list.length && <option value="">— sin datos —</option>}
            </select>
            {chosen && (
              <div style={{ background: '#f8fafc', border: '1px solid var(--line)', borderRadius: 8, padding: 8, fontSize: 12, marginBottom: 10 }}>
                <div><b>Serie:</b> {chosen.serial || '—'} · <b>Final:</b> {chosen.final}</div>
                <div style={{ color: '#64748b' }}>Creado: {chosen.created?.slice(0, 19).replace('T',' ')}</div>
              </div>
            )}
            {msg && <div className="toast-err" style={{ background: msg.startsWith('✅') ? '#e6f4ea' : undefined, borderColor: msg.startsWith('✅') ? '#a8d0b4' : undefined, color: msg.startsWith('✅') ? '#166534' : undefined }}>{msg}</div>}
            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 8, flexWrap: 'wrap' }}>
              <button className="btn" onClick={onClose}>Cancelar</button>
              <button className="btn primary" disabled={!sel || busy} onClick={doPrint}>{busy ? 'Enviando…' : '🖨️ Reimprimir'}</button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
