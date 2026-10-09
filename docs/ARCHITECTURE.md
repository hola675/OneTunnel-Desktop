# Architecture

## Implemented milestones

| Component | Current role |
| --- | --- |
| Xray | Local SOCKS5 → VLESS + Reality → 1VPN |
| Proxifier | External, user-managed selective application routing and corporate proxy compatibility |
| Wintun / tun2socks | Not used; deferred to M3 Native Windows Capture Evaluation |

M1 certifies the Xray + 1VPN Direct path. M2-R adds a selective application
path through externally configured Proxifier. It is not a system VPN: no
adapter, routes, DNS, IPv6, or firewall settings are changed. Runtime
verification checks hashes only; M2 does not run Wintun or tun2socks.

The accepted design history is in [ADR 0001](adr/0001-v1-network-architecture.md).
The implemented M2-R flow is in [NETWORK-FLOW](NETWORK-FLOW.md), its external
rule policy in [CORPORATE-PROXY](CORPORATE-PROXY.md), and future system capture
evaluation in [WINDOWS-VPN](WINDOWS-VPN.md).

## M2-R selective flow

```text
Selected application
    ↓ Proxifier: OneTunnel-Xray
Xray SOCKS5 at 127.0.0.1:10808 (M2-R.1 certification default)
    ↓ Xray VLESS + Reality
Xray outbound connection
    ↓ Proxifier: Corporate Proxy (user-managed)
1VPN:443 → Internet
```

Loopback is DIRECT; `xray.exe` is routed to Corporate Proxy; the selected app
is routed to OneTunnel-Xray; Default is DIRECT. Routing `xray.exe` back to its
own SOCKS listener is a loop and is prohibited. Proxifier remains external and
is never edited or controlled by OneTunnel.

## Future system capture evaluation

Wintun + tun2socks, snapshots, route planning, DNS policy, watchdog, and
rollback remain prototype/research material outside this milestone. They are
not active implementation. M3 — Native Windows Capture Evaluation must
evaluate their safety and scope before any Windows-wide capture is built.
