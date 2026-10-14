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

## F0.2 recovery attempt (2026-10-13)

Baseline at start: HEAD `3843af87ae43ef898b5be751129037543c1ddb06`; the working tree was clean. The earlier F0.1 report is preserved above.

### Connectivity

- DNS queries for `static.rust-lang.org`, `crates.io`, and `download.visualstudio.microsoft.com` returned records. The resolved addresses were environment-local and did not establish direct internet reachability.
- Direct HTTPS to the Rust channel checksum timed out (`curl` 28). The crates.io homepage returned HTTP 403; the Cargo sparse-index endpoint `https://index.crates.io/config.json` returned HTTP 200 through the system proxy.
- The user-level Windows proxy is enabled and WinHTTP reports a proxy. `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and `CARGO_HTTP_PROXY` are not set in the inherited environment. Proxy values and any credentials were not printed. No credentials were present in the explicit proxy value used for the session tests.
- Through the configured proxy, the Rust checksum endpoint returned HTTP 200 once and the Microsoft `aka.ms` Build Tools URL redirected (301). Later proxy checks failed with curl 5, `Could not resolve proxy`. This points to intermittent proxy-host DNS/connectivity in this session; it does not establish that corporate policy itself blocks the product domains. No proxy, DNS, TLS, or Proxifier settings were changed.
- Rustup's first proxied synchronization reached stable channel version `1.99.0` (2026-10-01) and began downloading six components. Subsequent checks found no active toolchain. A later attempt failed resolving the configured proxy (socket error 11001). Earlier direct attempts recorded errors 11002 and 10049. Rustup/Cargo mirror settings and TLS validation were not altered.
- The official Build Tools bootstrapper redirected successfully through the proxy, but its download timed out after 120 seconds at 2,143,984 of 4,478,032 bytes. It was not executed. The current Windows token is not elevated; no UAC or self-elevation was attempted. For completion, authorize reliable access to `static.rust-lang.org`, `index.crates.io`, `static.crates.io`, `aka.ms`, and `download.visualstudio.microsoft.com` through the approved proxy, then install Build Tools with **Desktop development with C++**, x64/x86 MSVC tools, and a Windows SDK from an elevated user session.

### Rust toolchain state

- Rustup is available at `C:\Users\Admin\.cargo\bin\rustup.exe` (version 1.29.1); the matching `rustc.exe` and `cargo.exe` are shims.
- The existing `stable-x86_64-pc-windows-msvc` installation was incomplete (`missing manifest`). A normal install and a `--force` repair both reported it as unchanged and could not read the compiler version.
- To preserve all files while allowing rustup to install into its normal toolchain path, the incomplete directory was moved, not deleted, to `C:\Users\Admin\.rustup\toolchains\stable-x86_64-pc-windows-msvc.incomplete-backup-20261013`. Keep this backup until a complete toolchain is verified. At final inspection the normal toolchain directory and compiler executable were absent; `rustc --version` and `cargo --version` therefore remain unavailable.

### Validation results

- `npm ci`: PASS; 73 packages installed, npm reported zero vulnerabilities. npm also warned that the local script policy has not approved esbuild's install script; this policy was not changed.
- `npm run build`: PASS (TypeScript check and Vite production build).
- `src-tauri/Cargo.lock`: absent; generation requires a working Cargo toolchain and registry access.
- `cargo check --locked`, `cargo test --locked`, `cargo fmt --all -- --check`, and `npm run tauri dev`: not certified. The earlier F0.1 attempts failed because Cargo reported `missing manifest in toolchain`; the native Tauri window, React in WebView, `get_app_info` IPC response, and clean shutdown remain unverified.
- MSVC, x64 linker, Windows SDK, MSBuild, and `vswhere.exe`: not detected. WebView2 remains installed at `154.0.4258.62`.
- Xray was not started. No Windows routes, DNS, firewall, adapters, or Proxifier configuration were changed.
- The first F0.2 attempt changed only this documentation (`README.md`, this file, and `ROADMAP.md`); it made no commit or push. The later recovery changes are recorded below.

## F0.2 continuation after network change (2026-10-13)

The network changed after the first F0.2 attempt. This continuation preserves the earlier error evidence above.

### Recovered toolchain and dependency resolution

- Direct DNS and HTTPS checks now work without a proxy. `static.rust-lang.org` and `index.crates.io/config.json` returned HTTP 200; the official Visual Studio `aka.ms` URL redirected successfully. No proxy environment variables or system proxy are configured now.
- Installed and selected `stable-x86_64-pc-windows-msvc`. Actual versions: `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`. `rustup show` confirms the x64 MSVC host and installed target.
- The prior incomplete toolchain remains preserved at `C:\Users\Admin\.rustup\toolchains\stable-x86_64-pc-windows-msvc.incomplete-backup-20261013`.
- Generated `src-tauri/Cargo.lock` with Cargo; it locks 406 packages. A real `tauri dev` warning showed Rust crate `tauri 2.12.3` did not match available `@tauri-apps/api 2.11.1`. Pinned the Rust crate to `=2.11.1` and updated the lockfile; the mismatch warning disappeared on the next launch attempt.

### Native prerequisites and certification

- `cargo fmt --manifest-path .\\src-tauri\\Cargo.toml --all -- --check`: PASS.
- `cargo check --manifest-path .\\src-tauri\\Cargo.toml --locked`: blocked at build scripts because `link.exe` is missing.
- `cargo test --manifest-path .\\src-tauri\\Cargo.toml --locked`: blocked by the same missing `link.exe`.
- `npm run tauri dev`: Vite started, then native compilation stopped at `link.exe`; no Windows window opened and IPC was not exercised. The launcher was interrupted after the failed native build, and port 5173 was confirmed free afterward.
- `cl.exe`, `link.exe`, `msbuild.exe`, `vswhere.exe`, and a Windows SDK are still absent. The Windows session is not elevated.
- Downloaded the official Build Tools 2022 bootstrapper from `https://aka.ms/vs/17/release/vs_BuildTools.exe` to `%TEMP%\\vs_BuildTools.exe`; download completed at 4,474,800 bytes and Authenticode reports a valid Microsoft Corporation signature. It has not been executed. From an elevated installer session, choose **Desktop development with C++**, MSVC x64/x86 tools, and a Windows SDK. See [Microsoft's workload component list](https://github.com/MicrosoftDocs/visualstudio-docs/blob/main/docs/install/includes/vs-2022/workload-component-id-vs-build-tools.md).
- `npm ci`: PASS (73 packages, zero audit findings); local npm policy warns esbuild's install script is not approved. `npm run build`: PASS. `package-lock.json` is unchanged and present.
- WebView2 remains installed at `154.0.4258.62`. No Xray process, Proxifier change, or Windows networking change occurred.

