use std::{fs::{self, File}, io::ErrorKind, path::{Path, PathBuf}};

use console::{style, Emoji};
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{file_storage::{s3::S3Client, FileStore}, models::{file_definition::FileDefinition, version_definition::VersionDefinition}};

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static HOURGLASS: Emoji<'_, '_> = Emoji("⌛ ", "");
static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "");

pub async fn run_switch(version_name: &str, output_dir: &str, storage_base_path: &str) {
    println!("{} {}Getting file list...", style("[1/2]").bold().dim(), LOOKING_GLASS);

    let stor_client = S3Client::new_from_env().await;
    let version_def = get_version(&stor_client, storage_base_path, version_name).await;

    let multi_progress = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:30.cyan/blue} {pos:>4}/{len:4} {wide_msg}",
    ).unwrap();

    let pb = multi_progress.add(ProgressBar::new(version_def.files.len() as u64));
    pb.set_style(sty.clone());

    println!("{} {}Processing {} files...", style("[2/2]").bold().dim(), HOURGLASS, version_def.files.len());
    let mut n_unchanged = 0; let mut n_changed = 0; let mut n_missing = 0;
    for file in version_def.files {
        pb.set_message(format!("Verifying {}", file.r_path));

        let full_path = Path::new(output_dir).join(&file.r_path);
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
                download_file(&file, full_path, &stor_client, &storage_base_path).await;
            }
            Some(existing_file) => {
                // Destination file exists, first check file size (which is cheap to check).
                if existing_file.len() != file.u_len as u64 {
                    // Size check indicates different file.
                    pb.set_message(format!("Updating {}", file.r_path));
                    n_changed = n_changed + 1;
                    download_file(&file, full_path, &stor_client, &storage_base_path).await;
                } else {
                    let sha256 = sha256::try_digest(&full_path).unwrap();
                    if sha256 != file.u_sha256 {
                        // Contents check indicates different file.
                        pb.set_message(format!("Updating {}", file.r_path));
                        n_changed = n_changed + 1;
                        download_file(&file, full_path, &stor_client, &storage_base_path).await;
                    } else {
                        n_unchanged = n_unchanged + 1;
                    }
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    println!("\n{}Successfully finished with {} unchanged, {} changed and {} missing files.", CHECKMARK, n_unchanged, n_changed, n_missing);
}

async fn get_version(stor_client: &S3Client, storage_base_path: &str, version_name: &str) -> VersionDefinition {
    let version_storage_path = Path::new(storage_base_path).join("versions").join(version_name);
    let fetched_version_file = stor_client.get_file(version_storage_path.as_path()).await.unwrap().unwrap();
    let fetched_version_file_chunks = fetched_version_file.stream.collect::<Vec<bytes::Bytes>>().await;
    let fetched_version_file_bytes = fetched_version_file_chunks.iter().flat_map(|b| b.as_ref()).cloned().collect::<Vec<u8>>();

    let version_yaml = String::from_utf8_lossy(&fetched_version_file_bytes);
    serde_yml::from_str(&version_yaml).unwrap()
}

async fn download_file(file: &FileDefinition, full_path: PathBuf, storage_client: &S3Client, upload_base_path: &str) {
    use futures::StreamExt;
    use std::io::Cursor;

    let download_path = Path::new(upload_base_path).join("files").join(&file.u_sha256);

    let file = storage_client.get_file(download_path.as_path()).await.unwrap().unwrap();

    // Collect the stream into a single buffer
    let bytes_vec = file.stream.collect::<Vec<bytes::Bytes>>().await;
    let data = bytes_vec.iter().flat_map(|b| b.as_ref()).cloned().collect::<Vec<u8>>();

    let mut reader = Cursor::new(data);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut local_file = File::create(full_path).unwrap();
    brotli::BrotliDecompress(&mut reader, &mut local_file).unwrap();
}
