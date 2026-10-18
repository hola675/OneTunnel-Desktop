import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

type Account = { username: string; isPremium: boolean; locationCount: number }
type LoginResult = { status: 'authenticated'; account: Account } | { status: 'requires_2fa' }

type AppInfo = {
  appName: string
  version: string
  platform: string
  coreStatus: string
}

type EngineStatus = {
  phase: 'disconnected' | 'starting' | 'running' | 'error'
  pid: number | null
  socksAddress: string
  socksReady: boolean
  message: string
}

type RuntimeCheck = {
  ready: boolean
  executable: string | null
  versionOutput: string | null
  message: string
}

export default function App() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [twoFactor, setTwoFactor] = useState('')
  const [requires2fa, setRequires2fa] = useState(false)
  const [loginBusy, setLoginBusy] = useState(false)
  const [loginError, setLoginError] = useState<string | null>(null)
  const [account, setAccount] = useState<Account | null>(null)
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null)
  const [runtime, setRuntime] = useState<RuntimeCheck | null>(null)
  const [engine, setEngine] = useState<EngineStatus | null>(null)
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
    invoke<Account | null>('get_account').then(setAccount).catch(() => setLoginError('Account status unavailable.'))
    const refresh = () => {
      invoke<EngineStatus>('get_engine_status')
        .then(setEngine)
        .catch(() => setError('Engine status unavailable.'))
    }
    refresh()
    const interval = window.setInterval(refresh, 2500)
    return () => window.clearInterval(interval)
  }, [])

  const submitLogin = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (loginBusy) return
    setLoginBusy(true)
    setLoginError(null)
    try {
      const result = await invoke<LoginResult>('login_1vpn', {
        email, password, token: requires2fa ? twoFactor : null
      })
      if (result.status === 'requires_2fa') {
        setRequires2fa(true)
        setTwoFactor('')
      } else {
        setAccount(result.account)
        setPassword('')
        setTwoFactor('')
        setRequires2fa(false)
      }
    } catch (reason) {
      setLoginError(typeof reason === 'string' ? reason : 'Login could not be completed.')
    } finally {
      setLoginBusy(false)
    }
  }

  const logout = async () => {
    try {
      await invoke('logout_1vpn')
      setAccount(null)
      setPassword('')
      setTwoFactor('')
      setRequires2fa(false)
      setLoginError(null)
    } catch {
      setLoginError('Could not log out.')
    }
  }

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

      <section className="detail-card" aria-label="1VPN account" style={{ marginTop: 16 }}>
        <p className="detail-label">1VPN ACCOUNT</p>
        {account ? (
          <div>
            <p className="detail-value">{account.username} · {account.isPremium ? 'Premium' : 'Free'}</p>
            <p className="muted">{account.locationCount} available locations reported</p>
            <button type="button" onClick={logout} style={{ marginTop: 12 }}>Log out</button>
          </div>
        ) : (
          <form onSubmit={submitLogin} autoComplete="on" style={{ display: 'grid', gap: 10, marginTop: 12 }}>
            <label htmlFor="vpn-email">Email</label>
            <input id="vpn-email" type="email" autoComplete="username" required
              value={email} onChange={e => setEmail(e.target.value)} maxLength={254} />
            <label htmlFor="vpn-password">Password</label>
            <input id="vpn-password" type="password" autoComplete="current-password" required
              value={password} onChange={e => setPassword(e.target.value)} />
            {requires2fa && <>
              <label htmlFor="vpn-2fa">Verification code</label>
              <input id="vpn-2fa" autoComplete="one-time-code" inputMode="numeric" required
                value={twoFactor} onChange={e => setTwoFactor(e.target.value)} maxLength={64} />
            </>}
            <button type="submit" disabled={loginBusy}>{loginBusy ? 'Signing in…' : requires2fa ? 'Verify and sign in' : 'Sign in to 1VPN'}</button>
            {loginError && <p className="backend-error" role="alert">{loginError}</p>}
            <p className="muted">Your password is not saved. The session lasts only while this app is running.</p>
          </form>
        )}
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
        <article className="detail-card">
          <p className="detail-label">ENGINE</p>
          <p className="detail-value">{engine?.phase ?? 'Checking…'}</p>
          <p className="availability-note" style={{ textAlign: 'left' }}>{engine?.message ?? 'Reading native state'} · SOCKS {engine?.socksReady ? 'responding' : 'not verified'}</p>
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
