import React, { useState } from 'react';
import { Stage, Layer, Text, Rect, Line } from 'react-konva';

// Editor visual drag & drop (doc 09): paleta -> canvas -> props.
// Touch-friendly, con zoom, deshacer/rehacer, QR/barcode, líneas y marcos.
export type LObj =
  | { id: string; kind: 'text'; x: number; y: number; content: string; fontSize: number }
  | { id: string; kind: 'rect'; x: number; y: number; w: number; h: number }
  | { id: string; kind: 'line'; x: number; y: number; x2: number; y2: number }
  | { id: string; kind: 'barcode'; x: number; y: number; w: number; data: string }
  | { id: string; kind: 'qr'; x: number; y: number; size: number; data: string };

const PALETTE: { label: string; make: () => LObj }[] = [
  { label: '📝 Texto', make: () => ({ id: nid(), kind: 'text', x: 120, y: 120, content: 'TEXTO', fontSize: 20 }) },
  { label: '📦 Modelo', make: () => ({ id: nid(), kind: 'text', x: 120, y: 140, content: 'Modelo: {modelo}', fontSize: 18 }) },
  { label: '🔢 Serie', make: () => ({ id: nid(), kind: 'text', x: 120, y: 160, content: 'Serie: {serie}', fontSize: 18 }) },
  { label: '✅ Estado', make: () => ({ id: nid(), kind: 'text', x: 120, y: 180, content: 'Estado: {estado}', fontSize: 20 }) },
  { label: '👤 Operador', make: () => ({ id: nid(), kind: 'text', x: 120, y: 200, content: 'Op: {operador}', fontSize: 16 }) },
  { label: '📊 Barcode', make: () => ({ id: nid(), kind: 'barcode', x: 60, y: 260, w: 300, data: '{modelo}-{serie}-{batch}' }) },
  { label: '📱 QR', make: () => ({ id: nid(), kind: 'qr', x: 420, y: 240, size: 90, data: '{modelo}|{serie}|{batch}|{estado}' }) },
  { label: '▬ Línea', make: () => ({ id: nid(), kind: 'line', x: 40, y: 230, x2: 460, y2: 230 }) },
  { label: '□ Marco', make: () => ({ id: nid(), kind: 'rect', x: 30, y: 20, w: 470, h: 300 }) },
];
const nid = () => `o${Date.now()}${Math.floor(Math.random() * 999)}`;

export function objsToTemplate(objs: LObj[], wMm = 100, hMm = 60): string {
  return JSON.stringify({
    label: { width_mm: wMm, height_mm: hMm, bg_color: '#FFFFFF' },
    objects: objs.map(o => {
      if (o.kind === 'text') return { id: o.id, type: 'text', x: o.x, y: o.y, content: o.content, font_size: o.fontSize };
      if (o.kind === 'rect') return { id: o.id, type: 'rect', x: o.x, y: o.y, w: o.w, h: o.h, thickness: 2 };
      if (o.kind === 'line') return { id: o.id, type: 'line', x: o.x, y: o.y, x2: o.x2, y2: o.y2, thickness: 2 };
      if (o.kind === 'barcode') return { id: o.id, type: 'barcode', x: o.x, y: o.y, data_source: o.data, height: 80 };
      return { id: o.id, type: 'qrcode', x: o.x, y: o.y, size: o.size, data_source: o.data };
    }),
  });
}

export function templateToObjs(canvasData: string): LObj[] {
  try {
    const t = JSON.parse(canvasData);
    return (t.objects ?? []).map((o: any, i: number): LObj => {
      const id = o.id ?? `o${i}`;
      if (o.type === 'rect') return { id, kind: 'rect', x: o.x ?? 0, y: o.y ?? 0, w: o.w ?? 200, h: o.h ?? 80 };
      if (o.type === 'line') return { id, kind: 'line', x: o.x ?? 0, y: o.y ?? 0, x2: o.x2 ?? 200, y2: o.y2 ?? 0 };
      if (o.type === 'barcode') return { id, kind: 'barcode', x: o.x ?? 0, y: o.y ?? 0, w: 300, data: o.data_source ?? '' };
      if (o.type === 'qrcode') return { id, kind: 'qr', x: o.x ?? 0, y: o.y ?? 0, size: o.size ?? 90, data: o.data_source ?? '' };
      return { id, kind: 'text', x: o.x ?? 0, y: o.y ?? 0, content: o.content ?? '', fontSize: o.font_size ?? 20 };
    });
  } catch { return []; }
}

