import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useUpdater } from '../hooks/useUpdater';

// Banner superior de update (doc 10).
export function UpdateNotification() {
  const { available, downloading, progress, updateInfo, error, downloadAndInstall } = useUpdater();
  const [gone, setGone] = useState(false);
  if (gone || !available) return null;
  return (
    <AnimatePresence>
      <motion.div initial={{ y: -100, opacity: 0 }} animate={{ y: 0, opacity: 1 }} exit={{ y: -100, opacity: 0 }}
        style={{ position: 'fixed', top: 0, left: 0, right: 0, zIndex: 9999,
                 background: 'linear-gradient(135deg,#1a237e,#283593)', color: '#fff',
                 padding: '12px 20px', display: 'flex', alignItems: 'center', gap: 16 }}>
        <div style={{ flex: 1 }}>
          <b>Nueva versión disponible: v{updateInfo?.version}</b>
          {downloading && <div>Descargando... {progress}%</div>}
          {error && <div style={{ color: '#ff8a80' }}>Error: {error}</div>}
        </div>
        {!downloading && (
          <>
            <button onClick={downloadAndInstall}
              style={{ background: '#4caf50', color: '#fff', border: 'none', borderRadius: 6,
                       padding: '10px 20px', minWidth: 120, fontWeight: 800 }}>Actualizar</button>
            <button onClick={() => setGone(true)}
              style={{ background: 'transparent', color: '#fff', border: '1px solid #fff',
                       borderRadius: 6, padding: '10px 16px' }}>Después</button>
          </>
        )}
      </motion.div>
    </AnimatePresence>
  );
}
