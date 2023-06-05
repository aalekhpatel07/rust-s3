use crate::bucket::{
    error_from_response_data, Bucket, CompleteMultipartUploadData, InitiateMultipartUploadResponse,
    Part, Read, Request, CHUNK_SIZE,
};
use crate::command::{Command, Multipart};
use crate::error::S3Error;
use crate::request::{AsyncRead, RequestImpl, ResponseData};

use crate::bucket::{CorsConfiguration, PutStreamResponse};

#[cfg_attr(
    all(
        not(feature = "with-async-std"),
        feature = "with-tokio",
        feature = "blocking"
    ),
    block_on("tokio")
)]
#[cfg_attr(
    all(
        not(feature = "with-tokio"),
        feature = "with-async-std",
        feature = "blocking"
    ),
    block_on("async-std")
)]
impl Bucket {
    #[maybe_async::maybe_async]
    pub async fn put_bucket_cors(
        &self,
        cors_config: CorsConfiguration,
    ) -> Result<ResponseData, S3Error> {
        let command = Command::PutBucketCors {
            configuration: cors_config,
        };
        let request = RequestImpl::new(self, "?cors", command)?;
        request.response_data(false).await
    }

    /// Stream file from local path to s3, generic over T: Write.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    /// use std::fs::File;
    /// use std::io::Write;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    /// let path = "path";
    /// let test: Vec<u8> = (0..1000).map(|_| 42).collect();
    /// let mut file = File::create(path)?;
    /// // tokio open file
    /// let mut async_output_file = tokio::fs::File::create("async_output_file").await.expect("Unable to create file");
    /// file.write_all(&test)?;
    ///
    /// // Generic over std::io::Read
    /// #[cfg(feature = "with-tokio")]
    /// let status_code = bucket.put_object_stream(&mut async_output_file, "/path").await?;
    ///
    ///
    /// #[cfg(feature = "with-async-std")]
    /// let mut async_output_file = async_std::fs::File::create("async_output_file").await.expect("Unable to create file");
    ///
    /// // `sync` feature will produce an identical method
    /// #[cfg(feature = "sync")]
    /// // Generic over std::io::Read
    /// let status_code = bucket.put_object_stream(&mut path, "/path")?;
    ///
    /// // Blocking variant, generated with `blocking` feature in combination
    /// // with `tokio` or `async-std` features.
    /// #[cfg(feature = "blocking")]
    /// let status_code = bucket.put_object_stream_blocking(&mut path, "/path")?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::async_impl]
    pub async fn put_object_stream<R: AsyncRead + Unpin>(
        &self,
        reader: &mut R,
        s3_path: impl AsRef<str>,
    ) -> Result<PutStreamResponse, S3Error> {
        self._put_object_stream_with_content_type(
            reader,
            s3_path.as_ref(),
            "application/octet-stream",
        )
        .await
    }

    #[maybe_async::sync_impl]
    pub fn put_object_stream<R: Read>(
        &self,
        reader: &mut R,
        s3_path: impl AsRef<str>,
    ) -> Result<u16, S3Error> {
        self._put_object_stream_with_content_type(
            reader,
            s3_path.as_ref(),
            "application/octet-stream",
        )
    }

