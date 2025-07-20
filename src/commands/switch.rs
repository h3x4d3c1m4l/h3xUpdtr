use std::{fs::{self, File}, io::{Cursor, ErrorKind}, path::{Path, PathBuf}};

use console::style;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar};
use snafu::{OptionExt, ResultExt, Whatever};

use crate::{cli, file_storage::FileStore, models::version_definition::*};

pub async fn run_switch(version_name: String, output_dir: String, storage_base_path: String, storage_client: impl FileStore) -> Result<(), Whatever> {
    println!("{} {}Getting file list...", style("[1/2]").bold().dim(), cli::LOOKING_GLASS);

    let version_def = get_version(&storage_client, &storage_base_path, &version_name).await.with_whatever_context(|_| format!("Could not get info of version {version_name}"))?;

    let multi_progress = MultiProgress::new();

    let pb = multi_progress.add(ProgressBar::new(version_def.files.len() as u64));
    pb.set_style(cli::PROGRESS_STYLE.clone());

    println!("{} {}Processing {} files...", style("[2/2]").bold().dim(), cli::HOURGLASS, version_def.files.len());
    let mut n_unchanged = 0; let mut n_changed = 0; let mut n_missing = 0;
    for file in version_def.files {
        pb.set_message(format!("Verifying {}", file.r_path));

        let full_path = Path::new(&output_dir).join(&file.r_path);
        let existing_file = fs::metadata(&full_path);
        let existing_file = match existing_file {
            Ok(f) if !f.is_file() => panic!("Expected a file"),
            Ok(f) => Some(f),
            Err(e) if e.kind() == ErrorKind::NotFound => None,
            _ => panic!("Unknown error"),
        };

        match existing_file {
            None => {
                pb.set_message(format!("Restoring missing {}", file.r_path));
                n_missing = n_missing + 1;
                download_file(&file, full_path, &storage_client, &storage_base_path).await?;
            }
            Some(existing_file) => {
                // Destination file exists, first check file size (which is cheap to check).
                if existing_file.len() != file.u_len as u64 {
                    // Size check indicates different file.
                    pb.set_message(format!("Updating {}", file.r_path));
                    n_changed = n_changed + 1;
                    download_file(&file, full_path, &storage_client, &storage_base_path).await?;
                } else {
                    let sha256 = sha256::try_digest(&full_path).with_whatever_context(|_| format!("Could not get SHA256 hash for file {:#?}", full_path))?;
                    if sha256 != file.u_sha256 {
                        // Contents check indicates different file.
                        pb.set_message(format!("Updating {}", file.r_path));
                        n_changed = n_changed + 1;
                        download_file(&file, full_path, &storage_client, &storage_base_path).await?;
                    } else {
                        n_unchanged = n_unchanged + 1;
                    }
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    println!("\n{}Successfully finished with {} unchanged, {} changed and {} missing files.", cli::CHECKMARK, n_unchanged, n_changed, n_missing);

    Ok(())
}

async fn get_version(stor_client: &impl FileStore, storage_base_path: &str, version_name: &str) -> Result<VersionDefinition, Whatever> {
    let version_storage_path = Path::new(storage_base_path).join("versions").join(version_name);
    let fetched_version_file = stor_client.get_file(version_storage_path.as_path()).await.with_whatever_context(|_| format!("Could not get file info for {:#?}", version_storage_path))?.with_whatever_context(|| format!("Could not find file {:#?}", version_storage_path))?;
    let fetched_version_file_chunks = fetched_version_file.stream.collect::<Vec<bytes::Bytes>>().await;
    let fetched_version_file_bytes = fetched_version_file_chunks.iter().flat_map(|b| b.as_ref()).cloned().collect::<Vec<u8>>();

    let version_yaml = String::from_utf8_lossy(&fetched_version_file_bytes);
    serde_yml::from_str(&version_yaml).with_whatever_context(|_| "Could not parse version YAML")
}

async fn download_file(file: &FileDefinition, full_path: PathBuf, storage_client: &impl FileStore, upload_base_path: &str) -> Result<(), Whatever> {
    let download_path = Path::new(upload_base_path).join("files").join(&file.u_sha256);

    let file = storage_client.get_file(download_path.as_path()).await
        .with_whatever_context(|_| format!("Could not get file info for {:#?}", download_path))?.with_whatever_context(|| format!("Could not find file {:#?}", download_path))?;

    // Collect the stream into a single buffer
    let bytes_vec = file.stream.collect::<Vec<bytes::Bytes>>().await;
    let data = bytes_vec.iter().flat_map(|b| b.as_ref()).cloned().collect::<Vec<u8>>();

    let mut reader = Cursor::new(data);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).with_whatever_context(|_| format!("Could not create directory {:#?}", parent))?;
    }
    let mut local_file = File::create(&full_path).with_whatever_context(|_| format!("Could not create file {:#?}", full_path))?;
    brotli::BrotliDecompress(&mut reader, &mut local_file).with_whatever_context(|_| format!("Could not decompress to {:#?}", full_path))?;

    Ok(())
}
