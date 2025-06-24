
use std::path::{self, Path};

use object_store::{aws::{self, AmazonS3, AmazonS3Builder}, ObjectStore, PutPayload};

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
    async fn upload_file<T: std::io::Read>(&self, relative_path: &Path, data_stream: T) -> Result<(), FileStoreError> {
        let unix_path = relative_path.to_str().unwrap().replace(path::MAIN_SEPARATOR_STR, "/");
        let obj_stor_path = object_store::path::Path::from(unix_path);

        let bytes: Vec<u8> = data_stream.bytes().collect::<Result<Vec<u8>, std::io::Error>>().unwrap();
        let payload = PutPayload::from(bytes);
        self.s3_client.put(&obj_stor_path, payload).await.unwrap();

        Ok(())
    }
}
