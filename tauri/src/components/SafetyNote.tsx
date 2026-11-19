import React from 'react';

const DEFAULT_TEXT =
  'Un Hipot aplica alto voltaje (cientos o miles de voltios). Puede ser letal. Solo debe operarlo personal capacitado, con tierra física, cables en buen estado, área despejada y sin tocar las puntas durante la prueba. No anule interlocks ni seguridades del equipo.';

function WarningIcon() {
  return (
    <svg width="17" height="17" viewBox="0 0 24 24" fill="none" aria-hidden style={{ flexShrink: 0 }}>
      <path d="M12 3 2.5 20.5h19L12 3Z" stroke="currentColor" strokeWidth="1.8" strokeLinejoin="round" />
      <path d="M12 9v4.5M12 17.2h.01" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}

export function SafetyNote({ compact = false, text }: { compact?: boolean; text?: string }) {
  const message = text ?? DEFAULT_TEXT;
  return (
    <div className={compact ? 'safety-note' : 'safety-strip'} role="note" aria-label="Advertencia de seguridad de alto voltaje">
      <WarningIcon />
      <div>
        <b>Advertencia de seguridad</b>
        <span>{message}</span>
      </div>
    </div>
  );
}
