# Git Folder Sync v0.5.13

Android startup crash fix.

Fixes:
- Removed early `keyring::cli::use_native_store(false)` initialization.
- Enabled `keyring`'s `android-native-keyring-store` backend.
- Credential storage is accessed only when credentials are explicitly saved or retrieved.
- Bumped application/package version to 0.5.13.
