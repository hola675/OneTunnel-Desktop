# OneTunnel Desktop

Cliente Windows 10/11 x64 para 1VPN con Xray VLESS + Reality. M1 certifica
conectividad Direct a través del SOCKS local de Xray. M2-R valida el modelo de
aplicación seleccionada con Proxifier externo y manual.

> **M2 IS NOT A SYSTEM VPN.** No captura todo Windows ni cambia rutas, DNS,
> IPv6, firewall o adaptadores.

## Estado

| Hito | Estado |
| --- | --- |
| M0 | ✅ Bootstrap |
| M0.2 | ✅ Runtime supply chain |
| M0.2.1 | ✅ PowerShell cross-shell |
| M1 | ✅ Xray + 1VPN Direct proof |
| M1.2 | ✅ Architecture consolidation (`82082c8`) |
| M2-R | ✅ Xray + Proxifier selective tunnel; M2-R.1 live rule certification passed |
| M3 | ⏳ Native Windows Capture Evaluation (not started) |

## M2-R flow

```text
Selected application
  ↓ Proxifier: OneTunnel-Xray
Xray SOCKS5 127.0.0.1:10808 (M2-R.1 default; must be free)
  ↓ VLESS + Reality
Xray outbound
  ↓ Proxifier: Corporate Proxy
1VPN:443 → Internet
```

Proxifier stays external and user-managed. Expected rules in order: localhost
and loopback DIRECT; `xray.exe` → Corporate Proxy; selected application →
OneTunnel-Xray; Default → DIRECT. OneTunnel does not edit Proxifier, manage
credentials, or start/stop it. M2 does not use Wintun/tun2socks; system capture
is deferred to M3 evaluation. See [M2-R milestone](docs/milestones/M2-PROXIFIER-TUNNEL.md).

## Validate

```powershell
.\scripts\verify-runtime.ps1
pwsh -NoProfile -File .\scripts\smoke-test.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
pwsh -NoProfile -File .\tests\test-native-process.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tests\test-native-process.ps1
pwsh -NoProfile -File .\tests\test-https-probe.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tests\test-https-probe.ps1
pwsh -NoProfile -File .\tests\test-xray-config.ps1
pwsh -NoProfile -File .\scripts\test-1vpn-direct.ps1
pwsh -NoProfile -File .\scripts\test-m2-proxifier-tunnel.ps1 -SocksPort 10808 -ProxifierLogPath 'C:\Logs\Proxifier.log'
```

The M2-R.1 command fails if port 10808 is occupied, then pauses for the user
to configure Proxifier and enable Verbose File Log manually before live probes.
It does not simulate or modify Proxifier. Runtime verification checks hashes;
Wintun and tun2socks are not run by the M2 script.

M1/M2 do not need administrator privileges. The Xray Direct gate requires
PowerShell 7 and Internet, uses public Free fixture values, and applies strict
TLS first with only the exact Schannel revocation-offline fallback.

## Documentation

- [Scope](docs/PROJECT-SCOPE.md), [architecture](docs/ARCHITECTURE.md),
  [network flows](docs/NETWORK-FLOW.md).
- [Corporate proxy policy](docs/CORPORATE-PROXY.md),
  [future Windows capture evaluation](docs/WINDOWS-VPN.md).
- [API 1VPN](docs/API-1VPN.md), [Xray](docs/XRAY-INTEGRATION.md),
  [runtime provenance](docs/RUNTIME-PROVENANCE.md),
  [security](docs/SECURITY-MODEL.md), [build](docs/BUILD.md).

The previous Wintun/tun2socks prototype remains preserved in an external
backup; it is deferred to M3 evaluation.
