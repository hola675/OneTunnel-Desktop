import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type AppInfo = {
  appName: string
  version: string
  platform: string
  coreStatus: string
}

type RuntimeCheck = {
  ready: boolean
  executable: string | null
  versionOutput: string | null
  message: string
}

export default function App() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null)
  const [runtime, setRuntime] = useState<RuntimeCheck | null>(null)
  const [error, setError] = useState<string | null>(null)

  const checkRuntime = () => {
    invoke<RuntimeCheck>('inspect_xray_runtime')
      .then(setRuntime)
      .catch(() => setError('Could not inspect the local Xray runtime.'))
  }

  useEffect(() => {
    invoke<AppInfo>('get_app_info')
      .then(setAppInfo)
      .catch(() => setError('Native application information is unavailable.'))
    checkRuntime()
  }, [])

  return (
    <main className="window-shell">
      <header className="topbar">
        <div className="brand-mark" aria-hidden="true">O</div>
        <div>
          <p className="eyebrow">ONETUNNEL</p>
          <h1>Desktop</h1>
        </div>
        <span className="build-tag">F1 PREFLIGHT</span>
      </header>

      <section className="connection-card" aria-labelledby="connection-title">
        <div className="status-line">
          <span className="status-dot" aria-hidden="true" />
          <span>DISCONNECTED</span>
        </div>
        <h2 id="connection-title">Your connection is off</h2>
        <p className="muted">Xray must be validated before any connection can start.</p>
        <button className="connect-button" type="button" disabled>
          Connect <span aria-hidden="true">→</span>
        </button>
        <p className="availability-note">Tunnel connection is not implemented yet.</p>
      </section>

      <section className="details-grid" aria-label="Connection details">
        <article className="detail-card">
          <p className="detail-label">ACCOUNT</p>
          <p className="detail-value">Not signed in</p>
        </article>
        <article className="detail-card">
          <p className="detail-label">SERVER</p>
          <p className="detail-value">Not selected</p>
        </article>
        <article className="detail-card socks-card">
          <div>
            <p className="detail-label">LOCAL SOCKS5</p>
            <p className="detail-value mono">127.0.0.1:10808</p>
          </div>
          <span className="future-badge">NOT LISTENING</span>
        </article>
        <article className="detail-card socks-card">
          <div>
            <p className="detail-label">XRAY RUNTIME</p>
            <p className="detail-value">{runtime ? (runtime.ready ? 'Executable detected' : 'Unavailable') : 'Checking…'}</p>
            <p className="availability-note" style={{ textAlign: 'left' }}>
              {runtime?.versionOutput ?? runtime?.message ?? 'Inspecting native runtime'}
            </p>
          </div>
          <button type="button" onClick={checkRuntime}>Recheck</button>
        </article>
      </section>

      <footer className="app-footer">
        {error ? (
          <span className="backend-error">{error}</span>
        ) : appInfo ? (
          <span>{appInfo.appName} · v{appInfo.version} · {appInfo.platform}</span>
        ) : (
          <span>Reading native application info…</span>
        )}
        <span className="core-state">CORE · {appInfo?.coreStatus ?? '…'}</span>
      </footer>
    </main>
  )
}
