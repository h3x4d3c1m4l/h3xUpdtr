
use std::{collections::HashMap, path::{self, Path}};

use object_store::{aws::{AmazonS3, AmazonS3Builder}, ObjectStore, PutOptions, PutPayload};

use crate::file_storage::{FileStore, FileStoreError};

pub struct S3Client {
    s3_client: AmazonS3,
}

impl S3Client {
    pub async fn new_from_env() -> S3Client {
        let store: AmazonS3 = AmazonS3Builder::from_env().build().unwrap();

        S3Client { s3_client: store }
    }
}

impl FileStore for S3Client {
    async fn upload_file<T: std::io::Read>(&self, relative_path: &Path, data_stream: T, metadata: HashMap<&str, &str>) -> Result<(), FileStoreError> {
        let unix_path = relative_path.to_str().unwrap().replace(path::MAIN_SEPARATOR_STR, "/");
        let obj_stor_path = object_store::path::Path::from(unix_path);

        let bytes: Vec<u8> = data_stream.bytes().collect::<Result<Vec<u8>, std::io::Error>>().unwrap();
        let payload = PutPayload::from(bytes);

        let mut options = PutOptions::default();
        for (key, value) in metadata.iter() {
            options.tags.push(key, value);
        }
        self.s3_client.put_opts(&obj_stor_path, payload, options).await.unwrap();

        Ok(())
    }
}
