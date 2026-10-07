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

## M0.2.1 — compatibilidad PowerShell

El smoke test soporta Windows PowerShell 5.1 y PowerShell 7. La ayuda de
tun2socks termina con código 0 y escribe 1244 caracteres en stderr (stdout
vacío). Con `ErrorActionPreference = Stop`, la captura nativa `2>&1` en 5.1
produce `NativeCommandError`/`RemoteException`; en 7.6.2 captura la ayuda
correctamente. No es un fallo del runtime.

`scripts/lib/NativeProcess.ps1` captura ambos streams mediante
`System.Diagnostics.Process`, sin shell ni ventanas, con lectura asíncrona y
timeout. Devuelve `ExitCode`, `StdOut`, `StdErr` y `CombinedOutput`; este último
concatena los streams y no promete orden cronológico. Los probes de las tres
CLIs utilizan esta misma abstracción. La validación SHA256 permanece intacta.

```powershell
pwsh -NoProfile -File .\tests\test-native-process.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tests\test-native-process.ps1
pwsh -NoProfile -File .\scripts\smoke-test.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
```
