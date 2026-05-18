pub mod commands;
pub mod models;
pub mod services;
pub mod utils;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
