import React, { useEffect, useState, useRef } from 'react';

interface Props {
  visible: boolean;
  layout: 'numeric' | 'text';
  onClose: () => void;
}

const NUMERIC_ROWS = [
  ['7','8','9','/'],
  ['4','5','6','*'],
  ['1','2','3','-'],
  ['0','.','+','Backspace'],
  ['Enter'],
];
const ALPHA_ROWS = [
  ['1','2','3','4','5','6','7','8','9','0'],
  ['q','w','e','r','t','y','u','i','o','p'],
  ['a','s','d','f','g','h','j','k','l'],
  ['z','x','c','v','b','n','m'],
  ['@','#','$','%','&','*','(',')','-','_','+','='],
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
  const [pos, setPos] = useState({ x: 0, y: 0 });
  const [preview, setPreview] = useState('');
  const dragging = useRef(false);
  const dragOffset = useRef({ x: 0, y: 0 });
  const kbRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!visible) { setShift(false); return; }
    // Centrar horizontalmente, colocar cerca del fondo
    const w = kbRef.current?.offsetWidth ?? 600;
    setPos({ x: Math.max(10, (window.innerWidth - w) / 2), y: window.innerHeight - (kbRef.current?.offsetHeight ?? 300) - 20 });
    // Preview inicial
    const el = document.activeElement as HTMLInputElement | null;
    setPreview(el?.value ?? '');
  }, [visible, layout]);

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      if (dragging.current) {
        setPos({ x: e.clientX - dragOffset.current.x, y: e.clientY - dragOffset.current.y });
      }
    };
    const onUp = () => { dragging.current = false; };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
    return () => { window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
  }, []);

  const startDrag = (e: React.MouseEvent) => {
    dragging.current = true;
    dragOffset.current = { x: e.clientX - pos.x, y: e.clientY - pos.y };
  };

  const handleKey = (k: string) => {
    insertKey(k);
    const el = document.activeElement as HTMLInputElement | null;
    setPreview(el?.value ?? '');
  };

  if (!visible) return null;

  const keyStyle: React.CSSProperties = {
    padding: '14px 18px', margin: 3, borderRadius: 10, border: '1px solid rgba(203,213,225,0.5)',
    background: 'rgba(241,245,249,0.85)', cursor: 'pointer', fontSize: 20, fontWeight: 600,
    minWidth: 52, userSelect: 'none', backdropFilter: 'blur(8px)',
  };
  const specialStyle: React.CSSProperties = { ...keyStyle, background: 'rgba(226,232,240,0.85)', fontSize: 14 };

  const renderKey = (k: string) => {
    const isSpecial = k === 'Backspace' || k === 'Enter' || k === 'Space';
    const display = k === 'Backspace' ? '⌫' : k === 'Enter' ? '↵' : k === 'Space' ? '␣' : (shift && layout === 'text' && k.length === 1 && /[a-z]/.test(k)) ? k.toUpperCase() : k;
    const val = (shift && layout === 'text' && k.length === 1 && /[a-z]/.test(k)) ? k.toUpperCase() : k;
    return (
      <button key={k} style={isSpecial ? specialStyle : keyStyle}
        onMouseDown={e => { e.preventDefault(); handleKey(k === 'Space' ? 'Space' : val); }}
      >
        {display}
      </button>
    );
  };

  return (
    <div ref={kbRef} style={{
      position: 'fixed', left: pos.x, top: pos.y, zIndex: 9999,
      background: 'rgba(30,41,59,0.75)', padding: 10, borderRadius: 14,
      border: '1px solid rgba(255,255,255,0.15)', backdropFilter: 'blur(12px)',
      boxShadow: '0 8px 32px rgba(0,0,0,0.3)', display: 'flex', flexDirection: 'column', gap: 6,
      cursor: 'move', minWidth: 320,
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4, cursor: 'move' }}
        onMouseDown={startDrag}>
        <span style={{ color: '#fff', fontSize: 12, fontWeight: 600, opacity: 0.7 }}>Teclado virtual</span>
        <button style={{ ...specialStyle, background: 'rgba(239,68,68,0.8)', color: '#fff', minWidth: 32, padding: '4px 8px' }}
          onMouseDown={e => { e.preventDefault(); onClose(); }}>✕</button>
      </div>
      {/* Cuadro de preview */}
      <div style={{
        background: 'rgba(255,255,255,0.15)', borderRadius: 8, padding: '6px 10px', marginBottom: 6,
        color: '#fff', fontSize: 16, fontFamily: 'monospace', minHeight: 28,
        border: '1px solid rgba(255,255,255,0.2)', wordBreak: 'break-all',
      }}>
        {preview || <span style={{ opacity: 0.4 }}>Escribiendo…</span>}
      </div>
      {layout === 'numeric' ? (
        <>
          {NUMERIC_ROWS.map((row, ri) => (
            <div key={ri} style={{ display: 'flex', justifyContent: 'center' }}>
              {row.map(k => renderKey(k))}
            </div>
          ))}
        </>
      ) : (
        <>
          {ALPHA_ROWS.map((row, ri) => (
            <div key={ri} style={{ display: 'flex', justifyContent: 'center' }}>
              {row.map(k => renderKey(k))}
            </div>
          ))}
          <div style={{ display: 'flex', justifyContent: 'center', gap: 6 }}>
            <button style={{ ...specialStyle, minWidth: 70 }} onMouseDown={e => { e.preventDefault(); setShift(s => !s); }}>⇧ Shift</button>
            <button style={{ ...keyStyle, minWidth: 220 }} onMouseDown={e => { e.preventDefault(); handleKey('Space'); }}>Espacio</button>
            <button style={specialStyle} onMouseDown={e => { e.preventDefault(); handleKey('Backspace'); }}>⌫</button>
            <button style={specialStyle} onMouseDown={e => { e.preventDefault(); handleKey('Enter'); }}>↵ Enter</button>
          </div>
        </>
      )}
    </div>
  );
}