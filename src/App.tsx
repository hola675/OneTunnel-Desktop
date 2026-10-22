import { useEffect, useState, type FormEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'
import TrafficRules from './TrafficRules'

type Account = { username: string; isPremium: boolean; locationCount: number }
type LoginResult = { status: 'authenticated'; account: Account } | { status: 'requires_2fa' }
type EngineStatus = {
  phase: 'disconnected' | 'starting' | 'running' | 'error'
  pid: number | null
  socksAddress: string
  socksReady: boolean
  verified: boolean
  egressIp: string | null
  message: string
}
type Location = {
  index: number
  name: string
  countryCode: string
  isPremium: boolean
  servers: { index: number; host: string }[]
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
  const [section, setSection] = useState<Section>('overview')
  const [preferences, setPreferences] = useState<Preference>(initialPreferences)
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null)
  const [runtime, setRuntime] = useState<RuntimeCheck | null>(null)
  const [engine, setEngine] = useState<EngineStatus | null>(null)
  const [account, setAccount] = useState<Account | null>(null)
  const [locations, setLocations] = useState<Location[]>([])
  const [selectedLocation, setSelectedLocation] = useState<number | null>(null)
  const [selectedServer, setSelectedServer] = useState<number | null>(null)
  const [connectionBusy, setConnectionBusy] = useState(false)
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [twoFactor, setTwoFactor] = useState('')
  const [requires2fa, setRequires2fa] = useState(false)
  const [loginBusy, setLoginBusy] = useState(false)
  const [refreshBusy, setRefreshBusy] = useState(false)
  const [loginError, setLoginError] = useState<string | null>(null)
  const [systemError, setSystemError] = useState<string | null>(null)
  const [lastChecked, setLastChecked] = useState<Date | null>(null)

  const readLocations = async () => {
    try {
      const available = await invoke<Location[]>('list_1vpn_locations')
      setLocations(available)
      setSelectedLocation(available[0]?.index ?? null)
      setSelectedServer(available[0]?.servers[0]?.index ?? null)
    } catch {
      setSystemError('No se pudieron obtener los servidores autorizados de 1VPN.')
    }
  }
  const readEngine = async () => {
    try {
      const status = await invoke<EngineStatus>('get_engine_status')
      setEngine(status)
      setLastChecked(new Date())
    } catch {
      setSystemError('No se pudo consultar el estado del motor.')
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
    void readLocations()
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
        await readLocations()
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
      setLocations([])
      setSelectedLocation(null)
      setSelectedServer(null)
      await readEngine()
      setPassword('')
      setTwoFactor('')
      setRequires2fa(false)
    } catch { setLoginError('No se pudo cerrar la sesión.') }
  }

  const activeLocation = locations.find(location => location.index === selectedLocation)
  const selectionReady = Boolean(activeLocation?.servers.some(server => server.index === selectedServer))
  const phase = engine?.phase ?? 'disconnected'
  const socksReady = Boolean(engine?.socksReady)
  const runtimeReady = Boolean(runtime?.ready)
  const connectionVerified = Boolean(engine?.verified && engine?.socksReady)
  const connectionLabel = phase === 'error' ? 'Error del motor' :
    connectionVerified ? 'Túnel HTTPS verificado' :
    phase === 'running' ? 'Motor activo sin verificar' :
    phase === 'starting' ? 'Comprobando conexión' : 'Desconectado'
  const readiness = [Boolean(account), runtimeReady, selectionReady]
  const toggleConnection = async () => {
    if (connectionBusy) return
    setConnectionBusy(true)
    setSystemError(null)
    try {
      const status = engine?.pid
        ? await invoke<EngineStatus>('disconnect_1vpn')
        : await invoke<EngineStatus>('connect_1vpn', {
            locationIndex: selectedLocation,
            serverIndex: selectedServer
          })
      setEngine(status)
      setLastChecked(new Date())
    } catch (reason) {
      setSystemError(typeof reason === 'string' ? reason : 'Error al iniciar o detener el túnel.')
      await readEngine()
    } finally {
      setConnectionBusy(false)
    }
  }
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

        {systemError && <div role="alert" className="message error-message">{systemError}<button type="button" onClick={() => setSystemError(null)} aria-label="Cerrar aviso">×</button></div>}

        {section === 'overview' && <>
          <section className="hero" aria-labelledby="connection-heading">
            <div className="hero-copy">
              <div className="status-row"><span className={`signal ${phase === 'error' ? 'signal-danger' : ''}`}/>{connectionLabel.toUpperCase()}</div>
              <h2 id="connection-heading">Tu túnel está {connectionVerified ? 'verificado' : 'sin verificar'}</h2>
              <p>Conexión VLESS/Reality a través de un proxy SOCKS5 local. No modifica las rutas de Windows.</p>
            </div>
            <div className="hero-action">
              {!engine?.pid && <div className="server-choice">
                <label htmlFor="vpn-location">Ubicación 1VPN</label>
                <select id="vpn-location" value={selectedLocation ?? ''} disabled={!locations.length || connectionBusy}
                  onChange={event => {
                    const index = Number(event.target.value)
                    const location = locations.find(item => item.index === index)
                    setSelectedLocation(index)
                    setSelectedServer(location?.servers[0]?.index ?? null)
                  }}>
                  {!locations.length && <option value="">Inicia sesión para elegir</option>}
                  {locations.map(location => <option key={location.index} value={location.index}>
                    {location.name}{location.isPremium ? ' · Premium' : ' · Free'}
                  </option>)}
                </select>
                <label htmlFor="vpn-server">Servidor</label>
                <select id="vpn-server" value={selectedServer ?? ''} disabled={!selectionReady || connectionBusy}
                  onChange={event => setSelectedServer(Number(event.target.value))}>
                  {(activeLocation?.servers ?? []).map(server => <option key={server.index} value={server.index}>
                    {server.host}
                  </option>)}
                </select>
              </div>}
              <button type="button" className="primary-button"
                onClick={() => void toggleConnection()}
                disabled={connectionBusy || (!engine?.pid && (!account || !runtimeReady || !selectionReady))}>
                {connectionBusy ? 'Procesando…' : engine?.pid ? 'Desconectar' : 'Conectar'}
                <span aria-hidden="true">↗</span>
              </button>
              <span>{connectionVerified ? 'SOCKS5 y salida HTTPS comprobados' : 'Solo las aplicaciones configuradas utilizarán el proxy SOCKS5'}</span>
            </div>
          </section>

          <div className="section-top"><h2>Resumen operativo</h2><span>Información proporcionada por el motor local</span></div>
          <div className="metrics">
            <article className="metric"><span>CUENTA 1VPN</span><strong>{account ? account.isPremium ? 'Premium' : 'Free' : 'Sin sesión'}</strong><small>{account ? account.username : 'Accede para ver tus ubicaciones'}</small></article>
            <article className="metric"><span>UBICACIONES DISPONIBLES</span><strong>{locations.length || '—'}</strong><small>{account ? 'Servidores autorizados y válidos' : 'Inicia sesión para consultar'}</small></article>
            <article className="metric"><span>SOCKS5 LOCAL</span><strong>{socksReady ? 'Responde' : 'No verificado'}</strong><small className="monospace">{engine?.socksAddress ?? '127.0.0.1:10808'}</small></article>
          </div>

          <div className="section-top"><h2>Preparación del túnel</h2><span>{readyCount} de 3 requisitos de conexión</span></div>
          <div className="readiness-track" aria-label={`${readyCount} de 3 requisitos de conexión listos`}><div style={{ width: `${readyCount / 3 * 100}%` }} /></div>
          <div className="checklist">
            <div><span className={account ? 'check-pass' : 'check-wait'}>{account ? '✓' : '○'}</span><div><strong>Cuenta</strong><small>{account ? 'Sesión activa' : 'Sin iniciar sesión'}</small></div><button type="button" onClick={() => setSection('account')}>Gestionar →</button></div>
            <div><span className={runtimeReady ? 'check-pass' : 'check-wait'}>{runtimeReady ? '✓' : '○'}</span><div><strong>Motor Xray</strong><small>{runtimeReady ? 'Hash y versión verificados' : 'Verificación pendiente'}</small></div><button type="button" onClick={() => setSection('diagnostics')}>Revisar →</button></div>
            <div><span className={selectionReady ? 'check-pass' : 'check-wait'}>{selectionReady ? '✓' : '○'}</span><div><strong>Servidor autorizado</strong><small>{selectionReady ? 'Destino Reality válido seleccionado' : 'Selecciona un servidor de tu cuenta'}</small></div><button type="button" onClick={() => setSection('overview')}>Seleccionar →</button></div>
          </div>
          <p className="footnote">Conectado solo significa que una prueba HTTPS pasó a través del SOCKS5 de OneTunnel. No hay VPN global, captura WFP ni garantía de protección de aplicaciones externas.</p>
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
              <span className="tiny-label">SALIDA HTTPS</span><strong>{connectionVerified ? 'Verificada' : 'Sin verificar'}</strong>
              <span className="small-muted">{connectionVerified ? 'Se verificó HTTPS sobre SOCKS5 al conectar (no es vigilancia continua).' : 'Requiere iniciar el motor y pasar la prueba HTTPS.'}</span>
              <div className="separator"/><span className="technical">IP de salida: {connectionVerified ? engine?.egressIp : '—'}</span>
            </article>
          </div>
          <p className="footnote">Última consulta local: {lastChecked?.toLocaleTimeString() ?? '—'}. Las métricas no verificadas se mantienen vacías.</p>
        </>}

        {section === 'settings' && <>
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
          <span>{appInfo?.coreStatus ?? 'Motor sin inicializar'} · Cliente SOCKS5</span>
        </footer>
      </div>
    </main>
  )
}
