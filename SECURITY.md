# Security model

- Git transport is HTTPS.
- Git credentials are entered locally and the PAT is stored through `keyring`'s native store. On Windows this selects the Windows credential store; on Android the keyring crate's native Android store is used. The project initializes the native store at startup.
- The PAT is never written to `localStorage`, config JSON, Git commits, or the GitHub Actions environment.
- Android's selected directory is accessed through `ACTION_OPEN_DOCUMENT_TREE` and a persisted SAF URI permission.
- Android Git data stays inside app-private storage. The selected SAF directory is mirrored into/out of that private working tree.
- Pull refuses to proceed when the working tree has local changes and refuses divergent histories; there is no automatic destructive merge.
