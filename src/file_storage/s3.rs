
use object_store::{aws::{AmazonS3, AmazonS3Builder}, ObjectStore, PutPayload};

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
    async fn upload_file<T: std::io::Read>(&self, relative_path: &str, data_stream: T) -> Result<(), FileStoreError> {
        let path = object_store::path::Path::parse(relative_path).unwrap();
        let bytes: Vec<u8> = data_stream.bytes().collect::<Result<Vec<u8>, std::io::Error>>().unwrap();
        let payload = PutPayload::from(bytes);
        self.s3_client.put(&path, payload).await.unwrap();

        Ok(())
    }
}
