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
      <div style={{ display: 'flex', gap: 8, minWidth: 0, alignItems: 'center' }}>
        <button
          type="button"
          title="Captura manual (teclado en pantalla)"
          aria-label="Captura manual de codigo de barras"
          onClick={() => {
            ref.current?.focus();
            (window as any).__TAURI__?.core?.invoke?.('show_touch_keyboard')?.catch?.(() => {});
          }}
          style={{
            flexShrink: 0, width: 36, height: 36, borderRadius: 8,
            border: '1px solid #cbd5e1', background: '#f1f5f9', cursor: 'pointer',
            fontSize: 16, display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}
        >⌨</button>
        <input
          ref={ref}
          value={v}
          autoFocus={autoFocus}
          inputMode="text"
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
