# 1VPN API audit

Esta auditoría es estática y se basa únicamente en los snapshots locales. No
se realizaron llamadas autenticadas ni se incluyeron credenciales personales,
tokens o valores Premium.

## Hosts observados

El código de ambas referencias declara el siguiente orden de fallback:

1. `1vpn.org` (primario)
2. `1vpn.co`
3. `onevpn.com`
4. `cloudlogcdn.com`

Las peticiones se construyen como `https://HOST/api/<endpoint>/`. El cliente
intenta el primario y luego los backups cuando hay error de red o respuesta que
no puede utilizarse; respuestas HTTP 2xx–4xx se conservan para que la capa de
API procese el error del servidor.

## Endpoints observados

| Operación | Método | Ruta | Query | Headers/cuerpo observados |
|---|---|---|---|---|
| Login | POST | `/api/login/` | `type=xray` | `Content-Type: application/json`, `Client-Type: app`; `{email,password,token?}` |
| Signup | POST | `/api/signup/` | `type=xray` | `Content-Type: application/json`, `Client-Type: app`; `{email,password}` |
| User data Android | POST | `/api/get_user_data_api/` | `type=xray` | `Content-Type: application/json`, `Client-Type: app`, opcional `Authorization: Token <token>` |
| User data extension | POST | `/api/get_user_data_web/` | no observado | `Content-Type: application/json`, opcional `Authorization: Token <token>`; cuerpo `{}` |
| Refresh token | POST | `/api/refresh_token/` | no observado | `Content-Type: application/json`, `Authorization: Token <token>` |

La extensión no implementa un formulario API de login en el snapshot auditado:
su botón abre `https://<active-host>/login/`; el contrato API de login está
explícito en Android.

## Autenticación y 2FA

- El esquema de autorización observado es `Token <sessionAuthToken>`.
- Android envía `token` opcional en el payload de login cuando el servidor
  solicita 2FA.
- Una respuesta de error con `code == 1001` hace que Android muestre el campo
  de token 2FA y reintente login con ese valor.
- El snapshot no muestra un endpoint separado para 2FA.
- Los mensajes de la extensión distinguen token 2FA requerido o inválido, pero
  no contienen el flujo de autenticación.

## Modelo UserData observado

```text
username: String
email: String
isPremium: Boolean
sessionAuthToken: String
uuid: String
publicKey: String
shortId: String
locations: List<Location>
```

`Location` contiene `city`, `cityCode`, `country`, `countryCode`, `servers` y
`isPremium`. Cada `Server` contiene `host` y `realityServerName` opcional.

## Free vs Premium

- **Free**: Android selecciona ubicaciones locales limitadas y usa parámetros
  Free embebidos en su referencia. Los valores exactos se omiten
  intencionadamente de esta documentación y no son credenciales de usuario.
- **Premium**: Android usa los datos `uuid`, `publicKey`, `shortId` y
  `locations` recibidos en `UserData` cuando `isPremium == true`; las
  ubicaciones Premium pueden incluir múltiples servidores.
- La aplicación futura no debe asumir que una respuesta Premium es válida sin
  verificar sesión, esquema y pertenencia de ubicación.

## Persistencia y límites para OneTunnel

La extensión almacena campos en `chrome.storage.local` y Android usa MMKV,
pero OneTunnel no copiará ese patrón sin protección. Contraseñas no se
persisten; el token persistente futuro deberá usar Windows Credential Manager o
DPAPI. Los valores de configuración y diagnóstico deben redactar tokens y
parámetros sensibles.
