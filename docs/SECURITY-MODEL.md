# Security model

Políticas obligatorias V1:

- No plaintext 1VPN passwords; contraseñas sólo en memoria.
- No auth/session tokens en logs.
- No UUID Premium ni otros parámetros sensibles en diagnósticos.
- SOCKS interno únicamente en loopback.
- Verificación SHA256 de archivo comprimido y runtime antes de uso.
- Rollback ante fallo; cada mutación Windows tiene una inversa conocida.
- No silent DNS leak y no silent IPv6 leak.
- No corporate proxy password storage in V1.

El token persistente futuro debe protegerse con Windows Credential Manager o
DPAPI. Se redactan Authorization, sessionAuthToken, contraseñas y parámetros
de configuración personales. La PoC actual no realiza llamadas autenticadas
ni usa valores Premium.

Las constantes Free upstream son públicas y no son secretos personales.
El fixture conserva su procedencia; los diagnósticos del runner también
redactan esas constantes e IPs públicas. Los resúmenes de egress guardan
sólo SHA256. No se incluyen claves privadas ni emails personales en nuestros
archivos.

## TLS del probe

scripts/lib/HttpsProbe.ps1 aplica STRICT first.
Sólo exit 35 y CRYPT_E_REVOCATION_OFFLINE o 0x80092013 en stderr habilitan
un único retry con --ssl-revoke-best-effort. No se desactiva la validación
de cadena, hostname, expiración o handshake; no se usan flags insecure
ni --ssl-no-revoke. Otros errores conservan su fallo.
La evidencia certificada está en [M1](milestones/M1-XRAY-DIRECT.md).

## Captura y recuperación futuras

[WINDOWS-VPN](WINDOWS-VPN.md) define snapshot y restauración de rutas, DNS
e IPv6. El estado CONNECTED debe incluir comprobaciones de fuga y tráfico.
Estas protecciones de sistema todavía no están implementadas ni certificadas.

Los procesos se gestionan por objetos/PIDs propios, sin matar por nombre.
La recuperación no modifica recursos ajenos y debe cubrir desconexión,
fallo y recuperación de estado posterior a interrupción.

## Compatibilidad corporativa

Proxifier configuration remains external. OneTunnel no almacena credenciales,
edita perfiles, inyecta reglas ni arranca/para Proxifier en M1.2.
Sólo xray.exe necesita la regla externa de salida; loopback permanece DIRECT.
Véase [CORPORATE-PROXY](CORPORATE-PROXY.md).
