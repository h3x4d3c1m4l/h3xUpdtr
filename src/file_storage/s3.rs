
use std::{borrow::Cow, collections::HashMap, path::{self, Path}};

use object_store::{aws::{AmazonS3, AmazonS3Builder}, GetOptions, ObjectStore, PutOptions, PutPayload};

use crate::file_storage::{self, FileStore, FileStoreError};

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
            options.attributes.insert(
                object_store::Attribute::Metadata(Cow::Owned((*key).to_string())),
                object_store::AttributeValue::from((*value).to_string())
            );
        }
        self.s3_client.put_opts(&obj_stor_path, payload, options).await.unwrap();

        Ok(())
    }

    async fn get_file_info(self, relative_path: &Path) -> Result<Option<file_storage::RemoteFileInfo>, FileStoreError> {
        let unix_path = relative_path.to_str().unwrap().replace(path::MAIN_SEPARATOR_STR, "/");
        let obj_stor_path = object_store::path::Path::from(unix_path);

        let mut options = GetOptions::default();
        options.head = true;
        let result = self.s3_client.get_opts(&obj_stor_path, options).await;

        match result {
            Ok(info) => {
                Ok(Some(file_storage::RemoteFileInfo {
                    c_len: info.meta.size as u32,
                    metadata: info
                        .attributes
                        .iter()
                        .filter_map(|(k, v)| {
                            if let object_store::Attribute::Metadata(key) = k {
                                Some((key.to_string(), v.to_string()))
                            } else {
                                None
                            }
                        })
                        .collect::<HashMap<String, String>>(),
                }))
            }
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => panic!("Handle me"),
        }
    }
}
