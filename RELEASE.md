# Release / GitHub Actions

## Manual build

GitHub → repository → Actions → **build** → **Run workflow**.

После завершения:

- artifact `git-folder-sync-windows`: `.exe` и `.msi`;
- artifact `git-folder-sync-android`: release `.apk`.

## Tagged build

```text
git tag v0.5.3
git push origin v0.5.3
```

Workflow автоматически запускается для тега `v*`.

## Почему здесь нет секретов

PAT пользователя для GitHub/GitLab не нужен во время CI-сборки. Он вводится уже в установленном приложении и сохраняется локально в системном credential store.

Для Android APK, который устанавливается вручную, отдельный signing key не обязателен для внутреннего использования. Для Play Store следует добавить отдельный production signing workflow и хранить keystore в GitHub Secrets.


## GitHub Actions CI

The workflow does not require `package-lock.json`: npm dependency caching is intentionally disabled so a fresh repository can be built with `npm install`.
The workflow uses Node.js 24-compatible GitHub Actions (`checkout@v7`, `setup-node@v7`, `setup-java@v5`, `setup-android@v4`, `upload-artifact@v7`).

Tauri builds the frontend automatically through `build.beforeBuildCommand = "npm run build"` before packaging the Windows or Android application.
