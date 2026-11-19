import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Lee el paro de emergencia del PLC cada 500 ms.
// Al activarse por primera vez, aborta PLC y Hipot.
export function useEmergencyStop(enabled = true) {
  const [active, setActive] = useState(false);
  const [connected, setConnected] = useState<boolean | null>(null);
  const [plcEnabled, setPlcEnabled] = useState<boolean | null>(null);
  const [raw, setRaw] = useState<boolean | null>(null);
  const [inverted, setInverted] = useState<boolean | null>(null);
  const [address, setAddress] = useState('I0.0');
  const wasActive = useRef(false);

  useEffect(() => {
    if (!enabled) return;
    let mounted = true;

    const poll = async () => {
      try {
        const r = await invoke<any>('plc_emergency_stop');
        if (!mounted) return;
        setActive(!!r?.active);
        setConnected(!!r?.connected);
        setPlcEnabled(typeof r?.enabled === 'boolean' ? r.enabled : null);
        setRaw(typeof r?.raw === 'boolean' ? r.raw : null);
        setInverted(typeof r?.inverted === 'boolean' ? r.inverted : null);
        if (typeof r?.address === 'string' && r.address) setAddress(r.address);
        if (r?.active && !wasActive.current) {
          wasActive.current = true;
          invoke('plc_abort').catch(() => {});
          invoke('hipot_abort').catch(() => {});
        }
        if (!r?.active) wasActive.current = false;
      } catch {
        if (mounted) {
          setConnected(false);
          setPlcEnabled(null);
        }
      }
    };

    poll();
    const id = window.setInterval(poll, 800);
    return () => {
      mounted = false;
      window.clearInterval(id);
    };
  }, [enabled]);

  return { active, connected, enabled: plcEnabled, raw, inverted, address };
}
