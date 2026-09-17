# Credential storage

The application stores the Git PAT through the Rust `keyring` crate using the platform-native credential store when credentials are accessed.

- Windows: native Windows credential store.
- Android: native Android keyring store backed by the Android secure storage facilities.
- The PAT is not written to `localStorage`, config files, `.git/config`, commits, or GitHub Actions secrets.
- The frontend only sends the token to the Rust credential command when the user explicitly presses **Сохранить токен**.

Git credentials are injected into libgit2's HTTPS credential callback only when Git operations need them.
