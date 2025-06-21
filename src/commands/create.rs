use std::{fs::File, io::BufReader};

use brotli::enc::BrotliEncoderParams;
use bytes::BufMut;
use console::{style, Emoji};
use indicatif::ProgressBar;
use walkdir::{DirEntry, WalkDir};

use crate::models::{
    definition_version::DefinitionVersion, file_definition::FileDefinition,
    version_definition::VersionDefinition,
};

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static HOURGLASS: Emoji<'_, '_> = Emoji("⌛  ", "");

pub fn run_create(input_dir: Option<&String>, output_file: Option<&String>) {
    let root_path = match input_dir {
        Some(input_dir_path) => input_dir_path,
        None => ".",
    };

    println!("{} {}Building file list...", style("[1/2]").bold().dim(), LOOKING_GLASS);

    let file_list: Vec<DirEntry> = WalkDir::new(root_path)
        .into_iter()
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().is_file())
        .collect();

    let mut version = VersionDefinition {
        version: DefinitionVersion::Version1,
        files: Vec::new(),
    };

    println!("{} {}Processing files...", style("[2/2]").bold().dim(), HOURGLASS);

    let bar = ProgressBar::new(file_list.len() as u64);
    for entry in file_list {
        let path = entry
            .path()
            .strip_prefix(root_path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let file = File::open(entry.path()).unwrap();
        let mut reader = BufReader::new(file);

        let mut buf: bytes::buf::Writer<Vec<u8>> = vec!().writer();

        let mut params = BrotliEncoderParams::default();
        params.quality = 8;

        brotli::BrotliCompress(&mut reader, &mut buf, &params).unwrap();

        version.files.push(FileDefinition {
            r_path: path,
            u_size: entry.metadata().unwrap().len() as u32,
            u_sha256: sha256::try_digest(entry.path()).unwrap(),
            c_size: buf.get_ref().len() as u32,
            c_sha256: sha256::digest(buf.get_ref()),
        });

        bar.inc(1);
    }
    bar.finish_and_clear();

    let yaml = serde_yml::to_string(&version).unwrap();
    println!("{}", yaml);
}
