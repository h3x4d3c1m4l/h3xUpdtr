
use std::{borrow::Cow, collections::HashMap, path::{self, Path}};

use object_store::{aws::{AmazonS3, AmazonS3Builder}, GetOptions, ObjectStore, PutOptions, PutPayload};
use url::Url;

use crate::file_storage::{self, FileStore, FileStoreError};

pub struct S3Client {
    s3_client: AmazonS3,
}

impl S3Client {
    pub fn new_from_env() -> S3Client {
        let store: AmazonS3 = AmazonS3Builder::from_env().build().unwrap();

        S3Client { s3_client: store }
    }

    pub fn new_from_url(url: &str) -> S3Client {
        let mut parsed_url = Url::parse(url).unwrap();
        let bucket_opt = parsed_url.path_segments().into_iter().flatten().next().map(|s| s.to_string());
        let bucket = bucket_opt.as_deref().unwrap();

        parsed_url.set_path("");

        let store: AmazonS3 = AmazonS3Builder::new()
            .with_endpoint(parsed_url.as_str())
            .with_bucket_name(bucket)
            .with_skip_signature(true)
            .build()
            .unwrap();

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

    async fn get_file_info(&self, relative_path: &Path) -> Result<Option<file_storage::RemoteFileInfo>, FileStoreError> {
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
            Err(_) => panic!("Handle me"),
        }
    }

    async fn get_file(&self, relative_path: &Path) -> Result<Option<file_storage::RemoteFile>, FileStoreError> {
        let unix_path = relative_path.to_str().unwrap().replace(path::MAIN_SEPARATOR_STR, "/");
        let obj_stor_path = object_store::path::Path::from(unix_path);

        let options = GetOptions::default();
        let result = self.s3_client.get_opts(&obj_stor_path, options).await;

        match result {
            Ok(info) => {
                Ok(Some(file_storage::RemoteFile {
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
                    stream: Box::pin(futures::StreamExt::map(info.into_stream(), |res| res.unwrap())),
                }))
            }
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(error) => panic!("{}", error),
        }
    }
}
