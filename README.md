# OneTunnel Desktop

Cliente VPN autónomo para Windows 10/11 x86_64 que utilizará 1VPN como
proveedor, Xray con VLESS + Reality y wstunnel como transporte WSS/TLS por
TCP/443.

## Estado

**M0.2 — Runtime Pinning**

Este hito contiene la auditoría del entorno y de las referencias upstream, la
arquitectura congelada, el modelo de seguridad, runtimes oficiales fijados,
hashes verificables y probes offline. La VPN todavía no funciona: no hay GUI,
Tauri, adaptador Wintun, conexión de sistema ni networking modificado.

## Arquitectura objetivo

```text
Windows Applications → Wintun → tun2socks → Xray SOCKS
  → Xray VLESS/Reality → wstunnel client → WSS/TLS TCP/443
  → wstunnel relay → 1VPN:443 → Internet
```

En la primera PoC se demostrará únicamente la cadena Xray → forwarding local →
wstunnel → relay → 1VPN. wstunnel es transporte y no sustituye VLESS/Reality.

## Auditoría

```powershell
.\scripts\audit-env.ps1
.\scripts\verify-runtime.ps1
```

Las referencias de 1VPN son copias locales de código para estudio y no se
modifican. La ubicación exacta y sus limitaciones de procedencia están en
[`docs/upstream-baseline.md`](docs/upstream-baseline.md).

## Roadmap

1. **M0** — Bootstrap, auditoría y arquitectura base (completado).
2. **M0.2** — Pinning de runtimes, versiones oficiales y SHA256 (actual).
3. **PoC de red** — Xray, forwarding local, wstunnel y relay restringido.
4. **Cliente Windows** — Tauri, Wintun, tun2socks, rutas, DNS, rollback y
   diagnóstico.
5. **Distribución** — ejecutable autónomo x86_64 con runtimes verificados.

No se debe interpretar este repositorio como un cliente VPN operativo hasta que
los hitos posteriores estén certificados.
