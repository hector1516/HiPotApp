from pathlib import Path

p = Path('tauri/src/components/ReprintPopup.tsx')
s = p.read_text(encoding='utf-8')
s = s.replace(
    "const [busy, setBusy] = useState(false);",
    "const [busy, setBusy] = useState(false);\n  const [loading, setLoading] = useState(false);",
)
s = s.replace("    setMsg('');\n    try {", "    setMsg(''); setLoading(true);\n    try {")
s = s.replace(
    "} catch (e: any) { setMsg(String(e?.message ?? e)); }\n  };",
    "} catch (e: any) { setMsg(String(e?.message ?? e)); }\n    finally { setLoading(false); }\n  };",
)
p.write_text(s, encoding='utf-8')
print('reprint loading added')
