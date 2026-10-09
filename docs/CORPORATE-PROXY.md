# Corporate proxy compatibility

## V1 policy

Proxifier es **optional, external, user-managed, not bundled,
not installed automatically**. Es un componente externo de compatibilidad.
OneTunnel no posee ni administra el proxy corporativo y no almacena sus
credenciales en V1. La configuración de Proxifier permanece fuera de OneTunnel.

Conservar el nombre del proceso **xray.exe**; no renombrarlo. El generador
sigue apuntando al servidor 1VPN:443 y mantiene todos sus parámetros Reality.

La experiencia corporativa aportada para la decisión observó acceso de Xray
a 1VPN mediante Proxifier y el proxy existente. M1 certifica automáticamente
sólo Direct; M1.2 documenta esta política y no certifica una VPN de sistema
corporativa.

## Reglas esperadas, gestionadas por el usuario

En Proxifier, la regla de aplicación debe seleccionar:

~~~text
Application: xray.exe
Action: Corporate Proxy
~~~

Exclusiones con precedencia sobre la regla de aplicación:

| Destino | Acción |
| --- | --- |
| 127.0.0.1 | DIRECT |
| localhost | DIRECT |
| ::1 | DIRECT |

Proxifier actúa únicamente sobre el tráfico outbound de xray.exe.
OneTunnel no debe obligar a pasar por ese proxy a OneTunnel.exe,
tun2socks.exe ni todas las aplicaciones Windows.
Las reglas loopback preservan el SOCKS interno y evitan recaptura local.

## Accesibilidad y loops

El proxy debe seguir accesible por el adaptador físico Wi-Fi/Ethernet.
La futura excepción CORPORATE_PROXY_IP/32 se define en
[WINDOWS-VPN](WINDOWS-VPN.md), sólo cuando sea necesaria.
Antes de activar captura, el supervisor deberá comprobar accesibilidad del
proxy y compatibilidad del modo. Si falla, detener conexión y hacer rollback.

## Límites de esta fase

M1.2 no edita perfiles, inyecta reglas, modifica configuración ni arranca/para
Proxifier. No guarda contraseñas del proxy y no detecta ni automatiza esta
integración. Esas comprobaciones vendrán en una fase posterior, conservando
la configuración externa administrada por el usuario.
