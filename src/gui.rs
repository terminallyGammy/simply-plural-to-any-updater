/* WORK-IN-PROGRESS */

use anyhow::{anyhow, Result};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIcon;
use tauri::{Emitter, Manager, State};

use crate::config::Config;
use crate::config_store;
use crate::updater::UpdaterState;
use crate::updater_loop;
use crate::CliArgs;

/* Payload for single instance of the program*/
#[derive(Clone, Serialize)]
struct SingleInstancePayload {
    args: Vec<String>,
    cwd: String,
}

struct AppState {
    cli_args: CliArgs,
    updater_state: Arc<Mutex<Vec<UpdaterState>>>,
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
fn get_config(state: State<AppState>) -> Result<config_store::LocalJsonConfigV2, String> {
    config_store::read_local_config_file(&state.cli_args).map_err(|e| e.to_string())
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
fn set_config(
    state: State<AppState>,
    config: config_store::LocalJsonConfigV2,
) -> Result<(), String> {
    config_store::write_local_config_file(&config, &state.cli_args).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn get_updaters_state(state: State<AppState>) -> Result<Vec<UpdaterState>, String> {
    state
        .updater_state
        .try_lock()
        .map_err(|e| e.to_string())
        .map(|d| d.clone())
}

pub fn run_tauri_gui(
    config: Config,
    updater_state: Arc<Mutex<Vec<UpdaterState>>>,
) -> Result<(), anyhow::Error> {
    let app_state = AppState {
        cli_args: config.cli_args.clone(),
        updater_state,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            eprintln!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit("single-instance", SingleInstancePayload { args: argv, cwd })
                .unwrap();
        }))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            get_updaters_state
        ])
        .setup(move |app| {
            eprintln!("Tauri application setup complete. Spawning core logic...");

            let shared_updater_state = app.handle().state::<AppState>().updater_state.clone();

            tauri::async_runtime::spawn(async move {
                updater_loop::run_loop(&config, shared_updater_state).await;
            });

            tauri_system_tray_handler(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow!(e))
}

fn tauri_system_tray_handler(app: &tauri::App) -> Result<TrayIcon> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

    tauri::tray::TrayIconBuilder::new()
        .icon(
            app.default_window_icon()
                .ok_or_else(|| anyhow!("No icon found."))?
                .clone(),
        )
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => {
                eprintln!("tauri: menu event quit.");
                app.exit(0);
            }
            "show" => {
                eprintln!("tauri: menu event show.");
                if let Some(window) = app.get_webview_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            "hide" => {
                eprintln!("tauri: menu event hide.");
                if let Some(window) = app.get_webview_window("main") {
                    window.hide().unwrap();
                }
            }
            unknown => {
                eprintln!("tauri: menu event {unknown} unknown.");
            }
        })
        .show_menu_on_left_click(true)
        .build(app)
        .map_err(|e| anyhow!(e))
}
