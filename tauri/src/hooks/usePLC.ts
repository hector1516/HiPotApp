import { useCallback, useEffect, useState } from 'react';
import { api } from '../api';

// Estado de conexión PLC (poll cada 2s).
export function usePLC(enabled = true) {
  const [connected, setConnected] = useState<boolean | null>(null);
  const [detail, setDetail] = useState('');
  const refresh = useCallback(async () => {
    try {
      const r = await api.get('/status/plc');
      setConnected(!!r.connected); setDetail(r.ip ?? '');
    } catch { setConnected(false); }
  }, []);
  useEffect(() => {
    if (!enabled) return;
    refresh();
    const id = setInterval(refresh, 2000);
    return () => clearInterval(id);
  }, [enabled, refresh]);
  return { connected, detail, refresh };
}
