fn main() {
    println!("cargo:rerun-if-changed=permissions/android-saf");

    let attributes = tauri_build::Attributes::new()
        .plugin(
            "android-saf",
            tauri_build::InlinedPlugin::new(),
        )
        .app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "save_credentials",
                "delete_credentials",
                "clone_or_initialize",
                "push_repo",
                "pull_repo",
                "repo_status",
            ]),
        );

    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
