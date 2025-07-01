use std::path::Path;

use console::{style, Emoji};
use futures::StreamExt;

use crate::file_storage::{s3::S3Client, FileStore};

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static HOURGLASS: Emoji<'_, '_> = Emoji("⌛  ", "");

pub async fn run_switch(version_name: &str, output_dir: &str, upload_base_path: &str) {
    println!("{} {}Getting file list...", style("[1/2]").bold().dim(), LOOKING_GLASS);

    let storage_client = S3Client::new_from_env().await;

    let version_remote_path = Path::new(upload_base_path).join("versions").join(version_name);
    let version_file = storage_client.get_file(version_remote_path.as_path()).await.unwrap().unwrap();
    let version_file_bytes_chunks = version_file.stream.collect::<Vec<bytes::Bytes>>().await;
    let version_file_data = version_file_bytes_chunks.iter().flat_map(|b| b.as_ref()).cloned().collect::<Vec<u8>>();

    let version_file_str = String::from_utf8_lossy(&version_file_data);
    println!("{}", version_file_str);
}
