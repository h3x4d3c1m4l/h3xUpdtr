mod models;
mod commands;
mod file_storage;

use clap::{arg, Command};
use envie::Envie;

fn cli() -> Command {
    Command::new("h3xup")
        .about("h3xUpdtr - A file based updater tool for incremental updates")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("create")
                .about("Creates new version")
                .args(&[
                    arg!(-o [OUTPUT_PATH] "The path where the update definition will be created (an existing file will be overwritten), will write stdout if ommitted"),
                    arg!([INPUT_DIR] "The directory to create the update from, will be the current folder if ommitted"),
                ])
        )
}

fn main() {
    // Load `.env` to system env vars.
    let _ = Envie::load().and_then(|env| env.export_to_system_env());

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            commands::create::run_create(sub_matches.get_one::<String>("INPUT_DIR"), sub_matches.get_one::<String>("OUTPUT_PATH"));
        },
        _ => unreachable!(),
    }
}
