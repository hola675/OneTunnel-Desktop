# Build and reproducibility

M0.2 no inicializa Tauri, no requiere elevación y no modifica networking. Con
un checkout limpio, los runtimes se reproducen y validan así:

```powershell
.\scripts\audit-env.ps1
.\scripts\fetch-runtime.ps1
.\scripts\verify-runtime.ps1
.\scripts\smoke-test.ps1
```

La segunda ejecución de `fetch-runtime.ps1` debe producir `PASS CACHED` para
cada runtime cuyo binario y hash sigan presentes.

La distribución futura será un `.exe` Windows x86_64 autocontenido. Cada
herramienta externa tiene release exacto, fuente oficial, licencia, SHA256 del
archivo comprimido y SHA256 del runtime extraído en `runtime/manifest.json`.

No se deben usar `npm create tauri-app`, `cargo tauri`, `npx` ni dependencias
`latest` durante M0.2.
