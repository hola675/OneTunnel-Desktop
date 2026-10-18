import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

type Action = 'tunnel' | 'direct' | 'block'
type Rule = { id: string; enabled: boolean; application: string | null; hostPattern: string | null; portStart: number | null; portEnd: number | null; action: Action }
type Policy = { revision: number; defaultAction: Action; rules: Rule[]; enforcementActive: boolean }
type Decision = { action: Action; matchedRule: string; applied: boolean; explanation: string }
type Capability = { wfpEngineRunning: boolean; interceptionActive: boolean; redirectSupported: boolean; blockingSupported: boolean }

export default function TrafficRules() {
  const [policy, setPolicy] = useState<Policy | null>(null)
  const [capability, setCapability] = useState<Capability | null>(null)
  const [app, setApp] = useState('firefox.exe')
  const [host, setHost] = useState('example.com')
  const [pattern, setPattern] = useState('*.example.com')
  const [port, setPort] = useState(443)
  const [action, setAction] = useState<Action>('tunnel')
  const [result, setResult] = useState<Decision | null>(null)
  const [error, setError] = useState<string | null>(null)
  const refresh = async () => {
    try {
      setPolicy(await invoke<Policy>('get_traffic_policy'))
      setCapability(await invoke<Capability>('get_capture_capability'))
      setError(null)
    } catch { setError('No se pudo consultar el motor de reglas.') }
  }
  useEffect(() => { void refresh() }, [])
  const makeRule = (id: string): Rule => ({
    id, enabled: true, application: app.trim() || null,
    hostPattern: pattern.trim() || null, portStart: port, portEnd: port, action
  })
  const preview = async () => {
    setError(null)
    try {
      setResult(await invoke<Decision>('preview_traffic_rule', {
        rules: [makeRule('preview')], defaultAction: 'direct',
        connection: { application: app, destinationHost: host, destinationPort: port }
      }))
    } catch (reason) { setError(String(reason)) }
  }
  const save = async (clear = false) => {
    if (!policy) return
    setError(null)
    try {
      const rules = clear ? [] : [...policy.rules, makeRule('rule-' + (policy.revision + 1))]
      const next = await invoke<Policy>('save_traffic_policy', {
        rules, defaultAction: 'direct', expectedRevision: policy.revision
      })
      setPolicy(next)
    } catch (reason) { setError(String(reason)) }
  }

  return <section className="panel preferences">
    <div className="section-top on-panel"><div><p className="kicker">TRÁFICO DE WINDOWS</p><h2>Reglas por aplicación y destino</h2></div></div>
    <p className="panel-subtitle">Simulador seguro: aún no captura, bloquea ni redirige conexiones reales. Proxifier sigue siendo externo.</p>
    <p className="footnote">WFP/BFE: {capability?.wfpEngineRunning ? 'detectado' : 'no verificado'} · Captura: {capability?.interceptionActive ? 'activa' : 'inactiva'} · Redirección: {capability?.redirectSupported ? 'disponible' : 'no disponible'}.</p>
    <div className="traffic-fields">
      <label>Aplicación<input value={app} maxLength={255} onChange={e => setApp(e.target.value)} /></label>
      <label>Destino para simular<input value={host} maxLength={255} onChange={e => setHost(e.target.value)} /></label>
      <label>Patrón de la regla<input value={pattern} maxLength={255} onChange={e => setPattern(e.target.value)} /></label>
      <label>Puerto<input type="number" min={1} max={65535} value={port} onChange={e => setPort(Number(e.target.value))} /></label>
      <label>Acción<select value={action} onChange={e => setAction(e.target.value as Action)}><option value="tunnel">Túnel</option><option value="direct">Directo</option><option value="block">Bloquear</option></select></label>
    </div>
    <div className="traffic-actions">
      <button className="secondary-button" onClick={() => void preview()} type="button">Simular</button>
      <button className="secondary-button" onClick={() => void save()} disabled={!policy} type="button">Guardar en memoria</button>
      <button className="secondary-button" onClick={() => void save(true)} disabled={!policy?.rules.length} type="button">Vaciar reglas</button>
    </div>
    {result && <p role="status" className="footnote">Simulación: {result.action.toUpperCase()} · Regla: {result.matchedRule} · Aplicada a Windows: NO.</p>}
    {error && <p className="message error-message" role="alert">{error}</p>}
    <p className="footnote">{policy ? `${policy.rules.length} reglas guardadas en memoria · revisión ${policy.revision}` : 'Política sin cargar'}</p>
    {policy?.rules.map(rule => <p className="footnote" key={rule.id}>{rule.application || '*'} → {rule.hostPattern || '*'}:{rule.portStart || '*'} · {rule.action}</p>)}
  </section>
}
