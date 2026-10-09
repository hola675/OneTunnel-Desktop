# Project scope

## Platform

Windows 10/11 x64 (x86_64).

## Certified/current

- M1: Xray + 1VPN Free Direct tunnel proof.
- M2-R: Xray SOCKS plus selective application routing through manually
  configured, external Proxifier rules; certification requires a live
  corporate test.
- Proxifier remains optional, user-managed, and outside OneTunnel.

## Future V1 scope

- 1VPN Free/Premium, account flows, and location selection.
- Xray VLESS + Reality, `xtls-rprx-vision`, TCP 443.
- Tauri desktop UI, reconnect, diagnostics, and packaging.
- Possible native Windows capture using Wintun/tun2socks, subject to M3 —
  Native Windows Capture Evaluation.

This is a roadmap, not a statement that those items are implemented. No
Wintun/tun2socks system capture, Windows routing/DNS changes, leak protection,
kill switch, reconnect, UI, or distribution is certified today.

## Out of scope

- OneTunnel-hosted VPN infrastructure.
- Linux, macOS, mobile, and ARM64.
- Multi-provider, WebTransport, and HTTP/2 tunnel.
- Browser extension and advanced split tunneling.

## M2-R boundaries

M2-R does not edit Proxifier profiles or manage credentials. It does not
change Windows routes, DNS, IPv6, firewall, or adapters and does not require
administrator privileges. The previous Wintun/tun2socks prototype is backed
up outside the repository and deferred to M3.
