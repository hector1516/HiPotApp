import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Estado de conexión PLC (poll cada 2s) vía Tauri command (sin backend HTTP).
export function usePLC(enabled = true) {
  const [connected, setConnected] = useState<boolean | null>(null);
  const [detail, setDetail] = useState('');
  const refresh = useCallback(async () => {
    try {
      const cfg = await invoke<any>('get_config');
      const ip = cfg?.plc_ip || '192.168.1.10';
      const rack = parseInt(cfg?.plc_rack || '0', 10);
      const slot = parseInt(cfg?.plc_slot || '1', 10);
      const r = await invoke<boolean>('plc_status', { ip, rack, slot });
      setConnected(!!r); setDetail(ip);
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
