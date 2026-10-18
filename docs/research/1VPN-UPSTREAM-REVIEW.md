# 1VPN upstream interoperability review — 2026-10-17

## Source repositories (read-only)

- Android: https://github.com/1vpn/android-app (GPL-3.0, fork of v2rayNG).
- Browser extension: https://github.com/1vpn/browser-extension (GPL-3.0).
- Android contract: `V2rayNG/app/src/1vpn/java/com/one/vpnapp/api/ApiService.kt`.
- Android models: `model/ApiResponse.kt`, `model/Location.kt`.
- Android runtime template: `V2rayNG/app/src/1vpn/assets/xray_config_template.json`.

## Source facts

Android uses VLESS over TCP 443 with Reality, `xtls-rprx-vision`, Chrome fingerprint and `spiderX: ""`. The Android template also contains a SOCKS inbound with UDP enabled, a separate HTTP inbound and geosite routing, **which OneTunnel intentionally does not reproduce** because the desktop V1 contract remains a localhost SOCKS-only client without UDP or China-specific policy.

The Android API interface defines:
- `POST login/?type=xray` with `email`, `password`, optional `token` (second-factor field).
- `POST signup/?type=xray`.
- `POST get_user_data_api/?type=xray` with authorization headers.
- `POST refresh_token/`.

The Android `UserData` fields are `username`, `email`, `isPremium`, `sessionAuthToken`, `uuid`, `publicKey`, `shortId` and `locations`.
Locations contain `country`, `countryCode`, nullable `city`, `cityCode`, nullable `servers` and `isPremium`. Servers contain `host` and nullable `realityServerName`. Android's `MainActivity.kt` uses the header `Authorization: Token <sessionAuthToken>`.

The browser extension uses a different proxy integration and should not be assumed to provide native Windows Xray endpoints. Its public release notes show that free server locations and login behaviour change; avoid hardcoding an unversioned catalogue.

## Implementation added

- `provider.rs` independent Rust deserialization for the observed API models, authorization check for premium locations and conversion to typed `RealityServer`.
- `config.rs` now uses the observed `spiderX: ""` value.
- Unit tests check premium authorization, required SNI, and absence of sensitive account fields from serialized public summary.

## Not proven

This is **contract compatibility**, not authenticated 1VPN API integration. We have not observed a successful network request from this client, tested a real premium token, refreshed tokens, validated current live locations or proved egress. Do not expose token or raw UserData over IPC; a Windows protected credential store and HTTPS API client still need to be implemented.

The GPL-3.0 upstream implementations are not copied. The provider model is a new implementation of observable interoperability data fields. Review upstream terms and dependencies before distribution.
