# OneTunnel Desktop

OneTunnel Desktop is a Windows desktop client foundation for a future 1VPN Free and Premium client. The planned connection engine is Xray-core using VLESS + Reality + XTLS Vision over TCP 443, exposed to selected applications through a configurable local SOCKS5 endpoint.

## Current state (F0)

- Tauri 2 desktop shell with a React, TypeScript, and Vite frontend.
- Initial disconnected screen; Connect stays disabled.
- A minimal Rust `get_app_info` command reports the app name, version, platform, and uninitialized core state.
- No VPN connection, account integration, Xray process, SOCKS listener, or Windows network changes are implemented.
- The native Windows build is not certified yet: the Rust toolchain download was blocked by DNS, and MSVC Build Tools are absent.

The SOCKS address shown in the UI (`127.0.0.1:10808`) is the planned default only; no listener is active in F0. Rust will own connection state. Proxifier remains an external, user-managed application and OneTunnel will not alter its configuration.

## Requirements

- Windows 10/11 x64
- Node.js 20.19+ or 22.12+ and npm
- Rust stable with the `x86_64-pc-windows-msvc` toolchain
- Microsoft C++ Build Tools with **Desktop development with C++** and a Windows SDK
- Microsoft Edge WebView2 Runtime (normally included on supported Windows 10/11 versions)

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for setup and environment audit notes.

## Run and build

```powershell
npm install
npm run build
npm run tauri dev
```

Native build prerequisites are required for `tauri dev` and `cargo check`.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Project scope](docs/PROJECT-SCOPE.md)
- [Roadmap](docs/ROADMAP.md)
- [Development](docs/DEVELOPMENT.md)
