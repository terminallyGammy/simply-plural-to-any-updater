// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use clap::{self, Parser};
use tokio::runtime;

mod config;
mod simply_plural;
mod vrchat;
mod vrchat_auth;
mod vrchat_status;
mod webserver;
mod gui;
mod app;

fn main() {
    let cli_args = Cli::parse();

    if cli_args.no_gui {
        eprintln!("Running console mode...");
        runtime::Runtime::new()
            .unwrap()
            .block_on(app::run_app_logic())
            .unwrap()
    } else {
        eprintln!("Starting tauri GUI...");
        gui::run_tauri_gui().unwrap()
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Run without the graphical user interface
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    no_gui: bool,

    // todo. refactor this to have te webserver in this place here instead of it being possible to be invoked with the gui together
}
