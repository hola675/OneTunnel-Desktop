# OneTunnel Desktop

Cliente Windows 10/11 x64 para 1VPN con Xray VLESS + Reality.
M1 certifica conectividad Direct mediante SOCKS local. La captura de tráfico
del sistema y la aplicación de escritorio siguen pendientes.

## Estado

| Hito | Estado |
| --- | --- |
| M0 | ✅ Bootstrap |
| M0.2 | ✅ Runtime supply chain |
| M0.2.1 | ✅ PowerShell cross-shell |
| M1 | ✅ Xray + 1VPN Direct proof |
| M1.2 | 🟡 Architecture consolidation — validación y revisión antes del commit |
| M2 | ⏳ Windows system VPN |

## Arquitectura V1

~~~text
Windows applications
  ↓
Wintun
  ↓
tun2socks
  ↓
Xray
  ↓ VLESS + Reality
1VPN
  ↓
Internet
~~~

Compatibilidad CORPORATE_PROXY:

~~~text
Xray.exe
  ↓
Proxifier
  ↓
Corporate Proxy
  ↓
1VPN
~~~

Proxifier es opcional, externo y administrado por el usuario. Sus reglas
aplican únicamente al tráfico outbound de xray.exe; OneTunnel no lo instala,
configura ni guarda credenciales corporativas en V1.
La decisión está en [ADR 0001](docs/adr/0001-v1-network-architecture.md).

## Validar

~~~powershell
.\scripts\fetch-runtime.ps1
.\scripts\verify-runtime.ps1
pwsh -NoProfile -File .\scripts\smoke-test.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
pwsh -NoProfile -File .\tests\test-native-process.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tests\test-native-process.ps1
pwsh -NoProfile -File .\tests\test-https-probe.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tests\test-https-probe.ps1
pwsh -NoProfile -File .\tests\test-xray-config.ps1
pwsh -NoProfile -File .\scripts\test-1vpn-direct.ps1
~~~

El smoke funciona offline en PowerShell 7 y Windows PowerShell 5.1.
El Direct probe requiere PowerShell 7 e Internet, sólo usa datos Free públicos
y no necesita administrador. Su éxito confirma HTTPS; el resumen registra
por separado el cambio de egress. La evidencia certificada está en
[M1 Xray Direct](docs/milestones/M1-XRAY-DIRECT.md), incluida la política
STRICT first y el fallback exacto de revocación offline de Schannel.

## Documentación

- [Scope](docs/PROJECT-SCOPE.md), [arquitectura y estados](docs/ARCHITECTURE.md),
  [flujos](docs/NETWORK-FLOW.md).
- [VPN Windows, rutas, DNS e IPv6](docs/WINDOWS-VPN.md),
  [proxy corporativo](docs/CORPORATE-PROXY.md).
- [API 1VPN](docs/API-1VPN.md), [Xray](docs/XRAY-INTEGRATION.md).
- [Dependencias](docs/DEPENDENCIES.md),
  [procedencia de runtimes](docs/RUNTIME-PROVENANCE.md),
  [seguridad](docs/SECURITY-MODEL.md), [build y validación](docs/BUILD.md).

reference/ conserva snapshots de interoperabilidad sin modificaciones;
[upstream baseline](docs/upstream-baseline.md) documenta sus límites de origen.
app/ y src-tauri/ siguen como placeholders. M1.2 no inicializa Tauri ni modifica
rutas, DNS, proxy, firewall, IPv6 o adaptadores Windows.

Siguiente paso: revisar y commitear M1.2; después M2 — Wintun + tun2socks +
Xray system VPN PoC.
