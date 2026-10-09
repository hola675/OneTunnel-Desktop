# Architecture

## Responsabilidades V1

| Componente | Responsabilidad |
| --- | --- |
| Wintun | Adaptador virtual y captura del tráfico IP Windows |
| tun2socks | Adaptación de paquetes IP a SOCKS5 |
| Xray | SOCKS loopback → VLESS + Reality → 1VPN |
| 1VPN | Proveedor VPN Free/Premium |
| Proxifier | Compatibilidad corporativa externa y opcional para xray.exe |

El supervisor futuro gestionará runtimes verificados, provider, selección de
servidor, adaptador, rutas, DNS, verificación, rollback y reconexión.
El nombre xray.exe debe conservarse para las reglas externas de aplicación.

La arquitectura aceptada está en [ADR 0001](adr/0001-v1-network-architecture.md).
[NETWORK-FLOW](NETWORK-FLOW.md) define exactamente los dos modos DIRECT y
CORPORATE_PROXY. [WINDOWS-VPN](WINDOWS-VPN.md) es el contrato principal de
networking y [CORPORATE-PROXY](CORPORATE-PROXY.md) el de compatibilidad externa.

## Máquina de estados objetivo

~~~text
DISCONNECTED
    ↓
PREPARING
    ↓
LOADING_PROVIDER
    ↓
SELECTING_SERVER
    ↓
STARTING_XRAY
    ↓
XRAY_READY
    ↓
CREATING_TUN
    ↓
STARTING_TUN2SOCKS
    ↓
CONFIGURING_ROUTES
    ↓
CONFIGURING_DNS
    ↓
VERIFYING
    ↓
CONNECTED
~~~

CORPORATE_PROXY inserta CHECKING_CORPORATE_PROXY después de SELECTING_SERVER
y antes de STARTING_XRAY. Es una futura comprobación de compatibilidad y
accesibilidad; la configuración y gestión de Proxifier permanecen externas.

En fallo: ERROR → ROLLBACK → DISCONNECTED. La desconexión solicitada también
debe ejecutar rollback. CONNECTED requiere tráfico verificado y política de
fugas satisfecha, no sólo procesos activos o listeners abiertos.

## Rollback obligatorio

Every Windows networking mutation must have a known inverse operation.

Antes de mutar se guarda el estado y se registra qué acciones efectivamente
tuvieron éxito. La recuperación debe ser idempotente y cubrir:

1. Restaurar DNS.
2. Restaurar rutas propias y configuración IPv6 modificada.
3. Parar tun2socks por su proceso propio.
4. Cerrar/eliminar Wintun propio.
5. Parar Xray por su proceso propio.
6. Restaurar el estado anterior restante y verificarlo.

La protección contra fugas debe mantenerse hasta completar la restauración.
No se deben tocar procesos, rutas o adaptadores ajenos.

Esta máquina y el rollback de sistema son diseño para M2/M3. M1/M1.2 sólo
ejecutan el gate Direct con cleanup de procesos y configs temporales.
