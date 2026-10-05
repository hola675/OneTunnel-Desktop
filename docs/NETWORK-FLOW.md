# Network flow

## Restricted Mode — principal

```text
Application
    ↓
Xray SOCKS / VLESS + Reality
    ↓
127.0.0.1:<wstunnel-local-port>
    ↓
wstunnel client
    ↓ WSS/TLS over TCP/443
wstunnel relay
    ↓
1VPN server:443
    ↓
Internet
```

El destino exterior visible desde el cliente para el transporte debe ser
únicamente el relay wstunnel por TCP/443. Xray no debe conectarse directamente
al servidor 1VPN cuando Restricted Mode está activo. wstunnel no interpreta ni
termina VLESS, Reality o XTLS Vision: solo reenvía TCP.

La primera PoC probará esta cadena sin Wintun ni tun2socks. El listener local
debe enlazar exclusivamente en `127.0.0.1` y usar un puerto dinámico reservado
por el supervisor.

## Direct Mode — futuro/fallback

```text
Xray
    ↓
1VPN server:443
```

Direct Mode queda documentado como fallback futuro. No es la prioridad de M0 y
no habilita ningún comportamiento operativo en esta fase.
