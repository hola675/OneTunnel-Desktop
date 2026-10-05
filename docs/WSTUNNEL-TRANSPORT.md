# wstunnel transport

La autoridad para la sintaxis es el README oficial de
[`erebe/wstunnel`](https://github.com/erebe/wstunnel). La forma actual de
forwarding TCP local a remoto es:

```text
wstunnel client -L tcp://LOCAL_PORT:DESTINATION:443 wss://RELAY:443
```

Ejemplo conceptual para Restricted Mode (todos los valores son placeholders):

```powershell
wstunnel client `
  -L "tcp://127.0.0.1:11443:TARGET_1VPN_HOST:443" `
  --tls-verify-certificate `
  --dns-resolver-prefer-ipv4 `
  wss://RELAY_HOST:443
```

El formato CLI documentado por upstream es `wstunnel client [OPTIONS]
<ws[s]|http[s]|wts://server[:port]>`; `-L/--local-to-remote` puede repetirse.
La PoC usará `wss://`, TCP/443 y verificación TLS. No se escriben aquí
hostnames reales del relay ni secretos de upgrade.

## Proxy corporativo futuro

El cliente podrá usar `--http-proxy` para redes que exijan proxy corporativo.
No se implementa ni prueba en M0; cualquier credencial de proxy deberá
inyectarse de forma segura y nunca aparecer en argumentos persistidos o logs.

## Requisitos del relay

El despliegue del relay debe cumplir todos estos requisitos:

- certificado TLS válido y nombre verificable;
- listener TCP/443;
- restricciones explícitas de destinos;
- ruta de upgrade/autenticación personalizada cuando proceda;
- ausencia de proxy abierto;
- logging mínimo y sin credenciales.

El servidor debe utilizar `--restrict-to` o `--restrict-config` para limitar los
destinos reenviables al conjunto permitido. La configuración debe rechazar
cualquier destino no autorizado; nunca se debe desplegar un relay genérico.
