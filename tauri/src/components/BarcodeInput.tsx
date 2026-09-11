import React, { useEffect, useRef, useState } from 'react';

// Input barcode — enfoque bloqueado permanente para escáner industrial.
export function BarcodeInput({ label, placeholder, onScan, autoFocus = true }: {
  label?: string; placeholder?: string;
  onScan: (code: string) => void; autoFocus?: boolean;
}) {
  const [v, setV] = useState('');
  const ref = useRef<HTMLInputElement>(null);

  const focusInput = () => {
    if (ref.current && document.activeElement !== ref.current) {
      ref.current.focus();
    }
  };

  useEffect(() => {
    if (!autoFocus) return;

    // Enfoque inicial
    focusInput();

    // 1. Re-enfocar automáticamente si pierde el foco (blur)
    const handleBlur = () => {
      setTimeout(focusInput, 10);
    };

    // 2. Al hacer clic o tocar cualquier parte de la pantalla, mantener el foco en el escáner
    const handleGlobalClick = (e: MouseEvent | TouchEvent) => {
      // Si el objetivo no es un botón explicito, mantener el foco en el input del escáner
      const target = e.target as HTMLElement | null;
      if (!target || (!target.closest('button') && !target.closest('select'))) {
        focusInput();
      }
    };

    // 3. Captura global de teclas (por si el foco se perdió temporalmente antes de escanear)
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.altKey || e.metaKey) return;
      if (document.activeElement !== ref.current && ref.current) {
        ref.current.focus();
      }
    };

    const el = ref.current;
    if (el) el.addEventListener('blur', handleBlur);
    window.addEventListener('pointerdown', handleGlobalClick);
    window.addEventListener('keydown', handleGlobalKeyDown);

    return () => {
      if (el) el.removeEventListener('blur', handleBlur);
      window.removeEventListener('pointerdown', handleGlobalClick);
      window.removeEventListener('keydown', handleGlobalKeyDown);
    };
  }, [autoFocus]);

  const submit = () => {
    const code = v.trim().toUpperCase();
    if (!code) return;
    onScan(code);
    setV('');
    focusInput();
  };

  return (
    <div style={{ minWidth: 0, width: '100%' }}>
      {label && <div style={{ fontSize: 14, fontWeight: 700, marginBottom: 6, color: '#475569' }}>{label}</div>}
      <div style={{ display: 'flex', gap: 10, minWidth: 0, width: '100%' }}>
        <input
          ref={ref}
          value={v}
          autoFocus={autoFocus}
          onChange={e => setV(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter') submit(); }}
          placeholder={placeholder ?? '📷 Escanee código de barras aquí…'}
          className="input"
          style={{
            flex: 1,
            minWidth: 0,
            height: 56,
            fontSize: 20,
            fontWeight: 700,
            textAlign: 'center',
            textTransform: 'uppercase',
            letterSpacing: '.06em',
            border: '2px solid #b7790f',
            borderRadius: 8,
            boxShadow: '0 0 0 3px rgba(183,121,15,.18)',
            background: '#ffffff',
          }}
        />
        <button onClick={submit} className="btn accent" style={{ flexShrink: 0, minWidth: 100, height: 56, fontSize: 16 }}>
          OK ↵
        </button>
      </div>
    </div>
  );
}

