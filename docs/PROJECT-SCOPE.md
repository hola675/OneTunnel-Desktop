# Project scope

## Plataforma V1

Windows 10/11 x64 (x86_64).

## Incluido en V1

- 1VPN Free, Premium, 2FA y selección de locations.
- Xray VLESS + Reality, xtls-rprx-vision y salida TCP443.
- Wintun y tun2socks para VPN de sistema.
- Modos DIRECT y CORPORATE_PROXY; compatibilidad externa opcional con Proxifier.
- Gestión de rutas y DNS, protección contra fugas IPv6 y rollback.
- Reconnect, kill switch, diagnósticos redactados y system tray.
- Tauri desktop UI; distribución portable y con instalador.

Es el alcance objetivo, no una lista de funcionalidades implementadas.
Sólo el gate Xray + 1VPN Free Direct está certificado.
Premium, 2FA, captura de sistema, protección de fugas, reconnect, kill switch,
UI y distribución quedan por implementar y verificar.

## Fuera de V1

- Infraestructura remota propia y servidor VPN OneTunnel/self-hosted.
- Linux, macOS, mobile y ARM64.
- Multi-provider, WebTransport y HTTP2 tunnel.
- Extensión de navegador y split tunneling avanzado por aplicación.

Las exclusiones de transporte y su justificación se declaran en
[ADR 0001](adr/0001-v1-network-architecture.md); no son elementos activos
del runtime ni del roadmap V1.

## Límites de M1.2

Consolida arquitectura, runtime, scripts y documentación.
No aplica networking Windows ni automatiza Proxifier.
reference/ no se modifica; app/ y src-tauri/ conservan sus placeholders.
No se inicializa Tauri y no se añade UI. El siguiente hito técnico es M2.
