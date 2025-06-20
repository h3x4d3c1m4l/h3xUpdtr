use walkdir::WalkDir;

use crate::models::{
    definition_version::DefinitionVersion, file_definition::FileDefinition,
    version_definition::VersionDefinition,
};

pub fn run_create(input_dir: Option<&String>, output_file: Option<&String>) {
    let root_path = match input_dir {
        Some(input_dir_path) => input_dir_path,
        None => ".",
    };

    let mut version = VersionDefinition {
        version: DefinitionVersion::Version1,
        files: Vec::new(),
    };

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter(|e| e.as_ref().unwrap().file_type().is_file())
    {
        let entry = entry.unwrap();
        let path = entry
            .path()
            .strip_prefix(root_path)
            .unwrap()
            .to_str()
            .unwrap_or("")
            .to_string();

        version.files.push(FileDefinition {
            r_path: path,
            u_sha256: sha256::try_digest(entry.path()).unwrap(),
            c_sha256: "FIXME".to_owned(),
            u_size: entry.metadata().unwrap().len(),
        });
    }

    let yaml = serde_yml::to_string(&version).unwrap();
    println!("{}", yaml);
}
