import { useCallback, useEffect, useState } from 'react';

// Auto-check cada 4h (doc 10). Degrada a "sin updates" fuera de Tauri.
interface Info { version: string; notes: string; date: string }
interface State {
  checking: boolean; available: boolean; downloading: boolean;
  progress: number; updateInfo: Info | null; error: string | null;
}

export function useUpdater() {
  const [s, setS] = useState<State>({
    checking: false, available: false, downloading: false,
    progress: 0, updateInfo: null, error: null,
  });

  const checkForUpdates = useCallback(async () => {
    setS(p => ({ ...p, checking: true, error: null }));
    try {
      const mod = await import('@tauri-apps/plugin-updater').catch(() => null);
      if (!mod) { setS(p => ({ ...p, checking: false })); return; }
      const update = await (mod as any).check();
      if (update) {
        setS(p => ({
          ...p, checking: false, available: true,
          updateInfo: { version: update.version, notes: update.body ?? '', date: update.date ?? new Date().toISOString() },
        }));
      } else setS(p => ({ ...p, checking: false, available: false }));
    } catch (e: any) {
      setS(p => ({ ...p, checking: false, error: String(e?.message ?? e) }));
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    setS(p => ({ ...p, downloading: true, progress: 0 }));
    try {
      const mod = await import('@tauri-apps/plugin-updater');
      const proc = await import('@tauri-apps/plugin-process');
      const update = await (mod as any).check();
      if (!update) { setS(p => ({ ...p, downloading: false })); return; }
      let done = 0, total = 0;
      await update.downloadAndInstall((ev: any) => {
        if (ev.event === 'Started') total = ev.data.contentLength ?? 0;
        else if (ev.event === 'Progress') {
          done += ev.data.chunkLength;
          setS(p => ({ ...p, progress: total ? Math.round((done / total) * 100) : 0 }));
        } else if (ev.event === 'Finished') setS(p => ({ ...p, progress: 100 }));
      });
      await (proc as any).relaunch();
    } catch (e: any) {
      setS(p => ({ ...p, downloading: false, error: String(e?.message ?? e) }));
    }
  }, []);

  useEffect(() => {
    checkForUpdates();
    const id = setInterval(checkForUpdates, 4 * 60 * 60 * 1000);
    return () => clearInterval(id);
  }, [checkForUpdates]);

  return { ...s, checkForUpdates, downloadAndInstall };
}
