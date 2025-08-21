// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use clap::Parser;

mod api;
mod auth;
mod config;
mod database;
mod jwt;
mod macros;
mod model;
mod platforms;
mod plurality;
mod setup;
mod updater;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = setup::CliArgs::parse();

    let app_setup = setup::application_setup(&cli_args).await?;

    api::start_application(app_setup).await
}
