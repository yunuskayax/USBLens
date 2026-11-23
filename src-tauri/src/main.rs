// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Explorer menu "copy" items finish here, without opening the window.
    if usblens_lib::handle_cli() {
        return;
    }
    usblens_lib::run()
}
