import React, { useRef, useState } from 'react';

export function BarcodeInput({ label, placeholder, onScan, autoFocus }: {
  label: string;
  placeholder?: string;
  onScan: (code: string) => void;
  autoFocus?: boolean;
}) {
  const [v, setV] = useState('');
  const [flash, setFlash] = useState(false);
  const ref = useRef<HTMLInputElement>(null);

  const submit = () => {
    const code = v.trim().toUpperCase();
    if (!code) return;
    onScan(code);
    setV('');
    setFlash(true);
    window.setTimeout(() => setFlash(false), 600);
    ref.current?.focus();
  };

  return (
    <div style={{ minWidth: 0 }}>
      {label && <div className="field" style={{ marginBottom: 6 }}><label>{label}</label></div>}
      <div style={{ display: 'flex', gap: 8, minWidth: 0, alignItems: 'center' }}>
        <button
          type="button"
          className="btn icon"
          title="Captura manual (teclado en pantalla)"
          aria-label="Captura manual de código de barras"
          onClick={() => {
            ref.current?.focus();
            window.dispatchEvent(new CustomEvent('show-virtual-keyboard', { detail: { layout: 'text' } }));
          }}
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden>
            <rect x="2.5" y="6" width="19" height="12" rx="2" stroke="currentColor" strokeWidth="1.7" />
            <path d="M6 9.5h.01M9 9.5h.01M12 9.5h.01M15 9.5h.01M18 9.5h.01M7 14.5h10" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
          </svg>
        </button>
        <input
          ref={ref}
          value={v}
          autoFocus={autoFocus}
          inputMode="text"
          onChange={e => setV(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter') submit(); }}
          placeholder={placeholder ?? 'Escanee o escriba…'}
          className={`input ${flash ? 'scan-flash' : ''}`}
          style={{ flex: 1, minWidth: 0, textAlign: 'center', textTransform: 'uppercase', letterSpacing: '.06em' }}
        />
        <button onClick={submit} className="btn primary" style={{ flexShrink: 0 }} disabled={!v.trim()}>OK</button>
      </div>
    </div>
  );
}
