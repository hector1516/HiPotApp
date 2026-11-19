import React, { useEffect, useState, useRef, useLayoutEffect } from 'react';

interface Props {
  visible: boolean;
  layout: 'numeric' | 'text';
  onClose: () => void;
}

const NUMERIC_ROWS = [
  ['7', '8', '9', '⌫'],
  ['4', '5', '6', '/'],
  ['1', '2', '3', '*'],
  ['0', '.', '-', '+'],
];
const ALPHA_ROWS = [
  ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
  ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
  ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
  ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.'],
  ['@', '#', '$', '%', '&', '*', '(', ')', '-', '_', '+', '='],
];

function activeField(): HTMLInputElement | HTMLTextAreaElement | null {
  const el = document.activeElement;
  if (!el) return null;
  return el.tagName === 'INPUT' || el.tagName === 'TEXTAREA'
    ? (el as HTMLInputElement | HTMLTextAreaElement)
    : null;
}

function insertKey(key: string) {
  const el = activeField();
  if (!el) return;
  const start = el.selectionStart ?? el.value.length;
  const end = el.selectionEnd ?? el.value.length;

  if (key === 'Tab') {
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }));
    return;
  }
  if (key === 'Enter') {
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    return;
  }

  let newVal = el.value;
  let newCursor = start;
  if (key === '⌫') {
    if (start === end && start > 0) {
      newVal = el.value.slice(0, start - 1) + el.value.slice(end);
      newCursor = start - 1;
    } else {
      newVal = el.value.slice(0, start) + el.value.slice(end);
      newCursor = start;
    }
  } else if (key === 'Space') {
    newVal = el.value.slice(0, start) + ' ' + el.value.slice(end);
    newCursor = start + 1;
  } else {
    newVal = el.value.slice(0, start) + key + el.value.slice(end);
    newCursor = start + key.length;
  }

  const nativeSetter = Object.getOwnPropertyDescriptor(
    el.tagName === 'TEXTAREA' ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype,
    'value',
  )?.set;
  nativeSetter?.call(el, newVal);
  el.selectionStart = el.selectionEnd = newCursor;
  el.dispatchEvent(new Event('input', { bubbles: true }));
}

function clearField() {
  const el = activeField();
  if (!el) return;
  const nativeSetter = Object.getOwnPropertyDescriptor(
    el.tagName === 'TEXTAREA' ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype,
    'value',
  )?.set;
  nativeSetter?.call(el, '');
  el.selectionStart = el.selectionEnd = 0;
  el.dispatchEvent(new Event('input', { bubbles: true }));
}

export default function VirtualKeyboard({ visible, layout, onClose }: Props) {
  const [shift, setShift] = useState(false);
  const [minimized, setMinimized] = useState(false);
  const [preview, setPreview] = useState('');
  const kbRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!visible) {
      setShift(false);
      setMinimized(false);
      document.body.classList.remove('kb-open');
      document.documentElement.style.setProperty('--kb-height', '0px');
      return;
    }
    setPreview(activeField()?.value ?? '');
    const el = activeField();
    if (el) {
      requestAnimationFrame(() => el.scrollIntoView({ block: 'center', behavior: 'smooth' }));
    }
    document.body.classList.add('kb-open');
  }, [visible, layout]);

  useLayoutEffect(() => {
    if (!visible) return;
    const el = kbRef.current;
    const apply = () => {
      const h = el?.offsetHeight ?? 0;
      document.documentElement.style.setProperty('--kb-height', `${h}px`);
    };
    apply();
    const ro = new ResizeObserver(apply);
    if (el) ro.observe(el);
    return () => ro.disconnect();
  }, [visible, layout, minimized]);

  useEffect(() => {
    if (!visible) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [visible, onClose]);

  if (!visible) return null;

  const handleKey = (k: string) => {
    if (k === 'Shift') { setShift(s => !s); return; }
    if (k === 'Limpiar') { clearField(); setPreview(''); return; }
    const value = shift && layout === 'text' && /^[a-z]$/.test(k) ? k.toUpperCase() : k;
    insertKey(value === 'Space' ? 'Space' : value);
    setPreview(activeField()?.value ?? '');
  };

  const display = (k: string) => {
    if (k === '⌫') return '⌫';
    if (k === 'Shift') return shift ? '⇧ Shift' : '⇧ Shift';
    if (k === 'Space') return 'Espacio';
    if (shift && layout === 'text' && /^[a-z]$/.test(k)) return k.toUpperCase();
    return k;
  };

  const isSpecial = (k: string) => k === '⌫' || k === 'Shift' || k === 'Space' || k === 'Limpiar' || k === 'Enter';

  return (
    <div ref={kbRef} className="vkeyboard" role="dialog" aria-label="Teclado virtual">
      <div className="vk-head">
        <span className="vk-title">Teclado virtual</span>
        <span className="vk-hint">Escribe en el campo activo · Esc para cerrar</span>
        <span className="spacer" />
        <button className="btn sm" onMouseDown={e => e.preventDefault()} onClick={() => setMinimized(m => !m)}>
          {minimized ? 'Expandir' : 'Minimizar'}
        </button>
        <button className="btn sm danger" onMouseDown={e => e.preventDefault()} onClick={onClose}>Cerrar</button>
      </div>

      {!minimized && (
        <>
          <div className="vk-preview" aria-live="polite">
            {preview || <span style={{ opacity: .45 }}>Escribiendo…</span>}
          </div>

          <div className="vk-keys">
            {layout === 'numeric' ? (
              <>
                {NUMERIC_ROWS.map((row, ri) => (
                  <div key={ri} className="vk-row">
                    {row.map(k => (
                      <button key={k} className={`vk-key ${isSpecial(k) ? 'special' : ''}`}
                        onMouseDown={e => { e.preventDefault(); handleKey(k); }}>
                        {display(k)}
                      </button>
                    ))}
                  </div>
                ))}
                <div className="vk-row">
                  <button className="vk-key special" style={{ flex: 1 }} onMouseDown={e => { e.preventDefault(); handleKey('Limpiar'); }}>Limpiar</button>
                  <button className="vk-key special" style={{ flex: 1 }} onMouseDown={e => { e.preventDefault(); handleKey('Enter'); }}>Enter</button>
                </div>
              </>
            ) : (
              <>
                {ALPHA_ROWS.map((row, ri) => (
                  <div key={ri} className="vk-row">
                    {row.map(k => (
                      <button key={k} className="vk-key" onMouseDown={e => { e.preventDefault(); handleKey(k); }}>
                        {display(k)}
                      </button>
                    ))}
                  </div>
                ))}
                <div className="vk-row">
                  <button className="vk-key special" style={{ minWidth: 88 }} onMouseDown={e => { e.preventDefault(); handleKey('Shift'); }}>⇧ Shift</button>
                  <button className="vk-key special" style={{ flex: 1 }} onMouseDown={e => { e.preventDefault(); handleKey('Space'); }}>Espacio</button>
                  <button className="vk-key special" onMouseDown={e => { e.preventDefault(); handleKey('Limpiar'); }}>Limpiar</button>
                  <button className="vk-key special" onMouseDown={e => { e.preventDefault(); handleKey('⌫'); }}>⌫</button>
                  <button className="vk-key special" onMouseDown={e => { e.preventDefault(); handleKey('Enter'); }}>Enter</button>
                </div>
              </>
            )}
          </div>
        </>
      )}
    </div>
  );
}
