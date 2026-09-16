# Git Folder Sync 0.5.7

Android CI fix: use the same Android NDK version as the Tauri-generated Android project (NDK 29.0.14206865) instead of mixing NDK 27.2 and 29. The workflow also prints the selected clang version and enables verbose Tauri Android build output so Gradle errors are visible if a future failure occurs.
