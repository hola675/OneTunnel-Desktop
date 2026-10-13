# Development

## Windows prerequisites

- Windows 10/11 x64
- Node.js 20.19+ or 22.12+ with npm
- Rust stable, MSVC host target `x86_64-pc-windows-msvc`
- Microsoft C++ Build Tools with **Desktop development with C++** and a Windows SDK
- Microsoft Edge WebView2 Runtime
- PowerShell 7 is recommended for the documented commands

Check versions:

```powershell
git --version
node --version
npm --version
rustc --version
cargo --version
rustup show
```

## Commands

```powershell
npm install
npm run dev
npm run build
npm run tauri dev
cargo check --manifest-path .\src-tauri\Cargo.toml
```

`npm run build` validates the web frontend. The Tauri and Cargo commands also need the Rust MSVC toolchain, Microsoft C++ Build Tools, and Windows SDK.

## Troubleshooting

- If `rustc` or `cargo` is not found, install Rust through rustup with the MSVC toolchain, then restart the terminal.
- If native compilation cannot find `link.exe` or Windows headers/libraries, install the C++ Build Tools workload and Windows SDK.
- If the window cannot initialize WebView2, repair/install the Evergreen WebView2 Runtime.
- Keep `package-lock.json` and `src-tauri/Cargo.lock` under version control for reproducible dependency resolution.

## Initial environment audit (2026-10-13)

Observed in the development environment:

- Windows 10 Pro 10.0.19045, 64-bit
- Git 2.54.0.windows.1
- Node.js v24.21.0; npm 11.19.0
- PowerShell 7.6.2
- Rustup metadata exists under `C:\Users\Admin\.rustup`, but the `stable-x86_64-pc-windows-msvc` toolchain directory is incomplete. Rustup reports `missing manifest in toolchain` and cannot report a compiler version.
- The official Rustup installer was downloaded from `static.rust-lang.org`; its SHA-256 matched the checksum at the same official distribution endpoint. Rustup could not sync the MSVC stable channel: the first attempt failed DNS resolution while fetching `channel-rust-stable.toml.sha256` (socket error 11002), and a retry failed to connect (socket error 10049). TLS validation was not changed.
- `cl.exe`, `link.exe`, `msbuild.exe`, `vswhere.exe`, and the Windows SDK include directory were not found. The `C:\Program Files (x86)\Microsoft Visual Studio\2022` directory exists but contains no detected installation. The current Windows token is not elevated (`IsAdministrator=False`). The official Build Tools bootstrapper download from `aka.ms/vs/17/release/vs_BuildTools.exe` stalled and was stopped before execution; no installation or elevation was attempted.
- WebView2 is registered under `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}` as version `154.0.4258.62`.
- `npm ci` completed and `npm run build` passed. npm reported that the local policy has not approved esbuild's install script; no approval setting was changed.
- `cargo check --manifest-path .\\src-tauri\\Cargo.toml --locked`, `cargo test --manifest-path .\\src-tauri\\Cargo.toml --locked`, `cargo fmt --manifest-path .\\src-tauri\\Cargo.toml --all -- --check`, and `npm run tauri dev` could not run because the toolchain lacks its manifest. No Cargo.lock was generated.
- Tauri did not open a native window; React-in-WebView, IPC `get_app_info`, window close behavior, and absence of IPC errors remain unverified. The Connect button remains disabled in the frontend build. No Xray process or Windows networking operation was started.
- Working tree baseline before F0.1 work was `3fceb90741d9b16fb811ffd93c9a69349a2bfacc`; the only existing untracked files were `.serena/project.local.yml` and `.serena/project.yml`. No commit or push was made for F0.1.

Native certification remains partial until Rustup can reach its official distribution endpoint and Microsoft C++ Build Tools with an x64 Windows SDK are installed. No administrative elevation, Proxifier changes, or network configuration changes were attempted.
