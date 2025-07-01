mod models;
mod commands;
mod file_storage;

use clap::{arg, Command};
use envie::Envie;

fn cli() -> Command {
    Command::new("h3xup")
        .about("h3xUpdtr - A file-based tool for creating and applying incremental updates")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("create")
                .about("Create a version and upload it to S3 storage")
                .args(&[
                    arg!(-n --"name" [VERSION_NAME] "The name of the version").id("VERSION_NAME"),
                    arg!(-p --"filestore-path-prefix" [FILESTORE_PATH_PREFIX] "The path prefix to prepend to all upload destinations").id("FILESTORE_PATH_PREFIX"),
                    arg!([INPUT_DIR] "The directory to create the update from (defaults to the current folder if omitted)").required(false).id("INPUT_DIR"),
                ])
        )
        .subcommand(
            Command::new("switch")
                .about("Switch a folder to a given version")
                .args(&[
                    arg!(-n --"name" [VERSION_NAME] "The name of the version").id("VERSION_NAME"),
                    arg!(-p --"filestore-path-prefix" [FILESTORE_PATH_PREFIX] "The path prefix to prepend to all download paths").id("FILESTORE_PATH_PREFIX"),
                    arg!([OUTPUT_DIR] "The directory to update").required(false).id("OUTPUT_DIR"),
                ])
        )
}

#[tokio::main]
async fn main() {
    // Load `.env` to system env vars.
    let _ = Envie::load().and_then(|env| env.export_to_system_env());

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            let version_name = get_arg_or_env_or_default(sub_matches, "VERSION_NAME", "UPDTR_VERSION_NAME", None).expect("Version name is mandatory");
            let input_dir = get_arg_or_env_or_default(sub_matches, "INPUT_DIR", "UPDTR_INPUT_DIR", Some(".")).unwrap();
            let path_prefix = get_arg_or_env_or_default(sub_matches, "FILESTORE_PATH_PREFIX", "UPDTR_FILESTORE_PATH_PREFIX", Some(".")).unwrap();

            commands::create::run_create(&version_name, &input_dir, &path_prefix).await;
        },
        Some(("switch", sub_matches)) => {
            let version_name = get_arg_or_env_or_default(sub_matches, "VERSION_NAME", "UPDTR_VERSION_NAME", None).expect("Version name is mandatory");
            let output_dir = get_arg_or_env_or_default(sub_matches, "OUTPUT_DIR", "UPDTR_OUTPUT_DIR", Some(".")).unwrap();
            let path_prefix = get_arg_or_env_or_default(sub_matches, "FILESTORE_PATH_PREFIX", "UPDTR_FILESTORE_PATH_PREFIX", Some(".")).unwrap();

            commands::switch::run_switch(&version_name, &output_dir, &path_prefix).await;
        },
        _ => unreachable!(),
    }
}

fn get_arg_or_env_or_default<'a>(
    matches: &'a clap::ArgMatches,
    arg_name: &str,
    env_var: &str,
    fallback: Option<&'a str>,
) -> Option<String> {
    if let Some(cli_val) = matches.get_one::<String>(arg_name) {
        return Some(cli_val.clone());
    }

    if let Ok(env_val) = std::env::var(env_var) {
        return Some(env_val);
    }

    fallback.map(|s| s.to_string())
}
