# Release / GitHub Actions

## Manual build

GitHub → repository → Actions → **build** → **Run workflow**.

После завершения:

- artifact `git-folder-sync-windows`: `.exe` и `.msi`;
- artifact `git-folder-sync-android`: release `.apk`.

## Tagged build

```text
git tag v0.5.0
git push origin v0.5.0
```

Workflow автоматически запускается для тега `v*`.

## Почему здесь нет секретов

PAT пользователя для GitHub/GitLab не нужен во время CI-сборки. Он вводится уже в установленном приложении и сохраняется локально в системном credential store.

Для Android APK, который устанавливается вручную, отдельный signing key не обязателен для внутреннего использования. Для Play Store следует добавить отдельный production signing workflow и хранить keystore в GitHub Secrets.
