use std::sync::LazyLock;

use clap::{Args, Parser, Subcommand};
use console::Emoji;
use indicatif::ProgressStyle;

// ////////////////////// //
// Output styling helpers //
// ////////////////////// //

pub static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
pub static HOURGLASS: Emoji<'_, '_> = Emoji("⌛ ", "");
pub static CHECKLIST: Emoji<'_, '_> = Emoji("📋 ", "");
pub static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "");

pub static PROGRESS_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template(
    "[{elapsed_precise}] {bar:30.cyan/blue} {pos:>4}/{len:4} {wide_msg}",
    ).unwrap()
});

// ///////////// //
// CLI interface //
// ///////////// //

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

    /// Update to the latest version of the currently used channel.
    Update(UpdateArgs),
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// The name(s) of the version (comma separated).
    #[clap(value_delimiter = ',', num_args = 1..)]
    pub names: Vec<String>,

    /// The display name of the version.
    #[arg(short, long)]
    pub display_version: Option<String>,

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

    /// The endpoint and bucket of the S3 (compatible) storage. Must currently use TLS with valid cert and files must be public.
    ///
    /// Example: https://my-example-storage.com/my-bucket
    #[arg(short, long)]
    pub s3_url: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// The name of the version.
    pub name: Option<String>,

    /// The path prefix to prepend to all download paths.
    #[arg(short('p'), long)]
    pub filestore_path_prefix: Option<String>,

    /// The directory to update.
    #[arg(short, long)]
    pub output_dir: Option<String>,

    /// The endpoint and bucket of the S3 (compatible) storage. Must currently use TLS with valid cert and files must be public.
    ///
    /// Example: https://my-example-storage.com/my-bucket
    #[arg(short, long)]
    pub s3_url: Option<String>,
}
