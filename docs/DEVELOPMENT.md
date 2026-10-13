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
- Rust, Cargo, and rustup commands were not available on PATH
- `cl.exe`, `link.exe`, and `msbuild.exe` were not found; no Visual Studio installation or Windows SDK include directory was detected
- WebView2 application files were present under `C:\Program Files (x86)\Microsoft\EdgeWebView\Application\154.0.4258.62`

Missing native build tools were documented without attempting administrative installation or system changes.
