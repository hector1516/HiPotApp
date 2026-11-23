import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { LabelEditor, LObj, objsToTemplate, templateToObjs } from '../components/LabelEditor';
import { askConfirm, notify } from '../components/Feedback';
import { SafetyNote } from '../components/SafetyNote';

const TABS = ['Usuarios', 'Recetas', 'PLC', 'Hipot', 'BD', 'Etiquetas', 'Logs', 'IO', 'Manual', 'Sistema'] as const;
type Tab = (typeof TABS)[number];

const NAV_GROUPS: { label: string; tabs: Tab[] }[] = [
  { label: 'Operación', tabs: ['Recetas', 'Etiquetas'] },
  { label: 'Hardware', tabs: ['PLC', 'IO', 'Hipot', 'Manual'] },
  { label: 'Datos', tabs: ['BD'] },
  { label: 'Sistema', tabs: ['Usuarios', 'Logs', 'Sistema'] },
];

export function AdminPanel({ onBack, adminBatch }: { onBack: () => void; adminBatch?: string }) {
  const [tab, setTab] = useState<Tab>('Recetas');
  return (
    <div className="admin-panel" style={{ display: 'flex', height: '100%', minHeight: 0, minWidth: 0 }}>
      <aside style={{
        width: 232, flexShrink: 0, background: 'linear-gradient(180deg, var(--rail) 0%, var(--rail-2) 100%)',
        borderRight: '1px solid rgba(255,255,255,.08)', padding: 12, display: 'flex', flexDirection: 'column', gap: 4,
        boxShadow: 'var(--shadow-rail)', position: 'relative', zIndex: 2,
      }}>
        <div style={{ padding: '4px 6px 12px 6px' }}>
          <div className="row" style={{ gap: 8, marginBottom: 6 }}>
            <img src="logo.png" alt="HiPotApp" style={{ height: 30, objectFit: 'contain' }} />
            <div className="rail-eyebrow" style={{ margin: 0 }}>Configuración</div>
          </div>
          <div style={{ fontFamily: 'var(--font-display)', fontSize: 18, fontWeight: 800, color: '#fff', marginTop: 2 }}>
            Panel de control
          </div>
          <div style={{ fontSize: 12, color: 'var(--on-rail-muted)', marginTop: 4 }}>
            {adminBatch && adminBatch.toUpperCase() !== 'GODMODE'
              ? `Admin: ${adminBatch}`
              : 'Acceso administrador'}
          </div>
        </div>

        <button className="btn" style={{ width: '100%', justifyContent: 'flex-start', background: 'rgba(255,255,255,.06)', borderColor: 'rgba(255,255,255,.1)', color: 'var(--on-rail)', minHeight: 42 }} onClick={onBack}>
          ← Volver al inicio
        </button>

        <div style={{ flex: 1, overflow: 'auto', marginTop: 8 }}>
          {NAV_GROUPS.map(group => (
            <div key={group.label} style={{ marginBottom: 10 }}>
              <div style={{ fontSize: 12, fontWeight: 800, letterSpacing: '.14em', textTransform: 'uppercase', color: 'var(--on-rail-muted)', padding: '8px 8px 4px 8px' }}>
                {group.label}
              </div>
              {group.tabs.map(t => {
                const active = tab === t;
                return (
                  <button
                    key={t}
                    onClick={() => setTab(t)}
                    style={{
                      width: '100%', minHeight: 40, display: 'flex', alignItems: 'center', gap: 8,
                      padding: '0 10px', marginBottom: 2, borderRadius: 8, cursor: 'pointer',
                      border: '1px solid transparent',
                      background: active ? 'rgba(180,83,9,.18)' : 'transparent',
                      color: active ? '#fff' : 'var(--on-rail-muted)',
                      fontWeight: active ? 800 : 600, fontSize: 13,
                      borderLeft: active ? '3px solid var(--accent-2)' : '3px solid transparent',
                      textAlign: 'left',
                    }}
                  >
                    <span style={{ width: 6, height: 6, borderRadius: 999, background: active ? 'var(--accent-2)' : 'rgba(255,255,255,.2)', flexShrink: 0 }} />
                    {t}
                  </button>
                );
              })}
            </div>
          ))}
        </div>

        <div style={{ borderTop: '1px solid rgba(255,255,255,.1)', paddingTop: 8, fontSize: 12, color: 'var(--on-rail-muted)' }}>
          Los cambios se guardan en SQL Express.
        </div>
      </aside>

      <div className="admin-content" style={{ flex: 1, padding: 20, overflow: 'auto', minWidth: 0, background: 'var(--bg)' }}>
        <motion.div
          key={tab}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.28 }}
        >
          {tab === 'Usuarios' && <UsersTab />}
          {tab === 'Recetas' && <RecipesTab adminBatch={adminBatch} />}
          {tab === 'PLC' && <PlcTab />}
          {tab === 'Hipot' && <HipotTab />}
          {tab === 'BD' && <DbTab />}
          {tab === 'Etiquetas' && <LabelsTab />}
          {tab === 'Logs' && <LogsTab />}
          {tab === 'IO' && <PlcIoTab />}
          {tab === 'Manual' && <ManualTab />}
          {tab === 'Sistema' && <SystemTab />}
        </motion.div>
      </div>
    </div>
  );
}

