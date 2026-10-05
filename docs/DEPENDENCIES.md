# Dependencies and runtimes

M0.2 fija los cuatro runtimes para Windows x86_64. Los archivos comprimidos de
`runtime/downloads/` no se versionan y los ejecutables de `runtime/bin/` se
ignoran en Git. La reproducción se realiza con `fetch-runtime.ps1`; las
licencias y notices extraídos sí se conservan bajo `runtime/licenses/`.

| Componente | Versión/tag | Artefacto | Arquitectura | Licencia | Archive SHA256 | Runtime SHA256 | Estado |
|---|---|---|---|---|---|---|---|
| wstunnel | 11.0.0 / `v11.0.0` | `wstunnel_11.0.0_windows_amd64.tar.gz` | Windows AMD64 | BSD-3-Clause | `024323c9c2dd1ed1c6f38d417d9b2776e2f0083bba507e80d4a8124c8451b164` | `2bc2e95072f7e0a3335dc00bb8185a419c8d1e9431f6e21cab67dde860ed13ae` | PASS |
| Xray-core | 26.9.9 / `v26.9.9` | `Xray-windows-64.zip` | Windows AMD64 | MPL-2.0 | `244deaba2098c2964e49bba90df3707777e5f5f428a82d2f29604015f24beec2` | `0d0fc0ea2b05641acb78c01fc36ad694e7b029861b2d5eb93da0e3e9fda9a98f` | PINNED_COMPATIBLE; pre-release upstream |
| tun2socks | 2.7.0 / `v2.7.0` | `tun2socks-windows-amd64.zip` | Windows AMD64 genérico | MIT | `c5d46e9452f6c9cc7c15ab9158d6d6a0169ceecd6bca019ce476b49337d2be43` | `076b3c3d6a372bae3f49f2b415a4105f70c30a3ed3caaed7979390e649892559` | PASS |
| Wintun | 0.14.1 | `wintun-0.14.1.zip` | `bin/amd64/wintun.dll` | upstream distribution license | `07c256185d6ee3652e09fa55c0b673e2624b565e02c4b9091c79ca7d2f24ef51` | `e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce` | PASS |

## Política

- Solo se aceptan URLs HTTPS oficiales persistentes del upstream.
- Nunca se persisten `latest`, `main`, `master`, `nightly` o `dev`.
- No se usa `tun2socks-windows-amd64-v3.zip`; la baseline es AMD64 genérica.
- Xray se clasifica explícitamente como `pre-release`; solo queda fijado porque
  `v26.9.9` pasó los probes de versión, ayuda y configuración VLESS/Reality.
- No se compilan runtimes desde source en M0.2.
- `runtime/bin/` no se distribuye desde Git; se materializa mediante fetch y se
  valida antes de cualquier uso futuro.
- La distribución empaquetada futura prevé wstunnel, Xray-core y tun2socks con
  sus notices. Wintun se distribuirá únicamente junto con la aplicación que lo
  usa mediante su API permitida y tras confirmar que se cumplen las condiciones
  de la licencia de sus binarios precompilados.
