# Project scope

## Plataforma V1

- Windows 10/11
- x86_64

## Incluido en V1

- 1VPN Free
- 1VPN Premium
- 1VPN 2FA
- 1VPN locations
- Xray
- VLESS + Reality
- wstunnel
- WSS/TLS
- TCP/443
- VPN de sistema
- Wintun
- tun2socks
- DNS seguro
- route management
- rollback
- auto reconnect
- basic kill switch
- system tray
- diagnostics
- portable distribution

## Fuera de V1

- Linux
- macOS
- Android
- iOS
- ARM64
- multi-provider
- browser extension
- server management GUI
- WebTransport
- UDP transport
- advanced per-app split tunneling

## Restricciones de M0

M0 no desarrolla GUI ni conexión VPN. No instala/configura Wintun, no cambia
rutas, DNS o Windows Firewall, no implementa Kill Switch ni tráfico completo del
sistema y no modifica las referencias oficiales de 1VPN. Las carpetas de app y
`src-tauri` son únicamente placeholders; Tauri no se inicializa aún.
