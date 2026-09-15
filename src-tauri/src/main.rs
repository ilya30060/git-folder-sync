#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    git_folder_sync_lib::run();
}
