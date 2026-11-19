import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';

export function DbSetupDialog({ open, onClose, onSaved }: {
  open: boolean;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [server, setServer] = useState('.\\SQLEXPRESS');
  const [name, setName] = useState('HiPotDB');
  const [auth, setAuth] = useState<'windows' | 'sql'>('windows');
  const [user, setUser] = useState('sa');
  const [password, setPassword] = useState('');
  const [createDb, setCreateDb] = useState(true);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState('');
  const [ok, setOk] = useState(false);

  useEffect(() => {
    if (!open) return;
    setMsg('');
    setOk(false);
    invoke<any>('local_db_config_get').then(c => {
      if (!c) return;
      setServer(c.server ?? '.\\SQLEXPRESS');
      setName(c.name ?? 'HiPotDB');
      const hasUser = !!(c.user && String(c.user).length);
      setAuth(hasUser ? 'sql' : 'windows');
      setUser(hasUser ? String(c.user) : 'sa');
      setPassword(c.password ?? '');
    }).catch(() => {});
  }, [open]);

  const config = () => ({
    use_sqlite: '0',
    server: server.trim(),
    name: name.trim(),
    user: auth === 'sql' ? user.trim() : '',
    password: auth === 'sql' ? password : '',
  });

  const test = async () => {
    setBusy(true); setMsg(''); setOk(false);
    try {
      const r = await invoke<string>('local_db_test_config', { config: config() });
      setMsg(r); setOk(true);
    } catch (e: any) {
      setMsg(String(e?.message ?? e));
    } finally { setBusy(false); }
  };

  const save = async () => {
    setBusy(true); setMsg(''); setOk(false);
    try {
      const r = await invoke<string>('local_db_setup', { config: config(), createDatabase: createDb });
      if (r !== 'saved') throw new Error(r);
      await invoke('db_retry').catch(() => {});
      setMsg('Configuración guardada. Reintentando conexión…');
      setOk(true);
      setTimeout(() => onSaved(), 500);
    } catch (e: any) {
      setMsg(String(e?.message ?? e));
    } finally { setBusy(false); }
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}
          style={{ position: 'fixed', inset: 0, background: 'rgba(15,23,42,.70)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 7000, padding: 16 }}>
          <motion.div initial={{ scale: 0.97, y: 10 }} animate={{ scale: 1, y: 0 }} exit={{ scale: 0.97, opacity: 0 }}
            className="card" style={{ width: 'min(560px, 96vw)', maxHeight: '92vh', overflow: 'auto' }}>
            <h3 style={{ margin: '0 0 6px 0' }}>Configurar SQL Express</h3>
            <p className="page-sub" style={{ marginTop: 0 }}>
              La app no pudo iniciar sesión en SQL Server. El error 18456 significa que SQL Server rechazó
              el usuario de Windows. Usa autenticación SQL (por ejemplo <b>sa</b>) o agrega tu usuario de Windows
              como administrador de SQL Server.
            </p>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
              <div className="field" style={{ gridColumn: '1 / -1' }}>
                <label>Servidor</label>
                <input className="input" value={server} onChange={e => setServer(e.target.value)} placeholder=".\\SQLEXPRESS" />
              </div>
              <div className="field">
                <label>Base de datos</label>
                <input className="input" value={name} onChange={e => setName(e.target.value)} placeholder="HiPotDB" />
              </div>
              <div className="field">
                <label>Autenticación</label>
                <select className="input" value={auth} onChange={e => setAuth(e.target.value as any)}>
                  <option value="windows">Windows (usuario actual)</option>
                  <option value="sql">SQL Server (usuario/contraseña)</option>
                </select>
              </div>
              {auth === 'sql' && (
                <>
                  <div className="field">
                    <label>Usuario SQL</label>
                    <input className="input" value={user} onChange={e => setUser(e.target.value)} placeholder="sa" autoComplete="username" />
                  </div>
                  <div className="field">
                    <label>Contraseña SQL</label>
                    <input className="input" type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="••••••" autoComplete="current-password" />
                  </div>
                </>
              )}
            </div>

            <label style={{ display: 'flex', gap: 8, alignItems: 'center', marginTop: 12, fontWeight: 600, fontSize: 13 }}>
              <input type="checkbox" checked={createDb} onChange={e => setCreateDb(e.target.checked)} style={{ width: 18, height: 18 }} />
              Crear la base de datos si no existe
            </label>
            <div style={{ fontSize: 11, color: 'var(--muted)', marginTop: 4 }}>
              Crear la BD requiere que el usuario tenga permiso <code>dbcreator</code> o <code>sysadmin</code>.
              Si usas Windows y el login falla, cambia a autenticación SQL con una cuenta con permisos.
            </div>

            {msg && (
              <div className="alert" style={{ marginTop: 12, background: ok ? '#e6f4ea' : undefined, borderColor: ok ? '#a8d0b4' : undefined, color: ok ? 'var(--ok)' : undefined }}>
                {msg}
              </div>
            )}

            <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 14, flexWrap: 'wrap' }}>
              <button className="btn" onClick={onClose} disabled={busy}>Cerrar</button>
              <button className="btn" onClick={test} disabled={busy}>{busy ? 'Probando…' : 'Probar conexión'}</button>
              <button className="btn primary" onClick={save} disabled={busy}>{busy ? 'Guardando…' : 'Guardar y reintentar'}</button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
