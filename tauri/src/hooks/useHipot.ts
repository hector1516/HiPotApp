import { useCallback, useEffect, useState } from 'react';
import { api } from '../api';

export function useHipot(enabled = true) {
  const [connected, setConnected] = useState<boolean | null>(null);
  const refresh = useCallback(async () => {
    try {
      const r = await api.get('/status/hipot');
      setConnected(!!r.connected);
    } catch { setConnected(false); }
  }, []);
  useEffect(() => {
    if (!enabled) return;
    refresh();
    const id = setInterval(refresh, 2000);
    return () => clearInterval(id);
  }, [enabled, refresh]);
  return { connected, refresh };
}
