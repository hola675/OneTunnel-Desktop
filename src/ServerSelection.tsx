import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

type Server = {
  locationIndex: number
  serverIndex: number
  country: string
  city: string | null
  premium: boolean
  eligible: boolean
  configured: boolean
}

type Props = { authenticated: boolean }

export default function ServerSelection({ authenticated }: Props) {
  const [servers, setServers] = useState<Server[]>([])
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [mode, setMode] = useState<'automatic' | 'manual'>('automatic')
  const [selected, setSelected] = useState<string>('')
  const [onlyAvailable, setOnlyAvailable] = useState(true)
  const load = async () => {
    if (!authenticated) { setServers([]); return }
    setBusy(true)
    setError(null)
    try { setServers(await invoke<Server[]>('list_1vpn_servers')) }
    catch { setError('No fue posible consultar los servidores de la sesión.') }
    finally { setBusy(false) }
  }
  useEffect(() => { void load() }, [authenticated]) // eslint-disable-line react-hooks/exhaustive-deps
  const candidates = useMemo(() => servers.filter(s => s.eligible && s.configured), [servers])
  const key = (s: Server) => `${s.locationIndex}:${s.serverIndex}`
  const chosen = mode === 'manual' ? candidates.find(s => key(s) === selected) : undefined
  const visibleServers = onlyAvailable ? candidates : servers
  const premiumLocked = servers.filter(s => !s.eligible).length
  const invalidServers = servers.filter(s => s.eligible && !s.configured).length
  return <section className="panel preferences" aria-labelledby="servers-heading">
    <div className="section-top on-panel"><div><p className="kicker">UBICACIONES 1VPN</p><h2 id="servers-heading">Selección de servidor</h2></div><button type="button" className="text-action" onClick={() => void load()} disabled={busy || !authenticated}>{busy ? 'Consultando…' : 'Actualizar ↻'}</button></div>
    <p className="panel-subtitle">Catálogo real de tu cuenta Free/Premium. La elección automática por latencia requiere mediciones de red que aún no están disponibles.</p>
    <div className="setting-row"><div><strong>Modo de selección</strong><p>La selección automática no se activa hasta medir y validar servidores.</p></div><select aria-label="Modo de selección de servidor" value={mode} onChange={e => setMode(e.target.value as 'automatic' | 'manual')}><option value="automatic">Automático (pendiente)</option><option value="manual">Manual</option></select></div>
    <div className="setting-row"><div><strong>Mostrar solo servidores disponibles</strong><p>Filtra ubicaciones Premium bloqueadas y configuraciones incompletas.</p></div><input type="checkbox" checked={onlyAvailable} onChange={e => setOnlyAvailable(e.target.checked)} aria-label="Mostrar solo servidores disponibles" /></div>
    {authenticated && <p className="small-muted" aria-live="polite">{candidates.length} aptos · {premiumLocked} restringidos por plan · {invalidServers} con configuración incompleta</p>}
    {mode === 'automatic' && <div className="message info-message">No se ha calculado un mejor servidor. Sin datos de latencia, pérdida de paquetes y salida HTTPS, no se asignan puntuaciones ficticias.</div>}
    {mode === 'manual' && <div className="setting-row"><div><strong>Servidor preferido</strong><p>Solo ubicaciones autorizadas y con configuración válida.</p></div><select aria-label="Servidor preferido" value={selected} onChange={e => setSelected(e.target.value)}><option value="">Seleccionar ubicación</option>{visibleServers.map(s => <option disabled={!s.eligible || !s.configured} key={key(s)} value={key(s)}>{s.country}{s.city ? ` · ${s.city}` : ''} · #{s.serverIndex + 1}{s.premium ? ' · Premium' : ' · Free'}</option>)}</select></div>}
    {chosen && <p className="footnote">Preferencia local: {chosen.country}{chosen.city ? ` · ${chosen.city}` : ''}. Todavía no inicia una conexión ni modifica Proxifier.</p>}
    {error && <p className="message error-message" role="alert">{error}</p>}
    <p className="footnote">{authenticated ? `${candidates.length} servidores autorizados y configurados de ${servers.length} publicados por tu cuenta.` : 'Inicia sesión en 1VPN para consultar los servidores.'} Sin pruebas de latencia ni reconexión automática activas.</p>
  </section>
}
