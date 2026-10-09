# Runtime provenance

Fecha de adquisición M0.2: **2026-10-05**. Todos los artefactos proceden de
URLs oficiales persistentes; no se usaron mirrors ni URLs firmadas temporales.

Runtime activo M1.2: Xray, tun2socks y Wintun, schemaVersion 3.
Los pins existentes se conservan sin cambios. Proxifier es externo y no pertenece
al runtime ni a esta matriz de procedencia.

## Xray-core

- Project: `XTLS/Xray-core`
- Official release: `https://github.com/XTLS/Xray-core/releases/tag/v26.9.9`
- Version/tag: `26.9.9` / `v26.9.9`
- Release date: 2026-09-08
- Classification: upstream pre-release; OneTunnel status `PINNED_COMPATIBLE`
- Candidate evaluated first: `v26.9.9`; fallback candidate retained in manifest:
  `v26.7.28`
- Artifact: `Xray-windows-64.zip`
- Artifact URL: `https://github.com/XTLS/Xray-core/releases/download/v26.9.9/Xray-windows-64.zip`
- Archive SHA256: `244deaba2098c2964e49bba90df3707777e5f5f428a82d2f29604015f24beec2`
- Official checksum source: GitHub release asset digest for `Xray-windows-64.zip`.
- Extracted runtime: `runtime/bin/xray.exe`
- Runtime SHA256: `0d0fc0ea2b05641acb78c01fc36ad694e7b029861b2d5eb93da0e3e9fda9a98f`
- License: MPL-2.0; `runtime/licenses/xray/LICENSE`
- Compatibility verification: `xray version`, `xray help`, and
  `xray run -test -config tests/fixtures/xray/reality-validation.json` passed.

## tun2socks

- Project: `xjasonlyu/tun2socks`
- Official release: `https://github.com/xjasonlyu/tun2socks/releases/tag/v2.7.0`
- Version/tag: `2.7.0` / `v2.7.0`
- Release date: 2026-07-12
- Classification: non-pre-release; generic AMD64 baseline
- Artifact: `tun2socks-windows-amd64.zip`
- Artifact URL: `https://github.com/xjasonlyu/tun2socks/releases/download/v2.7.0/tun2socks-windows-amd64.zip`
- Archive SHA256: `c5d46e9452f6c9cc7c15ab9158d6d6a0169ceecd6bca019ce476b49337d2be43`
- Official checksum source: GitHub release asset digest for `tun2socks-windows-amd64.zip`.
- Extracted runtime: `runtime/bin/tun2socks.exe`
- Runtime SHA256: `076b3c3d6a372bae3f49f2b415a4105f70c30a3ed3caaed7979390e649892559`
- License: MIT; official upstream raw license URL is recorded in the manifest
  and copied to `runtime/licenses/tun2socks/LICENSE`.
- Verification: `tun2socks --version`, `tun2socks --help`; no device started.

## Wintun

- Project/site: Wintun official distribution
- Official page: `https://www.wintun.net/`
- Version: `0.14.1`
- Artifact: `wintun-0.14.1.zip`
- Artifact URL: `https://www.wintun.net/builds/wintun-0.14.1.zip`
- Archive SHA256: `07c256185d6ee3652e09fa55c0b673e2624b565e02c4b9091c79ca7d2f24ef51`
- Official checksum source: SHA256 supplied in the M0.2 pin specification,
  verified against the official Wintun download archive.
- Extracted runtime: `runtime/bin/wintun.dll` from `bin/amd64/wintun.dll`
- Runtime SHA256: `e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce`
- License: signed DLL distribution license; `runtime/licenses/wintun/LICENSE.txt`
- Verification: archive hash and DLL hash; DLL was not loaded and no adapter was
  created.
