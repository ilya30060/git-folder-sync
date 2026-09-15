# Sync protocol

## Push

1. On Android, import the selected SAF directory into the private Git worktree.
2. Stage all changes including deletions.
3. If the tree is unchanged, do not create an empty commit.
4. Create a local commit.
5. Push the configured branch to `origin`.

## Pull

1. On Android, import the selected SAF directory first. This makes external changes visible to Git.
2. Refuse Pull when the working tree is dirty.
3. Fetch the configured branch.
4. If already up to date, stop.
5. If fast-forward is possible, update the branch and checkout.
6. If histories diverge, stop without automatic merge.
7. On Android, export the resulting worktree back to the selected SAF directory.

This is intentionally conservative: the application does not silently merge or overwrite a folder with uncommitted changes.
