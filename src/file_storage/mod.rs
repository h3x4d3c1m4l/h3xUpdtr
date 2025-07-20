use std::{collections::HashMap, io::Read, path::Path};

use snafu::Whatever;
use futures::stream::BoxStream;
use bytes::Bytes;

pub mod s3;

pub trait FileStore {
    async fn upload_file<T: Read>(&self, relative_path: &Path, data_stream: T, metadata: HashMap<&str, &str>) -> Result<(), Whatever>;
    async fn get_file_info(&self, relative_path: &Path) -> Result<Option<RemoteFileInfo>, Whatever>;
    async fn get_file(&self, relative_path: &Path) -> Result<Option<RemoteFile>, Whatever>;
}

#[derive(Debug)]
pub struct RemoteFileInfo {
    pub c_len: u32,
    pub metadata: HashMap<String, String>,
}

pub struct RemoteFile {
    pub c_len: u32,
    pub metadata: HashMap<String, String>,
    pub stream: BoxStream<'static, Bytes>,
}