    /// Stream file from local path to s3, generic over T: Write with explicit content type.
    ///
    /// # Example:
    ///
    /// ```rust,no_run
    /// use s3::bucket::Bucket;
    /// use s3::creds::Credentials;
    /// use anyhow::Result;
    /// use std::fs::File;
    /// use std::io::Write;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    ///
    /// let bucket_name = "rust-s3-test";
    /// let region = "us-east-1".parse()?;
    /// let credentials = Credentials::default()?;
    /// let bucket = Bucket::new(bucket_name, region, credentials)?;
    /// let path = "path";
    /// let test: Vec<u8> = (0..1000).map(|_| 42).collect();
    /// let mut file = File::create(path)?;
    /// file.write_all(&test)?;
    ///
    /// #[cfg(feature = "with-tokio")]
    /// let mut async_output_file = tokio::fs::File::create("async_output_file").await.expect("Unable to create file");
    ///
    /// #[cfg(feature = "with-async-std")]
    /// let mut async_output_file = async_std::fs::File::create("async_output_file").await.expect("Unable to create file");
    ///
    /// // Async variant with `tokio` or `async-std` features
    /// // Generic over std::io::Read
    /// let status_code = bucket
    ///     .put_object_stream_with_content_type(&mut async_output_file, "/path", "application/octet-stream")
    ///     .await?;
    ///
    /// // `sync` feature will produce an identical method
    /// #[cfg(feature = "sync")]
    /// // Generic over std::io::Read
    /// let status_code = bucket
    ///     .put_object_stream_with_content_type(&mut path, "/path", "application/octet-stream")?;
    ///
    /// // Blocking variant, generated with `blocking` feature in combination
    /// // with `tokio` or `async-std` features.
    /// #[cfg(feature = "blocking")]
    /// let status_code = bucket
    ///     .put_object_stream_with_content_type_blocking(&mut path, "/path", "application/octet-stream")?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::async_impl]
    pub async fn put_object_stream_with_content_type<R: AsyncRead + Unpin>(
        &self,
        reader: &mut R,
        s3_path: impl AsRef<str>,
        content_type: impl AsRef<str>,
    ) -> Result<PutStreamResponse, S3Error> {
        self._put_object_stream_with_content_type(reader, s3_path.as_ref(), content_type.as_ref())
            .await
    }

    #[maybe_async::sync_impl]
    pub fn put_object_stream_with_content_type<R: Read>(
        &self,
        reader: &mut R,
        s3_path: impl AsRef<str>,
        content_type: impl AsRef<str>,
    ) -> Result<u16, S3Error> {
        self._put_object_stream_with_content_type(reader, s3_path.as_ref(), content_type.as_ref())
    }

    #[maybe_async::async_impl]
    async fn make_multipart_request(
        &self,
        path: &str,
        chunk: Vec<u8>,
        part_number: u32,
        upload_id: &str,
        content_type: &str,
    ) -> Result<ResponseData, S3Error> {
        let command = Command::PutObject {
            content: &chunk,
            multipart: Some(Multipart::new(part_number, upload_id)), // upload_id: &msg.upload_id,
            content_type,
        };
        let request = RequestImpl::new(self, path, command)?;
        request.response_data(true).await
    }

