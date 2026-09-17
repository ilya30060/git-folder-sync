# Android SAF bridge

The Android side uses `ACTION_OPEN_DOCUMENT_TREE` because the normal Tauri desktop folder dialog is not the right API for Android folder selection.

The Kotlin sources are copied into the generated Tauri Android project by `scripts/prepare-android.mjs`. The Rust side registers the native plugin and exposes three ordinary Tauri commands to the frontend:

- `android_pick_directory`
- `android_import_tree`
- `android_export_tree`

The selected `content://` URI is persisted with `takePersistableUriPermission()`. Git itself never receives the URI. The Git worktree remains in the application's private storage.


## v0.5.13

The Android SAF plugin commands are explicitly granted through `src-tauri/permissions/android-saf.toml` and the Android-only capability.