export function LabelEditor({ initial, onSave }: {
  initial?: LObj[]; onSave: (objs: LObj[]) => void;
}) {
  const [objs, setObjs] = useState<LObj[]>(initial ?? [
    { id: 'o1', kind: 'text', x: 40, y: 30, content: 'HIPOT - RESULTADO', fontSize: 28 },
    { id: 'o2', kind: 'text', x: 40, y: 70, content: 'Modelo: {modelo}', fontSize: 18 },
  ]);
  const [past, setPast] = useState<LObj[][]>([]);
  const [future, setFuture] = useState<LObj[][]>([]);
  const [sel, setSel] = useState<string | null>('o1');
  const [zoom, setZoom] = useState(1);

  const commit = (next: LObj[]) => {
    setPast(p => [...p.slice(-29), objs]);
    setFuture([]);
    setObjs(next);
  };
  const undo = () => {
    if (!past.length) return;
    setFuture(f => [objs, ...f]);
    const prev = past[past.length - 1];
    setPast(p => p.slice(0, -1));
    setObjs(prev);
  };
  const redo = () => {
    if (!future.length) return;
    setPast(p => [...p, objs]);
    const [next, ...rest] = future;
    setFuture(rest);
    setObjs(next);
  };

  const move = (id: string, x: number, y: number) =>
    setObjs(os => os.map(o => o.id === id ? { ...o, x: Math.round(x), y: Math.round(y) } as LObj : o));

  const selected = objs.find(o => o.id === sel) ?? null;

  const num = (v: string, fb: number) => { const n = Number(v); return Number.isFinite(n) ? n : fb; };

  return (
    <div style={{ display: 'flex', gap: 12, alignItems: 'flex-start' }}>
      <div style={{ width: 180, flexShrink: 0 }}>
        <b>📋 Paleta</b>
        {PALETTE.map(p => (
          <button key={p.label} className="btn ghost" style={{ width: '100%', margin: '4px 0', minHeight: 52, fontSize: 15 }}
            onClick={() => { const o = p.make(); commit([...objs, o]); setSel(o.id); }}>
            {p.label}</button>
        ))}
        <div style={{ display: 'flex', gap: 4, marginTop: 8 }}>
          <button className="btn ghost" style={{ flex: 1 }} disabled={!past.length} onClick={undo}>↩️</button>
          <button className="btn ghost" style={{ flex: 1 }} disabled={!future.length} onClick={redo}>↪️</button>
        </div>
        <div style={{ display: 'flex', gap: 4, marginTop: 4, alignItems: 'center' }}>
          <button className="btn ghost" style={{ flex: 1 }} onClick={() => setZoom(z => Math.max(.5, +(z - .25).toFixed(2)))}>🔍−</button>
          <b>{Math.round(zoom * 100)}%</b>
          <button className="btn ghost" style={{ flex: 1 }} onClick={() => setZoom(z => Math.min(2, +(z + .25).toFixed(2)))}>🔍+</button>
        </div>
      </div>

      <div style={{ background: '#fff', border: '2px solid #333', borderRadius: 8, overflow: 'auto' }}>
        <Stage width={560 * zoom} height={340 * zoom} scaleX={zoom} scaleY={zoom}
          onMouseDown={e => { if (e.target === e.target.getStage()) setSel(null); }}>
          <Layer>
            {objs.map(o => {
              const common = {
                key: o.id, draggable: true,
                onClick: () => setSel(o.id),
                onDragEnd: (e: any) => { move(o.id, e.target.x(), e.target.y()); },
              };
              const hl = sel === o.id ? '#1976d2' : undefined;
              if (o.kind === 'text')
                return <Text {...common} x={o.x} y={o.y} text={o.content} fontSize={o.fontSize} stroke={hl} strokeWidth={hl ? 1 : 0} />;
              if (o.kind === 'rect')
                return <Rect {...common} x={o.x} y={o.y} width={o.w} height={o.h} stroke={hl ?? '#000'} strokeWidth={2} />;
              if (o.kind === 'line')
                return <Line {...common} points={[o.x, o.y, o.x2, o.y2]} stroke={hl ?? '#000'} strokeWidth={3} />;
              if (o.kind === 'barcode')
                return <Rect {...common} x={o.x} y={o.y} width={o.w} height={70}
                  fill="#fff" stroke={hl ?? '#000'} strokeWidth={2} />;
              return <Rect {...common} x={o.x} y={o.y} width={o.size} height={o.size}
                fill="#eee" stroke={hl ?? '#000'} strokeWidth={2} />;
            })}
          </Layer>
        </Stage>
        <div style={{ padding: 4, fontSize: 13, color: '#616161' }}>
          {objs.length} objetos · click para seleccionar · arrastra para mover
        </div>
      </div>

      <div style={{ width: 270, flexShrink: 0 }} className="card">
        <h3>Propiedades</h3>
        {!selected && <div style={{ color: '#757575' }}>Selecciona un objeto del canvas.</div>}
        {selected?.kind === 'text' && (
          <>
            <label>Contenido</label>
            <input className="input" value={selected.content}
              onChange={e => setObjs(os => os.map(x => x.id === sel ? { ...x, content: e.target.value } as LObj : x))} />
            <label>Tamaño</label>
            <input className="input" type="number" value={selected.fontSize}
              onChange={e => setObjs(os => os.map(x => x.id === sel ? { ...x, fontSize: num(e.target.value, 20) } as LObj : x))} />
          </>
        )}
        {(selected?.kind === 'barcode' || selected?.kind === 'qr') && (
          <>
            <label>Datos</label>
            <input className="input" value={selected.data}
              onChange={e => setObjs(os => os.map(x => x.id === sel ? { ...x, data: e.target.value } as LObj : x))} />
            {selected.kind === 'barcode' && (<><label>Ancho</label>
              <input className="input" type="number" value={selected.w}
                onChange={e => setObjs(os => os.map(x => x.id === sel ? { ...x, w: num(e.target.value, 300) } as LObj : x))} /></>)}
            {selected.kind === 'qr' && (<><label>Tamaño</label>
              <input className="input" type="number" value={selected.size}
                onChange={e => setObjs(os => os.map(x => x.id === sel ? { ...x, size: num(e.target.value, 90) } as LObj : x))} /></>)}
          </>
        )}
        {selected && selected.kind !== 'text' && selected.kind !== 'barcode' && selected.kind !== 'qr' && (
          <div style={{ fontSize: 14, color: '#616161' }}>Mueve el objeto arrastrándolo. Ajustes finos de tamaño: elimínalo y vuelve a agregarlo.</div>
        )}
        {selected && (
          <div style={{ display: 'flex', gap: 6, marginTop: 8 }}>
            <button className="btn ghost" style={{ flex: 1 }}
              onClick={() => {
                const c = { ...selected, id: nid() } as LObj;
                const n = { ...c, x: (c as any).x + 20, y: (c as any).y + 20 } as LObj;
                commit([...objs, n]); setSel(n.id);
              }}>📋 Duplicar</button>
            <button className="btn danger" style={{ flex: 1 }}
              onClick={() => { commit(objs.filter(x => x.id !== sel)); setSel(null); }}>🗑️</button>
          </div>
        )}
        <button className="btn ok big" style={{ width: '100%', marginTop: 12 }}
          onClick={() => onSave(objs)}>💾 Guardar</button>
      </div>
    </div>
  );
}
