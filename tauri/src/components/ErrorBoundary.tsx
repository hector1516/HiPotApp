import React from 'react';

type State = { error: Error | null };

export class ErrorBoundary extends React.Component<{ children: React.ReactNode }, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('UI render error:', error, info);
    import('@tauri-apps/api/core')
      .then(({ invoke }) =>
        invoke('log_action', {
          level: 'ERROR',
          category: 'UI',
          action: 'render_error',
          detail: String(error?.stack || error),
          errorMessage: String(error?.message || error),
          success: false,
        })
      )
      .catch(() => {});
  }

  render() {
    if (this.state.error) {
      return (
        <div style={{
          minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center',
          background: '#0f172a', color: '#e2e8f0', padding: 24, fontFamily: 'system-ui, sans-serif',
        }}>
          <div style={{ maxWidth: 760, width: '100%' }}>
            <h1 style={{ margin: '0 0 12px 0', fontSize: 24, color: '#fca5a5' }}>Error en la interfaz</h1>
            <p style={{ color: '#94a3b8', marginBottom: 16 }}>
              La aplicación encontró un error inesperado. El evento quedó registrado en el log.
            </p>
            <pre style={{
              background: '#1e293b', border: '1px solid #334155', borderRadius: 8,
              padding: 12, fontSize: 12, overflow: 'auto', maxHeight: 240, whiteSpace: 'pre-wrap',
            }}>{String(this.state.error.message || this.state.error)}</pre>
            <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
              <button className="btn primary" onClick={() => location.reload()}>Reiniciar aplicación</button>
              <button className="btn" onClick={() => this.setState({ error: null })}>Volver a intentar</button>
            </div>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
