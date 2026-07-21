mod commands;
mod error;
mod library;
mod pdf;
mod state;
mod store;

use state::{AppState, PendingOpenFiles};
use std::sync::Mutex;
use tauri::menu::{
    AboutMetadataBuilder, IsMenuItem, Menu, MenuItemBuilder, MenuItemKind, PredefinedMenuItem,
    SubmenuBuilder,
};
use tauri::{Emitter, Manager};

/// Adds our items to the top of the default menu's File submenu
/// (creating one if the platform's default menu has none).
fn setup_menu(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItemBuilder::with_id("open-file", "Open PDF…")
        .accelerator("CmdOrCtrl+O")
        .build(app)?;
    let change = MenuItemBuilder::with_id("change-library-folder", "Change Library Folder…")
        .accelerator("CmdOrCtrl+Shift+O")
        .build(app)?;
    let settings = MenuItemBuilder::with_id("open-settings", "Settings…")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let items: [&dyn IsMenuItem<_>; 3] = [&open, &change, &separator];

    let menu = Menu::default(app.handle())?;

    // The default About item carries no icon, so the native About panel shows
    // a placeholder in dev builds. Rebuild it with metadata that includes the
    // app icon; the panel renders the bitmap at its pixel size, so use the
    // largest one we bundle.
    let pkg = app.package_info();
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128@2x.png")).ok();
    let about_metadata = AboutMetadataBuilder::new()
        .name(Some(pkg.name.clone()))
        .version(Some(pkg.version.to_string()))
        .icon(icon)
        .build();
    // Settings lives in the app menu (next to About), matching macOS convention;
    // on platforms with no such menu it falls back to the File menu below.
    let mut settings_placed = false;
    for item in menu.items()? {
        let MenuItemKind::Submenu(sub) = item else {
            continue;
        };
        let about_pos = sub.items()?.iter().position(|item| {
            matches!(item, MenuItemKind::Predefined(p) if p.text().is_ok_and(|t| t.starts_with("About")))
        });
        if let Some(pos) = about_pos {
            sub.remove_at(pos)?;
            let about = PredefinedMenuItem::about(app, None, Some(about_metadata))?;
            sub.insert(&about, pos)?;
            sub.insert(&settings, pos + 1)?;
            settings_placed = true;
            break;
        }
    }
    let file_menu = menu.items()?.into_iter().find_map(|item| match item {
        MenuItemKind::Submenu(sub) if sub.text().is_ok_and(|t| t == "File") => Some(sub),
        _ => None,
    });
    match file_menu {
        Some(file_menu) => {
            file_menu.insert_items(&items, 0)?;
            if !settings_placed {
                file_menu.append(&settings)?;
            }
        }
        None => {
            let mut builder = SubmenuBuilder::new(app, "File").items(&items[..2]);
            if !settings_placed {
                builder = builder.item(&settings);
            }
            let file_menu = builder.build()?;
            // on macOS index 0 is the application menu
            let pos = if cfg!(target_os = "macos") { 1 } else { 0 };
            menu.insert(&file_menu, pos)?;
        }
    }
    app.set_menu(menu)?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .manage(PendingOpenFiles::default())
        .plugin(tauri_plugin_dialog::init())
        // Persists and restores the main window's size and position across launches.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .register_asynchronous_uri_scheme_protocol(pdf::protocol::SCHEME, pdf::protocol::handle)
        .setup(|app| {
            setup_menu(app)?;

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
            pdfium_dirs.push(
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/pdfium"),
            );

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
                open_cancel: Mutex::new(None),
                identity_cache: Mutex::new(library::identity::IdentityCache::new()),
            });
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open-file" => {
                let _ = app.emit("menu-open-file", ());
            }
            "change-library-folder" => {
                let _ = app.emit("menu-change-folder", ());
            }
            "open-settings" => {
                let _ = app.emit("menu-open-settings", ());
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::get_settings,
            commands::library::set_library_folder,
            commands::library::get_library,
            commands::library::set_last_file,
            commands::library::set_pinned,
            commands::library::set_shortcuts,
            commands::pdf::open_document,
            commands::pdf::get_page_text,
            commands::pdf::get_page_links,
            commands::pdf::open_url,
            commands::pdf::close_document,
            commands::pdf::take_pending_open_files,
            commands::reading::save_reading_state,
            commands::bookmarks::save_bookmarks,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {
            // Files opened via Finder ("Open With", double-click) arrive as an
            // Apple event, not argv. Buffer them and poke the frontend.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = _event {
                let paths: Vec<String> = urls
                    .into_iter()
                    .filter_map(|url| url.to_file_path().ok())
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect();
                if paths.is_empty() {
                    return;
                }
                _app.state::<PendingOpenFiles>()
                    .0
                    .lock()
                    .unwrap()
                    .extend(paths);
                let _ = _app.emit("open-file", ());
            }
        });
}
