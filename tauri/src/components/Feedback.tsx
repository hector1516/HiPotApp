import React, { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';

export type ToastVariant = 'info' | 'ok' | 'error' | 'warn';
type ToastItem = { id: number; message: string; variant: ToastVariant };
export type ConfirmOptions = {
  title?: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
};
type ConfirmState = (ConfirmOptions & { resolve: (value: boolean) => void }) | null;

let toasts: ToastItem[] = [];
let confirmState: ConfirmState = null;
let nextId = 1;
const toastListeners = new Set<(items: ToastItem[]) => void>();
const confirmListeners = new Set<(state: ConfirmState) => void>();

function emitToasts() {
  toastListeners.forEach(fn => fn([...toasts]));
}
function emitConfirm() {
  confirmListeners.forEach(fn => fn(confirmState));
}

export function notify(message: string, variant: ToastVariant = 'info') {
  const item = { id: nextId++, message, variant };
  toasts = [...toasts, item].slice(-4);
  emitToasts();
  window.setTimeout(() => {
    toasts = toasts.filter(t => t.id !== item.id);
    emitToasts();
  }, variant === 'error' ? 6500 : 4200);
}

export function askConfirm(input: string | ConfirmOptions): Promise<boolean> {
  const options: ConfirmOptions = typeof input === 'string'
    ? { title: 'Confirmar acción', message: input, danger: true, confirmLabel: 'Confirmar' }
    : input;
  return new Promise(resolve => {
    confirmState = { ...options, resolve };
    emitConfirm();
  });
}

function resolveConfirm(value: boolean) {
  if (!confirmState) return;
  const resolve = confirmState.resolve;
  confirmState = null;
  emitConfirm();
  resolve(value);
}

export function FeedbackHost() {
  const [items, setItems] = useState<ToastItem[]>([]);
  const [confirm, setConfirm] = useState<ConfirmState>(null);

  useEffect(() => {
    toastListeners.add(setItems);
    confirmListeners.add(setConfirm);
    setItems([...toasts]);
    setConfirm(confirmState);
    return () => {
      toastListeners.delete(setItems);
      confirmListeners.delete(setConfirm);
    };
  }, []);

  return (
    <>
      <div className="toast-host">
        <AnimatePresence>
          {items.map(t => (
            <motion.div
              key={t.id}
              className={`toast ${t.variant}`}
              initial={{ opacity: 0, y: 12, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 8, scale: 0.98 }}
              transition={{ duration: 0.2 }}
            >
              <span style={{ flex: 1 }}>{t.message}</span>
              <button className="close" onClick={() => {
                toasts = toasts.filter(x => x.id !== t.id);
                emitToasts();
              }}>×</button>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>

      <AnimatePresence>
        {confirm && (
          <motion.div
            className="confirm-overlay"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={() => resolveConfirm(false)}
          >
            <motion.div
              className="confirm-card"
              initial={{ opacity: 0, scale: 0.96, y: 10 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.96, y: 10 }}
              transition={{ duration: 0.2 }}
              onClick={e => e.stopPropagation()}
            >
              <h3>{confirm.title || 'Confirmar acción'}</h3>
              <p>{confirm.message}</p>
              <div className="confirm-actions">
                <button className="btn" onClick={() => resolveConfirm(false)}>
                  {confirm.cancelLabel || 'Cancelar'}
                </button>
                <button className={`btn ${confirm.danger ? 'danger' : 'primary'}`} onClick={() => resolveConfirm(true)}>
                  {confirm.confirmLabel || 'Confirmar'}
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
