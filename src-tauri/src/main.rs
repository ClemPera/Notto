// Minimal binary entry point for Tauri v2
// Delegates to the library entry point in lib.rs

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    notto_lib::run();
}
