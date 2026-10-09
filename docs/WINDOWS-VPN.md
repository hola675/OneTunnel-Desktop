# Windows system VPN

Contrato de implementación para M2/M3. M1.2 no crea adaptadores ni cambia
rutas, DNS, IPv6, proxy o firewall.

## Componentes M2

| Componente | Responsabilidad |
| --- | --- |
| Wintun | Virtual network adapter; captura del tráfico IP Windows |
| tun2socks | Paquetes IP → SOCKS5 |
| Xray | SOCKS → VLESS + Reality → 1VPN |

M1 certifica un inbound SOCKS TCP sin UDP. Eso no prueba captura del sistema,
resolución DNS por VPN ni tráfico UDP. M2/M3 deberán elegir y validar el
transporte DNS y su captura antes de anunciar protección de fugas.
No se debe convertir el fixture Direct en evidencia de VPN completa.

## Snapshot antes de conectar

Antes de activar captura o cambiar networking, guardar de forma suficiente
para restaurar:

- Adaptador físico Wi-Fi/Ethernet, índice de interfaz y estado.
- Gateway físico y rutas originales, incluidos prefijos y métricas.
- Configuración DNS original por interfaz.
- Estado IPv6 original y cualquier protección temporal propia.

Cada mutación debe tener su inversa conocida y registrar su éxito. Si falla
el snapshot o no puede garantizarse recuperación, no activar rutas VPN.

## Routing IPv4

Rutas objetivo hacia OneTunnel/Wintun:

~~~text
0.0.0.0/1
128.0.0.0/1
~~~

Antes se conserva la accesibilidad del siguiente destino exterior por la
interfaz física para evitar loops:

- DIRECT: IPs del servidor 1VPN seleccionado.
- CORPORATE_PROXY: IPs del proxy corporativo efectivo.

Una excepción IPv4 conceptual, sólo cuando sea necesaria:

~~~text
CORPORATE_PROXY_IP/32
      ↓
physical Wi-Fi/Ethernet gateway
~~~

El supervisor deberá resolver/revalidar el destino efectivo, conservar
excepciones preexistentes y registrar sólo rutas que haya creado/modificado.
La resolución inicial es un paso de bootstrap, no autorización para dejar DNS
fuera de la VPN durante CONNECTED. Si el proxy resuelve a varios destinos,
la política debe cubrir los efectivamente utilizados. No se instala ninguna
excepción ni ruta dividida en esta fase.

## DNS

V1: **DNS traffic must not leak outside the VPN**.

M2/M3 deberán guardar DNS original, configurar DNS VPN, comprobar consultas
por VPN y ausencia de fugas, y restaurar DNS original al desconectar o fallar.
También deben cubrir la resolución realizada por Xray y por aplicaciones.
Si no puede cumplirse esta política, no declarar CONNECTED.
La selección y el transporte DNS todavía no están implementados.

## IPv6

Preferencia para la primera V1: IPv6 temporalmente deshabilitado o bloqueado
mientras la VPN esté conectada, hasta implementar tunelización IPv6 completa.
Debe guardarse el estado previo y existir rollback. Los siguientes destinos
físicos inicialmente deben ser accesibles por IPv4; si la protección IPv6
impide conectar, fallar de forma explícita. M1.2 no cambia IPv6.

## Desconexión y fallo

Seguir el [rollback de arquitectura](ARCHITECTURE.md): restaurar DNS, rutas
e IPv6, parar tun2socks propio, cerrar/eliminar Wintun propio, parar Xray
propio y verificar el estado restaurado. Las acciones deben ser idempotentes.
No retirar la protección contra fugas antes de recuperar el estado original.

Kill switch, reconnect y comprobación de recuperación/crash quedan para
implementación y pruebas posteriores. Ninguna de estas políticas está
certificada por el gate Direct.
