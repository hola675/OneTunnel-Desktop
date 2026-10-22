# OneTunnel Desktop

OneTunnel is a Windows desktop client for **1VPN VLESS + Reality** using a local **SOCKS5** listener for selected applications. It does **not** capture or redirect Windows traffic by itself. Proxifier is external and user-managed.

## Implementation status (development)

- Tauri 2 + React/TypeScript UI and Rust IPC.
- Xray runtime preflight: local executable, pinned SHA-256 via environment, and version validation.
- Authenticated Free/Premium 1VPN login with optional 2FA, authorized location listing, and server selection.
- Single Xray process supervisor, local `127.0.0.1:10808` SOCKS listener, Reality configuration, and Connect / Disconnect controls.
- After startup, an HTTPS test must complete **through SOCKS5**, returning an external IP before the UI reports the tunnel as verified.
- Read-only WFP detection and routing **simulation**, not operational Windows capture.
- No packaged Xray binary yet, no anonymous Free catalog, no automatic server selection, no Premium refresh, no traffic statistics, and no crash-safe Windows Job Object.

**These features have not yet passed real Windows/1VPN connectivity certification.** Source and unit tests are not evidence of a working VPN service. See [F2 acceptance checklist](docs/milestones/F2-VERIFIED-CONNECTION.md).

## Requirements for developers

- Windows 10/11 x64, Node 22 or supported Node 20, npm.
- Rust stable (`x86_64-pc-windows-msvc`), MSVC Build Tools / Windows SDK, WebView2.
- Verified Xray executable compatible with the Reality parameters returned by your 1VPN account.
- `curl.exe` included with modern Windows releases, used for a bounded HTTPS-over-SOCKS test.
- The HTTPS verification calls `api.ipify.org` and discloses the tunnel's exit IP to that service.

## Build and run (Windows PowerShell)

```powershell
npm ci
npm run build
cargo check --manifest-path .\\src-tauri\\Cargo.toml --locked
cargo test --manifest-path .\\src-tauri\\Cargo.toml --locked
$env:ONETUNNEL_XRAY_PATH = 'C:\\trusted\\xray.exe'
$env:ONETUNNEL_XRAY_SHA256 = '<independently validated 64-character SHA-256>'
npm run tauri dev
```

Do not invent or copy a hash from an untrusted binary as proof of its origin. Xray must pass the hash check before it can start.

After signing in, select a server from the authorized 1VPN list and click **Conectar**. A successful SOCKS handshake is not enough: a valid IP from an HTTPS request through the SOCKS proxy is required. If Xray, the account, API, server or HTTPS check fails, the UI must not show a verified tunnel.

If you configure Proxifier separately, exclude the OneTunnel/Xray processes from their own proxied-app rules to avoid loops.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Project scope](docs/PROJECT-SCOPE.md)
- [Roadmap](docs/ROADMAP.md)
- [Development](docs/DEVELOPMENT.md)
- [F2 implementation and live-test matrix](docs/milestones/F2-VERIFIED-CONNECTION.md)

No credentials or raw Reality secrets belong in Git, runtime logs or support screenshots.
