# Network flow

Estos son los dos flujos de red objetivo V1. Wintun, rutas y DNS aún no se
activan en M1.2; el gate ya certificado está en
[M1 Xray Direct](milestones/M1-XRAY-DIRECT.md).

## DIRECT

~~~text
Application
   ↓
Windows IP stack
   ↓
Wintun
   ↓
tun2socks
   ↓
Xray SOCKS
   ↓
VLESS + Reality
   ↓
1VPN:443
   ↓
Internet
~~~

Xray conecta directamente al servidor 1VPN seleccionado. El SOCKS interno
escucha sólo en loopback. [WINDOWS-VPN](WINDOWS-VPN.md) define la exclusión
física del destino 1VPN necesaria para evitar recapturar su salida.

## CORPORATE_PROXY

~~~text
Application
   ↓
Windows IP stack
   ↓
Wintun
   ↓
tun2socks
   ↓
Xray SOCKS
   ↓
Xray outbound TCP
   ↓
Proxifier
   ↓
Corporate Proxy
   ↓
1VPN:443
   ↓
Internet
~~~

Proxifier actúa únicamente sobre el tráfico outbound de xray.exe.
Loopback permanece DIRECT. OneTunnel.exe, tun2socks.exe y todas las aplicaciones
Windows no se fuerzan al proxy mediante Proxifier.
Xray conserva VLESS + Reality y el destino 1VPN:443; la conexión al proxy
corporativo debe seguir usando Wi-Fi/Ethernet físico según
[WINDOWS-VPN](WINDOWS-VPN.md).
