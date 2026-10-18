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

## F1 follow-up — engine lifecycle scaffold

- Added `src-tauri/src/engine.rs` with a single-owner, mutex-protected Xray child process.
- Reads state using `get_engine_status` IPC; checks unexpected child exit on status requests.
- Rejects duplicate owned processes and occupied local SOCKS port.
- Includes internal start/stop methods and best-effort kill/reap on Engine drop.
- Start/stop are **not exposed to the UI yet**: package integrity verification and secure Reality configuration generation must be completed first.
- `running` means the child was launched, NOT that SOCKS is listening or 1VPN tunnel succeeded.
- Must test Windows close/crash scenarios; in particular, Job Object supervision for abrupt parent termination is not implemented.

### Required next implementation gates
1. Verify exact SHA-256 of the pinned Xray binary before any launch.
2. Validate authorized 1VPN server parameters, generate transient Xray JSON, and avoid leaking sensitive config to logs.
3. Wire start/stop behind gated commands; prove SOCKS5 readiness and HTTPS egress before declaring CONNECTED.
4. Run `cargo fmt`, `cargo check --locked`, `cargo test --locked`, and Windows process lifecycle tests.
