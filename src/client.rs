use std::env;

use aws_config::SdkConfig as AwsConfig;
use aws_sdk_s3::{presigning::PresigningConfig, Client as S3Client};

/// S3 client wrapper to expose semantic upload operations.
#[derive(Debug, Clone)]
pub struct Client {
    s3: S3Client,
    bucket_name: String,
}

impl Client {
    /// Construct S3 client wrapper.
    pub fn new(config: &AwsConfig) -> Client {
        Client {
            s3: S3Client::new(config),
            bucket_name: env::var("AWS_S3_BUCKET_NAME").unwrap(),
        }
    }

    pub async fn get_presigned_url(
        &self,
        object: &str,
        expires_in: u64,
    ) -> Result<String, S3ExampleError> {
        let expires_in: std::time::Duration = std::time::Duration::from_secs(expires_in);
        let expires_in: aws_sdk_s3::presigning::PresigningConfig =
            PresigningConfig::expires_in(expires_in).map_err(|err| {
                S3ExampleError::new(format!(
                    "Failed to convert expiration to PresigningConfig: {err:?}"
                ))
            })?;

        let presigned_request = self
            .s3
            .get_object()
            .bucket(&self.bucket_name)
            .key(object)
            .presigned(expires_in)
            .await?;

        println!("Object URI: {}", presigned_request.uri());

        Ok(presigned_request.uri().into())
    }

    pub async fn get_upload_presigned_url(
        &self,
        object: &str,
        expires_in: u64,
    ) -> Result<String, S3ExampleError> {
        let expires_in: std::time::Duration = std::time::Duration::from_secs(expires_in);
        let expires_in: aws_sdk_s3::presigning::PresigningConfig =
            PresigningConfig::expires_in(expires_in).map_err(|err| {
                S3ExampleError::new(format!(
                    "Failed to convert expiration to PresigningConfig: {err:?}"
                ))
            })?;
        let presigned_request = self
            .s3
            .put_object()
            .bucket(&self.bucket_name)
            .key(object)
            .presigned(expires_in)
            .await?;

        Ok(presigned_request.uri().into())
    }

    pub async fn delete_object(&self, object: &str) -> Result<(), S3ExampleError> {
        self.s3
            .delete_object()
            .bucket(&self.bucket_name)
            .key(object)
            .send()
            .await?;

        Ok(())
    }
}

/// S3ExampleError provides a From<T: ProvideErrorMetadata> impl to extract
/// client-specific error details. This serves as a consistent backup to handling
/// specific service errors, depending on what is needed by the scenario.
/// It is used throughout the code examples for the AWS SDK for Rust.
#[derive(Debug)]
pub struct S3ExampleError(String);
impl S3ExampleError {
    pub fn new(value: impl Into<String>) -> Self {
        S3ExampleError(value.into())
    }
}

impl<T: aws_sdk_s3::error::ProvideErrorMetadata> From<T> for S3ExampleError {
    fn from(value: T) -> Self {
        S3ExampleError(format!(
            "{}: {}",
            value
                .code()
                .map(String::from)
                .unwrap_or("unknown code".into()),
            value
                .message()
                .map(String::from)
                .unwrap_or("missing reason".into()),
        ))
    }
}

impl std::error::Error for S3ExampleError {}

impl std::fmt::Display for S3ExampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
