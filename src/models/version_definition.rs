use serde::Serialize;

use crate::models::{definition_version::DefinitionVersion, file_definition::FileDefinition};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDefinition {
    pub version: DefinitionVersion,
    pub files: Vec<FileDefinition>,
}
