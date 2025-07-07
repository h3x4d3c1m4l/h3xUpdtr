mod models;
mod commands;
mod file_storage;
mod cli;

use std::{collections::HashMap, env, fs::{self, File}, path::PathBuf};

use clap::Parser;
use envie::Envie;

use crate::{cli::{Cli, Commands, CreateArgs, SwitchArgs}, file_storage::s3::S3Client, models::folder_config::*};

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

    let file_storage = S3Client::new_from_env();
    commands::create::run_create(&args.names, &input_dir, &path_prefix, file_storage).await;
}

async fn try_run_switch(args: SwitchArgs) {
    let path_prefix = args.filestore_path_prefix.or_else(|| env::var("UPDTR_FILESTORE_PATH_PREFIX").ok()).unwrap_or_else(|| ".".to_string());

    let mut config = get_config();
    let output_dir = args.output_dir.or_else(|| env::var("UPDTR_OUTPUT_DIR").ok()).unwrap_or_else(|| ".".to_string());
    let canonical_output_dir = fs::canonicalize(PathBuf::from(output_dir)).unwrap();
    let folder_config = config.folders.get(canonical_output_dir.to_str().unwrap());

    let version = match &args.name {
        Some(name) => name,
        None => match folder_config {
            Some(folder_config) => &folder_config.last_installed_version,
            None => panic!("No version name provided and no name found in the configuration"),
        },
    };

    let s3_url = match &args.s3_url {
        Some(s3_url) => s3_url,
        None => match folder_config {
            Some(folder_config) => &folder_config.s3_url,
            None => panic!("No S3 URL provided and no S3 URL found in the configuration"),
        },
    };

    let file_storage = S3Client::new_from_url(s3_url);
    commands::switch::run_switch(&version, &canonical_output_dir.to_str().unwrap(), &path_prefix, file_storage).await;

    if folder_config.is_none_or(|f| &f.last_installed_version != version || &f.s3_url != s3_url) {
        config.folders.insert(canonical_output_dir.to_str().unwrap().to_owned(), FolderConfig { last_installed_version: version.to_owned(), s3_url: s3_url.to_owned() });
        save_config(config); //TODO: save s3 URL and base path
    }
}

// ////////////// //
// Config helpers //
// ////////////// //

fn get_config_path() -> PathBuf {
    dirs::config_local_dir().unwrap().join("h3xUpdtr").join("config.yaml")
}

fn get_config() -> Config {
    match std::fs::File::open(get_config_path()) {
        Ok(file) => {
            serde_yml::from_reader(file).unwrap()
        },
        Err(_) => {
            Config {
                folders: HashMap::new()
            }
        },
    }
}

fn save_config(config: Config) {
    fs::create_dir_all(get_config_path().parent().unwrap()).unwrap();
    let file = File::create(get_config_path()).unwrap();
    serde_yml::to_writer(file, &config).unwrap();
}
