import React, { useEffect, useState } from 'react';
import { api } from '../api';
import { LabelEditor, LObj, objsToTemplate, templateToObjs } from '../components/LabelEditor';

const TABS = ['Usuarios', 'Recetas', 'PLC', 'Hipot', 'BD', 'Etiquetas', 'Logs', 'Sistema'] as const;

export function AdminPanel({ onBack, adminBatch }: { onBack: () => void; adminBatch?: string }) {
  const [tab, setTab] = useState<(typeof TABS)[number]>('Recetas');
  return (
    <div style={{ display: 'flex', minHeight: 560, minWidth: 0 }}>
      <div style={{ width: 176, background: '#141b26', padding: 8, flexShrink: 0, borderRight: '1px solid #2a3a52', display: 'flex', flexDirection: 'column', gap: 4 }}>
        <button className="btn" style={{ width: '100%', background: '#1e2a3a', color: '#fff', borderColor: '#334155', fontSize: 13 }} onClick={onBack}>← Volver</button>
        <div style={{ height: 1, background: '#2a3a52', margin: '4px 0' }} />
        {TABS.map(t => (
          <button key={t} onClick={() => setTab(t)}
            className="btn" style={{ width: '100%', justifyContent: 'flex-start', fontSize: 13,
              background: tab === t ? '#b7790f' : '#1e2a3a', color: '#fff', borderColor: tab === t ? '#b7790f' : '#334155' }}>{t}</button>
        ))}
      </div>
      <div style={{ flex: 1, padding: 12, overflow: 'auto', minWidth: 0, background: '#eef2f6' }}>
        {tab === 'Usuarios' && <UsersTab />}
        {tab === 'Recetas' && <RecipesTab adminBatch={adminBatch} />}
        {tab === 'PLC' && <PlcTab />}
        {tab === 'Hipot' && <HipotTab />}
        {tab === 'BD' && <DbTab />}
        {tab === 'Etiquetas' && <LabelsTab />}
        {tab === 'Logs' && <LogsTab />}
        {tab === 'Sistema' && <SystemTab />}
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
  const load=()=>api.get('/users').then(setUsers).catch(()=>setUsers([]));
  useEffect(()=>{load();},[]);
  const rows=users.filter(u=>!q||u.username.includes(q.toUpperCase()));
  const doDelete=async(u:any)=>{
    if(!confirm(`Borrar usuario ${u.username}?`)) return;
    try{ await api.del(`/users/${u.id}`); setMsg(`Borrado ${u.username}`); load(); }catch(e:any){ setMsg(String(e?.message??e)); }
  };
  const doSave=async()=>{
    if(!editing) return;
    if(!editing.username.trim()){ setMsg('Batch es obligatorio'); return; }
    const payload:any={username:editing.username.trim().toUpperCase(),role:editing.role,is_active:editing.is_active};
    try{
      if(editing.id){ await api.put(`/users/${editing.id}`,payload); setMsg('Usuario actualizado'); }
      else {
        await api.post('/auth/register',{username:payload.username,password:'',role:payload.role});
        if(!payload.is_active) await api.put(`/users/${(await api.get('/users')).find((x:any)=>x.username===payload.username)?.id}`,{is_active:false});
        setMsg('Usuario creado — gafete listo');
      }
      setEditing(null); load();
    }catch(e:any){ setMsg(String(e?.message??e)); }
  };
  return (
    <div style={{ minWidth:0 }}>
      <h2 style={{ margin:'0 0 10px 0', fontSize:16 }}>Usuarios — solo batch + permisos</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Usuario')||msg.startsWith('Borrado')?'#e6f4ea':undefined, borderColor: msg.startsWith('Usuario')?'#a8d0b4':undefined, color: msg.startsWith('Usuario')?'#166534':undefined }}>{msg}</div>}
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
                  <button className="btn sm" onClick={async()=>{ const d=await api.get(`/users/${u.id}`); setViewPerm(d); }}>Ver permisos</button>
                  <button className="btn sm primary" onClick={()=>setEditing({id:u.id,username:u.username,role:u.role,is_active:u.is_active})}>Editar</button>
                  <button className="btn sm danger" onClick={()=>doDelete(u)}>Borrar</button>
                </div>
              </td></tr>
          ))}{!rows.length && <tr><td colSpan={4} style={{ textAlign:'center', color:'#64748b' }}>Sin usuarios</td></tr>}</tbody>
        </table>
      </div>

      {viewPerm && (
        <div style={{ position:'fixed', inset:0, background:'rgba(20,27,38,.55)', display:'flex', alignItems:'center', justifyContent:'center', zIndex:4000, padding:12 }} onClick={()=>setViewPerm(null)}>
          <div className="card" style={{ width:'min(420px,96vw)', minWidth:0 }} onClick={e=>e.stopPropagation()}>
            <h3 style={{ margin:'0 0 8px 0' }}>{viewPerm.username} — {viewPerm.role==='admin'?'Admin':'Operador'}</h3>
            <div style={{ fontSize:12, color: viewPerm.is_active?'#166534':'#991b1b', fontWeight:700, marginBottom:8 }}>{viewPerm.is_active?'Activo':'Inactivo'}</div>
            <div style={{ fontSize:13, fontWeight:700, marginBottom:4 }}>Permisos:</div>
            <ul style={{ margin:0, paddingLeft:18, fontSize:13, color:'#334155' }}>{(viewPerm.permissions??[]).map((p:string,i:number)=><li key={i}>{p}</li>)}</ul>
            <div style={{ display:'flex', justifyContent:'flex-end', marginTop:12 }}><button className="btn" onClick={()=>setViewPerm(null)}>Cerrar</button></div>
          </div>
        </div>
      )}

      {editing && (
        <div style={{ position:'fixed', inset:0, background:'rgba(20,27,38,.55)', display:'flex', alignItems:'center', justifyContent:'center', zIndex:4000, padding:12 }} onClick={()=>setEditing(null)}>
          <div className="card" style={{ width:'min(420px,96vw)', minWidth:0 }} onClick={e=>e.stopPropagation()}>
            <h3 style={{ margin:'0 0 10px 0' }}>{editing.id?'Editar usuario':'Alta de usuario'}</h3>
            <div style={{ display:'flex', flexDirection:'column', gap:8 }}>
              <label style={{ fontSize:12, fontWeight:700, color:'#475569' }}>Batch / Usuario — gafete (mayúsculas)</label>
              <input className="input" placeholder="Ej. BATCH-001" value={editing.username} onChange={e=>setEditing({...editing,username:e.target.value.toUpperCase()})} />
              <label style={{ fontSize:12, fontWeight:700, color:'#475569' }}>Permiso</label>
              <select className="input" value={editing.role} onChange={e=>setEditing({...editing,role:e.target.value as any})}>
                <option value="operator">Operador — gafete</option>
                <option value="admin">Admin — acceso a configuración</option>
              </select>
              <div style={{ fontSize:12, color:'#64748b', background:'#f1f5f9', padding:'6px 8px', borderRadius:6 }}>Todos entran con gafete (sin contraseña). El permiso define qué puede hacer.</div>
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
const COMPS = ['FAN','DOOR','FRAME','DRAIN','DEFROST','LENGTH'];

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
  const [editing,setEditing]=useState<null|{id:number|null,name:string,model:string,product:string,m:any,t:any}>(null);
  const load=()=>api.get('/recipes').then(setList).catch(()=>setList([]));
  useEffect(()=>{load();},[]);
  const rows=list.filter(r=>!q||r.model.includes(q));
  const openNew=()=>{
    const m:any={}; const t:any={};
    for(const v of VOLTS){ const vs=String(v); m[vs]={}; t[vs]={min:'',max:''}; for(const c of COMPS) m[vs][c]={min:'',max:'',enabled:false}; }
    setEditing({id:null,name:'',model:q||'',product:'',m,t});
  };
  const openEdit=async(r:any)=>{
    const d=await api.recipeByModel(r.model);
    const {m,t}=matrixFromDetail(d);
    setEditing({id:d.id,name:d.name,model:d.model,product:d.product||'',m,t});
  };
  const doDelete=async(r:any)=>{
    if(!confirm(`Borrar receta ${r.model} — ${r.name}?`)) return;
    await api.del(`/recipes/${r.id}`); setMsg(`Borrada ${r.model}`); load();
  };
  const doDup=async(r:any)=>{
    const model=prompt(`Modelo para la copia de ${r.model}:`, `${r.model}-COPIA`);
    if(!model) return;
    await api.post(`/recipes/${r.id}/duplicate`,{model}); setMsg(`Duplicada como ${model.toUpperCase()}`); load();
  };
  const doSave=async()=>{
    if(!editing) return;
    if(!editing.model.trim()||!editing.name.trim()){ setMsg('Modelo y nombre son obligatorios'); return; }
    const components:any[]=[]; const totals:any[]=[];
    for(const v of VOLTS){
      const vs=String(v);
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
    }
    const body:any={name:editing.name.trim(),model:editing.model.trim().toUpperCase(),product:editing.product||'',voltage_levels:JSON.stringify(VOLTS.map(v=>String(v))),components,totals,modified_by:(adminBatch||'').toUpperCase()||'ADMIN'};
    try{
      if(editing.id) await api.put(`/recipes/${editing.id}`,body);
      else await api.post('/recipes',body);
      setEditing(null); setMsg(editing.id?'Receta actualizada':'Receta creada'); load();
    }catch(err:any){ setMsg(String(err?.message ?? err)); }
  };
  return (
    <div style={{ minWidth:0 }}>
      <h2 style={{ margin:'0 0 10px 0', fontSize:16 }}>Recetas — altas / bajas / cambios</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Receta')||msg.startsWith('Duplicada')||msg.startsWith('Borrada')?'#e6f4ea':undefined, borderColor: msg.startsWith('Receta')?'#a8d0b4':undefined, color: msg.startsWith('Receta')?'#166534':undefined }}>{msg}</div>}
      <div className="card" style={{ marginBottom:10 }}>
        <div style={{ display:'flex', gap:8, minWidth:0, flexWrap:'wrap' }}>
          <input className="input" placeholder="Buscar modelo" value={q} onChange={e=>setQ(e.target.value.toUpperCase())} style={{ flex:'1 1 200px', minWidth:0 }} />
          <button className="btn primary" style={{ flexShrink:0 }} onClick={openNew}>+ Alta</button>
          <button className="btn" style={{ flexShrink:0 }} onClick={load}>Actualizar</button>
        </div>
      </div>
      <div className="card" style={{ overflow:'auto', minWidth:0 }}>
        <table className="grid"><thead><tr><th>Modelo</th><th>Nombre</th><th>Guardado</th><th style={{ minWidth:220 }}>Acciones</th></tr></thead>
          <tbody>{rows.map(r=>(
            <tr key={r.id}><td><b>{r.model}</b></td><td>{r.name}</td><td style={{ fontSize:11, color:'#64748b' }}>{r.modified_by?`${r.modified_by} · `:''}{r.updated_at?new Date(r.updated_at).toLocaleString('es-MX'):(r.created_at?new Date(r.created_at).toLocaleString('es-MX'):'—')}</td>
              <td>
                <div style={{ display:'flex', gap:6, flexWrap:'wrap' }}>
                  <button className="btn sm primary" onClick={()=>openEdit(r)}>Editar</button>
                  <button className="btn sm" onClick={()=>doDup(r)}>Duplicar</button>
                  <button className="btn sm danger" onClick={()=>doDelete(r)}>Borrar</button>
                </div>
              </td></tr>
          ))}{!rows.length && <tr><td colSpan={4} style={{ textAlign:'center', color:'#64748b' }}>Sin recetas — use + Alta</td></tr>}</tbody>
        </table>
      </div>

      {editing && (
        <div style={{ position:'fixed', inset:0, background:'rgba(20,27,38,.55)', display:'flex', alignItems:'center', justifyContent:'center', zIndex:4000, padding:12 }} onClick={()=>setEditing(null)}>
          <div className="card" style={{ width:'min(1180px,96vw)', maxHeight:'92vh', overflow:'auto', minWidth:0 }} onClick={e=>e.stopPropagation()}>
            <h3 style={{ margin:'0 0 10px 0' }}>{editing.id?'Editar receta':'Alta de receta'} — voltaje entero + check para descartar</h3>
            <div style={{ display:'flex', gap:8, minWidth:0, flexWrap:'wrap', marginBottom:10 }}>
              <input className="input" placeholder="Modelo (ej. RL)" value={editing.model} onChange={e=>setEditing({...editing,model:e.target.value.toUpperCase()})} style={{ flex:'1 1 140px' }} />
              <input className="input" placeholder="Nombre (ej. CONSUMOS RL)" value={editing.name} onChange={e=>setEditing({...editing,name:e.target.value})} style={{ flex:'2 1 220px' }} />
              <input className="input" placeholder="Producto (opcional)" value={editing.product} onChange={e=>setEditing({...editing,product:e.target.value})} style={{ flex:'1 1 180px' }} />
            </div>

            <div style={{ overflow:'auto', border:'1px solid var(--line)', borderRadius:8 }}>
              <table className="grid" style={{ minWidth:980, fontSize:12 }}>
                <thead>
                  <tr><th style={{ minWidth:60, background:'#1e293b', color:'#fff' }}>V</th>{COMPS.map(c=><th key={c} colSpan={3} style={{ textAlign:'center', background:'#1e293b', color:'#fff', minWidth:138 }}>{c}</th>)}<th colSpan={2} style={{ textAlign:'center', background:'#b7790f', color:'#fff', minWidth:138 }}>TOTAL</th></tr>
                  <tr><th style={{ background:'#f1f5f9' }}></th>{COMPS.map(c=><React.Fragment key={c}><th style={{ background:'#f1f5f9', fontSize:11 }}>MIN</th><th style={{ background:'#f1f5f9', fontSize:11 }}>MAX</th><th style={{ background:'#fdf1d0', fontSize:11, minWidth:36 }}>✓</th></React.Fragment>)}<th style={{ background:'#fdf1d0', fontSize:11 }}>MIN</th><th style={{ background:'#fdf1d0', fontSize:11 }}>MAX</th></tr>
                </thead>
                <tbody>
                  {VOLTS.map(v=>{
                    const vs=String(v);
                    return (
                    <tr key={vs}>
                      <td style={{ fontWeight:800, background:'#f8fafc', textAlign:'center' }}>{v}</td>
                      {COMPS.map(c=>{
                        const cell=editing.m[vs][c];
                        return <React.Fragment key={c}>
                          <td style={{ padding:3 }}><input disabled={!cell.enabled} value={cell.min} onChange={e=>setEditing({ ...editing, m:{...editing.m,[vs]:{...editing.m[vs],[c]:{...cell,min:e.target.value}}} })} placeholder="—" style={{ width:56, height:30, border:`1px solid ${cell.enabled?'#b7790f':'#d8e0ea'}`, borderRadius:6, textAlign:'center', fontSize:12, background: cell.enabled?'#fff':'#f1f5f9' }} /></td>
                          <td style={{ padding:3 }}><input disabled={!cell.enabled} value={cell.max} onChange={e=>setEditing({ ...editing, m:{...editing.m,[vs]:{...editing.m[vs],[c]:{...cell,max:e.target.value}}} })} placeholder="—" style={{ width:56, height:30, border:`1px solid ${cell.enabled?'#b7790f':'#d8e0ea'}`, borderRadius:6, textAlign:'center', fontSize:12, background: cell.enabled?'#fff':'#f1f5f9' }} /></td>
                          <td style={{ padding:3, textAlign:'center' }}><input type="checkbox" checked={cell.enabled} onChange={e=>setEditing({...editing,m:{...editing.m,[vs]:{...editing.m[vs],[c]:{...cell,enabled:e.target.checked}}}})} style={{ width:18, height:18 }} title="Activar/desactivar este componente en este voltaje" /></td>
                        </React.Fragment>;
                      })}
                      <td style={{ padding:3 }}><input value={editing.t[vs].min} onChange={e=>setEditing({...editing,t:{...editing.t,[vs]:{...editing.t[vs],min:e.target.value}}})} placeholder="—" style={{ width:56, height:30, border:'1px solid #d8e0ea', borderRadius:6, textAlign:'center', fontSize:12 }} /></td>
                      <td style={{ padding:3 }}><input value={editing.t[vs].max} onChange={e=>setEditing({...editing,t:{...editing.t,[vs]:{...editing.t[vs],max:e.target.value}}})} placeholder="—" style={{ width:56, height:30, border:'1px solid #d8e0ea', borderRadius:6, textAlign:'center', fontSize:12 }} /></td>
                    </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
            <div style={{ fontSize:11, color:'#64748b', marginTop:6 }}>Marque ✓ para incluir ese componente en ese voltaje. Desmarcado = esa parte no se prueba.</div>
            <div style={{ fontSize:11, color:'#64748b' }}>Se guarda con fecha/hora y usuario: <b>{(adminBatch||'ADMIN').toUpperCase()}</b></div>
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
  const [tags, setTags] = useState<any[]>([]);
  useEffect(() => { api.get('/admin/plc-tags').then(setTags).catch(() => setTags([])); }, []);
  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>PLC S7-1200 — DB1.DBW110 modo 1/2</h2>
      <div className="card" style={{ marginBottom: 10 }}>
        <button className="btn" onClick={() => api.get('/status/plc').then(setSt)}>Probar conexión</button>
        {st && <pre style={{ marginTop: 8, fontSize: 12, background: '#f1f5f9', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(st, null, 2)}</pre>}
      </div>
      <div className="card" style={{ overflow: 'auto' }}>
        <h3>Tags</h3>
        <table className="grid"><thead><tr><th>Nombre</th><th>DB</th><th>Offset</th><th>Tipo</th></tr></thead>
          <tbody>{tags.map((t: any) => <tr key={t.id}><td>{t.name}</td><td>{t.db}</td><td>{t.offset}</td><td>{t.type}</td></tr>)}</tbody>
        </table>
        <TagForm onDone={() => api.get('/admin/plc-tags').then(setTags).catch(() => {})} />
      </div>
    </div>
  );
}
function TagForm({ onDone }: { onDone: () => void }) {
  const [f, setF] = useState({ name: '', offset: '0' });
  return (
    <div style={{ display: 'flex', gap: 8, marginTop: 8, minWidth: 0 }}>
      <input className="input" placeholder="Nombre" value={f.name} onChange={e => setF({ ...f, name: e.target.value })} style={{ flex: 1, minWidth: 0 }} />
      <input className="input" placeholder="Offset" value={f.offset} onChange={e => setF({ ...f, offset: e.target.value })} style={{ maxWidth: 110, flexShrink: 0 }} />
      <button className="btn primary" style={{ flexShrink: 0 }} onClick={() => api.post('/admin/plc-tags', { name: f.name.toUpperCase(), db: 1, offset: +f.offset, type: 'REAL' }).then(onDone)}>Agregar</button>
    </div>
  );
}

function HipotTab() {
  const [st, setSt] = useState<any>(null);
  const [cmd, setCmd] = useState('STATUS?');
  const [resp, setResp] = useState<any>(null);
  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>Hipot 3865</h2>
      <div className="card" style={{ marginBottom: 10 }}>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <button className="btn" onClick={() => api.get('/status/hipot').then(setSt)}>Probar Hipot</button>
          <button className="btn" onClick={() => api.post('/admin/print-test').then(r => alert(JSON.stringify(r)))}>Test Zebra</button>
        </div>
        {st && <pre style={{ marginTop: 8, fontSize: 12, background: '#f1f5f9', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(st, null, 2)}</pre>}
      </div>
      <div className="card">
        <h3>Comando manual</h3>
        <div style={{ display: 'flex', gap: 8, minWidth: 0 }}>
          <input className="input" value={cmd} onChange={e => setCmd(e.target.value.toUpperCase())} style={{ flex: 1, minWidth: 0 }} />
          <button className="btn primary" style={{ flexShrink: 0 }} onClick={() => api.post('/admin/hipot-cmd', { command: cmd }).then(setResp)}>Enviar</button>
        </div>
        {resp && <pre style={{ marginTop: 8, fontSize: 12, background: '#f1f5f9', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(resp, null, 2)}</pre>}
      </div>
    </div>
  );
}

function DbTab() {
  const [cfg, setCfg] = useState<any>(null);
  const [log, setLog] = useState('');
  const [syncs, setSyncs] = useState<any[]>([]);
  const load = () => { api.get('/admin/config').then(setCfg).catch(() => setCfg(null)); api.get('/admin/sync-log').then(setSyncs).catch(() => setSyncs([])); };
  useEffect(() => { load(); }, []);
  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>Base de datos</h2>
      <div className="card" style={{ marginBottom: 10 }}>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <button className="btn primary" onClick={() => api.post('/admin/sync-now').then(r => { setLog(JSON.stringify(r)); load(); })}>Sync ahora</button>
          <button className="btn" onClick={() => api.get('/health').then(r => setLog(JSON.stringify(r, null, 2)))}>Estado</button>
        </div>
        {log && <pre style={{ marginTop: 8, fontSize: 12, background: '#f1f5f9', padding: 8, borderRadius: 6, overflow: 'auto' }}>{log}</pre>}
        <details style={{ marginTop: 8 }}><summary style={{ cursor: 'pointer', fontSize: 13 }}>Config actual</summary><pre style={{ fontSize: 12, background: '#f1f5f9', padding: 8, borderRadius: 6, overflow: 'auto' }}>{JSON.stringify(cfg, null, 2)}</pre></details>
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
  const load = () => api.get('/admin/templates').then(setList).catch(() => setList([]));
  useEffect(() => { load(); }, []);
  const openNew = () => { setSelId(null); setObjs(null); setName('Nueva plantilla'); setIsDef(false); setKey(k => k + 1); };
  const openTpl = async (id: number) => { const t = await api.get(`/admin/templates/${id}`); setSelId(id); setName(t.name); setIsDef(!!t.is_default); setObjs(templateToObjs(t.canvas_data || '{"objects":[]}')); setKey(k => k + 1); };
  const save = async (o: LObj[]) => {
    const body = { name, width_mm: 100, height_mm: 60, canvas_data: objsToTemplate(o), is_default: isDef };
    try { if (selId) await api.put(`/admin/templates/${selId}`, body); else { const r = await api.post('/admin/templates', body); setSelId(r.id); } setMsg('Plantilla guardada'); load(); } catch (e) { setMsg(String(e)); }
  };
  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>Etiquetas Zebra</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Plantilla') ? '#e6f4ea' : undefined, borderColor: msg.startsWith('Plantilla') ? '#a8d0b4' : undefined, color: msg.startsWith('Plantilla') ? '#166534' : undefined }}>{msg}</div>}
      <div className="card" style={{ marginBottom: 10 }}>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', minWidth: 0 }}>
          <select className="input" style={{ maxWidth: 260, flex: '1 1 200px' }} value={selId ?? ''} onChange={e => e.target.value ? openTpl(+e.target.value) : openNew()}>
            <option value="">— Nueva —</option>{list.map((t: any) => <option key={t.id} value={t.id}>{t.name}{t.is_default ? ' · default' : ''}</option>)}
          </select>
          <input className="input" style={{ maxWidth: 240, flex: '1 1 180px' }} value={name} onChange={e => setName(e.target.value)} />
          <label style={{ display: 'flex', gap: 6, alignItems: 'center', fontWeight: 600, fontSize: 13, flexShrink: 0 }}>
            <input type="checkbox" checked={isDef} onChange={e => setIsDef(e.target.checked)} style={{ width: 18, height: 18 }} /> Default
          </label>
          {selId && <button className="btn danger sm" style={{ flexShrink: 0 }} onClick={() => api.del(`/admin/templates/${selId}`).then(() => { openNew(); load(); })}>Eliminar</button>}
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
  const load = () => api.get(`/admin/logs?category=${cat}&level=${level}&q=${encodeURIComponent(q)}`).then(setLogs).catch(() => setLogs([]));
  useEffect(() => { load(); }, []);
  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>Logs</h2>
      <div className="card" style={{ marginBottom: 10 }}>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', minWidth: 0 }}>
          <select className="input" style={{ maxWidth: 150, flex: '1 1 130px' }} value={cat} onChange={e => setCat(e.target.value)}><option value="">Todas CAT</option>{['UI','PLC','HIPOT','DB','SYNC','BARCODE','SYSTEM'].map(c => <option key={c} value={c}>{c}</option>)}</select>
          <select className="input" style={{ maxWidth: 130, flex: '1 1 110px' }} value={level} onChange={e => setLevel(e.target.value)}><option value="">Todos</option>{['DEBUG','INFO','WARN','ERROR'].map(c => <option key={c} value={c}>{c}</option>)}</select>
          <input className="input" placeholder="Buscar…" value={q} onChange={e => setQ(e.target.value)} style={{ flex: '1 1 160px', minWidth: 0 }} />
          <button className="btn primary" style={{ flexShrink: 0 }} onClick={load}>Buscar</button>
        </div>
      </div>
      <div className="card" style={{ maxHeight: 380, overflow: 'auto', fontSize: 13, minWidth: 0 }}>
        {logs.map(l => <div key={l.id} style={{ borderBottom: '1px solid #e8edf3', padding: '6px 0', minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis' }}>
          <b>#{l.id}</b> [{l.ts}] <span className={`pill ${l.level === 'ERROR' ? 'off' : ''}`} style={{ fontSize: 11 }}>{l.level}</span> [{l.cat}] <b>{l.action}</b>: {l.detail}
        </div>)}
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
  const loadLog = () => api.get('/admin/import-export-log').then(setImpLog).catch(() => setImpLog([]));
  useEffect(() => { loadLog(); api.get('/admin/config').then(c => setHold(String(c?.result_hold_seconds ?? '10'))).catch(() => {}); }, []);
  const doExport = async () => { const r = await api.get('/admin/export'); const blob = new Blob([r.data], { type: 'application/json' }); const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = `config_export_${new Date().toISOString().slice(0,10)}.json`; a.click(); };
  const doImport = async (f: File) => { try { const t = await f.text(); const r = await api.post('/admin/import', { data: t, mode }); setMsg(`Import OK: ${r.imported} · Conflictos: ${(r.conflicts ?? []).length}`); loadLog(); } catch (e) { setMsg(String(e)); } };
  return (
    <div style={{ minWidth: 0 }}>
      <h2 style={{ margin: '0 0 10px 0', fontSize: 16 }}>Sistema</h2>
      {msg && <div className="toast-err" style={{ background: msg.startsWith('Import OK') ? '#e6f4ea' : undefined }}>{msg}</div>}
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Espera en resultado</h3>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap', minWidth: 0 }}>
          <span style={{ fontSize: 13, color: '#475569' }}>Segundos antes de volver al inicio:</span>
          <input className="input" type="number" min={3} max={60} value={hold} onChange={e => setHold(e.target.value)} style={{ maxWidth: 90 }} />
          <button className="btn primary" onClick={async () => {
            const n = Math.max(3, Math.min(60, parseInt(hold || '10', 10) || 10));
            await api.post('/admin/config', { result_hold_seconds: String(n) });
            setHold(String(n)); setHoldMsg(`Guardado: ${n}s`); setTimeout(() => setHoldMsg(''), 2000);
          }}>Guardar</button>
          {holdMsg && <span className="pill on" style={{ fontSize: 12 }}>{holdMsg}</span>}
        </div>
      </div>
      <div className="card" style={{ minWidth: 0 }}>
        <h3>Import / Export</h3>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', minWidth: 0 }}>
          <button className="btn primary" style={{ flex: '1 1 auto' }} onClick={doExport}>Exportar JSON</button>
          <label className="btn" style={{ cursor: 'pointer', flex: '1 1 auto' }}>Importar<input type="file" accept=".json" hidden onChange={e => e.target.files?.[0] && doImport(e.target.files[0])} /></label>
          <select className="input" style={{ maxWidth: 140, flex: '1 1 130px' }} value={mode} onChange={e => setMode(e.target.value)}><option value="overwrite">Sobreescribir</option><option value="add">Agregar</option><option value="merge">Merge</option></select>
        </div>
        <div style={{ marginTop: 10, fontSize: 12, color: '#64748b' }}>{impLog.map((r: any) => <div key={r.id}>#{r.id} {r.op} ({r.mode}) — {r.status}</div>)}</div>
      </div>
    </div>
  );
}
function ManualPlc() {
  const [db, setDb] = useState('1');
  const [off, setOff] = useState('0');
  const [val, setVal] = useState('0');
  const [out, setOut] = useState('');
  return (
    <div style={{ minWidth: 0 }}>
      <div style={{ display: 'flex', gap: 8, marginBottom: 8, minWidth: 0, flexWrap: 'wrap' }}>
        <input className="input" placeholder="DB" value={db} onChange={e => setDb(e.target.value)} style={{ flex: '1 1 60px' }} />
        <input className="input" placeholder="Offset" value={off} onChange={e => setOff(e.target.value)} style={{ flex: '1 1 80px' }} />
        <input className="input" placeholder="Valor" value={val} onChange={e => setVal(e.target.value)} style={{ flex: '1 1 80px' }} />
      </div>
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        <button className="btn" onClick={() => api.post('/admin/plc-read', { db: +db, offset: +off, type: 'REAL' }).then(r => setOut(`→ ${r.value}`))}>Leer</button>
        <button className="btn primary" onClick={() => api.post('/admin/plc-write', { db: +db, offset: +off, type: 'REAL', value: +val }).then(r => setOut(JSON.stringify(r)))}>Escribir</button>
      </div>
      {out && <pre style={{ marginTop: 8, fontSize: 12, background: '#f1f5f9', padding: 8, borderRadius: 6, overflow: 'auto', wordBreak: 'break-all' }}>{out}</pre>}
    </div>
  );
}