function UsersTab(){
  const [users,setUsers]=useState<any[]>([]);
  const [msg,setMsg]=useState('');
  const [q,setQ]=useState('');
  const [editing,setEditing]=useState<null|{id:number|null,username:string,role:'operator'|'admin',is_active:boolean}>(null);
  const [viewPerm,setViewPerm]=useState<any|null>(null);
  const load=()=>invoke<any[]>('list_users').then(setUsers).catch(()=>setUsers([]));
  useEffect(()=>{load();},[]);
  const rows=users.filter(u=>!q||u.username.includes(q.toUpperCase()));
  const doDelete=async(u:any)=>{
    if(!await askConfirm(`Borrar usuario ${u.username}?`)) return;
    try{
      const r = await invoke<string>('delete_user', { id: u.id });
      if (r !== 'deleted') throw new Error(r);
      setMsg(`Borrado ${u.username}`); load();
    }catch(e:any){ setMsg(String(e?.message??e)); }
  };
  const doSave=async()=>{
    if(!editing) return;
    if(!editing.username.trim()){ setMsg('Batch es obligatorio'); return; }
    const payload:any={username:editing.username.trim().toUpperCase(),role:editing.role,is_active:editing.is_active};
    try{
      if(editing.id){
        await invoke('user_update', { uid: editing.id, username: payload.username, role: payload.role, isActive: payload.is_active });
        setMsg('Usuario actualizado');
      }
      else {
        await invoke('add_user', { username: payload.username, password: '', role: payload.role });
        setMsg('Usuario creado - gafete listo');
      }
      setEditing(null); load();
    }catch(e:any){ setMsg(String(e?.message??e)); }
  };
  return (
    <div style={{ minWidth:0 }}>
      <h2 style={{ margin:'0 0 10px 0', fontSize:16 }}>Usuarios — solo batch + permisos</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Usuario')||msg.startsWith('Borrado')?'#e6f4ea':undefined, borderColor: msg.startsWith('Usuario')?'#a8d0b4':undefined, color: msg.startsWith('Usuario')?'var(--ok)':undefined }}>{msg}</div>}
      <div className="card" style={{ marginBottom:10 }}>
        <div style={{ display:'flex', gap:8, minWidth:0, flexWrap:'wrap' }}>
          <input className="input" placeholder="Buscar batch/usuario" value={q} onChange={e=>setQ(e.target.value.toUpperCase())} style={{ flex:'1 1 200px', minWidth:0 }} />
          <button className="btn primary" style={{ flexShrink:0 }} onClick={()=>setEditing({id:null,username:q, role:'operator', is_active:true})}>+ Alta</button>
          <button className="btn" style={{ flexShrink:0 }} onClick={load}>Actualizar</button>
        </div>
      </div>
      <div className="card" style={{ overflow:'auto', minWidth:0 }}>
        <table className="grid"><thead><tr><th>Batch (usuario)</th><th>Permiso</th><th>Estado</th><th style={{ minWidth:210 }}>Acciones</th></tr></thead>
          <tbody>{rows.map((u:any)=>(
            <tr key={u.id}><td><b>{u.username}</b></td><td><span className={`pill ${u.role==='admin'?'accent':''}`} style={{ fontSize:11 }}>{u.role==='admin'?'Admin':'Operador'}</span></td><td>{u.is_active?'Activo':'Inactivo'}</td>
              <td>
                <div style={{ display:'flex', gap:6, flexWrap:'wrap' }}>
                  <button className="btn sm" onClick={async()=>{ try { const d=await invoke<any>('user_get', { uid: u.id }); setViewPerm(d); } catch (e:any) { setMsg(String(e?.message??e)); } }}>Ver permisos</button>
                  <button className="btn sm primary" onClick={()=>setEditing({id:u.id,username:u.username,role:u.role,is_active:u.is_active})}>Editar</button>
                  <button className="btn sm danger" onClick={()=>doDelete(u)}>Borrar</button>
                </div>
              </td></tr>
          ))}{!rows.length && <tr><td colSpan={4} style={{ textAlign:'center', color:'var(--muted)' }}>Sin usuarios</td></tr>}</tbody>
        </table>
      </div>

      {viewPerm && (
        <div style={{ position:'fixed', inset:0, background:'rgba(20,27,38,.55)', display:'flex', alignItems:'center', justifyContent:'center', zIndex:4000, padding:12 }} onClick={()=>setViewPerm(null)}>
          <div className="card" style={{ width:'min(420px,96vw)', minWidth:0 }} onClick={e=>e.stopPropagation()}>
            <h3 style={{ margin:'0 0 8px 0' }}>{viewPerm.username} — {viewPerm.role==='admin'?'Admin':'Operador'}</h3>
            <div style={{ fontSize:12, color: viewPerm.is_active?'var(--ok)':'var(--bad)', fontWeight:700, marginBottom:8 }}>{viewPerm.is_active?'Activo':'Inactivo'}</div>
            <div style={{ fontSize:13, fontWeight:700, marginBottom:4 }}>Permisos:</div>
            <ul style={{ margin:0, paddingLeft:18, fontSize:13, color:'var(--text-2)' }}>{(viewPerm.permissions??[]).map((p:string,i:number)=><li key={i}>{p}</li>)}</ul>
            <div style={{ display:'flex', justifyContent:'flex-end', marginTop:12 }}><button className="btn" onClick={()=>setViewPerm(null)}>Cerrar</button></div>
          </div>
        </div>
      )}

      {editing && (
        <div style={{ position:'fixed', inset:0, background:'rgba(20,27,38,.55)', display:'flex', alignItems:'center', justifyContent:'center', zIndex:4000, padding:12 }} onClick={()=>setEditing(null)}>
          <div className="card" style={{ width:'min(420px,96vw)', minWidth:0 }} onClick={e=>e.stopPropagation()}>
            <h3 style={{ margin:'0 0 10px 0' }}>{editing.id?'Editar usuario':'Alta de usuario'}</h3>
            <div style={{ display:'flex', flexDirection:'column', gap:8 }}>
              <label style={{ fontSize:12, fontWeight:700, color:'var(--text-2)' }}>Batch / Usuario — gafete (mayúsculas)</label>
              <input className="input" placeholder="Ej. BATCH-001" value={editing.username} onChange={e=>setEditing({...editing,username:e.target.value.toUpperCase()})} />
              <label style={{ fontSize:12, fontWeight:700, color:'var(--text-2)' }}>Permiso</label>
              <select className="input" value={editing.role} onChange={e=>setEditing({...editing,role:e.target.value as any})}>
                <option value="operator">Operador — gafete</option>
                <option value="admin">Admin — acceso a configuración</option>
              </select>
              <div style={{ fontSize:12, color:'var(--muted)', background:'var(--surface-2)', padding:'6px 8px', borderRadius:6 }}>Todos entran con gafete (sin contraseña). El permiso define qué puede hacer.</div>
              <label style={{ display:'flex', gap:8, alignItems:'center', fontSize:13, fontWeight:600 }}>
                <input type="checkbox" checked={editing.is_active} onChange={e=>setEditing({...editing,is_active:e.target.checked})} style={{ width:18, height:18 }} /> Activo
              </label>
            </div>
            <div style={{ display:'flex', gap:8, justifyContent:'flex-end', marginTop:12, flexWrap:'wrap' }}>
              <button className="btn" onClick={()=>setEditing(null)}>Cancelar</button>
              <button className="btn primary" onClick={doSave}>{editing.id?'Guardar cambios':'Crear usuario'}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

const VOLTS = [120,208,220,240];
const COMPS = ['FAN','DOOR','FRAME','DRAIN','DEFROST','LIGHT'];

function matrixFromDetail(detail:any){
  const m: Record<string, Record<string,{min:string,max:string,enabled:boolean}>> = {};
  const t: Record<string,{min:string,max:string}> = {};
  for(const v of VOLTS){ const vs=String(v); m[vs]={}; t[vs]={min:'',max:''}; for(const c of COMPS) m[vs][c]={min:'',max:'',enabled:false}; }
  for(const c of (detail?.components ?? [])){
    const vs = String(c.voltage).replace('VAC','');
    if(m[vs] && m[vs][c.name] !== undefined){
      m[vs][c.name]={min:String(c.min ?? ''),max:String(c.max ?? ''),enabled: !!c.enabled};
    }
  }
  for(const x of (detail?.totals ?? [])){ const vs=String(x.voltage).replace('VAC',''); if(t[vs]!==undefined) t[vs]={min:String(x.min ?? ''),max:String(x.max ?? '')}; }
  return {m,t};
}

function RecipesTab({adminBatch}:{adminBatch?:string}){
  const [list,setList]=useState<any[]>([]);
  const [q,setQ]=useState('');
  const [msg,setMsg]=useState('');
  const [editing,setEditing]=useState<null|{id:number|null,name:string,model:string,product:string,voltage:string,m:any,t:any,hipotVoltage:string,hipotCurrent:string,hipotTime:string}>(null);
  const load=()=>invoke<any[]>('list_recipes').then(setList).catch(()=>setList([]));
  useEffect(()=>{load();},[]);
  const rows=list.filter(r=>!q||r.model.includes(q));
  const openNew=()=>{
    const m:any={}; const t:any={};
    for(const v of VOLTS){ const vs=String(v); m[vs]={}; t[vs]={min:'',max:''}; for(const c of COMPS) m[vs][c]={min:'',max:'',enabled:false}; }
    setEditing({id:null,name:'',model:q||'',product:'',voltage:'120',m,t,hipotVoltage:'',hipotCurrent:'',hipotTime:'10'});
  };
  const openEdit=async(r:any)=>{
    const d=await invoke<any>('get_recipe_by_model', { model: r.model });
    const {m,t}=matrixFromDetail(d);
    const compVoltages = (d?.components ?? [])
      .map((c:any)=>String(c.voltage).replace('VAC',''))
      .filter((v:string)=>VOLTS.map(String).includes(v));
    let voltage = compVoltages[0] ?? '120';
    if (!compVoltages.length) {
      let raw = d?.voltage_levels;
      if (typeof raw === 'string') { try { raw = JSON.parse(raw); } catch {} }
      const first = Array.isArray(raw) ? raw[0] : raw;
      const n = Number(String(first ?? '').replace(/[^0-9.]/g,''));
      if (Number.isFinite(n) && VOLTS.map(String).includes(String(n))) voltage = String(n);
    }
    setEditing({id:d.id,name:d.name,model:d.model,product:d.product||'',voltage,m,t,
      hipotVoltage:d.hipot_voltage!=null?String(d.hipot_voltage):'',
      hipotCurrent:d.hipot_current_max!=null?String(d.hipot_current_max):'',
      hipotTime:d.hipot_time!=null?String(d.hipot_time):'10'});
  };
  const doDelete=async(r:any)=>{
    if(!await askConfirm(`Borrar receta ${r.model} — ${r.name}?`)) return;
    await invoke('delete_recipe', { id: r.id }); setMsg(`Borrada ${r.model}`); load();
  };
  const doDup=async(r:any)=>{
    const model=prompt(`Modelo para la copia de ${r.model}:`, `${r.model}-COPIA`);
    if(!model) return;
    await invoke('duplicate_recipe', { id: r.id, newModel: model.toUpperCase() }); setMsg(`Duplicada como ${model.toUpperCase()}`); load();
  };
  const doSave=async()=>{
    if(!editing) return;
    if(!editing.model.trim()||!editing.name.trim()){ setMsg('Modelo y nombre son obligatorios'); return; }
    const vs = editing.voltage;
    if(!VOLTS.map(String).includes(vs)){ setMsg('Voltaje de receta inválido'); return; }
    const components:any[]=[]; const totals:any[]=[];
    for(const c of COMPS){
      const cell=editing.m[vs][c];
      if(cell.enabled){
        if(cell.min===''||cell.max===''){ setMsg(`${c} ${vs}V: complete MIN y MAX`); return; }
        const mn=parseFloat(cell.min), mx=parseFloat(cell.max);
        if(isNaN(mn)||isNaN(mx)||mn>=mx){ setMsg(`${c} ${vs}V: MIN debe ser < MAX`); return; }
        components.push({name:c,voltage:vs,enabled:true,min:mn,max:mx});
      }
    }
    const tv=editing.t[vs];
    if(tv.min!==''||tv.max!==''){
      const mn=parseFloat(tv.min), mx=parseFloat(tv.max);
      if(isNaN(mn)||isNaN(mx)||mn>=mx){ setMsg(`TOTAL ${vs}V: MIN < MAX`); return; }
      totals.push({voltage:vs,min:mn,max:mx});
    }
    const parseNum = (raw: string) => raw.trim() === '' ? null : Number(raw);
    const hipotVoltage = parseNum(editing.hipotVoltage);
    const hipotCurrent = parseNum(editing.hipotCurrent);
    let hipotTime = editing.hipotTime.trim() === '' ? 10 : Number(editing.hipotTime);
    if (!Number.isFinite(hipotTime) || hipotTime <= 0) hipotTime = 10;
    if ([hipotVoltage, hipotCurrent, hipotTime].some(v => v !== null && !Number.isFinite(v))) {
      setMsg('Parámetros Hipot inválidos'); return;
    }
    const body:any={name:editing.name.trim(),model:editing.model.trim().toUpperCase(),type:'current_consumption',product:editing.product||'',voltage_levels:JSON.stringify([vs]),components,totals,modified_by:(adminBatch||'').toUpperCase()||'ADMIN'};
    try{
      if(editing.id) {
        const r = await invoke<string>('update_recipe', { id: editing.id, name: body.name, model: body.model, type_: body.type, product: body.product, voltageLevels: body.voltage_levels, components: body.components, totals: body.totals, modifiedBy: body.modified_by, hipotVoltage, hipotCurrentMax: hipotCurrent, hipotTime });
        if (r !== 'updated') throw new Error(r);
      } else {
        const id = await invoke<number>('create_recipe', { name: body.name, model: body.model, type_: body.type, product: body.product, voltageLevels: body.voltage_levels, components: body.components, totals: body.totals, modifiedBy: body.modified_by, hipotVoltage, hipotCurrentMax: hipotCurrent, hipotTime });
        if (id < 0) throw new Error('No se pudo crear la receta. Verifique que el modelo no exista.');
      }
      setEditing(null); setMsg(editing.id?'Receta actualizada':'Receta creada'); load();
    }catch(err:any){ setMsg(String(err?.message ?? err)); }
  };
  const restoreDefaults = async () => {
    if (!await askConfirm('¿Restaurar las 4 recetas de fábrica (RL-120, RL-208, RL-220, RL-240)? Se actualizan si ya existen; la receta RL antigua se reemplaza si no tiene historial. Otras recetas no se borran.')) return;
    try {
      const r = await invoke<any>('recipes_restore_defaults');
      if (!r?.ok) throw new Error(r?.error || 'Error al restaurar');
      setMsg(`Recetas de fábrica: ${r.inserted ?? 0} nuevas, ${r.updated ?? 0} actualizadas`);
      load();
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };
  const editVoltage = editing?.voltage ?? '120';
  return (
    <div style={{ minWidth:0 }}>
      <h2 style={{ margin:'0 0 10px 0', fontSize:16 }}>Recetas — altas / bajas / cambios</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Receta')||msg.startsWith('Duplicada')||msg.startsWith('Borrada')?'#e6f4ea':undefined, borderColor: msg.startsWith('Receta')?'#a8d0b4':undefined, color: msg.startsWith('Receta')?'var(--ok)':undefined }}>{msg}</div>}
      <div className="card" style={{ marginBottom:10 }}>
        <div style={{ display:'flex', gap:8, minWidth:0, flexWrap:'wrap' }}>
          <input className="input" placeholder="Buscar modelo" value={q} onChange={e=>setQ(e.target.value.toUpperCase())} style={{ flex:'1 1 200px', minWidth:0 }} />
          <button className="btn primary" style={{ flexShrink:0 }} onClick={openNew}>+ Alta</button>
          <button className="btn" style={{ flexShrink:0 }} onClick={load}>Actualizar</button>
          <button className="btn" style={{ flexShrink:0 }} onClick={restoreDefaults}>Restaurar fábrica</button>
        </div>
      </div>
      <div className="card" style={{ overflow:'auto', minWidth:0 }}>
        <table className="grid"><thead><tr><th>Modelo</th><th>Nombre</th><th>Hipot</th><th>Guardado</th><th style={{ minWidth:220 }}>Acciones</th></tr></thead>
          <tbody>{rows.map(r=>(
            <tr key={r.id}><td><b>{r.model}</b></td><td>{r.name}</td><td className="mono" style={{ fontSize:12 }}>{(r.hipot_voltage ?? 0) > 0 && (r.hipot_current_max ?? 0) > 0 ? <>{`${r.hipot_voltage} V`} · {`${r.hipot_current_max} mA`} · {`${r.hipot_time ?? 10} s`}</> : <span className="pill warn">Hipot sin configurar</span>}</td><td style={{ fontSize:11, color:'var(--muted)' }}>{r.modified_by?`${r.modified_by} · `:''}{r.updated_at?new Date(r.updated_at).toLocaleString('es-MX'):(r.created_at?new Date(r.created_at).toLocaleString('es-MX'):'—')}</td>
              <td>
                <div style={{ display:'flex', gap:6, flexWrap:'wrap' }}>
                  <button className="btn sm primary" onClick={()=>openEdit(r)}>Editar</button>
                  <button className="btn sm" onClick={()=>doDup(r)}>Duplicar</button>
                  <button className="btn sm danger" onClick={()=>doDelete(r)}>Borrar</button>
                </div>
              </td></tr>
          ))}{!rows.length && <tr><td colSpan={5} style={{ textAlign:'center', color:'var(--muted)' }}>Sin recetas — use + Alta</td></tr>}</tbody>
        </table>
      </div>

      {editing && (
        <div style={{ position:'fixed', inset:0, background:'rgba(20,27,38,.55)', display:'flex', alignItems:'center', justifyContent:'center', zIndex:4000, padding:12 }} onClick={()=>setEditing(null)}>
          <div className="card" style={{ width:'min(1180px,96vw)', maxHeight:'92vh', overflow:'auto', minWidth:0 }} onClick={e=>e.stopPropagation()}>
            <h3 style={{ margin:'0 0 10px 0' }}>{editing.id?'Editar receta':'Alta de receta'} — una receta por fila (modelo + voltaje)</h3>
            <div style={{ display:'flex', gap:8, minWidth:0, flexWrap:'wrap', marginBottom:6 }}>
              <input className="input" placeholder="Modelo único (ej. RL-120)" value={editing.model} onChange={e=>setEditing({...editing,model:e.target.value.toUpperCase()})} style={{ flex:'1 1 160px' }} />
              <input className="input" placeholder="Nombre (ej. CONSUMOS RL 120VAC)" value={editing.name} onChange={e=>setEditing({...editing,name:e.target.value})} style={{ flex:'2 1 260px' }} />
              <input className="input" placeholder="Producto (opcional)" value={editing.product} onChange={e=>setEditing({...editing,product:e.target.value})} style={{ flex:'1 1 160px' }} />
              <div className="field" style={{ flex:'0 1 170px' }}>
                <label>Voltaje de la receta (VAC)</label>
                <select className="input" value={editing.voltage} onChange={e=>setEditing({...editing,voltage:e.target.value})}>
                  {VOLTS.map(v=><option key={v} value={String(v)}>{v} VAC</option>)}
                </select>
              </div>
            </div>
            <div style={{ fontSize:11, color:'var(--muted)', marginBottom:10 }}>
              Cada receta es una fila del Excel: un modelo + un voltaje. Recetas de fábrica: <b>RL-120, RL-208, RL-220, RL-240</b>.
            </div>

            <div style={{ overflow:'auto', border:'1px solid var(--line)', borderRadius:8 }}>
              <table className="grid" style={{ minWidth:980, fontSize:12 }}>
                <thead>
                  <tr><th style={{ minWidth:60, background:'var(--text)', color:'#fff' }}>V</th>{COMPS.map(c=><th key={c} colSpan={3} style={{ textAlign:'center', background:'var(--text)', color:'#fff', minWidth:138 }}>{c}</th>)}<th colSpan={2} style={{ textAlign:'center', background:'var(--accent)', color:'#fff', minWidth:138 }}>TOTAL</th></tr>
                  <tr><th style={{ background:'var(--surface-2)' }}></th>{COMPS.map(c=><React.Fragment key={c}><th style={{ background:'var(--surface-2)', fontSize:11 }}>MIN</th><th style={{ background:'var(--surface-2)', fontSize:11 }}>MAX</th><th style={{ background:'#fdf1d0', fontSize:11, minWidth:36 }}></th></React.Fragment>)}<th style={{ background:'#fdf1d0', fontSize:11 }}>MIN</th><th style={{ background:'#fdf1d0', fontSize:11 }}>MAX</th></tr>
                </thead>
                <tbody>
                  <tr key={editVoltage}>
                    <td style={{ fontWeight:800, background:'#f8fafc', textAlign:'center' }}>{editVoltage}</td>
                    {COMPS.map(c=>{
                      const cell=editing.m[editVoltage][c];
                      return <React.Fragment key={c}>
                        <td style={{ padding:3 }}><input disabled={!cell.enabled} value={cell.min} onChange={e=>setEditing({ ...editing, m:{...editing.m,[editVoltage]:{...editing.m[editVoltage],[c]:{...cell,min:e.target.value}}} })} placeholder="—" style={{ width:56, height:30, border:`1px solid ${cell.enabled?'var(--accent)':'var(--line)'}`, borderRadius:6, textAlign:'center', fontSize:12, background: cell.enabled?'#fff':'var(--surface-2)' }} /></td>
                        <td style={{ padding:3 }}><input disabled={!cell.enabled} value={cell.max} onChange={e=>setEditing({ ...editing, m:{...editing.m,[editVoltage]:{...editing.m[editVoltage],[c]:{...cell,max:e.target.value}}} })} placeholder="—" style={{ width:56, height:30, border:`1px solid ${cell.enabled?'var(--accent)':'var(--line)'}`, borderRadius:6, textAlign:'center', fontSize:12, background: cell.enabled?'#fff':'var(--surface-2)' }} /></td>
                        <td style={{ padding:3, textAlign:'center' }}><input type="checkbox" checked={cell.enabled} onChange={e=>setEditing({...editing,m:{...editing.m,[editVoltage]:{...editing.m[editVoltage],[c]:{...cell,enabled:e.target.checked}}}})} style={{ width:18, height:18 }} title="Activar/desactivar este componente" /></td>
                      </React.Fragment>;
                    })}
                    <td style={{ padding:3 }}><input value={editing.t[editVoltage].min} onChange={e=>setEditing({...editing,t:{...editing.t,[editVoltage]:{...editing.t[editVoltage],min:e.target.value}}})} placeholder="—" style={{ width:56, height:30, border:'1px solid var(--line)', borderRadius:6, textAlign:'center', fontSize:12 }} /></td>
                    <td style={{ padding:3 }}><input value={editing.t[editVoltage].max} onChange={e=>setEditing({...editing,t:{...editing.t,[editVoltage]:{...editing.t[editVoltage],max:e.target.value}}})} placeholder="—" style={{ width:56, height:30, border:'1px solid var(--line)', borderRadius:6, textAlign:'center', fontSize:12 }} /></td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div className="card" style={{ marginTop: 10, background: 'var(--surface-2)' }}>
              <h3 style={{ fontSize: 14, marginBottom: 8 }}>Parámetros Hipot</h3>
              <div className="row" style={{ alignItems: 'flex-end', gap: 10 }}>
                <div className="field" style={{ flex: '1 1 140px' }}>
                  <label>Voltaje de prueba (V)</label>
                  <input className="input" type="number" inputMode="decimal" placeholder="Ej. 500"
                    value={editing.hipotVoltage}
                    onChange={e => setEditing({ ...editing, hipotVoltage: e.target.value })} />
                </div>
                <div className="field" style={{ flex: '1 1 160px' }}>
                  <label>Límite de corriente (mA, ACW)</label>
                  <input className="input" type="number" inputMode="decimal" placeholder="Ej. 3.5"
                    value={editing.hipotCurrent}
                    onChange={e => setEditing({ ...editing, hipotCurrent: e.target.value })} />
                </div>
                <div className="field" style={{ flex: '1 1 140px' }}>
                  <label>Tiempo de prueba / Dwell (s)</label>
                  <input className="input" type="number" inputMode="decimal" placeholder="Ej. 60"
                    value={editing.hipotTime}
                    onChange={e => setEditing({ ...editing, hipotTime: e.target.value })} />
                </div>
              </div>
              <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 6 }}>
                Se usan para crear el paso <b>ACW</b> del 3865 con <code>ADD</code> antes de cada <code>TEST</code>.
                Vacío o 0 = el ciclo Hipot falla por falta de configuración. El límite de corriente va en <b>mA</b>
                (rango ACW del 3865: 0–20 mA).
              </div>
            </div>

            <div style={{ fontSize:11, color:'var(--muted)', marginTop:6 }}>Marque  para incluir ese componente en ese voltaje. Desmarcado = esa parte no se prueba.</div>
            <div style={{ fontSize:11, color:'var(--muted)', marginTop:2 }}>
              PLC: el máximo de corriente elige el canal <b>A</b> (≤1 A), <b>B</b> (≤10 A) o <b>C</b> (&gt;10 A).
              Voltajes 110/120 V usan la salida de 110 VAC; 208/220/240 V usan 220 VAC.
            </div>
            <div style={{ fontSize:11, color:'var(--muted)' }}>Se guarda con fecha/hora y usuario: <b>{(adminBatch||'ADMIN').toUpperCase()}</b></div>
            <div style={{ display:'flex', gap:8, marginTop:12, flexWrap:'wrap', justifyContent:'flex-end' }}>
              <button className="btn" onClick={()=>setEditing(null)}>Cancelar</button>
              <button className="btn primary" onClick={doSave}>{editing.id?'Guardar cambios':'Crear receta'}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function PlcTab() {
  const [st, setSt] = useState<any>(null);
  const [cfg, setCfg] = useState<{ ip: string; rack: number; slot: number }>({
    ip: '192.168.1.10',
    rack: 0,
    slot: 1,
  });
  const [enabled, setEnabled] = useState(false);
  const [tags, setTags] = useState<any[]>([]);
  const [writeVals, setWriteVals] = useState<Record<number, string>>({});
  const [msg, setMsg] = useState('');

  const loadConfig = () => {
    invoke<any>('plc_config_get')
      .then((res: any) => { if (res) setCfg(res); })
      .catch(() => {});
    invoke<any>('get_config')
      .then((c: any) => setEnabled(c?.plc_enabled === '1'))
      .catch(() => {});
  };

  const loadTags = () => {
    invoke<any>('plc_tags_list')
      .then(setTags)
      .catch(() => setTags([]));
  };

  useEffect(() => {
    loadConfig();
    loadTags();
    const interval = setInterval(loadTags, 2500);
    return () => clearInterval(interval);
  }, []);

  const saveConfig = async () => {
    try {
      const r = await invoke<string>('plc_config_set', { ip: cfg.ip, rack: cfg.rack, slot: cfg.slot });
      if (r !== 'saved') throw new Error(r);
      const r2 = await invoke<string>('set_config', { values: { plc_enabled: enabled ? '1' : '0' } });
      if (r2 !== 'saved') throw new Error(r2);
      setMsg(enabled ? 'PLC guardado y habilitado' : 'PLC guardado (deshabilitado)');
      invoke<any>('plc_status_info').then(setSt).catch(() => {});
    } catch (e: any) {
      setMsg(`Error al guardar: ${String(e?.message ?? e)}`);
    }
  };

  const doWrite = async (tag: any) => {
    const valStr = writeVals[tag.id];
    if (valStr === undefined || valStr === '') {
      setMsg(`Ingrese un valor para escribir en ${tag.name}`);
      return;
    }
    try {
      const r = await invoke<string>('plc_tag_write_by_id', { tagId: tag.id, value: valStr });
      if (r !== 'OK') throw new Error(r);
      setMsg(`Escrito exitosamente en ${tag.name}: ${valStr}`);
      setWriteVals({ ...writeVals, [tag.id]: '' });
      loadTags();
    } catch (e: any) {
      setMsg(`Error al escribir en ${tag.name}: ${String(e?.message ?? e)}`);
    }
  };

  const doDeleteTag = async (tag: any) => {
    if (!await askConfirm(`¿Eliminar variable ${tag.name}?`)) return;
    try {
      const r = await invoke<string>('plc_tags_delete', { id: tag.id });
      if (r !== 'deleted') throw new Error(r);
      setMsg(`Variable ${tag.name} eliminada`);
      loadTags();
    } catch (e: any) {
      setMsg(`Error al eliminar: ${String(e?.message ?? e)}`);
    }
  };

  const testConnection = async () => {
    setMsg('');
    try {
      const r = await invoke<any>('plc_probe');
      setSt(r);
      setMsg(r?.connected ? 'Conexión PLC correcta' : (r?.error || 'PLC no conectado'));
    } catch (e: any) {
      setMsg(`Error al probar PLC: ${String(e?.message ?? e)}`);
    }
  };

  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>PLC Siemens S7-1200 — Red, Lectura en Vivo y Escritura</h2>
      {msg && (
        <div className="toast-err" style={{ background: msg.startsWith('Configuración') || msg.startsWith('Escrito') || msg.startsWith('Variable') ? '#e6f4ea' : undefined, borderColor: msg.startsWith('Configuración') || msg.startsWith('Escrito') ? '#a8d0b4' : undefined, color: msg.startsWith('Configuración') || msg.startsWith('Escrito') ? 'var(--ok)' : undefined }}>
          {msg}
        </div>
      )}

      {/* Configuración de Red PLC */}
      <div className="card" style={{ marginBottom: 12 }}>
        <h3 style={{ margin: '0 0 10px 0', fontSize: 14 }}>Parámetros de Conexión de Red</h3>
        <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', alignItems: 'center' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <label style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-2)' }}>IP del PLC</label>
            <input className="input" placeholder="192.168.1.10" value={cfg.ip} onChange={e => setCfg({ ...cfg, ip: e.target.value })} style={{ width: 140 }} />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <label style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-2)' }}>Rack</label>
            <input className="input" type="number" inputMode="numeric" value={cfg.rack} onChange={e => setCfg({ ...cfg, rack: parseInt(e.target.value) || 0 })} style={{ width: 64 }} />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <label style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-2)' }}>Slot</label>
            <input className="input" type="number" inputMode="numeric" value={cfg.slot} onChange={e => setCfg({ ...cfg, slot: parseInt(e.target.value) || 1 })} style={{ width: 64 }} />
          </div>
          <label style={{ display: 'flex', gap: 6, alignItems: 'center', marginTop: 16, fontWeight: 600 }}>
            <input type="checkbox" checked={enabled} onChange={e => setEnabled(e.target.checked)} style={{ width: 18, height: 18 }} />
            Habilitar PLC
          </label>
          <div style={{ display: 'flex', gap: 6, marginTop: 16 }}>
            <button className="btn primary" onClick={saveConfig}>Guardar Red PLC</button>
            <button className="btn" onClick={testConnection}>Probar Conexión</button>
          </div>
        </div>
        {st && (
          <div className="card" style={{ marginTop: 10, background: st.connected ? '#e6f4ea' : '#fef2f2', borderColor: st.connected ? '#a8d0b4' : '#fecaca' }}>
            <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
              <span className={`dot ${st.connected ? 'ok' : 'bad'}`} />
              <b>{st.connected ? 'PLC conectado' : st.enabled === false ? 'PLC deshabilitado' : 'PLC no conectado'}</b>
              <span className="mono" style={{ fontSize: 12 }}>{st.ip}{st.rack != null ? ` · Rack ${st.rack} · Slot ${st.slot}` : ''}</span>
            </div>
            {st.connected && (
              <div style={{ fontSize: 12, marginTop: 6 }}>
                {st.enabled === false
                  ? 'La IP responde. Marca “Habilitar PLC” y guarda para usarlo en las pruebas.'
                  : 'La IP responde correctamente.'}
              </div>
            )}
            {!st.connected && st.error && <div style={{ fontSize: 12, marginTop: 6, color: 'var(--bad)' }}>{st.error}</div>}
            <details style={{ marginTop: 6 }}>
              <summary style={{ cursor: 'pointer', fontSize: 11, color: 'var(--muted)' }}>Detalles técnicos</summary>
              <pre style={{ fontSize: 11, margin: '6px 0 0 0', background: 'var(--surface-2)', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(st, null, 2)}</pre>
            </details>
          </div>
        )}
        <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 8 }}>
          Si está deshabilitado, la app no intentará conectarse al PLC ni bloqueará la interfaz. Habilítelo después de verificar la IP con “Probar Conexión”.
        </div>
      </div>

      {/* Tabla de Variables PLC */}
      <div className="card" style={{ overflow: 'auto' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 10 }}>
          <h3 style={{ margin: 0, fontSize: 14 }}>Variables Configuradas (Lectura y Escritura por Renglón)</h3>
          <button className="btn sm" onClick={loadTags}>Actualizar valores</button>
        </div>

        <table className="grid">
          <thead>
            <tr>
              <th>Nombre</th>
              <th>DB</th>
              <th>Offset</th>
              <th>Tipo</th>
              <th>Valor Leído (En vivo)</th>
              <th style={{ minWidth: 200 }}>Escribir Valor</th>
              <th>Acciones</th>
            </tr>
          </thead>
          <tbody>
            {tags.map((t: any) => (
              <tr key={t.id}>
                <td><b>{t.name}</b></td>
                <td>DB{t.db}</td>
                <td>{t.offset}</td>
                <td><span className="pill" style={{ fontSize: 12 }}>{t.type}</span></td>
                <td>
                  <span style={{ fontWeight: 800, color: '#0f172a', background: '#e2e8f0', padding: '4px 8px', borderRadius: 4, fontFamily: 'monospace' }}>
                    {t.value !== null && t.value !== undefined ? String(t.value) : '—'}
                  </span>
                </td>
                <td>
                  <div style={{ display: 'flex', gap: 6 }}>
                    <input
                      className="input sm"
                      placeholder="Nuevo valor"
                      value={writeVals[t.id] ?? ''}
                      onChange={e => setWriteVals({ ...writeVals, [t.id]: e.target.value })}
                      style={{ flex: 1, minWidth: 80 }}
                    />
                    <button className="btn sm primary" onClick={() => doWrite(t)}>Escribir</button>
                  </div>
                </td>
                <td>
                  <button className="btn sm danger" onClick={() => doDeleteTag(t)}>Borrar</button>
                </td>
              </tr>
            ))}
            {!tags.length && (
              <tr>
                <td colSpan={7} style={{ textAlign: 'center', color: 'var(--muted)' }}>
                  No hay variables de PLC configuradas. Agregue una abajo.
                </td>
              </tr>
            )}
          </tbody>
        </table>

        <TagForm onDone={loadTags} />
      </div>
    </div>
  );
}

function TagForm({ onDone }: { onDone: () => void }) {
  const [f, setF] = useState({ name: '', db: '1', offset: '0', type: 'REAL', bit: '0' });
  const submit = async () => {
    if (!f.name.trim()) return;
    await invoke('plc_tags_add', {
      name: f.name.toUpperCase().trim(),
      dbNumber: parseInt(f.db) || 1,
      offset: parseInt(f.offset) || 0,
      dataType: f.type,
      bit: parseInt(f.bit) || 0,
    });
    setF({ name: '', db: '1', offset: '0', type: 'REAL', bit: '0' });
    onDone();
  };
  return (
    <div style={{ marginTop: 12, paddingTop: 12, borderTop: '1px solid #e2e8f0' }}>
      <h4 style={{ margin: '0 0 8px 0', fontSize: 13 }}>+ Agregar Nueva Variable</h4>
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', minWidth: 0 }}>
        <input className="input" placeholder="Nombre (ej. MODO_PRUEBA)" value={f.name} onChange={e => setF({ ...f, name: e.target.value })} style={{ flex: '2 1 140px' }} />
        <input className="input" placeholder="DB (ej. 1)" value={f.db} onChange={e => setF({ ...f, db: e.target.value })} style={{ flex: '1 1 60px' }} />
        <input className="input" placeholder="Offset" value={f.offset} onChange={e => setF({ ...f, offset: e.target.value })} style={{ flex: '1 1 70px' }} />
        <select className="input" value={f.type} onChange={e => setF({ ...f, type: e.target.value })} style={{ flex: '1 1 90px' }}>
          <option value="REAL">REAL (Float)</option>
          <option value="WORD">WORD / INT</option>
          <option value="BOOL">BOOL</option>
        </select>
        <button className="btn primary" onClick={submit}>Agregar Variable</button>
      </div>
    </div>
  );
}

function HipotTab() {
  const [st, setSt] = useState<any>(null);
  const [cfg, setCfg] = useState<{ port: string; baud: number }>({
    port: 'COM3', baud: 38400,
  });
  const [ports, setPorts] = useState<string[]>([]);
  const [enabled, setEnabled] = useState(false);
  const [cmd, setCmd] = useState('*IDN?');
  const [resp, setResp] = useState<any>(null);
  const [msg, setMsg] = useState('');
  const [logs, setLogs] = useState<any[]>([]);
  const loadLogs = () => invoke<any[]>('list_logs', { category: 'HIPOT', level: '', q: '', limit: 30 })
    .then(setLogs).catch(() => setLogs([]));

  useEffect(() => {
    invoke<any>('hipot_config_get').then(c => c && setCfg(c)).catch(() => {});
    invoke<string[]>('hipot_list_ports').then(p => setPorts(p || [])).catch(() => {});
    invoke<any>('get_config').then(c => setEnabled(c?.hipot_enabled === '1')).catch(() => {});
    loadLogs();
    const id = window.setInterval(loadLogs, 3000);
    return () => window.clearInterval(id);
  }, []);

  const save = async () => {
    try {
      const r = await invoke<string>('hipot_config_set', { port: cfg.port, baud: cfg.baud });
      if (r !== 'saved') throw new Error(r);
      const r2 = await invoke<string>('set_config', { values: { hipot_enabled: enabled ? '1' : '0' } });
      if (r2 !== 'saved') throw new Error(r2);
      setMsg(enabled ? 'Configuración Hipot guardada y habilitada' : 'Configuración Hipot guardada (deshabilitada)');
      invoke<any>('hipot_status').then(setSt).catch(() => {});
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };

  const send = async () => {
    try { setResp(await invoke<any>('hipot_command', { command: cmd })); }
    catch (e: any) { setResp({ error: String(e?.message ?? e) }); }
    finally { loadLogs(); }
  };

  const sendCmd = async (c: string) => {
    setCmd(c);
    try { setResp(await invoke<any>('hipot_command', { command: c })); }
    catch (e: any) { setResp({ error: String(e?.message ?? e) }); }
    finally { loadLogs(); }
  };

  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>Hipot 3865 - USB/RS-232</h2>
      <SafetyNote compact />
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Configuración') ? '#e6f4ea' : undefined, borderColor: msg.startsWith('Configuración') ? '#a8d0b4' : undefined, color: msg.startsWith('Configuración') ? 'var(--ok)' : undefined }}>{msg}</div>}

      <div className="card" style={{ marginBottom: 10 }}>
        <h3 style={{ margin: '0 0 8px 0', fontSize: 14 }}>Puerto serie</h3>
        <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', alignItems: 'center' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <label style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-2)' }}>Puerto</label>
            <select className="input" value={cfg.port} onChange={e => setCfg({ ...cfg, port: e.target.value })} style={{ width: 140 }}>
              {ports.map(p => <option key={p} value={p}>{p}</option>)}
              {!ports.includes(cfg.port) && <option value={cfg.port}>{cfg.port}</option>}
            </select>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <label style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-2)' }}>Baudios</label>
            <input className="input" type="number" inputMode="numeric" value={cfg.baud} onChange={e => setCfg({ ...cfg, baud: parseInt(e.target.value) || 38400 })} style={{ width: 100 }} />
          </div>
          <label style={{ display: 'flex', gap: 6, alignItems: 'center', marginTop: 16, fontWeight: 600 }}>
            <input type="checkbox" checked={enabled} onChange={e => setEnabled(e.target.checked)} style={{ width: 18, height: 18 }} />
            Habilitar Hipot
          </label>
          <div style={{ display: 'flex', gap: 6, marginTop: 16, flexWrap: 'wrap' }}>
            <button className="btn primary" onClick={save}>Guardar</button>
            <button className="btn" onClick={() => { invoke<any>('hipot_probe').then(setSt).catch(() => {}); loadLogs(); }}>Probar Hipot</button>
            <button className="btn" onClick={() => invoke('hipot_close').catch(() => {})}>Cerrar puerto</button>
            <button className="btn" onClick={async () => {
              const c = await invoke<any>('get_config').catch(() => ({}));
              const ip = c?.printer_ip || '';
              const port = parseInt(c?.printer_port || '9100', 10);
              const name = c?.printer_name || 'Zebra ZT231';
              const r = await invoke<string>('print_test_zebra', { printerIp: ip, printerPort: port, printerName: name });
              notify(r, r === 'OK' ? 'ok' : 'error');
            }}>Probar Zebra</button>
          </div>
        </div>
        <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 8 }}>
          3865: 38400 baud, 8N1, sin handshaking. Terminador LF. Respuesta ACK/NAK.
          Comandos reales: <code>*IDN?</code>, <code>TD?</code>, <code>TEST</code>, <code>RESET</code>, <code>FL</code>, <code>ADD</code>.
        </div>
        {st && <pre style={{ marginTop: 8, fontSize: 12, background: 'var(--surface-2)', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(st, null, 2)}</pre>}
      </div>

      <div className="card">
        <h3>Comando manual</h3>
        <div className="row" style={{ marginBottom: 8, flexWrap: 'wrap' }}>
          {['*IDN?', 'TD?', 'LF?', 'ST?', 'TEST', 'RESET'].map(c => (
            <button key={c} className="btn sm" onClick={() => sendCmd(c)}>{c}</button>
          ))}
        </div>
        <div style={{ display: 'flex', gap: 8, minWidth: 0 }}>
          <input className="input" value={cmd} onChange={e => setCmd(e.target.value.toUpperCase())} style={{ flex: 1, minWidth: 0 }} />
          <button className="btn primary" style={{ flexShrink: 0 }} onClick={send}>Enviar</button>
        </div>
        <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 6 }}>
          <b>TEST</b> inicia alto voltaje. <b>RESET</b> detiene/aborta. El comando manual no reemplaza la secuencia automática.
        </div>
        {resp && <pre style={{ marginTop: 8, fontSize: 12, background: 'var(--surface-2)', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(resp, null, 2)}</pre>}
      </div>

      <div className="card" style={{ marginTop: 10 }}>
        <div className="row" style={{ justifyContent: 'space-between', marginBottom: 8 }}>
          <h3 style={{ margin: 0 }}>Últimos comandos Hipot</h3>
          <button className="btn sm" onClick={loadLogs}>Actualizar</button>
        </div>
        <div className="table-wrap" style={{ maxHeight: 260 }}>
          <table className="grid">
            <thead><tr><th>Hora</th><th>Acción</th><th>Enviado</th><th>Recibido</th><th>Error</th></tr></thead>
            <tbody>
              {logs.map((l: any) => (
                <tr key={l.id}>
                  <td className="mono" style={{ whiteSpace: 'nowrap' }}>{(l.ts || '').slice(11, 19)}</td>
                  <td>{l.action}</td>
                  <td className="mono" style={{ maxWidth: 170, overflow: 'hidden', textOverflow: 'ellipsis' }}>{l.value_sent || '—'}</td>
                  <td className="mono" style={{ maxWidth: 300, overflow: 'hidden', textOverflow: 'ellipsis' }}>{l.value_received || '—'}</td>
                  <td style={{ color: 'var(--bad)' }}>{l.error_message || ''}</td>
                </tr>
              ))}
              {!logs.length && <tr><td colSpan={5} style={{ textAlign: 'center', color: 'var(--muted)' }}>Sin comandos registrados</td></tr>}
            </tbody>
          </table>
        </div>
        <div className="page-sub" style={{ marginTop: 6 }}>También en Logs → categoría HIPOT.</div>
      </div>
    </div>
  );
}

