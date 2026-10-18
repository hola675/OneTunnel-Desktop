# UI/UX review — operational dashboard (October 2026)

## Evidence available
This is a **heuristic review**, not an analysis of production telemetry. The repository exposes real account tier, a server location count from the active session, Xray integrity/version preflight, engine phase/PID, and SOCKS5 handshake readiness. No product analytics events, measured conversion funnels, traffic counters, speed tests, user research, real server pings, or HTTPS egress verification are available in this codebase.

## Audit of prior UI
1. Account details were duplicated and the summary incorrectly displayed "Not signed in" after login.
2. Local SOCKS showed a hardcoded "NOT LISTENING" badge rather than the returned `socksReady` signal.
3. All operations were displayed as a long unsorted list, without navigation.
4. No distinction between "process running", SOCKS handshake, and actual verified connection.
5. Form controls used inline styles and had no clear group hierarchy.
6. Missing configuration screen led to ambiguity about network settings and Proxifier ownership.
7. There was no single-place technical diagnosis showing runtime hash/version/PID and last-check time.

## Implemented
- Sidebar navigation: Inicio, Cuenta, Diagnóstico, Ajustes; single primary page title.
- Home hero makes unsupported Connect action visibly disabled.
- Account card is unified and switches dynamically between login/2FA and signed-in profile.
- Real overview tiles: plan, reported location count, SOCKS handshake status.
- Three readiness signals reflect native data, **not** a "VPN connected" assertion.
- Diagnostics separates Xray process, executable integrity, SOCKS handshake and HTTPS egress (not available).
- Settings expose only real UI controls: diagnostic refresh cadence and compact density. Missing integration controls are disabled/read-only, never fake-functional.
- Improved keyboard focus, error feedback, responsive breakpoints and reduced-motion preference.
- No external analytics SDK, no tracking, and no fabricated latency, byte counters or rankings.

## Next product analytics instruments (proposal, NOT implemented)
Local, opt-in aggregated UX instrumentation could count navigation success, sign-in failure categories (without email or credentials), readiness-check errors and time-to-connect from actual instrumentation. Never report invented statistics. Privacy review and explicit consent are required before external analytics.

## Acceptance checks
- TypeScript + Vite build on Windows CI.
- Visual QA in native desktop window at common breakpoints.
- Keyboard tab traversal, login, two-factor entry, logout, refresh, and settings interactions.
- Validate no false "Connected" or traffic claims when Xray process runs but HTTPS egress is unavailable.
