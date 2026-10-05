# Upstream baseline

Fecha de auditoría: **2026-10-05**.

Las dos referencias encontradas son copias de trabajo/snapshots sin directorio
`.git` en su raíz ni metadatos Git anidados detectables. Por ello no es posible
afirmar de forma reproducible un remote, branch o commit SHA. No se inventan

## 1VPN Browser Extension

| Campo | Valor |
|---|---|
| repository | snapshot local `reference/browser-extension-main/browser-extension-main` |
| branch | no disponible: no hay `.git` |
| commit SHA | no disponible: no hay `.git` |
| remote | no disponible: no hay `.git` |
| working tree | no aplicable; snapshot no versionado |
| fuente declarada | `1vpn/browser-extension` (según el encargo) |

## 1VPN Android

| Campo | Valor |
|---|---|
| repository | snapshot local `reference/android-app-master/android-app-master` |
| branch | no disponible: no hay `.git` |
| commit SHA | no disponible: no hay `.git` |
| remote | no disponible: no hay `.git` |
| working tree | no aplicable; snapshot no versionado |
| fuente declarada | `1vpn/android-app` (según el encargo) |

## Integridad y movimiento

No se modificaron archivos dentro de ninguno de los snapshots durante M0.
Tampoco se movieron: hacerlo sin sus metadatos Git no preservaría historial ni
permitiría certificar el origen. Se reservaron los directorios canónicos
`reference/1vpn-browser-extension/` y `reference/1vpn-android-app/` con
marcadores que apuntan a las ubicaciones actuales. El movimiento seguro futuro
requiere obtener clones Git completos y registrar remote, branch, SHA y estado
limpio antes y después.
