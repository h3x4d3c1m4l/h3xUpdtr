
use object_store::{aws::{AmazonS3, AmazonS3Builder}, ObjectStore, PutPayload};
use snafu::ResultExt;

use crate::file_storage::{CreateFileSnafu, FileStore, FileStoreError};

struct S3Client {
    s3_client: AmazonS3,
}

impl S3Client {
    pub async fn new_from_env(bucket_name: String) -> S3Client {
        let store: AmazonS3 = AmazonS3Builder::from_env().with_bucket_name(bucket_name).build().unwrap();

        S3Client { s3_client: store }
    }
}

impl FileStore for S3Client {
    async fn create_file<T: std::io::Read>(self, relative_path: String, data_stream: T) -> Result<(), FileStoreError> {
        let x = object_store::path::Path::parse(relative_path).unwrap();
        let bytes: Vec<u8> = data_stream.bytes().collect::<Result<Vec<u8>, std::io::Error>>().unwrap();
        let payload = PutPayload::from(bytes);
        self.s3_client.put(&x, payload).await.unwrap();

        Ok(())
    }
}
