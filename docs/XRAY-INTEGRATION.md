# Xray integration

La configuración conceptual se extrajo del template Android de 1VPN. El
contrato de la primera integración es:

```text
protocol: vless
port final: 443
flow: xtls-rprx-vision
security: reality
network: tcp
fingerprint: chrome
```

Los parámetros Reality del servidor 1VPN final que deben conservarse son:

```text
UUID
publicKey
shortId
serverName (realityServerName)
fingerprint
flow
```

El template observado usa un outbound VLESS con `encryption: none`,
`streamSettings.network: tcp`, `security: reality` y `realitySettings` con
`fingerprint: chrome`, `serverName`, `publicKey` y `shortId`. El inbound Android
es SOCKS local en loopback; OneTunnel debe mantener listeners internos solo en
`127.0.0.1`.

## Transformación de transporte

Original Android:

```text
Xray → 1VPN_HOST:443
```

OneTunnel Restricted:

```text
Xray → 127.0.0.1:LOCAL_WSTUNNEL_PORT
      → wstunnel → 1VPN_HOST:443
```

La única transformación inicial es el `address`/puerto de conexión de Xray
hacia el forwarding local. Xray conserva los parámetros Reality del destino
1VPN final; wstunnel no los interpreta.

M0 no implementa el generador final de configuración ni descarga Xray.
