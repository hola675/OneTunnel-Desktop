# Security model

Estas reglas son obligatorias desde M0:

- No almacenar contraseñas en texto plano.
- No incluir credenciales Premium en archivos de configuración.
- No incluir tokens de autenticación en logs.
- No incluir `UUID`, `publicKey` o `shortId` en bundles de diagnóstico salvo
  sanitización explícita.
- Verificar SHA256 de cada runtime antes de ejecutarlo.
- Enlazar listeners internos únicamente en `127.0.0.1`.
- El relay nunca debe ser un proxy sin restricciones.
- Redactar de logs: `password`, `Authorization`, `sessionAuthToken`, UUID
  cuando proceda y credenciales de upgrade.

El token persistente futuro se protegerá mediante Windows Credential Manager o
DPAPI. Las credenciales solo vivirán en memoria durante el uso y se eliminarán
de estructuras temporales al terminar. No se realizarán llamadas autenticadas
durante la auditoría.
