use clap::{Args, Parser, Subcommand};

/// h3xUpdtr - A file-based tool for creating and applying incremental updates.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Creates a version and uploads it to the configured storage.
    Create(CreateArgs),

    /// Switch a folder to a given version.
    Switch(SwitchArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// The name of the version.
    pub name: String,

    /// The path prefix to prepend to all upload destinations.
    #[arg(short('p'), long)]
    pub filestore_path_prefix: Option<String>,

    /// The directory to create the update from (defaults to the current folder if omitted).
    #[arg(short, long)]
    pub input_dir: Option<String>,
}

#[derive(Args, Debug)]
pub struct SwitchArgs {
    /// The name of the version.
    pub name: String,

    /// The path prefix to prepend to all download paths.
    #[arg(short('p'), long)]
    pub filestore_path_prefix: Option<String>,

    /// The directory to update.
    #[arg(short, long)]
    pub output_dir: Option<String>,
}