function DbTab() {
  const [cfg, setCfg] = useState<any>(null);
  const [log, setLog] = useState('');
  const [syncs, setSyncs] = useState<any[]>([]);
  const load = () => { invoke<any>('get_config').then(setCfg).catch(() => setCfg(null)); invoke<any[]>('sync_log_list', { limit: 20 }).then(setSyncs).catch(() => setSyncs([])); };
  useEffect(() => { load(); }, []);
  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>Base de datos</h2>
      <div className="card" style={{ marginBottom: 10 }}>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <button className="btn primary" onClick={async () => {
            try {
              const driver = 'ODBC Driver 17 for SQL Server';
              const server = cfg.sql_server || '192.168.1.200';
              const db = cfg.sql_database || 'HiPotDB_Central';
              const user = cfg.sql_user || '';
              const pwd = cfg.sql_password || '';
              const connStr = user ? `Driver={${driver}};Server=${server};Database=${db};Uid=${user};Pwd=${pwd};` : `Driver={${driver}};Server=${server};Database=${db};Trusted_Connection=yes;`;
              const r = await invoke<string>('sync_pending', { remoteConnStr: connStr });
              setLog(r); load();
            } catch (e: any) { setLog(String(e?.message || e)); }
          }}>Sincronizar ahora</button>
          <button className="btn" onClick={() => invoke<any>('app_health').then(r => setLog(JSON.stringify(r, null, 2))).catch(e => setLog(String(e?.message || e)))}>Consultar estado</button>
        </div>
        {log && <pre style={{ marginTop: 8, fontSize: 12, background: 'var(--surface-2)', padding: 8, borderRadius: 6, overflow: 'auto' }}>{log}</pre>}
        <details style={{ marginTop: 8 }}><summary style={{ cursor: 'pointer', fontSize: 13 }}>Config actual</summary><pre style={{ fontSize: 12, background: 'var(--surface-2)', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(cfg, null, 2)}</pre></details>
      </div>
      <div className="card" style={{ overflow: 'auto' }}>
        <h3>Sync log</h3>
        <table className="grid"><thead><tr><th>#</th><th>Dirección</th><th>Tabla</th><th>N</th><th>Estado</th></tr></thead>
          <tbody>{syncs.map((s: any) => <tr key={s.id}><td>{s.id}</td><td>{s.dir}</td><td>{s.table}</td><td>{s.n}</td><td>{s.status}</td></tr>)}</tbody>
        </table>
      </div>
    </div>
  );
}

