# Git Folder Sync 0.5

Небольшой Tauri 2 клиент для синхронизации **одной выбранной папки** с **одним GitHub/GitLab Git-репозиторием**.

## Возможности

- Windows 11: системный выбор обычной папки.
- Android 11+: выбор папки через Storage Access Framework.
- HTTPS + Personal Access Token.
- PAT хранится через native credential store.
- Clone / Initialize, Pull, Push, Status.
- Pull только fast-forward; расхождение веток не мержится автоматически.
- Push не создаёт пустой commit.
- Android Git working tree хранится в приватном каталоге приложения.

## Сборка через GitHub Actions

Локальный Rust/Android toolchain не нужен. Workflow `.github/workflows/build.yml` сам поднимает окружение и собирает:

- Windows `.exe` + `.msi`;
- Android installable debug `.apk`.

### Быстрый путь

1. Создайте новый репозиторий на GitHub.
2. Распакуйте архив проекта.
3. Загрузите содержимое проекта в этот репозиторий, включая `.github/workflows/build.yml`.
4. Откройте **Actions → build**.
5. Нажмите **Run workflow**.
6. После завершения откройте run и скачайте artifacts:
   - `git-folder-sync-windows`;
   - `git-folder-sync-android`.

Workflow также запускается при push тега `v*`, например `v0.5.3`.

### Почему Android artifact — debug APK

Для установки на свой телефон без отдельного release keystore workflow собирает debug APK, подписанный стандартным debug-ключом Android. Это подходит для личного использования и тестирования. Для Google Play нужен отдельный production signing/AAB процесс.

## Синхронизация

### Push

Android сначала импортирует выбранную SAF-папку в приватный Git worktree. Затем Git получает изменения, включая удаления, создаёт commit и делает push.

### Pull

Android сначала импортирует выбранную папку. Если Git видит локальные изменения, Pull останавливается. Иначе выполняется fetch и только fast-forward. После успешного Pull рабочее дерево экспортируется обратно в выбранную SAF-папку.

## Безопасность

PAT не хранится в `localStorage` или файлах проекта. Он передаётся в Rust только для работы credential callback libgit2 и хранится через native credential store.

## CI build

GitHub Actions builds Windows and Android without a local Rust/Android toolchain.
Run **Actions → build → Run workflow**. Artifacts are uploaded as `git-folder-sync-windows`
and `git-folder-sync-android-arm64`.
