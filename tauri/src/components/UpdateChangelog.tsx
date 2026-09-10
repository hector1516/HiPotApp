import React from 'react';
import { motion } from 'framer-motion';

// Modal changelog post-update (doc 10).
export function UpdateChangelog({ version, notes, date, onClose }: {
  version: string; notes: string; date: string; onClose: () => void;
}) {
  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }}
      onClick={onClose}
      style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,.7)',
               display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 10000 }}>
      <div onClick={e => e.stopPropagation()}
        style={{ background: '#fff', borderRadius: 16, padding: 32, maxWidth: 500, width: '90%' }}>
        <h2 style={{ color: '#1a237e' }}>HiPotApp v{version}</h2>
        <div style={{ color: '#666', fontSize: 14 }}>{new Date(date).toLocaleDateString('es-MX')}</div>
        <pre style={{ background: '#f5f5f5', padding: 16, borderRadius: 8, whiteSpace: 'pre-wrap' }}>
          {notes || 'Sin notas.'}
        </pre>
        <button onClick={onClose}
          style={{ background: '#1a237e', color: '#fff', border: 'none', borderRadius: 8,
                   padding: '12px 32px', fontWeight: 800, minHeight: 56 }}>Entendido</button>
      </div>
    </motion.div>
  );
}
