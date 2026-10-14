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

## F0.3 icon resource and native certification (2026-10-14)

### Icon resource

- Created a neutral, non-branded 1024×1024 SVG at `src-tauri/icons/placeholder.svg` and ran the official local CLI: `npm run tauri icon .\src-tauri\icons\placeholder.svg`.
- The generator succeeded and produced `src-tauri/icons/icon.ico` (11,152 bytes) plus Tauri PNGs including `32x32.png`, `64x64.png`, `128x128.png`, `128x128@2x.png`, and `icon.png`. The CLI also produced its standard platform icon outputs. No temporary generator files were found.
- The placeholder is technical only; replace it with approved OneTunnel artwork before public distribution.

### Build and test results

- `cargo check --manifest-path .\src-tauri\Cargo.toml --locked`: PASS.
- `cargo test --manifest-path .\src-tauri\Cargo.toml --locked`: PASS (0 unit tests and 0 doc tests are defined; all test targets built and exited successfully).
- `cargo fmt --manifest-path .\src-tauri\Cargo.toml --all -- --check`: PASS.
- `npm run build`: PASS.
- The locked Tauri graph needed a technically demonstrated compatibility alignment: with `tauri 2.11.1`, locked runtime 2.12.1 and newer macro/codegen crates failed compilation. Lockfile-only changes aligned `tauri-runtime` and `tauri-runtime-wry` to 2.11.1, and `tauri-macros` / `tauri-codegen` to 2.6.1. The subsequent locked check and test passed. `Cargo.toml` and `package-lock.json` were not changed for this alignment.

### Native window, React, IPC, and shutdown

- `npm run tauri dev` launched the real Windows executable `target\debug\onetunnel-desktop.exe`; Windows UI Automation identified a native top-level window named `OneTunnel Desktop` and its WebView content. This is the recorded native-window evidence; the Integrated Browser also displayed the frontend, but its screenshot is not represented as a native-window screenshot.
- The native window’s accessibility tree confirmed `DISCONNECTED`, the `Connect` button with `enabled=False`, and footer values returned by Rust: `OneTunnel`, version `0.1.0`, platform `windows`, core status `not_initialized`. This verifies that React rendered and `get_app_info` returned through Tauri IPC.
- No runtime/IPC error appeared in the Tauri dev output during this run. The application’s native window was closed, but the `onetunnel-desktop.exe` process and Vite listener remained active afterward. The managed dev process was then stopped to ensure no process or port-5173 listener remained; the runner reported that `beforeDevCommand` terminated non-zero during that forced cleanup. Therefore clean process shutdown is **not certified**.

### Reproducibility and repository state

- `src-tauri/Cargo.lock` and `package-lock.json` both exist. `package-lock.json` remains unchanged.
- The user-provided expected HEAD was `3843af87ae43ef898b5be751129037543c1ddb06`; the current repository was already at `5858f2cdc3aa5b52af73f95d1307992f4823208f` (`docs: document toolchain recovery and native build blocker`) before F0.3 edits. No reset or checkout was performed.
- No React component, layout, connection engine, service, or Windows networking configuration was changed. `.kilo/`, `.serena/`, and other pre-existing user files were preserved. No commit or push was made.

F0.3 is **PARTIAL**, not PASS: build, tests, formatting, React rendering, native window, disabled Connect state, and Rust IPC response are verified; clean termination of the development runner after closing the native window remains unverified. No F1 work was started.

## F0.4 lifecycle and shutdown certification (2026-10-14)

### Initial state and lifecycle inspection

- `git status --short` was empty before this task. The requested incoming HEAD was `5858f2cdc3aa5b52af73f95d1307992f4823208f`, but the actual repository was already at `8a2ab4dc72303a2073654a9dea679c205bca5b20` (`docs: document F0.3 native certification and provisional icon`, tracking `origin/main`). No reset, checkout, clean, commit, or push was performed.
- Lifecycle entry points: `src-tauri/src/main.rs` calls `onetunnel_desktop_lib::run()`; `src-tauri/src/lib.rs` builds one default Tauri window, registers `get_app_info`, and calls `.run(generate_context!())`. No close-prevention handler or custom process-exit code exists. React invokes `get_app_info` in `src/App.tsx` and renders the result in the footer. `package.json` exposes `tauri: tauri`; `tauri.conf.json` starts `npm run dev` as `beforeDevCommand`, with Vite at `localhost:5173`.
- No lifecycle/config/source changes were made. `Cargo.lock` and `package-lock.json` were preserved. The CLI is `@tauri-apps/cli 2.11.4`; Vite is 7.3.6.

