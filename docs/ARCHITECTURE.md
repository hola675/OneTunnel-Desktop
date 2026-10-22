# OneTunnel architecture — implementation and boundaries

```text
React/TypeScript UI
  |  strict Tauri IPC: login, authorized locations, connect, disconnect, status
  v
Rust AuthState  --------> 1VPN HTTPS login / 2FA (control plane)
  | authenticated server selection only
  v
Rust Engine (single-owner process + serialized connect/stop)
  | SHA-256 runtime gate + transient validated Xray JSON
  v
xray.exe (child process) — VLESS + Reality + XTLS Vision :443
  ^ 
  | SOCKS5 listener on 127.0.0.1:10808 only
  |
External Proxifier / apps configured by the user
```

## State contract

- `EnginePhase::Starting`: Xray process created, verification pending.
- `EnginePhase::Running` + `verified=false`: process running but tunnel **not** proven.
- `EnginePhase::Running` + `verified=true`: SOCKS handshake and HTTPS exit probe returned a valid IP. Not continual monitoring.
- `EnginePhase::Error`: process failed or lost the previously verified local listener; never advertise a verified tunnel.
- `EnginePhase::Disconnected`: owned Xray child stopped.

The egress verifier invokes Windows' System32 `curl.exe` with a short timeout, certificate checks and the explicit `socks5h://127.0.0.1:10808` proxy to `api.ipify.org`. Its response is *only* a tunneled public egress observation, not a Windows-wide privacy guarantee.

## Trust boundaries

- No executable paths, Xray JSON, UUIDs, Reality keys or auth tokens from the React UI.
- Only Rust-generated configs from currently authenticated, authorized 1VPN server indices.
- Xray hash pinned outside the executable with `ONETUNNEL_XRAY_SHA256` during development; a verified, signed release manifest remains outstanding.
- Passwords and tokens remain in Rust memory; no credential persistence yet.
- Rust blocks redirects from 1VPN authentication endpoints and controls their response sizes.
- Proxifier is external, and WFP policy is not enforced.
- Unexpected parent termination cleanup requires a Windows Job Object before release; current Drop handler is best effort only.

## Outstanding architecture milestones

Anonymous Free catalog, Premium refresh, permanent signed manifest and bundle, live diagnostics and periodic tunnel rechecks, automatic server benchmarking and traffic statistics. Production Windows/enterprise network testing is required before declaring the VPN complete.
