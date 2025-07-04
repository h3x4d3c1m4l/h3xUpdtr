use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use brotli::enc::BrotliEncoderParams;
use bytes::BufMut;
use console::style;
use indicatif::ProgressBar;
use walkdir::{DirEntry, WalkDir};

use crate::{cli, file_storage::{s3::S3Client, FileStore}, models::{
    definition_version::DefinitionVersion, file_definition::FileDefinition,
    version_definition::VersionDefinition,
}};

pub async fn run_create(version_names: &Vec<String>, input_dir: &str, storage_base_path: &str) {
    println!("{} {}Building file list...", style("[1/3]").bold().dim(), cli::LOOKING_GLASS);

    let file_list: Vec<DirEntry> = WalkDir::new(input_dir)
        .into_iter()
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().is_file())
        .collect();

    let mut version = VersionDefinition {
        version: DefinitionVersion::Version1,
        files: Vec::new(),
    };

    let storage_client = S3Client::new_from_env().await;

    let pb = ProgressBar::new(file_list.len() as u64);
    pb.set_style(cli::PROGRESS_STYLE.clone());

    println!("{} {}Processing {} files...", style("[2/3]").bold().dim(), cli::HOURGLASS, file_list.len());
    let mut n_already_existing = 0; let mut n_uploaded = 0;
    for entry in file_list {
        let rel_file_path = entry
            .path()
            .strip_prefix(input_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        pb.set_message(format!("Hashing {}", rel_file_path));

        let uncompressed_sha256 = sha256::try_digest(entry.path()).unwrap();
        let remote_path = Path::new(storage_base_path).join("files").join(uncompressed_sha256.clone());

        pb.set_message(format!("Checking existing {}", rel_file_path));

        // Check if file exists already.
        let existing_file_info = storage_client.get_file_info(&remote_path).await.unwrap();

        let file = match existing_file_info {
            Some(file_info) => {
                // File already exists on remote storage.
                n_already_existing = n_already_existing + 1;
                FileDefinition {
                    r_path: rel_file_path.clone(),
                    u_len: entry.metadata().unwrap().len() as u32,
                    u_sha256: uncompressed_sha256.clone(),
                    c_algo: "brotli".to_owned(),
                    c_len: file_info.c_len,
                    c_sha256: file_info.metadata.get("c_sha256").unwrap().to_owned(),
                }
            },
            None => {
                // File needs to be uploaded.
                pb.set_message(format!("Compressing {}", rel_file_path));
                n_uploaded = n_uploaded + 1;
                let file = File::open(entry.path()).unwrap();
                let mut reader = BufReader::new(file);
                let mut buf: bytes::buf::Writer<Vec<u8>> = Vec::new().writer();

                // Compress using Brotli.
                let mut params = BrotliEncoderParams::default();
                params.quality = 8;
                brotli::BrotliCompress(&mut reader, &mut buf, &params).unwrap();
                let compressed = buf.into_inner();
                let compressed_sha256 = sha256::digest(&compressed);

                pb.set_message(format!("Uploading {}", rel_file_path));
                storage_client.upload_file(remote_path.as_path(), compressed.as_slice(), HashMap::from([
                    ("c_algo", "brotli"),
                    ("c_sha256", &compressed_sha256),
                ])).await.unwrap();

                FileDefinition {
                    r_path: rel_file_path.clone(),
                    u_len: entry.metadata().unwrap().len() as u32,
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
    let yaml_bytes = serde_yml::to_string(&version).unwrap().into_bytes();
    for version_name in version_names {
        let remote_path = Path::new(storage_base_path).join("versions").join(version_name);
        let mut yaml_cursor = std::io::Cursor::new(&yaml_bytes);
        storage_client.upload_file(remote_path.as_path(), &mut yaml_cursor, HashMap::new()).await.unwrap();
    }

    println!("\n{}Successfully finished with {} already existing and {} uploaded files.", cli::CHECKMARK, n_already_existing, n_uploaded);
}
