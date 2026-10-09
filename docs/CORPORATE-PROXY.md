# Corporate proxy compatibility

Proxifier is optional, external, user-managed, and not bundled. OneTunnel does
not edit its profile, create or delete rules, start or stop it, or store proxy
credentials. M2-R detects a running `Proxifier.exe` and reports
`PROXIFIER_DETECTED`; detection does not certify that a rule is active.

## M2-R manual policy

Configure these rules manually in Proxifier, in this order:

| Order | Application / destination | Action |
| --- | --- | --- |
| 1 | Localhost and loopback (`127.0.0.1`, `::1`, `localhost`) | DIRECT |
| 2 | `xray.exe` | Corporate Proxy |
| 3 | Selected application (the M2-R.1 test uses `curl.exe`) | `OneTunnel-Xray` SOCKS5 at `127.0.0.1:10808` |
| 4 | Default | DIRECT |

The M2-R.1 test defaults to SOCKS port 10808 and fails if it is occupied. It
prints the endpoint before waiting for manual setup. A different port must be
passed explicitly and registered exactly. Never route `xray.exe` to
`OneTunnel-Xray`, which would send Xray back into its own SOCKS inbound.
Localhost must remain DIRECT.

Proxifier processes rules from top to bottom and provides application, target,
and port matching with Direct, Proxy, Chain, and Block actions. See the
[official rules guide](https://proxifier.com/docs/win-v4/rules.html). Keep the
default action DIRECT for this selective-routing milestone.

## Evidence and limits

The M2-R.1 script reads Verbose File Log during the probe interval to verify
the Xray corporate route, selected app rule, and an unselected DIRECT action.
The selected `curl.exe` receives no proxy argument or proxy environment
variables. The unselected control is a different executable. The script
records only hashes of egress IPs and never prints or copies log lines.
Proxifier logging is configured manually; see the
[official logging guide](https://www.proxifier.com/docs/win-v4/logging.html).

OneTunnel never requests, saves, or prints corporate usernames/passwords.
Avoid enabling verbose logs that expose secrets. Do not pass profile paths or
credentials to the script.

## Scope

M2-R has no Wintun adapter, tun2socks process, system route/DNS/IPv6/firewall
mutation, or administrator requirement. Native Windows packet capture using
Wintun/tun2socks is deferred to **M3 — Native Windows Capture Evaluation**.
