# Network flow

M1 certifies the direct Xray tunnel. M2-R adds selective routing through
user-managed Proxifier rules. Neither milestone captures all Windows traffic.

## Xray Direct control

```text
Xray SOCKS client (HTTPS control)
   ↓ SOCKS5 127.0.0.1:<port>
Xray
   ↓ VLESS + Reality
1VPN:443 → Internet
```

This control demonstrates Xray tunnel traffic. It does not certify routing by
Proxifier or a Windows system VPN.

## M2-R selected application

```text
Selected application
   ↓ Proxifier rule: OneTunnel-Xray
Xray SOCKS5 at 127.0.0.1:10808 (M2-R.1 certification default)
   ↓ Xray VLESS + Reality
Xray outbound
   ↓ Proxifier rule: Corporate Proxy
Corporate Proxy → 1VPN:443 → Internet
```

Expected rule order: localhost/loopback DIRECT; `xray.exe` → Corporate Proxy;
selected test application → OneTunnel-Xray; Default → DIRECT. Proxifier rules
are configured manually. The OneTunnel probe reports PARTIAL unless it has
evidence of both rules; an HTTPS request through Xray SOCKS alone is not enough.

## Unselected application

An application outside the OneTunnel-Xray rule remains on the normal
corporate/network path. The M2 script uses a separate PowerShell HTTPS request
as the unselected control and hashes egress values instead of persisting full
IP addresses.

## System capture

```text
Wintun → tun2socks → Xray
```

This is deferred research for M3 — Native Windows Capture Evaluation. M2-R
does not execute these runtimes or modify Windows routes, DNS, IPv6, firewall,
or adapters.
