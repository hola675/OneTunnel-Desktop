# Architecture

## Principio

La aplicación Windows será autónoma y empaquetará runtimes fijados. La
separación de responsabilidades es obligatoria:

- **Wintun**: interfaz de red virtual futura.
- **tun2socks**: futura conversión de tráfico de la interfaz a SOCKS.
- **Xray**: SOCKS local y protocolo VLESS + Reality hacia el servidor 1VPN.
- **wstunnel**: solo transporte TCP sobre WSS/TLS hacia un relay restringido.
- **relay**: forwarding controlado hacia el host 1VPN final, nunca proxy abierto.

La arquitectura congelada es:

```text
Windows Applications
        │
     Wintun
        │
   tun2socks
        │
   Xray SOCKS
        │
 Xray VLESS/Reality
        │
127.0.0.1:<dynamic-port>
        │
 wstunnel client
        │ WSS/TLS TCP/443
 wstunnel relay
        │
 1VPN server:443
        │
 Internet
```

En Restricted Mode Xray conserva VLESS, Reality y XTLS Vision, pero conecta a
un puerto local de forwarding; wstunnel realiza el transporte exterior.

## Máquina de estados

```text
DISCONNECTED
    ↓
PREPARING
    ↓
RESOLVING_RELAY
    ↓
STARTING_WSTUNNEL
    ↓
WSTUNNEL_READY
    ↓
STARTING_XRAY
    ↓
XRAY_READY
    ↓
STARTING_TUN
    ↓
CONFIGURING_ROUTES
    ↓
CONFIGURING_DNS
    ↓
VERIFYING
    ↓
CONNECTED
```

En cualquier fallo: `ERROR → ROLLBACK → DISCONNECTED`.

Cada acción que modifique Windows debe tener una acción inversa conocida. M0
solo documenta este contrato; no ejecuta ninguna acción de red.
