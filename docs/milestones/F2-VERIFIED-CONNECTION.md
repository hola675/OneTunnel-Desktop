# F2 — First authenticated 1VPN SOCKS5 connection (2026-10-22)

## Scope delivered in this branch

- `list_1vpn_locations`: returns only authorized, validated destinations from a currently authenticated 1VPN response, never private Reality credentials.
- `connect_1vpn`: securely resolves selected indices in Rust, repeats the Xray binary hash check, creates a fresh transient Xray configuration, runs `xray run -test -config`, and supervises a single Xray child process.
- Exposes a SOCKS5 listener only on `127.0.0.1:10808`. No DNS, Windows routing, WFP, or Proxifier settings are changed.
- Runs an HTTPS probe over `socks5h://127.0.0.1:10808` against `https://api.ipify.org`. TLS verification remains enabled. A valid returned egress IP is required before `verified=true`.
- `disconnect_1vpn`, `logout_1vpn` and abnormal child exits clean owned processes and temporary configuration files (best effort).
- UI supports selecting an available location/node and connecting/disconnecting. Only a successful HTTPS exit probe may show the verified state.

## Important limitations

This is **not a fully certified VPN release**. CI can compile and run unit tests but cannot prove live access to 1VPN or an enterprise proxy. The live authenticated Free and Premium matrix still requires real, authorized accounts on Windows. Anonymous Free catalog discovery is not implemented because there is no verified current API contract for it.

Other deferred features: automatic best-server selection, traffic counters, hardened Windows Job Object crash cleanup, credential persistence, background health checks, installer shipping a verified Xray binary, and policy enforcement. Policy/WFP views remain simulations.

The HTTPS check currently uses the `curl.exe` distributed with Windows 10/11, invoked by absolute System32 path, with `-q`, remote SOCKS DNS, certificate checking, a 12-second timeout and bounded response size. The check contacts the third-party `api.ipify.org` service, which observes the exit address. Do not interpret the probe as whole-system privacy protection.

## Windows validation procedure

Set these only after acquiring Xray from a trusted release and independently validating its expected hash:

```powershell
$env:ONETUNNEL_XRAY_PATH = 'C:\\path\\to\\xray.exe'
$env:ONETUNNEL_XRAY_SHA256 = '<independently verified SHA-256 from trusted release manifest>'
npm ci
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --locked
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri dev
```

1. Confirm Diagnostics reports matching SHA-256 and Xray version.
2. Authenticate to a real 1VPN account (Free or Premium) and select an authorized node.
3. Click Connect; verify the UI shows a live Xray PID, responsive SOCKS5, verified HTTPS, and an egress IP.
4. Configure your own Proxifier rule to redirect only chosen applications to the local SOCKS listener; exclude `xray.exe` and OneTunnel to avoid loops.
5. Check HTTPS from a chosen application, then Disconnect; verify the Xray PID closes and SOCKS is no longer available.
6. Repeat after wrong credentials, 2FA, occupied port, unreachable server, Xray crash, logout, repeated clicks, and a restart.
7. Repeat separately on unrestricted and corporate networks. Do not claim PASS for scenarios that were not executed.

## Acceptance gates not yet met

- No live 1VPN credentials or network proof available in GitHub CI.
- Xray needs an externally pinned hash and its executable is not yet included in the NSIS release.
- Free anonymous connection, Premium refresh and auto server selection remain unimplemented.
- Abrupt parent-process termination may still leave Xray alive; Windows Job Object supervision remains a priority before public release.
