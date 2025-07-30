use anyhow::{anyhow, Result};
use tauri::{self, tray::TrayIcon};

use crate::{config::Config, vrchat};

pub fn run_tauri_gui(config: Config) -> Result<(), anyhow::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(move |app| {
            eprintln!("Tauri application setup complete. Spawning core logic...");

            tauri::async_runtime::spawn(async move { vrchat::run_updater_loop(&config).await });

            tauri_system_tray_handler(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow!(e))
}

fn tauri_system_tray_handler(app: &mut tauri::App) -> Result<TrayIcon> {
    let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = tauri::menu::Menu::with_items(app, &[&quit_i])?;

    tauri::tray::TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => {
                eprintln!("tauri: menu event quit.");
                app.exit(0);
            }
            unknown => {
                eprintln!("tauri: menu event {unknown} unknown.");
            }
        })
        .build(app)
        .map_err(|e| anyhow!(e))
}
