import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function useHipot(enabled = true) {
  const [connected, setConnected] = useState<boolean | null>(null);
  const refresh = useCallback(async () => {
    try {
      const r = await invoke<any>('hipot_status');
      setConnected(!!r?.connected);
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
