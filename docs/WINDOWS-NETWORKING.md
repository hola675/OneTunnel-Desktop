# Windows networking

## Loop prevention (obligatorio)

Cuando posteriormente todo el tráfico pase por Wintun, el relay wstunnel debe
quedar fuera de la VPN. Antes de activar las rutas por defecto hay que obtener:

```text
physical interface
physical gateway
relay IPv4
```

Después se crea una ruta específica `/32` al relay usando el gateway físico y
la interfaz física. Solo entonces se pueden instalar las rutas divididas:

```text
Relay IP
  ↓
physical Wi-Fi/Ethernet gateway

0.0.0.0/1
128.0.0.0/1
  ↓
Wintun
```

Esto impide el bucle:

```text
wstunnel → Wintun → Xray → wstunnel → ...
```

El supervisor debe conservar el estado previo y una operación inversa para
cada ruta. Si no puede resolverse el relay o instalarse su excepción física,
no debe activar el default route. M0 solo documenta este contrato: todavía no
se aplican rutas, DNS, firewall ni cambios de interfaz.
