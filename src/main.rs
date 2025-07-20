mod models;
mod commands;
mod file_storage;
mod cli;

use std::{collections::HashMap, env, fs::{self, File}, path::PathBuf};

use clap::Parser;
use envie::Envie;
use snafu::{whatever, OptionExt, ResultExt, Whatever};

use crate::{cli::{Cli, Commands, CreateArgs, SwitchArgs, UpdateArgs}, file_storage::s3::S3Client, models::folder_config::*};

// ////////// //
// Entrypoint //
// ////////// //

#[tokio::main]
#[snafu::report]
async fn main() -> Result<(), Whatever> {
    // Load `.env` to system env vars.
    let _ = Envie::load().and_then(|env| env.export_to_system_env());

    let cli = Cli::parse();
    match cli.command {
        Commands::Create(args) => try_run_create(args).await.with_whatever_context(|_| "Create command failed"),
        Commands::Switch(args) => try_run_switch(args).await.with_whatever_context(|_| "Switch command failed"),
        Commands::Update(args) => try_run_update(args).await.with_whatever_context(|_| "Update command failed"),
    }?;

    Ok(())
}

// ////////////////// //
// Subcommand runners //
// ////////////////// //

async fn try_run_create(args: CreateArgs) -> Result<(), Whatever> {
    let input_dir = args.input_dir.or_else(|| env::var("UPDTR_INPUT_DIR").ok()).unwrap_or_else(|| ".".to_string());
    let path_prefix = args.filestore_path_prefix.clone().or_else(|| env::var("UPDTR_FILESTORE_PATH_PREFIX").ok()).unwrap_or_else(|| ".".to_string());

    let file_storage = S3Client::new_from_env()?;
    commands::create::run_create(args.display_version, &args.names, &input_dir, &path_prefix, file_storage).await?;

    Ok(())
}

async fn try_run_switch(args: SwitchArgs) -> Result<(), Whatever> {
    run_switch_or_update(
        Some(args.name.clone()),
        args.s3_url.clone(),
        args.filestore_path_prefix.clone(),
        args.output_dir.clone(),
        "UPDTR_OUTPUT_DIR",
        get_config,
        |name, dir, prefix, storage| Box::pin(async move {
            let _ = commands::switch::run_switch(name, dir, prefix, storage).await;
        }),
    ).await?;

    Ok(())
}

async fn try_run_update(args: UpdateArgs) -> Result<(), Whatever> {
    run_switch_or_update(
        args.name.clone(),
        args.s3_url.clone(),
        args.filestore_path_prefix.clone(),
        args.output_dir.clone(),
        "UPDTR_OUTPUT_DIR",
        get_config,
        |name, dir, prefix, storage| Box::pin(async move {
            let _ = commands::switch::run_switch(name, dir, prefix, storage).await;
        }),
    ).await?;

    Ok(())
}

// ///////////////// //
// Operation helpers //
// ///////////////// //

async fn run_switch_or_update(
    name: Option<String>,
    s3_url: Option<String>,
    filestore_path_prefix: Option<String>,
    output_dir: Option<String>,
    env_output_dir: &str,
    config_getter: fn() -> Result<Config, Whatever>,
    run_switch: fn(String, String, String, S3Client) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>,
) -> Result<(), Whatever> {
    let mut config = config_getter()?;
    let output_dir = output_dir.or_else(|| env::var(env_output_dir).ok()).unwrap_or_else(|| ".".to_string());
    let output_dir_for_path = output_dir.clone();
    let canonical_output_dir = fs::canonicalize(PathBuf::from(output_dir_for_path)).with_whatever_context(|_| format!("Could not canonicalize {output_dir}"))?;
    let canonical_output_dir_str = canonical_output_dir.to_str().with_whatever_context(|| format!("Could not convert path {:#?} to string", canonical_output_dir))?.to_owned();
    let folder_config = config.folders.get(&canonical_output_dir_str);

    let version = match &name {
        Some(name) => name.clone(),
        None => match folder_config {
            Some(folder_config) => folder_config.last_installed_version.clone(),
            None => whatever!("No version name provided and no name found in the configuration"),
        },
    };

    let s3_url = match &s3_url {
        Some(s3_url) => s3_url.clone(),
        None => match folder_config {
            Some(folder_config) => folder_config.s3_url.clone(),
            None => whatever!("No S3 URL provided and no S3 URL found in the configuration"),
        },
    };

    let path_prefix_ref = match filestore_path_prefix.as_ref() {
        Some(path_prefix) => path_prefix.clone(),
        None => match folder_config {
            Some(folder_config) => folder_config.storage_path_prefix.clone().unwrap_or_else(|| ".".to_string()),
            None => ".".to_string(),
        },
    };

    let file_storage = S3Client::new_from_url(&s3_url)?;
    let version_clone = version.clone();
    run_switch(version_clone, canonical_output_dir_str.clone(), path_prefix_ref.clone(), file_storage).await;

    if folder_config.is_none_or(|f| f.last_installed_version != version || f.s3_url != s3_url) {
        config.folders.insert(
            canonical_output_dir_str.clone(),
            FolderConfig {
                last_installed_version: version,
                s3_url: s3_url,
                storage_path_prefix: Some(path_prefix_ref),
            },
        );
        save_config(config).with_whatever_context(|_| "Switch or update succeeded, but could not update folder config")?;
    }

    Ok(())
}

// ////////////// //
// Config helpers //
// ////////////// //

fn get_config_path() -> Result<PathBuf, Whatever> {
    let config_dir = dirs::config_local_dir().with_whatever_context(|| format!("Could not resolve config local directory"))?;
    Ok(config_dir.join("h3xUpdtr").join("config.yaml"))
}

fn get_config() -> Result<Config, Whatever> {
    let config_path = get_config_path()?;

    match std::fs::File::open(&config_path) {
        Ok(file) => Ok(serde_yml::from_reader(file).with_whatever_context(|_| format!("Could not read {:#?}", config_path))?),
        Err(_) => Ok(Config { folders: HashMap::new() }),
    }
}

fn save_config(config: Config) -> Result<(), Whatever> {
    let config_path = get_config_path()?;
    let parent_dir = config_path.parent().with_whatever_context(|| "Could not get parent directory of config path")?;
    fs::create_dir_all(parent_dir).with_whatever_context(|_| format!("Could not create directory {:#?}", parent_dir))?;
    let file = File::create(&config_path).with_whatever_context(|_| format!("Could not create file {:#?}", config_path))?;
    serde_yml::to_writer(file, &config).with_whatever_context(|_| format!("Config file {:#?} created, but could not be written to", config_path))?;
    Ok(())
}
