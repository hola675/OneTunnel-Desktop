# Dependencies and runtimes

Los runtimes externos se distribuirán con versión exacta y SHA256 verificado.
No se usará `latest`, descarga automática ni binario no registrado. En M0 no
se descargan ejecutables y todos los campos permanecen `null` en
`runtime/manifest.json`.

| Componente | Fuente declarada | Estado M0 |
|---|---|---|
| wstunnel | `erebe/wstunnel` | no fijado |
| xray | `XTLS/Xray-core` | no fijado |
| tun2socks | `xjasonlyu/tun2socks` | no fijado |
| wintun | `WireGuard/wintun` | no fijado |

La compatibilidad Windows x86_64, procedencia oficial, licencia y hash se
verificarán conjuntamente en M0.2. Los binarios se almacenarán solo bajo
`runtime/bin/` después de esa certificación.
