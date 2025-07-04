mod models;
mod commands;
mod file_storage;
mod cli;

use std::env;

use clap::Parser;
use envie::Envie;

use crate::cli::{Cli, Commands, CreateArgs, SwitchArgs};

// ////////// //
// Entrypoint //
// ////////// //

#[tokio::main]
async fn main() {
    // Load `.env` to system env vars.
    let _ = Envie::load().and_then(|env| env.export_to_system_env());

    let cli = Cli::parse();
    match cli.command {
        Commands::Create(args) => try_run_create(args).await,
        Commands::Switch(args) => try_run_switch(args).await,
    }
}

// ////////////////// //
// Subcommand runners //
// ////////////////// //

async fn try_run_create(args: CreateArgs) {
    let input_dir = args.input_dir.or_else(|| env::var("UPDTR_INPUT_DIR").ok()).unwrap_or_else(|| ".".to_string());
    let path_prefix = args.filestore_path_prefix.or_else(|| env::var("UPDTR_FILESTORE_PATH_PREFIX").ok()).unwrap_or_else(|| ".".to_string());

    commands::create::run_create(&args.names, &input_dir, &path_prefix).await;
}

async fn try_run_switch(args: SwitchArgs) {
    let output_dir = args.output_dir.or_else(|| env::var("UPDTR_OUTPUT_DIR").ok()).unwrap_or_else(|| ".".to_string());
    let path_prefix = args.filestore_path_prefix.or_else(|| env::var("UPDTR_FILESTORE_PATH_PREFIX").ok()).unwrap_or_else(|| ".".to_string());

    commands::switch::run_switch(&args.name, &output_dir, &path_prefix).await;
}

// //////////////////////////// //
// Argument/Environment helpers //
// //////////////////////////// //

fn parse_bool(value: &str) -> bool {
    match value.to_lowercase().as_str() {
        "y" | "yes" | "true" => true,
        _ => false
    }
}
