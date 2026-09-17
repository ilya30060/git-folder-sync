fn main() {
    println!("cargo:rerun-if-changed=permissions/android-saf");

    let attributes = tauri_build::Attributes::new().plugin(
        "android-saf",
        tauri_build::InlinedPlugin::new(),
    );

    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
