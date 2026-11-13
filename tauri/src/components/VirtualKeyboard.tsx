import React, { useEffect, useState } from 'react';

interface Props {
  visible: boolean;
  layout: 'numeric' | 'text';
  onClose: () => void;
}

const NUMERIC = ['1','2','3','4','5','6','7','8','9','0','.','-','Backspace','Enter'];
const ALPHA_ROWS = [
  ['1','2','3','4','5','6','7','8','9','0'],
  ['q','w','e','r','t','y','u','i','o','p'],
  ['a','s','d','f','g','h','j','k','l'],
  ['z','x','c','v','b','n','m'],
];

function insertKey(key: string) {
  const el = document.activeElement as HTMLInputElement | HTMLTextAreaElement | null;
  if (!el || (el.tagName !== 'INPUT' && el.tagName !== 'TEXTAREA')) return;
  const start = el.selectionStart ?? el.value.length;
  const end = el.selectionEnd ?? el.value.length;
  const nativeSetter = Object.getOwnPropertyDescriptor(
    el.tagName === 'TEXTAREA' ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype,
    'value'
  )?.set;
  let newVal = el.value;
  let newCursor = start;
  if (key === 'Backspace') {
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
  } else if (key === 'Enter') {
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    return;
  } else {
    newVal = el.value.slice(0, start) + key + el.value.slice(end);
    newCursor = start + key.length;
  }
  nativeSetter?.call(el, newVal);
  el.selectionStart = el.selectionEnd = newCursor;
  el.dispatchEvent(new Event('input', { bubbles: true }));
}

export default function VirtualKeyboard({ visible, layout, onClose }: Props) {
  const [shift, setShift] = useState(false);

  useEffect(() => {
    if (!visible) setShift(false);
  }, [visible]);

  if (!visible) return null;

  const keyStyle: React.CSSProperties = {
    padding: '10px 12px', margin: 2, borderRadius: 8, border: '1px solid #cbd5e1',
    background: '#f1f5f9', cursor: 'pointer', fontSize: 16, fontWeight: 600,
    minWidth: 40, userSelect: 'none',
  };
  const specialStyle: React.CSSProperties = { ...keyStyle, background: '#e2e8f0', fontSize: 13 };

  const renderKey = (k: string) => {
    const display = k === 'Backspace' ? '⌫' : k === 'Enter' ? '↵' : k === 'Space' ? '␣' : (shift && layout === 'text' && k.length === 1 && /[a-z]/.test(k)) ? k.toUpperCase() : k;
    const value = (shift && layout === 'text' && k.length === 1 && /[a-z]/.test(k)) ? k.toUpperCase() : k;
    return (
      <button key={k} style={k === 'Backspace' || k === 'Enter' || k === 'Space' ? specialStyle : keyStyle}
        onMouseDown={e => { e.preventDefault(); insertKey(k === 'Space' ? 'Space' : (shift && layout === 'text' && k.length === 1 && /[a-z]/.test(k)) ? k.toUpperCase() : k); }}
      >
        {display}
      </button>
    );
  };

  return (
    <div style={{
      position: 'fixed', bottom: 0, left: 0, right: 0, zIndex: 9999,
      background: 'var(--surface, #1e293b)', padding: 8, borderTop: '1px solid var(--border, #334155)',
      display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4,
    }}>
      <div style={{ display: 'flex', justifyContent: 'flex-end', width: '100%', marginBottom: 4 }}>
        <button style={{ ...specialStyle, background: '#ef4444', color: '#fff' }} onMouseDown={e => { e.preventDefault(); onClose(); }}>✕ Cerrar</button>
      </div>
      {layout === 'numeric' ? (
        <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'center', maxWidth: 400 }}>
          {NUMERIC.map(k => renderKey(k))}
        </div>
      ) : (
        <>
          {ALPHA_ROWS.map((row, ri) => (
            <div key={ri} style={{ display: 'flex', justifyContent: 'center' }}>
              {row.map(k => renderKey(k))}
            </div>
          ))}
          <div style={{ display: 'flex', justifyContent: 'center', gap: 4 }}>
            <button style={{ ...specialStyle, minWidth: 60 }} onMouseDown={e => { e.preventDefault(); setShift(s => !s); }}>⇧ Shift</button>
            <button style={{ ...keyStyle, minWidth: 200 }} onMouseDown={e => { e.preventDefault(); insertKey('Space'); }}>Espacio</button>
            <button style={specialStyle} onMouseDown={e => { e.preventDefault(); insertKey('Backspace'); }}>⌫</button>
            <button style={specialStyle} onMouseDown={e => { e.preventDefault(); insertKey('Enter'); }}>↵ Enter</button>
          </div>
        </>
      )}
    </div>
  );
}