### Git state for this continuation

- HEAD at continuation start: `3843af87ae43ef898b5be751129037543c1ddb06`.
- Expected changes: this F0.2 record, `README.md`, `docs/ROADMAP.md`, `src-tauri/Cargo.toml`, and the new `src-tauri/Cargo.lock`.
- No commit or push was made.

## F0.2 final recovery verification (2026-10-14)

This continuation records the current machine state and supersedes only the earlier statements about whether MSVC and the Windows SDK are installed. Earlier connectivity and incomplete-toolchain failures above are retained as historical evidence.

### Connectivity and Rust

- DNS resolved all three requested domains (`static.rust-lang.org`, `crates.io`, `download.visualstudio.microsoft.com`). The answers were environment-local addresses; DNS success alone is not treated as internet reachability.
- `https://static.rust-lang.org/dist/channel-rust-stable.toml.sha256` returned HTTP 200 with valid TLS. Cargo successfully downloaded and compiled registry dependencies from crates.io; the separate homepage request to `https://crates.io` returned HTTP 403, so that homepage status does not represent Cargo registry availability.
- The official Microsoft Build Tools URL `https://aka.ms/vs/17/release/vs_BuildTools.exe` redirected to `download.visualstudio.microsoft.com` and returned HTTP 200 (4,474,800-byte bootstrapper metadata). Connectivity to both download services is `NETWORK_READY`; the bootstrapper was not executed because Build Tools are already installed.
- Proxy environment variables (`HTTP_PROXY`, `HTTPS_PROXY`, `CARGO_HTTP_PROXY`, `ALL_PROXY`, including lowercase variants) are unset; the current-user Windows proxy flag is disabled. No proxy values or credentials were printed, configured, or persisted. TLS verification remained enabled.
- The prior error `11002` is a Winsock name-resolution retry/DNS-server failure class; `10049` is `WSAEADDRNOTAVAIL` (address not valid in its context). Neither alone proves a product-domain block. On this verification run, successful DNS plus HTTPS 200 responses and Cargo downloads classify Rust, Cargo registry, and Microsoft downloads as `NETWORK_READY`. The earlier causes cannot be attributed more specifically from the retained codes alone.
- `rustup toolchain install stable-x86_64-pc-windows-msvc --profile default` confirmed the existing install unchanged; `rustup default stable-x86_64-pc-windows-msvc` succeeded. `rustup show` reports active/default x86_64 MSVC stable. Versions: `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`. The incomplete-backup toolchain remains preserved; no Rustup directories were deleted.

