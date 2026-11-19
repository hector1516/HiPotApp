import { motion } from 'framer-motion';

// Alarma de paro de emergencia: cubre toda la app, roja, animada y con
// flecha inferior apuntando al botón físico.
export function EmergencyStopOverlay() {
  return (
    <motion.div
      className="estop-overlay"
      role="alertdialog"
      aria-live="assertive"
      aria-label="Paro de emergencia activado"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
    >
      <div className="estop-pulse" />

      <motion.div
        className="estop-button"
        animate={{ scale: [1, 1.045, 1] }}
        transition={{ duration: 0.9, repeat: Infinity, ease: 'easeInOut' }}
      >
        <svg viewBox="0 0 220 220" width="100%" height="100%" aria-hidden>
          <ellipse cx="110" cy="196" rx="78" ry="16" fill="rgba(0,0,0,.26)" />
          <rect x="78" y="142" width="64" height="48" rx="10" fill="#7f1d1d" />
          <rect x="88" y="132" width="44" height="18" rx="6" fill="#991b1b" />
          <circle cx="110" cy="104" r="72" fill="#dc2626" stroke="#7f1d1d" strokeWidth="10" />
          <circle cx="110" cy="104" r="56" fill="#b91c1c" />
          <text
            x="110"
            y="113"
            textAnchor="middle"
            fontFamily="Segoe UI, sans-serif"
            fontSize="22"
            fontWeight="900"
            fill="#fff"
            letterSpacing="2"
          >
            STOP
          </text>
        </svg>
      </motion.div>

      <div className="estop-title">PARO DE EMERGENCIA</div>
      <div className="estop-sub">El botón físico está presionado</div>
      <div className="estop-hint">Suelte / libere el botón para continuar</div>

      <div className="estop-arrow">
        <span>BOTÓN FÍSICO ABAJO</span>
        <motion.svg
          viewBox="0 0 64 64"
          width="64"
          height="64"
          aria-hidden
          animate={{ y: [0, 12, 0] }}
          transition={{ duration: 0.9, repeat: Infinity, ease: 'easeInOut' }}
        >
          <path d="M32 8v38" stroke="#fff" strokeWidth="8" strokeLinecap="round" />
          <path d="M14 34l18 18 18-18" fill="none" stroke="#fff" strokeWidth="8" strokeLinecap="round" strokeLinejoin="round" />
        </motion.svg>
      </div>
    </motion.div>
  );
}
