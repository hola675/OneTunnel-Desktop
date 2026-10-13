# Architecture

## F0 boundary

```text
React UI
   ↓ Tauri IPC (`get_app_info` only)
Rust Core (connection state source of truth; not initialized in F0)
   ↓ future process management
xray.exe (not included in F0)
   ↓ VLESS + Reality + XTLS Vision (future)
1VPN
```

The frontend renders state and invokes narrowly scoped native commands. It does not start processes or generate Xray configuration. Rust owns future application state, process lifecycle, configuration validation, health, and session management.

## Planned responsibilities

### Desktop UI

Connect/disconnect, connection status, Free/Premium account, server selection (including automatic selection), traffic statistics, settings, and diagnostics.

### Rust Core

Application state, Xray process lifecycle, configuration validation, connection health, and session management. Rust remains the source of truth for connection state.

### 1VPN Provider

Free server catalog, Premium login and 2FA, session token, account information, locations, and server configuration.

### Xray Engine

SOCKS5 listener, VLESS + Reality transport, server connection, traffic statistics, and connection verification.

## Traffic path (future)

Selected applications → externally configured Proxifier → local SOCKS5 (planned default `127.0.0.1:10808`) → `xray.exe` → 1VPN. Proxifier selection and corporate-proxy routing remain outside OneTunnel's control.
