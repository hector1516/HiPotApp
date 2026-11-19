import React from 'react';
import { motion } from 'framer-motion';

// Layout base: panel contextual a la izquierda + contenido a la derecha.
// Se usa en todas las pantallas para mantener la misma jerarquía visual.
export function SplitLayout({
  aside,
  children,
  compact = false,
  className = '',
}: {
  aside: React.ReactNode;
  children: React.ReactNode;
  compact?: boolean;
  className?: string;
}) {
  return (
    <div className={`split ${className}`}>
      <motion.aside
        className={`split-aside ${compact ? 'compact' : ''}`}
        initial={{ x: -28, opacity: 0 }}
        animate={{ x: 0, opacity: 1 }}
        transition={{ duration: 0.38, ease: [0.22, 1, 0.36, 1] }}
      >
        {aside}
      </motion.aside>
      <motion.main
        className="split-main"
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.38, delay: 0.05, ease: [0.22, 1, 0.36, 1] }}
      >
        {children}
      </motion.main>
    </div>
  );
}
