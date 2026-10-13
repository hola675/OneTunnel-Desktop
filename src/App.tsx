import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type AppInfo = {
  appName: string
  version: string
  platform: string
  coreStatus: string
}

export default function App() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    invoke<AppInfo>('get_app_info')
      .then(setAppInfo)
      .catch(() => setError('Native application information is unavailable.'))
  }, [])

  return (
    <main className="window-shell">
      <header className="topbar">
        <div className="brand-mark" aria-hidden="true">O</div>
        <div>
          <p className="eyebrow">ONETUNNEL</p>
          <h1>Desktop</h1>
        </div>
        <span className="build-tag">FOUNDATION</span>
      </header>

      <section className="connection-card" aria-labelledby="connection-title">
        <div className="status-line">
          <span className="status-dot" aria-hidden="true" />
          <span>DISCONNECTED</span>
        </div>
        <h2 id="connection-title">Your connection is off</h2>
        <p className="muted">Choose an account and server to get started.</p>
        <button className="connect-button" type="button" disabled>
          Connect <span aria-hidden="true">→</span>
        </button>
        <p className="availability-note">Connection will be available in a future release.</p>
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
          <span className="future-badge">PLANNED</span>
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