    #[maybe_async::async_impl]
    async fn _put_object_stream_with_content_type<R: AsyncRead + Unpin>(
        &self,
        reader: &mut R,
        s3_path: &str,
        content_type: &str,
    ) -> Result<PutStreamResponse, S3Error> {
        // If the file is smaller CHUNK_SIZE, just do a regular upload.
        // Otherwise perform a multi-part upload.
        let first_chunk = crate::utils::read_chunk_async(reader).await?;
        if first_chunk.len() < CHUNK_SIZE {
            let total_size = first_chunk.len();
            let response_data = self
                .put_object_with_content_type(s3_path, first_chunk.as_slice(), content_type)
                .await?;
            if response_data.status_code() >= 300 {
                return Err(error_from_response_data(response_data)?);
            }
            return Ok(PutStreamResponse::new(
                response_data.status_code(),
                total_size,
            ));
        }

        let msg = self
            .initiate_multipart_upload(s3_path, content_type)
            .await?;
        let path = msg.key;
        let upload_id = &msg.upload_id;

        let mut part_number: u32 = 0;
        let mut etags = Vec::new();

        // Collect request handles
        let mut handles = vec![];
        let mut total_size = 0;
        loop {
            let chunk = if part_number == 0 {
                first_chunk.clone()
            } else {
                crate::utils::read_chunk_async(reader).await?
            };
            total_size += chunk.len();

            let done = chunk.len() < CHUNK_SIZE;

            // Start chunk upload
            part_number += 1;
            handles.push(self.make_multipart_request(
                &path,
                chunk,
                part_number,
                upload_id,
                content_type,
            ));

            if done {
                break;
            }
        }

        // Wait for all chunks to finish (or fail)
        let responses = futures::future::join_all(handles).await;

        for response in responses {
            let response_data = response?;
            if !(200..300).contains(&response_data.status_code()) {
                // if chunk upload failed - abort the upload
                match self.abort_upload(&path, upload_id).await {
                    Ok(_) => {
                        return Err(error_from_response_data(response_data)?);
                    }
                    Err(error) => {
                        return Err(error);
                    }
                }
            }

            let etag = response_data.as_str()?;
            etags.push(etag.to_string());
        }

        // Finish the upload
        let inner_data = etags
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, x)| Part {
                etag: x,
                part_number: i as u32 + 1,
            })
            .collect::<Vec<Part>>();
        let response_data = self
            .complete_multipart_upload(&path, &msg.upload_id, inner_data)
            .await?;

        Ok(PutStreamResponse::new(
            response_data.status_code(),
            total_size,
        ))
    }

    #[maybe_async::sync_impl]
    fn _put_object_stream_with_content_type<R: Read>(
        &self,
        reader: &mut R,
        s3_path: &str,
        content_type: &str,
    ) -> Result<u16, S3Error> {
        let msg = self.initiate_multipart_upload(s3_path, content_type)?;
        let path = msg.key;
        let upload_id = &msg.upload_id;

        let mut part_number: u32 = 0;
        let mut etags = Vec::new();
        loop {
            let chunk = crate::utils::read_chunk(reader)?;

            if chunk.len() < CHUNK_SIZE {
                if part_number == 0 {
                    // Files is not big enough for multipart upload, going with regular put_object
                    self.abort_upload(&path, upload_id)?;

                    self.put_object(s3_path, chunk.as_slice())?;
                } else {
                    part_number += 1;
                    let part = self.put_multipart_chunk(
                        chunk,
                        &path,
                        part_number,
                        upload_id,
                        content_type,
                    )?;
                    etags.push(part.etag);
                    let inner_data = etags
                        .into_iter()
                        .enumerate()
                        .map(|(i, x)| Part {
                            etag: x,
                            part_number: i as u32 + 1,
                        })
                        .collect::<Vec<Part>>();
                    return Ok(self
                        .complete_multipart_upload(&path, upload_id, inner_data)?
                        .status_code());
                    // let response = std::str::from_utf8(data.as_slice())?;
                }
            } else {
                part_number += 1;
                let part =
                    self.put_multipart_chunk(chunk, &path, part_number, upload_id, content_type)?;
                etags.push(part.etag.to_string());
            }
        }
    }

    /// Initiate multipart upload to s3.
    #[maybe_async::async_impl]
    pub async fn initiate_multipart_upload(
        &self,
        s3_path: &str,
        content_type: &str,
    ) -> Result<InitiateMultipartUploadResponse, S3Error> {
        let command = Command::InitiateMultipartUpload { content_type };
        let request = RequestImpl::new(self, s3_path, command)?;
        let response_data = request.response_data(false).await?;
        if response_data.status_code() >= 300 {
            return Err(error_from_response_data(response_data)?);
        }

        let msg: InitiateMultipartUploadResponse =
            quick_xml::de::from_str(response_data.as_str()?)?;
        Ok(msg)
    }

    #[maybe_async::sync_impl]
    pub fn initiate_multipart_upload(
        &self,
        s3_path: &str,
        content_type: &str,
    ) -> Result<InitiateMultipartUploadResponse, S3Error> {
        let command = Command::InitiateMultipartUpload { content_type };
        let request = RequestImpl::new(self, s3_path, command)?;
        let response_data = request.response_data(false)?;
        if response_data.status_code() >= 300 {
            return Err(error_from_response_data(response_data)?);
        }

        let msg: InitiateMultipartUploadResponse =
            quick_xml::de::from_str(response_data.as_str()?)?;
        Ok(msg)
    }

    /// Upload a streamed multipart chunk to s3 using a previously initiated multipart upload
    #[maybe_async::async_impl]
    pub async fn put_multipart_stream<R: Read + Unpin>(
        &self,
        reader: &mut R,
        path: &str,
        part_number: u32,
        upload_id: &str,
        content_type: &str,
    ) -> Result<Part, S3Error> {
        let chunk = crate::utils::read_chunk(reader)?;
        self.put_multipart_chunk(chunk, path, part_number, upload_id, content_type)
            .await
    }

    #[maybe_async::sync_impl]
    pub async fn put_multipart_stream<R: Read + Unpin>(
        &self,
        reader: &mut R,
        path: &str,
        part_number: u32,
        upload_id: &str,
        content_type: &str,
    ) -> Result<Part, S3Error> {
        let chunk = crate::utils::read_chunk(reader)?;
        self.put_multipart_chunk(chunk, path, part_number, upload_id, content_type)
    }

    /// Upload a buffered multipart chunk to s3 using a previously initiated multipart upload
    #[maybe_async::async_impl]
    pub async fn put_multipart_chunk(
        &self,
        chunk: Vec<u8>,
        path: &str,
        part_number: u32,
        upload_id: &str,
        content_type: &str,
    ) -> Result<Part, S3Error> {
        let command = Command::PutObject {
            // part_number,
            content: &chunk,
            multipart: Some(Multipart::new(part_number, upload_id)), // upload_id: &msg.upload_id,
            content_type,
        };
        let request = RequestImpl::new(self, path, command)?;
        let response_data = request.response_data(true).await?;
        if !(200..300).contains(&response_data.status_code()) {
            // if chunk upload failed - abort the upload
            match self.abort_upload(path, upload_id).await {
                Ok(_) => {
                    return Err(error_from_response_data(response_data)?);
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
        let etag = response_data.as_str()?;
        Ok(Part {
            etag: etag.to_string(),
            part_number,
        })
    }

    #[maybe_async::sync_impl]
    pub fn put_multipart_chunk(
        &self,
        chunk: Vec<u8>,
        path: &str,
        part_number: u32,
        upload_id: &str,
        content_type: &str,
    ) -> Result<Part, S3Error> {
        let command = Command::PutObject {
            // part_number,
            content: &chunk,
            multipart: Some(Multipart::new(part_number, upload_id)), // upload_id: &msg.upload_id,
            content_type,
        };
        let request = RequestImpl::new(self, path, command)?;
        let response_data = request.response_data(true)?;
        if !(200..300).contains(&response_data.status_code()) {
            // if chunk upload failed - abort the upload
            match self.abort_upload(path, upload_id) {
                Ok(_) => {
                    return Err(error_from_response_data(response_data)?);
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
        let etag = response_data.as_str()?;
        Ok(Part {
            etag: etag.to_string(),
            part_number,
        })
    }

    /// Completes a previously initiated multipart upload, with optional final data chunks
    #[maybe_async::async_impl]
    pub async fn complete_multipart_upload(
        &self,
        path: &str,
        upload_id: &str,
        parts: Vec<Part>,
    ) -> Result<ResponseData, S3Error> {
        let data = CompleteMultipartUploadData { parts };
        let complete = Command::CompleteMultipartUpload { upload_id, data };
        let complete_request = RequestImpl::new(self, path, complete)?;
        complete_request.response_data(false).await
    }

    #[maybe_async::sync_impl]
    pub fn complete_multipart_upload(
        &self,
        path: &str,
        upload_id: &str,
        parts: Vec<Part>,
    ) -> Result<ResponseData, S3Error> {
        let data = CompleteMultipartUploadData { parts };
        let complete = Command::CompleteMultipartUpload { upload_id, data };
        let complete_request = RequestImpl::new(self, path, complete)?;
        complete_request.response_data(false)
    }

    /// Put into an S3 bucket, with explicit content-type.
    ///
    /// # Example:
    ///
    /// ```no_run
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
    /// let content = "I want to go to S3".as_bytes();
    ///
    /// // Async variant with `tokio` or `async-std` features
    /// let response_data = bucket.put_object_with_content_type("/test.file", content, "text/plain").await?;
    ///
    /// // `sync` feature will produce an identical method
    /// #[cfg(feature = "sync")]
    /// let response_data = bucket.put_object_with_content_type("/test.file", content, "text/plain")?;
    ///
    /// // Blocking variant, generated with `blocking` feature in combination
    /// // with `tokio` or `async-std` features.
    /// #[cfg(feature = "blocking")]
    /// let response_data = bucket.put_object_with_content_type_blocking("/test.file", content, "text/plain")?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn put_object_with_content_type<S: AsRef<str>>(
        &self,
        path: S,
        content: &[u8],
        content_type: &str,
    ) -> Result<ResponseData, S3Error> {
        let command = Command::PutObject {
            content,
            content_type,
            multipart: None,
        };
        let request = RequestImpl::new(self, path.as_ref(), command)?;
        request.response_data(true).await
    }

    /// Put into an S3 bucket.
    ///
    /// # Example:
    ///
    /// ```no_run
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
    /// let content = "I want to go to S3".as_bytes();
    ///
    /// // Async variant with `tokio` or `async-std` features
    /// let response_data = bucket.put_object("/test.file", content).await?;
    ///
    /// // `sync` feature will produce an identical method
    /// #[cfg(feature = "sync")]
    /// let response_data = bucket.put_object("/test.file", content)?;
    ///
    /// // Blocking variant, generated with `blocking` feature in combination
    /// // with `tokio` or `async-std` features.
    /// #[cfg(feature = "blocking")]
    /// let response_data = bucket.put_object_blocking("/test.file", content)?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn put_object<S: AsRef<str>>(
        &self,
        path: S,
        content: &[u8],
    ) -> Result<ResponseData, S3Error> {
        self.put_object_with_content_type(path, content, "application/octet-stream")
            .await
    }

    /// Tag an S3 object.
    ///
    /// # Example:
    ///
    /// ```no_run
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
    /// // Async variant with `tokio` or `async-std` features
    /// let response_data = bucket.put_object_tagging("/test.file", &[("Tag1", "Value1"), ("Tag2", "Value2")]).await?;
    ///
    /// // `sync` feature will produce an identical method
    /// #[cfg(feature = "sync")]
    /// let response_data = bucket.put_object_tagging("/test.file", &[("Tag1", "Value1"), ("Tag2", "Value2")])?;
    ///
    /// // Blocking variant, generated with `blocking` feature in combination
    /// // with `tokio` or `async-std` features.
    /// #[cfg(feature = "blocking")]
    /// let response_data = bucket.put_object_tagging_blocking("/test.file", &[("Tag1", "Value1"), ("Tag2", "Value2")])?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn put_object_tagging<S: AsRef<str>>(
        &self,
        path: &str,
        tags: &[(S, S)],
    ) -> Result<ResponseData, S3Error> {
        let content = self._tags_xml(tags);
        let command = Command::PutObjectTagging { tags: &content };
        let request = RequestImpl::new(self, path, command)?;
        request.response_data(false).await
    }

    /// Abort a running multipart upload.
    ///
    /// # Example:
    ///
    /// ```no_run
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
    /// // Async variant with `tokio` or `async-std` features
    /// let results = bucket.abort_upload("/some/file.txt", "ZDFjM2I0YmEtMzU3ZC00OTQ1LTlkNGUtMTgxZThjYzIwNjA2").await?;
    ///
    /// // `sync` feature will produce an identical method
    /// #[cfg(feature = "sync")]
    /// let results = bucket.abort_upload("/some/file.txt", "ZDFjM2I0YmEtMzU3ZC00OTQ1LTlkNGUtMTgxZThjYzIwNjA2")?;
    ///
    /// // Blocking variant, generated with `blocking` feature in combination
    /// // with `tokio` or `async-std` features.
    /// #[cfg(feature = "blocking")]
    /// let results = bucket.abort_upload_blocking("/some/file.txt", "ZDFjM2I0YmEtMzU3ZC00OTQ1LTlkNGUtMTgxZThjYzIwNjA2")?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn abort_upload(&self, key: &str, upload_id: &str) -> Result<(), S3Error> {
        let abort = Command::AbortMultipartUpload { upload_id };
        let abort_request = RequestImpl::new(self, key, abort)?;
        let response_data = abort_request.response_data(false).await?;

        if (200..300).contains(&response_data.status_code()) {
            Ok(())
        } else {
            let utf8_content = String::from_utf8(response_data.as_slice().to_vec())?;
            Err(S3Error::HttpFailWithBody(
                response_data.status_code(),
                utf8_content,
            ))
        }
    }
}
