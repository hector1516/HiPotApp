import React, { useRef, useState } from 'react';

// Input barcode — sobrio, sin borde azul saturado.
export function BarcodeInput({ label, placeholder, onScan, autoFocus }: {
  label: string; placeholder?: string;
  onScan: (code: string) => void; autoFocus?: boolean;
}) {
  const [v, setV] = useState('');
  const ref = useRef<HTMLInputElement>(null);
  const submit = () => {
    const code = v.trim().toUpperCase();
    if (!code) return;
    onScan(code);
    setV('');
    ref.current?.focus();
  };
  return (
    <div style={{ minWidth: 0 }}>
      {label && <div style={{ fontSize: 13, fontWeight: 700, marginBottom: 6, color: '#475569' }}>{label}</div>}
      <div style={{ display: 'flex', gap: 8, minWidth: 0 }}>
        <input
          ref={ref}
          value={v}
          autoFocus={autoFocus}
          onChange={e => setV(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter') submit(); }}
          placeholder={placeholder ?? 'Escanee o escriba…'}
          className="input"
          style={{ flex: 1, minWidth: 0, textAlign: 'center', textTransform: 'uppercase', letterSpacing: '.04em' }}
        />
        <button onClick={submit} className="btn primary" style={{ flexShrink: 0 }}>OK</button>
      </div>
    </div>
  );
}
