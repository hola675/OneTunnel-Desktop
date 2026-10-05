# Build and reproducibility

M0 no inicializa Tauri, Node frontend ni Rust backend. La auditoría local se
ejecuta sin instalar nada:

```powershell
.\scripts\audit-env.ps1
.\scripts\verify-runtime.ps1
.\scripts\smoke-test.ps1
```

La distribución futura será un `.exe` Windows x86_64 autocontenido. Cada
herramienta externa tendrá release exacto, fuente oficial, licencia y SHA256 en
`runtime/manifest.json` antes de poder entrar al empaquetado.

No se deben usar `npm create tauri-app`, `cargo tauri`, `npx` ni dependencias
`latest` durante M0.
