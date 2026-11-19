import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Estado de conexión PLC (poll cada 2s) vía Tauri command (sin backend HTTP).
export function usePLC(enabled = true) {
  const [connected, setConnected] = useState<boolean | null>(null);
  const [detail, setDetail] = useState('');
  const refresh = useCallback(async () => {
    try {
      const r = await invoke<any>('plc_status_info');
      setConnected(!!r?.connected);
      setDetail(`${r?.ip ?? ''}${r?.simulate ? ' (sim)' : ''}`);
    } catch { setConnected(false); }
  }, []);
  useEffect(() => {
    if (!enabled) return;
    refresh();
    const id = setInterval(refresh, 5000);
    return () => clearInterval(id);
  }, [enabled, refresh]);
  return { connected, detail, refresh };
}
