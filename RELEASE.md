
## v0.5.17-fixed7

- Fixed Rust build error from `git2::opts::set_ssl_cert_file` requiring an `unsafe` block.
- Moved Android CA initialization to app startup, before the Tauri event loop, so the global libgit2 SSL option is configured before any worker can issue Git operations.
- Removed the unnecessary `mut` from the existing remote handle.

# Git Folder Sync v0.5.12

Android startup crash fix.

Fixes:
- Removed early `keyring::cli::use_native_store(false)` initialization.
- Enabled `keyring`'s `android-native-keyring-store` backend.
- Credential storage is accessed only when credentials are explicitly saved or retrieved.
- Bumped application/package version to 0.5.12.
