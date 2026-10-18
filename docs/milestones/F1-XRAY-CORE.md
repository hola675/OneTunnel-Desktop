# F1 — Xray SOCKS5 Core

## Scope
Implement the Windows Rust controller for the previously certified Xray 26.9.9 runtime, without Wintun, tun2socks, wstunnel, Windows routing changes, or Proxifier automation.

## Required contract
- Bind SOCKS5 to 127.0.0.1:10808 by default; fail explicitly on port conflict.
- Generate VLESS + Reality + Vision configuration using validated 1VPN Free server metadata. Never commit credentials or generated live configs.
- Spawn and supervise only the owned xray.exe process. Ensure shutdown and crash cleanup, and prevent concurrent sessions.
- Verify the local SOCKS listener and HTTPS through the tunnel before reporting CONNECTED. Differentiate local readiness from verified VPN egress.
- Provide connect, disconnect, get_status commands to Tauri with typed states and redacted diagnostics.
- Preserve existing get_app_info IPC and F0 tests.
- Keep Proxifier external; corporate proxy routing remains configured by the user.

## Gates
1. cargo fmt --all -- --check
2. cargo check --locked
3. cargo test --locked
4. npm run build
5. Windows native smoke: connect, verify egress, disconnect, port conflict, Xray crash, repeat connect/disconnect.
6. Record logs without tokens, UUIDs or private keys.

## Later stages
F2: Desktop UI and tray.
F3: 1VPN Premium login, 2FA, refresh and protected credentials.
F4: real server benchmarking and automatic selection.
F5: Xray traffic counters and diagnostics.
F6: NSIS installer with verified bundled Xray.
V1: Windows Free/Premium × direct/corporate acceptance matrix.

This is a development specification, not a claim that the engine is implemented or certified.
