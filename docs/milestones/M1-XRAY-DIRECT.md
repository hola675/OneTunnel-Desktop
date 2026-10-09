# M1 — Xray + 1VPN Direct Gate

**PASS, 7 de octubre de 2026.** Este hito certifica únicamente la conectividad
Direct de 1VPN Free mediante Xray. La VPN del sistema queda para M2.

## Baseline

Commit Direct M1: cb3f6480226f97f86438324a09f76b724175d5e3,
test: certify direct 1vpn xray connectivity; publicado en origin/main antes
de M1.2, con working tree limpio. M1.2 reorganiza estos archivos y conserva
el gate certificado.

M0.2.1: 48720970fcc00625b13aec84a0fd3ed4bcc4b7d4,
test: make runtime smoke cross-shell compatible. Smoke y helper nativo
certificados en PowerShell 7.6.2 y Windows PowerShell 5.1.19041.6456.

## Flujo certificado

~~~text
Windows
  ↓ HTTPS probe / SOCKS loopback
Xray
  ↓ VLESS + Reality / TCP443
1VPN
  ↓
Internet
~~~

tests/fixtures/1vpn/free-contract.json modela datos públicos Free del snapshot
Android: ServerConfigUtil.kt, FreeLocations.kt y xray_config_template.json.
Sus UUID, public key y short ID son constantes públicas upstream, sin secretos
personales ni datos Premium. Hay seis servidores: Amsterdam, Singapore y
Los Angeles, dos por ubicación.

scripts/new-xray-config.ps1 genera un único inbound SOCKS5
127.0.0.1:<dynamic-port>, UDP desactivado, y un único outbound VLESS/TCP/Reality
a 1VPN:443: encryption none, xtls-rprx-vision, fingerprint chrome, public key
y short ID Free y SNI del servidor. No añade DNS ni routing. Los archivos
únicos se generan en %TEMP%\OneTunnel\xray\.

## Evidencia certificada

Ejecución original: %TEMP%\OneTunnel\M1\e561b98cf1d44f018143fd04df7ebb58\.
result.json confirma DirectGate PASS, sin fallo técnico.
La recertificación previa al commit M1 también pasó:
%TEMP%\OneTunnel\M1\791e2e8bfb324affac3581d4adbc588f\.
En ambas ejecuciones egress cambió, el fallback fue exclusivamente por
revocación offline y todos los procesos/configs propios se limpiaron.

| Comprobación | Resultado |
| --- | --- |
| 1VPN Free | PASS — Los Angeles node 1 |
| Xray | PASS — 26.9.9 |
| VLESS / xtls-rprx-vision | PASS |
| Reality | PASS — REALITY_CONFIRMED_BY_TRAFFIC |
| DNS / TCP443 / config offline | PASS |
| SOCKS loopback del PID propio | PASS |
| HTTPS a api.ipify.org mediante SOCKS | PASS_REVOCATION_BEST_EFFORT |
| Egress VPN distinto de egress base | PASS — XRAY_DIRECT_EGRESS_CONFIRMED |
| Cleanup del Xray propio | PASS |
| Configuraciones temporales propias | Eliminadas |
| Cambios de networking Windows | Ninguno |
| Administrador | No requerido |

Servidor: free-los-angeles-node-1.cloudwidecdn.com; Reality SNI:
www.apple.com. La comparación conserva sólo SHA256 de las IPs.
Los logs redactan IPs y constantes Free; no persisten IPs públicas completas.

## Validación tras M1.2

El runner renombrado test-1vpn-direct.ps1 pasó el 7 de octubre de 2026,
sin elevación, después de migrar a schemaVersion 3.
Evidencia: %TEMP%\OneTunnel\M1\079fa021a0b14d0aad5e987b5d1bf2a1\.
Resultado DIRECT_CONTROL_PASS, DirectGate PASS, Reality confirmada por
tráfico, egress distinto, cleanup PASS. STRICT dio el error exacto Schannel
y el único retry best-effort pasó en baseline y SOCKS.
No cambió networking Windows. El runtime activo se verificó con Xray,
tun2socks y Wintun; Wintun y tun2socks no participan en este gate.
El smoke offline posterior pasó en PowerShell 7 y Windows PowerShell 5.1.

## Schannel y política TLS

curl 8.13.0 / Schannel reportó
**CRYPT_E_REVOCATION_OFFLINE (0x80092013)** tanto para HTTPS base como por SOCKS.

1. **STRICT first**: éxito válido devuelve PASS_STRICT, sin retry.
2. Sólo ante exit **35** y el marcador exacto CRYPT_E_REVOCATION_OFFLINE
   o 0x80092013 en **stderr**, repetir una vez con --ssl-revoke-best-effort.
3. Éxito: PASS_REVOCATION_BEST_EFFORT, warning SCHANNEL_REVOCATION_OFFLINE;
   nunca hay un tercer intento.
4. Otros fallos TLS, HTTP, cuerpo IP inválido o conectividad conservan su fallo.

Se mantienen cadena de confianza, hostname, expiración y handshake TLS.
No se usan -k, --insecure ni --ssl-no-revoke. -q va primero para ignorar
curlrc; proxy y noproxy son explícitos para evitar proxies heredados y bypass
del SOCKS. Se comprueba la capacidad best-effort antes de iniciar Xray.

tests/test-https-probe.ps1 verifica offline ambos marcadores, éxitos STRICT,
fallback exitoso/fallido, ausencia de tercer intento, errores TLS genéricos,
timeout, HTTP, IP inválida, streams separados y flags. Corre en ambos shells.

## Runner y límites

~~~powershell
pwsh -NoProfile -File .\tests\test-xray-config.ps1
pwsh -NoProfile -File .\tests\test-https-probe.ps1
pwsh -NoProfile -File .\scripts\test-1vpn-direct.ps1
~~~

El runner prueba únicamente Direct y devuelve DIRECT_CONTROL_PASS / exit 0
si el gate y cleanup pasan; fallos técnicos devuelven FAIL / exit 1.
Una IP sin cambio queda como WARN_IP_UNCHANGED; requiere revisión y no
certifica cambio de egress. La ejecución certificada sí cambió la IP.
Se prueban hasta seis candidatos, empezando por la ubicación seleccionada.

Usa try/finally y objetos Process propios; nunca termina por nombre.
Conserva procesos ajenos. No carga Wintun ni ejecuta tun2socks con un device.
No modifica Schannel, registro, certificados, proxy, rutas, DNS, IPv6 o firewall.
No inicializa Tauri. El siguiente hito técnico es la VPN de sistema Windows.
