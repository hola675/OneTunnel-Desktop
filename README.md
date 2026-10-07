# OneTunnel Desktop

Cliente Windows 10/11 x64 para 1VPN. M1 certifica Xray + VLESS + Reality
mediante SOCKS local; todavía no captura tráfico del sistema.

## Estado

| Hito | Resultado |
| --- | --- |
| M0 | PASS — bootstrap y auditoría |
| M0.2 | PASS — runtimes fijados y SHA256 |
| M0.2.1 | PASS — smoke PowerShell 7 / Windows PowerShell 5.1 |
| M1 | PASS — Xray + 1VPN Free Direct |
| M1.2 | Pendiente — consolidación de arquitectura |
| M2 | Pendiente — Windows system VPN |

## Gate certificado

~~~text
Windows HTTPS probe → SOCKS loopback → Xray → VLESS + Reality → 1VPN → Internet
~~~

La evidencia y la política Schannel están en
[Direct Gate M1](docs/M1-TRANSPORT-POC.md).
La prueba usa datos públicos Free, conserva xtls-rprx-vision, confirma HTTPS
y cambio de egress, termina sus procesos propios y elimina sus configuraciones.
No requiere administrador ni cambia rutas, DNS, proxy, firewall o adaptadores.

## Reproducir

~~~powershell
.\scripts\fetch-runtime.ps1
.\scripts\verify-runtime.ps1
pwsh -NoProfile -File .\scripts\smoke-test.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
pwsh -NoProfile -File .\tests\test-poc-config.ps1
pwsh -NoProfile -File .\tests\test-https-probe.ps1
pwsh -NoProfile -File .\scripts\test-m1-transport-poc.ps1
~~~

Las referencias de interoperabilidad permanecen bajo reference/, sin cambios.
Su procedencia y limitaciones están en [upstream baseline](docs/upstream-baseline.md).
app/ y src-tauri/ son placeholders; Tauri todavía no está inicializado.
