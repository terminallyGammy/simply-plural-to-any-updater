// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use std::future::Future;
use config::setup_and_load_config;
use tokio::runtime;
use clap::{self, Parser};

mod config;
mod simply_plural;
mod vrchat;
mod vrchat_auth;
mod vrchat_status;
mod webserver;
mod gui;

fn main() {
    let cli_args = config::CliArgs::parse();

    let config = run_async_blocking(setup_and_load_config(&cli_args)).unwrap();

    if cli_args.webserver {
        eprintln!("Running in Webserver mode ...");
        run_async_blocking(webserver::run_server(config.clone())).unwrap();
        return;
    }

    if cli_args.no_gui {
        eprintln!("Running SP2Any Updater in console mode ...");
        run_async_blocking(vrchat::run_updater_loop(&config)).unwrap();
    } else {
        eprintln!("Starting SP2Any Updater in GUI mode ...");
        gui::run_tauri_gui(config.clone()).unwrap()
    }
}

fn run_async_blocking<T>(f: impl Future<Output = T>) -> T {
    runtime::Runtime::new()
        .unwrap()
        .block_on(f)
}
