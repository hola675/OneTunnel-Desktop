# OneTunnel roadmap (2026-10-22)

| Phase | Implementation | Certification |
| --- | --- | --- |
| F0 | Tauri 2, React/Vite, Rust IPC, native Windows shell | Foundation certified historically |
| F1 | Xray runtime discovery, SHA-256, Reality config validation, supervised process and local SOCKS handshake | Unit tests; real Windows regression still required |
| F2 | Authenticated Free/Premium authorized-server picker and gated SOCKS→HTTPS verified connection | **Awaiting live Windows/1VPN certification** |
| F3 | 1VPN login, optional 2FA and in-memory session; login is independent of the VPN connection | Real Free/Premium authentication, refresh and credential-store work remains |
| F4 | Desktop Connect/Disconnect and verified status; diagnostic controls | UI build in CI; native E2E pending |
| F5 | Smart selection based on measured SOCKS/HTTPS performance | Not implemented |
| F6 | Live traffic statistics and historical meters | Not implemented |
| F7 | Package trusted Xray version, manifest, signing and NSIS installer | Not certified |
| F8 | Security/lifecycle/crash and real network validation | Not certified |

The routing policy/WFP components are **simulation and read-only diagnostics only**; Proxifier remains external. No system-wide VPN claims.

See [F2 live test matrix](milestones/F2-VERIFIED-CONNECTION.md) for remaining certification criteria.
