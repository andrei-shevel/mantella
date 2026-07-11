mod commands;
mod error;
mod library;
mod pdf;
mod state;
mod store;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol(pdf::protocol::SCHEME, pdf::protocol::handle)
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let store = store::Store::load(data_dir);

            // Where to look for the pdfium dynamic library: the bundled
            // resource dir first, then the source tree during development.
            let mut pdfium_dirs = Vec::new();
            if let Ok(resource_dir) = app.path().resource_dir() {
                pdfium_dirs.push(resource_dir.join("pdfium"));
            }
            #[cfg(debug_assertions)]
            pdfium_dirs
                .push(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/pdfium"));

            let pdf = pdf::engine::PdfWorker::spawn(pdfium_dirs);

            let watcher = store
                .settings
                .library_path
                .clone()
                .and_then(|root| library::watcher::start(app.handle().clone(), root).ok());

            app.manage(AppState {
                store: Mutex::new(store),
                pdf,
                watcher: Mutex::new(watcher),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::get_settings,
            commands::library::set_library_folder,
            commands::library::get_library,
            commands::library::set_pinned,
            commands::pdf::open_document,
            commands::pdf::close_document,
            commands::reading::save_reading_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
