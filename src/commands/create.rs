use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use brotli::enc::BrotliEncoderParams;
use bytes::BufMut;
use console::{style, Emoji};
use indicatif::ProgressBar;
use walkdir::{DirEntry, WalkDir};

use crate::{file_storage::{s3::S3Client, FileStore}, models::{
    definition_version::DefinitionVersion, file_definition::FileDefinition,
    version_definition::VersionDefinition,
}};

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static HOURGLASS: Emoji<'_, '_> = Emoji("⌛  ", "");

pub async fn run_create(version_name: &str, input_dir: &str, upload_base_path: &str) {
    println!("{} {}Building file list...", style("[1/2]").bold().dim(), LOOKING_GLASS);

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

    println!("{} {}Processing {} files...", style("[2/2]").bold().dim(), HOURGLASS, file_list.len());

    let bar = ProgressBar::new(file_list.len() as u64);
    for entry in file_list {
        let rel_file_path = entry
            .path()
            .strip_prefix(input_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let file = File::open(entry.path()).unwrap();
        let mut reader = BufReader::new(file);

        let mut buf: bytes::buf::Writer<Vec<u8>> = Vec::new().writer();

        let mut params = BrotliEncoderParams::default();
        params.quality = 8;

        brotli::BrotliCompress(&mut reader, &mut buf, &params).unwrap();

        let compressed = buf.into_inner();

        let uncompressed_sha256 = sha256::try_digest(entry.path()).unwrap();
        let compressed_sha256 = sha256::digest(&compressed);
        version.files.push(FileDefinition {
            r_path: rel_file_path.clone(),
            u_size: entry.metadata().unwrap().len() as u32,
            u_sha256: uncompressed_sha256.clone(),
            c_algo: "brotli".to_owned(),
            c_size: compressed.len() as u32,
            c_sha256: compressed_sha256.clone(),
        });

        let remote_path = Path::new(upload_base_path).join("files").join(uncompressed_sha256.clone());
        storage_client.upload_file(remote_path.as_path(), compressed.as_slice(), HashMap::from([
            ("c_algo", "brotli"),
            ("c_sha256", &compressed_sha256),
        ])).await.unwrap();

        bar.inc(1);
    }
    bar.finish_and_clear();

    let yaml_bytes = serde_yml::to_string(&version).unwrap().into_bytes();
    let remote_path = Path::new(upload_base_path).join("versions").join(version_name);
    let mut yaml_cursor = std::io::Cursor::new(yaml_bytes);
    storage_client.upload_file(remote_path.as_path(), &mut yaml_cursor, HashMap::new()).await.unwrap();
}