### Test A — foreground terminal `tauri dev`

- Ran `npm run tauri dev` in a foreground PowerShell command under the VS 2022 x64 developer environment, not through `background_process`. The command remained active while the app was observed and returned `TAURI_DEV_EXIT=0` after the native window close request.
- Windows process inspection captured the Tauri CLI (`node .../@tauri-apps/cli/tauri.js dev`), its cargo-run children, `onetunnel-desktop.exe`, `beforeDevCommand`/npm/Vite processes, and the TCP listener owner. One complete captured chain included Tauri CLI PID 13196 (parent `cmd.exe` PID 12876), cargo PID 6844, nested cargo PID 2696, native app PID 17416 (parent 2696), and Vite PID 15972 (parent `cmd.exe` 2136); Vite owned port 5173. A repeated UI-verification run independently captured native PID 4952, CLI PID 5348, Vite PID 6936, and port 5173 owned by PID 6936.
- The native window was identified as `OneTunnel Desktop`. Windows UI Automation showed `DISCONNECTED`, `Connect` disabled, and the Rust response fields `OneTunnel`, `0.1.0`, `windows`, `not_initialized`; this confirms the rendered React screen and `get_app_info` IPC response.
- Closed the native top-level window with the Windows UI Automation `WindowPattern.Close` action (normal window close request; no process termination API). The native PID, Tauri CLI, Vite PID, and listener all disappeared naturally. The foreground CLI returned exit code 0 with no shutdown error. No OneTunnel-owned Vite listener remained on 5173.
- The shell/tool session had no native-desktop screenshot facility; the recorded native-window evidence is the Windows UI Automation window/control tree and process/listener data. No browser screenshot is claimed as a native-window screenshot.

### Test B — standalone Release executable

- `npm run tauri build -- --no-bundle`: PASS. Release executable: `src-tauri/target/release/onetunnel-desktop.exe`. No installer was generated. The build emitted only the existing `STATIC_VCRUNTIME` deprecation warning; no config change was made.
- Launched the executable directly three times. Runs 1–3 used distinct PIDs 11256, 12632, and 18304. Each rendered the native window, exposed `DISCONNECTED`, disabled `Connect`, and returned version `0.1.0` / platform `windows` via `get_app_info`.
- Each window was closed through the normal Windows UI Automation close action. Each corresponding process PID terminated within the wait window; there was no Vite listener in these standalone runs. Result: 3/3 natural app-process exits.

### Root cause and disposition

- Classification: **CASE C — automated process runner/termination**, not an application lifecycle defect. In F0.3, the managed background-process runner was explicitly stopped; that interrupted `beforeDevCommand` and reported a non-zero status. In the foreground terminal, closing the native window allowed Tauri CLI normal cleanup, CLI exit 0, Vite exit, and port release. The independent Release executable also exited normally in all three runs.
- This matches the failure mode described by upstream Tauri issue [#15930](https://github.com/tauri-apps/tauri/issues/15930): on Windows, forcibly terminating the CLI can bypass its cleanup path for `beforeDevCommand`. The normal foreground-close path in this project did not reproduce an orphan. The prior F0.3 non-zero result belongs to the managed runner stop, not OneTunnel code.
- No correction was justified. No Rust close handler, `std::process::exit`, dependency update, or runner workaround was added. Do not use forced runner termination as lifecycle certification evidence.

### Regression and final state

- `cargo check --manifest-path .\\src-tauri\\Cargo.toml --locked`: PASS.
- `cargo test --manifest-path .\\src-tauri\\Cargo.toml --locked`: PASS (all targets succeeded; 0 unit/doc tests are defined).
- `cargo fmt --manifest-path .\\src-tauri\\Cargo.toml --all -- --check`: PASS.
- `npm run build`: PASS.
- `git diff --check`: PASS after this documentation update.
- No changes to Rust or React source, `Cargo.toml`, `Cargo.lock`, `package.json`, or `package-lock.json` in F0.4. The only F0.4 files changed are this development record, `README.md`, and `docs/ROADMAP.md`.
- F0 is **PASS / certified**. The provisional icon remains marked for replacement before public distribution. No F1 implementation was started; the F0 gate is technically satisfied for a later, separately authorized F1 task.
