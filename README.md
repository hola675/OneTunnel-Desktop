# OneTunnel Desktop

OneTunnel Desktop is a Windows desktop client foundation for a future 1VPN Free and Premium client. The planned connection engine is Xray-core using VLESS + Reality + XTLS Vision over TCP 443, exposed to selected applications through a configurable local SOCKS5 endpoint.

## Current state (F0)

- Tauri 2 desktop shell with a React, TypeScript, and Vite frontend.
- Initial disconnected screen; Connect stays disabled.
- A minimal Rust `get_app_info` command reports the app name, version, platform, and uninitialized core state.
- No VPN connection, account integration, Xray process, SOCKS listener, or Windows network changes are implemented.
- F0 is certified: the stable Rust MSVC toolchain and Visual Studio Build Tools are verified; Cargo checks/tests, formatting, and frontend build pass; the native window, React state, and `get_app_info` IPC were verified. Normal foreground `tauri dev` and three standalone Release runs closed their native PIDs; the dev CLI exited 0 and Vite released port 5173. The neutral technical icon remains provisional and must be replaced before public distribution. See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for lifecycle evidence.

The SOCKS address shown in the UI (`127.0.0.1:10808`) is the planned default only; no listener is active in F0. Rust will own connection state. Proxifier remains an external, user-managed application and OneTunnel will not alter its configuration.

## Requirements

- Windows 10/11 x64
- Node.js 20.19+ or 22.12+ and npm
- Rust stable with the `x86_64-pc-windows-msvc` toolchain
- Microsoft C++ Build Tools with **Desktop development with C++** and a Windows SDK
- Microsoft Edge WebView2 Runtime (normally included on supported Windows 10/11 versions)

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for setup and environment audit notes.

## Run and build

```powershell
npm install
npm run build
npm run tauri dev
```

Native build prerequisites are required for `tauri dev` and `cargo check`.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Project scope](docs/PROJECT-SCOPE.md)
- [Roadmap](docs/ROADMAP.md)
- [Development](docs/DEVELOPMENT.md)

## Estado funcional y límites de seguridad (octubre de 2026)

- La interfaz de escritorio permite autenticación y diagnóstico, pero **no ofrece aún una conexión VPN extremo a extremo certificada**. El botón Conectar permanece deshabilitado hasta implementar el flujo real y su verificación.
- El proceso Xray administrado por la aplicación utiliza SOCKS5 local en `127.0.0.1:10808`. El estado `running` significa únicamente que el handshake SOCKS5 local responde; **no** acredita IP de salida, conectividad HTTPS ni protección del tráfico del sistema.
- Proxifier es externo y responsable de las reglas efectivas de redirección de aplicaciones. Las reglas de enrutamiento de OneTunnel son **simulaciones** y no interceptan, bloquean ni redirigen tráfico real de Windows.
- La comprobación WFP es solo de diagnóstico: no instala controladores ni habilita interceptación.
- La pantalla Diagnóstico incluye un registro **temporal y limitado a 80 cambios de estado**, sin persistencia en disco; no constituye un registro de tráfico ni de actividad de red.
- El motor serializa los intentos de arranque/parada y comprueba la disponibilidad SOCKS5 del proceso supervisado. No se ha certificado todavía el comportamiento en Windows con Xray real ni las rutas Free/Premium.

### Validación pendiente antes de distribución

Ejecutar en un entorno Windows con Rust, Node y el runtime Xray verificado:

```powershell
cd src-tauri
cargo fmt --all -- --check
cargo check
cargo test
cd ..
npm ci
npm run build
```

Validar manualmente inicio/cierre repetidos, puerto 10808 ocupado, caída de Xray, pérdida de handshake, ausencia de procesos huérfanos, limpieza de configuración temporal y verificación HTTPS a través del proxy antes de habilitar Conectar.