### Microsoft native tools

- Visual Studio 2022 Build Tools are installed at `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools` (17.14.42). MSVC x64 compiler and linker are present (toolset 14.44.35207; compiler/linker 19.44.35229); MSBuild is present.
- The tools are not on a regular PowerShell `PATH`, but `VsDevCmd.bat -arch=x64 -host_arch=x64` configures them correctly. `where cl`, `where link`, `where msbuild`, and `cl /Bv` succeeded in that developer environment. They must not be classified as absent based only on ordinary PowerShell.
- Windows SDK 10.0.26100.0 is installed and its `um\\Windows.h` header is present. No elevation or installer run was needed.

### Certification results

- `npm ci`: PASS (72 packages installed; npm warned that esbuild's install script is not approved by local policy; no policy change was made).
- `npm run build`: PASS (TypeScript and Vite production build).
- `cargo fmt --manifest-path .\\src-tauri\\Cargo.toml --all -- --check`: PASS. `cargo metadata --locked`: PASS; `src-tauri/Cargo.lock` is present and locked dependency resolution succeeds. `package-lock.json` is present and unchanged.
- The first locked native build exposed mismatched Tauri ecosystem versions: `tauri 2.11.1` was paired with `tauri-runtime 2.12.1` and `tauri-macros 2.7.2`. The lock was minimally aligned to `tauri-runtime`/`tauri-runtime-wry` 2.11.1 and `tauri-macros` 2.6.1, with compatible transitive versions; no framework or application architecture was changed. The remaining blocker is the project build resource omission: `src-tauri/icons/icon.ico` is absent, and `tauri-build` requires it to generate the Windows resource. After dependency alignment, `cargo check --locked` and `cargo test --locked` both stop at that build-script error before project tests can run. No icon was invented because the task explicitly excludes UI changes.
- `npm run tauri dev`: Vite started, but the Tauri native build stopped on the missing `src-tauri/icons/icon.ico`; no native window opened. React-in-WebView, disconnected controls, `get_app_info` IPC, console/IPC behavior, and clean window shutdown are unverified.
- No Proxifier rules, Windows routes, DNS configuration, firewall, adapters, or other Windows networking settings were changed. No commit or push was made.

### Current outcome

F0.2 is **PARTIAL**, not PASS. Connectivity and the native toolchain are recovered, but the documented native certification sequence is incomplete because the project lacks the required Windows icon resource. The requested baseline HEAD was `3fceb90741d9b16fb811ffd93c9a69349a2bfacc`; the actual HEAD at this session start and end is `3843af87ae43ef898b5be751129037543c1ddb06`. Existing local documentation edits, `.kilo/` files, `src-tauri/Cargo.toml` edits, and the untracked lockfile were preserved; no unrelated untracked files were removed.
