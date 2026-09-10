import React from 'react';
import { createRoot } from 'react-dom/client';
import { motion } from 'framer-motion';

// Splash transparente y puro visual: Rust muestra el main y cierra
// esta ventana a los ~2.6 s. Sin lógica de ventanas aquí.
function Splash() {
  return (
    <div
      style={{
        width: '100vw', height: '100vh', display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center',
        background: 'transparent', backgroundColor: 'transparent',
        fontFamily: '"Segoe UI", system-ui, sans-serif',
      }}
    >
      <motion.img
        src="logo.png"
        alt="HiPot Tester"
        initial={{ opacity: 0, scale: 0.85, y: 18 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        transition={{ duration: 0.9, ease: 'easeOut' }}
        style={{ width: 400, filter: 'drop-shadow(0 18px 34px rgba(10,15,25,.45))' }}
      />
      <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.5 }}
        style={{ marginTop: 10, textAlign: 'center' }}>
        <div style={{ fontSize: 34, fontWeight: 900, color: '#141b26', letterSpacing: 2 }}>
          HiPot<span style={{ color: '#e8930c' }}>App</span>
        </div>
        <div style={{ fontSize: 14, fontWeight: 700, color: '#5d666f', letterSpacing: 4 }}>
          ESTACIÓN DE PRUEBAS
        </div>
      </motion.div>
      <div style={{ width: 300, height: 8, borderRadius: 999, background: 'rgba(20,27,38,.15)',
                    marginTop: 22, overflow: 'hidden' }}>
        <motion.div
          initial={{ x: '-110%' }} animate={{ x: '320%' }}
          transition={{ duration: 1.3, repeat: Infinity, ease: 'easeInOut' }}
          style={{ width: '32%', height: '100%', borderRadius: 999,
                   background: 'linear-gradient(90deg,#e8930c,#ffcf6e)' }}
        />
      </div>
    </div>
  );
}

createRoot(document.getElementById('splash')!).render(<Splash />);
