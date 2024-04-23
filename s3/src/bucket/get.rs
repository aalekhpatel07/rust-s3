use crate::bucket::{Bucket, Request};
use crate::command::Command;
use crate::error::S3Error;
use crate::request::RequestImpl;
use crate::request::ResponseData;

use crate::request::{AsyncWrite, ResponseDataStream};

impl Bucket {
    /// Gets file from an S3 path.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let response_data = bucket.get_object("/test.file").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object<S: AsRef<str>>(&self, path: S) -> Result<ResponseData, S3Error> {
        let command = Command::GetObject;
        let request = RequestImpl::new(self, path.as_ref(), command)?;
        request.response_data(false).await
    }

    /// Gets torrent from an S3 path.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let response_data = bucket.get_object_torrent("/test.file").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object_torrent<S: AsRef<str>>(
        &self,
        path: S,
    ) -> Result<ResponseData, S3Error> {
        let command = Command::GetObjectTorrent;
        let request = RequestImpl::new(self, path.as_ref(), command)?;
        request.response_data(false).await
    }

    /// Gets specified inclusive byte range of file from an S3 path.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    ///
    /// let response_data = bucket.get_object_range("/test.file", 0, Some(31)).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object_range<S: AsRef<str>>(
        &self,
        path: S,
        start: u64,
        end: Option<u64>,
    ) -> Result<ResponseData, S3Error> {
        if let Some(end) = end {
            assert!(start < end);
        }

        let command = Command::GetObjectRange { start, end };
        let request = RequestImpl::new(self, path.as_ref(), command)?;
        request.response_data(false).await
    }

    /// Stream range of bytes from S3 path to a local file, generic over T: Write.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    /// use std::fs::File;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    /// let mut output_file = File::create("output_file").expect("Unable to create file");
    /// let mut async_output_file = tokio::fs::File::create("async_output_file").await.expect("Unable to create file");
    ///
    /// let start = 0;
    /// let end = Some(1024);
    ///
    /// let status_code = bucket.get_object_range_to_writer("/test.file", start, end, &mut async_output_file).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object_range_to_writer<T: AsyncWrite + Send + Unpin, S: AsRef<str>>(
        &self,
        path: S,
        start: u64,
        end: Option<u64>,
        writer: &mut T,
    ) -> Result<u16, S3Error> {
        if let Some(end) = end {
            assert!(start < end);
        }

        let command = Command::GetObjectRange { start, end };
        let request = RequestImpl::new(self, path.as_ref(), command)?;
        request.response_data_to_writer(writer).await
    }

    /// Stream file from S3 path to a local file, generic over T: Write.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    /// use std::fs::File;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    /// let mut output_file = File::create("output_file").expect("Unable to create file");
    /// let mut async_output_file = tokio::fs::File::create("async_output_file").await.expect("Unable to create file");
    ///
    /// let status_code = bucket.get_object_to_writer("/test.file", &mut async_output_file).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object_to_writer<T: AsyncWrite + Send + Unpin, S: AsRef<str>>(
        &self,
        path: S,
        writer: &mut T,
    ) -> Result<u16, S3Error> {
        let command = Command::GetObject;
        let request = RequestImpl::new(self, path.as_ref(), command)?;
        request.response_data_to_writer(writer).await
    }

    /// Stream file from S3 path to a local file using an async stream.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    /// use tokio_stream::StreamExt;
    /// use tokio::io::AsyncWriteExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    /// let path = "path";
    ///
    /// let mut response_data_stream = bucket.get_object_stream(path).await?;
    ///
    /// let mut async_output_file = tokio::fs::File::create("async_output_file").await.expect("Unable to create file");
    ///
    /// while let Some(chunk) = response_data_stream.bytes().next().await {
    ///     // async_output_file.write_all(&chunk.unwrap()).await?;
    /// }
    ///
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_object_stream<S: AsRef<str>>(
        &self,
        path: S,
    ) -> Result<ResponseDataStream, S3Error> {
        let command = Command::GetObject;
        let request = RequestImpl::new(self, path.as_ref(), command)?;
        request.response_data_to_stream().await
    }
}
