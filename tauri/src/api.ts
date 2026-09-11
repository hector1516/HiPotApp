// Cliente HTTP al backend FastAPI (offline-first: reintenta con 127.0.0.1 para evitar demoras IPv6 en Windows).
const BASE = (import.meta as any).env?.VITE_API_URL ?? 'http://127.0.0.1:8000';

async function req(path: string, init?: RequestInit, retries = 3): Promise<any> {
  let lastErr: any = null;
  for (let i = 0; i < retries; i++) {
    try {
      const r = await fetch(`${BASE}${path}`, {
        headers: { 'Content-Type': 'application/json' },
        ...init,
      });
      if (!r.ok) {
        const t = await r.text().catch(() => '');
        throw new Error(t || `HTTP ${r.status}`);
      }
      return await r.json().catch(() => ({}));
    } catch (err) {
      lastErr = err;
      if (i < retries - 1) {
        await new Promise(res => setTimeout(res, 350 * (i + 1)));
      }
    }
  }
  throw lastErr;
}

export const api = {
  get: (p: string) => req(p),
  post: (p: string, body?: any) =>
    req(p, { method: 'POST', body: body === undefined ? undefined : JSON.stringify(body) }),
  put: (p: string, body?: any) =>
    req(p, { method: 'PUT', body: body === undefined ? undefined : JSON.stringify(body) }),
  del: (p: string) => req(p, { method: 'DELETE' }),
  scan: (code: string, barcode_type: string, scan_order: number, execution_id?: number) =>
    req('/barcode/scan', { method: 'POST', body: JSON.stringify({ code, barcode_type, scan_order, execution_id }) }),
  recipeByModel: (m: string) => req(`/recipes/model/${encodeURIComponent(m)}`),
};

