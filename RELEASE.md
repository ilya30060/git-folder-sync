## v0.5.8\n\n- GitHub Actions builds Android ARM64 only; Windows job removed.\n- Android Gradle worker/daemon settings constrained for CI reliability.\n- NDK remains pinned to 29.0.14206865.\n\n# Git Folder Sync 0.5.7

Android CI fix: use the same Android NDK version as the Tauri-generated Android project (NDK 29.0.14206865) instead of mixing NDK 27.2 and 29. The workflow also prints the selected clang version and enables verbose Tauri Android build output so Gradle errors are visible if a future failure occurs.
