use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use brotli::enc::BrotliEncoderParams;
use bytes::BufMut;
use console::style;
use indicatif::ProgressBar;
use snafu::{OptionExt, ResultExt, Whatever};
use walkdir::{DirEntry, WalkDir};

use crate::{cli, file_storage::FileStore, models::version_definition::*};

pub async fn run_create(display_version: Option<String>, version_names: &Vec<String>, input_dir: &str, storage_base_path: &str, storage_client: impl FileStore) -> Result<(), Whatever> {
    println!("{} {}Building file list...", style("[1/3]").bold().dim(), cli::LOOKING_GLASS);

    let file_list: Vec<DirEntry> = WalkDir::new(input_dir)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .with_whatever_context(|_| format!("Failed to walk directory {}", input_dir))?
        .into_iter()
        .filter(|e| e.file_type().is_file())
        .collect();

    let mut version = VersionDefinition {
        version: DefinitionVersion::Version1,
        display_version: display_version,
        files: Vec::new(),
    };

    let pb = ProgressBar::new(file_list.len() as u64);
    pb.set_style(cli::PROGRESS_STYLE.clone());

    println!("{} {}Processing {} files...", style("[2/3]").bold().dim(), cli::HOURGLASS, file_list.len());
    let mut n_already_existing = 0; let mut n_uploaded = 0;
    for entry in file_list {
        let rel_file_path = entry
            .path()
            .strip_prefix(input_dir)
            .with_whatever_context(|_| format!("Could not strip path prefix {:#?} of {:#?}", input_dir, entry.path()))?
            .to_str()
            .with_whatever_context(|| format!("Could not convert path {:#?} stripped of {:#?} to string", input_dir, entry.path()))?
            .to_string();

        pb.set_message(format!("Hashing {}", rel_file_path));

        let uncompressed_sha256 = sha256::try_digest(entry.path()).with_whatever_context(|_| format!("Could not get SHA256 hash for file {:#?}", entry.path()))?;
        let remote_path = Path::new(storage_base_path).join("files").join(uncompressed_sha256.clone());

        pb.set_message(format!("Checking existing {}", rel_file_path));

        // Check if file exists already.
        let existing_file_info = storage_client.get_file_info(&remote_path).await.with_whatever_context(|_| format!("Could not get file info for {:#?}", remote_path))?;

        let file = match existing_file_info {
            Some(file_info) => {
                // File already exists on remote storage.
                n_already_existing = n_already_existing + 1;
                FileDefinition {
                    r_path: rel_file_path.clone(),
                    u_len: entry.metadata().with_whatever_context(|_| format!("Could not get metadata of file {:#?}", remote_path))?.len() as u32,
                    u_sha256: uncompressed_sha256.clone(),
                    c_algo: "brotli".to_owned(),
                    c_len: file_info.c_len,
                    c_sha256: file_info.metadata.get("c_sha256").with_whatever_context(|| format!("c_sha256 missing of file {:#?}", remote_path))?.to_owned(),
                }
            },
            None => {
                // File needs to be uploaded.
                pb.set_message(format!("Compressing {}", rel_file_path));
                n_uploaded = n_uploaded + 1;
                let file = File::open(entry.path()).with_whatever_context(|_| format!("Could not open file {:#?}", entry.path()))?;
                let mut reader = BufReader::new(file);
                let mut buf: bytes::buf::Writer<Vec<u8>> = Vec::new().writer();

                // Compress using Brotli.
                let mut params = BrotliEncoderParams::default();
                params.quality = 8;
                brotli::BrotliCompress(&mut reader, &mut buf, &params).with_whatever_context(|_| format!("Could not compress file {:#?}", entry.path()))?;
                let compressed = buf.into_inner();
                let compressed_sha256 = sha256::digest(&compressed);

                pb.set_message(format!("Uploading {}", rel_file_path));
                storage_client.upload_file(remote_path.as_path(), compressed.as_slice(), HashMap::from([
                    ("c_algo", "brotli"),
                    ("c_sha256", &compressed_sha256),
                ])).await.with_whatever_context(|_| format!("Could not upload file {:#?}", remote_path))?;

                FileDefinition {
                    r_path: rel_file_path.clone(),
                    u_len: entry.metadata().with_whatever_context(|_| format!("Could not get metadata of file {:#?}", remote_path))?.len() as u32,
                    u_sha256: uncompressed_sha256.clone(),
                    c_algo: "brotli".to_owned(),
                    c_len: compressed.len() as u32,
                    c_sha256: compressed_sha256.clone(),
                }
            },
        };

        version.files.push(file);
        pb.inc(1);
    }

    pb.finish_and_clear();

    println!("{} {}Uploading version definition(s)...", style("[3/3]").bold().dim(), cli::CHECKLIST);
    let yaml_bytes = serde_yml::to_string(&version).with_whatever_context(|_| "Could not convert version info to YAML")?.into_bytes();
    for version_name in version_names {
        let remote_path = Path::new(storage_base_path).join("versions").join(version_name);
        let mut yaml_cursor = std::io::Cursor::new(&yaml_bytes);
        storage_client.upload_file(remote_path.as_path(), &mut yaml_cursor, HashMap::new()).await.with_whatever_context(|_| format!("Could not upload file {:#?}", remote_path.as_path()))?;
    }

    println!("\n{}Successfully finished with {} already existing and {} uploaded files.", cli::CHECKMARK, n_already_existing, n_uploaded);

    Ok(())
}
