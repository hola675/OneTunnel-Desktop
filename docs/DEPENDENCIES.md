# Dependencies

## Runtime Dependencies

Sólo hay tres componentes activos en runtime/manifest.json (schemaVersion 3):

| Componente | Versión/tag | Arquitectura | Licencia | Responsabilidad |
| --- | --- | --- | --- | --- |
| Xray-core | 26.9.9 / v26.9.9 | Windows AMD64 | MPL-2.0 | SOCKS → VLESS + Reality |
| tun2socks | 2.7.0 / v2.7.0 | Windows AMD64 genérico | MIT | IP → SOCKS5 |
| Wintun | 0.14.1 | Windows AMD64 | Upstream distribution license | Adaptador virtual |

Archive SHA256, runtime SHA256, artefactos y URLs oficiales son canónicos en
el manifiesto. [RUNTIME-PROVENANCE](RUNTIME-PROVENANCE.md) registra procedencia,
clasificación y fuente de comprobación. Xray se mantiene PINNED_COMPATIBLE,
con pre-release upstream declarado; su config Reality y Direct gate pasan.

## Política de adquisición

- Releases exactas y HTTPS upstream oficial; nunca latest/main/master/nightly/dev.
- Archive SHA256 y runtime SHA256 fijados y verificados antes de uso.
- fetch-runtime es idempotente; no recalcula ni actualiza los hashes fijados.
- tun2socks usa AMD64 genérico; no exige la variante AMD64-v3.
- bin/ y downloads/ son locales e ignorados. No son distribución del producto.
- Se conservan textos upstream intactos bajo runtime/licenses/.
- Wintun se empaquetará con la aplicación que usa su API, respetando las
  condiciones de distribución de sus binarios precompilados.

## Optional External Compatibility

Proxifier: opcional, externo, administrado por el usuario, no empaquetado ni
instalado automáticamente. No forma parte de Runtime Dependencies.
Véase [CORPORATE-PROXY](CORPORATE-PROXY.md).

## Application build stack futuro

Rust, Tauri, React y TypeScript. Todavía no se han inicializado; app/ y
src-tauri/ son placeholders. M1.2 no instala este stack ni añade UI.
