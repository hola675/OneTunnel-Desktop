# Build and reproducibility

M1.2 mantiene los placeholders de aplicación. No inicializa Tauri, no requiere
elevación ni modifica networking Windows.

## Runtime

~~~powershell
.\scripts\audit-env.ps1
.\scripts\fetch-runtime.ps1
.\scripts\verify-runtime.ps1
~~~

El manifiesto schemaVersion 3 sólo admite Xray, tun2socks y Wintun.
Cada archivo comprimido y runtime tiene SHA256 fijado y URL upstream oficial.
La segunda ejecución de fetch-runtime produce PASS CACHED para los tres
binarios correctos. Se rechazan archivos cacheados o extraídos cuyo hash no
coincida; fetch no reescribe los pins.

bin/ y downloads/ se ignoran en Git. Los artefactos locales de dependencias
retiradas pueden permanecer allí: no se descargan, buscan, ejecutan ni
empaquetan como componentes activos. No es necesario limpiar esas caches.

La distribución futura será portable y con instalador Windows x64, con los
runtimes y notices requeridos; no se debe empaquetar indiscriminadamente toda
la cache. No ejecutar npm create tauri-app, cargo tauri ni npx en esta fase.

## Smoke offline en ambos shells

~~~powershell
pwsh -NoProfile -File .\scripts\smoke-test.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
~~~

Matriz: Xray version/config Reality, tun2socks version/help y hash Wintun.
La DLL no se carga ni se crea un adaptador.

scripts/lib/NativeProcess.ps1 captura ambos streams mediante
System.Diagnostics.Process sin shell ni ventanas, con lectura asíncrona y
timeout. La ayuda tun2socks escribe en stderr con exit 0; se comprueba
CombinedOutput, evitando NativeCommandError en Windows PowerShell 5.1.
CombinedOutput concatena streams y no garantiza orden cronológico.

## Tests y Direct

~~~powershell
pwsh -NoProfile -File .\tests\test-native-process.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tests\test-native-process.ps1
pwsh -NoProfile -File .\tests\test-https-probe.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tests\test-https-probe.ps1
pwsh -NoProfile -File .\tests\test-xray-config.ps1
pwsh -NoProfile -File .\scripts\test-1vpn-direct.ps1
~~~

test-xray-config incluye el contrato Free en tests/contracts/1vpn/ y valida
los seis configs sin conexión remota. HTTPS policy se prueba sin Internet.
El Direct probe requiere PowerShell 7 e Internet; sólo crea Xray y configs
propios temporales, sin modificar networking. Se puede seleccionar Location
(ams/sgp/lax) y ServerIndex (1/2).

La certificación Direct y la política STRICT / best-effort exacto están en
[M1 Xray Direct](milestones/M1-XRAY-DIRECT.md).
git diff --check debe pasar antes de revisión. M1.2 queda sin commit/push.
