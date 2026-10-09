---
title: "OneTunnel V1 Architecture"
status: "Accepted"
date: "2026-10-07"
authors: "OneTunnel maintainers"
tags: ["architecture", "windows", "network"]
supersedes: ""
superseded_by: ""
---

# OneTunnel V1 Architecture

## Status

Accepted for V1. La decisión establece el objetivo; no certifica que la VPN
del sistema ni el modo corporativo completo ya estén implementados.

## Context

M1 certificó Xray → 1VPN Free con VLESS + Reality, xtls-rprx-vision,
SOCKS loopback, HTTPS y egress distinto del directo.
La validación corporativa aportada para esta decisión observó que Xray
alcanza 1VPN mediante el proxy corporativo existente cuando xray.exe se
enruta por Proxifier. El gate automatizado del repositorio certifica Direct.

## Decision

| Elemento | Elección |
| --- | --- |
| VPN engine | Xray |
| VPN protocol | VLESS + Reality |
| System capture | Wintun |
| IP → proxy adaptation | tun2socks |
| Provider | 1VPN |
| Network modes | DIRECT, CORPORATE_PROXY |

### DIRECT

~~~text
Windows Applications
  ↓
Wintun
  ↓
tun2socks
  ↓
Xray
  ↓
1VPN
  ↓
Internet
~~~

No external compatibility dependency. El proveedor sigue siendo 1VPN;
OneTunnel no necesita infraestructura remota propia para este modo.

### CORPORATE_PROXY

~~~text
Windows Applications
  ↓
Wintun
  ↓
tun2socks
  ↓
Xray.exe
  ↓
Proxifier
  ↓
Corporate Proxy
  ↓
1VPN
  ↓
Internet
~~~

Proxifier is an OPTIONAL EXTERNAL COMPATIBILITY COMPONENT.
OneTunnel does NOT bundle Proxifier, own the corporate proxy or store
corporate proxy credentials in V1. Lo administra el usuario y no se instala
automáticamente. El proceso conserva el nombre xray.exe.
Loopback permanece DIRECT y sólo el outbound de xray.exe se proxifica.

### WSTUNNEL DECISION

~~~text
wstunnel: REMOVED_FROM_V1
~~~

A remote wstunnel relay would require infrastructure owned or operated by
OneTunnel. Testing supplied for this decision demonstrated that Xray can
already reach 1VPN through the existing corporate proxy when xray.exe is
routed through Proxifier. Therefore a custom relay adds unnecessary
infrastructure, latency, maintenance and operational dependency to V1.

No habrá servidor externo OneTunnel, relay propio ni WSS relay.
wstunnel may be reconsidered in the future, but it is NOT runtime,
NOT active dependency, NOT V1 feature and NOT active roadmap item.
Git conserva la historia; no se crea un árbol archive ni provenance activo.

## Consequences

### Positive

- **POS-001**: Tres runtimes con pins y hashes verificables.
- **POS-002**: El modo Direct aprovecha el gate ya certificado.
- **POS-003**: Se elimina operación de infraestructura OneTunnel y su coste
  de latencia y mantenimiento.

### Negative

- **NEG-001**: Compatibilidad corporativa depende de un componente externo
  configurado por el usuario y de las capacidades del proxy existente.
- **NEG-002**: Captura Windows, DNS, IPv6, rutas y rollback requieren diseño
  e implementación adicional antes de certificar VPN del sistema.
- **NEG-003**: La evidencia Direct no certifica cada red corporativa ni Premium.

## Alternatives Considered

### Remote wstunnel / WSS relay

- **ALT-001**: Transporte TCP a infraestructura remota operada para OneTunnel.
- **ALT-002**: Rechazado para V1 por dependencia operativa innecesaria dado el
  acceso corporativo observado con Proxifier.

### Automatizar o empaquetar Proxifier

- **ALT-003**: Instalar/configurar reglas y gestionar credenciales desde OneTunnel.
- **ALT-004**: Fuera de esta fase y de la política de distribución V1:
  configuración externa y ninguna persistencia de contraseñas corporativas.

## Implementation Notes

- **IMP-001**: runtime/manifest.json schemaVersion 3 sólo contiene xray,
  tun2socks y wintun; se mantienen versiones, SHA256 y licencias upstream.
- **IMP-002**: M1.2 no modifica networking Windows ni Proxifier ni inicializa
  Tauri. Rutas, DNS y protección IPv6 se documentan para M2/M3.
- **IMP-003**: Toda mutación Windows debe tener inversa conocida, snapshot,
  rollback y verificación de recuperación.
- **IMP-004**: El Direct Gate se conserva como milestone y runner independiente.

## References

- **REF-001**: [Network flow](../NETWORK-FLOW.md).
- **REF-002**: [Windows VPN](../WINDOWS-VPN.md).
- **REF-003**: [Corporate proxy policy](../CORPORATE-PROXY.md).
- **REF-004**: [M1 Xray Direct](../milestones/M1-XRAY-DIRECT.md).
