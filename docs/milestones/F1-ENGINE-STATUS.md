# F1 — Xray SOCKS engine: work in progress

## Delivered in this branch
- A pure Rust runtime preflight module, with missing-resource and path-safety unit tests.
- Tauri `get_xray_preflight` IPC resolving the application resource directory.
- Read-only runtime status in React. Connect remains disabled by design.
- No Proxifier changes, external downloads, elevation, Windows network changes, or arbitrary shell execution.

## Not delivered — do not report as PASS
- A bundled, SHA-256 verified `xray.exe` 26.9.9 with verified upstream provenance.
- Actual Xray lifecycle orchestration: spawn, PID tracking, crash recovery, shutdown and Windows Job Object.
- Validated Xray SOCKS5/Reality configuration and real Free/ Premium credentials.
- SOCKS handshake, HTTPS through the tunnel, egress validation and corporate-proxy acceptance.
- Free/Premium auth, smart server choice, statistics, installer or final Windows certification.

## Native gates before enabling Connect
1. Verify official Xray runtime SHA-256 and version against the pinned manifest. Bundle at expected resource path; fail closed otherwise.
2. `cargo fmt --all -- --check`, `cargo check --locked`, `cargo test --locked`, `npm run build`.
3. Windows native `tauri dev` smoke test: initial disconnected state and truthful preflight.
4. Implement exclusive process ownership and launch using generated configs with ephemeral credentials, loopback-only SOCKS, deterministic cleanup and port collision errors.
5. Verify SOCKS5 and HTTPS egress against direct egress on a Windows machine before presenting CONNECTED.
6. Test the preexisting corporate path using Proxifier external rules without changing Proxifier.
7. Close F1 only when logs and test evidence are attached to the milestone.

## Constraints
- Xray is the only tunnel engine. No Wintun, tun2socks, relay, wstunnel or system VPN.
- SOCKS5 default: `127.0.0.1:10808`, not currently active.
- Proxifier remains independently configured and externally operated.
- Do not store, log or commit Premium tokens or Reality client secrets.
- Nothing in this branch proves a working VPN connection.
