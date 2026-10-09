# Xray integration

## Contrato V1

El template Android de 1VPN define:

~~~text
protocol: vless
destination: selected 1VPN host:443
flow: xtls-rprx-vision
encryption: none
security: reality
network: tcp
fingerprint: chrome
~~~

Se conservan UUID, publicKey, shortId, serverName/realityServerName,
fingerprint y flow del servidor/provider seleccionado.
El proceso mantiene su nombre xray.exe y el inbound interno escucha sólo en
127.0.0.1. En ambos modos el outbound Xray conserva el destino 1VPN:443.
CORPORATE_PROXY usa la compatibilidad externa descrita en
[CORPORATE-PROXY](CORPORATE-PROXY.md).

## Generador actual

scripts/new-xray-config.ps1 consume un único fixture público
tests/fixtures/1vpn/free-contract.json y admite Location (ams/sgp/lax),
ServerIndex (1/2) y SocksPort (0 elige un puerto dinámico).

Produce un config Free mínimo: SOCKS5 loopback, UDP desactivado y un único
outbound VLESS/TCP/Reality, sin DNS ni routing propios. Devuelve Path,
SocksPort, Location, Server y RealityServerName. Las configuraciones únicas
se escriben en %TEMP%\OneTunnel\xray\; quien las genera debe eliminarlas.
La reserva de puerto se libera antes de arrancar Xray: el runner comprueba
que el listener posterior pertenece a su PID y falla si el arranque colisiona.

tests/contracts/1vpn/test-free-contract.ps1 verifica procedencia, constantes
Free y emparejamientos host/SNI del snapshot. tests/test-xray-config.ps1 llama
ese contrato y valida offline las seis configuraciones con el runtime fijado.

## Gate actual y evolución

scripts/test-1vpn-direct.ps1 verifica runtimes, configura y arranca Xray
propio, prueba HTTPS por su SOCKS y compara hashes de egress; luego limpia
procesos/configs. Su evidencia está en
[M1 Xray Direct](milestones/M1-XRAY-DIRECT.md).

El generador actual es exclusivamente Free. Premium, 2FA y selección dinámica
de provider se implementarán a partir del [contrato API](API-1VPN.md);
no hay credenciales Premium de ejemplo. DNS, UDP y captura de sistema deberán
validarse en M2/M3; el config Direct no certifica ausencia de fugas.
