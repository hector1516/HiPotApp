import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { api } from '../api';
import { BarcodeInput } from './BarcodeInput';

export function ConfigGatePopup({ open, onClose, onAdmin }: { open: boolean; onClose: () => void; onAdmin: (batch:string)=>void }) {
  const [msg, setMsg] = useState('');
  const [busy, setBusy] = useState(false);

  const tryBatch = async (code: string) => {
    const batch = code.trim().toUpperCase();
    if (!batch) return;
    setBusy(true); setMsg('');
    try {
      const r:any = await api.post('/auth/login', { username: batch, password: '' });
      if (r.role === 'admin') {
        onClose();
        onAdmin(batch);
      } else {
        setMsg('⛔ Solo admin puede entrar a configuración. Este gafete es de operador.');
      }
    } catch (e:any) {
      setMsg(`❌ Gafete no encontrado o inactivo: ${batch}`);
    } finally { setBusy(false); }
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
          onClick={onClose}
          style={{ position: 'fixed', inset: 0, background: 'rgba(20,27,38,.60)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 5000, padding: 12 }}>
          <motion.div initial={{ scale: 0.97, y: 12, opacity: 0 }} animate={{ scale: 1, y: 0, opacity: 1 }} exit={{ scale: 0.97, opacity: 0 }}
            onClick={e=>e.stopPropagation()}
            className="card" style={{ width: 'min(420px,96vw)', textAlign: 'center', minWidth: 0 }}>
            <img src="logo.png" alt="" style={{ width: 180, margin: '0 auto', display: 'block' }} />
            <h3 style={{ margin: '8px 0 4px 0' }}>🔒 Configuración — escanea tu gafete</h3>
            <div style={{ fontSize: 12, color: '#64748b', marginBottom: 10 }}>Solo <b>admin</b> entra. Si es operador, se denegará.</div>
            <BarcodeInput label="" placeholder="Escanea tu BATCH / gafete aquí" onScan={tryBatch} autoFocus />
            {busy && <div style={{ fontSize: 12, color: '#64748b', marginTop: 6 }}>Verificando…</div>}
            {msg && <div className="toast-err" style={{ background: msg.startsWith('⛔')||msg.startsWith('❌') ? '#fef2f2' : '#e6f4ea', color: msg.startsWith('⛔')||msg.startsWith('❌') ? '#991b1b' : '#166534', marginTop: 8 }}>{msg}</div>}
            <div style={{ display: 'flex', gap: 8, justifyContent: 'center', marginTop: 12 }}>
              <button className="btn" onClick={onClose}>Cancelar</button>
              <span style={{ fontSize: 11, color: '#94a3b8', alignSelf: 'center' }}>GodMode: Ctrl+Shift+K</span>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
