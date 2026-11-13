p = 'tauri/src/pages/AdminPanel.tsx'
content = open(p, encoding='utf-8').read()
marker = '      <div className="card" style={{ minWidth: 0 }}>\n        <h3>Import / Export</h3>'
card = '''      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Actualizacion y version</h3>
        <div style={{ fontSize: 12, color: '#475569', marginBottom: 8 }}>
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
          {updateStatus && <span style={{ fontSize: 12, color: '#64748b' }}>{updateStatus}</span>}
        </div>
      </div>
      <div className="card" style={{ marginBottom: 10 }}>
        <h3>Datos de debug</h3>
        <div style={{ fontSize: 12, color: '#475569', display: 'flex', flexDirection: 'column', gap: 4 }}>
          <div>Backend: <span className="font-mono">http://127.0.0.1:8000</span></div>
          <div>PLC: <span className="font-mono">{plcStatus?.ip || cfg?.plc_ip || '-'}</span> ({plcStatus?.connected ? 'conectado' : 'desconectado'})</div>
          <div>Hipot: <span className="font-mono">{hipotStatus?.port || '-'}</span></div>
          <div>SQL: <span className="font-mono">{cfg?.sql_server || '-'}</span> / {cfg?.sql_database || '-'}</div>
          <button className="btn ghost" style={{ fontSize: 11, marginTop: 4 }} onClick={async () => {
            try {
              const h = await api.get('/health');
              const p = await api.get('/status/plc');
              const hp = await api.get('/status/hipot');
              alert(`Health: ${JSON.stringify(h)}\\nPLC: ${JSON.stringify(p)}\\nHipot: ${JSON.stringify(hp)}`);
            } catch (e: any) { alert('Error: ' + (e?.message || e)); }
          }}>Ver estado completo</button>
        </div>
      </div>
'''
if marker in content:
    content = content.replace(marker, card + marker, 1)
    open(p, 'w', encoding='utf-8').write(content)
    print("inserted")
else:
    print("marker not found")