// Cliente HTTP al backend FastAPI (offline-first: reintenta, no bloquea).
const BASE = (import.meta as any).env?.VITE_API_URL ?? 'http://127.0.0.1:8765';

async function req(path: string, init?: RequestInit, retries = 2) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 3000);
  try {
    const r = await fetch(`${BASE}${path}`, {
      headers: { 'Content-Type': 'application/json' },
      signal: controller.signal,
      ...init,
    });
    clearTimeout(timeoutId);
    if (!r.ok) {
      const t = await r.text().catch(() => '');
      throw new Error(t || `HTTP ${r.status}`);
    }
    return r.json().catch(() => ({}));
  } catch (err) {
    clearTimeout(timeoutId);
    throw err;
  }
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
