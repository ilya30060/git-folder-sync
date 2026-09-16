# Git Folder Sync v0.5.9

Android-only CI release.

Fixes:
- Replaced obsolete Android `Plugin.onActivityResult` override with Tauri 2 `@ActivityCallback` + `startActivityForResult`.
- Keeps SAF directory permissions using `takePersistableUriPermission`.
- Windows CI remains disabled.
- Android build remains ARM64 (`aarch64`).