function LabelsTab() {
  const [list, setList] = useState<any[]>([]);
  const [selId, setSelId] = useState<number | null>(null);
  const [name, setName] = useState('Nueva plantilla');
  const [isDef, setIsDef] = useState(false);
  const [objs, setObjs] = useState<LObj[] | null>(null);
  const [msg, setMsg] = useState('');
  const [key, setKey] = useState(0);
  const load = () => invoke<any[]>('template_list').then(setList).catch(() => setList([]));
  useEffect(() => { load(); }, []);
  const openNew = () => { setSelId(null); setObjs(null); setName('Nueva plantilla'); setIsDef(false); setKey(k => k + 1); };
  const openTpl = async (id: number) => {
    try {
      const t = await invoke<any>('template_get', { tid: id });
      setSelId(id); setName(t.name); setIsDef(!!t.is_default);
      setObjs(templateToObjs(t.canvas_data || '{"objects":[]}')); setKey(k => k + 1);
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };
  const save = async (o: LObj[]) => {
    const canvas = objsToTemplate(o);
    try {
      if (selId) {
        await invoke('template_update', {
          tid: selId, name, widthMm: 100, heightMm: 60,
          canvasData: canvas, isDefault: isDef,
        });
      } else {
        const id = await invoke<number>('template_create', {
          name, widthMm: 100, heightMm: 60,
          canvasData: canvas, isDefault: isDef,
        });
        setSelId(id);
      }
      setMsg('Plantilla guardada'); load();
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };
  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>Etiquetas Zebra</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Plantilla') ? '#e6f4ea' : undefined, borderColor: msg.startsWith('Plantilla') ? '#a8d0b4' : undefined, color: msg.startsWith('Plantilla') ? 'var(--ok)' : undefined }}>{msg}</div>}
      <div className="card" style={{ marginBottom: 10 }}>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', minWidth: 0 }}>
          <select className="input" style={{ maxWidth: 260, flex: '1 1 200px' }} value={selId ?? ''} onChange={e => e.target.value ? openTpl(+e.target.value) : openNew()}>
            <option value="">— Nueva —</option>{list.map((t: any) => <option key={t.id} value={t.id}>{t.name}{t.is_default ? ' · default' : ''}</option>)}
          </select>
          <input className="input" style={{ maxWidth: 240, flex: '1 1 180px' }} value={name} onChange={e => setName(e.target.value)} />
          <label style={{ display: 'flex', gap: 6, alignItems: 'center', fontWeight: 600, fontSize: 13, flexShrink: 0 }}>
            <input type="checkbox" checked={isDef} onChange={e => setIsDef(e.target.checked)} style={{ width: 18, height: 18 }} /> Default
          </label>
          {selId && <button className="btn danger sm" style={{ flexShrink: 0 }} onClick={() => invoke('template_delete', { tid: selId }).then(() => { openNew(); load(); }).catch((e:any) => setMsg(String(e?.message ?? e)))}>Eliminar</button>}
        </div>
      </div>
      <LabelEditor key={key} initial={objs ?? undefined} onSave={save} />
    </div>
  );
}

function LogsTab() {
  const [logs, setLogs] = useState<any[]>([]);
  const [cat, setCat] = useState('');
  const [level, setLevel] = useState('');
  const [q, setQ] = useState('');
  const load = () => invoke<any>('list_logs', { category: cat, level, q, limit: 200 }).then(setLogs).catch(() => setLogs([]));
  useEffect(() => { load(); }, []);
  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>Logs</h2>
      <div className="card" style={{ marginBottom: 10 }}>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', minWidth: 0 }}>
          <select className="input" style={{ maxWidth: 150, flex: '1 1 130px' }} value={cat} onChange={e => setCat(e.target.value)}><option value="">Todas CAT</option>{['UI','PLC','HIPOT','DB','SYNC','BARCODE','SYSTEM'].map(c => <option key={c} value={c}>{c}</option>)}</select>
          <select className="input" style={{ maxWidth: 130, flex: '1 1 110px' }} value={level} onChange={e => setLevel(e.target.value)}><option value="">Todos</option>{['DEBUG','INFO','WARN','ERROR'].map(c => <option key={c} value={c}>{c}</option>)}</select>
          <input className="input" placeholder="Buscar…" value={q} onChange={e => setQ(e.target.value)} style={{ flex: '1 1 160px', minWidth: 0 }} />
          <button className="btn primary" style={{ flexShrink: 0 }} onClick={load}>Buscar</button>
          <button className="btn" style={{ flexShrink: 0 }} onClick={async () => {
            if (!await askConfirm('¿Eliminar logs con más de 30 días y conservar solo los últimos 5000?')) return;
            try {
              const r = await invoke<any>('logs_prune', { days: 30, keep: 5000 });
              notify(`Logs restantes: ${r.remaining ?? '?'}`);
              load();
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Limpiar antiguos</button>
        </div>
      </div>
      <div className="card" style={{ maxHeight: 420, overflow: 'auto', fontSize: 12, minWidth: 0 }}>
        {logs.map(l => (
          <div key={l.id} style={{ borderBottom: '1px solid #e8edf3', padding: '6px 0', minWidth: 0 }}>
            <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', alignItems: 'center' }}>
              <b>#{l.id}</b>
              <span style={{ color: 'var(--muted)' }}>{l.ts}</span>
              <span className={`pill ${l.level === 'ERROR' ? 'off' : l.level === 'WARN' ? 'accent' : ''}`} style={{ fontSize: 12 }}>{l.level}</span>
              <span className="pill" style={{ fontSize: 12 }}>{l.cat}</span>
              <b>{l.action}</b>
              {l.execution_id && <span style={{ fontSize: 12, color: 'var(--muted)' }}>Exec #{l.execution_id}</span>}
              {l.duration_ms != null && <span style={{ fontSize: 12, color: 'var(--muted)' }}>{l.duration_ms}ms</span>}
              {l.success === 0 && <span className="pill off" style={{ fontSize: 12 }}>FAIL</span>}
            </div>
            {l.detail && <div style={{ fontSize: 12, color: 'var(--text-2)', marginTop: 2, wordBreak: 'break-word' }}>{l.detail}</div>}
            {(l.value_sent || l.value_received) && (
              <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 2, fontFamily: 'monospace' }}>
                {l.value_sent && <span>Enviado: {l.value_sent} </span>}
                {l.value_received && <span>Recibido: {l.value_received}</span>}
              </div>
            )}
            {l.error_message && <div style={{ fontSize: 12, color: 'var(--bad)', marginTop: 2 }}>{l.error_message}</div>}
          </div>
        ))}
      </div>
    </div>
  );
}

function SystemTab() {
  const [msg, setMsg] = useState('');
  const [impLog, setImpLog] = useState<any[]>([]);
  const [mode, setMode] = useState('overwrite');
  const [hold, setHold] = useState('10');
  const [holdMsg, setHoldMsg] = useState('');
  const [appVersion, setAppVersion] = useState('');
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateStatus, setUpdateStatus] = useState('');
  const [plcStatus, setPlcStatus] = useState<any>(null);
  const [hipotStatus, setHipotStatus] = useState<any>(null);
  const [cfg, setCfg] = useState<any>({});
  const [localDbCfg, setLocalDbCfg] = useState<any>({ use_sqlite: '0', server: '.\\SQLEXPRESS', name: 'HiPotDB', user: '', password: '' });
  const [tableNames, setTableNames] = useState<string[]>([]);
  const [previewStatus, setPreviewStatus] = useState<'GOOD' | 'BAD' | null>(null);
  const [kbMode, setKbMode] = useState(() => localStorage.getItem('keyboard_mode') || 'integrated');
  const [contrast, setContrast] = useState(() => localStorage.getItem('contrast_mode') === '1');
  const [estopInput, setEstopInput] = useState('I0.0');
  const [estopNc, setEstopNc] = useState('1');
  const [estopSource, setEstopSource] = useState('DB');
  const [estopDb, setEstopDb] = useState('100');
  const [estopOffset, setEstopOffset] = useState('0');
  const [estopBit, setEstopBit] = useState('0');
  const remoteConnStr = () => {
    const driver = 'ODBC Driver 17 for SQL Server';
    const server = cfg?.sql_server || '192.168.1.200';
    const db = cfg?.sql_database || 'HiPotDB_Central';
    const user = cfg?.sql_user || '';
    const pwd = cfg?.sql_password || '';
    return user
      ? `Driver={${driver}};Server=${server};Database=${db};Uid=${user};Pwd=${pwd};`
      : `Driver={${driver}};Server=${server};Database=${db};Trusted_Connection=yes;`;
  };
  const loadLog = () => invoke<any[]>('import_export_log_list', { limit: 20 }).then(setImpLog).catch(() => setImpLog([]));
  useEffect(() => {
    loadLog();
    invoke<any>('get_config').then(c => {
      setHold(String(c?.result_hold_seconds ?? '10'));
      setCfg(c || {});
      setEstopInput(String(c?.estop_input ?? 'I0.0'));
      setEstopNc(String(c?.estop_nc ?? '1'));
      setEstopSource(String(c?.estop_source ?? 'DB').toUpperCase());
      setEstopDb(String(c?.estop_db ?? '100'));
      setEstopOffset(String(c?.estop_offset ?? '0'));
      setEstopBit(String(c?.estop_bit ?? '0'));
    }).catch(() => {});
    invoke<any>('local_db_config_get').then(r => setLocalDbCfg(r || { use_sqlite: '0', server: '.\\SQLEXPRESS', name: 'HiPotDB', user: '', password: '' })).catch(() => {});
    invoke<any>('audit_table_names').then(r => setTableNames(r?.tables ?? [])).catch(() => {});
   
    getVersion().then(setAppVersion).catch(() => setAppVersion('1.0.0'));
    invoke<any>('plc_status_info').then(setPlcStatus).catch(() => {});
    invoke<any>('hipot_status').then(setHipotStatus).catch(() => {});
  }, []);
  const doExport = async () => {
    try {
      const data = await invoke<string>('config_export');
      const blob = new Blob([data], { type: 'application/json' });
      const a = document.createElement('a'); a.href = URL.createObjectURL(blob);
      a.download = `config_export_${new Date().toISOString().slice(0,10)}.json`; a.click();
      loadLog();
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };
  const doImport = async (f: File) => {
    try {
      const t = await f.text();
      const r: any = await invoke('config_import', { data: t, mode });
      setMsg(`Import OK: ${r.imported} · Conflictos: ${(r.conflicts ?? []).length}`);
      loadLog();
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };
  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>Sistema</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Import OK') ? '#e6f4ea' : undefined }}>{msg}</div>}
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Espera en resultado</h3>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', minWidth: 0 }}>
          <span style={{ fontSize: 13, color: 'var(--text-2)' }}>Segundos antes de volver al inicio:</span>
          <input className="input" type="number" inputMode="numeric" min={3} max={60} value={hold} onChange={e => setHold(e.target.value)} style={{ maxWidth: 90 }} />
          <button className="btn primary" onClick={async () => {
            const n = Math.max(3, Math.min(60, parseInt(hold || '10', 10) || 10));
            await invoke('set_config', { values: { result_hold_seconds: String(n) } });
            setHold(String(n)); setHoldMsg(`Guardado: ${n}s`); setTimeout(() => setHoldMsg(''), 2000);
          }}>Guardar</button>
          {holdMsg && <span className="pill on" style={{ fontSize: 12 }}>{holdMsg}</span>}
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Actualizacion y version</h3>
        <div style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 8 }}>
          Version de la app: <span className="font-mono">{appVersion || '1.0.0'}</span>
        </div>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
          <button className="btn ok" disabled={checkingUpdate} onClick={async () => {
            setCheckingUpdate(true);
            setUpdateStatus('Buscando actualizaciones...');
            try {
              const update = await check();
              if (update) {
                setUpdateStatus(`Nueva version ${update.version}. Descargando...`);
                await update.downloadAndInstall();
                setUpdateStatus('Instalada. Reiniciando...');
                await relaunch();
              } else {
                setUpdateStatus('Ya tienes la ultima version.');
              }
            } catch (e: any) {
              setUpdateStatus('Error: ' + (e?.message || e));
            } finally {
              setCheckingUpdate(false);
            }
          }}>{checkingUpdate ? 'Buscando...' : 'Buscar actualizaciones'}</button>
          {updateStatus && <span style={{ fontSize: 12, color: 'var(--muted)' }}>{updateStatus}</span>}
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Datos de debug</h3>
        <div style={{ fontSize: 12, color: 'var(--text-2)', display: 'flex', flexDirection: 'column', gap: 4 }}>
          <div>Núcleo: <span className="font-mono">Rust (sin HTTP)</span></div>
          <div>PLC: <span className="font-mono">{plcStatus?.ip || cfg?.plc_ip || '-'}</span> ({plcStatus?.connected ? 'conectado' : 'desconectado'})</div>
          <div>Hipot: <span className="font-mono">{hipotStatus?.port || '-'}</span></div>
          <div>SQL: <span className="font-mono">{cfg?.sql_server || '-'}</span> / {cfg?.sql_database || '-'}</div>
          <button className="btn ghost" style={{ fontSize: 12, marginTop: 4 }} onClick={async () => {
            try {
              const h = await invoke<any>('app_health');
              let p: any = { connected: false };
              try {
                const c = await invoke<any>('get_config');
                p = { connected: await invoke<boolean>('plc_status', {
                  ip: c?.plc_ip || '192.168.1.10',
                  rack: parseInt(c?.plc_rack || '0', 10),
                  slot: parseInt(c?.plc_slot || '1', 10),
                }) };
              } catch {}
              let hp: any = { connected: false };
              try { hp = await invoke<any>('hipot_status'); } catch {}
              notify(`Health: ${JSON.stringify(h)}\nPLC: ${JSON.stringify(p)}\nHipot: ${JSON.stringify(hp)}`);
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Ver estado completo</button>
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Previsualizacion de resultado</h3>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
          <button className="btn ok" onClick={() => setPreviewStatus('GOOD')}>Previsualizar OK</button>
          <button className="btn danger" onClick={() => setPreviewStatus('BAD')}>Previsualizar MALO</button>
          {previewStatus && (
            <div style={{
              marginLeft: 12, padding: '6px 14px', borderRadius: 8, fontWeight: 700, fontSize: 14,
              background: previewStatus === 'GOOD' ? '#dcfce7' : '#fee2e2',
              color: previewStatus === 'GOOD' ? 'var(--ok)' : 'var(--bad)',
              border: `1px solid ${previewStatus === 'GOOD' ? '#86efac' : '#fca5a5'}`,
            }}>
              {previewStatus === 'GOOD' ? ' RESULTADO: BUENO' : ' RESULTADO: MALO'}
            </div>
          )}
        </div>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 6 }}>
          Solo previsualizacion. No se guarda ni se registra.
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Base de datos local</h3>
        <div style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 8 }}>
          Motor: <b>SQL Server Express</b> (obligatorio en producción)
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 8 }}>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Servidor</label>
            <input className="input" value={localDbCfg.server ?? ''} onChange={e => setLocalDbCfg({ ...localDbCfg, use_sqlite: '0', server: e.target.value })} placeholder=".\\SQLEXPRESS" inputMode="text" />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Base de datos</label>
            <input className="input" value={localDbCfg.name ?? ''} onChange={e => setLocalDbCfg({ ...localDbCfg, use_sqlite: '0', name: e.target.value })} placeholder="HiPotDB" inputMode="text" />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Usuario</label>
            <input className="input" value={localDbCfg.user ?? ''} onChange={e => setLocalDbCfg({ ...localDbCfg, use_sqlite: '0', user: e.target.value })} placeholder="(vacío = trusted)" inputMode="text" />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Contraseña</label>
            <input className="input" type="password" value={localDbCfg.password ?? ''} onChange={e => setLocalDbCfg({ ...localDbCfg, use_sqlite: '0', password: e.target.value })} placeholder="****" inputMode="text" />
          </div>
        </div>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
          <button className="btn ok" onClick={async () => {
            try {
              const r = await invoke<string>('local_db_config_set', { config: localDbCfg });
              notify(r === 'saved' ? 'Guardado. Reinicia la app para aplicar.' : r);
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Guardar</button>
          <button className="btn" onClick={async () => {
            try {
              const r = await invoke<string>('local_db_test_connection');
              notify(`SQL Express: ${r}`, 'ok');
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Probar SQL Express</button>
          <button className="btn" onClick={async () => {
            try {
              const r = await invoke<any>('local_db_ensure_tables');
              notify(`Tablas locales verificadas. Creadas: ${r.created ?? 0}`, 'ok');
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Crear tablas locales</button>
          <button className="btn" onClick={async () => {
            if (!await askConfirm('¿Migrar los datos de hipot_local.db (SQLite) a SQL Express? Se omite lo que ya exista.')) return;
            try {
              const r = await invoke<any>('local_db_migrate');
              notify(`Migración completada:\n${JSON.stringify(r, null, 2)}`, 'ok');
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Migrar SQLite → SQL Express</button>
          <button className="btn ghost" onClick={() => (window as any).__TAURI__?.core?.invoke?.('open_external', { url: 'https://www.microsoft.com/es-es/sql-server/sql-server-downloads' }).catch?.(() => window.open('https://www.microsoft.com/es-es/sql-server/sql-server-downloads', '_blank'))}>Descargar SQL Express 2025</button>
        </div>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 6 }}>
          Fase 7: prepara SQL Express, crea las tablas y migra los datos. La app seguirá usando SQLite hasta completar la migración de comandos (7B/7C).
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Tablas de la base de datos</h3>
        <div style={{ fontSize: 12, color: 'var(--text-2)', marginBottom: 6 }}>
          Tablas que se crean automaticamente (local y remota):
        </div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
          {tableNames.map(t => <span key={t} className="pill" style={{ fontSize: 12 }}>{t}</span>)}
          {!tableNames.length && <span style={{ fontSize: 12, color: 'var(--muted)' }}>Cargando...</span>}
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Base de datos remota (SQL Server)</h3>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 8 }}>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Servidor</label>
            <input className="input" value={cfg.sql_server ?? ''} onChange={e => setCfg({ ...cfg, sql_server: e.target.value })} placeholder="192.168.1.200" inputMode="text" />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Base de datos</label>
            <input className="input" value={cfg.sql_database ?? ''} onChange={e => setCfg({ ...cfg, sql_database: e.target.value })} placeholder="HiPotDB_Central" inputMode="text" />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Usuario</label>
            <input className="input" value={cfg.sql_user ?? ''} onChange={e => setCfg({ ...cfg, sql_user: e.target.value })} placeholder="sa" inputMode="text" />
          </div>
          <div>
            <label style={{ fontSize: 12, color: 'var(--text-2)' }}>Contraseña</label>
            <input className="input" type="password" value={cfg.sql_password ?? ''} onChange={e => setCfg({ ...cfg, sql_password: e.target.value })} placeholder="****" inputMode="text" />
          </div>
        </div>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
          <button className="btn ok" onClick={async () => {
            const vals: any = {};
            ['sql_server', 'sql_database', 'sql_user', 'sql_password'].forEach(k => { if (cfg[k] !== undefined) vals[k] = cfg[k]; });
            try { await invoke('set_config', { values: vals }); notify('Guardado', 'ok'); } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Guardar</button>
          <button className="btn ghost" onClick={async () => {
            try {
              const r = await invoke<any>('audit_ensure_tables', { connStr: remoteConnStr() });
              notify(r.exists ? 'Tablas creadas/verificadas' : 'No se pudo crear', r.exists ? 'ok' : 'error');
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Crear tablas en remoto</button>
          <button className="btn ghost" onClick={async () => {
            try {
              const r = await invoke<any>('audit_tables_status', { connStr: remoteConnStr() });
              notify(`Conexión: ${r.connected ? 'OK' : 'Sin conexión'}\nTabla: ${r.table_exists ? 'Existe' : 'No existe'}${r.error ? '\n' + r.error : ''}`);
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Verificar conexión</button>
          <button className="btn ghost" onClick={() => { (window as any).__TAURI__?.core?.invoke?.('open_external', { url: 'https://www.microsoft.com/es-es/sql-server/sql-server-downloads' }).catch?.(() => window.open('https://www.microsoft.com/es-es/sql-server/sql-server-downloads', '_blank')); }}>Descargar SQL Express 2025</button>
        </div>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 6 }}>
          Para usar SQL Express local, define USE_SQLITE=0 y DB_LOCAL_SERVER en el entorno.
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Teclado en pantalla</h3>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
          <span style={{ fontSize: 13, color: 'var(--text-2)' }}>Modo:</span>
          <select className="input" style={{ maxWidth: 200 }} value={kbMode} onChange={e => {
            const v = e.target.value;
            setKbMode(v);
            localStorage.setItem('keyboard_mode', v);
          }}>
            <option value="integrated">Integrado (app)</option>
            <option value="windows">Teclado de Windows</option>
          </select>
        </div>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 6 }}>
          {kbMode === 'integrated' ? 'Se usa el teclado virtual de la app.' : 'Se usa el teclado tactil de Windows (TabTip).'}
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Paro de emergencia</h3>
        <div className="row" style={{ alignItems: 'flex-end', gap: 10 }}>
          <div className="field" style={{ flex: '0 1 150px' }}>
            <label>Fuente</label>
            <select className="input" value={estopSource} onChange={e => setEstopSource(e.target.value)}>
              <option value="I">Entrada I</option>
              <option value="DB">DB del PLC</option>
            </select>
          </div>
          {estopSource === 'I' ? (
            <div className="field" style={{ flex: '0 1 160px' }}>
              <label>Entrada PLC</label>
              <input className="input" value={estopInput} onChange={e => setEstopInput(e.target.value.toUpperCase())} placeholder="I0.0" />
            </div>
          ) : (
            <>
              <div className="field" style={{ flex: '0 1 80px' }}>
                <label>DB</label>
                <input className="input" type="number" value={estopDb} onChange={e => setEstopDb(e.target.value)} placeholder="100" />
              </div>
              <div className="field" style={{ flex: '0 1 110px' }}>
                <label>Byte offset</label>
                <input className="input" type="number" value={estopOffset} onChange={e => setEstopOffset(e.target.value)} placeholder="0" />
              </div>
              <div className="field" style={{ flex: '0 1 70px' }}>
                <label>Bit</label>
                <input className="input" type="number" value={estopBit} onChange={e => setEstopBit(e.target.value)} placeholder="0" />
              </div>
            </>
          )}
          <div className="field" style={{ flex: '0 1 240px' }}>
            <label>Tipo de contacto</label>
            <select className="input" value={estopNc} onChange={e => setEstopNc(e.target.value)}>
              <option value="1">NC - normalmente cerrado</option>
              <option value="0">NO - normalmente abierto</option>
            </select>
          </div>
          <button className="btn primary" onClick={async () => {
            try {
              await invoke('set_config', { values: {
                estop_input: estopInput,
                estop_nc: estopNc,
                estop_source: estopSource,
                estop_db: estopDb,
                estop_offset: estopOffset,
                estop_bit: estopBit,
              }});
              notify('Paro de emergencia guardado', 'ok');
            } catch (e: any) { notify('Error: ' + (e?.message || e), 'error'); }
          }}>Guardar</button>
        </div>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 6 }}>
          Usa <b>DB100.IB0</b>, bit <b>0</b>: el PLC puente copia ahí la entrada del paro.
          La copia debe ser directa: liberado = 1, presionado = 0; en la app selecciona <b>NC</b>.
          Si el PLC usa <b>NOT "E-STOP"</b>, selecciona <b>NO</b>.
          El PLC puente copia el estado del paro en DB100.IB0, bit 0.
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Accesibilidad</h3>
        <label style={{ display: 'flex', gap: 8, alignItems: 'center', fontSize: 13, fontWeight: 600 }}>
          <input
            type="checkbox"
            style={{ width: 18, height: 18 }}
            checked={contrast}
            onChange={e => {
              const on = e.target.checked;
              setContrast(on);
              localStorage.setItem('contrast_mode', on ? '1' : '0');
              document.body.classList.toggle('high-contrast', on);
            }}
          />
          Alto contraste (texto y bordes reforzados)
        </label>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 6 }}>
          Útil en ambientes con mucha luz o pantallas de baja calidad.
        </div>
      </div>
      <div className="card" style={{ minWidth: 0 }}>
        <h3>Import / Export</h3>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', minWidth: 0 }}>
          <button className="btn primary" style={{ flex: '1 1 auto' }} onClick={doExport}>Exportar JSON</button>
          <label className="btn" style={{ cursor: 'pointer', flex: '1 1 auto' }}>Importar<input type="file" accept=".json" hidden onChange={e => e.target.files?.[0] && doImport(e.target.files[0])} /></label>
          <select className="input" style={{ maxWidth: 140, flex: '1 1 130px' }} value={mode} onChange={e => setMode(e.target.value)}><option value="overwrite">Sobreescribir</option><option value="add">Agregar</option><option value="merge">Merge</option></select>
        </div>
        <div style={{ marginTop: 10, fontSize: 12, color: 'var(--muted)' }}>{impLog.map((r: any) => <div key={r.id}>#{r.id} {r.op} ({r.mode}) — {r.status}</div>)}</div>
      </div>
    </div>
  );
}


function ManualTab() {
  const [tags, setTags] = useState<any[]>([]);
  const [msg, setMsg] = useState('');
  const [hipot, setHipot] = useState<{ raw: string; parsed: any } | null>(null);
  const [voltage, setVoltage] = useState(120);
  const [writing, setWriting] = useState<number | null>(null);

  const load = () => invoke<any>('plc_tags_list').then(setTags).catch(() => setTags([]));
  useEffect(() => { load(); }, []);

  const writeTag = async (tag: any, value: any) => {
    setWriting(tag.id);
    try {
      const r = await invoke<string>('plc_tag_write_by_id', { tagId: tag.id, value });
      if (r !== 'OK') throw new Error(r);
      setMsg(`Escrito ${tag.name} = ${value}`);
      load();
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
    finally { setWriting(null); }
  };

  const hipotCmd = async (action: 'START' | 'STOP' | 'STATUS') => {
    try {
      const r = await invoke<any>('hipot_test', { action });
      if (r?.error) throw new Error(r.error);
      setHipot({ raw: r.response ?? '', parsed: r.parsed ?? {} });
    } catch (e: any) { setMsg(String(e?.message ?? e)); }
  };

  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>Manual avanzado (sin registro)</h2>
      {msg && <div style={{ marginBottom: 8, fontSize: 12, color: 'var(--text-2)' }}>{msg}</div>}

      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Salidas del PLC (tags)</h3>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 8 }}>
          Escribe valores directamente en los tags del PLC. No se guarda en BD.
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6, maxHeight: 240, overflow: 'auto' }}>
          {tags.map((t: any) => (
            <div key={t.id} style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', minWidth: 0 }}>
              <span style={{ flex: '1 1 120px', fontSize: 13, fontWeight: 600 }}>{t.name}</span>
              <span style={{ fontSize: 12, color: 'var(--muted)', flex: '0 1 80px' }}>{t.data_type}</span>
              <input className="input" style={{ maxWidth: 100, flex: '0 1 100px' }}
                defaultValue={t.value ?? ''} id={`tag-${t.id}`}
                placeholder="valor" />
              <button className="btn primary" style={{ fontSize: 12 }}
                disabled={writing === t.id}
                onClick={() => {
                  const inp = document.getElementById(`tag-${t.id}`) as HTMLInputElement;
                  writeTag(t, inp?.value ?? '');
                }}>
                {writing === t.id ? '...' : 'Escribir'}
              </button>
            </div>
          ))}
          {!tags.length && <div style={{ fontSize: 12, color: 'var(--muted)' }}>No hay tags configurados.</div>}
        </div>
      </div>

      <div className="card" style={{ minWidth: 0 }}>
        <h3>Hipot manual (sin registro)</h3>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', marginBottom: 8 }}>
          <span style={{ fontSize: 13, color: 'var(--text-2)' }}>Voltaje:</span>
          <select className="input" style={{ maxWidth: 120 }} value={voltage} onChange={e => setVoltage(+e.target.value)}>
            {[120, 208, 220, 240].map(v => <option key={v} value={v}>{v} V</option>)}
          </select>
        </div>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' }}>
          <button className="btn ok" onClick={() => hipotCmd('START')}>Iniciar Hipot</button>
          <button className="btn danger" onClick={() => hipotCmd('STOP')}>Detener</button>
          <button className="btn" onClick={() => hipotCmd('STATUS')}>Consultar estado</button>
          {hipot && (
            <span style={{ fontSize: 12, color: 'var(--text-2)', marginLeft: 8 }}>
              {hipot.raw || '-'} {hipot.parsed?.voltage != null ? `→ ${hipot.parsed.voltage}V` : ''} {hipot.parsed?.pass ? 'PASS' : hipot.parsed?.pass === false ? 'FAIL' : ''}
            </span>
          )}
        </div>
        <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 6 }}>
          No guarda en BD. Solo visualización y control directo.
        </div>
      </div>
    </div>
  );
}


function PlcIoTab() {
  const [status, setStatus] = useState<any>(null);
  const [err, setErr] = useState('');

  const load = () => {
    invoke<any>('plc_io_status').then(setStatus).catch(e => setErr(String(e?.message ?? e)));
  };

  useEffect(() => {
    load();
    const id = setInterval(load, 2000);
    return () => clearInterval(id);
  }, []);

  const toggle = async (q: string, val: boolean) => {
    try {
      const r = await invoke<string>('plc_io_write', { q, value: val });
      if (r !== 'OK') throw new Error(r);
      load();
    } catch (e: any) { setErr(String(e?.message ?? e)); }
  };

  const Q_LABELS: Record<string, string> = {
    "Q0.0": "KM2 (120VAC)", "Q0.1": "KM1 (220VAC)", "Q0.2": "K2 (SSR 0-20A)",
    "Q0.3": "K8 (Relay 0-1A)", "Q0.4": "K9 (Relay 0-10A)", "Q0.5": "K3 (FAN)",
    "Q0.6": "K4 (LIGHT)", "Q0.7": "K5 (FRAME)", "Q1.0": "K6 (DOOR)",
    "Q1.1": "K7 (DRAIN)", "Q2.0": "K1 (DEFROST)", "Q2.1": "Libre", "Q2.2": "Libre",
  };
  const I_LABELS: Record<string, string> = {
    "I0.0": "Paro de Emergencia", "I0.1": "NA KM1 (220VAC)", "I0.2": "NA KM2 (110VAC)",
  };
  const IA_LABELS: Record<string, string> = {
    "IA1": "Amplificador 0-1A", "IA2": "Amplificador 0-10A",
    "IA3": "Amplificador 0-20A", "IA4": "Amplificador 0-220VAC",
  };

  return (
    <div style={{ minWidth: 0 }}>
      <h2 className="page-title" style={{ marginBottom: 14 }}>Monitor PLC (IO) - Lectura y escritura de Q</h2>
      {(err || status?.error) && <div className="toast-err">{err || status?.error}</div>}
      <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 10 }}>
        Actualizacion automatica cada 2 segundos. Los indicadores muestran el estado en tiempo real.
      </div>

      {status?.estop_address && (
        <div className="card" style={{ marginBottom: 10, background: status?.estop_raw ? '#e6f4ea' : '#fef2f2', borderColor: status?.estop_raw ? '#a8d0b4' : '#fecaca' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
            <b>Paro de emergencia</b>
            <span className="mono" style={{ fontSize: 12 }}>{status.estop_address}</span>
            <span className={`pill ${status.estop_raw ? 'on' : 'off'}`}>
              {status.estop_raw == null ? 'sin lectura' : status.estop_raw ? '1' : '0'}
            </span>
            {status.read_ok === false && <span style={{ fontSize: 12, color: 'var(--bad)' }}>No se pudo leer</span>}
          </div>
        </div>
      )}

      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Salidas digitales (Q)</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 6 }}>
          {Object.entries(status?.outputs ?? {}).map(([q, v]) => (
            <div key={q} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 8px', background: '#fff', borderRadius: 6, border: '1px solid var(--line)' }}>
              <span style={{ width: 10, height: 10, borderRadius: '50%', background: v ? 'var(--ok)' : '#94a3b8', flexShrink: 0 }} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <b style={{ fontSize: 12 }}>{q}</b>
                <div style={{ fontSize: 12, color: 'var(--muted)', overflow: 'hidden', textOverflow: 'ellipsis' }}>{Q_LABELS[q] || ''}</div>
              </div>
              <span className={`pill ${v ? 'on' : ''}`} style={{ fontSize: 12 }}>{v ? 'ON' : 'OFF'}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Entradas digitales (I)</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 6 }}>
          {Object.entries(status?.inputs ?? {}).map(([i, v]) => (
            <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 8px', background: '#fff', borderRadius: 6, border: '1px solid var(--line)' }}>
              <span style={{ width: 10, height: 10, borderRadius: '50%', background: v ? 'var(--accent)' : '#94a3b8', flexShrink: 0 }} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <b style={{ fontSize: 12 }}>{i}</b>
                <div style={{ fontSize: 12, color: 'var(--muted)', overflow: 'hidden', textOverflow: 'ellipsis' }}>{I_LABELS[i] || ''}</div>
              </div>
              <span className={`pill ${v ? 'accent' : ''}`} style={{ fontSize: 12 }}>{v ? 'ON' : 'OFF'}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="card" style={{ minWidth: 0 }}>
        <h3>Entradas analogicas (IA)</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 6 }}>
          {Object.entries(status?.analog ?? {}).map(([ia, v]) => (
            <div key={ia} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 8px', background: '#fff', borderRadius: 6, border: '1px solid var(--line)' }}>
              <div style={{ flex: 1, minWidth: 0 }}>
                <b style={{ fontSize: 12 }}>{ia}</b>
                <div style={{ fontSize: 12, color: 'var(--muted)', overflow: 'hidden', textOverflow: 'ellipsis' }}>{IA_LABELS[ia] || ''}</div>
              </div>
              <span className="pill" style={{ fontSize: 12, fontVariantNumeric: 'tabular-nums' }}>{Number(v).toFixed(2)}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
