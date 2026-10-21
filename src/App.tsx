import { useEffect, useRef, useState, type FormEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'
import TrafficRules from './TrafficRules'
import ServerSelection from './ServerSelection'

type DiagnosticEvent = { at: string; category: 'motor' | 'sistema' | 'cuenta'; message: string }
type Account = { username: string; isPremium: boolean; locationCount: number }
type LoginResult = { status: 'authenticated'; account: Account } | { status: 'requires_2fa' }
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
type AppInfo = { appName: string; version: string; platform: string; coreStatus: string }
type Section = 'overview' | 'account' | 'diagnostics' | 'settings'
type Preference = { refreshInterval: number; compact: boolean; notifications: boolean }
const initialPreferences: Preference = { refreshInterval: 2500, compact: false, notifications: true }

export default function App() {
  const pollingInFlight = useRef(false)
  const lastEngineEvent = useRef<string | null>(null)
  const [events, setEvents] = useState<DiagnosticEvent[]>([])
  const recordEvent = (category: DiagnosticEvent['category'], message: string) => {
    setEvents(previous => [{ at: new Date().toLocaleTimeString(), category, message }, ...previous].slice(0, 80))
  }
  const [section, setSection] = useState<Section>('overview')
  const [preferences, setPreferences] = useState<Preference>(initialPreferences)
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null)
  const [runtime, setRuntime] = useState<RuntimeCheck | null>(null)
  const [engine, setEngine] = useState<EngineStatus | null>(null)
  const [account, setAccount] = useState<Account | null>(null)
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [twoFactor, setTwoFactor] = useState('')
  const [requires2fa, setRequires2fa] = useState(false)
  const [loginBusy, setLoginBusy] = useState(false)
  const [refreshBusy, setRefreshBusy] = useState(false)
  const [stopBusy, setStopBusy] = useState(false)
  const [enginePollFailures, setEnginePollFailures] = useState(0)
  const [loginError, setLoginError] = useState<string | null>(null)
  const [systemError, setSystemError] = useState<string | null>(null)
  const [lastChecked, setLastChecked] = useState<Date | null>(null)

  const readEngine = async () => {
    if (pollingInFlight.current) return
    pollingInFlight.current = true
    try {
      const status = await invoke<EngineStatus>('get_engine_status')
      setEngine(status)
      setEnginePollFailures(0)
      setSystemError(previous => previous === 'No se pudo consultar el estado del motor.' ? null : previous)
      setLastChecked(new Date())
      const signature = [status.phase, status.pid ?? '-', status.socksReady, status.message].join('|')
      if (lastEngineEvent.current !== signature) {
        lastEngineEvent.current = signature
        recordEvent('motor', `${status.phase} · SOCKS5: ${status.socksReady ? 'disponible' : 'no disponible'} · ${status.message}`)
      }
    } catch {
      setEnginePollFailures(previous => previous + 1)
      setEngine(null)
      if (lastEngineEvent.current !== 'unreachable') {
        lastEngineEvent.current = 'unreachable'
        recordEvent('sistema', 'No se pudo consultar el estado del motor.')
      }
      setSystemError('No se pudo consultar el estado del motor.')
    } finally {
      pollingInFlight.current = false
    }
  }
  const checkRuntime = async () => {
    try { setRuntime(await invoke<RuntimeCheck>('inspect_xray_runtime')) }
    catch { setSystemError('No se pudo verificar el ejecutable Xray.') }
  }
  const refreshDiagnostics = async () => {
    if (refreshBusy) return
    setRefreshBusy(true)
    setSystemError(null)
    await Promise.all([readEngine(), checkRuntime()])
    setRefreshBusy(false)
  }

  useEffect(() => {
    invoke<AppInfo>('get_app_info').then(setAppInfo)
      .catch(() => setSystemError('El backend de OneTunnel no responde.'))
    invoke<Account | null>('get_account').then(setAccount)
      .catch(() => setSystemError('No se pudo consultar la cuenta.'))
    void readEngine()
    void checkRuntime()
    // Keep polling independent of the selected screen.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    const timer = window.setInterval(() => { void readEngine() }, preferences.refreshInterval)
    return () => window.clearInterval(timer)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [preferences.refreshInterval])

  const submitLogin = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (loginBusy) return
    setLoginBusy(true)
    setLoginError(null)
    try {
      const result = await invoke<LoginResult>('login_1vpn', {
        email: email.trim(), password, token: requires2fa ? twoFactor.trim() : null
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
      setLoginError(typeof reason === 'string' ? reason : 'No se pudo iniciar sesión.')
    } finally {
      setLoginBusy(false)
    }
  }
  const logout = async () => {
    setLoginError(null)
    try {
      await invoke('logout_1vpn')
      setAccount(null)
      setPassword('')
      setTwoFactor('')
      setRequires2fa(false)
    } catch { setLoginError('No se pudo cerrar la sesión.') }
  }

  const stopEngine = async () => {
    if (stopBusy) return
    setStopBusy(true)
    try {
      setEngine(await invoke<EngineStatus>('stop_engine'))
      recordEvent('motor', 'Desconexión manual solicitada; proceso Xray propio detenido.')
    } catch { setSystemError('No se pudo detener el motor Xray. Comprueba el diagnóstico.') }
    finally { setStopBusy(false) }
  }

  const phase = engine?.phase ?? 'disconnected'
  const socksReady = Boolean(engine?.socksReady)
  const runtimeReady = Boolean(runtime?.ready)
  const connectionVerified = false // No HTTPS egress verification IPC exists yet.
  const connectionLabel = phase === 'error' ? 'Error del motor' :
    phase === 'running' ? 'Motor en ejecución (sin verificar)' :
    phase === 'starting' ? 'Iniciando motor' : 'Desconectado'
  const readiness = [Boolean(account), runtimeReady, socksReady]
  const readyCount = readiness.filter(Boolean).length
  const nav: { id: Section; label: string; symbol: string }[] = [
    { id: 'overview', label: 'Inicio', symbol: '◉' },
    { id: 'account', label: 'Cuenta', symbol: '♙' },
    { id: 'diagnostics', label: 'Diagnóstico', symbol: '▤' },
    { id: 'settings', label: 'Ajustes', symbol: '⚙' }
  ]

  return (
    <main className={`desktop ${preferences.compact ? 'compact' : ''}`}>
      <aside className="sidebar" aria-label="Navegación principal">
        <div className="brand">
          <div className="brand-icon" aria-hidden="true">O</div>
          <div><strong>OneTunnel</strong><span>DESKTOP VPN</span></div>
        </div>
        <nav className="navigation" aria-label="Secciones">
          {nav.map(item => (
            <button key={item.id} type="button"
              aria-current={section === item.id ? 'page' : undefined}
              className={`nav-link ${section === item.id ? 'selected' : ''}`}
              onClick={() => setSection(item.id)}>
              <span className="nav-symbol" aria-hidden="true">{item.symbol}</span>{item.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <div className="tiny-label">ESTADO DEL SERVICIO</div>
          <div className="sidebar-state"><span className={`signal ${phase === 'error' ? 'signal-danger' : ''}`} />{connectionLabel}</div>
          <span className="small-muted">Proxy local · Sin VPN de sistema</span>
        </div>
      </aside>

      <div className="workspace">
        <header className="workspace-header">
          <div>
            <p className="kicker">PANEL DE CONTROL</p>
            <h1>{section === 'overview' ? 'Tu conexión' : section === 'account' ? 'Tu cuenta' : section === 'diagnostics' ? 'Centro de diagnóstico' : 'Configuración'}</h1>
          </div>
          <div className="header-actions">
            <span className="account-chip">{account ? (account.isPremium ? '★ PREMIUM' : 'FREE') : 'SIN SESIÓN'}</span>
            <button type="button" className="icon-action" title="Actualizar diagnóstico" aria-label="Actualizar diagnóstico"
              onClick={() => void refreshDiagnostics()} disabled={refreshBusy}>{refreshBusy ? '…' : '↻'}</button>
          </div>
        </header>

        {enginePollFailures >= 2 && <div role="status" className="message error-message">Se ha perdido la comunicación con el motor ({enginePollFailures} consultas fallidas). No se puede confirmar el estado de la conexión. La aplicación seguirá comprobándolo.</div>}
        {systemError && <div role="alert" className="message error-message">{systemError}<button type="button" onClick={() => setSystemError(null)} aria-label="Cerrar aviso">×</button></div>}

        {section === 'overview' && <>
          <section className="hero" aria-labelledby="connection-heading">
            <div className="hero-copy">
              <div className="status-row"><span className={`signal ${phase === 'error' ? 'signal-danger' : ''}`}/>{connectionLabel.toUpperCase()}</div>
              <h2 id="connection-heading">Tu túnel está {connectionVerified ? 'verificado' : 'sin verificar'}</h2>
              <p>Conexión VLESS/Reality a través de un proxy SOCKS5 local. No modifica las rutas de Windows.</p>
            </div>
            <div className="hero-action">
              {engine?.pid != null ? <button type="button" className="primary-button" onClick={() => void stopEngine()} disabled={stopBusy}>{stopBusy ? 'Deteniendo…' : 'Desconectar motor'} <span aria-hidden="true">■</span></button> : <button type="button" className="primary-button" disabled title="Requiere conexión y verificación HTTPS del túnel">Conectar <span aria-hidden="true">↗</span></button>}
              <span>{engine?.pid != null ? 'Detiene solo Xray administrado por OneTunnel; no cambia Proxifier.' : 'Disponible al completar la validación de red'}</span>
            </div>
          </section>

          <div className="section-top"><h2>Resumen operativo</h2><span>Información proporcionada por el motor local</span></div>
          <div className="metrics">
            <article className="metric"><span>CUENTA 1VPN</span><strong>{account ? account.isPremium ? 'Premium' : 'Free' : 'Sin sesión'}</strong><small>{account ? account.username : 'Accede para ver tus ubicaciones'}</small></article>
            <article className="metric"><span>UBICACIONES DISPONIBLES</span><strong>{account ? account.locationCount : '—'}</strong><small>{account ? 'Según la sesión 1VPN' : 'Inicia sesión para consultar'}</small></article>
            <article className="metric"><span>SOCKS5 LOCAL</span><strong>{socksReady ? 'Responde' : 'No verificado'}</strong><small className="monospace">{engine?.socksAddress ?? '127.0.0.1:10808'}</small></article>
          </div>

          <div className="section-top"><h2>Preparación del túnel</h2><span>{readyCount} de 3 comprobaciones básicas</span></div>
          <div className="readiness-track" aria-label={`${readyCount} de 3 comprobaciones básicas listas`}><div style={{ width: `${readyCount / 3 * 100}%` }} /></div>
          <div className="checklist">
            <div><span className={account ? 'check-pass' : 'check-wait'}>{account ? '✓' : '○'}</span><div><strong>Cuenta</strong><small>{account ? 'Sesión activa' : 'Sin iniciar sesión'}</small></div><button type="button" onClick={() => setSection('account')}>Gestionar →</button></div>
            <div><span className={runtimeReady ? 'check-pass' : 'check-wait'}>{runtimeReady ? '✓' : '○'}</span><div><strong>Motor Xray</strong><small>{runtimeReady ? 'Hash y versión verificados' : 'Verificación pendiente'}</small></div><button type="button" onClick={() => setSection('diagnostics')}>Revisar →</button></div>
            <div><span className={socksReady ? 'check-pass' : 'check-wait'}>{socksReady ? '✓' : '○'}</span><div><strong>Puerto SOCKS5</strong><small>{socksReady ? 'Responde al handshake' : 'Sin respuesta verificada'}</small></div><button type="button" onClick={() => setSection('diagnostics')}>Revisar →</button></div>
          </div>
          <p className="footnote">Las comprobaciones básicas no equivalen a una VPN conectada. Para ello hace falta verificar salida HTTPS real. No se muestran velocidades ni tráfico inventados.</p>
        </>}

        {section === 'account' && <section className="panel account-panel">
          <div className="section-top on-panel"><div><p className="kicker">AUTENTICACIÓN</p><h2>Cuenta 1VPN</h2></div><span className="account-chip">{account?.isPremium ? 'PREMIUM' : account ? 'FREE' : 'NO AUTENTICADO'}</span></div>
          {account ? <>
            <div className="profile-row"><div className="avatar">{account.username.slice(0, 1).toUpperCase()}</div><div><strong>{account.username}</strong><p>{account.isPremium ? 'Cuenta Premium' : 'Cuenta Free'} · {account.locationCount} ubicaciones reportadas</p></div></div>
            <div className="message info-message">Sesión protegida en la memoria del proceso. Se cerrará al salir de OneTunnel.</div>
            <button type="button" className="secondary-button" onClick={() => void logout()}>Cerrar sesión</button>
          </> : <>
            <p className="panel-subtitle">Inicia sesión con tu correo y contraseña de 1VPN. El acceso Free/Premium lo determina tu cuenta.</p>
            <form className="login-form" onSubmit={submitLogin}>
              <label htmlFor="vpn-email">Correo electrónico</label>
              <input id="vpn-email" type="email" required autoComplete="username" placeholder="tu@correo.com"
                value={email} onChange={e => setEmail(e.target.value)} maxLength={254} />
              <label htmlFor="vpn-password">Contraseña</label>
              <input id="vpn-password" type="password" required autoComplete="current-password" placeholder="Introduce tu contraseña"
                value={password} onChange={e => setPassword(e.target.value)} />
              {requires2fa && <>
                <div className="message info-message">1VPN solicita verificación adicional. Introduce el código de tu cuenta.</div>
                <label htmlFor="vpn-2fa">Código de verificación</label>
                <input id="vpn-2fa" required inputMode="numeric" autoComplete="one-time-code"
                  value={twoFactor} onChange={e => setTwoFactor(e.target.value)} maxLength={64} />
              </>}
              {loginError && <div className="message error-message" role="alert">{loginError}</div>}
              <button type="submit" className="primary-button" disabled={loginBusy}>{loginBusy ? 'Verificando…' : requires2fa ? 'Verificar cuenta' : 'Iniciar sesión'} <span aria-hidden="true">→</span></button>
            </form>
            <p className="footnote">No guardamos la contraseña. El acceso requiere conexión HTTPS con 1VPN.</p>
          </>}
        </section>}

        {section === 'diagnostics' && <>
          <div className="section-top"><h2>Estado técnico</h2><button className="text-action" type="button" disabled={refreshBusy} onClick={() => void refreshDiagnostics()}>{refreshBusy ? 'Actualizando…' : 'Actualizar ahora ↻'}</button></div>
          <div className="diagnostics-grid">
            <article className="panel diagnostic">
              <span className="tiny-label">PROCESO XRAY</span><strong>{phase.toUpperCase()}</strong>
              <span className="small-muted">{engine?.message ?? 'Esperando respuesta de Rust'}</span>
              <div className="separator"/><span className="technical">PID: {engine?.pid ?? '—'}</span>
            </article>
            <article className="panel diagnostic">
              <span className="tiny-label">INTEGRIDAD DEL EJECUTABLE</span><strong>{runtimeReady ? 'Verificada' : 'Pendiente'}</strong>
              <span className="small-muted">{runtime?.message ?? 'Sin datos de verificación'}</span>
              <div className="separator"/><span className="technical">{runtime?.versionOutput ?? 'Sin versión confirmada'}</span>
            </article>
            <article className="panel diagnostic">
              <span className="tiny-label">PROXY LOCAL</span><strong>{socksReady ? 'Handshake OK' : 'No comprobado'}</strong>
              <span className="small-muted">SOCKS5 · Solo aplicaciones configuradas</span>
              <div className="separator"/><span className="technical">{engine?.socksAddress ?? '127.0.0.1:10808'}</span>
            </article>
            <article className="panel diagnostic">
              <span className="tiny-label">SALIDA HTTPS</span><strong>Pendiente</strong>
              <span className="small-muted">No existe aún una prueba de salida VPN extremo a extremo.</span>
              <div className="separator"/><span className="technical">IP de salida: — · Latencia: —</span>
            </article>
          </div>
          <section className="panel preferences" aria-label="Registro de eventos de diagnóstico">
            <div className="section-top on-panel"><h2>Registro de eventos</h2><button type="button" className="text-action" onClick={() => setEvents([])} disabled={events.length === 0}>Limpiar registro</button></div>
            <p className="footnote">Últimos 80 cambios locales de estado. Registro temporal, sin contraseñas ni tokens. No representa tráfico VPN.</p>
            <div role="log" aria-live="off" style={{ maxHeight: 240, overflowY: 'auto' }}>
              {events.length === 0 ? <p className="small-muted">Todavía no hay eventos registrados.</p> : events.map((event, index) => (
                <div className="setting-row" key={`${event.at}-${index}`}><span className="technical">{event.at} · {event.category}</span><span className="small-muted">{event.message}</span></div>
              ))}
            </div>
          </section>
          <p className="footnote">Última consulta local: {lastChecked?.toLocaleTimeString() ?? '—'}. Las métricas no verificadas se mantienen vacías.</p>
        </>}

        {section === 'settings' && <>
          <ServerSelection authenticated={Boolean(account)} />
          <TrafficRules />
          <section className="panel preferences">
            <div className="section-top on-panel"><div><p className="kicker">PREFERENCIAS</p><h2>Interfaz y diagnóstico</h2></div></div>
            <div className="setting-row"><div><strong>Actualización del estado</strong><p>Frecuencia de lectura del estado local del motor</p></div>
              <select aria-label="Frecuencia de actualización" value={preferences.refreshInterval}
                onChange={e => setPreferences(p => ({ ...p, refreshInterval: Number(e.target.value) }))}>
                <option value={2500}>2,5 segundos</option><option value={5000}>5 segundos</option><option value={10000}>10 segundos</option>
              </select></div>
            <div className="setting-row"><div><strong>Interfaz compacta</strong><p>Reduce los espacios entre tarjetas</p></div>
              <input type="checkbox" aria-label="Interfaz compacta" checked={preferences.compact}
                onChange={e => setPreferences(p => ({ ...p, compact: e.target.checked }))}/></div>
            <div className="setting-row"><div><strong>Notificaciones del sistema</strong><p>Próximamente: avisos de conexión, errores y reconexión</p></div>
              <input type="checkbox" checked={preferences.notifications} disabled aria-label="Notificaciones aún no disponibles" readOnly /></div>
            <p className="footnote">Los cambios de interfaz son temporales y no alteran la red corporativa ni Proxifier.</p>
          </section>
          <section className="panel preferences"><div className="section-top on-panel"><div><p className="kicker">RED Y SEGURIDAD</p><h2>Configuración de túnel</h2></div></div>
            <div className="setting-row"><div><strong>Tipo de conexión</strong><p>Proxy SOCKS5 para aplicaciones compatibles</p></div><span className="static-value">VLESS / Reality</span></div>
            <div className="setting-row"><div><strong>Puerto local</strong><p>Interfaz de escucha</p></div><span className="static-value monospace">127.0.0.1:10808</span></div>
            <div className="setting-row"><div><strong>Proxy corporativo</strong><p>Gestionado externamente por Proxifier; OneTunnel no modifica sus reglas</p></div><span className="static-value">Externo</span></div>
            <div className="setting-row"><div><strong>Selección automática / estadísticas</strong><p>Activable después de probar servidores y tráfico real</p></div><span className="static-value">No disponible</span></div>
          </section>
        </>}

        <footer className="workspace-footer">
          <span>{appInfo ? `${appInfo.appName} · v${appInfo.version} · ${appInfo.platform}` : 'OneTunnel Desktop · Consultando versión'}</span>
          <span>{appInfo?.coreStatus ?? 'Motor sin inicializar'} · Interfaz de diagnóstico</span>
        </footer>
      </div>
    </main>
  )
}
