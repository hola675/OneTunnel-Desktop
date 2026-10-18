# 1VPN Premium login — implementation notes

The public Android project at https://github.com/1vpn/android-app defines `POST https://1vpn.org/api/login/?type=xray` with JSON `email`, `password` and optional `token` for 2FA. Its API client lists `1vpn.org`, `1vpn.co`, `onevpn.com`, and `cloudlogcdn.com`.

## Delivered
- Native `login_1vpn` command passes credentials to Rust. React never receives raw `sessionAuthToken`.
- Native `get_account` returns only username, tier, and location count; `logout_1vpn` clears the in-memory account.
- Supports Android observed error code `1001` to request an additional verification code.
- HTTPS only; redirects are disabled; timeouts and an upper bound on response bodies.
- A definitive HTTP authentication response does not trigger retries against alternate hosts.
- UI exposes email, password, optional code, account status, and logout.

## Limitations
- **Not tested with an actual Premium account.** No production credentials were used.
- Session is memory-only; it is lost on app restart. Windows Credential Manager is not implemented yet.
- 2FA code entry is a second login request including the same password; no credential cache is written.
- API availability may vary in corporate networks; no Proxifier manipulation is performed.
- The API is inferred from upstream client code, not guaranteed by a stable documented public contract.
- Never put credentials or API tokens in CI artifacts, logs, or test fixtures.
- A successful login does **not** assert that the VPN is connected.

## Validation
Execute `npm run build`, `cargo fmt --all -- --check`, `cargo check --locked`, and `cargo test --locked` on Windows; then conduct user-driven login/2FA tests with valid personal credentials and confirm that logging out removes the native session.
