p = 'tauri/src/pages/AdminPanel.tsx'
s = open(p, encoding='utf-8').read()
old = """      else {
        await api.post('/auth/register',{username:payload.username,password:'',role:payload.role});
        if(!payload.is_active) await api.put(`/users/${(await api.get('/users')).find((x:any)=>x.username===payload.username)?.id}`,{is_active:false});
        setMsg('Usuario creado �?" gafete listo');
      }"""
new = """      else {
        await invoke('add_user', { username: payload.username, password: '', role: payload.role });
        setMsg('Usuario creado - gafete listo');
      }"""
if old in s:
    s = s.replace(old, new)
    open(p, 'w', encoding='utf-8').write(s)
    print("done")
else:
    print("not found")