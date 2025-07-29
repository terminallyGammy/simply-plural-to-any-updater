// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use clap::{Parser, self};
use tokio::runtime;

mod config;
mod simply_plural;
mod vrchat;
mod vrchat_auth;
mod vrchat_status;
mod webserver;


fn main() {
    let cli_args = Cli::parse();

    if cli_args.no_gui {
        eprintln!("Running console mode...");
        runtime::Runtime::new().unwrap().block_on(run_app_logic()).unwrap()
    } else {
        eprintln!("Starting tauri GUI...");
        run_tauri_gui().unwrap()
    }
}


fn run_tauri_gui() -> Result<(), anyhow::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(move |_app| {
            eprintln!("Tauri application setup complete. Spawning core logic...");

            tauri::async_runtime::spawn(async move {
                run_app_logic().await.unwrap()
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(anyhow::Error::from)
}


async fn run_app_logic() -> Result<()> {
    eprintln!("Starting Simply Plural to Any Updater...");

    let config = config::setup_and_load_config().await?;

    if config.serve_api {
        eprintln!("Running in Webserver mode.");
        webserver::run_server(&config).await?;
    } else {
        eprintln!("Running in VRChat Updater mode.");
        vrchat::run_updater_loop(&config).await?;
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Run without the graphical user interface
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    no_gui: bool,
}
