use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub folders: HashMap<String, FolderConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct FolderConfig {
    pub last_installed_version: String,
    pub s3_url: String,
    pub storage_path_prefix: Option<String>,
}
