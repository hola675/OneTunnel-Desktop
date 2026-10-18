# Architecture study: Proxifier-class transparent routing for OneTunnel

## Verified references
- Proxifier Windows v4: https://www.proxifier.com/docs/win-v4/install.html — uses a Windows Filtering Platform callout driver and requires administrator installation.
- Rules: https://www.proxifier.com/docs/win-v4/rules.html — top-down application, destination, port matching with proxy/chain, direct and block decisions; immutable local bypass recommended.
- Microsoft's ALE redirect: https://learn.microsoft.com/en-us/windows/win32/fwp/ale-layers and https://learn.microsoft.com/en-us/windows-hardware/drivers/network/using-bind-or-connect-redirection.

## Independent architecture
1. Policy engine (implemented): evaluate selected connection metadata against ordered rules. Actions: Tunnel, Direct, Block. Local loopback always bypasses. API exposes a **simulation only**.
2. Interception driver (NOT implemented): signed WFP callout driver at ALE_CONNECT_REDIRECT IPv4/IPv6 and independent host service. The kernel must not read Rust UI configuration directly; use a validated policy channel.
3. Loop prevention: exclude the OneTunnel process, loopback, system DNS resolver, local SOCKS listener and the upstream proxy control connection. Redirected connection records must be handled exactly according to WFP documentation.
4. Forwarding service (NOT implemented): listen for intercepted TCP flows, recover original destination, apply policy and forward via local SOCKS5 (Xray), direct socket, or reject. Support IPv4/IPv6, backpressure, cancellation, bounded buffers and cleanup.
5. Guardrails: no system-wide route changes, no automatic global interception or blocking, no deep packet inspection or TLS decryption, explicit administrator approval before driver installation, explicit user approval of scope and default action.
6. Diagnostics (NOT implemented): expose redacted connection metadata and aggregated byte counts without capturing packet payloads. Never record credentials, URLs containing tokens, request bodies or DNS queries by default.
7. Corporate mode: Proxifier can continue to handle the outbound xray.exe to corporate proxy. Transparent per-app capture must avoid redirecting Xray itself, otherwise redirection loops occur.

## Current implementation
`routing.rs` includes bounded, deterministic first-match policy evaluation and validation, with local loopback bypass. `preview_traffic_rule` is exposed over Tauri IPC to a UI form. **The result always includes `applied: false`**. No Windows traffic is intercepted or blocked.

## Release gates
WFP signing/driver installation, performance testing with multithreaded TCP, DNS leak tests, IPv6, loop prevention, compatibility with Proxifier and antivirus, Windows networking restart, fail-open/fail-closed policies, UAC/rollback installer, and user consent. Until these pass, all capture/block options stay disabled.

## Follow-up: guarded policy management

- `policy.rs` now maintains validated in-memory routing rules, a monotonic revision number, default direct routing, and an explicit `enforcementActive: false`.
- `save_traffic_policy` accepts replacements only when their expected revision matches, avoiding accidental overwrites.
- In the desktop UI, rules can be added, reviewed and cleared for simulation; closing the application loses the configuration by design until a secure persistent schema exists.
- The evaluator excludes the Xray executable and OneTunnel process from any requested block/redirect action, in addition to localhost, to prevent routing loops.
- **Nothing in this change installs a kernel driver or routes packets.** Real interception requires separately reviewed Windows signing, installation, consent, rollback and compatibility.

## WFP discovery and driver verification (next iteration)

- Windows already includes Windows Filtering Platform; the `BFE` (Base Filtering Engine) Windows service is checked read-only by `capture.rs` via `sc.exe query BFE`.
- `get_capture_capability` now returns `wfpEngineRunning` to the UI independently from `driverInstalled`, `interceptionActive`, `redirectSupported`, and `blockingSupported`. Only the WFP service readiness is measured; the others remain false until an audited real implementation is available.
- The helper `scripts/windows/Test-OneTunnelWfpDriver.ps1` is an **audit-only** utility by default. It refuses unsigned packages and verifies the provided, independently trusted `OneTunnelWfp.inf` SHA-256. It accepts `-Install` only with elevation and PowerShell confirmation. No driver binaries are included in this repository.
- Example for a future certified package, audit only: `pwsh -File scripts/windows/Test-OneTunnelWfpDriver.ps1 -InfPath C:\\approved\\OneTunnelWfp.inf -ExpectedInfSha256 <verified-hash>`
- The installation tool itself does not activate transparent interception. That requires implementing, signing, registering and testing a WFP callout and TCP forwarding service.

### Installation and readiness gate

Do not run `pnputil` against arbitrary INF files or rely on the WFP service being active as evidence that transparent routing is available. The service, driver lifecycle, redirect records, privileged APIs, rollback and interoperability with existing Proxifier still require an independently reviewed implementation.

References:
- https://learn.microsoft.com/en-us/windows/win32/fwp/ale-layers
- https://learn.microsoft.com/en-us/windows-hardware/drivers/network/using-bind-or-connect-redirection
- https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/pnputil-command-syntax
