# F1 — Xray runtime preflight (incremental implementation)

This branch begins F1 without modifying the certified F0 baseline.

## Implemented

- Native Rust module `src-tauri/src/xray.rs` locates `xray.exe` relative to the installed application, a `runtime/` directory, a `resources/` directory, or `ONETUNNEL_XRAY_PATH` for development.
- Direct execution of `xray.exe version` (no shell) and a structured status returned via Tauri IPC `inspect_xray_runtime`.
- React displays executable readiness and the returned version, with a manual recheck.
- No Windows routing, DNS or Proxifier changes.

## Not yet implemented or certified

- Executable SHA-256 verification, pinned runtime packaging, VLESS/Reality config generation.
- Start/stop supervision, SOCKS5 listener, functional HTTPS egress proof.
- 1VPN Free catalog and Premium account flow, automatic selection, metrics, installer.
- No native Windows builds, installer tests, live 1VPN connections or corporate proxy tests have run from this environment.

## Local validation (Windows)

```powershell
npm ci
npm run build
cargo fmt --manifest-path .\src-tauri\Cargo.toml --all -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml --locked
cargo check --manifest-path .\src-tauri\Cargo.toml --locked
$env:ONETUNNEL_XRAY_PATH = "C:\path\to\verified\xray.exe"
npm run tauri dev
```

Do not enable Connect merely because `inspect_xray_runtime().ready` returns true: this only certifies that a local executable responded to a version query. Before launching any binary, verify the pinned expected SHA-256 and source. In V1, Proxifier remains external and user-managed.
